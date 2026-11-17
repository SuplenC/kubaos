//! Switch jobs: every `home-manager switch` the KubaOS CLI, the Command menu or Kōbō asks for, run
//! one at a time from a queue, in a transient systemd user unit so that closing the front end does
//! not stop it.
//!
//! A job is a file in the jobs folder, `<id>.json`, with its output in `<id>.log`. Submitting a job
//! writes the file and starts a worker unit for it. A worker takes the one switch lock, then runs
//! every queued job, oldest first, until none is left. Each job's own worker still starts, and
//! finds nothing to do when an earlier worker already ran its job. A front end follows a job by
//! reading its two files.
//!
//! A job records what it switches to ([`Target`]): the home as it stands, the release, or a KubaOS
//! checkout for a live preview (`preview.rs`), whose `kubaos` input the switch overrides. A job
//! that switches to a checkout records the preview when it succeeds, and one to the release clears
//! it, both under the switch lock.
//!
//! Before it switches, a job builds the packages the state directory lists, as the switch will see
//! them, and holds each one that fails at the version the home has (`held.rs`), so the rest of the
//! home still switches. The job records the packages it held, and its log names each.
//!
//! A job records the name of its worker unit. A job still queued whose unit is gone, while no
//! worker holds the lock, will never run (its program failed to start, say), and [`Queue::reap`]
//! fails it; following a job and listing the jobs both reap.
//!
//! ```text
//! queued ──► running ──► succeeded
//!    │          └──────► failed (the switch failed, or its worker died)
//!    └─────────────────► failed (the worker unit did not start, or died before taking the job)
//! ```
//!
//! # Owns
//!
//! - The job files, the queue order and the switch lock.
//! - What each job switches to, and keeping the preview file in step with the switches that ran.
//! - The pre-build before each switch, and keeping `held.toml` in step with it.
//! - Following a job's output until it ends.
//! - [`Host`], the seam to systemd and home-manager, and [`Systemd`], the real one.
//!
//! # Not here
//!
//! - What a switch applies: the user flake and its state directory (`state.rs`).
//! - Which packages a pre-build holds, and `held.toml`'s format: `held.rs`.
//! - Which switch a `kubaos switch` asks for, and the preview record: `preview.rs`.
//! - The `kubaos switch` and `kubaos job` commands: `kubaos-cli`.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::fs::File;
use std::fs::TryLockError;
use std::io;
use std::io::BufRead;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::Command;
use std::process::Stdio;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use serde::Deserialize;
use serde::Serialize;

use crate::held;
use crate::held::CurrentHome;
use crate::held::Evaluated;
use crate::held::Found;
use crate::held::HeldFile;
use crate::held::Holds;
use crate::held::Outcome;
use crate::held::Prebuilt;
use crate::held::Retry;
use crate::preview;
use crate::preview::Checkout;
use crate::preview::Preview;
use crate::preview::PreviewFile;
use crate::state::GENERATION_RECORD;
use crate::state::lock_file;
use crate::state::write_atomic;

/// The environment variables a worker unit gets from the process that submits it. The user
/// manager's own environment lacks the login shell's `PATH`, where `home-manager` and `nix` are.
pub const WORKER_ENV: [&str; 6] = [
    "PATH",
    "XDG_CONFIG_HOME",
    "XDG_STATE_HOME",
    "NIX_PATH",
    "LANG",
    "LOCALE_ARCHIVE",
];

/// The name of one job, sortable by when it was queued: `<nanoseconds>-<pid>-<counter>`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct JobId(String);

/// A job name that is not one: it must be digits and `-`, so it is always a plain file name.
#[derive(Debug, thiserror::Error)]
#[error("{0:?} is not a job id")]
pub struct InvalidJobId(String);

impl JobId {
    /// Checks `id` and makes it a job id.
    ///
    /// # Errors
    ///
    /// [`InvalidJobId`] unless `id` is non-empty, digits and `-`.
    pub fn parse(id: impl Into<String>) -> Result<Self, InvalidJobId> {
        let id = id.into();
        let valid = !id.is_empty() && id.chars().all(|c| c.is_ascii_digit() || c == '-');
        if valid { Ok(Self(id)) } else { Err(InvalidJobId(id)) }
    }

    /// The id as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// A new id, later than every id this process made before.
    fn next() -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(format!("{nanos:020}-{:07}-{n:04}", process::id()))
    }
}

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for JobId {
    type Error = InvalidJobId;

    fn try_from(id: String) -> Result<Self, Self::Error> {
        Self::parse(id)
    }
}

impl From<JobId> for String {
    fn from(id: JobId) -> Self {
        id.0
    }
}

/// Where a job is in its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Queued,
    Running,
    Succeeded,
    Failed,
}

impl JobState {
    /// Whether the job has ended and will not change again.
    pub fn is_done(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed)
    }
}

/// What a switch job builds the home from.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Target {
    /// The home as it stands when the job runs: the preview in effect then, else the user flake as
    /// locked. What every switch that applies a change of the state directory asks for, so a
    /// package added during a preview keeps it.
    #[default]
    Current,
    /// The user flake as locked: the release the machine is on. Clears the preview.
    Release,
    /// The user flake with its `kubaos` input overridden to the checkout at `path`: a preview.
    Checkout { path: PathBuf },
}

/// One switch job, as `<id>.json` holds it. Times are milliseconds since the Unix epoch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub state: JobState,
    pub queued_at: u64,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
    /// Why a failed job failed, for a front end to show.
    pub error: Option<String>,
    /// The systemd user unit that runs it, or `None` when it runs in the process that queued it.
    #[serde(default)]
    pub unit: Option<String>,
    /// What it switches to.
    #[serde(default)]
    pub target: Target,
    /// The listed packages it installed at the version the home had, since they couldn't be
    /// built (`held.rs`), by name.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub held: Vec<String>,
    /// Whether its pre-build builds a held package whose derivation hasn't changed since it
    /// failed.
    #[serde(default)]
    pub retry_held: Retry,
}

/// What the queue needs from the machine: starting a worker unit, the switch itself, and reading
/// and checking a checkout to preview.
pub trait Host {
    /// The name of the unit [`Host::start_worker`] starts for `job`.
    fn worker_unit(&self, job: &JobId) -> String;

    /// Starts a worker for `job`, which runs [`Queue::work`] in a process of its own.
    ///
    /// # Errors
    ///
    /// [`HostError`] when the worker could not be started.
    fn start_worker(&self, job: &JobId) -> Result<(), HostError>;

    /// Whether the worker unit `unit` is still starting or running.
    ///
    /// # Errors
    ///
    /// [`HostError`] when its state could not be read.
    fn worker_alive(&self, unit: &str) -> Result<bool, HostError>;

    /// Runs one switch to the end, its output going to `log`: of the user flake as locked, or with
    /// its `kubaos` input overridden to the checkout `from`.
    ///
    /// # Errors
    ///
    /// [`HostError`] when the switch could not start or failed.
    fn switch(&self, log: File, from: Option<&Path>) -> Result<(), HostError>;

    /// The branch and commit of the checkout at `path`, and whether it has uncommitted changes.
    ///
    /// # Errors
    ///
    /// [`HostError`] when `path` is no git checkout.
    fn describe(&self, path: &Path) -> Result<Checkout, HostError>;

    /// Evaluates the user flake's home with its `kubaos` input overridden to `path`, building
    /// nothing, so a checkout that would fail the switch is refused before it starts.
    ///
    /// # Errors
    ///
    /// [`HostError`] when the evaluation fails.
    fn evaluate(&self, path: &Path) -> Result<(), HostError>;

    /// Evaluates every package the state directory lists, as the switch from `from` will see it:
    /// in the home configuration `home-manager switch` picks, Nix's messages going to `log`.
    ///
    /// # Errors
    ///
    /// [`HostError`] when the packages could not be evaluated.
    fn listed(&self, log: &File, from: Option<&Path>) -> Result<Vec<Evaluated>, HostError>;

    /// Builds `installables` as far as each builds, the build's output going to `log`, and
    /// returns the derivations Nix said failed ([`held::failed_derivations`]).
    ///
    /// # Errors
    ///
    /// [`HostError`] when Nix could not run or was killed. A package that fails to build is no
    /// error.
    fn build(&self, log: &File, installables: &[String]) -> Result<BTreeSet<String>, HostError>;

    /// Which of `paths` are missing from the store.
    ///
    /// # Errors
    ///
    /// [`HostError`] when the store could not be asked.
    fn missing(&self, paths: &[String]) -> Result<BTreeSet<String>, HostError>;

    /// The listed packages the home has now, from the home-manager generation it is on.
    ///
    /// # Errors
    ///
    /// [`HostError`] when the generation's record or its packages could not be read.
    fn current_home(&self) -> Result<CurrentHome, HostError>;
}

/// A program the [`Host`] ran failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HostError {
    /// It could not be started.
    #[error("starting {program}")]
    Spawn {
        program: String,
        #[source]
        source: io::Error,
    },
    /// It ran and exited with an error, or was killed (no code).
    #[error("{program} exited with {}", code.map_or_else(|| "a signal".to_owned(), |c| format!("code {c}")))]
    Exited { program: String, code: Option<i32> },
    /// It printed something other than the answer it was asked for.
    #[error("reading what {program} printed")]
    Answer {
        program: String,
        #[source]
        source: serde_json::Error,
    },
    /// A file it reads could not be.
    #[error("reading {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

impl HostError {
    /// `program` could not be started.
    #[must_use]
    pub fn spawn(program: impl Into<String>, source: io::Error) -> Self {
        Self::Spawn {
            program: program.into(),
            source,
        }
    }

    /// `program` exited with `code`, or was killed.
    #[must_use]
    pub fn exited(program: impl Into<String>, code: Option<i32>) -> Self {
        Self::Exited {
            program: program.into(),
            code,
        }
    }
}

/// The queue's files could not be used, or a worker could not be started.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum QueueError {
    /// A job file, the log or the lock could not be read or written.
    #[error("using {}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// A job file is not a job.
    #[error("parsing {}", path.display())]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    /// No job has this id.
    #[error("no job {0}")]
    NoSuchJob(JobId),
    /// The job was queued, but its worker did not start; the job is marked failed.
    #[error("starting the worker for job {job}")]
    Start {
        job: JobId,
        #[source]
        source: HostError,
    },
    /// Whether the job's worker unit is alive could not be found out.
    #[error("checking the worker of job {job}")]
    Check {
        job: JobId,
        #[source]
        source: HostError,
    },
}

impl QueueError {
    fn io(path: &Path) -> impl FnOnce(io::Error) -> Self {
        let path = path.to_path_buf();
        move |source| Self::Io { path, source }
    }
}

/// The jobs folder: the queue of switch jobs and their lock.
#[derive(Clone, Debug)]
pub struct Queue {
    dir: PathBuf,
    /// The preview file the jobs keep in step; without one, previews aren't recorded and
    /// [`Target::Current`] is the user flake as locked.
    previews: Option<PreviewFile>,
    /// The hold file the jobs keep in step with their pre-builds; without one, a job builds
    /// nothing before its switch and holds nothing.
    held: Option<HeldFile>,
    /// What the jobs it queues retry of the held packages.
    retry: Retry,
}

impl Queue {
    /// The queue in `dir` (`Paths::jobs`). The folder is made on first use.
    #[must_use]
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            previews: None,
            held: None,
            retry: Retry::Changed,
        }
    }

    /// The same queue, recording previews in `file` (`Paths::preview`).
    #[must_use]
    pub fn previews(mut self, file: PreviewFile) -> Self {
        self.previews = Some(file);
        self
    }

    /// The same queue, building the listed packages before each switch and holding those that
    /// fail in `file` (`held.rs`).
    #[must_use]
    pub fn holding(mut self, file: HeldFile) -> Self {
        self.held = Some(file);
        self
    }

    /// The same queue, whose jobs build every held package again, as an update's switch does.
    #[must_use]
    pub fn retrying_held(mut self) -> Self {
        self.retry = Retry::All;
        self
    }

    /// Queues a switch job of the home as it stands, without starting a worker for it.
    ///
    /// # Errors
    ///
    /// [`QueueError::Io`] when the job file cannot be written.
    pub fn enqueue(&self) -> Result<JobId, QueueError> {
        self.enqueue_to(Target::Current)
    }

    /// Queues a switch job to `target`, without starting a worker for it.
    ///
    /// # Errors
    ///
    /// [`QueueError::Io`] when the job file cannot be written.
    pub fn enqueue_to(&self, target: Target) -> Result<JobId, QueueError> {
        self.enqueue_with(target, |_| None)
    }

    /// Queues a job to `target` whose worker unit, if any, `unit` names from its id.
    fn enqueue_with(&self, target: Target, unit: impl FnOnce(&JobId) -> Option<String>) -> Result<JobId, QueueError> {
        fs::create_dir_all(&self.dir).map_err(QueueError::io(&self.dir))?;
        let id = JobId::next();
        let job = Job {
            unit: unit(&id),
            id: id.clone(),
            state: JobState::Queued,
            queued_at: now_ms(),
            started_at: None,
            finished_at: None,
            error: None,
            target,
            held: Vec::new(),
            retry_held: self.retry,
        };
        self.write_job(&job)?;
        Ok(id)
    }

    /// Queues a switch job of the home as it stands and starts its worker through `host`.
    ///
    /// # Errors
    ///
    /// [`QueueError::Start`] when the worker does not start, after marking the job failed, and
    /// [`QueueError::Io`] when the job file cannot be written.
    pub fn submit(&self, host: &impl Host) -> Result<JobId, QueueError> {
        self.submit_to(host, Target::Current)
    }

    /// Queues a switch job to `target` and starts its worker through `host`.
    ///
    /// # Errors
    ///
    /// [`QueueError::Start`] when the worker does not start, after marking the job failed, and
    /// [`QueueError::Io`] when the job file cannot be written.
    pub fn submit_to(&self, host: &impl Host, target: Target) -> Result<JobId, QueueError> {
        let id = self.enqueue_with(target, |id| Some(host.worker_unit(id)))?;
        if let Err(source) = host.start_worker(&id) {
            let mut job = self.job(&id)?;
            job.state = JobState::Failed;
            job.finished_at = Some(now_ms());
            job.error = Some(error_chain(&source));
            self.write_job(&job)?;
            return Err(QueueError::Start { job: id, source });
        }
        Ok(id)
    }

    /// Takes the switch lock, waiting for any other worker, then runs every queued job oldest first
    /// until none is left. Returns the jobs it ran, in order.
    ///
    /// A job found running while this worker holds the lock lost its worker mid-switch, and is
    /// marked failed.
    ///
    /// # Errors
    ///
    /// [`QueueError`] when the lock or a job file cannot be used. A failed switch is not an error
    /// here: it fails its job, and the next job still runs.
    pub fn work(&self, host: &impl Host) -> Result<Vec<JobId>, QueueError> {
        let _lock = self.lock()?;
        let mut ran = Vec::new();
        for mut job in self.jobs()? {
            if job.state == JobState::Running {
                job.state = JobState::Failed;
                job.finished_at = Some(now_ms());
                job.error = Some("interrupted: its worker stopped during the switch".to_owned());
                self.write_job(&job)?;
            }
        }
        while let Some(mut job) = self.jobs()?.into_iter().find(|j| j.state == JobState::Queued) {
            job.state = JobState::Running;
            job.started_at = Some(now_ms());
            self.write_job(&job)?;
            let log_path = self.log_path(&job.id);
            let log = File::options()
                .create(true)
                .append(true)
                .open(&log_path)
                .map_err(QueueError::io(&log_path))?;
            let result = self.run_job(host, &mut job, log);
            job.finished_at = Some(now_ms());
            match result {
                Ok(()) => job.state = JobState::Succeeded,
                Err(e) => {
                    job.state = JobState::Failed;
                    job.error = Some(e);
                }
            }
            self.write_job(&job)?;
            ran.push(job.id);
        }
        Ok(ran)
    }

    /// Runs one job's pre-build and switch, records the packages it held on `job`, and records the
    /// preview it makes or clears when it succeeds. The caller holds the switch lock, so the
    /// preview file has one writer at a time.
    fn run_job(&self, host: &impl Host, job: &mut Job, log: File) -> Result<(), String> {
        let target = job.target.clone();
        let retry = job.retry_held;
        // Only the home as it stands depends on the record: the release empties it whatever it
        // holds, and a checkout replaces it, so a record that doesn't read never blocks the way
        // out.
        let active = match (&self.previews, &target) {
            (None, _) | (_, Target::Release) => None,
            (Some(file), Target::Checkout { .. }) => file.read().ok().flatten(),
            (Some(file), Target::Current) => file.read().map_err(|e| error_chain(&e))?,
        };
        let from = match &target {
            Target::Release => None,
            Target::Checkout { path } => Some(path.clone()),
            Target::Current => active.as_ref().map(|preview| preview.path.clone()),
        };
        if let (Target::Current, Some(gone)) = (&target, from.as_ref().filter(|path| !path.is_dir())) {
            return Err(format!(
                "the preview's checkout {} is gone: `kubaos switch --release` goes back to the release",
                gone.display()
            ));
        }
        let Some(from) = from else {
            let holding = self.hold(host, &log, None, retry);
            let unkept = holding.unkept_error();
            job.held = holding.held;
            unkept?;
            host.switch(log, None).map_err(|e| error_chain(&e))?;
            if let Some(file) = &self.previews
                && target == Target::Release
            {
                file.clear().map_err(|e| error_chain(&e))?;
            }
            return Ok(());
        };
        let checkout = host
            .describe(&from)
            .map_err(|e| format!("reading the checkout {}: {}", from.display(), error_chain(&e)))?;
        let holding = self.hold(host, &log, Some(&from), retry);
        let unkept = holding.unkept_error();
        job.held = holding.held;
        unkept?;
        host.switch(log, Some(&from)).map_err(|e| error_chain(&e))?;
        let Some(file) = &self.previews else {
            return Ok(());
        };
        let since = active
            .filter(|preview| preview.path == from)
            .map_or_else(now_ms, |preview| preview.since);
        let preview = Preview {
            path: from,
            branch: checkout.branch,
            commit: checkout.commit,
            dirty: checkout.dirty,
            since,
        };
        file.write(&preview).map_err(|e| error_chain(&e))
    }

    /// Builds the listed packages as the switch from `from` will see them, and holds each that
    /// failed at the version the home has, writing `held.toml`; each step and each hold is a line
    /// of `log`. A held package whose derivation is still the one that failed isn't built again
    /// unless `retry` is [`Retry::All`]. Returns the packages the switch installs held, and those
    /// that failed with no version to keep.
    ///
    /// When the packages don't evaluate, the build stops for another reason than a package or the
    /// file can't be written, the holds stay as they were, and the switch goes on to succeed or
    /// fail as it would.
    fn hold(&self, host: &impl Host, log: &File, from: Option<&Path>, retry: Retry) -> Holding {
        let Some(file) = &self.held else {
            return Holding::default();
        };
        let before = file.read().unwrap_or_else(|e| {
            note(log, &format!("not reading the held packages: {}", error_chain(&e)));
            Holds::new()
        });
        let kept = |before: Holds| Holding {
            held: before.into_keys().collect(),
            unkept: Vec::new(),
        };
        let listed = file.state().declared().map_or(true, |declared| !declared.is_empty());
        if !listed && before.is_empty() {
            return Holding::default();
        }
        note(log, "building the packages kubaos.d lists before the switch");
        let prebuilt = match Self::prebuild(host, log, from, retry, &before) {
            Ok(prebuilt) => prebuilt,
            Err(e) => {
                note(log, &format!("no hold changes: {e}"));
                return kept(before);
            }
        };
        let current = host.current_home().unwrap_or_else(|e| {
            note(log, &format!("not reading what your home has: {}", error_chain(&e)));
            CurrentHome::Nothing
        });
        let decision = held::decide(&prebuilt, &current, &before);
        for name in &decision.unkept {
            note(log, &held::unkept_message(name));
        }
        for (name, hold) in &decision.held {
            note(log, &format!("{} (kept at {})", held::message(name), hold.path));
        }
        for package in prebuilt
            .iter()
            .filter(|p| p.outcome == Outcome::Built && before.contains_key(&p.name))
        {
            note(log, &format!("{} builds again: it is no longer held", package.name));
        }
        if let Err(e) = file.write(&decision.held) {
            note(log, &format!("the holds stay as they were: {}", error_chain(&e)));
            return Holding {
                unkept: decision.unkept,
                ..kept(before)
            };
        }
        Holding {
            held: decision.held.into_keys().collect(),
            unkept: decision.unkept,
        }
    }

    /// Evaluates the listed packages and builds them, leaving out the held ones `before` names
    /// whose derivation hasn't changed since it failed and whose outputs are still missing, unless
    /// `retry` is [`Retry::All`].
    fn prebuild(
        host: &impl Host,
        log: &File,
        from: Option<&Path>,
        retry: Retry,
        before: &Holds,
    ) -> Result<Vec<Prebuilt>, String> {
        let chain = |e: HostError| error_chain(&e);
        let evaluated = host
            .listed(log, from)
            .map_err(|e| format!("the packages kubaos.d lists don't evaluate: {}", chain(e)))?;
        let outputs_of = |names: &dyn Fn(&str) -> bool| -> Vec<&Found> {
            evaluated
                .iter()
                .filter(|one| names(&one.name))
                .filter_map(|one| one.found.as_ref())
                .collect()
        };
        let unchanged = match retry {
            Retry::All => BTreeSet::new(),
            Retry::Changed => held::unchanged(&evaluated, before),
        };
        let mut build = held::Build::default();
        let unchanged_found = outputs_of(&|name| unchanged.contains(name));
        if !unchanged_found.is_empty() {
            build.missing = host.missing(&outputs(&unchanged_found)).map_err(chain)?;
        }
        // An unchanged hold whose outputs turned up in the store (a cache, a build by hand) needs
        // no build: the check finds it built.
        let skipped: BTreeSet<String> = evaluated
            .iter()
            .filter(|one| unchanged.contains(&one.name))
            .filter(|one| {
                one.found
                    .as_ref()
                    .is_some_and(|found| found.outputs.values().any(|path| build.missing.contains(path)))
            })
            .map(|one| one.name.clone())
            .collect();
        for name in &skipped {
            note(
                log,
                &format!(
                    "not building {name} again: its build is the one that failed; the next update tries it, as \
                     `kubaos switch --retry-held` does"
                ),
            );
        }
        let to_build = outputs_of(&|name| !unchanged.contains(name));
        if !to_build.is_empty() {
            let installables: Vec<String> = to_build.iter().map(|found| found.installable()).collect();
            build.failed = host.build(log, &installables).map_err(chain)?;
            build.missing.extend(host.missing(&outputs(&to_build)).map_err(chain)?);
        }
        held::prebuilt(&evaluated, &build, &skipped).map_err(|e| e.to_string())
    }

    /// Fails every queued job that will never run: its worker unit is gone, and no other worker
    /// holds the lock to run it. Returns the jobs it failed. A job without a unit runs in the
    /// process that queued it, and is left alone.
    ///
    /// # Errors
    ///
    /// [`QueueError`] when a job file or the lock cannot be used, or a unit's state cannot be read.
    pub fn reap(&self, host: &impl Host) -> Result<Vec<JobId>, QueueError> {
        let mut reaped = Vec::new();
        for job in self.jobs()? {
            if self.reap_job(host, &job)? {
                reaped.push(job.id);
            }
        }
        Ok(reaped)
    }

    /// Fails `job` if it is queued, its unit is gone and the lock is free; returns whether it did.
    fn reap_job(&self, host: &impl Host, job: &Job) -> Result<bool, QueueError> {
        let Some(unit) = job.unit.as_deref() else {
            return Ok(false);
        };
        if job.state != JobState::Queued {
            return Ok(false);
        }
        let alive = host.worker_alive(unit).map_err(|source| QueueError::Check {
            job: job.id.clone(),
            source,
        })?;
        if alive {
            return Ok(false);
        }
        // HAZARD: re-read the job under the lock. Its unit may have run it between the first read
        // and the unit's check, and failing it then would overwrite a finished job.
        let Some(_lock) = self.try_lock()? else {
            return Ok(false);
        };
        let mut job = self.job(&job.id)?;
        if job.state != JobState::Queued {
            return Ok(false);
        }
        job.state = JobState::Failed;
        job.finished_at = Some(now_ms());
        job.error = Some(format!(
            "its worker unit {unit} stopped without running it (journalctl --user -u {unit})"
        ));
        self.write_job(&job)?;
        Ok(true)
    }

    /// One job.
    ///
    /// # Errors
    ///
    /// [`QueueError::NoSuchJob`] when there is none with this id, else [`QueueError`] when its
    /// file cannot be read.
    pub fn job(&self, id: &JobId) -> Result<Job, QueueError> {
        let path = self.job_path(id);
        match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|source| QueueError::Parse { path, source }),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Err(QueueError::NoSuchJob(id.clone())),
            Err(e) => Err(QueueError::io(&path)(e)),
        }
    }

    /// Every job, oldest first. A missing jobs folder has none.
    ///
    /// # Errors
    ///
    /// [`QueueError`] when the folder or a job file cannot be read.
    pub fn jobs(&self) -> Result<Vec<Job>, QueueError> {
        let entries = match fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(QueueError::io(&self.dir)(e)),
        };
        let mut ids = Vec::new();
        for entry in entries {
            let path = entry.map_err(QueueError::io(&self.dir))?.path();
            if path.extension() == Some(OsStr::new("json"))
                && let Some(id) = path
                    .file_stem()
                    .and_then(OsStr::to_str)
                    .and_then(|s| JobId::parse(s).ok())
            {
                ids.push(id);
            }
        }
        ids.sort();
        ids.iter().map(|id| self.job(id)).collect()
    }

    /// The file a job's switch writes its output to.
    pub fn log_path(&self, id: &JobId) -> PathBuf {
        self.dir.join(format!("{id}.log"))
    }

    /// Passes the job's output to `on_output` as it grows, checking every `poll`, until the job
    /// ends. Returns the job as it ended.
    ///
    /// # Errors
    ///
    /// [`QueueError`] when the job or its log cannot be read.
    pub fn follow(&self, id: &JobId, poll: Duration, on_output: impl FnMut(&str)) -> Result<Job, QueueError> {
        self.follow_with(id, poll, None::<&Systemd>, on_output)
    }

    /// [`Queue::follow`] for a job [`Queue::submit`] queued, which also fails the job when its
    /// worker unit dies before running it ([`Queue::reap`]), so the follower never waits forever.
    ///
    /// # Errors
    ///
    /// [`QueueError`] when the job or its log cannot be read, or its unit's state cannot be.
    pub fn follow_worker(
        &self,
        id: &JobId,
        poll: Duration,
        host: &impl Host,
        on_output: impl FnMut(&str),
    ) -> Result<Job, QueueError> {
        self.follow_with(id, poll, Some(host), on_output)
    }

    fn follow_with<H: Host>(
        &self,
        id: &JobId,
        poll: Duration,
        host: Option<&H>,
        mut on_output: impl FnMut(&str),
    ) -> Result<Job, QueueError> {
        let log_path = self.log_path(id);
        let mut offset = 0;
        let mut pending = Vec::new();
        loop {
            // HAZARD: read the state before the log. Read after, a job that ends between the two
            // reads loses the output it wrote last.
            let mut job = self.job(id)?;
            if let Some(host) = host
                && self.reap_job(host, &job)?
            {
                job = self.job(id)?;
            }
            offset += read_from(&log_path, offset, &mut pending).map_err(QueueError::io(&log_path))?;
            let text_end = if job.state.is_done() {
                pending.len()
            } else {
                valid_utf8_prefix(&pending)
            };
            if text_end > 0 {
                on_output(&String::from_utf8_lossy(&pending[..text_end]));
                pending.drain(..text_end);
            }
            if job.state.is_done() {
                return Ok(job);
            }
            thread::sleep(poll);
        }
    }

    fn job_path(&self, id: &JobId) -> PathBuf {
        self.dir.join(format!("{id}.json"))
    }

    fn write_job(&self, job: &Job) -> Result<(), QueueError> {
        let path = self.job_path(&job.id);
        let json = serde_json::to_string_pretty(job).map_err(|source| QueueError::Parse {
            path: path.clone(),
            source,
        })?;
        write_atomic(&path, &json).map_err(QueueError::io(&path))
    }

    /// Takes the switch lock, held until the returned file is dropped.
    fn lock(&self) -> Result<File, QueueError> {
        let path = self.lock_path();
        lock_file(&path).map_err(QueueError::io(&path))
    }

    /// Takes the switch lock if no worker holds it.
    fn try_lock(&self) -> Result<Option<File>, QueueError> {
        let path = self.lock_path();
        let file = File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .map_err(QueueError::io(&path))?;
        match file.try_lock() {
            Ok(()) => Ok(Some(file)),
            Err(TryLockError::WouldBlock) => Ok(None),
            Err(TryLockError::Error(e)) => Err(QueueError::io(&path)(e)),
        }
    }

    fn lock_path(&self) -> PathBuf {
        self.dir.join("switch.lock")
    }
}

/// What a job's pre-build held, and the packages it couldn't hold.
#[derive(Debug, Default)]
struct Holding {
    /// The packages the switch installs held, by name.
    held: Vec<String>,
    /// The packages that failed and that the home has no version of.
    unkept: Vec<String>,
}

impl Holding {
    /// The job's error when a package failed with no version to keep: the switch would build it
    /// again and fail the same way.
    fn unkept_error(&self) -> Result<(), String> {
        if self.unkept.is_empty() {
            return Ok(());
        }
        let messages: Vec<String> = self.unkept.iter().map(|name| held::unkept_message(name)).collect();
        Err(messages.join("; "))
    }
}

/// The outputs the home installs of `found`, one list.
fn outputs(found: &[&Found]) -> Vec<String> {
    found.iter().flat_map(|found| found.outputs.values().cloned()).collect()
}

/// The real host: worker units through `systemd-run --user`, switches through `home-manager`.
#[derive(Clone, Debug)]
pub struct Systemd {
    kubaos: PathBuf,
    flake: PathBuf,
    /// home-manager's GC root to the generation the home is on (`Paths::home_generation`).
    generation: Option<PathBuf>,
}

impl Systemd {
    /// A host whose workers run `kubaos` (this binary) and switch the user flake at `flake`.
    #[must_use]
    pub fn new(kubaos: impl Into<PathBuf>, flake: impl Into<PathBuf>) -> Self {
        Self {
            kubaos: kubaos.into(),
            flake: flake.into(),
            generation: None,
        }
    }

    /// The same host, reading what the home has from the generation `root` links to
    /// (`Paths::home_generation`). Without it, the home has no listed package to keep.
    #[must_use]
    pub fn generation(mut self, root: impl Into<PathBuf>) -> Self {
        self.generation = Some(root.into());
        self
    }

    /// The `systemd-run` command that starts the worker unit for `job`, passing `env` on.
    pub fn worker_command(&self, job: &JobId, env: &[(&str, OsString)]) -> Vec<OsString> {
        let mut argv: Vec<OsString> = vec![
            "systemd-run".into(),
            "--user".into(),
            format!("--unit={}", self.worker_unit(job)).into(),
            format!("--description=KubaOS switch {job}").into(),
            "--collect".into(),
            "--quiet".into(),
        ];
        for (name, value) in env {
            let mut arg = OsString::from(format!("--setenv={name}="));
            arg.push(value);
            argv.push(arg);
        }
        argv.extend([self.kubaos.as_os_str().to_owned(), "job".into(), "work".into()]);
        argv
    }

    /// The `systemctl` command that prints a unit's `ActiveState`. A unit systemd no longer knows,
    /// as `--collect` makes of a failed one, is `inactive`.
    pub fn unit_state_command(unit: &str) -> Vec<OsString> {
        ["systemctl", "--user", "show", "--property=ActiveState", "--value", unit]
            .into_iter()
            .map(OsString::from)
            .collect()
    }

    /// The switch itself, of the user flake as locked or with KubaOS from the checkout `from`.
    /// `-b hm-backup` moves a file home-manager would replace aside, as `system/60-nix.sh` does.
    pub fn switch_command(&self, from: Option<&Path>) -> Vec<OsString> {
        let mut argv: Vec<OsString> = ["home-manager", "switch", "-b", "hm-backup", "--flake"]
            .into_iter()
            .map(OsString::from)
            .chain([self.flake.as_os_str().to_owned()])
            .collect();
        argv.extend(from.map(override_args).into_iter().flatten());
        argv
    }

    /// The evaluation [`Host::evaluate`] runs: every home configuration of the user flake to its
    /// activation package's derivation, with KubaOS from the checkout `from`. It instantiates the
    /// whole home and builds nothing.
    pub fn evaluate_command(&self, from: &Path) -> Vec<OsString> {
        let mut installable = self.flake.as_os_str().to_owned();
        installable.push("#homeConfigurations");
        let mut argv: Vec<OsString> = vec!["nix".into(), "eval".into(), "--json".into(), installable];
        argv.extend(override_args(from));
        argv.extend([
            "--apply".into(),
            "configs: map (config: config.activationPackage.drvPath) (builtins.attrValues configs)".into(),
        ]);
        argv
    }

    /// The evaluation [`Host::listed`] runs: the listed packages (`kubaos.listedPackages`,
    /// `home/state.nix`) of the home configuration `home-manager switch` picks from `names`, with
    /// KubaOS from the checkout `from` when it is a preview, as a JSON list of [`Evaluated`].
    pub fn listed_command(&self, from: Option<&Path>, names: &[String]) -> Vec<OsString> {
        let mut installable = self.flake.as_os_str().to_owned();
        installable.push("#homeConfigurations");
        let mut argv: Vec<OsString> = vec!["nix".into(), "eval".into(), "--json".into(), installable];
        argv.extend(from.map(override_args).into_iter().flatten());
        let names: Vec<String> = names.iter().map(|name| nix_string(name)).collect();
        let apply = LISTED.replace("@NAMES@", &names.join(" "));
        argv.extend(["--apply".into(), apply.into()]);
        argv
    }

    /// The names `home-manager switch` looks the home configuration up by, the last one the flake
    /// has winning: `$USER`, then `$USER@` with the fully qualified, the plain and the short host
    /// name, as home-manager's own script tries them.
    fn configuration_names() -> Vec<String> {
        let Some(user) = std::env::var("USER").ok().filter(|user| !user.is_empty()) else {
            return Vec::new();
        };
        let mut names = vec![user.clone()];
        for flag in ["-f", "", "-s"] {
            let argv: Vec<OsString> = ["hostname", flag]
                .into_iter()
                .filter(|arg| !arg.is_empty())
                .map(OsString::from)
                .collect();
            if let Ok(host) = output(&argv) {
                names.push(format!("{user}@{}", host.trim()));
            }
        }
        names
    }

    /// The build [`Host::build`] runs: the outputs the home installs of every listed package,
    /// as far as each builds.
    pub fn build_command(installables: &[String]) -> Vec<OsString> {
        let mut argv: Vec<OsString> = ["nix", "build", "--no-link", "--keep-going"]
            .into_iter()
            .map(OsString::from)
            .collect();
        argv.extend(installables.iter().map(OsString::from));
        argv
    }

    /// The check after the build: it prints each of `paths` that is not in the store.
    pub fn missing_command(paths: &[String]) -> Vec<OsString> {
        let mut argv: Vec<OsString> = ["nix-store", "--check-validity", "--print-invalid"]
            .into_iter()
            .map(OsString::from)
            .collect();
        argv.extend(paths.iter().map(OsString::from));
        argv
    }

    /// The query of the store paths a generation's packages' environment refers to, for a
    /// generation without the record of its listed packages.
    pub fn references_command(generation: &Path) -> Vec<OsString> {
        vec![
            "nix-store".into(),
            "--query".into(),
            "--references".into(),
            generation.join("home-path").into_os_string(),
        ]
    }

    /// The `git worktree list` a branch given to `--from` is looked up in ([`preview::resolve`]).
    pub fn worktrees_command(repo: &Path) -> Vec<OsString> {
        let mut argv: Vec<OsString> = vec!["git".into(), "-C".into(), repo.as_os_str().to_owned()];
        argv.extend(["worktree", "list", "--porcelain"].map(OsString::from));
        argv
    }

    /// `repo`'s worktrees, as `git worktree list --porcelain` prints them.
    ///
    /// # Errors
    ///
    /// [`HostError`] when git fails, as outside a repository.
    pub fn worktrees(repo: &Path) -> Result<String, HostError> {
        output(&Self::worktrees_command(repo))
    }
}

/// The function `nix eval` applies to the user flake's home configurations, `@NAMES@` the
/// configuration's names to try ([`Systemd::configuration_names`]): each listed package of the
/// last one the flake has, with its derivation, the outputs the home installs and its name without
/// the version, or `null` for one the user's nixpkgs lacks or that doesn't evaluate. A home from a
/// KubaOS without `kubaos.listedPackages` lists none, and so does a flake without any of the names.
const LISTED: &str = r#"configs:
  let
    names = builtins.filter (name: configs ? ${name}) [ @NAMES@ ];
    homes = if names == [ ] then [ ] else [ configs.${builtins.elemAt names (builtins.length names - 1)} ];
  in builtins.concatMap (home:
  let listed = home.config.kubaos.listedPackages or { };
  in map (name:
    let
      package = listed.${name};
      found =
        if package ? drvPath then {
          drv = package.drvPath;
          outputs = builtins.listToAttrs (map (output: {
            name = output;
            value = package.${output}.outPath;
          }) (package.meta.outputsToInstall or [ package.outputName or "out" ]));
          pname = (builtins.parseDrvName package.name).name;
        } else null;
      tried = builtins.tryEval (builtins.deepSeq found found);
    in { inherit name; found = if tried.success then tried.value else null; })
    (builtins.attrNames listed))
  homes"#;

/// `text` as a Nix string literal: quoted, with the characters Nix reads in one escaped.
fn nix_string(text: &str) -> String {
    let mut quoted = String::from("\"");
    for c in text.chars() {
        match c {
            '\\' | '"' | '$' => {
                quoted.push('\\');
                quoted.push(c);
            }
            _ => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

/// The arguments that override the user flake's `kubaos` input to the checkout `from`. The lock
/// file is never written: it keeps the release, which `--release` goes back to.
fn override_args(from: &Path) -> [OsString; 4] {
    [
        "--override-input".into(),
        "kubaos".into(),
        preview::flake_ref(from).into(),
        "--no-write-lock-file".into(),
    ]
}

impl Host for Systemd {
    fn worker_unit(&self, job: &JobId) -> String {
        format!("kubaos-switch-{job}")
    }

    fn worker_alive(&self, unit: &str) -> Result<bool, HostError> {
        let argv = Self::unit_state_command(unit);
        let program = "systemctl";
        let out = Command::new(&argv[0])
            .args(&argv[1..])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .map_err(|e| HostError::spawn(program, e))?;
        if !out.status.success() {
            return Err(HostError::exited(program, out.status.code()));
        }
        Ok(is_alive_state(String::from_utf8_lossy(&out.stdout).trim()))
    }

    fn start_worker(&self, job: &JobId) -> Result<(), HostError> {
        let env: Vec<(&str, OsString)> = WORKER_ENV
            .iter()
            .filter_map(|name| std::env::var_os(name).map(|v| (*name, v)))
            .collect();
        run(&self.worker_command(job, &env), Stdio::inherit(), Stdio::inherit())
    }

    fn switch(&self, log: File, from: Option<&Path>) -> Result<(), HostError> {
        let program = "home-manager";
        let stderr = log.try_clone().map_err(|e| HostError::spawn(program, e))?;
        run(&self.switch_command(from), Stdio::from(log), Stdio::from(stderr))
    }

    fn describe(&self, path: &Path) -> Result<Checkout, HostError> {
        let git = |args: &[&str]| {
            let mut argv: Vec<OsString> = vec!["git".into(), "-C".into(), path.as_os_str().to_owned()];
            argv.extend(args.iter().map(OsString::from));
            argv
        };
        let root = PathBuf::from(output(&git(&["rev-parse", "--show-toplevel"]))?.trim());
        let commit = output(&git(&["rev-parse", "--verify", "HEAD"]))?.trim().to_owned();
        // On a detached head `symbolic-ref` exits 1: no branch.
        let branch = output(&git(&["symbolic-ref", "--quiet", "--short", "HEAD"]))
            .ok()
            .map(|branch| branch.trim().to_owned())
            .filter(|branch| !branch.is_empty());
        let status = output(&git(&["status", "--porcelain", "--untracked-files=no"]))?;
        Ok(Checkout {
            root,
            branch,
            commit,
            dirty: !status.trim().is_empty(),
        })
    }

    fn evaluate(&self, path: &Path) -> Result<(), HostError> {
        // Nix's errors reach the terminal: they say why the checkout doesn't evaluate.
        run(&self.evaluate_command(path), Stdio::null(), Stdio::inherit())
    }

    fn listed(&self, log: &File, from: Option<&Path>) -> Result<Vec<Evaluated>, HostError> {
        let argv = self.listed_command(from, &Self::configuration_names());
        let printed = output_to(&argv, log_stdio(log, "nix")?)?;
        serde_json::from_str(&printed).map_err(|source| HostError::Answer {
            program: "nix eval".to_owned(),
            source,
        })
    }

    fn build(&self, log: &File, installables: &[String]) -> Result<BTreeSet<String>, HostError> {
        let argv = Self::build_command(installables);
        let program = "nix";
        let mut child = kubaos_git::clear(&mut Command::new(&argv[0]))
            .args(&argv[1..])
            .stdin(Stdio::null())
            .stdout(log_stdio(log, program)?)
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| HostError::spawn(program, e))?;
        let mut stderr = String::new();
        if let Some(pipe) = child.stderr.take() {
            for line in io::BufReader::new(pipe).lines() {
                let line = line.map_err(|e| HostError::spawn(program, e))?;
                note_raw(log, &line);
                stderr.push_str(&line);
                stderr.push('\n');
            }
        }
        let status = child.wait().map_err(|e| HostError::spawn(program, e))?;
        // A package that fails to build fails the build, which Nix's messages name; a build
        // killed by a signal says nothing of the packages it didn't get to.
        if status.code().is_none() {
            return Err(HostError::exited(program, None));
        }
        Ok(held::failed_derivations(&stderr))
    }

    fn missing(&self, paths: &[String]) -> Result<BTreeSet<String>, HostError> {
        let missing = output(&Self::missing_command(paths))?;
        Ok(missing.lines().map(str::to_owned).collect())
    }

    fn current_home(&self) -> Result<CurrentHome, HostError> {
        let Some(root) = self.generation.as_deref().filter(|root| root.exists()) else {
            return Ok(CurrentHome::Nothing);
        };
        let record = root.join(GENERATION_RECORD);
        match fs::read(&record) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map(CurrentHome::Recorded)
                .map_err(|source| HostError::Answer {
                    program: record.display().to_string(),
                    source,
                }),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let paths = output(&Self::references_command(root))?;
                Ok(CurrentHome::Unrecorded(paths.lines().map(str::to_owned).collect()))
            }
            Err(source) => Err(HostError::Read { path: record, source }),
        }
    }
}

/// `log` as the output of `program`.
fn log_stdio(log: &File, program: &str) -> Result<Stdio, HostError> {
    log.try_clone()
        .map(Stdio::from)
        .map_err(|e| HostError::spawn(program, e))
}

/// Writes `line` to a job's log as KubaOS's own: a log that can't be written loses the line, never
/// the switch.
fn note(log: &File, line: &str) {
    note_raw(log, &format!("kubaos: {line}"));
}

/// Writes `line` to a job's log as it is.
fn note_raw(mut log: &File, line: &str) {
    let _ = writeln!(log, "{line}");
}

/// Whether a unit in systemd's `ActiveState` `state` still has, or is about to have, a process.
fn is_alive_state(state: &str) -> bool {
    matches!(state, "active" | "activating" | "deactivating" | "reloading")
}

/// Runs `argv` to the end with stdin closed and none of git's variables: Nix reads the flakes
/// from their git checkouts, which must be theirs, not a repository the caller's environment names.
fn run(argv: &[OsString], stdout: Stdio, stderr: Stdio) -> Result<(), HostError> {
    let Some((executable, rest)) = argv.split_first() else {
        return Err(HostError::spawn("", io::Error::other("empty command")));
    };
    let program = executable.to_string_lossy().into_owned();
    let status = kubaos_git::clear(&mut Command::new(executable))
        .args(rest)
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .status()
        .map_err(|e| HostError::spawn(&program, e))?;
    if status.success() {
        Ok(())
    } else {
        Err(HostError::exited(program, status.code()))
    }
}

/// Runs `argv` to the end with stdin closed, stderr dropped and none of git's variables (it may
/// be git: [`Systemd::worktrees`]), returning what it printed.
fn output(argv: &[OsString]) -> Result<String, HostError> {
    output_to(argv, Stdio::null())
}

/// [`output`], its stderr going to `stderr`.
fn output_to(argv: &[OsString], stderr: Stdio) -> Result<String, HostError> {
    let Some((executable, rest)) = argv.split_first() else {
        return Err(HostError::spawn("", io::Error::other("empty command")));
    };
    let program = executable.to_string_lossy().into_owned();
    let out = kubaos_git::clear(&mut Command::new(executable))
        .args(rest)
        .stdin(Stdio::null())
        .stderr(stderr)
        .output()
        .map_err(|e| HostError::spawn(&program, e))?;
    if !out.status.success() {
        return Err(HostError::exited(program, out.status.code()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// `e` and its causes, joined with `: `, for a job file a front end reads without the error types.
fn error_chain(e: &dyn std::error::Error) -> String {
    let mut text = e.to_string();
    let mut source = e.source();
    while let Some(cause) = source {
        text.push_str(": ");
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}

fn now_ms() -> u64 {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    u64::try_from(ms).unwrap_or(u64::MAX)
}

/// Appends what `path` holds past `offset` to `buf`, returning how much. A missing file has
/// nothing yet.
fn read_from(path: &Path, offset: u64, buf: &mut Vec<u8>) -> io::Result<u64> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(e) => return Err(e),
    };
    file.seek(SeekFrom::Start(offset))?;
    let read = file.read_to_end(buf)?;
    Ok(read as u64)
}

/// How much of `bytes` is whole UTF-8, so a character split across two reads waits for its rest.
fn valid_utf8_prefix(bytes: &[u8]) -> usize {
    match std::str::from_utf8(bytes) {
        Err(e) if e.error_len().is_none() => e.valid_up_to(),
        _ => bytes.len(),
    }
}

#[cfg(test)]
mod tests;
