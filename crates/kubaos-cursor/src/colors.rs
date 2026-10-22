//! The pointer's colours: the palette file matugen writes, `~/.local/state/theme/cursor.toml`, with
//! a table per recoloured style.
//!
//! ```toml
//! [dark]
//! fill = "#1e0f0e"
//! outline = "#fadbd9"
//! watch = "#1e0f0e"
//! spinner = ["#ffb3ae", "#ffdad6", "#ab3537", "#891d22"]
//!
//! [accent]
//! fill = "#ffb3ae"
//! outline = "#68000c"
//! watch = "#68000c"
//! spinner = ["#ffb3ae", "#ffdad6", "#ab3537", "#891d22"]
//! ```
//!
//! # Owns
//!
//! - Parsing that file and the `#rrggbb` colours in it.
//!
//! # Not here
//!
//! - Which roles fill each slot: the template, `config/matugen/templates/cursor.toml`.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::str::FromStr;

use serde::Deserialize;

use crate::settings::Style;

/// A colour, `#rrggbb` in the palette file and in the SVGs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgb(pub [u8; 3]);

/// A colour that isn't `#rrggbb`.
#[derive(Debug, thiserror::Error)]
#[error("not a #rrggbb colour: {0:?}")]
pub struct RgbError(String);

impl FromStr for Rgb {
    type Err = RgbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = || RgbError(s.to_owned());
        let hex = s.strip_prefix('#').ok_or_else(invalid)?;
        if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(invalid());
        }
        let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| invalid());
        Ok(Self([byte(0)?, byte(2)?, byte(4)?]))
    }
}

impl fmt::Display for Rgb {
    /// Uppercase, as Bibata writes its placeholders.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [r, g, b] = self.0;
        write!(f, "#{r:02X}{g:02X}{b:02X}")
    }
}

impl<'de> Deserialize<'de> for Rgb {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// The colours of one style: the body of every cursor, its outline, the disc behind the busy
/// cursors' spinner, and the spinner's four segments (also the four diagonal resize cursors).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Colors {
    pub fill: Rgb,
    pub outline: Rgb,
    pub watch: Rgb,
    pub spinner: [Rgb; 4],
}

/// The palette file: the colours of each recoloured style.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct Palette {
    pub dark: Colors,
    pub accent: Colors,
}

/// Why the palette file couldn't be read.
#[derive(Debug, thiserror::Error)]
pub enum PaletteError {
    #[error("reading the cursor palette")]
    Read(#[source] io::Error),
    #[error("parsing the cursor palette")]
    Parse(#[source] toml::de::Error),
}

impl Palette {
    /// Parses the palette file's text.
    ///
    /// # Errors
    ///
    /// [`PaletteError::Parse`] when a table, a key or a colour is missing or malformed.
    pub fn parse(text: &str) -> Result<Self, PaletteError> {
        toml::from_str(text).map_err(PaletteError::Parse)
    }

    /// Reads and parses the palette file at `path`.
    ///
    /// # Errors
    ///
    /// [`PaletteError::Read`] when the file can't be read, [`PaletteError::Parse`] as in
    /// [`Palette::parse`].
    pub fn load(path: &Path) -> Result<Self, PaletteError> {
        Self::parse(&fs::read_to_string(path).map_err(PaletteError::Read)?)
    }

    /// The colours of `style`, or `None` for `classic`, which isn't recoloured.
    pub fn colors(&self, style: Style) -> Option<Colors> {
        match style {
            Style::Dark => Some(self.dark),
            Style::Accent => Some(self.accent),
            Style::Classic => None,
        }
    }
}

#[cfg(test)]
mod tests;
