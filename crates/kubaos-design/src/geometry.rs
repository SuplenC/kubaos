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
use serde::Deserialize;

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

#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(Box<Frame>),
    Many(Vec<Frame>),
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
        let read: OneOrMany = serde_json::from_str(&text).map_err(|source| Error::Frames {
            path: path.to_path_buf(),
            source,
        })?;
        match read {
            OneOrMany::One(frame) => frames.push(*frame),
            OneOrMany::Many(many) => frames.extend(many),
        }
    }
    Ok(kubaos_geometry::check(&frames, rules))
}
