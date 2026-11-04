//! `kubaos-depth`: a picture's foreground cut-out and the lock clock's place and tint on it,
//! written into a folder.
//!
//! ```text
//! kubaos-depth run PICTURE --out DIR --models DIR    cutout.png (when one is kept) and depth.toml
//! ```
//!
//! # Owns
//!
//! - The command line, the process's low priority and its log on stderr: `KUBAOS_DEPTH_LOG=debug`
//!   shows each candidate's numbers. Exit codes: 2 for a usage error, 1 when the run failed.
//!
//! # Not here
//!
//! - The run itself: the library.

use std::env;
use std::ffi::OsString;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;

const USAGE: &str = "\
usage: kubaos-depth run PICTURE --out DIR --models DIR

Writes PICTURE's foreground cut-out (cutout.png, when one is confident) and the lock clock's place
and tint (depth.toml) into DIR, with the ONNX models in --models. Runs at the lowest priority.";

/// The niceness the run takes: the lowest priority, so a desktop in use never waits on it.
const NICENESS: i32 = 19;

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The usage, the outcome and the error the run stops on are the command's output."
)]
fn main() -> ExitCode {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let Some(command) = parse(&args) else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    start_log();
    if let Err(e) = rustix::process::setpriority_process(None, NICENESS) {
        log::warn!("can't lower the priority: {e}");
    }
    let threads = thread::available_parallelism().map_or(1, usize::from);
    match kubaos_depth::run(&command.picture, &command.out, &command.models, threads) {
        Ok(sidecar) => {
            println!("{:?} {:.3}", sidecar.method, sidecar.score);
            ExitCode::SUCCESS
        }
        Err(e) => {
            let mut message = format!("kubaos-depth: {e}");
            let mut source = std::error::Error::source(&e);
            while let Some(cause) = source {
                message.push_str(": ");
                message.push_str(&cause.to_string());
                source = cause.source();
            }
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug)]
struct Command {
    picture: PathBuf,
    out: PathBuf,
    models: PathBuf,
}

fn parse(args: &[OsString]) -> Option<Command> {
    let [run, rest @ ..] = args else {
        return None;
    };
    if run != "run" {
        return None;
    }
    let (mut picture, mut out, mut models) = (None, None, None);
    let mut rest = rest.iter();
    while let Some(arg) = rest.next() {
        match arg.to_str() {
            Some("--out") => out = Some(PathBuf::from(rest.next()?)),
            Some("--models") => models = Some(PathBuf::from(rest.next()?)),
            Some(flag) if flag.starts_with("--") => return None,
            _ if picture.is_none() => picture = Some(PathBuf::from(arg)),
            _ => return None,
        }
    }
    Some(Command {
        picture: picture?,
        out: out?,
        models: models?,
    })
}

fn start_log() {
    let level = match env::var("KUBAOS_DEPTH_LOG").as_deref() {
        Ok("debug") => log::LevelFilter::Debug,
        Ok("warn") => log::LevelFilter::Warn,
        _ => log::LevelFilter::Info,
    };
    if log::set_logger(&Stderr).is_ok() {
        log::set_max_level(level);
    }
}

/// The run's log: a line on stderr each, which the Suite's systemd job hands to the journal.
struct Stderr;

impl log::Log for Stderr {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::max_level() && metadata.target().starts_with("kubaos_depth")
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            let _ = writeln!(std::io::stderr(), "{}", record.args());
        }
    }

    fn flush(&self) {}
}
