//! Running programs, behind [`Runner`], so what the actions and toggles run
//! is tested without the programs.
//!
//! # Owns
//!
//! - [`Runner`], and [`Programs`], which runs them for real.
//!
//! # Not here
//!
//! - Detaching a launched program and its systemd scope: `kotodama_extension::launch`.
//! - Which programs run: [`crate::registry`], [`crate::toggles`].

use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use kotodama_extension::Chain;
use kotodama_extension::launch;
use kotodama_extension::log;

/// The delaying thread only sleeps, then starts a program; a background program's threads only
/// wait for it, or read its lines.
const DELAY_STACK: usize = 64 * 1024;
/// How long a background program's output may stay open after it exits: something it started
/// and left running may hold it for ever.
const OUTPUT_GRACE: Duration = Duration::from_millis(500);

/// What a background program's lines go to, one at a time.
pub type OnLine = Box<dyn FnMut(String) + Send>;
/// What a background program's end goes to: whether it exited with success.
pub type OnEnd = Box<dyn FnOnce(bool) + Send>;

/// What starts the programs.
pub trait Runner {
    /// Runs `argv` until it exits, and returns what it wrote to stdout.
    ///
    /// # Errors
    ///
    /// When it can't start, or exits with a failure.
    fn output(&self, argv: &[String]) -> Result<String, RunError>;

    /// Starts `argv` once `delay` has passed, and leaves it running.
    ///
    /// # Errors
    ///
    /// When it can't start. After a delay, that is only logged.
    fn launch(&self, argv: &[String], delay: Duration) -> Result<(), RunError>;

    /// Starts `argv` and leaves it running, giving `line` each line it writes to stdout or
    /// stderr, then `ended` whether it exited with success.
    ///
    /// # Errors
    ///
    /// When it, or a thread to follow it, can't start.
    fn background(&self, argv: &[String], line: OnLine, ended: OnEnd) -> Result<(), RunError>;
}

/// Why a program didn't do its job.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    /// It couldn't be started.
    #[error("couldn't start {program}")]
    Start {
        program: String,
        #[source]
        source: io::Error,
    },
    /// It ran and exited with a failure, saying this on stderr.
    #[error("{program} failed: {stderr}")]
    Failed { program: String, stderr: String },
    /// Launching failed, as `kotodama_extension::launch` says in words for the user.
    #[error(transparent)]
    Launch(io::Error),
}

impl RunError {
    fn start(program: &str) -> impl FnOnce(io::Error) -> RunError + '_ {
        move |source| RunError::Start {
            program: program.to_owned(),
            source,
        }
    }
}

/// Runs the programs for real.
#[derive(Clone, Copy, Debug, Default)]
pub struct Programs;

impl Runner for Programs {
    fn output(&self, argv: &[String]) -> Result<String, RunError> {
        let Some((program, rest)) = argv.split_first() else {
            return Err(RunError::start("")(io::Error::from(io::ErrorKind::InvalidInput)));
        };
        let output = Command::new(program)
            .args(rest)
            .stdin(Stdio::null())
            .output()
            .map_err(RunError::start(program))?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        let said = stderr.lines().rev().find(|line| !line.trim().is_empty());
        Err(RunError::Failed {
            program: program.clone(),
            stderr: said.map_or_else(|| output.status.to_string(), |line| line.trim().to_owned()),
        })
    }

    fn launch(&self, argv: &[String], delay: Duration) -> Result<(), RunError> {
        if delay.is_zero() {
            return launch::launch(argv, None).map_err(RunError::Launch);
        }
        let argv = argv.to_vec();
        let program = argv.first().cloned().unwrap_or_default();
        thread::Builder::new()
            .name("delayed".into())
            .stack_size(DELAY_STACK)
            .spawn(move || {
                thread::sleep(delay);
                if let Err(err) = launch::launch(&argv, None) {
                    log!("{}", Chain(&err));
                }
            })
            .map(drop)
            .map_err(RunError::start(&program))
    }

    fn background(&self, argv: &[String], mut line: OnLine, ended: OnEnd) -> Result<(), RunError> {
        let Some((program, rest)) = argv.split_first() else {
            return Err(RunError::start("")(io::Error::from(io::ErrorKind::InvalidInput)));
        };
        let start = RunError::start(program);
        let (reader, writer) = io::pipe().map_err(RunError::start(program))?;
        let stderr = writer.try_clone().map_err(RunError::start(program))?;
        // A group of its own, so a signal to the Extension's group, as the Daemon stopping it
        // sends, doesn't stop an install halfway.
        let mut child = Command::new(program)
            .args(rest)
            .stdin(Stdio::null())
            .stdout(writer)
            .stderr(stderr)
            .process_group(0)
            .spawn()
            .map_err(start)?;
        let (closed, output_closed) = mpsc::channel();
        thread::Builder::new()
            .name("background-output".into())
            .stack_size(DELAY_STACK)
            .spawn(move || {
                for read in BufReader::new(reader).lines() {
                    let Ok(read) = read else { break };
                    line(read);
                }
                let _ = closed.send(());
            })
            .map_err(RunError::start(program))?;
        thread::Builder::new()
            .name("background".into())
            .stack_size(DELAY_STACK)
            .spawn(move || {
                let succeeded = child.wait().is_ok_and(|status| status.success());
                let _ = output_closed.recv_timeout(OUTPUT_GRACE);
                ended(succeeded);
            })
            .map(drop)
            .map_err(RunError::start(program))
    }
}

#[cfg(test)]
mod tests;
