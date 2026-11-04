//! The files `kubaos-depth` writes for a picture, in a folder of their own, which the shipped
//! wallpapers carry too and the lock reads: [`CUTOUT_FILE`] and [`SIDECAR_FILE`]. Their contract is
//! `docs/CONTRACTS.md`, "Wallpaper depth".
//!
//! # Owns
//!
//! - The files' names, the sidecar's format ([`Sidecar`]), and writing and reading it.
//!
//! # Not here
//!
//! - Making the cut-out: `crate::run`.

use std::fs;
use std::io;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use crate::choose::Method;
use crate::clock::Place;

/// The cut-out: a PNG with alpha, the picture's size, present only when one was kept.
pub const CUTOUT_FILE: &str = "cutout.png";

/// The sidecar: the clock's place and tint, and what made the cut-out.
pub const SIDECAR_FILE: &str = "depth.toml";

/// The sidecar's format: a reader refuses a file with another, and a change that would mislead an
/// older reader raises it.
pub const FORMAT: u32 = 1;

/// What the sidecar says about a picture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sidecar {
    /// [`FORMAT`].
    pub format: u32,
    /// The version of `kubaos-depth` that wrote it.
    pub tool: String,
    /// Which cut-out was kept, or `none`, when [`CUTOUT_FILE`] isn't there.
    pub method: Kept,
    /// The best candidate's score, kept or not: the cut-out's confidence, or how far the best one
    /// fell short.
    pub score: f32,
    pub clock: Clock,
}

/// Which cut-out was kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kept {
    Sky,
    Object,
    None,
}

impl From<Option<Method>> for Kept {
    fn from(method: Option<Method>) -> Self {
        match method {
            Some(Method::Sky) => Self::Sky,
            Some(Method::Object) => Self::Object,
            None => Self::None,
        }
    }
}

/// The rest clock on the picture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clock {
    /// The middle of the digits' baseline, a share of the picture's width from its left.
    pub x: f32,
    /// The same, a share of the picture's height from its top.
    pub y: f32,
    /// The digits' and the kicker's colour at rest, `#rrggbb`.
    pub tint: String,
}

impl Sidecar {
    #[must_use]
    pub fn new(method: Option<Method>, score: f32, place: Place, tint: String) -> Self {
        Self {
            format: FORMAT,
            tool: env!("CARGO_PKG_VERSION").to_owned(),
            method: method.into(),
            score: round(score),
            clock: Clock {
                x: round(place.x),
                y: round(place.y),
                tint,
            },
        }
    }

    /// Writes the sidecar into `dir`, through a temporary file renamed into place, so a reader
    /// never finds half of one.
    ///
    /// # Errors
    ///
    /// The I/O error writing or renaming it.
    pub fn write(&self, dir: &Path) -> io::Result<()> {
        let text = toml::to_string(self).map_err(io::Error::other)?;
        let partial = dir.join(format!(".{SIDECAR_FILE}.partial"));
        fs::write(&partial, text)?;
        fs::rename(partial, dir.join(SIDECAR_FILE))
    }

    /// The sidecar in `dir`.
    ///
    /// # Errors
    ///
    /// [`ReadError`] when it can't be read, isn't a sidecar or has another [`FORMAT`].
    pub fn read(dir: &Path) -> Result<Self, ReadError> {
        let text = fs::read_to_string(dir.join(SIDECAR_FILE)).map_err(ReadError::Io)?;
        let sidecar: Self = toml::from_str(&text).map_err(ReadError::Parse)?;
        if sidecar.format != FORMAT {
            return Err(ReadError::Format(sidecar.format));
        }
        Ok(sidecar)
    }
}

/// Reading a sidecar failed.
#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("reading {SIDECAR_FILE}")]
    Io(#[source] io::Error),
    #[error("parsing {SIDECAR_FILE}")]
    Parse(#[source] toml::de::Error),
    #[error("{SIDECAR_FILE} is format {0}, not {FORMAT}")]
    Format(u32),
}

/// `v` to four places: a ten-thousandth of a 4K picture is under half a pixel.
fn round(v: f32) -> f32 {
    (v * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests;
