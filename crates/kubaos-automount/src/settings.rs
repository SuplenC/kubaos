//! Whether drives are mounted when they're plugged in: on, unless the file
//! `~/.config/kubaos/files/no-automount` exists. A file and not a key in a config file, so the
//! setting needs no parser and lives in the user layer next to the other Files settings.
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
use std::path::PathBuf;

/// The file that turns automount off, under the config folder.
const OFF: &str = "kubaos/files/no-automount";

/// Where the setting is kept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    off: PathBuf,
}

impl Settings {
    /// The setting under the config folder `config`.
    #[must_use]
    pub fn new(config: impl Into<PathBuf>) -> Self {
        Settings {
            off: config.into().join(OFF),
        }
    }

    /// The setting under `XDG_CONFIG_HOME`, else `~/.config`; none without either.
    pub fn from_env() -> Option<Self> {
        let var = |name| {
            std::env::var_os(name)
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
        };
        var("XDG_CONFIG_HOME")
            .or_else(|| var("HOME").map(|home| home.join(".config")))
            .map(Settings::new)
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
