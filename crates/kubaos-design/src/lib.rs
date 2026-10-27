//! KubaOS's design checks, as a library behind the `kubaos-design` command.
//!
//! ```text
//! QML, Rust UI code ─▶ lint::scan ─▶ Violation ─▶ baseline::compare ─▶ over or under the baseline
//! frames (JSON) ─────▶ geometry::check_files ─▶ kubaos_geometry::Finding
//! ```
//!
//! # Owns
//!
//! - The lint ([`lint`]) and its baseline ([`baseline`]), and reading geometry frames from files
//!   with the tokens' rules ([`geometry`]).
//!
//! # Not here
//!
//! - The geometry rules: `kubaos-geometry`. Rendering iced pages into frames: `kubaos-specimen
//!   check`, through `kubaos-ui`'s probes.

#![forbid(unsafe_code)]

pub mod baseline;
pub mod geometry;
pub mod lint;

use std::io;
use std::path::Path;
use std::path::PathBuf;

/// Why a check couldn't run.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("reading {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("writing {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("reading the manifest {}", path.display())]
    Manifest {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("reading {}", baseline::BASELINE_PATH)]
    Baseline(#[source] toml::de::Error),
    #[error("{} names {rule:?}, which isn't a lint rule", path)]
    UnknownRule { path: String, rule: String },
    #[error("the baseline never grows, and these files are over it: {}", .0.join(", "))]
    Grows(Vec<String>),
    #[error("reading the frames in {}", path.display())]
    Frames {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("loading the tokens")]
    Tokens(#[source] kubaos_tokens::Error),
}

impl Error {
    fn read(path: &Path, source: io::Error) -> Error {
        Error::Read {
            path: path.to_path_buf(),
            source,
        }
    }
}
