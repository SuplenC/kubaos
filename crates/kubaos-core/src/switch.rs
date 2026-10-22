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
//! - Following a job's output until it ends.
//! - [`Host`], the seam to systemd and home-manager, and [`Systemd`], the real one.
//!
//! # Not here
//!
//! - What a switch applies: the user flake and its state directory (`state.rs`).
//! - The `kubaos switch` and `kubaos job` commands: `kubaos-cli`.

use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::fs::File;
use std::fs::TryLockError;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
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
}

/// What the queue needs from the machine: starting a worker unit, and the switch itself.
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

    /// Runs one switch to the end, its output going to `log`.
    ///
    /// # Errors
    ///
    /// [`HostError`] when the switch could not start or failed.
    fn switch(&self, log: File) -> Result<(), HostError>;
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
}

impl Queue {
    /// The queue in `dir` (`Paths::jobs`). The folder is made on first use.
    #[must_use]
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    /// Queues a switch job, without starting a worker for it.
    ///
    /// # Errors
    ///
    /// [`QueueError::Io`] when the job file cannot be written.
    pub fn enqueue(&self) -> Result<JobId, QueueError> {
        self.enqueue_with(|_| None)
    }

    /// Queues a job whose worker unit, if any, `unit` names from its id.
    fn enqueue_with(&self, unit: impl FnOnce(&JobId) -> Option<String>) -> Result<JobId, QueueError> {
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
        };
        self.write_job(&job)?;
        Ok(id)
    }

    /// Queues a switch job and starts its worker through `host`.
    ///
    /// # Errors
    ///
    /// [`QueueError::Start`] when the worker does not start, after marking the job failed, and
    /// [`QueueError::Io`] when the job file cannot be written.
    pub fn submit(&self, host: &impl Host) -> Result<JobId, QueueError> {
        let id = self.enqueue_with(|id| Some(host.worker_unit(id)))?;
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
            let result = host.switch(log);
            job.finished_at = Some(now_ms());
            match result {
                Ok(()) => job.state = JobState::Succeeded,
                Err(e) => {
                    job.state = JobState::Failed;
                    job.error = Some(error_chain(&e));
                }
            }
            self.write_job(&job)?;
            ran.push(job.id);
        }
        Ok(ran)
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

/// The real host: worker units through `systemd-run --user`, switches through `home-manager`.
#[derive(Clone, Debug)]
pub struct Systemd {
    kubaos: PathBuf,
    flake: PathBuf,
}

impl Systemd {
    /// A host whose workers run `kubaos` (this binary) and switch the user flake at `flake`.
    #[must_use]
    pub fn new(kubaos: impl Into<PathBuf>, flake: impl Into<PathBuf>) -> Self {
        Self {
            kubaos: kubaos.into(),
            flake: flake.into(),
        }
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

    /// The switch itself. `-b hm-backup` moves a file home-manager would replace aside, as
    /// `system/60-nix.sh` does.
    pub fn switch_command(&self) -> Vec<OsString> {
        ["home-manager", "switch", "-b", "hm-backup", "--flake"]
            .into_iter()
            .map(OsString::from)
            .chain([self.flake.as_os_str().to_owned()])
            .collect()
    }
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

    fn switch(&self, log: File) -> Result<(), HostError> {
        let program = "home-manager";
        let stderr = log.try_clone().map_err(|e| HostError::spawn(program, e))?;
        run(&self.switch_command(), Stdio::from(log), Stdio::from(stderr))
    }
}

/// Whether a unit in systemd's `ActiveState` `state` still has, or is about to have, a process.
fn is_alive_state(state: &str) -> bool {
    matches!(state, "active" | "activating" | "deactivating" | "reloading")
}

/// Runs `argv` to the end with stdin closed.
fn run(argv: &[OsString], stdout: Stdio, stderr: Stdio) -> Result<(), HostError> {
    let Some((executable, rest)) = argv.split_first() else {
        return Err(HostError::spawn("", io::Error::other("empty command")));
    };
    let program = executable.to_string_lossy().into_owned();
    let status = Command::new(executable)
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
