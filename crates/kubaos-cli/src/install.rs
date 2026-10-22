//! The install commands: the headless installer. See docs/INSTALLER.md.
//!
//!   kubaos probe [--json] [--payload DIR]
//!   kubaos validate --config FILE
//!   kubaos plan --config FILE [--json]
//!   kubaos run --config FILE (--secrets FILE | --secrets-stdin) [--events json|human] [--dry-run]
//!   kubaos demo [--speed N] [--fail-at STEP] [--events json|human]
//!
//! Exit codes: 0 ok, 1 a step failed (the last event is `failed`), 2 usage
//! or config error.

use std::fs::File;
use std::io::{IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use clap::Subcommand;
use kubaos_core::engine::{self, RunOptions, machine, runner};
use kubaos_core::{Event, InstallConfig, Level, Secrets, StepId, demo};
use signal_hook::consts::SIGHUP;
use signal_hook::consts::SIGINT;
use signal_hook::consts::SIGTERM;
use signal_hook::iterator::Signals;

use crate::Events;
use crate::usage_error;

const LIVE_LOG: &str = "/var/log/dotinstall.log";

#[derive(Subcommand)]
pub(crate) enum InstallCmd {
    /// Machine facts: disks, keymaps, timezones, Wi-Fi, the payload.
    Probe {
        #[arg(long)]
        json: bool,
        /// Payload directory (default /run/dotinstall/payload).
        #[arg(long)]
        payload: Option<PathBuf>,
    },
    /// Check install.toml against this machine and the payload.
    Validate {
        #[arg(long)]
        config: PathBuf,
    },
    /// The steps `run` would take.
    Plan {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Install. Must be root (except with --dry-run).
    Run {
        #[arg(long)]
        config: PathBuf,
        /// TOML file with the secrets, mode 0600.
        #[arg(long, conflicts_with = "secrets_stdin")]
        secrets: Option<PathBuf>,
        /// Read the secrets as one JSON object from stdin.
        #[arg(long)]
        secrets_stdin: bool,
        #[arg(long, value_enum, default_value = "human")]
        events: Events,
        /// Print the commands each step would run; change nothing.
        #[arg(long)]
        dry_run: bool,
        /// Where to mount the target (default /mnt/dotinstall).
        #[arg(long)]
        target: Option<String>,
        /// Log file (default /var/log/dotinstall.log when installing).
        #[arg(long)]
        log: Option<PathBuf>,
        /// Show debug lines (every command and its output) in human mode.
        #[arg(long, short)]
        verbose: bool,
    },
    /// A fake install with real event shapes, for GUI work.
    Demo {
        #[arg(long, default_value_t = 1.0)]
        speed: f32,
        #[arg(long, value_parser = parse_step)]
        fail_at: Option<StepId>,
        #[arg(long, value_enum, default_value = "human")]
        events: Events,
    },
}

fn parse_step(s: &str) -> Result<StepId, String> {
    serde_json::from_value(serde_json::Value::String(s.into()))
        .map_err(|_| format!("unknown step {s}"))
}

pub(crate) fn main(cmd: InstallCmd) -> ExitCode {
    match cmd {
        InstallCmd::Probe { json, payload } => cmd_probe(json, payload),
        InstallCmd::Validate { config } => cmd_validate(&config),
        InstallCmd::Plan { config, json } => cmd_plan(&config, json),
        InstallCmd::Run {
            config,
            secrets,
            secrets_stdin,
            events,
            dry_run,
            target,
            log,
            verbose,
        } => cmd_run(
            &config,
            secrets,
            secrets_stdin,
            events,
            dry_run,
            target,
            log,
            verbose,
        ),
        InstallCmd::Demo {
            speed,
            fail_at,
            events,
        } => {
            let opts = demo::DemoOptions { speed, fail_at };
            let mut out = Printer::new(events, None, false);
            let ok = demo::run(&opts, &mut |e| out.event(&e));
            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
    }
}

fn load_config(path: &Path) -> Result<InstallConfig, String> {
    let s = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    InstallConfig::from_toml(&s).map_err(|e| format!("{}: {e}", path.display()))
}

fn cmd_probe(json: bool, payload: Option<PathBuf>) -> ExitCode {
    let dir = payload.unwrap_or_else(|| PathBuf::from(machine::DEFAULT_PAYLOAD_DIR));
    let p = match engine::probe_with(&dir) {
        Ok(p) => p,
        Err(e) => return usage_error(e),
    };
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&p).expect("probe serializes")
        );
        return ExitCode::SUCCESS;
    }
    println!("arch {}  uefi {}  online {}", p.arch, p.uefi, p.online);
    for d in &p.disks {
        let flags = [
            (d.removable, "removable"),
            (d.is_boot_medium, "boot medium"),
        ]
        .iter()
        .filter(|(on, _)| *on)
        .map(|(_, s)| *s)
        .collect::<Vec<_>>()
        .join(", ");
        println!(
            "{}  {}  {}  {flags}",
            d.path,
            machine::gib(d.size_bytes),
            d.model
        );
        for part in &d.partitions {
            println!(
                "  {}  {}  {}  {}",
                part.path,
                machine::gib(part.size_bytes),
                part.fstype.as_deref().unwrap_or("-"),
                part.os_hint.as_deref().unwrap_or("")
            );
        }
        for r in &d.free_regions {
            println!(
                "  free {}..{}  {}",
                r.start,
                r.end,
                machine::gib(r.end - r.start)
            );
        }
    }
    println!(
        "{} keymaps, {} timezones, {} locales, {} Wi-Fi networks",
        p.keymaps.len(),
        p.timezones.len(),
        p.locales.len(),
        p.wifi.len()
    );
    match &p.payload {
        Some(m) => println!(
            "payload {} ({}, {}) for {}@{}, {} unpacked",
            m.version,
            m.arch,
            m.commit,
            m.username,
            m.hostname,
            machine::gib(m.installed_bytes)
        ),
        None => println!("no payload in {}", dir.display()),
    }
    ExitCode::SUCCESS
}

fn validate_config(c: &InstallConfig) -> Result<(), String> {
    let probe = engine::probe_with(Path::new(&c.payload.dir)).map_err(|e| e.to_string())?;
    engine::validate(c, &probe).map_err(|e| match e {
        kubaos_core::Error::Config(s) => s,
        other => other.to_string(),
    })
}

fn cmd_validate(config: &Path) -> ExitCode {
    let c = match load_config(config) {
        Ok(c) => c,
        Err(e) => return usage_error(e),
    };
    match validate_config(&c) {
        Ok(()) => {
            println!("ok");
            ExitCode::SUCCESS
        }
        Err(problems) => {
            for p in problems.lines() {
                eprintln!("{p}");
            }
            ExitCode::from(2)
        }
    }
}

fn cmd_plan(config: &Path, json: bool) -> ExitCode {
    let c = match load_config(config) {
        Ok(c) => c,
        Err(e) => return usage_error(e),
    };
    let steps = engine::plan(&c);
    if json {
        let e = Event::Plan {
            steps: steps.iter().map(|s| s.info()).collect(),
        };
        println!("{}", e.to_json_line());
    } else {
        for s in steps {
            println!(
                "{:<12} {:>3}  {}",
                serde_json::to_value(s).unwrap().as_str().unwrap_or(""),
                s.weight(),
                s.title()
            );
        }
    }
    ExitCode::SUCCESS
}

fn read_secrets(file: Option<PathBuf>, stdin: bool) -> Result<Option<Secrets>, String> {
    if stdin {
        let mut s = String::new();
        std::io::stdin()
            .read_to_string(&mut s)
            .map_err(|e| format!("stdin: {e}"))?;
        return serde_json::from_str(s.trim())
            .map(Some)
            .map_err(|e| format!("secrets on stdin: {e}"));
    }
    let Some(path) = file else { return Ok(None) };
    use std::os::unix::fs::MetadataExt;
    let meta = std::fs::metadata(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    if meta.mode() & 0o077 != 0 {
        return Err(format!(
            "{} must not be readable by others (chmod 600)",
            path.display()
        ));
    }
    let s = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&s)
        .map(Some)
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// Turns SIGINT, SIGTERM and SIGHUP into `runner::INTERRUPTED`, which stops the install and
/// unwinds it instead of killing the process with the disk half done.
fn stop_on_signals() {
    let mut signals = match Signals::new([SIGINT, SIGTERM, SIGHUP]) {
        Ok(signals) => signals,
        Err(e) => {
            eprintln!("kubaos: can't catch signals: {e}");
            return;
        }
    };
    std::thread::spawn(move || {
        for _ in signals.forever() {
            runner::INTERRUPTED.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn cmd_run(
    config: &Path,
    secrets: Option<PathBuf>,
    secrets_stdin: bool,
    events: Events,
    dry_run: bool,
    target: Option<String>,
    log: Option<PathBuf>,
    verbose: bool,
) -> ExitCode {
    let c = match load_config(config) {
        Ok(c) => c,
        Err(e) => return usage_error(e),
    };
    let secrets = match read_secrets(secrets, secrets_stdin) {
        Ok(Some(s)) => s,
        Ok(None) if dry_run => Secrets {
            luks_passphrase: Some("<passphrase>".into()),
            user_password: "<password>".into(),
            wifi_psk: None,
        },
        Ok(None) => return usage_error("run needs --secrets FILE or --secrets-stdin"),
        Err(e) => return usage_error(e),
    };
    if !dry_run {
        if !rustix::process::geteuid().is_root() {
            return usage_error("run must be root (or use --dry-run)");
        }
        if let Err(problems) = validate_config(&c) {
            for p in problems.lines() {
                eprintln!("{p}");
            }
            return ExitCode::from(2);
        }
        if let Err(e) = engine::validate_secrets(&c, &secrets) {
            return usage_error(e);
        }
    }
    stop_on_signals();

    let log_path = log.or_else(|| (!dry_run).then(|| PathBuf::from(LIVE_LOG)));
    let log_file = log_path.as_ref().and_then(|p| {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(p)
            .map_err(|e| eprintln!("kubaos: can't write the log {}: {e}", p.display()))
            .ok()
    });
    let opts = RunOptions {
        dry_run,
        target_root: target,
        log_file: log_path.filter(|_| log_file.is_some()),
    };
    let mut out = Printer::new(events, log_file, verbose);
    match engine::run(&c, &secrets, &opts, &mut |e| out.event(&e)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(kubaos_core::Error::Step { .. }) => ExitCode::from(1),
        Err(e) => {
            eprintln!("kubaos: {e}");
            ExitCode::from(1)
        }
    }
}

/// Writes events: JSON lines or human text on stdout, human text on stderr
/// (JSON mode) and always into the log file.
struct Printer {
    mode: Events,
    log: Option<File>,
    verbose: bool,
    started: Instant,
    color: bool,
    last_pct: Option<u32>,
}

impl Printer {
    fn new(mode: Events, log: Option<File>, verbose: bool) -> Self {
        let color = mode == Events::Human && std::io::stdout().is_terminal();
        Printer {
            mode,
            log,
            verbose,
            started: Instant::now(),
            color,
            last_pct: None,
        }
    }

    fn human(&mut self, e: &Event, for_log: bool) -> Option<String> {
        let (b, off) = if self.color && !for_log {
            ("\x1b[1;34m", "\x1b[0m")
        } else {
            ("", "")
        };
        Some(match e {
            Event::Plan { steps } => format!(
                "plan: {}",
                steps
                    .iter()
                    .map(|s| s.title.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Event::StepStarted { id } => format!("{b}==> {}{off}", id.title()),
            Event::StepFinished { elapsed_ms, .. } => {
                format!("    done in {:.1}s", *elapsed_ms as f64 / 1000.0)
            }
            Event::Progress {
                fraction, detail, ..
            } => {
                // Every 10% is plenty for a human.
                let pct = (fraction.unwrap_or(0.0) * 100.0) as u32 / 10 * 10;
                if for_log || self.last_pct == Some(pct) {
                    return None;
                }
                self.last_pct = Some(pct);
                format!(
                    "    {pct}%{}",
                    detail
                        .as_ref()
                        .map(|d| format!("  {d}"))
                        .unwrap_or_default()
                )
            }
            Event::Log { level, line, .. } => {
                if *level == Level::Debug && !self.verbose && !for_log {
                    return None;
                }
                let tag = match level {
                    Level::Debug | Level::Info => "   ",
                    Level::Warn => " !!",
                    Level::Error => " xx",
                };
                format!("{tag} {line}")
            }
            Event::Failed { error, hint, .. } => {
                let mut s = format!("!!! {error}");
                if let Some(h) = hint {
                    s.push_str(&format!("\n    {h}"));
                }
                s
            }
            Event::Done { elapsed_ms } => {
                format!("installed in {:.1}s", *elapsed_ms as f64 / 1000.0)
            }
        })
    }

    fn event(&mut self, e: &Event) {
        if matches!(e, Event::StepStarted { .. }) {
            self.last_pct = None;
        }
        if self.log.is_some()
            && let Some(line) = self.human(e, true)
        {
            let t = self.started.elapsed().as_secs_f64();
            if let Some(f) = self.log.as_mut() {
                for l in line.lines() {
                    let _ = writeln!(f, "[{t:8.2}] {l}");
                }
            }
        }
        match self.mode {
            Events::Json => {
                let mut so = std::io::stdout().lock();
                let _ = writeln!(so, "{}", e.to_json_line());
                let _ = so.flush();
                if let Some(line) = self.human(e, false) {
                    eprintln!("{line}");
                }
            }
            Events::Human => {
                if let Some(line) = self.human(e, false) {
                    println!("{line}");
                }
            }
        }
    }
}
