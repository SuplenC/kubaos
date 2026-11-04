//! The update's Nix phase: the daemon's Nix moved to the newest patch release of its minor, the
//! way `system/60-nix.sh` leaves Nix set up, and the daemon restarted.
//!
//! In order: the installed version from the multi-user profile's `nix --version` (no phase
//! without that profile); the user's switch; the listing of the minor's releases, and of the next
//! minor's, which only a KubaOS release brings; the release's `fallback-paths.nix`, which must
//! name that release's Nix for this machine; `nix upgrade-nix` of the profile, pointed at that
//! file, as root; the version read again; the daemon's units copied from the new Nix where they
//! differ, as the install script copies them; and `systemctl restart nix-daemon`.
//!
//! # Owns
//!
//! - Running curl, Nix and systemctl for the Nix daemon.
//!
//! # Not here
//!
//! - Which release, and reading the listing: `update::nix`.

use std::fs;
use std::path::PathBuf;

use super::Updater;
use super::program::CURL;
use super::program::Moved;
use super::program::ProgramError;
use crate::update::event::Phase;
use crate::update::event::Program;
use crate::update::event::ProgramChange;
use crate::update::machine::Cmd;
use crate::update::machine::Machine;
use crate::update::nix;
use crate::update::version::Version;

impl<M: Machine> Updater<'_, M> {
    /// Nix's phase, when the multi-user profile has Nix.
    pub(super) fn update_nix(&mut self) -> Option<ProgramChange> {
        let installed = self.nix_installed();
        self.program_phase(Phase::Nix, Program::Nix, installed, Self::move_nix)
    }

    fn nix_profile(&self) -> PathBuf {
        self.sys.root().join(nix::PROFILE)
    }

    fn nix_bin(&self) -> PathBuf {
        self.nix_profile().join("bin/nix")
    }

    fn nix_installed(&mut self) -> Option<Version> {
        let bin = self.nix_bin();
        if !bin.is_file() {
            return None;
        }
        let text = self.quiet(&Cmd::user(bin).arg("--version")).ok()?;
        nix::installed_version(&text)
    }

    fn move_nix(&mut self, installed: &Version) -> Result<Moved, ProgramError> {
        self.j.step(format!("Listing Nix's {} releases", installed.series()));
        let listing = self
            .quiet(&curl(nix::listing_url(installed.major, installed.minor)))
            .map_err(ProgramError::step(format!(
                "listing Nix's {} releases",
                installed.series()
            )))?;
        let target = nix::newest_patch(installed, &nix::parse_listing(&listing));
        let waiting = self.next_nix_minor(installed);
        if let Some(target) = &target {
            self.upgrade_nix(target)?;
        }
        Ok(Moved { to: target, waiting })
    }

    /// The newest release of the next minor, which waits for a KubaOS release. Only a note
    /// depends on it, so a listing that fails says nothing.
    fn next_nix_minor(&mut self, installed: &Version) -> Option<Version> {
        let listing = self
            .quiet(&curl(nix::listing_url(installed.major, installed.minor + 1)))
            .ok()?;
        nix::parse_listing(&listing).into_iter().max()
    }

    fn upgrade_nix(&mut self, target: &Version) -> Result<(), ProgramError> {
        let arch = self
            .quiet(&Cmd::user("dpkg").arg("--print-architecture"))
            .map_err(ProgramError::step("reading the machine's architecture"))?;
        let system = nix::system_of(&arch).ok_or_else(|| ProgramError::Arch {
            arch: arch.trim().to_owned(),
        })?;
        let url = nix::fallback_paths_url(target);
        self.j.step(format!("Reading Nix {target}'s store paths"));
        let paths = self
            .quiet(&curl(&url))
            .map_err(ProgramError::step(format!("reading Nix {target}'s store paths")))?;
        // `nix upgrade-nix` reads the same file again; reading it first refuses one that names
        // another release, or no build for this machine, before anything changes.
        if nix::store_path(&paths, system, target).is_none() {
            return Err(ProgramError::NoStorePath {
                version: target.clone(),
                system: system.to_owned(),
            });
        }
        let upgrade = Cmd::root(self.nix_bin())
            .args([
                "--extra-experimental-features",
                "nix-command",
                "upgrade-nix",
                "--profile",
            ])
            .arg(self.nix_profile())
            .args(["--nix-store-paths-url", &url]);
        self.j.step(format!("Upgrading the Nix daemon to {target}"));
        self.run(&upgrade)
            .map_err(ProgramError::step(format!("upgrading Nix to {target}")))?;
        let found = self.nix_installed();
        if found.as_ref() != Some(target) {
            return Err(ProgramError::NotUpgraded {
                wanted: target.clone(),
                found,
            });
        }
        self.restart_nix_daemon()
    }

    /// Copies the new Nix's units over `/etc/systemd/system`'s where they differ, and restarts
    /// the daemon. `/nix` is its own subvolume, which systemd mounts after it reads the units, so
    /// the install script keeps copies there, never links into `/nix`.
    fn restart_nix_daemon(&mut self) -> Result<(), ProgramError> {
        let units = self.nix_profile().join("lib/systemd/system");
        let etc = self.sys.root().join("etc/systemd/system");
        let mut copied = false;
        for unit in nix::UNITS {
            let (src, dest) = (units.join(unit), etc.join(unit));
            let present = dest.is_file() || dest.is_symlink();
            let same = !dest.is_symlink() && fs::read(&src).ok().is_some_and(|new| fs::read(&dest).ok() == Some(new));
            if !src.is_file() || !present || same {
                continue;
            }
            self.j.step(format!("Copying the new {unit}"));
            let copy = Cmd::root("cp").arg("--remove-destination").arg(&src).arg(&dest);
            self.quiet(&copy)
                .map_err(ProgramError::step(format!("copying the new {unit}")))?;
            copied = true;
        }
        if !self.sys.root().join("run/systemd/system").is_dir() {
            self.j
                .note("No systemd here: the Nix daemon runs the new Nix from its next start");
            return Ok(());
        }
        if copied {
            self.quiet(&Cmd::root("systemctl").arg("daemon-reload"))
                .map_err(ProgramError::step("reloading systemd's units"))?;
        }
        self.j.step("Restarting the Nix daemon");
        self.quiet(&Cmd::root("systemctl").args(["restart", "nix-daemon.service"]))
            .map(drop)
            .map_err(ProgramError::step("restarting the Nix daemon"))
    }
}

fn curl(url: impl AsRef<str>) -> Cmd {
    Cmd::user("curl").args(CURL).arg(url.as_ref())
}
