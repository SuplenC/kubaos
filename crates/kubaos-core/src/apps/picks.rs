//! The user's picks, `~/.config/kubaos/apps.toml`: the apps of the catalog they want on the
//! machine. It is the source of truth `kubaos update` reconciles to; `kubaos app add|remove`
//! rewrite it whole, sorted, under the update lock.
//!
//! It also keeps, under `[nix_added]`, the nixpkgs names each app put into the User flake's
//! `packages.toml`, and only those: a name the user had already added with `kubaos package add`
//! isn't the app's, so removing the app leaves it there.
//!
//! # Owns
//!
//! - [`Picks`] and the file's format.
//!
//! # Not here
//!
//! - What the picks mean for the machine: `plan.rs`. Writing them as a run goes: the updater's
//!   `apps.rs`.

use std::collections::BTreeMap;
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
# the machine lacks. `kubaos app add|remove` rewrite this file, comments too. [nix_added] lists
# the nixpkgs packages each app added to packages.toml, which removing the app takes out again.
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
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    nix_added: BTreeMap<String, BTreeSet<String>>,
}

/// The ids the user picked, and the nixpkgs names each app added. An id the catalog doesn't have
/// (an app a later release dropped) is kept, and passed over.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Picks {
    apps: BTreeSet<String>,
    nix_added: BTreeMap<String, BTreeSet<String>>,
}

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
        Ok(Self {
            apps: file.apps.into_iter().collect(),
            nix_added: file.nix_added,
        })
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
            apps: self.apps.iter().cloned().collect(),
            nix_added: self.nix_added.clone(),
        })
        .map_err(|e| write_err(io::Error::other(e)))?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(write_err)?;
        }
        write_atomic(path, &format!("{HEADER}{body}")).map_err(write_err)
    }

    /// Whether `id` is picked.
    pub fn has(&self, id: &str) -> bool {
        self.apps.contains(id)
    }

    /// Picks `id`; whether it wasn't yet.
    pub fn add(&mut self, id: &str) -> bool {
        self.apps.insert(id.to_owned())
    }

    /// Drops `id`; whether it was picked. What it added stays recorded until [`Self::take_added`].
    pub fn remove(&mut self, id: &str) -> bool {
        self.apps.remove(id)
    }

    /// The ids, sorted.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.apps.iter().map(String::as_str)
    }

    /// Picks exactly the apps `other` picks, keeping what this records the apps added: a run that
    /// fails puts the picks back, while the packages it wrote are still the apps'.
    pub fn restore_apps(&mut self, other: &Self) {
        self.apps.clone_from(&other.apps);
    }

    /// The nixpkgs names app `id` added, sorted.
    pub fn added(&self, id: &str) -> Vec<&str> {
        self.nix_added
            .get(id)
            .into_iter()
            .flatten()
            .map(String::as_str)
            .collect()
    }

    /// Records that app `id` added the nixpkgs `names`.
    pub fn record_added(&mut self, id: &str, names: impl IntoIterator<Item = impl Into<String>>) {
        let mut names = names.into_iter().map(Into::into).peekable();
        if names.peek().is_some() {
            self.nix_added.entry(id.to_owned()).or_default().extend(names);
        }
    }

    /// Forgets what app `id` added, and returns it.
    pub fn take_added(&mut self, id: &str) -> BTreeSet<String> {
        self.nix_added.remove(id).unwrap_or_default()
    }
}
