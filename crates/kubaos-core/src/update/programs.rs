//! The user's switches for the update's program phases: whether `kubaos update` moves Bitwarden
//! desktop and Nix on within their series. Each is on unless its flag file is in the user layer's
//! `desktop/update/` folder: `no-bitwarden`, `no-nix`, no contents.
//!
//! # Owns
//!
//! - Reading and writing the flag files ([`Switches`]).
//!
//! # Not here
//!
//! - The phases: `updater/bitwarden.rs` and `updater/nix.rs`. The menu's state: `kotodama-kubaos`'
//!   `states.rs`.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use kubaos_user_layer::App;

use super::event::Program;

/// The flag file that turns `program`'s updates off.
pub fn flag(program: Program) -> &'static str {
    match program {
        Program::Bitwarden => "no-bitwarden",
        Program::Nix => "no-nix",
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

    /// The flag file of `program`.
    pub fn path(&self, program: Program) -> PathBuf {
        self.dir.join(flag(program))
    }

    /// Whether the update moves `program` on.
    pub fn is_on(&self, program: Program) -> bool {
        !self.path(program).exists()
    }

    /// Turns `program`'s updates on: its flag file goes.
    ///
    /// # Errors
    ///
    /// When the flag file is there and can't be removed.
    pub fn turn_on(&self, program: Program) -> io::Result<()> {
        match fs::remove_file(self.path(program)) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }

    /// Turns `program`'s updates off: its flag file is made, with its folder.
    ///
    /// # Errors
    ///
    /// When the folder or the file can't be made.
    pub fn turn_off(&self, program: Program) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        fs::write(self.path(program), "")
    }
}

#[cfg(test)]
mod tests;
