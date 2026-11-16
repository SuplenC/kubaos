//! The machine an app's actions change: the seam between the action vocabulary and apt, dpkg,
//! systemd, logind, the groups, the User flake and the programs a `run` names.
//!
//! The actions change files themselves, at [`Host::place`] of the path they name, so a fake
//! machine is a folder; everything else goes through the [`Host`]'s methods.
//!
//! # Owns
//!
//! - [`Host`], [`HostError`] and [`UnitState`]. The real machine: [`SystemHost`].
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
pub struct UnitState {
    pub exists: bool,
    pub enabled: bool,
    pub active: bool,
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

    /// The version dpkg has of `package`, when it is installed.
    ///
    /// # Errors
    ///
    /// When dpkg can't be asked.
    fn version(&mut self, package: &str) -> Result<Option<String>, HostError>;

    /// The vendor repository `repo` set up, when there is one, then `packages` installed.
    ///
    /// # Errors
    ///
    /// When apt fails.
    fn apt_install(&mut self, repo: Option<&str>, packages: &[String]) -> Result<(), HostError>;

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

    /// Runs `argv` in `dir`, with the home and [`RUN_PATH`](super::action::RUN_PATH). `./<script>`
    /// is a script in `dir`.
    ///
    /// # Errors
    ///
    /// [`HostError::Failed`] when it exits with another code than 0, else when it can't be started.
    fn run(&mut self, argv: &[String], dir: &Path) -> Result<(), HostError>;
}
