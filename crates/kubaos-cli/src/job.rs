//! `kubaos switch` and `kubaos job`: queueing a home-manager switch of the user flake and following
//! it.
//!
//!   kubaos switch [--detach | --foreground]
//!   kubaos job list [--json]
//!   kubaos job follow ID
//!   kubaos job work
//!
//! A switch runs in a transient systemd user unit, one at a time; the command follows its output
//! until it ends, and Ctrl-C stops only the following. Following a job, and listing the jobs, fail
//! a queued job whose unit died before it ran. `--foreground` runs the queue in this
//! process instead, for a session without a systemd user manager. Exit codes: 0 ok, 1 the switch
//! failed, 2 usage.
//!
//! # Owns
//!
//! - The command line of `switch` and `job`, and printing a job's output.
//!
//! # Not here
//!
//! - The queue, the lock and the systemd unit: `kubaos_core::switch`.
//! - The `package` commands, which end in a switch: `package.rs`.

use std::io;
use std::io::Write;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use clap::Args;
use clap::Subcommand;
use kubaos_core::state::Paths;
use kubaos_core::switch::Job;
use kubaos_core::switch::JobId;
use kubaos_core::switch::JobState;
use kubaos_core::switch::Queue;
use kubaos_core::switch::Systemd;

use crate::usage_error;

/// How often a follower looks at the job's files.
const FOLLOW_POLL: Duration = Duration::from_millis(250);

#[derive(Subcommand)]
pub enum JobCmd {
    /// Every switch job, oldest first.
    List {
        #[arg(long)]
        json: bool,
    },
    /// Print a job's output as it runs, until it ends.
    Follow { id: String },
    /// Run every queued job, one at a time. The worker unit runs this.
    #[command(hide = true)]
    Work,
}

/// How a command that ends in a switch waits for it.
#[derive(Args, Clone, Copy, Debug)]
#[group(multiple = false)]
pub struct SwitchMode {
    /// Queue the switch and return at once; `kubaos job follow` shows it.
    #[arg(long)]
    detach: bool,
    /// Run the switch in this process rather than in a systemd user unit, still behind the lock.
    #[arg(long)]
    foreground: bool,
}

pub fn main(cmd: JobCmd) -> ExitCode {
    let paths = match Paths::from_env() {
        Ok(paths) => paths,
        Err(e) => return usage_error(e),
    };
    let queue = Queue::new(paths.jobs());
    let host = match host(&paths) {
        Ok(host) => host,
        Err(code) => return code,
    };
    match cmd {
        JobCmd::List { json } => list(&queue, &host, json),
        JobCmd::Follow { id } => match JobId::parse(id) {
            Ok(id) => follow(&queue, &id, Some(&host)),
            Err(e) => usage_error(e),
        },
        JobCmd::Work => match queue.work(&host) {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => report(e),
        },
    }
}

/// Queues a switch of the user flake and waits for it as `mode` says.
pub fn switch(paths: &Paths, mode: SwitchMode) -> ExitCode {
    if rustix::process::geteuid().is_root() {
        return usage_error("run kubaos switch as yourself, not root: it switches your home");
    }
    let host = match host(paths) {
        Ok(host) => host,
        Err(code) => return code,
    };
    let queue = Queue::new(paths.jobs());
    if mode.foreground {
        let id = match queue.enqueue() {
            Ok(id) => id,
            Err(e) => return report(e),
        };
        let worker = {
            let queue = queue.clone();
            thread::spawn(move || queue.work(&host))
        };
        let code = follow(&queue, &id, None);
        return match worker.join().expect("the switch worker thread panicked") {
            Ok(_) => code,
            Err(e) => report(e),
        };
    }
    let id = match queue.submit(&host) {
        Ok(id) => id,
        Err(e) => return report(e),
    };
    if mode.detach {
        println!("switch queued as job {id}: kubaos job follow {id}");
        return ExitCode::SUCCESS;
    }
    println!("switch queued as job {id}; Ctrl-C stops following, not the switch");
    follow(&queue, &id, Some(&host))
}

fn host(paths: &Paths) -> Result<Systemd, ExitCode> {
    match std::env::current_exe() {
        Ok(exe) => Ok(Systemd::new(exe, paths.flake())),
        Err(e) => Err(report(e)),
    }
}

/// Prints the job's output until it ends. With `host`, the job runs in a worker unit, and fails
/// if that unit dies before running it.
fn follow(queue: &Queue, id: &JobId, host: Option<&Systemd>) -> ExitCode {
    match wait(queue, id, host) {
        Ok(()) => {
            println!("switch {id} done");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("kubaos: switch {id} failed: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Prints the job's output until it ends; why it failed, when it did.
fn wait(queue: &Queue, id: &JobId, host: Option<&Systemd>) -> Result<(), String> {
    let mut out = io::stdout();
    let print = |chunk: &str| {
        let _ = out.write_all(chunk.as_bytes()).and_then(|()| out.flush());
    };
    let job = match host {
        Some(host) => queue.follow_worker(id, FOLLOW_POLL, host, print),
        None => queue.follow(id, FOLLOW_POLL, print),
    };
    match job {
        Ok(Job {
            state: JobState::Succeeded,
            ..
        }) => Ok(()),
        Ok(job) => Err(job.error.unwrap_or_else(|| "no reason recorded".to_owned())),
        Err(e) => Err(chain(&e)),
    }
}

/// Where a switch runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwitchIn {
    /// A transient systemd user unit.
    Unit,
    /// This process, for a session without a systemd user manager.
    Process,
}

/// Runs one switch of the user flake to its end, for a command that switches as one of its
/// steps.
///
/// # Errors
///
/// Why the switch failed.
pub fn switch_step(paths: &Paths, place: SwitchIn) -> Result<(), String> {
    let host = std::env::current_exe()
        .map(|exe| Systemd::new(exe, paths.flake()))
        .map_err(|e| chain(&e))?;
    let queue = Queue::new(paths.jobs());
    if place == SwitchIn::Process {
        let id = queue.enqueue().map_err(|e| chain(&e))?;
        let worker = {
            let queue = queue.clone();
            thread::spawn(move || queue.work(&host))
        };
        let waited = wait(&queue, &id, None);
        worker
            .join()
            .expect("the switch worker thread panicked")
            .map_err(|e| chain(&e))?;
        return waited;
    }
    let id = queue.submit(&host).map_err(|e| chain(&e))?;
    wait(&queue, &id, Some(&host))
}

fn list(queue: &Queue, host: &Systemd, json: bool) -> ExitCode {
    if let Err(e) = queue.reap(host) {
        eprintln!("kubaos: not checking the worker units: {e}");
    }
    let jobs = match queue.jobs() {
        Ok(jobs) => jobs,
        Err(e) => return report(e),
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&jobs).expect("jobs serialize"));
        return ExitCode::SUCCESS;
    }
    for job in jobs {
        let state = format!("{:?}", job.state).to_lowercase();
        match job.error {
            Some(error) => println!("{}  {state}  {error}", job.id),
            None => println!("{}  {state}", job.id),
        }
    }
    ExitCode::SUCCESS
}

/// Prints `e` with its causes on stderr and returns exit code 1.
pub fn report(e: impl std::error::Error) -> ExitCode {
    eprintln!("kubaos: {}", chain(&e));
    ExitCode::FAILURE
}

/// `e` and its causes, on one line.
pub fn chain(e: &dyn std::error::Error) -> String {
    let mut text = e.to_string();
    let mut source = e.source();
    while let Some(cause) = source {
        text.push_str(": ");
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}
