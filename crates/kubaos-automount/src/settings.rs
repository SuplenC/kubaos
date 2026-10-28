//! Whether drives are mounted when they're plugged in: on, unless the file
//! `~/.config/kubaos/desktop/files/no-automount` exists. A file and not a key in a config file, so
//! the setting needs no parser and lives in the user layer next to the other Files settings.
//!
//! # Owns
//!
//! - Reading and changing the setting.
//!
//! # Not here
//!
//! - The Command menu's Items that change it: `config/kotodama/kubaos.toml`.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use kubaos_user_layer::App;

/// The file that turns automount off, in the Files folder of the user layer.
const OFF: &str = "no-automount";

/// Where the setting is kept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    off: PathBuf,
}

impl Settings {
    /// The setting under the config folder `config`.
    #[must_use]
    pub fn new(config: impl AsRef<Path>) -> Self {
        Settings {
            off: kubaos_user_layer::dir(config.as_ref(), App::Files).join(OFF),
        }
    }

    /// The setting under `XDG_CONFIG_HOME`, else `~/.config`; none without either.
    pub fn from_env() -> Option<Self> {
        kubaos_user_layer::config_home().map(Settings::new)
    }

    /// The file whose existence turns automount off.
    pub fn path(&self) -> &Path {
        &self.off
    }

    pub fn automount(&self) -> bool {
        !self.off.exists()
    }

    /// Turns automount on or off.
    ///
    /// # Errors
    ///
    /// When the file can't be made or removed.
    pub fn set_automount(&self, on: bool) -> io::Result<()> {
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
