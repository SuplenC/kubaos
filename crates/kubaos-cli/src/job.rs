//! `kubaos switch` and `kubaos job`: queueing a home-manager switch of the user flake and following
//! it.
//!
//!   kubaos switch [--from PATH|BRANCH | --release] [--detach | --foreground]
//!   kubaos job list [--json]
//!   kubaos job follow ID
//!   kubaos job work
//!
//! A switch runs in a transient systemd user unit, one at a time; the command follows its output
//! until it ends, and Ctrl-C stops only the following. Following a job, and listing the jobs, fail
//! a queued job whose unit died before it ran. `--foreground` runs the queue in this
//! process instead, for a session without a systemd user manager. `--from` switches the home to a
//! KubaOS checkout for a live preview, once it evaluates; `--release` goes back to the release; a
//! plain `kubaos switch` during a preview is refused, naming both (`kubaos_core::preview`). Exit
//! codes: 0 ok, 1 the switch failed or the checkout doesn't evaluate, 2 usage.
//!
//! # Owns
//!
//! - The command line of `switch` and `job`, and printing a job's output.
//! - The live preview as the updater sees it ([`active_preview`], [`leave_preview`]).
//!
//! # Not here
//!
//! - The queue, the lock and the systemd unit: `kubaos_core::switch`.
//! - The preview's guards and record: `kubaos_core::preview`.
//! - The `package` commands, which end in a switch: `package.rs`.

use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use clap::Args;
use clap::Subcommand;
use kubaos_core::preview;
use kubaos_core::preview::Notice;
use kubaos_core::preview::PlanError;
use kubaos_core::preview::Preview;
use kubaos_core::preview::PreviewFile;
use kubaos_core::preview::Request;
use kubaos_core::state::Paths;
use kubaos_core::switch::Job;
use kubaos_core::switch::JobId;
use kubaos_core::switch::JobState;
use kubaos_core::switch::Queue;
use kubaos_core::switch::Systemd;
use kubaos_core::switch::Target;

use crate::out::err;
use crate::out::out;
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

/// What `kubaos switch` switches the home to: without a flag, the home as it stands.
#[derive(Args, Clone, Debug)]
#[group(multiple = false)]
pub struct SwitchTo {
    /// Preview a KubaOS checkout: a folder, or a branch of a worktree of ~/Projects/kubaos. Only
    /// what the home builds changes; the system stays the release's.
    #[arg(long, value_name = "PATH|BRANCH")]
    from: Option<String>,
    /// Go back from a preview to the release the machine is on: the user flake as locked.
    #[arg(long)]
    release: bool,
}

/// The switch jobs' queue, keeping the preview file in step with them.
fn queue(paths: &Paths) -> Queue {
    Queue::new(paths.jobs()).previews(PreviewFile::new(paths.preview()))
}

pub fn main(cmd: JobCmd) -> ExitCode {
    let paths = match Paths::from_env() {
        Ok(paths) => paths,
        Err(e) => return usage_error(e),
    };
    let queue = queue(&paths);
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

/// `kubaos switch`: the switch `to` asks for, checked against the preview in effect, queued and
/// waited for as `mode` says.
pub fn switch_to(paths: &Paths, mode: SwitchMode, to: &SwitchTo) -> ExitCode {
    if rustix::process::geteuid().is_root() {
        return usage_error("run kubaos switch as yourself, not root: it switches your home");
    }
    let host = match host(paths) {
        Ok(host) => host,
        Err(code) => return code,
    };
    let active = match PreviewFile::new(paths.preview()).read() {
        Ok(active) => active,
        // Only a plain switch needs the record: the release and a new preview replace it.
        Err(e) if to.release || to.from.is_some() => {
            err!("kubaos: {}", chain(&e));
            None
        }
        Err(e) => return report(e),
    };
    // A switch to the release still queued leaves the preview before this one runs.
    let active = active.filter(|_| !releasing(&queue(paths)));
    let request = match (&to.from, to.release) {
        (Some(arg), _) => match resolve(paths, arg) {
            Ok(path) => {
                out!("checking that your home evaluates with {} as KubaOS", path.display());
                Request::From(path)
            }
            Err(e) => return usage_error(e),
        },
        (None, true) => Request::Release,
        (None, false) => Request::Plain,
    };
    let plan = match preview::plan(request, active.as_ref(), &host) {
        Ok(plan) => plan,
        Err(e @ PlanError::Active { .. }) => return usage_error(e),
        Err(e) => return report(e),
    };
    match &plan.notice {
        Some(Notice::Replaces(old)) => out!("this preview replaces the one of {}", describe(old)),
        Some(Notice::Leaves(old)) => out!("leaving the preview of {}: back to the release", describe(old)),
        Some(Notice::NoPreview) => out!("no preview is on: switching to the release as locked"),
        None => {}
    }
    if let Target::Checkout { path } = &plan.target {
        out!(
            "previewing {}: only what your home builds changes; `kubaos switch --release` goes back",
            path.display()
        );
    }
    run(paths, &host, mode, plan.target)
}

/// A switch of the home as it stands, for a command that changed what it declares (a package, a
/// Suite): during a preview, it builds the preview again, and says so.
pub fn switch(paths: &Paths, mode: SwitchMode) -> ExitCode {
    if rustix::process::geteuid().is_root() {
        return usage_error("run kubaos switch as yourself, not root: it switches your home");
    }
    let host = match host(paths) {
        Ok(host) => host,
        Err(code) => return code,
    };
    match PreviewFile::new(paths.preview()).read() {
        Ok(Some(active)) => out!(
            "the switch keeps the preview of {}; `kubaos switch --release` goes back to the release",
            describe(&active)
        ),
        Ok(None) => {}
        Err(e) => return report(e),
    }
    run(paths, &host, mode, Target::Current)
}

/// Whether a switch to the release is queued or running.
fn releasing(queue: &Queue) -> bool {
    queue.jobs().is_ok_and(|jobs| {
        jobs.iter()
            .any(|job| !job.state.is_done() && job.target == Target::Release)
    })
}

/// The checkout `--from <arg>` names: a folder, or a branch of the main checkout's worktrees.
fn resolve(paths: &Paths, arg: &str) -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("finding the current folder: {e}"))?;
    let repo = paths.projects_root().join(preview::MAIN_CHECKOUT);
    preview::resolve(arg, &cwd, &repo, Systemd::worktrees).map_err(|e| chain(&e))
}

/// A preview as a line names it: its branch, its commit, and the folder.
fn describe(preview: &Preview) -> String {
    let changes = if preview.dirty { ", uncommitted changes" } else { "" };
    format!(
        "{} (at {}{changes}, {})",
        preview.name(),
        preview.short_commit(),
        preview.path.display()
    )
}

/// Queues a switch to `target` and waits for it as `mode` says.
fn run(paths: &Paths, host: &Systemd, mode: SwitchMode, target: Target) -> ExitCode {
    let queue = queue(paths);
    if mode.foreground {
        let host = host.clone();
        let id = match queue.enqueue_to(target) {
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
    let id = match queue.submit_to(host, target) {
        Ok(id) => id,
        Err(e) => return report(e),
    };
    if mode.detach {
        out!("switch queued as job {id}: kubaos job follow {id}");
        return ExitCode::SUCCESS;
    }
    out!("switch queued as job {id}; Ctrl-C stops following, not the switch");
    follow(&queue, &id, Some(host))
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
    let mut out = io::stdout();
    let print = |chunk: &str| {
        let _ = out.write_all(chunk.as_bytes()).and_then(|()| out.flush());
    };
    match wait(queue, id, host, print) {
        Ok(()) => {
            out!("switch {id} done");
            ExitCode::SUCCESS
        }
        Err(e) => {
            err!("kubaos: switch {id} failed: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Hands the job's output to `on_output` until it ends; why it failed, when it did.
fn wait(queue: &Queue, id: &JobId, host: Option<&Systemd>, on_output: impl FnMut(&str)) -> Result<(), String> {
    let job = match host {
        Some(host) => queue.follow_worker(id, FOLLOW_POLL, host, on_output),
        None => queue.follow(id, FOLLOW_POLL, on_output),
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

/// Runs one switch of the user flake as locked (the release, so it ends any preview) to its end,
/// for a command that switches as one of its steps (an update), handing each chunk of its output
/// to `on_output`.
///
/// # Errors
///
/// Why the switch failed.
pub fn switch_step(paths: &Paths, place: SwitchIn, on_output: &mut dyn FnMut(&str)) -> Result<(), String> {
    let host = std::env::current_exe()
        .map(|exe| Systemd::new(exe, paths.flake()))
        .map_err(|e| chain(&e))?;
    let queue = queue(paths);
    if place == SwitchIn::Process {
        let id = queue.enqueue_to(Target::Release).map_err(|e| chain(&e))?;
        let worker = {
            let queue = queue.clone();
            thread::spawn(move || queue.work(&host))
        };
        let waited = wait(&queue, &id, None, on_output);
        worker
            .join()
            .expect("the switch worker thread panicked")
            .map_err(|e| chain(&e))?;
        return waited;
    }
    let id = queue.submit_to(&host, Target::Release).map_err(|e| chain(&e))?;
    wait(&queue, &id, Some(&host), on_output)
}

/// The live preview the home is on, as a line names it, for the updater. A preview file that
/// doesn't read counts as one: the switch back to the release empties it.
pub fn active_preview(paths: &Paths) -> Option<String> {
    match PreviewFile::new(paths.preview()).read() {
        Ok(active) => active.as_ref().map(describe),
        Err(_) => Some(format!("{} (which doesn't read)", paths.preview().display())),
    }
}

/// Switches the home from its live preview back to the release, for the updater before it
/// changes anything, handing each chunk of the switch's output to `on_output`.
///
/// # Errors
///
/// Why the preview could not be left.
pub fn leave_preview(paths: &Paths, place: SwitchIn, on_output: &mut dyn FnMut(&str)) -> Result<(), String> {
    let file = PreviewFile::new(paths.preview());
    preview::leave(&file, || switch_step(paths, place, on_output))
        .map(|_| ())
        .map_err(|e| chain(&e))
}

fn list(queue: &Queue, host: &Systemd, json: bool) -> ExitCode {
    if let Err(e) = queue.reap(host) {
        err!("kubaos: not checking the worker units: {e}");
    }
    let jobs = match queue.jobs() {
        Ok(jobs) => jobs,
        Err(e) => return report(e),
    };
    if json {
        out!("{}", serde_json::to_string_pretty(&jobs).expect("jobs serialize"));
        return ExitCode::SUCCESS;
    }
    for job in jobs {
        let state = format!("{:?}", job.state).to_lowercase();
        match job.error {
            Some(error) => out!("{}  {state}  {error}", job.id),
            None => out!("{}  {state}", job.id),
        }
    }
    ExitCode::SUCCESS
}

/// Prints `e` with its causes on stderr and returns exit code 1.
pub fn report(e: impl std::error::Error) -> ExitCode {
    err!("kubaos: {}", chain(&e));
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
