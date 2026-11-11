//! The update's Bitwarden phase: Bitwarden desktop moved to the newest release of its series,
//! installed as `system/70-bitwarden.sh` installs it, from the `.deb` GitHub publishes, checked
//! against the sha256 digest GitHub publishes for that asset.
//!
//! In order: the installed version from dpkg (no phase without one, as on arm64, which has no
//! `.deb`); the user's switch; GitHub's release list; the choice (`update::bitwarden`); the
//! download into `~/.local/state/kubaos/downloads`; `sha256sum` against the published digest, a
//! mismatch or a missing digest installing nothing; `apt-get install` of the file; the file
//! removed.
//!
//! # Owns
//!
//! - Running curl, sha256sum and apt for Bitwarden.
//!
//! # Not here
//!
//! - Which release, and the digest check: `update::bitwarden`. The bw CLI, which stays pinned and
//!   is only checked (`Updater::check_bitwarden`).

use std::fs;
use std::io;
use std::path::Path;

use super::DPKG_KEEP_CONFIG;
use super::Updater;
use super::program::CURL;
use super::program::Moved;
use super::program::ProgramError;
use crate::update::bitwarden;
use crate::update::bitwarden::Release;
use crate::update::event::Phase;
use crate::update::event::Program;
use crate::update::event::ProgramChange;
use crate::update::machine::Cmd;
use crate::update::machine::Machine;
use crate::update::version::Version;

impl<M: Machine> Updater<'_, M> {
    /// Bitwarden's phase, when Bitwarden desktop is installed.
    pub(super) fn update_bitwarden(&mut self) -> Option<ProgramChange> {
        let installed = self.bitwarden_installed();
        self.program_phase(Phase::Bitwarden, Program::Bitwarden, installed, Self::move_bitwarden)
    }

    fn bitwarden_installed(&mut self) -> Option<Version> {
        let query = Cmd::user("dpkg-query").args(["-W", "-f=${Version}", "bitwarden"]);
        let text = self.quiet(&query).ok()?;
        bitwarden::installed_version(&text)
    }

    fn move_bitwarden(&mut self, installed: &Version) -> Result<Moved, ProgramError> {
        self.j.step("Asking GitHub for Bitwarden's releases");
        let list =
            Cmd::user("curl")
                .args(CURL)
                .args(["-H", "Accept: application/vnd.github+json", bitwarden::RELEASES_API]);
        let text = self
            .quiet(&list)
            .map_err(ProgramError::step("asking GitHub for Bitwarden's releases"))?;
        let releases = bitwarden::parse_releases(&text).map_err(ProgramError::Releases)?;
        let choice = bitwarden::choose(installed, &releases);
        let to = match choice.update {
            Some(release) => {
                self.install_bitwarden(&release)?;
                Some(release.version)
            }
            None => None,
        };
        Ok(Moved {
            to,
            waiting: choice.waiting,
        })
    }

    /// Downloads `release`'s `.deb`, checks it, and installs it; the file goes either way.
    fn install_bitwarden(&mut self, release: &Release) -> Result<(), ProgramError> {
        let dir = self.user.update_downloads();
        fs::create_dir_all(&dir).map_err(ProgramError::io(&dir))?;
        let deb = dir.join(release.asset());
        let installed = self.download_and_install(release, &deb);
        if let Err(e) = fs::remove_file(&deb)
            && e.kind() != io::ErrorKind::NotFound
        {
            self.j.warn(format!("{} stays behind: {e}", deb.display()));
        }
        installed
    }

    fn download_and_install(&mut self, release: &Release, deb: &Path) -> Result<(), ProgramError> {
        let version = &release.version;
        self.j.step(format!("Downloading Bitwarden {version}"));
        let download = Cmd::user("curl").args(CURL).arg(release.url()).arg("-o").arg(deb);
        self.quiet(&download)
            .map_err(ProgramError::step(format!("downloading Bitwarden {version}")))?;
        self.j.step("Checking the download against GitHub's sha256 digest");
        let sum = self
            .quiet(&Cmd::user("sha256sum").arg(deb))
            .map_err(ProgramError::step("computing the download's sha256"))?;
        bitwarden::check_digest(release.digest.as_deref(), &sum).map_err(|source| ProgramError::Digest {
            version: version.clone(),
            source,
        })?;
        // apt resolves the .deb's dependencies, as the install script has it do. `_apt` can't
        // read the user's state folder, so apt reads the file as root, with a notice.
        let install = Cmd::root("apt-get")
            .args(["install", "-y", "--no-install-recommends"])
            .args(DPKG_KEEP_CONFIG)
            .arg(deb)
            .env("DEBIAN_FRONTEND", "noninteractive");
        self.j.step(format!("Installing Bitwarden {version}"));
        self.run(&install)
            .map_err(ProgramError::step(format!("installing Bitwarden {version}")))
    }
}
