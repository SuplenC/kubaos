//! The user's switches for the parts of `kubaos update` that can be turned off: moving Bitwarden
//! desktop and Nix on within their series, and firmware through fwupd. Each is on unless its flag
//! file is in the user layer's `desktop/update/` folder: `no-bitwarden`, `no-nix`, `no-firmware`,
//! no contents, so a user layer from before a switch keeps that part on.
//!
//! # Owns
//!
//! - [`Part`], and reading and writing the flag files ([`Switches`]).
//!
//! # Not here
//!
//! - The phases: `updater/bitwarden.rs`, `updater/nix.rs` and `updater/firmware.rs`. The menu's
//!   states: `kotodama-kubaos`' `states.rs`. The command: `kubaos update --turn-on|--turn-off`.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use kubaos_user_layer::App;

use super::event::Program;

/// A part of the update the user can turn off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// Bitwarden desktop, moved on within its series.
    Bitwarden,
    /// The Nix daemon, moved on within its minor.
    Nix,
    /// Firmware, through fwupd and the LVFS.
    Firmware,
}

impl Part {
    /// The flag file that turns it off.
    pub fn flag(self) -> &'static str {
        match self {
            Self::Bitwarden => "no-bitwarden",
            Self::Nix => "no-nix",
            Self::Firmware => "no-firmware",
        }
    }
}

impl From<Program> for Part {
    fn from(program: Program) -> Self {
        match program {
            Program::Bitwarden => Self::Bitwarden,
            Program::Nix => Self::Nix,
        }
    }
}

/// The switches in one user layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Switches {
    dir: PathBuf,
}

impl Switches {
    /// The switches of the user layer under the config home `config_home`, `~/.config`.
    #[must_use]
    pub fn new(config_home: &Path) -> Self {
        Self {
            dir: kubaos_user_layer::dir(config_home, App::Update),
        }
    }

    /// The flag file of `part`.
    pub fn path(&self, part: Part) -> PathBuf {
        self.dir.join(part.flag())
    }

    /// Whether the update runs `part`.
    pub fn is_on(&self, part: Part) -> bool {
        !self.path(part).exists()
    }

    /// Turns `part` on: its flag file goes.
    ///
    /// # Errors
    ///
    /// When the flag file is there and can't be removed.
    pub fn turn_on(&self, part: Part) -> io::Result<()> {
        match fs::remove_file(self.path(part)) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }

    /// Turns `part` off: its flag file is made, with its folder.
    ///
    /// # Errors
    ///
    /// When the folder or the file can't be made.
    pub fn turn_off(&self, part: Part) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        fs::write(self.path(part), "")
    }
}

#[cfg(test)]
mod tests;
