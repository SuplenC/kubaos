//! Packages the user adds from their own nixpkgs: `packages.toml` in the state directory, and
//! trying a package without installing it.
//!
//! Every edit is a read, change and atomic rewrite of `packages.toml` under one lock, so two
//! `kubaos package add` at once both land. The file is the tool's: it is rewritten whole, sorted
//! and without duplicates, and says so in its first line.
//!
//! # Owns
//!
//! - Adding, removing and listing the names in `packages.toml`.
//! - What a package name may be: a Nix attribute path in nixpkgs.
//! - The `nix shell` command that tries a package.
//!
//! # Not here
//!
//! - Reading the whole state directory, Suites included: `state.rs`.
//! - The switch that installs what the file lists: `switch.rs`.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

use crate::state::ReadError;
use crate::state::StateDir;
use crate::state::read_state_file;
use crate::state::write_atomic;

/// The comment `packages.toml` starts with.
const HEADER: &str = "\
# Packages from your nixpkgs that `kubaos package add` installed. The kubaos command rewrites this
# file: change it with `kubaos package add|remove`, and put your own packages in home.nix.
";

/// `packages.toml` in one state directory, and the lock its edits take.
#[derive(Clone, Debug)]
pub struct Packages {
    file: PathBuf,
    lock: PathBuf,
}

/// What an add or a remove did, each list sorted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Change {
    /// The names it added or removed.
    pub changed: Vec<String>,
    /// The names that were already there (add) or not there (remove).
    pub unchanged: Vec<String>,
}

impl Change {
    /// Whether the file changed, and so a switch has something to do.
    pub fn is_empty(&self) -> bool {
        self.changed.is_empty()
    }
}

/// A name that is not a Nix attribute path: segments of letters, digits, `_` and `-`, each
/// starting with a letter or `_`, joined by `.`.
#[derive(Debug, thiserror::Error)]
#[error("{0:?} is not a nixpkgs attribute name")]
pub struct InvalidName(String);

/// Editing `packages.toml` failed. The file is as it was.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EditError {
    /// One of the names is not a package name.
    #[error("invalid package name")]
    InvalidName(#[source] InvalidName),
    /// The edit lock could not be taken.
    #[error("locking {}", path.display())]
    Lock {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The current file could not be read.
    #[error("reading the current packages")]
    Read(#[source] ReadError),
    /// The new file could not be written.
    #[error("writing {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Serialize)]
struct PackagesFile<'a> {
    packages: Vec<&'a str>,
}

/// Checks that `name` is a Nix attribute path, so it is safe as a `nixpkgs#…` argument and as a
/// TOML string, and means one package.
///
/// # Errors
///
/// [`InvalidName`] when it is not.
pub fn validate_name(name: &str) -> Result<(), InvalidName> {
    let segment_ok = |s: &str| {
        let mut chars = s.chars();
        chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    };
    if name.split('.').all(segment_ok) {
        Ok(())
    } else {
        Err(InvalidName(name.to_owned()))
    }
}

/// The command that opens a shell with `name` from the user flake's nixpkgs, installing nothing.
///
/// # Errors
///
/// [`InvalidName`] when `name` is not a package name.
pub fn try_command(flake: &Path, name: &str) -> Result<Vec<OsString>, InvalidName> {
    validate_name(name)?;
    Ok(vec![
        "nix".into(),
        "shell".into(),
        "--inputs-from".into(),
        flake.as_os_str().to_owned(),
        format!("nixpkgs#{name}").into(),
    ])
}

impl Packages {
    /// `packages.toml` in `state`, with `lock` as the edit lock (`Paths::edit_lock`).
    #[must_use]
    pub fn new(state: &StateDir, lock: impl Into<PathBuf>) -> Self {
        Self {
            file: state.packages_file(),
            lock: lock.into(),
        }
    }

    /// The names in `packages.toml`, sorted. A missing file lists none.
    ///
    /// # Errors
    ///
    /// [`ReadError`] when the file exists but cannot be read or parsed.
    pub fn list(&self) -> Result<Vec<String>, ReadError> {
        let names: BTreeSet<String> = read_state_file(&self.file)?.packages.into_iter().collect();
        Ok(names.into_iter().collect())
    }

    /// Adds `names` to `packages.toml`, creating the state directory when it is missing.
    ///
    /// # Errors
    ///
    /// [`EditError`] when a name is invalid, or the lock, the read or the write fails.
    pub fn add(&self, names: &[impl AsRef<str>]) -> Result<Change, EditError> {
        self.edit(names, |current, name| current.insert(name.to_owned()))
    }

    /// Removes `names` from `packages.toml`. A missing file stays missing.
    ///
    /// # Errors
    ///
    /// [`EditError`] when a name is invalid, or the lock, the read or the write fails.
    pub fn remove(&self, names: &[impl AsRef<str>]) -> Result<Change, EditError> {
        self.edit(names, BTreeSet::remove)
    }

    /// Applies `apply` to each name under the lock, and rewrites the file when one changed it.
    fn edit(
        &self,
        names: &[impl AsRef<str>],
        mut apply: impl FnMut(&mut BTreeSet<String>, &str) -> bool,
    ) -> Result<Change, EditError> {
        for name in names {
            validate_name(name.as_ref()).map_err(EditError::InvalidName)?;
        }
        let _lock = self.lock()?;
        let mut current: BTreeSet<String> = self.list().map_err(EditError::Read)?.into_iter().collect();
        let mut changed = BTreeSet::new();
        let mut unchanged = BTreeSet::new();
        for name in names {
            let name = name.as_ref();
            if apply(&mut current, name) {
                changed.insert(name.to_owned());
            } else if !changed.contains(name) {
                unchanged.insert(name.to_owned());
            }
        }
        if !changed.is_empty() {
            self.write(&current)?;
        }
        Ok(Change {
            changed: changed.into_iter().collect(),
            unchanged: unchanged.into_iter().collect(),
        })
    }

    fn write(&self, names: &BTreeSet<String>) -> Result<(), EditError> {
        let write_err = |source| EditError::Write {
            path: self.file.clone(),
            source,
        };
        let body = toml::to_string(&PackagesFile {
            packages: names.iter().map(String::as_str).collect(),
        })
        .map_err(|e| write_err(io::Error::other(e)))?;
        if let Some(dir) = self.file.parent() {
            fs::create_dir_all(dir).map_err(write_err)?;
        }
        write_atomic(&self.file, &format!("{HEADER}{body}")).map_err(write_err)
    }

    /// Takes the edit lock, held until the returned file is dropped.
    fn lock(&self) -> Result<File, EditError> {
        let lock_err = |source| EditError::Lock {
            path: self.lock.clone(),
            source,
        };
        if let Some(dir) = self.lock.parent() {
            fs::create_dir_all(dir).map_err(lock_err)?;
        }
        let file = File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&self.lock)
            .map_err(lock_err)?;
        file.lock().map_err(lock_err)?;
        Ok(file)
    }
}

#[cfg(test)]
mod tests;
