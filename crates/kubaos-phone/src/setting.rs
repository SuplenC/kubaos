//! Whether the phone link is on: the file `~/.config/kubaos/desktop/phone/phone-link` exists.
//! A file in the user layer, not a key in `kubaos.d`: turning the link on or off takes effect at
//! once, with no home-manager switch, and the user unit tests the same path
//! (`ConditionPathExists`, `home/phone.nix`), so a login starts kdeconnectd only once it exists.
//!
//! Beside it, `plugins` lists the KDE Connect plugins the user allows a paired phone on top of
//! the phone link's own ([`crate::phone::LINK_PLUGINS`]), one name a line.
//!
//! # Owns
//!
//! - Reading and changing the setting, and reading the user's plugins.
//!
//! # Not here
//!
//! - Starting and stopping kdeconnectd: [`crate::kdeconnect`].

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use kubaos_user_layer::App;

/// The file that turns the phone link on, in the phone folder of the user layer.
pub const FILE: &str = "phone-link";

/// The user's extra plugins, beside it.
pub const PLUGINS: &str = "plugins";

/// Where the setting is kept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setting {
    file: PathBuf,
}

impl Setting {
    /// The setting under the config folder `config`.
    #[must_use]
    pub fn new(config: impl AsRef<Path>) -> Self {
        Setting {
            file: kubaos_user_layer::dir(config.as_ref(), App::Phone).join(FILE),
        }
    }

    /// The setting under `XDG_CONFIG_HOME`, else `~/.config`; none without either.
    pub fn from_env() -> Option<Self> {
        kubaos_user_layer::config_home().map(Setting::new)
    }

    pub fn path(&self) -> &Path {
        &self.file
    }

    pub fn is_on(&self) -> bool {
        self.file.exists()
    }

    /// The plugins the user allows on top of the phone link's own: the lines of `plugins`,
    /// leaving out blank ones and `#` comments. None without the file.
    pub fn extra_plugins(&self) -> Vec<String> {
        let Some(dir) = self.file.parent() else {
            return vec![];
        };
        fs::read_to_string(dir.join(PLUGINS))
            .unwrap_or_default()
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_string)
            .collect()
    }

    /// Turns the phone link on or off.
    ///
    /// # Errors
    ///
    /// When the file can't be made or removed.
    pub fn set(&self, on: bool) -> io::Result<()> {
        if !on {
            return match fs::remove_file(&self.file) {
                Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
                other => other,
            };
        }
        if let Some(dir) = self.file.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(&self.file, "")
    }
}

#[cfg(test)]
mod tests;
