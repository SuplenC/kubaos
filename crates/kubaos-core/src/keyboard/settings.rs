//! The keyboard settings, in the user layer: `~/.config/kubaos/desktop/keyboard/keyboard.toml`.
//!
//! ```toml
//! mac_accents = false
//! layouts = ["us", "it(intl)"]
//! ```
//!
//! The Command menu and the control center change them through `kubaos keyboard`, which edits the
//! one line it changes, so the user's comments and the rest of the file stay as they are.
//!
//! # Owns
//!
//! - Whether the Mac-style accents on Left Alt are on, and the user's keyboard layouts: reading the
//!   file and changing it.
//!
//! # Not here
//!
//! - Applying them (keyd, Hyprland): `kubaos keyboard` in the CLI. The seed with its comments:
//!   `user-layer/desktop/keyboard/keyboard.toml`. Hyprland's own reads of the file:
//!   `config/hypr/accents.lua` and `config/hypr/input.lua`.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use kubaos_user_layer::App;
use serde::Deserialize;

use super::Layout;

/// The accents are on until the user turns them off: Left Alt with `e`, `` ` ``, `u`, `i` and `n`
/// are the only Alt chords they take, and macOS hands have them in their fingers.
pub const DEFAULT_MAC_ACCENTS: bool = true;

/// The settings file in the keyboard's folder of the user layer.
const FILE: &str = "keyboard.toml";

const MAC_ACCENTS: &str = "mac_accents";

const LAYOUTS: &str = "layouts";

/// The settings file for the config home `config_home`, usually `~/.config`.
pub fn path(config_home: &Path) -> PathBuf {
    kubaos_user_layer::dir(config_home, App::Keyboard).join(FILE)
}

/// The keyboard settings.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    /// Left Alt with `e`, `` ` ``, `u`, `i` or `n` starts an accent for the next letter.
    pub mac_accents: bool,
    /// The layouts the user types with, spelled `it` or `us(intl)`, the first the default. Empty
    /// until `kubaos keyboard` first changes them; until then they are the system's.
    pub layouts: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            mac_accents: DEFAULT_MAC_ACCENTS,
            layouts: Vec::new(),
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
    set(path, MAC_ACCENTS, &(accents == MacAccents::On).to_string())
}

/// Writes `layouts` as the user's in the settings file at `path`, creating it when missing.
///
/// # Errors
///
/// [`SettingsError::Read`] or [`SettingsError::Write`] when the file can't be read or replaced.
pub fn set_layouts(path: &Path, layouts: &[Layout]) -> Result<(), SettingsError> {
    // The spellings are XKB names, which `problems` keeps to letters, digits, `_`, `-` and the
    // variant's parentheses, so a plain quoted string holds each.
    let list: Vec<String> = layouts.iter().map(|l| format!("\"{l}\"")).collect();
    set(path, LAYOUTS, &format!("[{}]", list.join(", ")))
}

/// Sets `key` to `value` in the settings file at `path`, through a temporary file and a rename.
fn set(path: &Path, key: &str, value: &str) -> Result<(), SettingsError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(SettingsError::Read(e)),
    };
    let new = with_value(&text, key, value);
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
