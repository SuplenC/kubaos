//! The cursor settings, in the user layer: `~/.config/kubaos/desktop/cursor/cursor.toml`.
//!
//! ```toml
//! style = "dark"
//! size = 24
//! ```
//!
//! The Command menu changes them through `kubaos cursor style|size`, which edits the one line it
//! changes, so the user's comments and the rest of the file stay as they are.
//!
//! # Owns
//!
//! - The styles, the size's range, reading the file and changing a value in it.
//!
//! # Not here
//!
//! - Applying them: `kubaos-cli`'s `cursor.rs`. The seed with its comments:
//!   `user-layer/desktop/cursor/cursor.toml`.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::str::FromStr;

use serde::Deserialize;

/// How the pointer is coloured.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Style {
    /// A dark surface body with a light outline and the accent behind the spinner: readable on any
    /// wallpaper.
    #[default]
    Dark,
    /// The accent as the body, with a dark outline.
    Accent,
    /// Bibata-Modern-Classic as shipped, black with a white outline.
    Classic,
}

impl Style {
    pub const ALL: [Style; 3] = [Style::Dark, Style::Accent, Style::Classic];

    pub fn as_str(self) -> &'static str {
        match self {
            Style::Dark => "dark",
            Style::Accent => "accent",
            Style::Classic => "classic",
        }
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A style name that isn't one of [`Style::ALL`].
#[derive(Debug, thiserror::Error)]
#[error("no cursor style {0:?}; one of dark, accent, classic")]
pub struct StyleError(String);

impl FromStr for Style {
    type Err = StyleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Style::ALL
            .into_iter()
            .find(|style| style.as_str() == s)
            .ok_or_else(|| StyleError(s.to_owned()))
    }
}

/// The size in pixels at scale 1 that Bibata is drawn at: Hyprland's default, and the size the
/// shipped XCursor theme was set up with.
pub const DEFAULT_SIZE: u32 = 24;
/// The smallest size the settings take; Bibata's own builds start at 16.
pub const MIN_SIZE: u32 = 16;
/// The largest size the settings take, Bibata's own largest XCursor size.
pub const MAX_SIZE: u32 = 96;

/// The cursor settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub style: Style,
    pub size: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            style: Style::default(),
            size: DEFAULT_SIZE,
        }
    }
}

/// Why the settings file couldn't be read or changed.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("reading the cursor settings")]
    Read(#[source] io::Error),
    #[error("parsing the cursor settings")]
    Parse(#[source] toml::de::Error),
    #[error("cursor size {0} is outside {MIN_SIZE}..={MAX_SIZE}")]
    Size(u32),
    #[error("writing the cursor settings")]
    Write(#[source] io::Error),
}

impl Settings {
    /// Parses the settings file's text; a key left out takes its default.
    ///
    /// # Errors
    ///
    /// [`SettingsError::Parse`] for malformed TOML, an unknown key or style, and
    /// [`SettingsError::Size`] for a size outside [`MIN_SIZE`]..=[`MAX_SIZE`].
    pub fn parse(text: &str) -> Result<Self, SettingsError> {
        let settings: Self = toml::from_str(text).map_err(SettingsError::Parse)?;
        check_size(settings.size)?;
        Ok(settings)
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

/// Sets `style` in the settings file at `path`, creating it when missing.
///
/// # Errors
///
/// [`SettingsError::Read`] or [`SettingsError::Write`] when the file can't be read or replaced.
pub fn set_style(path: &Path, style: Style) -> Result<(), SettingsError> {
    set_value(path, "style", &format!("\"{style}\""))
}

/// Sets `size` in the settings file at `path`, creating it when missing.
///
/// # Errors
///
/// [`SettingsError::Size`] for a size outside [`MIN_SIZE`]..=[`MAX_SIZE`], else as in
/// [`set_style`].
pub fn set_size(path: &Path, size: u32) -> Result<(), SettingsError> {
    check_size(size)?;
    set_value(path, "size", &size.to_string())
}

fn check_size(size: u32) -> Result<(), SettingsError> {
    if (MIN_SIZE..=MAX_SIZE).contains(&size) {
        Ok(())
    } else {
        Err(SettingsError::Size(size))
    }
}

fn set_value(path: &Path, key: &str, value: &str) -> Result<(), SettingsError> {
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
pub fn with_value(text: &str, key: &str, value: &str) -> String {
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
