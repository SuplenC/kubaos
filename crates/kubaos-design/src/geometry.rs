//! Geometry frames from files, checked against the tokens' rules.
//!
//! A toolkit with no Rust probes (the shell's QML) writes its pages as JSON, one [`Frame`] or an
//! array of them per file, and `kubaos-design geometry FILE…` checks them with the rules the iced
//! pages are held to.
//!
//! # Owns
//!
//! - [`rules`], the geometry rules from `design/tokens.toml`, and [`check_files`].
//!
//! # Not here
//!
//! - The rules themselves: `kubaos-geometry`.

use std::fs;
use std::path::Path;

use kubaos_geometry::Finding;
use kubaos_geometry::Frame;
use kubaos_geometry::Rules;

use crate::Error;

/// The geometry rules of the tokens under `root`: their grid and spacing scale.
///
/// # Errors
///
/// [`Error::Tokens`] when the tokens can't be loaded or break their own rules.
pub fn rules(root: &Path) -> Result<Rules, Error> {
    let tokens = kubaos_tokens::load(root).map_err(Error::Tokens)?;
    Ok(Rules {
        grid: f32::from(tokens.rules.grid),
        spacing: tokens.space.values.iter().map(|&v| f32::from(v)).collect(),
    })
}

/// The frames in each of `paths`, checked together, so a component is held identical across them.
///
/// # Errors
///
/// When a file can't be read or isn't frames.
pub fn check_files(paths: &[&Path], rules: &Rules) -> Result<Vec<Finding>, Error> {
    let mut frames = Vec::new();
    for path in paths {
        let text = fs::read_to_string(path).map_err(|source| Error::read(path, source))?;
        let frames_error = |source| Error::Frames {
            path: path.to_path_buf(),
            source,
        };
        // A file holds one frame or an array of them. serde's untagged enums can't read a frame,
        // whose role is a flattened tagged enum, so the shape is told apart by hand.
        let value: serde_json::Value = serde_json::from_str(&text).map_err(frames_error)?;
        if value.is_array() {
            frames.extend(serde_json::from_value::<Vec<Frame>>(value).map_err(frames_error)?);
        } else {
            frames.push(serde_json::from_value::<Frame>(value).map_err(frames_error)?);
        }
    }
    Ok(kubaos_geometry::check(&frames, rules))
}
