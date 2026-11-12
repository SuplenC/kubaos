//! Whether apps open on the workspace they were launched from: on, unless the file
//! `~/.config/kubaos/desktop/workspaces/no-open-where-launched` exists. A file and not a key in a
//! config file, so the setting needs no parser and sits in the user layer beside the workspace
//! groups.
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

/// The file that turns it off, in the Workspaces folder of the user layer.
const OFF: &str = "no-open-where-launched";

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
            off: kubaos_user_layer::dir(config.as_ref(), App::Workspaces).join(OFF),
        }
    }

    /// The setting under `XDG_CONFIG_HOME`, else `~/.config`; none without either.
    pub fn from_env() -> Option<Self> {
        kubaos_user_layer::config_home().map(Settings::new)
    }

    /// The file whose existence turns it off.
    pub fn path(&self) -> &Path {
        &self.off
    }

    pub fn open_where_launched(&self) -> bool {
        !self.off.exists()
    }

    /// Turns it on or off.
    ///
    /// # Errors
    ///
    /// When the file can't be made or removed.
    pub fn set_open_where_launched(&self, on: bool) -> io::Result<()> {
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
