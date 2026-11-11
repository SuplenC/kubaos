//! What the program phases share: running one as an optional phase, which the user's switch turns
//! off and whose failure is a warning, and the ways one fails.
//!
//! # Owns
//!
//! - [`Updater::program_phase`], [`Moved`] and [`ProgramError`].
//!
//! # Not here
//!
//! - The phases themselves: `bitwarden.rs` and `nix.rs` beside this file. The switches:
//!   `update::programs`.

use std::io;
use std::path::Path;
use std::path::PathBuf;

use super::Updater;
use crate::update::bitwarden::DigestError;
use crate::update::bitwarden::ReleasesError;
use crate::update::event::Phase;
use crate::update::event::Program;
use crate::update::event::ProgramChange;
use crate::update::event::ProgramResult;
use crate::update::event::chain;
use crate::update::machine::Machine;
use crate::update::machine::MachineError;
use crate::update::programs::Part;
use crate::update::programs::Switches;
use crate::update::version::Version;

/// How curl downloads in the program phases, as `system/70-bitwarden.sh` and `60-nix.sh` do: HTTPS
/// only, TLS 1.2 at least, and an HTTP error is a failure.
pub(super) const CURL: [&str; 4] = ["--proto", "=https", "--tlsv1.2", "-fsSL"];

/// Where a program phase left its program.
pub(super) struct Moved {
    /// The version it moved to; none when the installed one is the newest of its series.
    pub to: Option<Version>,
    /// The newest release of a later series, which waits for a KubaOS release.
    pub waiting: Option<Version>,
}

/// A program phase failed.
#[derive(Debug, thiserror::Error)]
pub(super) enum ProgramError {
    /// A program it ran failed.
    #[error("{what}")]
    Step {
        what: String,
        #[source]
        source: MachineError,
    },
    /// A file could not be made or read.
    #[error("using {}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// GitHub's answer is not its list of releases.
    #[error("asking GitHub for Bitwarden's releases")]
    Releases(#[source] ReleasesError),
    /// The download is not the asset GitHub published.
    #[error("the download of Bitwarden {version} isn't the one GitHub published, so nothing was installed")]
    Digest {
        version: Version,
        #[source]
        source: DigestError,
    },
    /// Nix publishes no build for the machine's architecture.
    #[error("Nix publishes no build for the {arch} architecture")]
    Arch { arch: String },
    /// The release's `fallback-paths.nix` names no build of it for the machine.
    #[error("Nix {version}'s fallback-paths.nix names no {system} build of Nix {version}")]
    NoStorePath { version: Version, system: String },
    /// Nix isn't the release it was moved to.
    #[error("Nix reads {} after the upgrade, not {wanted}", found.as_ref().map_or_else(|| "no version".to_owned(), ToString::to_string))]
    NotUpgraded { wanted: Version, found: Option<Version> },
}

impl ProgramError {
    pub(super) fn step(what: impl Into<String>) -> impl FnOnce(MachineError) -> Self {
        let what = what.into();
        move |source| Self::Step { what, source }
    }

    pub(super) fn io(path: &Path) -> impl FnOnce(io::Error) -> Self {
        let path = path.to_path_buf();
        move |source| Self::Io { path, source }
    }
}

impl<M: Machine> Updater<'_, M> {
    /// Runs `program`'s `phase` when the machine has it at version `installed`: skipped while the
    /// user's switch is off, else `work` moves it on. A failure ends the phase failed with a
    /// warning, and the update goes on: these programs aren't what the release needs to run.
    pub(super) fn program_phase(
        &mut self,
        phase: Phase,
        program: Program,
        installed: Option<Version>,
        work: impl FnOnce(&mut Self, &Version) -> Result<Moved, ProgramError>,
    ) -> Option<ProgramChange> {
        let installed = installed?;
        self.j.phase_start(phase);
        if !Switches::new(self.user.config_home()).is_on(Part::from(program)) {
            self.j
                .skip(format!("{installed}; its updates are off in your settings"));
            self.j.phase_end(true);
            return Some(ProgramChange {
                program,
                result: ProgramResult::Off,
                waiting: None,
            });
        }
        let change = match work(self, &installed) {
            Ok(moved) => self.report_moved(program, &installed, moved),
            Err(e) => {
                let error = chain(&e);
                self.j
                    .warn(format!("{}: {error}. The update goes on without it.", program.title()));
                ProgramChange {
                    program,
                    result: ProgramResult::Failed { error },
                    waiting: None,
                }
            }
        };
        self.j
            .phase_end_optional(!matches!(change.result, ProgramResult::Failed { .. }));
        Some(change)
    }

    fn report_moved(&mut self, program: Program, installed: &Version, moved: Moved) -> ProgramChange {
        if let Some(waiting) = &moved.waiting {
            self.j.note(format!(
                "{} {waiting} is out, a new series: it comes with a KubaOS release",
                program.title()
            ));
        }
        let result = if let Some(to) = moved.to {
            self.j.detail(format!("{installed} → {to}"));
            ProgramResult::Updated {
                from: installed.to_string(),
                to: to.to_string(),
            }
        } else {
            self.j.detail(format!("{installed}, the newest of its series"));
            ProgramResult::Current {
                version: installed.to_string(),
            }
        };
        ProgramChange {
            program,
            result,
            waiting: moved.waiting.map(|version| version.to_string()),
        }
    }
}
