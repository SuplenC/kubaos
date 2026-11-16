//! The machine an app's actions change: the seam between the action vocabulary and apt, dpkg,
//! systemd, logind, the groups, the User flake and the programs a `run` names.
//!
//! The actions change files themselves, at [`Host::place`] of the path they name, so a fake
//! machine is a folder; everything else goes through the [`Host`]'s methods.
//!
//! # Owns
//!
//! - [`Host`], [`HostError`], [`UnitState`], [`AptPlan`], [`AptSimulation`] and [`Downgrade`]. The
//!   real machine: [`SystemHost`].
//!
//! # Not here
//!
//! - What an action asks of the host: `action/perform.rs`.

#[cfg(test)]
pub mod fake;
pub mod system;

use std::collections::BTreeSet;
use std::io;
use std::path::Path;
use std::path::PathBuf;

pub use self::system::SystemHost;
use super::action::Scope;
use crate::unfree::Unfree;

/// What the machine refused.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HostError {
    #[error("starting {program}")]
    Spawn {
        program: String,
        #[source]
        source: io::Error,
    },
    #[error("`{command}` exited with {}{}", code.map_or_else(|| "a signal".to_owned(), |c| format!("code {c}")),
        if stderr.is_empty() { String::new() } else { format!(": {stderr}") })]
    Failed {
        command: String,
        code: Option<i32>,
        stderr: String,
    },
    #[error("{url} is not the download its action pins (sha256 {sha256})")]
    Sha256 { url: String, sha256: String },
    #[error("{}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("the User flake's packages.toml")]
    Nix(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("the package manifest")]
    Manifest(#[source] crate::apt::manifest::ManifestError),
}

impl HostError {
    /// An I/O error on `path`.
    pub fn io(path: &Path) -> impl FnOnce(io::Error) -> Self + '_ {
        move |source| Self::Io {
            path: path.to_path_buf(),
            source,
        }
    }
}

/// Whether a unit is there, enabled and running.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Each is a yes or no systemd answers on its own."
)]
pub struct UnitState {
    pub exists: bool,
    pub enabled: bool,
    pub active: bool,
    /// A oneshot that ran since the machine started and exited cleanly: one that did its work.
    pub ran: bool,
}

/// The properties of `systemctl show` that [`UnitState::from_show`] reads.
pub const UNIT_PROPERTIES: &str =
    "LoadState,UnitFileState,ActiveState,Result,ExecMainExitTimestampMonotonic,Type,RemainAfterExit";

impl UnitState {
    /// How a unit stands from what `systemctl show --property=`[`UNIT_PROPERTIES`] says of it.
    ///
    /// Only a oneshot without `RemainAfterExit` counts as run when it is inactive after a clean
    /// exit: a long-running service that was stopped also exits cleanly, with `Result` success
    /// and an exit timestamp, and is not started.
    #[must_use]
    pub fn from_show(shown: &str) -> Self {
        let value = |key: &str| {
            shown
                .lines()
                .find_map(|line| line.strip_prefix(key)?.strip_prefix('='))
                .unwrap_or_default()
        };
        let oneshot = value("Type") == "oneshot" && value("RemainAfterExit") != "yes";
        Self {
            exists: value("LoadState") == "loaded",
            enabled: matches!(value("UnitFileState"), "enabled" | "enabled-runtime"),
            active: matches!(value("ActiveState"), "active" | "activating" | "reloading"),
            ran: oneshot
                && value("ActiveState") == "inactive"
                && value("Result") == "success"
                && value("ExecMainExitTimestampMonotonic")
                    .parse::<u64>()
                    .is_ok_and(|t| t > 0),
        }
    }

    /// Whether it is started as a unit action means it: running, or run and exited cleanly.
    pub fn started(self) -> bool {
        self.active || self.ran
    }
}

/// What apt says it would do to carry out a plan, from a simulation that changes nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AptSimulation {
    /// The installed packages it would remove.
    pub removes: BTreeSet<String>,
    /// The packages it would newly install: those asked for and what they need that isn't there.
    pub installs: BTreeSet<String>,
}

/// Whether an install may move a package dpkg has to a lower version: only to replace another
/// build of the name, never as apt's default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Downgrade {
    Refuse,
    Allow,
}

/// A change apt is asked to simulate, to learn what else it would install or remove.
#[derive(Clone, Copy, Debug)]
pub enum AptPlan<'a> {
    Install(&'a [String]),
    Deb(&'a Path),
    Remove(&'a [String]),
}

/// One machine, as the user who asked or as root acting for them.
pub trait Host {
    /// The user the run is for.
    fn user(&self) -> &str;

    /// Their home.
    fn home(&self) -> &Path;

    /// The machine's architecture, as dpkg names it.
    fn arch(&self) -> &str;

    /// Where a path an action names is on this machine: itself on a real one.
    fn place(&self, path: &Path) -> PathBuf;

    /// Which of `packages` dpkg has installed.
    ///
    /// # Errors
    ///
    /// When dpkg can't be asked.
    fn installed(&mut self, packages: &[String]) -> Result<BTreeSet<String>, HostError>;

    /// Which of `packages` apt marks installed by hand rather than pulled in for another.
    ///
    /// # Errors
    ///
    /// When apt can't be asked.
    fn manual(&mut self, packages: &[String]) -> Result<BTreeSet<String>, HostError>;

    /// The version dpkg has of `package`, when it is installed.
    ///
    /// # Errors
    ///
    /// When dpkg can't be asked.
    fn version(&mut self, package: &str) -> Result<Option<String>, HostError>;

    /// The version of `package` apt would install, from the repositories it knows now.
    ///
    /// # Errors
    ///
    /// When apt can't be asked.
    fn candidate(&mut self, package: &str) -> Result<Option<String>, HostError>;

    /// The vendor repository `repo` of the package manifest set up.
    ///
    /// # Errors
    ///
    /// When its setup fails.
    fn apt_repo(&mut self, repo: &str) -> Result<(), HostError>;

    /// Whether the vendor repository `repo` of the package manifest is set up as KubaOS sets it
    /// up: its sources file, `<repo>.sources`, signed by the manifest's keyrings, each of them
    /// there. A vendor's own setup of the same repository (Slack's `slack.list`, the
    /// `zulip-desktop.sources` Zulip's package ships, signed by its own keyring) isn't.
    ///
    /// # Errors
    ///
    /// When the package manifest can't be read.
    fn apt_repo_ready(&mut self, repo: &str) -> Result<bool, HostError>;

    /// What apt would install and remove to carry out `plan`, from a simulation that changes
    /// nothing.
    ///
    /// # Errors
    ///
    /// When apt can't simulate it.
    fn apt_simulate(&mut self, plan: AptPlan<'_>) -> Result<AptSimulation, HostError>;

    /// `packages` installed. apt moves a package it has at a higher version than the one it picks
    /// down only under [`Downgrade::Allow`].
    ///
    /// # Errors
    ///
    /// When apt fails, or would move a package down under [`Downgrade::Refuse`].
    fn apt_install(&mut self, packages: &[String], downgrade: Downgrade) -> Result<(), HostError>;

    /// A downloaded `.deb` installed.
    ///
    /// # Errors
    ///
    /// When apt fails.
    fn deb_install(&mut self, deb: &Path) -> Result<(), HostError>;

    /// `packages` removed, their configuration kept.
    ///
    /// # Errors
    ///
    /// When apt fails.
    fn apt_remove(&mut self, packages: &[String]) -> Result<(), HostError>;

    /// `url` downloaded to `to`, kept only when its sha256 is `sha256`.
    ///
    /// # Errors
    ///
    /// [`HostError::Sha256`] for another download, else the download that failed.
    fn fetch(&mut self, url: &str, sha256: &str, to: &Path) -> Result<(), HostError>;

    /// The names in the User flake's `packages.toml`.
    ///
    /// # Errors
    ///
    /// When it can't be read.
    fn nix_list(&mut self) -> Result<BTreeSet<String>, HostError>;

    /// Adds `names` to the User flake's `packages.toml`; a switch after the run installs them.
    ///
    /// # Errors
    ///
    /// When it can't be written.
    fn nix_add(&mut self, names: &[String]) -> Result<(), HostError>;

    /// Removes `names` from it.
    ///
    /// # Errors
    ///
    /// When it can't be written.
    fn nix_remove(&mut self, names: &[String]) -> Result<(), HostError>;

    /// Whether the user's nixpkgs takes unfree packages: `nixpkgs.toml` beside `packages.toml`.
    ///
    /// # Errors
    ///
    /// When it can't be read.
    fn unfree(&mut self) -> Result<Unfree, HostError>;

    /// Whether `group` exists.
    ///
    /// # Errors
    ///
    /// When the group database can't be read.
    fn group_exists(&mut self, group: &str) -> Result<bool, HostError>;

    /// Whether the user is in `group`.
    ///
    /// # Errors
    ///
    /// When the group database can't be read.
    fn member(&mut self, group: &str) -> Result<bool, HostError>;

    /// `group` created.
    ///
    /// # Errors
    ///
    /// When `groupadd` fails.
    fn group_create(&mut self, group: &str) -> Result<(), HostError>;

    /// `group` deleted.
    ///
    /// # Errors
    ///
    /// When `groupdel` fails.
    fn group_delete(&mut self, group: &str) -> Result<(), HostError>;

    /// The user in `group`, or out of it.
    ///
    /// # Errors
    ///
    /// When `usermod` or `gpasswd` fails.
    fn set_member(&mut self, group: &str, member: bool) -> Result<(), HostError>;

    /// How `unit` stands in the `scope` manager.
    ///
    /// # Errors
    ///
    /// When systemctl can't be asked.
    fn unit_state(&mut self, scope: Scope, unit: &str) -> Result<UnitState, HostError>;

    /// `systemctl daemon-reload` of the `scope` manager.
    ///
    /// # Errors
    ///
    /// When it fails.
    fn daemon_reload(&mut self, scope: Scope) -> Result<(), HostError>;

    /// `unit` enabled or disabled.
    ///
    /// # Errors
    ///
    /// When systemctl fails.
    fn set_enabled(&mut self, scope: Scope, unit: &str, enabled: bool) -> Result<(), HostError>;

    /// `unit` started or stopped.
    ///
    /// # Errors
    ///
    /// When systemctl fails.
    fn set_active(&mut self, scope: Scope, unit: &str, active: bool) -> Result<(), HostError>;

    /// Whether logind keeps the user's services running without a session.
    ///
    /// # Errors
    ///
    /// When logind can't be asked.
    fn lingers(&mut self) -> Result<bool, HostError>;

    /// Lingering on or off.
    ///
    /// # Errors
    ///
    /// When loginctl fails.
    fn set_linger(&mut self, on: bool) -> Result<(), HostError>;

    /// Runs `argv` in `dir` with [`RUN_PATH`](super::action::RUN_PATH): as the user, with their
    /// environment and home; as root, with root's own environment and none of the user's.
    /// `./<script>` is a script in `dir`.
    ///
    /// # Errors
    ///
    /// [`HostError::Failed`] when it exits with another code than 0, else when it can't be started.
    fn run(&mut self, argv: &[String], dir: &Path) -> Result<(), HostError>;
}
