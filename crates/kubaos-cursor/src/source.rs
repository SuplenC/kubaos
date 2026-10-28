//! Bibata's sources: the cursors its XCursor build config names, with their hotspots, aliases and
//! frame delay, and the Modern SVGs they're drawn from. `home/theme.nix` pins the source tree and
//! links it to `~/.local/share/kubaos/cursor/bibata`.
//!
//! Reading the names and hotspots from Bibata's own config, `configs/normal/x.build.toml`, keeps
//! them right for the pinned version without a copy here.
//! <https://github.com/ful1e5/Bibata_Cursor/tree/v2.0.7/configs/normal>
//!
//! # Owns
//!
//! - Where in the tree the config and the SVGs are, and turning them into [`Shape`]s.
//!
//! # Not here
//!
//! - The placeholder colours in the SVGs: `recolour`.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;

/// The side of Bibata's square SVG canvas, in the units its hotspots are given in.
pub const CANVAS: u32 = 256;

const CONFIG: &str = "configs/normal/x.build.toml";
const SVG_DIR: &str = "svg/modern";

/// One cursor: its XCursor name, the names it also answers to, and its frames, one for a still
/// cursor and several for an animated one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shape {
    pub name: String,
    pub aliases: Vec<String>,
    /// The hotspot on the [`CANVAS`], x then y.
    pub hotspot: (u32, u32),
    /// Each frame's SVG, in order.
    pub frames: Vec<String>,
    /// How long each frame of an animated cursor shows, in milliseconds.
    pub delay_ms: u32,
}

/// Why Bibata's sources couldn't be read.
#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("reading Bibata's config {}", .0.display())]
    ReadConfig(PathBuf, #[source] io::Error),
    #[error("parsing Bibata's config {}", .0.display())]
    ParseConfig(PathBuf, #[source] toml::de::Error),
    #[error("reading Bibata's SVG {}", .0.display())]
    ReadSvg(PathBuf, #[source] io::Error),
    #[error("Bibata's animated cursor {} has no frames", .0.display())]
    NoFrames(PathBuf),
}

#[derive(Deserialize)]
struct Config {
    cursors: BTreeMap<String, toml::Value>,
}

#[derive(Deserialize)]
struct Fallback {
    x_hotspot: u32,
    y_hotspot: u32,
    x11_delay: u32,
}

#[derive(Deserialize)]
struct Entry {
    png: String,
    x11_name: String,
    x_hotspot: Option<u32>,
    y_hotspot: Option<u32>,
    x11_delay: Option<u32>,
    #[serde(default)]
    x11_symlinks: Vec<String>,
}

/// Every cursor of the Bibata source tree at `root`, sorted by name.
///
/// # Errors
///
/// A [`SourceError`] naming the config or the SVG that couldn't be read.
pub fn load(root: &Path) -> Result<Vec<Shape>, SourceError> {
    let config_path = root.join(CONFIG);
    let text = fs::read_to_string(&config_path).map_err(|e| SourceError::ReadConfig(config_path.clone(), e))?;
    let parse_error = |e| SourceError::ParseConfig(config_path.clone(), e);
    let mut config: Config = toml::from_str(&text).map_err(parse_error)?;
    let fallback: Fallback = config
        .cursors
        .remove("fallback_settings")
        .map_or_else(
            || toml::from_str("x_hotspot = 128\ny_hotspot = 128\nx11_delay = 40"),
            toml::Value::try_into,
        )
        .map_err(parse_error)?;

    let svgs = root.join(SVG_DIR);
    let mut shapes = Vec::with_capacity(config.cursors.len());
    for value in config.cursors.into_values() {
        let entry: Entry = value.try_into().map_err(parse_error)?;
        shapes.push(Shape {
            frames: frames(&svgs, &entry.png)?,
            hotspot: (
                entry.x_hotspot.unwrap_or(fallback.x_hotspot),
                entry.y_hotspot.unwrap_or(fallback.y_hotspot),
            ),
            delay_ms: entry.x11_delay.unwrap_or(fallback.x11_delay),
            name: entry.x11_name,
            aliases: entry.x11_symlinks,
        });
    }
    shapes.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(shapes)
}

/// The SVGs a config entry's `png` stands for: `left_ptr.png` is `left_ptr.svg`, and an animated
/// `wait-*.png` is every SVG in the `wait/` folder, in name order.
fn frames(svgs: &Path, png: &str) -> Result<Vec<String>, SourceError> {
    let stem = png.strip_suffix(".png").unwrap_or(png);
    let read = |path: PathBuf| fs::read_to_string(&path).map_err(|e| SourceError::ReadSvg(path, e));
    let Some(animated) = stem.strip_suffix("-*") else {
        return Ok(vec![read(svgs.join(format!("{stem}.svg")))?]);
    };
    let dir = svgs.join(animated);
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|e| SourceError::ReadSvg(dir.clone(), e))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "svg"))
        .collect();
    if paths.is_empty() {
        return Err(SourceError::NoFrames(dir));
    }
    paths.sort();
    paths.into_iter().map(read).collect()
}

#[cfg(test)]
mod tests;
