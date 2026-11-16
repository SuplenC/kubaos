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
use std::io::PipeReader;
use std::io::Read;
use std::io::Write;
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
/// The longest line of a background program's that is passed on whole, in bytes; the rest of a
/// longer one is dropped. Far above an update event's JSON, which stays under a few KiB.
const LINE_CAP: usize = 64 * 1024;
/// What follows a background program's own output once it exits: its own line, as the newline
/// before it ends a last line the program left open, and a NUL no line of text or JSON starts
/// with.
const END_MARK: &[u8] = b"\n\0kubaos-ended\n";

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

    /// Starts `argv` and leaves it running, giving `line` each line that isn't blank it writes to
    /// stdout or stderr, then, after the last of them, `ended` whether it exited with success.
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

    fn background(&self, argv: &[String], line: OnLine, ended: OnEnd) -> Result<(), RunError> {
        let Some((program, rest)) = argv.split_first() else {
            return Err(RunError::start("")(io::Error::from(io::ErrorKind::InvalidInput)));
        };
        let start = RunError::start(program);
        let (reader, writer) = io::pipe().map_err(RunError::start(program))?;
        let stderr = writer.try_clone().map_err(RunError::start(program))?;
        let mut mark = writer.try_clone().map_err(RunError::start(program))?;
        // A group of its own, so a signal to the Extension's group, such as Ctrl+C in the
        // terminal a Daemon runs in, doesn't stop an install halfway. The Daemon itself kills
        // only the Extension's own process, so the install outlives it: its writes to the closed
        // output then fail, which `kubaos app --json` ignores, and the run goes on to its end.
        let mut child = Command::new(program)
            .args(rest)
            .stdin(Stdio::null())
            .stdout(writer)
            .stderr(stderr)
            .process_group(0)
            .spawn()
            .map_err(start)?;
        let (exited, exit) = mpsc::channel();
        thread::Builder::new()
            .name("background-output".into())
            .stack_size(DELAY_STACK)
            .spawn(move || {
                follow(reader, line);
                ended(exit.recv().unwrap_or(false));
            })
            .map_err(RunError::start(program))?;
        thread::Builder::new()
            .name("background".into())
            .stack_size(DELAY_STACK)
            .spawn(move || {
                let succeeded = child.wait().is_ok_and(|status| status.success());
                let _ = exited.send(succeeded);
                // Behind everything the program wrote, as a pipe keeps its order. Once the
                // output thread stopped, it is closed and the write fails, harmlessly.
                let _ = mark.write_all(END_MARK);
            })
            .map(drop)
            .map_err(RunError::start(program))
    }
}

/// Passes each line of `output` that isn't blank to `line`, decoded lossily and cut to
/// [`LINE_CAP`] bytes, until [`END_MARK`] or the output closes.
///
/// It stops at the mark, though something the program started and left running may still hold
/// the output open: the Job is over, and reading on would keep this thread and the pipe for as
/// long as that process lives. Its later writes then fail, as they would had the Extension gone.
fn follow(mut output: PipeReader, mut line: OnLine) {
    let mark = &END_MARK[1..END_MARK.len() - 1];
    let mut bytes = Vec::new();
    let mut buf = [0; 4096];
    loop {
        let n = match output.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        };
        for &byte in &buf[..n] {
            if byte != b'\n' {
                if bytes.len() < LINE_CAP {
                    bytes.push(byte);
                }
                continue;
            }
            if bytes == mark {
                return;
            }
            pass_on(&mut bytes, &mut line);
        }
    }
    pass_on(&mut bytes, &mut line);
}

/// Passes `bytes` to `line` as text unless it is blank, and empties it.
fn pass_on(bytes: &mut Vec<u8>, line: &mut OnLine) {
    let text = String::from_utf8_lossy(bytes);
    if !text.trim().is_empty() {
        line(text.into_owned());
    }
    bytes.clear();
}

#[cfg(test)]
mod tests;
