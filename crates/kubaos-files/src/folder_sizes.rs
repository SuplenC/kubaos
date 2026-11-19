//! Whether Files' Total Size column adds up each folder: on, unless the file
//! `~/.config/kubaos/desktop/files/no-folder-sizes` exists. The column is a nautilus-python
//! extension, `home/files/kubaos-folder-sizes.py`, which checks for the file on each folder it
//! shows, so a change takes effect on the next folder opened.
//!
//! # Owns
//!
//! - Reading and changing the setting.
//!
//! # Not here
//!
//! - The column, and adding a folder up: `home/files/kubaos-folder-sizes.py`.
//! - The Command menu's Items that change it: `config/kotodama/kubaos.toml`.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use kubaos_user_layer::App;

/// The file that turns the folder sizes off, in the Files folder of the user layer.
const OFF: &str = "no-folder-sizes";

/// Where the setting is kept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setting {
    off: PathBuf,
}

impl Setting {
    /// The setting under the config folder `config`.
    #[must_use]
    pub fn new(config: impl AsRef<Path>) -> Self {
        Setting {
            off: kubaos_user_layer::dir(config.as_ref(), App::Files).join(OFF),
        }
    }

    /// The file whose existence turns the folder sizes off.
    pub fn path(&self) -> &Path {
        &self.off
    }

    pub fn is_on(&self) -> bool {
        !self.off.exists()
    }

    /// Turns the folder sizes on or off.
    ///
    /// # Errors
    ///
    /// When the file can't be made or removed.
    pub fn set(&self, on: bool) -> io::Result<()> {
        if on {
            return match fs::remove_file(&self.off) {
                Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
                other => other,
            };
        }
        if let Some(dir) = self.off.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(&self.off, "")
    }
}

#[cfg(test)]
mod tests;
