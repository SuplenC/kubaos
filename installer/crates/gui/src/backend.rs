//! Where probes and installs come from: the real `dotinstall` binary, or the
//! built-in demo. Both are blocking; the app calls them on worker threads.

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};

use dotinstall_core::probe::{DiskInfo, PartitionInfo, Region, WifiNetwork};
use dotinstall_core::{Event, InstallConfig, Manifest, Probe, Secrets, StepId, demo};

/// What an install reports, in order: events, then exactly one `Exited`.
#[derive(Debug, Clone)]
pub enum RunMsg {
    Event(Event),
    /// A line the engine wrote to stderr (human-readable log).
    Stderr(String),
    /// The run is over. `Err` carries a reason when the process died without
    /// sending a `failed` event (crash, missing binary, bad exit code).
    Exited(Result<(), String>),
}

pub trait Backend: Send + Sync {
    fn probe(&self) -> Result<Probe, String>;
    fn run(&self, config: &InstallConfig, secrets: &Secrets, emit: &mut dyn FnMut(RunMsg));
    fn reboot(&self) -> Result<(), String>;
    /// Shown in the corner so nobody mistakes the demo for a real install.
    fn is_demo(&self) -> bool;
}

/// Runs `dotinstall` (docs/INSTALLER.md, "CLI").
pub struct Process {
    pub bin: PathBuf,
}

impl Backend for Process {
    fn probe(&self) -> Result<Probe, String> {
        let out = Command::new(&self.bin)
            .args(["probe", "--json"])
            .stdin(Stdio::null())
            .output()
            .map_err(|e| format!("could not start {}: {e}", self.bin.display()))?;
        if !out.status.success() {
            return Err(format!(
                "dotinstall probe failed ({}): {}",
                out.status,
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        serde_json::from_slice(&out.stdout).map_err(|e| format!("bad probe output: {e}"))
    }

    fn run(&self, config: &InstallConfig, secrets: &Secrets, emit: &mut dyn FnMut(RunMsg)) {
        let result = self.run_inner(config, secrets, emit);
        emit(RunMsg::Exited(result));
    }

    fn reboot(&self) -> Result<(), String> {
        Command::new("systemctl")
            .arg("reboot")
            .status()
            .map_err(|e| e.to_string())
            .and_then(|s| {
                if s.success() {
                    Ok(())
                } else {
                    Err(format!("systemctl reboot: {s}"))
                }
            })
    }

    fn is_demo(&self) -> bool {
        false
    }
}

impl Process {
    fn run_inner(
        &self,
        config: &InstallConfig,
        secrets: &Secrets,
        emit: &mut dyn FnMut(RunMsg),
    ) -> Result<(), String> {
        let toml =
            toml::to_string(config).map_err(|e| format!("could not write install.toml: {e}"))?;
        let config_file = TempConfig::create(&toml)?;

        let mut child = Command::new(&self.bin)
            .arg("run")
            .arg("--config")
            .arg(&config_file.path)
            .args(["--secrets-stdin", "--events", "json"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("could not start {}: {e}", self.bin.display()))?;

        // Secrets: one JSON object, then EOF.
        {
            let mut stdin = child.stdin.take().expect("piped stdin");
            let json = serde_json::to_string(secrets).expect("secrets serialize");
            stdin
                .write_all(json.as_bytes())
                .and_then(|()| stdin.write_all(b"\n"))
                .map_err(|e| format!("could not pass secrets to dotinstall: {e}"))?;
        }

        // Both pipes on their own threads, merged into one channel, so a quiet
        // stdout never delays stderr lines (or the other way round).
        enum Line {
            Out(String),
            Err(String),
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let stdout = child.stdout.take().expect("piped stdout");
        let stderr = child.stderr.take().expect("piped stderr");
        let out_tx = tx.clone();
        let readers = [
            std::thread::spawn(move || {
                for l in BufReader::new(stdout).lines().map_while(Result::ok) {
                    if out_tx.send(Line::Out(l)).is_err() {
                        break;
                    }
                }
            }),
            std::thread::spawn(move || {
                for l in BufReader::new(stderr).lines().map_while(Result::ok) {
                    if tx.send(Line::Err(l)).is_err() {
                        break;
                    }
                }
            }),
        ];

        let mut saw_end = false;
        let mut last_err = String::new();
        for line in rx {
            match line {
                Line::Err(l) => {
                    if !l.trim().is_empty() {
                        last_err = l.trim().to_string();
                    }
                    emit(RunMsg::Stderr(l))
                }
                Line::Out(l) if l.trim().is_empty() => {}
                Line::Out(l) => match Event::from_json_line(&l) {
                    Ok(ev) => {
                        saw_end |= matches!(ev, Event::Done { .. } | Event::Failed { .. });
                        emit(RunMsg::Event(ev));
                    }
                    Err(e) => emit(RunMsg::Stderr(format!("(unparsed event: {e}) {l}"))),
                },
            }
        }
        for r in readers {
            let _ = r.join();
        }
        let status = child.wait().map_err(|e| e.to_string())?;
        drop(config_file);

        match (status.success(), saw_end) {
            (true, _) => Ok(()),
            // The failure was already reported as an event.
            (false, true) => Ok(()),
            (false, false) => Err(match status.code() {
                // The last stderr line is the engine saying why; show that.
                Some(2) if !last_err.is_empty() => {
                    format!(
                        "dotinstall refused to start: {}",
                        last_err.trim_start_matches("dotinstall: ")
                    )
                }
                Some(2) => "dotinstall refused the configuration and didn't say why".into(),
                Some(c) if !last_err.is_empty() => {
                    format!("dotinstall died (exit {c}): {last_err}")
                }
                Some(c) => format!("dotinstall died with exit code {c} and no explanation"),
                None => "dotinstall was killed.".into(),
            }),
        }
    }
}

/// `install.toml` in a 0600 file that is removed when dropped.
struct TempConfig {
    path: PathBuf,
}

impl TempConfig {
    fn create(contents: &str) -> Result<Self, String> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        let path =
            std::env::temp_dir().join(format!("dotinstall-{}-{nanos}.toml", std::process::id()));
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut f = opts
            .open(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        f.write_all(contents.as_bytes())
            .map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(Self { path })
    }
}

impl Drop for TempConfig {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Canned machine and `dotinstall_core::demo::run` on a thread.
pub struct Demo {
    pub speed: f32,
    pub fail_at: Option<StepId>,
    /// `--demo-fail-at` fails the first attempt only, so Retry can be shown working.
    failed_once: AtomicBool,
}

impl Demo {
    pub fn new(speed: f32, fail_at: Option<StepId>) -> Self {
        Self {
            speed,
            fail_at,
            failed_once: AtomicBool::new(false),
        }
    }
}

impl Backend for Demo {
    fn probe(&self) -> Result<Probe, String> {
        Ok(demo_probe())
    }

    fn run(&self, _config: &InstallConfig, _secrets: &Secrets, emit: &mut dyn FnMut(RunMsg)) {
        let fail_at = if self.failed_once.swap(true, Ordering::SeqCst) {
            None
        } else {
            self.fail_at
        };
        let opts = demo::DemoOptions {
            speed: self.speed,
            fail_at,
        };
        demo::run(&opts, &mut |ev| {
            emit(RunMsg::Event(demo_detail(ev)));
        });
        emit(RunMsg::Exited(Ok(())));
    }

    fn reboot(&self) -> Result<(), String> {
        std::process::exit(0);
    }

    fn is_demo(&self) -> bool {
        true
    }
}

/// core's demo reports unpack progress in whole gigabytes ("4 / 9.4 GB");
/// show what the real engine will send instead ("4.1 / 9.4 GB").
fn demo_detail(ev: Event) -> Event {
    match ev {
        Event::Progress {
            id: StepId::Unpack,
            fraction: Some(f),
            detail: Some(_),
        } => Event::Progress {
            id: StepId::Unpack,
            fraction: Some(f),
            detail: Some(format!("{:.1} / 9.4 GB", 9.4 * f)),
        },
        other => other,
    }
}

const GB: u64 = 1_000_000_000;

pub fn demo_probe() -> Probe {
    let strings = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    Probe {
        arch: "x86_64".into(),
        uefi: true,
        disks: vec![
            DiskInfo {
                path: "/dev/nvme0n1".into(),
                model: "Samsung SSD 990 PRO 2TB".into(),
                size_bytes: 2000 * GB,
                removable: false,
                is_boot_medium: false,
                partitions: vec![
                    PartitionInfo {
                        path: "/dev/nvme0n1p1".into(),
                        size_bytes: 260_000_000,
                        fstype: Some("vfat".into()),
                        label: Some("SYSTEM".into()),
                        os_hint: Some("EFI system".into()),
                    },
                    PartitionInfo {
                        path: "/dev/nvme0n1p2".into(),
                        size_bytes: 16_000_000,
                        fstype: None,
                        label: None,
                        os_hint: Some("Microsoft reserved".into()),
                    },
                    PartitionInfo {
                        path: "/dev/nvme0n1p3".into(),
                        size_bytes: 1_180 * GB,
                        fstype: Some("ntfs".into()),
                        label: Some("Windows".into()),
                        os_hint: Some("Windows".into()),
                    },
                    PartitionInfo {
                        path: "/dev/nvme0n1p4".into(),
                        size_bytes: 750_000_000,
                        fstype: Some("ntfs".into()),
                        label: Some("Recovery".into()),
                        os_hint: Some("Windows recovery".into()),
                    },
                ],
                free_regions: vec![Region {
                    start: 1_181_100 * 1_000_000,
                    end: 1_999_900 * 1_000_000,
                }],
            },
            DiskInfo {
                path: "/dev/sda".into(),
                model: "Crucial MX500 500GB".into(),
                size_bytes: 500 * GB,
                removable: false,
                is_boot_medium: false,
                partitions: vec![PartitionInfo {
                    path: "/dev/sda1".into(),
                    size_bytes: 500 * GB - 2_000_000,
                    fstype: Some("ext4".into()),
                    label: Some("data".into()),
                    os_hint: None,
                }],
                free_regions: vec![],
            },
            DiskInfo {
                path: "/dev/sdb".into(),
                model: "SanDisk Extreme USB".into(),
                size_bytes: 64 * GB,
                removable: true,
                is_boot_medium: true,
                partitions: vec![],
                free_regions: vec![],
            },
        ],
        keymaps: strings(&[
            "us", "us(intl)", "gb", "it", "de", "fr", "es", "pl", "jp", "dvorak", "colemak",
        ]),
        timezones: crate::data::TIMEZONES
            .iter()
            .map(|s| s.to_string())
            .collect(),
        locales: strings(&[
            "en_US.UTF-8",
            "en_GB.UTF-8",
            "it_IT.UTF-8",
            "pl_PL.UTF-8",
            "de_DE.UTF-8",
            "fr_FR.UTF-8",
            "es_ES.UTF-8",
            "ja_JP.UTF-8",
        ]),
        wifi: vec![
            WifiNetwork {
                ssid: "Nerv HQ".into(),
                signal: 92,
                secured: true,
            },
            WifiNetwork {
                ssid: "Section 9".into(),
                signal: 71,
                secured: true,
            },
            WifiNetwork {
                ssid: "Shinjuku Free Wi-Fi".into(),
                signal: 48,
                secured: false,
            },
            WifiNetwork {
                ssid: "Vodafone-A1B2".into(),
                signal: 33,
                secured: true,
            },
            WifiNetwork {
                ssid: "DIRECT-roku-417".into(),
                signal: 12,
                secured: true,
            },
        ],
        online: false,
        payload: Some(Manifest {
            version: "0.1.0".into(),
            codename: Some("Shinjuku".into()),
            arch: "x86_64".into(),
            built_at: "2026-10-16T09:41:00Z".into(),
            commit: "42c4611".into(),
            username: "kuba".into(),
            hostname: "tuxedo".into(),
            rootfs: "rootfs.squashfs".into(),
            rootfs_bytes: 3_900_000_000,
            installed_bytes: 9_400_000_000,
            sha256: "0".repeat(64),
            kernel: "6.17.0-5-generic".into(),
            home_generation: "/nix/store/demo-home-manager-generation".into(),
            git_name: Some("Kuba Example".into()),
            git_email: Some("kuba@example.com".into()),
        }),
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    /// A stand-in `dotinstall`: `probe` prints the demo probe, `run` checks
    /// its config file is private, echoes the secrets to stderr and replays
    /// a few events, then exits with `$EXIT`.
    fn fake(dir: &std::path::Path, exit: i32, events: &[Event]) -> PathBuf {
        let probe = serde_json::to_string(&demo_probe()).unwrap();
        let lines: String = events
            .iter()
            .map(|e| format!("echo '{}'\n", e.to_json_line()))
            .collect();
        let script = format!(
            "#!/bin/sh\ncase \"$1\" in\nprobe) echo '{probe}' ;;\nrun)\n  \
             cfg=\"$3\"\n  [ \"$(ls -l \"$cfg\" | cut -c1-10)\" = -rw------- ] || exit 9\n  \
             grep -q '^\\[disk\\]' \"$cfg\" || exit 8\n  read -r secrets\n  echo \"got $secrets\" >&2\n{lines}  exit {exit} ;;\nesac\n"
        );
        let path = dir.join("dotinstall");
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    fn run(bin: PathBuf) -> Vec<RunMsg> {
        let p = Process { bin };
        let probe = p.probe().unwrap();
        let mut form = crate::model::Form::new(&probe);
        form.full_name = "J".into();
        form.password = "pw".into();
        let mut out = vec![];
        p.run(&form.to_config(&probe), &form.to_secrets(), &mut |m| {
            out.push(m)
        });
        out
    }

    #[test]
    fn process_streams_events_and_secrets() {
        let dir = std::env::temp_dir().join(format!("dotinstall-gui-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let evs = [
            Event::StepStarted {
                id: StepId::Preflight,
            },
            Event::Done { elapsed_ms: 5 },
        ];
        let out = run(fake(&dir, 0, &evs));
        let events: Vec<&Event> = out
            .iter()
            .filter_map(|m| {
                if let RunMsg::Event(e) = m {
                    Some(e)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(events, evs.iter().collect::<Vec<_>>());
        assert!(
            out.iter()
                .any(|m| matches!(m, RunMsg::Stderr(l) if l.contains("\"user_password\":\"pw\"")))
        );
        assert!(matches!(out.last(), Some(RunMsg::Exited(Ok(())))));

        // Dies without a `failed` event: the GUI gets a reason.
        let out = run(fake(&dir, 3, &[]));
        assert!(matches!(out.last(), Some(RunMsg::Exited(Err(e))) if e.contains("exit 3")));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
