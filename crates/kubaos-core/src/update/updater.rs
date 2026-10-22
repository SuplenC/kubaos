//! What `kubaos update`, `upgrade`, `rollback`, `channel` and the daily check do, step by step,
//! through a [`Machine`].
//!
//! An update, in order: the lock and a free-space check; fetching `origin` into the checkout,
//! which is the network check; the release policy's pick; showing it and asking; a snapper
//! snapshot, the user's `flake.lock` saved and the update recorded in the history; apt; moving
//! `release`; the Hyprland pin and the system migrations; `nix flake update` of the user flake and
//! the switch; relinking what `system/` took from KubaOS's nixpkgs; the Bitwarden check; the user
//! migrations; and the new release record. Packages first, then migrations, which are written
//! against the new packages.
//!
//! # Owns
//!
//! - [`Updater`] and the order of its steps, and [`UpdateError`].
//!
//! # Not here
//!
//! - Which release to move to: `policy.rs`. The files' formats: `record.rs`, `notice.rs`.
//! - Running programs and the switch: `machine.rs` and `switch.rs`.

use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use std::fs::TryLockError;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use super::git::GitRefs;
use super::git::RemoteRefs;
use super::hyprland;
use super::machine::Cmd;
use super::machine::Machine;
use super::machine::MachineError;
use super::migrations;
use super::migrations::Migration;
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
use super::version::Version;
use crate::engine::files::utc_datetime;
use crate::state::Paths;
use crate::state::write_atomic;

/// The space an update needs free on `/` and `/nix`: a new home-manager generation, a kernel and
/// apt's downloads.
pub const MIN_FREE_BYTES: u64 = 3 * 1024 * 1024 * 1024;

/// How many commits of a release the summary lists.
const LOG_LINES: usize = 20;

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

/// How an update goes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UpdateOptions {
    /// Don't ask before changing anything.
    pub yes: bool,
    /// Update packages only: don't move `release` or run migrations.
    pub packages_only: bool,
}

/// What the apply step does on top of packages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Update,
    Upgrade,
}

/// Updates, upgrades and rolls back one machine for one user.
#[derive(Debug)]
pub struct Updater<'a, M: Machine> {
    m: &'a mut M,
    sys: SystemPaths,
    user: Paths,
    now: SystemTime,
}

impl<'a, M: Machine> Updater<'a, M> {
    /// The updater of the machine at `sys`, for the user at `user`, as of `now`.
    #[must_use]
    pub fn new(m: &'a mut M, sys: SystemPaths, user: Paths, now: SystemTime) -> Self {
        Self { m, sys, user, now }
    }

    fn date(&self) -> String {
        utc_datetime(self.unix())
    }

    fn unix(&self) -> u64 {
        self.now.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
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
        let _lock = self.lock()?;
        self.check_space()?;
        let installed = self.installed()?;
        self.m.say(&format!("Installed: KubaOS {}", installed.describe()));
        let pick = self.fetch_and_pick(&installed)?;
        if let Some(upgrade) = &pick.upgrade {
            self.m.say(&format!("KubaOS {} is available: kubaos upgrade", upgrade_name(upgrade)));
        }
        let target = if opts.packages_only { None } else { pick.update };
        if target.is_none() && !opts.packages_only {
            self.m.say("KubaOS is at the newest release of its channel: updating packages");
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
        let _lock = self.lock()?;
        let installed = self.installed()?;
        if self.channel(&installed) == Channel::Edge {
            return Err(UpdateError::EdgeUpgrade);
        }
        self.check_space()?;
        let pick = self.fetch_and_pick(&installed)?;
        if let Some(patch) = pick.update {
            return Err(UpdateError::PatchFirst { label: patch.label });
        }
        let target = pick.upgrade.ok_or(UpdateError::NoUpgrade)?;
        self.m.say(&format!("Upgrade: KubaOS {} → {}", installed.describe(), upgrade_name(&target)));
        let opts = UpdateOptions {
            yes: false,
            packages_only: false,
        };
        self.apply(&installed, Some(&target), opts, Kind::Upgrade)
    }

    /// Moves `release` back to where the last update found it, puts back the user's `flake.lock`
    /// of then, and switches. Migration markers stay: migrations only go forward.
    ///
    /// # Errors
    ///
    /// [`UpdateError::NothingToRollBack`] with no history, else the step that failed.
    pub fn rollback(&mut self, yes: bool) -> Result<(), UpdateError> {
        let _lock = self.lock()?;
        let installed = self.installed()?;
        let path = self.sys.history();
        let mut history = parse_history(&read_or_empty(&path)?);
        let entry = history.pop().ok_or(UpdateError::NothingToRollBack { path: path.clone() })?;
        self.m.say(&format!(
            "Roll back the update of {}: KubaOS {} ({}) → {} ({})",
            entry.date,
            entry.to_version,
            short(&entry.to_commit),
            entry.from_version,
            short(&entry.from_commit)
        ));
        let lock = entry.lock.as_ref().filter(|lock| lock.is_file());
        match lock {
            Some(lock) => self.m.say(&format!("Your flake.lock goes back to {}", lock.display())),
            None => self.m.warn("that update saved no flake.lock: your packages stay where they are"),
        }
        if !yes && !self.m.confirm("Roll back?") {
            return Err(UpdateError::Declined);
        }
        self.snapshot(&format!("before kubaos rollback {}", self.date()));
        if entry.from_commit != installed.commit {
            self.move_release(&entry.from_commit)?;
        }
        if let Some(lock) = lock {
            let dest = self.user.flake_lock();
            fs::copy(lock, &dest).map_err(UpdateError::io(&dest))?;
        }
        self.m.switch().map_err(UpdateError::step("switching your home"))?;
        self.relink()?;
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
        self.m.say(&format!("Rolled back to KubaOS {}", rolled_back.describe()));
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
            Channel::Edge => self.m.say("Channel: edge. kubaos update follows origin/main from now on."),
            Channel::Stable => self.m.say(
                "Channel: stable. kubaos update moves to the next verified release at or above this one.",
            ),
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
        let installed = self.installed()?;
        let channel = self.channel(&installed);
        let local = self.refs();
        let url = local.origin_url().ok_or_else(|| UpdateError::NoRelease {
            path: self.sys.checkout(),
        })?;
        let refs = RemoteRefs::fetch(&url, local).map_err(|e| UpdateError::Policy(PolicyError::Refs(e)))?;
        let pick = policy::for_channel(channel)
            .pick(&installed, &refs)
            .map_err(UpdateError::Policy)?;
        let available = Available::from_pick(&pick);
        self.write_available(&available)?;
        let notified_path = self.user.notified();
        let mut notified = parse_notified(&read_or_empty(&notified_path)?);
        for (key, text) in available.notifications(&notified.clone()) {
            let cmd = Cmd::user("notify-send").args(["--app-name=KubaOS", "--urgency=low", "KubaOS", &text]);
            match self.m.output(&cmd) {
                Ok(_) => {
                    notified.insert(key);
                }
                Err(e) => self.m.warn(&format!("notifying: {e}")),
            }
        }
        let text: String = notified.iter().map(|key| format!("{key}\n")).collect();
        write_user(&notified_path, &text)?;
        Ok(available)
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
        self.m.output(&fetch).map_err(UpdateError::Offline)?;
        let channel = self.channel(installed);
        let pick = policy::for_channel(channel)
            .pick(installed, &self.refs())
            .map_err(UpdateError::Policy)?;
        for note in &pick.notes {
            self.m.warn(note);
        }
        Ok(pick)
    }

    /// The steps shared by an update and an upgrade, from the summary on.
    fn apply(
        &mut self,
        installed: &Installed,
        target: Option<&Target>,
        opts: UpdateOptions,
        kind: Kind,
    ) -> Result<(), UpdateError> {
        let flake = self.user.flake().to_path_buf();
        if !flake.join("flake.nix").is_file() {
            return Err(UpdateError::NoUserFlake { path: flake });
        }
        self.summarise(installed, target)?;
        // An upgrade always asks: it moves to a new Ubuntu-and-nixpkgs pairing.
        let ask = !opts.yes || kind == Kind::Upgrade;
        if ask && !self.m.confirm("Update now?") {
            return Err(UpdateError::Declined);
        }

        let to = target.map_or_else(|| installed.version.to_string(), |t| t.label.clone());
        self.snapshot(&format!("before kubaos {} {} ({} → {to})", kind_word(kind), self.date(), installed.version));
        let lock = self.save_lock()?;
        self.record_history(installed, target, lock)?;

        self.m
            .stream(&Cmd::root("apt-get").args(["upgrade", "--with-new-pkgs", "-y"]))
            .map_err(UpdateError::step("upgrading apt packages"))?;

        if let Some(target) = target {
            self.move_release(&target.commit)?;
        }
        if !opts.packages_only {
            self.apply_hyprland_pin()?;
            self.run_system_migrations()?;
        }

        self.m
            .stream(&Cmd::user("nix").arg("flake").arg("update").arg("--flake").arg(&flake))
            .map_err(UpdateError::step("updating your flake's inputs (nix flake update)"))?;
        self.m.switch().map_err(UpdateError::step("switching your home"))?;
        self.relink()?;
        self.check_bitwarden();
        if !opts.packages_only {
            self.run_user_migrations()?;
        }

        let now = self.record(installed, target)?;
        self.clear_available(&now)?;
        self.m.say(&format!("Done: KubaOS {}", now.describe()));
        if self.sys.root().join("var/run/reboot-required").exists() {
            self.m.warn("reboot required: an updated package asks for it");
        }
        Ok(())
    }

    /// Shows what the update brings: the release's commits, the apt packages, and whether the
    /// Hyprland pin changes.
    fn summarise(&mut self, installed: &Installed, target: Option<&Target>) -> Result<(), UpdateError> {
        self.m
            .output(&Cmd::root("apt-get").args(["update", "-q"]))
            .map_err(UpdateError::step("refreshing apt's package lists"))?;
        if let Some(target) = target {
            self.m.say(&format!("KubaOS {} → {}", installed.version, target.label));
            let refs = self.refs();
            for line in refs.log(&installed.commit, &target.commit, LOG_LINES) {
                self.m.say(&format!("  {line}"));
            }
            let pin = refs.file_at(&target.commit, record::RELEASE_PIN);
            if pin.is_some() && pin != fs::read_to_string(self.sys.apt_pin()).ok() {
                self.m.say("The Hyprland pin changes: Hyprland is installed at the release's versions.");
            }
            let markers = self.sys.markers();
            let new: Vec<String> = refs
                .file_list(&target.commit, "migrations/system")
                .into_iter()
                .filter(|name| migrations::stamp_of(name).is_some() && !markers.join(name).exists())
                .collect();
            if !new.is_empty() {
                self.m.say(&format!("System migrations to run: {}", new.join(", ")));
            }
        }
        let simulated = self
            .m
            .output(&Cmd::user("apt-get").args(["-s", "-q", "upgrade", "--with-new-pkgs"]))
            .unwrap_or_default();
        let packages = simulated.lines().filter(|line| line.starts_with("Inst ")).count();
        self.m.say(&format!("apt: {packages} packages to upgrade"));
        Ok(())
    }

    fn snapshot(&mut self, description: &str) {
        if !self.sys.root().join("etc/snapper/configs/root").is_file() {
            self.m.warn("snapper isn't set up: no snapshot to go back to this time");
            return;
        }
        let cmd = Cmd::root("snapper").args([
            "-c",
            "root",
            "create",
            "--cleanup-algorithm",
            "number",
            "--description",
            description,
        ]);
        match self.m.output(&cmd) {
            Ok(_) => self.m.say(&format!("Snapshot: {description}")),
            Err(e) => self.m.warn(&format!("no snapshot: {e}")),
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

    fn move_release(&mut self, commit: &str) -> Result<(), UpdateError> {
        let cmd = Cmd::root("git")
            .arg("-C")
            .arg(self.sys.checkout())
            .args(["checkout", "--quiet", "-B", record::RELEASE_BRANCH, commit]);
        self.m
            .output(&cmd)
            .map_err(UpdateError::step(format!("moving release to {}", short(commit))))?;
        self.m.say(&format!("release is at {}", short(commit)));
        Ok(())
    }

    /// Installs the release's Hyprland pin when it differs from the one apt has: the pinned
    /// versions exactly, then the whole stack held again, whatever happened.
    fn apply_hyprland_pin(&mut self) -> Result<(), UpdateError> {
        let Ok(wanted) = fs::read_to_string(self.sys.checkout().join(record::RELEASE_PIN)) else {
            return Ok(());
        };
        if fs::read_to_string(self.sys.apt_pin()).ok().as_deref() == Some(wanted.as_str()) {
            return Ok(());
        }
        let arch = self
            .m
            .output(&Cmd::user("dpkg").arg("--print-architecture"))
            .unwrap_or_default();
        if arch.trim() != "amd64" {
            self.m.say("Hyprland pin: skipped, Hyprland comes from KubaOS's nixpkgs on this architecture");
            return Ok(());
        }
        let pins = hyprland::parse(&wanted);
        self.write_root(&self.sys.apt_pin(), &wanted)?;
        let held: BTreeSet<String> = self
            .m
            .output(&Cmd::user("apt-mark").arg("showhold"))
            .unwrap_or_default()
            .lines()
            .map(str::trim)
            .filter(|p| hyprland::is_held_stack(p))
            .map(str::to_owned)
            .collect();
        if !held.is_empty() {
            self.m
                .output(&Cmd::root("apt-mark").arg("unhold").args(&held))
                .map_err(UpdateError::step("unholding the Hyprland stack"))?;
        }
        let install = Cmd::root("apt-get")
            .args(["install", "-y", "--no-install-recommends", "--allow-downgrades"])
            .args(pins.iter().map(hyprland::Pin::install_arg));
        // TODO(#14): when the PPA no longer has a pinned version, install the release's .debs,
        // checked against its signed SHA256SUMS; releases don't attach them yet.
        let installed = self
            .m
            .stream(&install)
            .map_err(UpdateError::step("installing the pinned Hyprland"));
        let mut hold = held;
        hold.extend(self.installed_hypr_packages());
        let rehold = if hold.is_empty() {
            Ok(())
        } else {
            self.m
                .output(&Cmd::root("apt-mark").arg("hold").args(&hold))
                .map(drop)
                .map_err(UpdateError::step("holding the Hyprland stack again"))
        };
        installed?;
        rehold?;
        self.m.say(&format!("Hyprland pin applied, {} packages held", hold.len()));
        if self.m.output(&Cmd::user("Hyprland").arg("--verify-config")).is_err() {
            self.m
                .warn("Hyprland --verify-config reports problems: run it and fix your config before logging out");
        }
        Ok(())
    }

    fn installed_hypr_packages(&mut self) -> Vec<String> {
        self.m
            .output(&Cmd::user("dpkg-query").args(["-W", "-f", "${db:Status-Status} ${Package}\n"]))
            .unwrap_or_default()
            .lines()
            .filter_map(|line| line.strip_prefix("installed "))
            .filter(|p| hyprland::is_held_stack(p))
            .map(str::to_owned)
            .collect()
    }

    fn run_system_migrations(&mut self) -> Result<(), UpdateError> {
        let dir = self.sys.system_migrations();
        let pending = migrations::pending(&dir, &self.sys.markers()).map_err(UpdateError::io(&dir))?;
        for migration in pending {
            self.m.say(&format!("System migration {}", migration.name));
            let cmd = Cmd::root("bash")
                .args(["-euo", "pipefail"])
                .arg(&migration.path)
                .env("KUBAOS_DIR", self.sys.checkout())
                .cwd(self.sys.checkout());
            self.m.stream(&cmd).map_err(|source| UpdateError::Migration {
                name: migration.name.clone(),
                source,
            })?;
            let marker = self.sys.markers().join(&migration.name);
            self.write_root(&marker, &format!("{}\n", self.date()))?;
        }
        Ok(())
    }

    fn run_user_migrations(&mut self) -> Result<(), UpdateError> {
        let dir = self.sys.user_migrations();
        let markers = self.user.migration_markers();
        let pending: Vec<Migration> = migrations::pending(&dir, &markers).map_err(UpdateError::io(&dir))?;
        for migration in pending {
            self.m.say(&format!("Your migration {}", migration.name));
            let cmd = Cmd::user("bash")
                .args(["-euo", "pipefail"])
                .arg(&migration.path)
                .env("KUBAOS_DIR", self.sys.checkout())
                .env("KUBAOS_USER_FLAKE", self.user.flake())
                .env("KUBAOS_BACKUPS", self.user.backups())
                .cwd(self.user.home());
            self.m.stream(&cmd).map_err(|source| UpdateError::Migration {
                name: migration.name.clone(),
                source,
            })?;
            write_user(&markers.join(&migration.name), &format!("{}\n", self.date()))?;
        }
        Ok(())
    }

    /// Points what `system/` linked from KubaOS's nixpkgs (the lock screen's PAM modules, the
    /// recorder's helper, the wallpaper sets) at the release's builds.
    fn relink(&mut self) -> Result<(), UpdateError> {
        let script = self.sys.checkout().join("system/relink.sh");
        if !script.is_file() {
            return Ok(());
        }
        self.m
            .stream(&Cmd::user(&script))
            .map_err(UpdateError::step("relinking the lock screen's PAM modules and the recorder"))
    }

    /// Reports newer Bitwarden releases, which are pinned downloads, not apt packages.
    fn check_bitwarden(&mut self) {
        let script = self.sys.checkout().join("system/70-bitwarden.sh");
        if script.is_file() && self.m.stream(&Cmd::user(&script).arg("--check-updates")).is_err() {
            self.m.warn("the Bitwarden release check failed");
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
        };
        self.write_available(&kept)
    }

    fn write_available(&self, available: &Available) -> Result<(), UpdateError> {
        let path = self.user.update_available();
        if available.is_empty() {
            return match fs::remove_file(&path) {
                Err(e) if e.kind() != io::ErrorKind::NotFound => Err(UpdateError::io(&path)(e)),
                _ => Ok(()),
            };
        }
        write_user(&path, &available.to_text())
    }

    fn write_root(&mut self, path: &Path, text: &str) -> Result<(), UpdateError> {
        let cmd = Cmd::root("sh").args(["-c", WRITE_FILE, "sh"]).arg(path).stdin(text);
        self.m
            .output(&cmd)
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

fn kind_word(kind: Kind) -> &'static str {
    match kind {
        Kind::Update => "update",
        Kind::Upgrade => "upgrade",
    }
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

#[cfg(test)]
mod tests;
