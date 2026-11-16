//! The user's picks, `~/.config/kubaos/apps.toml`: the apps of the catalog they want on the
//! machine. It is the source of truth `kubaos update` reconciles to; `kubaos app add|remove`
//! rewrite it whole, sorted, under the update lock.
//!
//! # Owns
//!
//! - [`Picks`] and the file's format.
//!
//! # Not here
//!
//! - What the picks mean for the machine: `plan.rs`. Writing them as a run goes: the updater's
//!   `apps.rs`.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::state::write_atomic;

/// The picks, inside the user layer (`~/.config/kubaos`).
pub const PICKS: &str = "apps.toml";

/// The comment the file starts with.
const HEADER: &str = "\
# The apps of KubaOS's catalog you picked (`kubaos app list`). kubaos update installs a picked app
# the machine lacks. `kubaos app add|remove` rewrite this file, comments too.
";

/// The picks file could not be read or written, or isn't one.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PicksError {
    #[error("reading {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{} is not a list of apps: apps = [\"id\", ...]", path.display())]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("writing {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct File {
    #[serde(default)]
    apps: Vec<String>,
}

/// The ids the user picked. An id the catalog doesn't have (an app a later release dropped) is
/// kept, and passed over.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Picks(BTreeSet<String>);

impl Picks {
    /// Reads the picks at `path`; none when the file is missing.
    ///
    /// # Errors
    ///
    /// [`PicksError`] when it can't be read or isn't a list of apps.
    pub fn read(path: &Path) -> Result<Self, PicksError> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => {
                return Err(PicksError::Read {
                    path: path.to_path_buf(),
                    source,
                });
            }
        };
        let file: File = toml::from_str(&text).map_err(|source| PicksError::Parse {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(Self(file.apps.into_iter().collect()))
    }

    /// Writes the picks to `path`, whole, through a temporary file, making its folder.
    ///
    /// # Errors
    ///
    /// [`PicksError::Write`] when the folder or the file can't be written.
    pub fn write(&self, path: &Path) -> Result<(), PicksError> {
        let write_err = |source| PicksError::Write {
            path: path.to_path_buf(),
            source,
        };
        let body = toml::to_string(&File {
            apps: self.0.iter().cloned().collect(),
        })
        .map_err(|e| write_err(io::Error::other(e)))?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(write_err)?;
        }
        write_atomic(path, &format!("{HEADER}{body}")).map_err(write_err)
    }

    /// Whether `id` is picked.
    pub fn has(&self, id: &str) -> bool {
        self.0.contains(id)
    }

    /// Picks `id`; whether it wasn't yet.
    pub fn add(&mut self, id: &str) -> bool {
        self.0.insert(id.to_owned())
    }

    /// Drops `id`; whether it was picked.
    pub fn remove(&mut self, id: &str) -> bool {
        self.0.remove(id)
    }

    /// The ids, sorted.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(String::as_str)
    }
}
