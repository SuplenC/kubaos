//! What `kubaos update`, `upgrade`, `rollback`, `channel` and the daily check do, step by step,
//! through a [`Machine`], and what they report as they go.
//!
//! An update, in order: the lock and a free-space check; fetching `origin` into the checkout,
//! which is the network check; the release policy's pick; showing it and asking; a snapper
//! snapshot, the user's `flake.lock` saved and the update recorded in the history; apt; moving
//! `release`; the Hyprland pin, the package manifest and the system migrations; `nix flake update`
//! of the user flake; relinking what `system/` took from KubaOS's nixpkgs; the switch; the
//! Bitwarden check; the user migrations; and the new release record. Packages first, then
//! migrations, which are written against the new packages.
//!
//! Each stage is a [`Phase`] of the progress stream (`event.rs`). Programs run with their output
//! captured: every line goes to the run's log and to the stream, never to the terminal directly.
//!
//! # Owns
//!
//! - [`Updater`] and the order of its steps, and [`UpdateError`].
//!
//! # Not here
//!
//! - Which release to move to: `policy.rs`. The files' formats: `record.rs`, `notice.rs`.
//! - Running programs and the switch: `machine.rs` and `switch.rs`.
//! - The package manifest's phase, its status and `kubaos packages enable|disable`:
//!   `updater/manifest.rs`.
//! - The events and the log: `event.rs` and `journal.rs`.

use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use std::fs::TryLockError;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use super::event::Command;
use super::event::Phase;
use super::event::Preview;
use super::event::Prompt;
use super::event::Release;
use super::event::Restart;
use super::event::Sink;
use super::event::Summary;
use super::event::UpdateEvent;
use super::event::chain;
use super::git::GitRefs;
use super::git::RemoteRefs;
use super::hyprland;
use super::journal::Journal;
use super::machine::Cmd;
use super::machine::Machine;
use super::machine::MachineError;
use super::migrations;
use super::migrations::Migration;
use super::notice::APT_SECURITY_KEY;
use super::notice::Available;
use super::notice::parse_notified;
use super::policy;
use super::policy::Pick;
use super::policy::PolicyError;
use super::policy::Refs;
use super::policy::Target;
use super::policy::Verified;
use super::record;
use super::record::Channel;
use super::record::HistoryEntry;
use super::record::Installed;
use super::record::RecordError;
use super::record::SystemPaths;
use super::record::parse_history;
use super::record::short;
use super::security;
use super::security::CountError;
use super::version::Version;
use crate::apt::Manifest;
use crate::apt::ManifestError;
use crate::apt::manifest::MANIFEST;
use crate::engine::files::utc_datetime;
use crate::state::Paths;
use crate::state::write_atomic;

/// The space an update needs free on `/` and `/nix`: a new home-manager generation, a kernel and
/// apt's downloads.
pub const MIN_FREE_BYTES: u64 = 3 * 1024 * 1024 * 1024;

/// How many commits of a release the summary lists.
const LOG_LINES: usize = 20;

/// How often, and how far apart, the update lock is tried before another update counts as running.
const LOCK_TRIES: u32 = 10;
const LOCK_WAIT: Duration = Duration::from_millis(20);

/// Writes stdin to the file `$1`, made with its folder, through a temporary file and a rename,
/// world-readable.
const WRITE_FILE: &str =
    r#"mkdir -p -- "$(dirname -- "$1")" && cat >"$1.tmp" && chmod 0644 "$1.tmp" && mv -f -- "$1.tmp" "$1""#;

/// An update, upgrade or rollback stopped.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UpdateError {
    /// `kubaos update` runs as the desktop user: it switches their home.
    #[error("run it as yourself, not root: it switches your home, and asks for sudo where it needs root")]
    Root,
    /// Another update, upgrade or rollback holds the lock.
    #[error("another kubaos update, upgrade or rollback is running")]
    Busy,
    /// A filesystem is too full to update.
    #[error("{} has {} MiB free; an update needs {} MiB", path.display(), free / 1024 / 1024, need / 1024 / 1024)]
    LowSpace { path: PathBuf, free: u64, need: u64 },
    /// `origin` can't be reached.
    #[error("can't reach KubaOS's origin (offline?)")]
    Offline(#[source] MachineError),
    /// A step's program failed.
    #[error("{what}")]
    Step {
        what: String,
        #[source]
        source: MachineError,
    },
    /// A migration failed; the ones after it did not run.
    #[error("migration {name} failed; fix what it says and run kubaos update again")]
    Migration {
        name: String,
        #[source]
        source: MachineError,
    },
    /// A file could not be read or written.
    #[error("using {}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// `/var/lib/kubaos/release` is not a release record.
    #[error("reading {}", path.display())]
    Record {
        path: PathBuf,
        #[source]
        source: RecordError,
    },
    /// The checkout has no `release` branch, or its `VERSION` is not a version.
    #[error("{} has no release to read the installed version from", path.display())]
    NoRelease { path: PathBuf },
    /// There is no user flake to update.
    #[error("no user flake at {}", path.display())]
    NoUserFlake { path: PathBuf },
    /// The release policy refused.
    #[error("picking the release")]
    Policy(#[source] PolicyError),
    /// The user said no.
    #[error("nothing changed")]
    Declined,
    /// The history has no update to roll back.
    #[error("no update to roll back in {}", path.display())]
    NothingToRollBack { path: PathBuf },
    /// Edge follows `main`, which has every release on it.
    #[error("the edge channel follows main, so kubaos update brings every release; there is no upgrade")]
    EdgeUpgrade,
    /// An upgrade starts from the newest patch of the installed series.
    #[error("{label} is a newer patch of this release: kubaos update first, then kubaos upgrade")]
    PatchFirst { label: String },
    /// No newer series is out.
    #[error("no newer KubaOS release to upgrade to")]
    NoUpgrade,
    /// The release's package manifest can't be read.
    #[error("reading the package manifest")]
    Manifest(#[source] ManifestError),
    /// The checked-out release has no package manifest.
    #[error("no package manifest at {}", path.display())]
    NoManifest { path: PathBuf },
    /// The manifest has no group of that name.
    #[error("the package manifest has no group {name}")]
    NoGroup { name: String },
    /// The group isn't one the user may turn on or off.
    #[error("{name} is part of KubaOS: it can't be turned on or off")]
    NotOptional { name: String },
}

impl UpdateError {
    fn io(path: &Path) -> impl FnOnce(io::Error) -> Self {
        let path = path.to_path_buf();
        move |source| Self::Io { path, source }
    }

    fn step(what: impl Into<String>) -> impl FnOnce(MachineError) -> Self {
        let what = what.into();
        move |source| Self::Step { what, source }
    }
}

/// Whether a command asks before it changes anything.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Confirm {
    #[default]
    Ask,
    AssumeYes,
}

/// How an update goes: asking first, and moving to the channel's release too.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UpdateOptions {
    confirm: Confirm,
    packages_only: bool,
}

impl UpdateOptions {
    /// An update that asks first and moves to the channel's release.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Asks, or doesn't, before changing anything.
    #[must_use]
    pub fn confirm(mut self, confirm: Confirm) -> Self {
        self.confirm = confirm;
        self
    }

    /// Updates packages only: `release` stays and no migration runs.
    #[must_use]
    pub fn packages_only(mut self) -> Self {
        self.packages_only = true;
        self
    }

    /// Moves to the channel's release too.
    #[must_use]
    pub fn no_packages_only(mut self) -> Self {
        self.packages_only = false;
        self
    }
}

/// What dpkg does with a config file the user changed and the package changed too: keep the
/// user's. Programs run with no terminal input, so dpkg can't ask.
const DPKG_KEEP_CONFIG: [&str; 4] = [
    "-o",
    "Dpkg::Options::=--force-confdef",
    "-o",
    "Dpkg::Options::=--force-confold",
];

/// The command that undoes an update or upgrade.
const UNDO: &str = "kubaos rollback";

/// What the apply step does on top of packages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Update,
    Upgrade,
}

impl Kind {
    fn command(self) -> Command {
        match self {
            Self::Update => Command::Update,
            Self::Upgrade => Command::Upgrade,
        }
    }
}

/// Updates, upgrades and rolls back one machine for one user.
#[derive(Debug)]
pub struct Updater<'a, M: Machine> {
    m: &'a mut M,
    j: Journal<'a>,
    sys: SystemPaths,
    user: Paths,
    now: SystemTime,
}

impl<'a, M: Machine> Updater<'a, M> {
    /// The updater of the machine at `sys`, for the user at `user`, as of `now`. It reports to
    /// nothing until given a sink.
    #[must_use]
    pub fn new(m: &'a mut M, sys: SystemPaths, user: Paths, now: SystemTime) -> Self {
        Self {
            m,
            j: Journal::new(),
            sys,
            user,
            now,
        }
    }

    /// Sends what the updater does to `sink`, as [`UpdateEvent`]s.
    #[must_use]
    pub fn reporting(mut self, sink: &'a mut dyn Sink) -> Self {
        self.j.send_to(sink);
        self
    }

    fn date(&self) -> String {
        utc_datetime(self.unix())
    }

    fn unix(&self) -> u64 {
        self.now.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
    }

    fn refs(&self) -> GitRefs {
        GitRefs::new(self.sys.checkout(), &self.sys.allowed_signers())
    }

    /// Updates packages, and moves to the release the channel's policy picks.
    ///
    /// # Errors
    ///
    /// [`UpdateError`] for the step that failed; the steps after it did not run.
    pub fn update(&mut self, opts: UpdateOptions) -> Result<(), UpdateError> {
        self.begin(Command::Update);
        let result = self.try_update(opts);
        self.end(result)
    }

    fn try_update(&mut self, opts: UpdateOptions) -> Result<(), UpdateError> {
        let (_lock, installed) = self.phase(Phase::Checking, |u| {
            let lock = u.lock()?;
            u.check_space()?;
            let installed = u.checked_installed()?;
            Ok((lock, installed))
        })?;
        let pick = self.fetch_and_pick(&installed)?;
        if let Some(upgrade) = &pick.upgrade {
            self.j
                .note(format!("KubaOS {} is available: kubaos upgrade", upgrade_name(upgrade)));
        }
        let target = if opts.packages_only { None } else { pick.update };
        if target.is_none() && !opts.packages_only {
            self.j
                .note("KubaOS is at the newest release of its channel: updating packages");
        }
        self.apply(&installed, target.as_ref(), opts, Kind::Update)
    }

    /// Moves to the next series, from the newest patch of the installed one. It always asks.
    ///
    /// # Errors
    ///
    /// [`UpdateError::EdgeUpgrade`] on edge, [`UpdateError::PatchFirst`] when a patch is pending,
    /// [`UpdateError::NoUpgrade`] when no newer series is out, else the step that failed.
    pub fn upgrade(&mut self) -> Result<(), UpdateError> {
        self.begin(Command::Upgrade);
        let result = self.try_upgrade();
        self.end(result)
    }

    fn try_upgrade(&mut self) -> Result<(), UpdateError> {
        let (_lock, installed) = self.phase(Phase::Checking, |u| {
            let lock = u.lock()?;
            let installed = u.checked_installed()?;
            if u.channel(&installed) == Channel::Edge {
                return Err(UpdateError::EdgeUpgrade);
            }
            u.check_space()?;
            Ok((lock, installed))
        })?;
        let pick = self.fetch_and_pick(&installed)?;
        if let Some(patch) = pick.update {
            return Err(UpdateError::PatchFirst { label: patch.label });
        }
        let target = pick.upgrade.ok_or(UpdateError::NoUpgrade)?;
        self.j.note(format!(
            "Upgrade: KubaOS {} → {}",
            installed.describe(),
            upgrade_name(&target)
        ));
        self.apply(&installed, Some(&target), UpdateOptions::new(), Kind::Upgrade)
    }

    /// Moves `release` back to where the last update found it, puts back the user's `flake.lock`
    /// of then, and switches. Migration markers stay: migrations only go forward.
    ///
    /// # Errors
    ///
    /// [`UpdateError::NothingToRollBack`] with no history, else the step that failed.
    pub fn rollback(&mut self, confirm: Confirm) -> Result<(), UpdateError> {
        self.begin(Command::Rollback);
        let result = self.try_rollback(confirm);
        self.end(result)
    }

    fn try_rollback(&mut self, confirm: Confirm) -> Result<(), UpdateError> {
        let path = self.sys.history();
        let (_lock, installed, mut history) = self.phase(Phase::Checking, |u| {
            let lock = u.lock()?;
            let installed = u.checked_installed()?;
            let history = parse_history(&read_or_empty(&path)?);
            Ok((lock, installed, history))
        })?;
        let entry = history
            .pop()
            .ok_or(UpdateError::NothingToRollBack { path: path.clone() })?;
        self.j.note(format!(
            "Roll back the update of {}: KubaOS {} ({}) → {} ({})",
            entry.date,
            entry.to_version,
            short(&entry.to_commit),
            entry.from_version,
            short(&entry.from_commit)
        ));
        let lock = entry.lock.clone().filter(|lock| lock.is_file());
        match &lock {
            Some(lock) => self.j.note(format!("Your flake.lock goes back to {}", lock.display())),
            None => self
                .j
                .warn("that update saved no flake.lock: your packages stay where they are"),
        }
        if confirm == Confirm::Ask && !self.confirm("Roll back?") {
            return Err(UpdateError::Declined);
        }
        let description = format!("before kubaos rollback {}", self.date());
        self.phase(Phase::Snapshot, |u| {
            u.snapshot(&description);
            Ok(())
        })?;
        if entry.from_commit != installed.commit {
            self.phase(Phase::Release, |u| u.move_release(&entry.from_commit))?;
        }
        self.phase(Phase::Home, |u| {
            if let Some(lock) = &lock {
                let dest = u.user.flake_lock();
                u.j.step("Putting your flake.lock back");
                fs::copy(lock, &dest).map_err(UpdateError::io(&dest))?;
            }
            u.relink()?;
            u.switch()?;
            u.report_reloaded();
            Ok(())
        })?;
        let refs = self.refs();
        let rolled_back = Installed {
            version: Version::parse(&entry.from_version).unwrap_or(installed.version.clone()),
            codename: file_line(&refs, &entry.from_commit, "CODENAME"),
            commit: entry.from_commit.clone(),
            channel: self.channel(&installed),
            date: self.date(),
        };
        self.write_root(&self.sys.release(), &rolled_back.to_text())?;
        let rest: String = history.iter().map(HistoryEntry::line).collect();
        self.write_root(&path, &rest)?;
        self.j.note(format!(
            "{} shit the bed? You're on KubaOS {} again, like it never happened.",
            entry.to_version,
            rolled_back.describe()
        ));
        let summary = Summary {
            command: Command::Rollback,
            from: release_of(&installed),
            to: release_of(&rolled_back),
            channel: rolled_back.channel,
            changes: Vec::new(),
            packages: 0,
            apt: Vec::new(),
            inputs: Vec::new(),
            migrations: Vec::new(),
            reloaded: self.j.reloaded.clone(),
            restart: Restart::None,
            restart_reason: None,
            undo: None,
            log: self.j.log_path(),
        };
        self.j.emit(&UpdateEvent::Summary(summary));
        Ok(())
    }

    /// Sets the channel the next update follows.
    ///
    /// # Errors
    ///
    /// [`UpdateError::Step`] when `/etc/kubaos/channel` can't be written.
    pub fn set_channel(&mut self, channel: Channel) -> Result<(), UpdateError> {
        self.write_root(&self.sys.channel(), &format!("{channel}\n"))?;
        match channel {
            Channel::Edge => self
                .j
                .note("Channel: edge. kubaos update follows origin/main from now on."),
            Channel::Stable => self
                .j
                .note("Channel: stable. kubaos update moves to the next verified release at or above this one."),
        }
        Ok(())
    }

    /// The channel the machine follows: `/etc/kubaos/channel`, else the release record's.
    pub fn channel(&self, installed: &Installed) -> Channel {
        fs::read_to_string(self.sys.channel())
            .ok()
            .and_then(|text| text.parse().ok())
            .unwrap_or(installed.channel)
    }

    /// The daily check: what `origin` offers, from `git ls-remote` as the user, written to
    /// `update-available`, and a notification for each new thing, once.
    ///
    /// # Errors
    ///
    /// [`UpdateError`] when origin can't be listed or the files can't be written.
    pub fn check(&mut self) -> Result<Available, UpdateError> {
        self.begin(Command::Check);
        let result = self.try_check();
        self.end(result)
    }

    fn try_check(&mut self) -> Result<Available, UpdateError> {
        let installed = self.phase(Phase::Checking, Self::checked_installed)?;
        let channel = self.channel(&installed);
        let (refs, refreshed) = self.phase(Phase::Fetching, |u| {
            let local = u.refs();
            let url = local
                .origin_url()
                .ok_or_else(|| UpdateError::NoRelease { path: u.sys.checkout() })?;
            u.j.step(format!("git ls-remote {url}"));
            let refs = RemoteRefs::fetch(&url, local).map_err(|e| UpdateError::Policy(PolicyError::Refs(e)))?;
            Ok((refs, u.refresh_apt_lists()))
        })?;
        let pick = self.phase(Phase::Verifying, |u| {
            let pick = policy::for_channel(channel)
                .pick(&installed, &refs)
                .map_err(UpdateError::Policy)?;
            u.j.detail(pick_detail(&pick));
            Ok(pick)
        })?;
        // The update's lock is held from counting to writing: an update that finishes in between
        // would otherwise have its cleared count written back.
        let _lock = match self.lock() {
            Ok(lock) => lock,
            Err(UpdateError::Busy) => {
                self.j
                    .note("An update is running: the check leaves update-available to it");
                return Ok(Available::default());
            }
            Err(e) => return Err(e),
        };
        let mut available = Available::from_pick(&pick);
        available.apt_security = self.apt_security_after(refreshed)?;
        self.write_available(&available)?;
        self.notify(&available)?;
        self.j.emit(&UpdateEvent::Available {
            update: available.update.as_ref().map(upgrade_name),
            upgrade: available.upgrade.as_ref().map(upgrade_name),
            apt_security: available.apt_security.len(),
        });
        Ok(available)
    }

    /// Sends the notifications `available` has not sent yet, and forgets the apt security ones
    /// that are no longer pending so `notified` stays as small as the pending set.
    fn notify(&mut self, available: &Available) -> Result<(), UpdateError> {
        let notified_path = self.user.notified();
        let mut notified = parse_notified(&read_or_empty(&notified_path)?);
        for (key, text) in available.notifications(&notified.clone()) {
            let cmd = Cmd::user("notify-send").args(["--app-name=KubaOS", "--urgency=low", "KubaOS", &text]);
            match self.quiet(&cmd) {
                Ok(_) if key.starts_with(APT_SECURITY_KEY) => notified.extend(available.apt_security_keys()),
                Ok(_) => {
                    notified.insert(key);
                }
                Err(e) => self.j.warn(format!("notifying: {e}")),
            }
        }
        let pending = available.apt_security_keys();
        notified.retain(|key| !key.starts_with(APT_SECURITY_KEY) || pending.contains(key));
        let text: String = notified.iter().flat_map(|key| [key.as_str(), "\n"]).collect();
        write_user(&notified_path, &text)
    }

    /// Refreshes the check's own apt lists as the user (`security.rs`): the system's are only
    /// refreshed by `kubaos update`. `--error-on=any` makes a list that failed to download fail
    /// the refresh, where apt would exit 0 and go on with the old one.
    fn refresh_apt_lists(&mut self) -> Result<(), CountError> {
        let dir = self.user.apt_lists();
        self.j.step("Refreshing the check's apt package lists");
        for path in security::apt_dirs(&dir) {
            fs::create_dir_all(&path).map_err(CountError::dir(&path))?;
        }
        let refresh = Cmd::user("apt-get")
            .args(security::apt_options(&dir))
            .args(["update", "-q", "--error-on=any"]);
        self.quiet(&refresh).map(drop).map_err(CountError::Refresh)
    }

    /// The apt packages `kubaos update` would upgrade from a security pocket. When the lists
    /// couldn't be refreshed or apt can't say, the previous check's answer stands, with a
    /// warning, so a flaky network neither drops the marker nor notifies twice.
    fn apt_security_after(&mut self, refreshed: Result<(), CountError>) -> Result<Vec<String>, UpdateError> {
        let counted = match refreshed {
            Ok(()) => self.count_apt_security(),
            Err(e) => Err(e),
        };
        match counted {
            Ok(pending) => Ok(pending),
            Err(e) => {
                self.j.warn(format!("counting the apt security updates: {}", chain(&e)));
                let text = read_or_empty(&self.user.update_available())?;
                Ok(Available::apt_security_of(&text))
            }
        }
    }

    /// Simulates the upgrade against the check's lists, then asks `apt-cache policy` about the
    /// candidates from `-updates` alone: one may carry a security fix newer than the installed
    /// version while being newer than the fix itself.
    fn count_apt_security(&mut self) -> Result<Vec<String>, CountError> {
        let options = security::apt_options(&self.user.apt_lists());
        self.j.step("Counting the apt security updates");
        let simulate = Cmd::user("apt-get")
            .args(&options)
            .args(["-s", "-q", "upgrade", "--with-new-pkgs"]);
        let candidates = security::candidates(&self.quiet(&simulate).map_err(CountError::Simulate)?);
        let others: Vec<&str> = candidates
            .iter()
            .filter(|c| !c.security)
            .map(|c| c.name.as_str())
            .collect();
        let newer = if others.is_empty() {
            BTreeSet::new()
        } else {
            let policy = Cmd::user("apt-cache").args(&options).arg("policy").args(others);
            security::with_newer_security(&self.quiet(&policy).map_err(CountError::Policy)?)
        };
        Ok(security::pending(&candidates, &newer))
    }

    /// The release the machine runs: its record, or, on a machine whose install wrote none, what
    /// the checkout's `release` branch holds.
    ///
    /// # Errors
    ///
    /// [`UpdateError::Record`] for a record that can't be parsed, [`UpdateError::NoRelease`] when
    /// there is no record and no `release` to read.
    pub fn installed(&self) -> Result<Installed, UpdateError> {
        let path = self.sys.release();
        match fs::read_to_string(&path) {
            Ok(text) => return Installed::parse(&text).map_err(|source| UpdateError::Record { path, source }),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(UpdateError::io(&path)(e)),
        }
        let refs = self.refs();
        let no_release = || UpdateError::NoRelease {
            path: self.sys.checkout(),
        };
        let commit = refs
            .commit_of(&format!("refs/heads/{}", record::RELEASE_BRANCH))
            .ok_or_else(no_release)?;
        let version = file_line(&refs, &commit, "VERSION")
            .and_then(|v| Version::parse(&v).ok())
            .ok_or_else(no_release)?;
        Ok(Installed {
            codename: file_line(&refs, &commit, "CODENAME"),
            channel: channel_of_commit(&refs, &commit),
            version,
            commit,
            date: String::new(),
        })
    }

    /// The installed release, reported as the running phase's result.
    fn checked_installed(&mut self) -> Result<Installed, UpdateError> {
        let installed = self.installed()?;
        let channel = self.channel(&installed);
        self.j
            .detail(format!("KubaOS {}, {channel}", release_of(&installed).describe()));
        Ok(installed)
    }

    /// Starts a run: its log, unless it is a check, and the `started` event.
    fn begin(&mut self, command: Command) {
        let stamp = self.date().replace(' ', "T").replace(':', "-");
        let opened = if command == Command::Check {
            Ok(())
        } else {
            self.j.open_log(&self.user.update_logs(), &stamp, command)
        };
        self.j.emit(&UpdateEvent::Started {
            command,
            log: self.j.log_path(),
        });
        if let Err(e) = opened {
            self.j.warn(format!("no log of this run: {e}"));
        }
    }

    /// Ends a run that stopped with its `declined` or `failed` event.
    fn end<T>(&mut self, result: Result<T, UpdateError>) -> Result<T, UpdateError> {
        if let Err(e) = &result {
            self.j.failed(e);
        }
        result
    }

    /// Runs `work` as `phase`, which ends failed when `work` fails.
    fn phase<T>(
        &mut self,
        phase: Phase,
        work: impl FnOnce(&mut Self) -> Result<T, UpdateError>,
    ) -> Result<T, UpdateError> {
        self.j.phase_start(phase);
        let result = work(self);
        self.j.phase_end(result.is_ok());
        result
    }

    /// Asks `question` through the machine, reported as a prompt and its answer.
    fn confirm(&mut self, question: &str) -> bool {
        self.j.prompt(Prompt::Confirm {
            question: question.to_owned(),
        });
        let yes = self.m.confirm(question);
        self.j.emit(&UpdateEvent::Answered { yes });
        yes
    }

    /// Has sudo take the password before `cmd`, when it needs sudo and sudo would ask: sudo asks
    /// on the terminal, which the captured program can't show.
    fn authorise(&mut self, cmd: &Cmd) -> Result<(), MachineError> {
        if !cmd.uses_sudo() || !self.m.needs_password() {
            return Ok(());
        }
        self.j.prompt(Prompt::Password);
        let taken = self.m.authenticate();
        self.j.emit(&UpdateEvent::Answered { yes: taken.is_ok() });
        taken
    }

    /// Runs `cmd` for its stdout; the command line and a failure go to the log.
    fn quiet(&mut self, cmd: &Cmd) -> Result<String, MachineError> {
        self.authorise(cmd)?;
        self.j.command(cmd);
        let out = self.m.output(cmd);
        if let Err(e) = &out {
            self.j.log_line(&format!("failed: {}", chain(e)));
        }
        out
    }

    /// Runs `cmd` with every line of its output reported.
    fn run(&mut self, cmd: &Cmd) -> Result<(), MachineError> {
        self.run_with(cmd, |_| {})
    }

    /// [`Updater::run`], handing each line to `also` too.
    fn run_with(&mut self, cmd: &Cmd, mut also: impl FnMut(&str)) -> Result<(), MachineError> {
        self.authorise(cmd)?;
        self.j.command(cmd);
        let j = &mut self.j;
        let ran = self.m.run(cmd, &mut |line| {
            j.output(line);
            also(line);
        });
        match &ran {
            Ok(()) => self.j.succeeded(),
            Err(e) => self.j.log_line(&format!("failed: {}", chain(e))),
        }
        ran
    }

    /// The home-manager switch, its output reported.
    fn switch(&mut self) -> Result<(), UpdateError> {
        self.j.step("Switching your home");
        self.j.command_line("home-manager switch (kubaos switch)");
        let j = &mut self.j;
        let switched = self.m.switch(&mut |line| j.output(line));
        match switched {
            Ok(()) => {
                self.j.succeeded();
                Ok(())
            }
            Err(e) => Err(UpdateError::step("switching your home")(e)),
        }
    }

    /// Reports what the session reloaded after the switch as the running phase's result.
    fn report_reloaded(&mut self) {
        if !self.j.reloaded.is_empty() {
            let reloaded = self.j.reloaded.join(", ");
            self.j.detail(reloaded);
        }
    }

    fn lock(&self) -> Result<File, UpdateError> {
        let path = self.user.update_lock();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(UpdateError::io(dir))?;
        }
        let file = File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .map_err(UpdateError::io(&path))?;
        // A child forked by this process keeps a copy of a just-dropped lock's descriptor until
        // it execs, and the lock lives as long as any copy does. A short wait outlasts that.
        for _ in 0..LOCK_TRIES - 1 {
            match file.try_lock() {
                Ok(()) => return Ok(file),
                Err(TryLockError::WouldBlock) => std::thread::sleep(LOCK_WAIT),
                Err(TryLockError::Error(e)) => return Err(UpdateError::io(&path)(e)),
            }
        }
        match file.try_lock() {
            Ok(()) => Ok(file),
            Err(TryLockError::WouldBlock) => Err(UpdateError::Busy),
            Err(TryLockError::Error(e)) => Err(UpdateError::io(&path)(e)),
        }
    }

    fn check_space(&self) -> Result<(), UpdateError> {
        let nix = self.sys.root().join("nix");
        for path in [self.sys.root().to_path_buf(), nix] {
            if !path.is_dir() {
                continue;
            }
            let free = self.m.free_bytes(&path).map_err(UpdateError::io(&path))?;
            if free < MIN_FREE_BYTES {
                return Err(UpdateError::LowSpace {
                    path,
                    free,
                    need: MIN_FREE_BYTES,
                });
            }
        }
        Ok(())
    }

    fn fetch_and_pick(&mut self, installed: &Installed) -> Result<Pick, UpdateError> {
        let fetch = Cmd::root("git")
            .arg("-C")
            .arg(self.sys.checkout())
            .args(["fetch", "--quiet", "--tags", "--force", "origin"]);
        self.phase(Phase::Fetching, |u| {
            u.j.step("git fetch origin");
            u.quiet(&fetch).map(drop).map_err(UpdateError::Offline)
        })?;
        let channel = self.channel(installed);
        self.phase(Phase::Verifying, |u| {
            let pick = policy::for_channel(channel)
                .pick(installed, &u.refs())
                .map_err(UpdateError::Policy)?;
            for note in &pick.notes {
                u.j.warn(note.clone());
            }
            u.j.detail(pick_detail(&pick));
            Ok(pick)
        })
    }

    /// The steps shared by an update and an upgrade, from the preview on.
    fn apply(
        &mut self,
        installed: &Installed,
        target: Option<&Target>,
        opts: UpdateOptions,
        kind: Kind,
    ) -> Result<(), UpdateError> {
        let flake = self.user.flake().to_path_buf();
        let preview = self.phase(Phase::Reviewing, |u| {
            if !flake.join("flake.nix").is_file() {
                return Err(UpdateError::NoUserFlake { path: flake.clone() });
            }
            u.review(installed, target)
        })?;
        let packages = preview.packages;
        let changes = preview.changes.clone();
        self.j.emit(&UpdateEvent::Preview(preview));
        // An upgrade always asks: it moves to a new Ubuntu-and-nixpkgs pairing.
        let ask = opts.confirm == Confirm::Ask || kind == Kind::Upgrade;
        if ask && !self.confirm("Update now?") {
            return Err(UpdateError::Declined);
        }

        let to = target.map_or_else(|| installed.version.to_string(), |t| t.label.clone());
        let description = format!(
            "before kubaos {} {} ({} → {to})",
            kind.command().word(),
            self.date(),
            installed.version
        );
        self.phase(Phase::Snapshot, |u| {
            u.snapshot(&description);
            let lock = u.save_lock()?;
            u.record_history(installed, target, lock)
        })?;
        self.phase(Phase::SystemPackages, |u| u.upgrade_apt(packages))?;
        if let Some(target) = target {
            self.phase(Phase::Release, |u| u.move_release(&target.commit))?;
        }
        let mut migrations = Vec::new();
        let mut relogin = false;
        if !opts.packages_only {
            relogin = self.apply_hyprland_pin()?;
        }
        let apt = if self.sys.manifest().is_file() {
            self.phase(Phase::Manifest, |u| u.sync_manifest(None))?
        } else {
            Vec::new()
        };
        if !opts.packages_only {
            migrations.extend(self.run_system_migrations()?);
        }
        self.phase(Phase::Inputs, |u| u.update_inputs(&flake))?;
        self.phase(Phase::Home, |u| {
            u.relink()?;
            u.switch()?;
            u.check_bitwarden();
            u.report_reloaded();
            Ok(())
        })?;
        if !opts.packages_only {
            migrations.extend(self.run_user_migrations()?);
        }

        let now = self.record(installed, target)?;
        self.clear_available(&now)?;
        let (restart, restart_reason) = if self.sys.root().join("var/run/reboot-required").exists() {
            (Restart::Reboot, Some("an updated package asks for it".to_owned()))
        } else if relogin {
            (
                Restart::Relogin,
                Some("Hyprland was updated, and the running one is the old version".to_owned()),
            )
        } else {
            (Restart::None, None)
        };
        let summary = Summary {
            command: kind.command(),
            from: release_of(installed),
            to: release_of(&now),
            channel: now.channel,
            changes,
            packages,
            apt,
            inputs: self.j.inputs.clone(),
            migrations,
            reloaded: self.j.reloaded.clone(),
            restart,
            restart_reason,
            undo: Some(UNDO.to_owned()),
            log: self.j.log_path(),
        };
        self.j.emit(&UpdateEvent::Summary(summary));
        Ok(())
    }

    /// What the update brings: the release's commits, the apt packages, whether the Hyprland pin
    /// changes, and the system migrations it will run.
    fn review(&mut self, installed: &Installed, target: Option<&Target>) -> Result<Preview, UpdateError> {
        self.j.step("Refreshing apt's package lists");
        self.quiet(&Cmd::root("apt-get").args(["update", "-q"]))
            .map_err(UpdateError::step("refreshing apt's package lists"))?;
        let mut preview = Preview {
            from: release_of(installed),
            to: None,
            channel: self.channel(installed),
            changes: Vec::new(),
            packages: 0,
            apt: Vec::new(),
            hyprland_pin: false,
            migrations: Vec::new(),
        };
        // The manifest the phase will read: the release's, once it is checked out.
        let mut manifest = Manifest::read(&self.sys.manifest()).ok().flatten();
        if let Some(target) = target {
            self.j.step("Reading the release's changes");
            let refs = self.refs();
            preview.to = Some(Release {
                version: target
                    .version
                    .as_ref()
                    .map_or_else(|| target.label.clone(), ToString::to_string),
                codename: target.codename.clone(),
                commit: target.commit.clone(),
            });
            preview.changes = refs.log(&installed.commit, &target.commit, LOG_LINES);
            let pin = refs.file_at(&target.commit, record::RELEASE_PIN);
            preview.hyprland_pin = pin.is_some() && pin != fs::read_to_string(self.sys.apt_pin()).ok();
            let markers = self.sys.markers();
            preview.migrations = refs
                .file_list(&target.commit, "migrations/system")
                .into_iter()
                .filter(|name| migrations::stamp_of(name).is_some() && !markers.join(name).exists())
                .collect();
            manifest = refs
                .file_at(&target.commit, MANIFEST)
                .and_then(|text| Manifest::parse(&text).ok());
        }
        if let Some(manifest) = manifest {
            self.j.step("Listing the package manifest's new packages");
            // The manifest phase reports its own failures; the preview goes without the list.
            preview.apt = self.manifest_installs(manifest).unwrap_or_default();
        }
        self.j.step("Counting the apt packages to upgrade");
        let simulated = self
            .quiet(&Cmd::user("apt-get").args(["-s", "-q", "upgrade", "--with-new-pkgs"]))
            .unwrap_or_default();
        preview.packages = simulated.lines().filter(|line| line.starts_with("Inst ")).count();
        let mut detail = vec![count(preview.packages, "apt package")];
        if preview.to.is_some() {
            detail.push(count(preview.changes.len(), "commit"));
        }
        self.j.detail(detail.join(", "));
        Ok(preview)
    }

    fn snapshot(&mut self, description: &str) {
        if !self.sys.root().join("etc/snapper/configs/root").is_file() {
            self.j.warn("snapper isn't set up: no snapshot to go back to this time");
            self.j.detail("no snapshot");
            return;
        }
        self.j.step("Taking a snapper snapshot");
        let cmd = Cmd::root("snapper").args([
            "-c",
            "root",
            "create",
            "--cleanup-algorithm",
            "number",
            "--description",
            description,
        ]);
        match self.quiet(&cmd) {
            Ok(_) => self.j.detail(description),
            Err(e) => {
                self.j.warn(format!("no snapshot: {e}"));
                self.j.detail("no snapshot");
            }
        }
    }

    /// Copies the user flake's `flake.lock` to the saved locks, because `$HOME` is not in the
    /// snapshot. `None` when there is no lock yet.
    fn save_lock(&mut self) -> Result<Option<PathBuf>, UpdateError> {
        let lock = self.user.flake_lock();
        if !lock.is_file() {
            return Ok(None);
        }
        let dir = self.user.saved_locks();
        fs::create_dir_all(&dir).map_err(UpdateError::io(&dir))?;
        let mut saved = dir.join(format!("{}.lock", self.unix()));
        let mut n = 1;
        while saved.exists() {
            saved = dir.join(format!("{}-{n}.lock", self.unix()));
            n += 1;
        }
        fs::copy(&lock, &saved).map_err(UpdateError::io(&saved))?;
        Ok(Some(saved))
    }

    fn record_history(
        &mut self,
        installed: &Installed,
        target: Option<&Target>,
        lock: Option<PathBuf>,
    ) -> Result<(), UpdateError> {
        let path = self.sys.history();
        let entry = HistoryEntry {
            date: self.date(),
            from_commit: installed.commit.clone(),
            from_version: installed.version.to_string(),
            to_commit: target.map_or_else(|| installed.commit.clone(), |t| t.commit.clone()),
            to_version: target
                .and_then(|t| t.version.as_ref())
                .unwrap_or(&installed.version)
                .to_string(),
            lock,
        };
        let text = read_or_empty(&path)? + &entry.line();
        self.write_root(&path, &text)
    }

    /// `apt-get upgrade`, its `Setting up` lines counted against the `packages` it will upgrade.
    fn upgrade_apt(&mut self, packages: usize) -> Result<(), UpdateError> {
        let cmd = Cmd::root("apt-get")
            .args(["upgrade", "--with-new-pkgs", "-y"])
            .args(DPKG_KEEP_CONFIG)
            .env("DEBIAN_FRONTEND", "noninteractive");
        self.j.step("apt-get upgrade");
        self.j.count("Setting up ", packages);
        self.run(&cmd).map_err(UpdateError::step("upgrading apt packages"))?;
        self.j.detail(count(packages, "package"));
        Ok(())
    }

    fn move_release(&mut self, commit: &str) -> Result<(), UpdateError> {
        let cmd = Cmd::root("git").arg("-C").arg(self.sys.checkout()).args([
            "checkout",
            "--quiet",
            "-B",
            record::RELEASE_BRANCH,
            commit,
        ]);
        self.j.step(format!("Moving release to {}", short(commit)));
        self.quiet(&cmd)
            .map_err(UpdateError::step(format!("moving release to {}", short(commit))))?;
        self.j.detail(format!("release is at {}", short(commit)));
        Ok(())
    }

    /// Installs the release's Hyprland pin when it differs from the one apt has. Whether it was
    /// installed, so the running Hyprland is an old one.
    fn apply_hyprland_pin(&mut self) -> Result<bool, UpdateError> {
        let Ok(wanted) = fs::read_to_string(self.sys.checkout().join(record::RELEASE_PIN)) else {
            return Ok(false);
        };
        if fs::read_to_string(self.sys.apt_pin()).ok().as_deref() == Some(wanted.as_str()) {
            return Ok(false);
        }
        self.phase(Phase::Hyprland, |u| u.install_hyprland_pin(&wanted))
    }

    /// The pinned versions exactly, then the whole stack held again, whatever happened.
    fn install_hyprland_pin(&mut self, wanted: &str) -> Result<bool, UpdateError> {
        let arch = self
            .quiet(&Cmd::user("dpkg").arg("--print-architecture"))
            .unwrap_or_default();
        if arch.trim() != "amd64" {
            self.j.skip("Hyprland comes from KubaOS's nixpkgs on this architecture");
            return Ok(false);
        }
        let pins = hyprland::parse(wanted);
        let held: BTreeSet<String> = self
            .quiet(&Cmd::user("apt-mark").arg("showhold"))
            .unwrap_or_default()
            .lines()
            .map(str::trim)
            .filter(|p| hyprland::is_held_stack(p))
            .map(str::to_owned)
            .collect();
        if !held.is_empty() {
            self.j.step("Unholding the Hyprland stack");
            self.quiet(&Cmd::root("apt-mark").arg("unhold").args(&held))
                .map_err(UpdateError::step("unholding the Hyprland stack"))?;
        }
        let install = Cmd::root("apt-get")
            .args(["install", "-y", "--no-install-recommends", "--allow-downgrades"])
            .args(DPKG_KEEP_CONFIG)
            .args(pins.iter().map(hyprland::Pin::install_arg))
            .env("DEBIAN_FRONTEND", "noninteractive");
        // TODO(#14): when the PPA no longer has a pinned version, install the release's .debs,
        // checked against its signed SHA256SUMS; releases don't attach them yet.
        self.j.step("Installing the pinned Hyprland");
        let pinned = self
            .run(&install)
            .map_err(UpdateError::step("installing the pinned Hyprland"));
        let mut to_hold = held;
        to_hold.extend(self.installed_hypr_packages());
        let rehold = if to_hold.is_empty() {
            Ok(())
        } else {
            self.j.step("Holding the Hyprland stack again");
            self.quiet(&Cmd::root("apt-mark").arg("hold").args(&to_hold))
                .map(drop)
                .map_err(UpdateError::step("holding the Hyprland stack again"))
        };
        pinned?;
        rehold?;
        // Written once the versions are in, so a failed install is tried again on the next update.
        self.write_root(&self.sys.apt_pin(), wanted)?;
        self.j
            .detail(format!("applied, {} held", count(to_hold.len(), "package")));
        if self.quiet(&Cmd::user("Hyprland").arg("--verify-config")).is_err() {
            self.j
                .warn("Hyprland --verify-config reports problems: run it and fix your config before logging out");
        }
        Ok(true)
    }

    fn installed_hypr_packages(&mut self) -> Vec<String> {
        self.quiet(&Cmd::user("dpkg-query").args(["-W", "-f", "${db:Status-Status} ${Package}\n"]))
            .unwrap_or_default()
            .lines()
            .filter_map(|line| line.strip_prefix("installed "))
            .filter(|p| hyprland::is_held_stack(p))
            .map(str::to_owned)
            .collect()
    }

    /// The release's pending system migrations, as root; the names of those that ran.
    fn run_system_migrations(&mut self) -> Result<Vec<String>, UpdateError> {
        let dir = self.sys.system_migrations();
        let pending = migrations::pending(&dir, &self.sys.markers()).map_err(UpdateError::io(&dir))?;
        if pending.is_empty() {
            return Ok(Vec::new());
        }
        self.phase(Phase::SystemMigrations, |u| {
            u.each_migration(&pending, |u, migration| {
                let cmd = Cmd::root("bash")
                    .args(["-euo", "pipefail"])
                    .arg(&migration.path)
                    .env("KUBAOS_DIR", u.sys.checkout())
                    .cwd(u.sys.checkout());
                u.run(&cmd).map_err(|source| UpdateError::Migration {
                    name: migration.name.clone(),
                    source,
                })?;
                let marker = u.sys.markers().join(&migration.name);
                u.write_root(&marker, &format!("{}\n", u.date()))
            })
        })
    }

    /// The release's pending user migrations; the names of those that ran.
    fn run_user_migrations(&mut self) -> Result<Vec<String>, UpdateError> {
        let dir = self.sys.user_migrations();
        let markers = self.user.migration_markers();
        let pending: Vec<Migration> = migrations::pending(&dir, &markers).map_err(UpdateError::io(&dir))?;
        if pending.is_empty() {
            return Ok(Vec::new());
        }
        self.phase(Phase::UserMigrations, |u| {
            u.each_migration(&pending, |u, migration| {
                let cmd = Cmd::user("bash")
                    .args(["-euo", "pipefail"])
                    .arg(&migration.path)
                    .env("KUBAOS_DIR", u.sys.checkout())
                    .env("KUBAOS_USER_FLAKE", u.user.flake())
                    .env("KUBAOS_BACKUPS", u.user.backups())
                    .cwd(u.user.home());
                u.run(&cmd).map_err(|source| UpdateError::Migration {
                    name: migration.name.clone(),
                    source,
                })?;
                write_user(&markers.join(&migration.name), &format!("{}\n", u.date()))
            })
        })
    }

    /// Runs `one` for each migration in turn, reporting the progress; the names of those that ran.
    fn each_migration(
        &mut self,
        pending: &[Migration],
        mut one: impl FnMut(&mut Self, &Migration) -> Result<(), UpdateError>,
    ) -> Result<Vec<String>, UpdateError> {
        let total = pending.len();
        let mut ran = Vec::new();
        for migration in pending {
            self.j.progress(ran.len(), total);
            self.j.step(migration.name.clone());
            one(self, migration)?;
            ran.push(migration.name.clone());
        }
        self.j.progress(total, total);
        self.j.detail(format!("{} ran", count(total, "migration")));
        Ok(ran)
    }

    /// `nix flake update` of the user flake, reporting the inputs it moved.
    fn update_inputs(&mut self, flake: &Path) -> Result<(), UpdateError> {
        self.j.step("nix flake update");
        self.run(&Cmd::user("nix").arg("flake").arg("update").arg("--flake").arg(flake))
            .map_err(UpdateError::step("updating your flake's inputs (nix flake update)"))?;
        let detail = if self.j.inputs.is_empty() {
            "none moved".to_owned()
        } else {
            self.j.inputs.join(", ")
        };
        self.j.detail(detail);
        Ok(())
    }

    /// Points what `system/` linked from KubaOS's nixpkgs (the lock screen's PAM modules, the
    /// recorder's helper, the wallpaper sets) at the release's builds. Before the switch: the
    /// new home reads the wallpaper link.
    fn relink(&mut self) -> Result<(), UpdateError> {
        let script = self.sys.checkout().join("system/relink.sh");
        if !script.is_file() {
            return Ok(());
        }
        self.j.step("Relinking the lock screen's PAM modules and the recorder");
        self.run(&Cmd::user(&script).runs_sudo()).map_err(UpdateError::step(
            "relinking the lock screen's PAM modules and the recorder",
        ))
    }

    /// Reports newer Bitwarden releases, which are pinned downloads, not apt packages.
    fn check_bitwarden(&mut self) {
        let script = self.sys.checkout().join("system/70-bitwarden.sh");
        if !script.is_file() {
            return;
        }
        self.j.step("Looking for new Bitwarden releases");
        let mut news = Vec::new();
        let checked = self.run_with(&Cmd::user(&script).arg("--check-updates"), |line| {
            news.extend(bitwarden_news(line));
        });
        if checked.is_err() {
            self.j.warn("the Bitwarden release check failed");
        }
        for text in news {
            self.j.warn(text);
        }
    }

    fn record(&mut self, installed: &Installed, target: Option<&Target>) -> Result<Installed, UpdateError> {
        let refs = self.refs();
        let commit = target.map_or_else(|| installed.commit.clone(), |t| t.commit.clone());
        let version = target
            .and_then(|t| t.version.clone())
            .or_else(|| file_line(&refs, &commit, "VERSION").and_then(|v| Version::parse(&v).ok()))
            .unwrap_or_else(|| installed.version.clone());
        let now = Installed {
            codename: file_line(&refs, &commit, "CODENAME").or_else(|| installed.codename.clone()),
            channel: self.channel(installed),
            date: self.date(),
            version,
            commit,
        };
        self.write_root(&self.sys.release(), &now.to_text())?;
        Ok(now)
    }

    /// Drops what the update installed from `update-available`, keeping an upgrade still ahead.
    fn clear_available(&mut self, now: &Installed) -> Result<(), UpdateError> {
        let path = self.user.update_available();
        let Ok(text) = fs::read_to_string(&path) else {
            return Ok(());
        };
        let upgrade = text
            .lines()
            .find_map(|line| line.strip_prefix("upgrade="))
            .and_then(|v| Version::parse(v).ok())
            .filter(|v| v > &now.version);
        let kept = Available {
            update: None,
            upgrade: upgrade.map(|version| Target {
                commit: value(&text, "upgrade-commit").unwrap_or_default(),
                codename: value(&text, "upgrade-codename"),
                label: version.to_string(),
                version: Some(version),
            }),
            apt_security: Vec::new(),
        };
        self.write_available(&kept)
    }

    /// Writes `update-available`, empty when nothing is: the bar watches the file, and a watch
    /// needs a file to follow.
    fn write_available(&self, available: &Available) -> Result<(), UpdateError> {
        write_user(&self.user.update_available(), &available.to_text())
    }

    fn write_root(&mut self, path: &Path, text: &str) -> Result<(), UpdateError> {
        let cmd = Cmd::root("sh").args(["-c", WRITE_FILE, "sh"]).arg(path).stdin(text);
        self.quiet(&cmd)
            .map(drop)
            .map_err(UpdateError::step(format!("writing {}", path.display())))
    }
}

/// The channel a machine without a channel file follows: stable when its release is a verified
/// tag, else edge, as for a machine installed from a payload built at no verified tag.
fn channel_of_commit(refs: &GitRefs, commit: &str) -> Channel {
    let tagged = refs.tags().unwrap_or_default().into_iter().any(|tag| {
        tag.commit == commit
            && Version::parse(&tag.name).is_ok()
            && refs.verify_tag(&tag.name).ok() == Some(Verified::Good)
    });
    if tagged { Channel::Stable } else { Channel::Edge }
}

/// The first line of `path` at `commit`, trimmed, when it is there and not empty.
fn file_line(refs: &dyn Refs, commit: &str, path: &str) -> Option<String> {
    let text = refs.file_at(commit, path)?;
    let line = text.lines().next()?.trim();
    (!line.is_empty()).then(|| line.to_owned())
}

fn value(text: &str, key: &str) -> Option<String> {
    text.lines()
        .find_map(|line| line.strip_prefix(&format!("{key}=")))
        .map(str::to_owned)
}

fn release_of(installed: &Installed) -> Release {
    Release {
        version: installed.version.to_string(),
        codename: installed.codename.clone(),
        commit: installed.commit.clone(),
    }
}

/// What the policy found, in a few words.
fn pick_detail(pick: &Pick) -> String {
    match (&pick.update, &pick.upgrade) {
        (Some(update), _) => format!("found {}", update.label),
        (None, Some(upgrade)) => format!("found {}", upgrade_name(upgrade)),
        (None, None) => "nothing newer".to_owned(),
    }
}

/// "1 package", "12 packages".
fn count(n: usize, what: &str) -> String {
    if n == 1 {
        format!("1 {what}")
    } else {
        format!("{n} {what}s")
    }
}

/// A release `system/70-bitwarden.sh --check-updates` reports, from its `!!` lines: "bw CLI
/// 2025.2.0 is out (pinned: 2025.1.0)".
fn bitwarden_news(line: &str) -> Option<String> {
    let text = line.trim().strip_prefix("!!")?.trim();
    let text = text.split(". New pins").next().unwrap_or(text);
    Some(format!("Bitwarden: {text}"))
}

/// "Akihabara (0.2.0)", or "0.2.0" without a codename.
fn upgrade_name(target: &Target) -> String {
    match &target.codename {
        Some(codename) => format!("{} ({})", record::capitalise(codename), target.label),
        None => target.label.clone(),
    }
}

fn read_or_empty(path: &Path) -> Result<String, UpdateError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(UpdateError::io(path)(e)),
    }
}

fn write_user(path: &Path, text: &str) -> Result<(), UpdateError> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(UpdateError::io(dir))?;
    }
    write_atomic(path, text).map_err(UpdateError::io(path))
}

mod manifest;

#[cfg(test)]
mod tests;
