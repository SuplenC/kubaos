//! The machine an app's actions change: the seam between the action vocabulary and apt, dpkg,
//! systemd, logind, the groups, the User flake and the programs a `run` names.
//!
//! The actions change files themselves, at [`Host::place`] of the path they name, so a fake
//! machine is a folder; everything else goes through the [`Host`]'s methods.
//!
//! # Owns
//!
//! - [`Host`], [`HostError`], [`UnitState`], [`AptPlan`], [`AptSimulation`], [`Downgrade`],
//!   [`Removal`], [`Known`], [`DkmsStatus`], [`DkmsInstall`], [`Mok`] and [`SecureBoot`]. The real
//!   machine: [`SystemHost`].
//!
//! # Not here
//!
//! - What an action asks of the host: `action/perform.rs`.

#[cfg(test)]
pub mod fake;
pub mod system;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsString;
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
    #[error("the certificate DKMS signs with can't be worked out from its configuration: {why}")]
    Mok { why: String },
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
    /// The installed packages it would move to a lower version.
    pub downgrades: BTreeSet<String>,
}

/// Whether an install may move a package dpkg has to a lower version: only to replace another
/// build of the name, never as apt's default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Downgrade {
    Refuse,
    Allow,
}

/// How a tarball is compressed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Compression {
    Gzip,
    Zstd,
    Xz,
}

/// tar's arguments that unpack `archive` into `into`, as [`Host::untar`] says.
pub fn untar_args(archive: &Path, into: &Path, compression: Compression, strip: u32) -> Vec<OsString> {
    let mut args: Vec<OsString> = vec!["-x".into(), compression.tar_option().into(), "-f".into()];
    args.push(archive.into());
    args.push("-C".into());
    args.push(into.into());
    args.push(format!("--strip-components={strip}").into());
    args.push("--no-same-owner".into());
    args.push("--no-same-permissions".into());
    args
}

impl Compression {
    /// tar's option that reads it.
    pub fn tar_option(self) -> &'static str {
        match self {
            Self::Gzip => "--gzip",
            Self::Zstd => "--zstd",
            Self::Xz => "--xz",
        }
    }
}

/// How an apt removal leaves a package's configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Removal {
    /// Kept (`apt-get remove`).
    KeepConfig,
    /// Purged with the package (`apt-get purge`).
    Purge,
    /// Purged, Essential and Protected packages too (`--allow-remove-essential`): only for a
    /// foreign architecture's packages an app brought, which nothing of the machine's needs.
    PurgeEssential,
}

/// How dpkg has a package.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Known {
    /// Installed, or in any state but not installed and configuration left.
    Installed,
    /// Removed, its configuration files left (`rc`).
    ConfigFiles,
}

/// What a `dkms install` for one kernel did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DkmsInstall {
    Installed,
    /// The module's `dkms.conf` leaves the kernel out.
    Excluded,
}

/// What DKMS has of a module at a version.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DkmsStatus {
    /// It is in DKMS's tree (`dkms add`), so a new kernel builds it.
    pub added: bool,
    /// The kernels it is installed for.
    pub installed: BTreeSet<String>,
}

/// The Machine Owner Key DKMS signs modules with: its certificate, and that certificate's serial
/// as `modinfo` names the key that signed a module (`sig_key`), uppercase hex pairs with `:`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mok {
    pub certificate: PathBuf,
    pub serial: String,
}

/// Whether the firmware loads only signed modules, and whether it takes the MOK's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SecureBoot {
    /// Off: any module loads.
    #[default]
    Off,
    /// On, and the MOK is enrolled.
    Enrolled,
    /// On, and the MOK is queued for enrolment at the next boot (`mokutil --import`).
    Queued,
    /// On, and the MOK is neither enrolled nor queued: modules signed with it don't load.
    NotEnrolled,
    /// On, with no MOK to sign with, or what mokutil said wasn't one of the above.
    Unknown,
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
    /// up, as [`Repo::in_place`](crate::apt::manifest::Repo::in_place) reads it: its sources file,
    /// `<repo>.sources`, signed by the manifest's primary keyring and none but its keyrings. A
    /// vendor's own setup of the same repository (Slack's `slack.list`, the `zulip-desktop.
    /// sources` Zulip's package ships, signed by its own keyring) isn't.
    ///
    /// # Errors
    ///
    /// When the package manifest can't be read.
    fn apt_repo_ready(&mut self, repo: &str) -> Result<bool, HostError>;

    /// Which of the vendor repositories `repos` of the package manifest have a key, or a next
    /// key, that [`Host::apt_repo`] would fetch again (`fetch_key`'s rule: missing, not the pinned
    /// fingerprint's alone, or expiring within 30 days), read from the keyrings, which everyone
    /// may, with nothing changed. A repository the manifest lacks is never due.
    ///
    /// # Errors
    ///
    /// When the keyrings can't be read.
    fn apt_keys_due(&mut self, repos: &[String]) -> Result<BTreeSet<String>, HostError>;

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

    /// `packages` removed, their configuration kept or purged as `removal` says; packages with
    /// only their configuration left are purged by a purge.
    ///
    /// # Errors
    ///
    /// When apt fails, or would remove a package it marks Essential or Protected but under
    /// [`Removal::PurgeEssential`].
    fn apt_remove(&mut self, packages: &[String], removal: Removal) -> Result<(), HostError>;

    /// Which of `packages`, installed, apt won't remove without `--allow-remove-essential`:
    /// those whose control fields say `Essential: yes` or `Protected: yes`, of any
    /// architecture, as apt marks them important.
    ///
    /// # Errors
    ///
    /// When dpkg can't be asked.
    fn important(&mut self, packages: &[String]) -> Result<BTreeSet<String>, HostError>;

    /// `url` downloaded to `to`, kept only when its sha256 is `sha256`.
    ///
    /// # Errors
    ///
    /// [`HostError::Sha256`] for another download, else the download that failed.
    fn fetch(&mut self, url: &str, sha256: &str, to: &Path) -> Result<(), HostError>;

    /// The tarball `archive`, compressed with `compression`, unpacked into the folder `into`, its
    /// first `strip` folders left out, owned by whoever unpacks it, with no mode of the archive's.
    ///
    /// # Errors
    ///
    /// When tar fails, or can't be started.
    fn untar(&mut self, archive: &Path, into: &Path, compression: Compression, strip: u32) -> Result<(), HostError>;

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

    /// `unit`, which runs, restarted.
    ///
    /// # Errors
    ///
    /// When systemctl fails.
    fn restart(&mut self, scope: Scope, unit: &str) -> Result<(), HostError>;

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

    /// The architectures dpkg installs packages of beside the machine's own.
    ///
    /// # Errors
    ///
    /// When dpkg can't be asked.
    fn foreign_arches(&mut self) -> Result<BTreeSet<String>, HostError>;

    /// dpkg taking packages of `arch` too, or no longer: `dpkg --add-architecture` or
    /// `--remove-architecture`, which dpkg refuses while it has a package of it.
    ///
    /// # Errors
    ///
    /// When dpkg fails.
    fn set_foreign_arch(&mut self, arch: &str, on: bool) -> Result<(), HostError>;

    /// The packages of `arch` dpkg knows, as the engine names them (`libc6:i386`), each
    /// installed or with only its configuration left: while one is there, dpkg keeps the
    /// architecture.
    ///
    /// # Errors
    ///
    /// When dpkg can't be asked.
    fn arch_packages(&mut self, arch: &str) -> Result<BTreeMap<String, Known>, HostError>;

    /// apt's lists read again from the sources (`apt-get update`).
    ///
    /// # Errors
    ///
    /// When apt fails.
    fn apt_update(&mut self) -> Result<(), HostError>;

    /// The kernels DKMS builds modules for: those installed with their headers
    /// (`/lib/modules/<kernel>/build`).
    ///
    /// # Errors
    ///
    /// When the kernels' folder can't be read.
    fn kernels(&mut self) -> Result<BTreeSet<String>, HostError>;

    /// What DKMS has of `module` at `version` (`dkms status`).
    ///
    /// # Errors
    ///
    /// When dkms can't be asked.
    fn dkms_status(&mut self, module: &str, version: &str) -> Result<DkmsStatus, HostError>;

    /// `module` at `version`, whose source is in `/usr/src/<module>-<version>`, added to DKMS's
    /// tree (`dkms add`).
    ///
    /// # Errors
    ///
    /// When dkms fails.
    fn dkms_add(&mut self, module: &str, version: &str) -> Result<(), HostError>;

    /// `module` at `version` built, signed as DKMS signs, and installed for `kernel`, over what
    /// it installed before (`dkms install --force`); or not, for a kernel its `dkms.conf` leaves
    /// out (`BUILD_EXCLUSIVE_*`, dkms's exit code 77).
    ///
    /// # Errors
    ///
    /// When dkms fails.
    fn dkms_install(&mut self, module: &str, version: &str, kernel: &str) -> Result<DkmsInstall, HostError>;

    /// `module` at `version` taken out of DKMS's tree and every kernel (`dkms remove --all`).
    ///
    /// # Errors
    ///
    /// When dkms fails.
    fn dkms_remove(&mut self, module: &str, version: &str) -> Result<(), HostError>;

    /// The key that signed `module` as installed for `kernel`, as `modinfo` names it
    /// (`sig_key`); none for one unsigned, or not there.
    ///
    /// # Errors
    ///
    /// When modinfo can't be started.
    fn module_signer(&mut self, module: &str, kernel: &str) -> Result<Option<String>, HostError>;

    /// The MOK DKMS signs modules for `kernel` with, as its configuration names it (which may
    /// name it through `$kernelver`); none until DKMS has made one.
    ///
    /// # Errors
    ///
    /// When the configuration names it in a way that can't be worked out, or the certificate
    /// can't be read.
    fn mok(&mut self, kernel: &str) -> Result<Option<Mok>, HostError>;

    /// Whether Secure Boot is on, and whether it takes `certificate`, the MOK's.
    ///
    /// # Errors
    ///
    /// When mokutil can't be started.
    fn secure_boot(&mut self, certificate: Option<&Path>) -> Result<SecureBoot, HostError>;

    /// How many users the loaded kernel module `module` has, the modules and handles holding it
    /// and the devices bound to its drivers (a HID driver's controllers, which its count leaves
    /// out); none when it isn't loaded.
    ///
    /// # Errors
    ///
    /// When `/sys/module` can't be read.
    fn module_users(&mut self, module: &str) -> Result<Option<u32>, HostError>;

    /// The loaded kernel module `module`, which nothing uses, unloaded (`modprobe -r`).
    ///
    /// # Errors
    ///
    /// When modprobe fails.
    fn unload(&mut self, module: &str) -> Result<(), HostError>;

    /// Runs `argv` in `dir` with [`RUN_PATH`](super::action::RUN_PATH): as the user, with their
    /// environment and home; as root, with root's own environment and none of the user's.
    /// `./<script>` is a script in `dir`.
    ///
    /// # Errors
    ///
    /// [`HostError::Failed`] when it exits with another code than 0, else when it can't be started.
    fn run(&mut self, argv: &[String], dir: &Path) -> Result<(), HostError>;
}
