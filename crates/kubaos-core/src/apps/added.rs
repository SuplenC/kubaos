//! Which nixpkgs names each app of the catalog put into the User flake's `packages.toml`,
//! `~/.local/state/kubaos/apps-added.toml`, and only those: a name the user had already added with
//! `kubaos package add` isn't the app's, so removing the app leaves it there.
//!
//! It describes this machine's `packages.toml`, so it is state, not one of the user's choices: a
//! `~/.config` copied to another machine doesn't carry it, where a removal would otherwise take
//! names the user added there.
//!
//! # Owns
//!
//! - [`Added`] and the file's format.
//!
//! # Not here
//!
//! - Who gets a name when its app goes: `plan.rs`. Writing it as a run goes: the updater's
//!   `apps.rs`.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use crate::state::write_atomic;

/// The record, inside KubaOS's state folder (`~/.local/state/kubaos`).
pub const ADDED: &str = "apps-added.toml";

/// The comment the file starts with.
const HEADER: &str = "\
# The nixpkgs packages each app of KubaOS's catalog added to the User flake's packages.toml,
# which removing the app takes out again. `kubaos app add|remove` rewrite this file.
";

/// The record could not be read or written, or isn't one.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AddedError {
    #[error("reading {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{} is not a record of apps' packages: <id> = [\"name\", ...]", path.display())]
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

/// The nixpkgs names each app added, by app id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Added(BTreeMap<String, BTreeSet<String>>);

impl Added {
    /// Reads the record at `path`; empty when the file is missing.
    ///
    /// # Errors
    ///
    /// [`AddedError`] when it can't be read or isn't a record.
    pub fn read(path: &Path) -> Result<Self, AddedError> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => {
                return Err(AddedError::Read {
                    path: path.to_path_buf(),
                    source,
                });
            }
        };
        toml::from_str(&text).map(Self).map_err(|source| AddedError::Parse {
            path: path.to_path_buf(),
            source,
        })
    }

    /// Writes the record to `path`, whole, through a temporary file, making its folder.
    ///
    /// # Errors
    ///
    /// [`AddedError::Write`] when the folder or the file can't be written.
    pub fn write(&self, path: &Path) -> Result<(), AddedError> {
        let write_err = |source| AddedError::Write {
            path: path.to_path_buf(),
            source,
        };
        let body = toml::to_string(&self.0).map_err(|e| write_err(io::Error::other(e)))?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(write_err)?;
        }
        write_atomic(path, &format!("{HEADER}{body}")).map_err(write_err)
    }

    /// The nixpkgs names app `id` added, sorted.
    pub fn of(&self, id: &str) -> Vec<&str> {
        self.0.get(id).into_iter().flatten().map(String::as_str).collect()
    }

    /// Records that app `id` added the nixpkgs `names`.
    pub fn record(&mut self, id: &str, names: impl IntoIterator<Item = impl Into<String>>) {
        let mut names = names.into_iter().map(Into::into).peekable();
        if names.peek().is_some() {
            self.0.entry(id.to_owned()).or_default().extend(names);
        }
    }

    /// Forgets what app `id` added, and returns it.
    pub fn take(&mut self, id: &str) -> BTreeSet<String> {
        self.0.remove(id).unwrap_or_default()
    }
}
