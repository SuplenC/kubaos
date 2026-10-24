//! The keyboard settings, in the user layer: `~/.config/kubaos/keyboard/keyboard.toml`.
//!
//! ```toml
//! mac_accents = false
//! ```
//!
//! The Command menu changes them through `kubaos keyboard accents on|off`, which edits the one
//! line it changes, so the user's comments and the rest of the file stay as they are.
//!
//! # Owns
//!
//! - Whether the Mac-style accents on Left Alt are on, reading the file and changing it.
//!
//! # Not here
//!
//! - Applying them (keyd, Hyprland): `kubaos keyboard accents` in the CLI. The seed with its
//!   comments: `user-layer/keyboard/keyboard.toml`. Hyprland's own read of the file:
//!   `config/hypr/accents.lua`.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;

/// The accents are on until the user turns them off: Left Alt + e, `, u, i and n are the only
/// Alt chords they take, and macOS hands have them in their fingers.
pub const DEFAULT_MAC_ACCENTS: bool = true;

/// The settings file under the config home (`~/.config`).
const FILE: &str = "kubaos/keyboard/keyboard.toml";

const MAC_ACCENTS: &str = "mac_accents";

/// The settings file for the config home `config_home`, usually `~/.config`.
pub fn path(config_home: &Path) -> PathBuf {
    config_home.join(FILE)
}

/// The keyboard settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    /// Left Alt + e, `, u, i or n starts an accent for the next letter.
    pub mac_accents: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            mac_accents: DEFAULT_MAC_ACCENTS,
        }
    }
}

/// Why the settings file couldn't be read or changed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SettingsError {
    #[error("reading the keyboard settings")]
    Read(#[source] io::Error),
    #[error("parsing the keyboard settings")]
    Parse(#[source] toml::de::Error),
    #[error("writing the keyboard settings")]
    Write(#[source] io::Error),
}

impl Settings {
    /// Parses the settings file's text; a key left out takes its default.
    ///
    /// # Errors
    ///
    /// [`SettingsError::Parse`] for malformed TOML, an unknown key or a value of the wrong type.
    pub fn parse(text: &str) -> Result<Self, SettingsError> {
        toml::from_str(text).map_err(SettingsError::Parse)
    }

    /// Reads the settings file; a missing file is the defaults.
    ///
    /// # Errors
    ///
    /// [`SettingsError::Read`] when the file exists but can't be read, else as in
    /// [`Settings::parse`].
    pub fn load(path: &Path) -> Result<Self, SettingsError> {
        match fs::read_to_string(path) {
            Ok(text) => Self::parse(&text),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(SettingsError::Read(e)),
        }
    }
}

/// The accents on or off, as `kubaos keyboard accents` prints it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MacAccents {
    On,
    Off,
}

impl From<Settings> for MacAccents {
    fn from(settings: Settings) -> Self {
        if settings.mac_accents {
            MacAccents::On
        } else {
            MacAccents::Off
        }
    }
}

impl fmt::Display for MacAccents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MacAccents::On => "on",
            MacAccents::Off => "off",
        })
    }
}

/// Turns the accents on or off in the settings file at `path`, creating it when missing.
///
/// # Errors
///
/// [`SettingsError::Read`] or [`SettingsError::Write`] when the file can't be read or replaced.
pub fn set_mac_accents(path: &Path, accents: MacAccents) -> Result<(), SettingsError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(SettingsError::Read(e)),
    };
    let value = accents == MacAccents::On;
    let new = with_value(&text, MAC_ACCENTS, &value.to_string());
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(SettingsError::Write)?;
    }
    let tmp = path.with_extension("toml.new");
    fs::write(&tmp, new).map_err(SettingsError::Write)?;
    fs::rename(&tmp, path).map_err(SettingsError::Write)
}

/// `text` with `key`'s line set to `key = value`: the first line assigning it is replaced, or the
/// line is appended. Comments, including a commented-out example of the key, stay.
fn with_value(text: &str, key: &str, value: &str) -> String {
    let line = format!("{key} = {value}");
    let mut out = String::with_capacity(text.len() + line.len() + 1);
    let mut replaced = false;
    for old in text.lines() {
        if !replaced && assigns(old, key) {
            out.push_str(&line);
            replaced = true;
        } else {
            out.push_str(old);
        }
        out.push('\n');
    }
    if !replaced {
        out.push_str(&line);
        out.push('\n');
    }
    out
}

fn assigns(line: &str, key: &str) -> bool {
    line.trim_start()
        .strip_prefix(key)
        .is_some_and(|rest| rest.trim_start().starts_with('='))
}

#[cfg(test)]
mod tests;
