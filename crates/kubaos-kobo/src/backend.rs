//! Where probes and installs come from: the engine in this process, or the built-in demo. Both
//! block; the app calls them on worker threads.
//!
//! # Owns
//!
//! - The `Backend` seam the install screens talk to, and its two sides, `Engine` and `Demo`.
//! - The live system's install log and what a signal does while an install runs.
//! - Stopping an install and taking the machine down (`systemctl poweroff` and `reboot`).
//! - Which battery the chrome shows: sysfs for `Engine`, a fixed one for `Demo`.
//!
//! # Not here
//!
//! - The install itself: `kubaos_core::engine`.
//! - First-boot setup's backend: `setup/backend.rs`.
//! - Reading and combining the batteries: `battery.rs`.

use std::fs::File;
use std::io;
use std::io::Write;
use std::panic;
use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use kubaos_core::Event;
use kubaos_core::InstallConfig;
use kubaos_core::Level;
use kubaos_core::Manifest;
use kubaos_core::Probe;
use kubaos_core::Secrets;
use kubaos_core::StepId;
use kubaos_core::demo;
use kubaos_core::engine;
use kubaos_core::engine::RunOptions;
use kubaos_core::engine::runner;
use kubaos_core::probe::DiskInfo;
use kubaos_core::probe::PartitionInfo;
use kubaos_core::probe::Region;
use kubaos_core::probe::WallpaperFile;
use kubaos_core::probe::WallpaperSet;
use kubaos_core::probe::WifiNetwork;
use signal_hook::consts::SIGHUP;
use signal_hook::consts::SIGINT;
use signal_hook::consts::SIGTERM;
use signal_hook::iterator::Signals;

use crate::battery;
use crate::battery::Battery;
use crate::battery::Supply;

/// What an install reports, in order: events, then exactly one `Exited`.
#[derive(Debug, Clone)]
pub enum RunMsg {
    Event(Event),
    /// The run is over. `Err` carries a reason when the install ended without a `failed` event:
    /// it never started, or the engine crashed.
    Exited(Result<(), String>),
}

pub trait Backend: Send + Sync {
    fn probe(&self) -> Result<Probe, String>;
    fn run(&self, config: &InstallConfig, secrets: &Secrets, emit: &mut dyn FnMut(RunMsg));
    /// Asks the running install to stop. It unwinds on `run`'s thread, which then reports a
    /// `failed` event and `Exited`, as it does for any failure. Does nothing when no install runs.
    fn stop(&self);
    /// `systemctl reboot`: returns once systemd has the job, before the machine goes down.
    ///
    /// # Errors
    ///
    /// systemctl couldn't run or refused, with its reason.
    fn reboot(&self) -> Result<(), String>;
    /// `systemctl poweroff`, as `reboot` does.
    ///
    /// # Errors
    ///
    /// systemctl couldn't run or refused, with its reason.
    fn poweroff(&self) -> Result<(), String>;
    /// The machine's batteries combined, or `None` when it has none. Reads files, so the app
    /// calls it off the UI thread.
    fn battery(&self) -> Option<Battery>;
    /// Shown in the corner so nobody mistakes the demo for a real install.
    fn is_demo(&self) -> bool;
}

/// The live system's install log. The engine copies it onto the target when the install finishes.
const INSTALL_LOG: &str = "/var/log/kubaos-install.log";

/// Set while the engine installs, so a signal stops the install instead of killing Kōbō.
static INSTALLING: AtomicBool = AtomicBool::new(false);

/// Runs the install in this process through `kubaos_core::engine` (docs/INSTALL-INTERNALS.md).
/// Kōbō runs as root under `cage` on the live system.
#[derive(Debug)]
pub struct Engine {
    log: PathBuf,
    /// Prints the commands each step would run and changes nothing. Needs neither root nor a
    /// payload.
    dry_run: bool,
}

impl Engine {
    /// The real install, logged to `/var/log/kubaos-install.log`.
    #[must_use]
    pub fn live() -> Self {
        Self {
            log: PathBuf::from(INSTALL_LOG),
            dry_run: false,
        }
    }
}

impl Backend for Engine {
    fn probe(&self) -> Result<Probe, String> {
        engine::probe().map_err(|e| format!("probe failed: {}", problems(e)))
    }

    fn run(&self, config: &InstallConfig, secrets: &Secrets, emit: &mut dyn FnMut(RunMsg)) {
        let result = self.install(config, secrets, emit);
        emit(RunMsg::Exited(result));
    }

    /// The runner refuses new commands and kills the one streaming, then the engine unmounts and
    /// closes what it opened, as it does on SIGTERM.
    fn stop(&self) {
        if INSTALLING.load(Ordering::SeqCst) {
            runner::INTERRUPTED.store(true, Ordering::SeqCst);
        }
    }

    fn reboot(&self) -> Result<(), String> {
        systemctl("reboot")
    }

    fn poweroff(&self) -> Result<(), String> {
        systemctl("poweroff")
    }

    fn battery(&self) -> Option<Battery> {
        battery::read(Path::new(battery::SYSFS))
    }

    fn is_demo(&self) -> bool {
        false
    }
}

fn systemctl(verb: &str) -> Result<(), String> {
    let status = Command::new("systemctl")
        .arg(verb)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("systemctl {verb}: {status}"))
    }
}

impl Engine {
    /// Checks what `kubaos run` checks before it starts, then installs. A failing step is reported
    /// by its `failed` event, so only a refusal or a crash comes back as `Err`.
    fn install(&self, config: &InstallConfig, secrets: &Secrets, emit: &mut dyn FnMut(RunMsg)) -> Result<(), String> {
        // Marked before preflight, so a stop or a signal during it holds and the engine stops at
        // its first step. Marked after, `Installing::start` would clear it.
        let _installing = Installing::start();
        if !self.dry_run {
            if !rustix::process::geteuid().is_root() {
                return Err("Kōbō has to run as root to install".into());
            }
            preflight(config, secrets).map_err(|p| format!("the install didn't start: {p}"))?;
        }

        let mut log = match InstallLog::open(&self.log) {
            Ok(log) => Some(log),
            Err(e) => {
                emit(RunMsg::Event(Event::Log {
                    id: StepId::Preflight,
                    level: Level::Warn,
                    line: format!("can't write the log {}: {e}", self.log.display()),
                }));
                None
            }
        };
        let opts = RunOptions {
            dry_run: self.dry_run,
            target_root: None,
            log_file: log.as_ref().map(|_| self.log.clone()),
        };

        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            engine::run(config, secrets, &opts, &mut |ev| {
                if let Some(log) = log.as_mut() {
                    log.write(&ev);
                }
                emit(RunMsg::Event(ev));
            })
        }));
        exit_reason(result)
    }
}

/// Every reason the engine refuses this install, one per line, as `kubaos validate` lists them.
fn preflight(config: &InstallConfig, secrets: &Secrets) -> Result<(), String> {
    let probe = engine::probe_with(Path::new(&config.payload.dir)).map_err(problems)?;
    engine::validate(config, &probe).map_err(problems)?;
    engine::validate_secrets(config, secrets).map_err(problems)
}

/// The engine's own message, without the "invalid config" prefix the CLI prints.
fn problems(e: kubaos_core::Error) -> String {
    match e {
        kubaos_core::Error::Config(problems) => problems.lines().collect::<Vec<_>>().join("; "),
        other => other.to_string(),
    }
}

/// What the install screens make of the engine's result. A step failure was already reported as
/// a `failed` event.
fn exit_reason(result: thread::Result<Result<(), kubaos_core::Error>>) -> Result<(), String> {
    match result {
        Ok(Ok(()) | Err(kubaos_core::Error::Step { .. })) => Ok(()),
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err("the install engine crashed. The reason is in /var/log/kobo.log".into()),
    }
}

/// Marks an install as running until dropped, also when the engine panics.
struct Installing;

impl Installing {
    fn start() -> Self {
        runner::INTERRUPTED.store(false, Ordering::SeqCst);
        INSTALLING.store(true, Ordering::SeqCst);
        Self
    }
}

impl Drop for Installing {
    fn drop(&mut self) {
        INSTALLING.store(false, Ordering::SeqCst);
    }
}

/// Routes SIGINT, SIGTERM and SIGHUP. During an install they set `runner::INTERRUPTED`, which
/// stops the install and unwinds it, as `kubaos run` does: everything is unmounted and closed and
/// the last event is `failed`. Otherwise Kōbō exits, as it would without a handler.
pub fn stop_install_on_signals() {
    let mut signals = match Signals::new([SIGINT, SIGTERM, SIGHUP]) {
        Ok(signals) => signals,
        Err(e) => {
            log::warn!("can't catch signals: {e}");
            return;
        }
    };
    thread::spawn(move || {
        for signal in signals.forever() {
            if !INSTALLING.load(Ordering::SeqCst) {
                std::process::exit(128 + signal);
            }
            runner::INTERRUPTED.store(true, Ordering::SeqCst);
        }
    });
}

/// The human-readable install log: one timestamped line per event, the same lines `kubaos run`
/// writes to its log.
struct InstallLog {
    file: File,
    started: Instant,
}

impl InstallLog {
    fn open(path: &Path) -> io::Result<Self> {
        let file = File::options().create(true).append(true).open(path)?;
        Ok(Self {
            file,
            started: Instant::now(),
        })
    }

    /// Writes the event's lines. A failed write loses a log line, never the install.
    fn write(&mut self, ev: &Event) {
        let Some(text) = log_text(ev) else {
            return;
        };
        let t = self.started.elapsed().as_secs_f64();
        for line in text.lines() {
            let _ = writeln!(self.file, "[{t:8.2}] {line}");
        }
    }
}

/// An event as the log shows it. Progress is left out: the step lines around it say enough.
fn log_text(ev: &Event) -> Option<String> {
    let text = match ev {
        Event::Plan { steps } => {
            let titles: Vec<&str> = steps.iter().map(|s| s.title.as_str()).collect();
            format!("plan: {}", titles.join(", "))
        }
        Event::StepStarted { id } => format!("==> {}", id.title()),
        Event::StepFinished { elapsed_ms, .. } => format!("    done in {}", seconds(*elapsed_ms)),
        Event::Progress { .. } => return None,
        Event::Log { level, line, .. } => {
            let tag = match level {
                Level::Debug | Level::Info => "   ",
                Level::Warn => " !!",
                Level::Error => " xx",
            };
            format!("{tag} {line}")
        }
        Event::Failed { error, hint, .. } => match hint {
            Some(hint) => format!("!!! {error}\n    {hint}"),
            None => format!("!!! {error}"),
        },
        Event::Done { elapsed_ms } => format!("installed in {}", seconds(*elapsed_ms)),
    };
    Some(text)
}

/// `1250` ms as `1.2s`.
fn seconds(ms: u64) -> String {
    format!("{:.1}s", Duration::from_millis(ms).as_secs_f64())
}

/// Canned machine and `kubaos_core::demo::run` on a thread.
pub struct Demo {
    pub speed: f32,
    pub fail_at: Option<StepId>,
    /// `--demo-fail-at` fails the first attempt only, so Retry can be shown working.
    failed_once: AtomicBool,
    /// Set by `stop`, cleared when a run starts.
    stopped: AtomicBool,
}

impl Demo {
    pub fn new(speed: f32, fail_at: Option<StepId>) -> Self {
        Self {
            speed,
            fail_at,
            failed_once: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
        }
    }
}

impl Backend for Demo {
    fn probe(&self) -> Result<Probe, String> {
        Ok(demo_probe())
    }

    fn run(&self, _config: &InstallConfig, _secrets: &Secrets, emit: &mut dyn FnMut(RunMsg)) {
        self.stopped.store(false, Ordering::SeqCst);
        let fail_at = if self.failed_once.swap(true, Ordering::SeqCst) {
            None
        } else {
            self.fail_at
        };
        let opts = demo::DemoOptions {
            speed: self.speed,
            fail_at,
        };
        // core's demo can't be stopped, so it runs on a thread of its own and a stopped run leaves
        // it to finish unheard.
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            demo::run(&opts, &mut |ev| {
                let _ = tx.send(demo_detail(ev));
            });
        });
        let mut step = StepId::Preflight;
        loop {
            if self.stopped.load(Ordering::SeqCst) {
                emit(RunMsg::Event(Event::Failed {
                    id: step,
                    error: "interrupted".into(),
                    hint: None,
                }));
                break;
            }
            // design-allow(duration): how long one poll of the engine's channel waits; not motion.
            match rx.recv_timeout(Duration::from_millis(20)) {
                Ok(ev) => {
                    if let Event::StepStarted { id } = ev {
                        step = id;
                    }
                    emit(RunMsg::Event(ev));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        emit(RunMsg::Exited(Ok(())));
    }

    fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
    }

    fn reboot(&self) -> Result<(), String> {
        std::process::exit(0);
    }

    fn poweroff(&self) -> Result<(), String> {
        std::process::exit(0);
    }

    fn battery(&self) -> Option<Battery> {
        Some(DEMO_BATTERY)
    }

    fn is_demo(&self) -> bool {
        true
    }
}

/// The demo's laptop, on battery and well above low, so the chrome shows the indicator.
pub const DEMO_BATTERY: Battery = Battery {
    percent: 64,
    supply: Supply::Discharging,
};

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

#[expect(
    clippy::too_many_lines,
    reason = "One literal: the demo machine's disks, keymaps, zones and locales."
)]
pub fn demo_probe() -> Probe {
    let strings = |v: &[&str]| v.iter().map(ToString::to_string).collect::<Vec<_>>();
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
        timezones: crate::data::TIMEZONES.iter().map(ToString::to_string).collect(),
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
            wallpaper_sets: DEMO_WALLPAPERS
                .iter()
                .map(|(id, names)| demo_wallpaper_set(id, names))
                .collect(),
        }),
    }
}

/// The sets of SuplenC/kubaos-wallpapers, and their files.
const DEMO_WALLPAPERS: [(&str, &[&str]); 3] = [
    (
        "cyberpunk",
        &[
            "akihabara-after-rain.jpg",
            "shibuya-rainy-corner.jpg",
            "shinjuku-neon-canyon.jpg",
        ],
    ),
    ("japanese", &["shinjuku-purple-hour.jpg", "tokyo-tower-blue-hour.jpg"]),
    (
        "shinjuku",
        &[
            "kabukicho-red-gate.jpg",
            "nishi-shinjuku-violet-rain.jpg",
            "omoide-yokocho-lanterns.jpg",
            "skyline-fuji-dusk.jpg",
            "train-neon-streak.jpg",
        ],
    ),
];

/// A set with its thumbnails where a payload has them. The demo has no payload dir, so they're
/// placeholders unless the app is pointed at one (`App::payload_dir`, the screenshots'
/// `GALLERY_PAYLOAD`).
fn demo_wallpaper_set(id: &str, names: &[&str]) -> WallpaperSet {
    let files: Vec<WallpaperFile> = names
        .iter()
        .map(|name| WallpaperFile {
            name: (*name).into(),
            thumb: Some(format!("wallpapers/{id}/{name}.jpg")),
        })
        .collect();
    WallpaperSet {
        id: id.into(),
        images: u32::try_from(files.len()).unwrap_or(u32::MAX),
        preview: files.first().and_then(|f| f.thumb.clone()),
        files,
    }
}

#[cfg(test)]
mod tests;
