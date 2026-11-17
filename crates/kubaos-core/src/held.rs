//! Held packages: a package the state directory lists that couldn't be built at a switch keeps the
//! store path the home has, so one package that won't build never holds back the rest of the home.
//!
//! Before every switch, the switch job builds the listed packages as the user's nixpkgs has them
//! (`kubaos.listedPackages`, `home/state.nix`), keeping going past a failure. A package that failed
//! and that the home has now, by the record its generation keeps
//! (`~/.local/share/kubaos/installed.json`), is held: `held.toml` in the state directory names it
//! with that store path and the derivation that failed, and `home/state.nix` installs the path in
//! place of the attribute. A package that failed and that the home doesn't have has no version to
//! keep, and fails the switch as it would without this.
//!
//! A held package whose derivation is still the one that failed isn't built again: its build would
//! fail the same way, and a download that times out costs minutes on every switch. It stays held
//! as it was ([`unchanged`]). The switch an update runs, and `kubaos switch --retry-held`, build
//! every held package again ([`Retry`]). A held package that builds loses its hold, and the file
//! goes with the last hold.
//!
//! ```toml
//! [held.discord]
//! path = "/nix/store/84k7g556fj4ib11413jna62l3hgfzyna-discord-1.0.159"
//! failed = "/nix/store/b2qzjq7gkvybdpdf7hh2hsi6x3cpag3s-discord-1.0.159.drv"
//! ```
//!
//! # Owns
//!
//! - `held.toml`: reading it, and writing it under the state directory's edit lock ([`HeldFile`]).
//! - Which held packages a switch doesn't build again ([`unchanged`]), and which packages it holds
//!   ([`decide`]), from what the pre-build found ([`prebuilt`]) and what the home has
//!   ([`CurrentHome`]).
//! - What a person reads about a held package ([`message`]).
//!
//! # Not here
//!
//! - Running the pre-build and the switch, and recording the holds on the job: `switch.rs`.
//! - The Nix side: `home/state.nix`.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::state::GitError;
use crate::state::ReadError;
use crate::state::StateDir;
use crate::state::lock_file;
use crate::state::write_atomic;

/// The comment `held.toml` starts with.
const HEADER: &str = "\
# Packages that couldn't be built, each kept at the store path your home had. A switch tries one
# again once its build changes, and every update tries them all; the kubaos command rewrites this
# file, and it goes once none is held.
";

/// One held package.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hold {
    /// The store path it keeps, which `home/state.nix` installs.
    pub path: String,
    /// The derivation whose build failed, when it evaluated; a switch builds the package again
    /// only once its derivation is another.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed: Option<String>,
}

/// The held packages, by name as the state directory lists them.
pub type Holds = BTreeMap<String, Hold>;

/// Whether a switch builds a held package whose derivation hasn't changed since it failed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Retry {
    /// It stays held without a build: its build would fail the same way.
    #[default]
    Changed,
    /// Every held package is built again, as in the switch an update runs.
    All,
}

/// What a person reads about the held package `name`.
pub fn message(name: &str) -> String {
    format!("{name} stays at its current version: it couldn't be built")
}

/// One listed package as the pre-build's evaluation found it in one home configuration.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Evaluated {
    pub name: String,
    /// Its derivation, or `None` when the user's nixpkgs has no such package or it doesn't
    /// evaluate.
    pub found: Option<Found>,
}

/// A listed package's derivation.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Found {
    /// The `.drv` store path.
    pub drv: String,
    /// The outputs the home installs, by output name, to their store paths.
    pub outputs: BTreeMap<String, String>,
    /// The package's name without its version, as `builtins.parseDrvName` reads it.
    pub pname: String,
}

impl Found {
    /// What `nix build` takes to build the outputs the home installs: `<drv>^out,man`.
    pub fn installable(&self) -> String {
        let outputs: Vec<&str> = self.outputs.keys().map(String::as_str).collect();
        format!("{}^{}", self.drv, outputs.join(","))
    }
}

/// The held packages of `before` the pre-build leaves out: those whose every derivation in
/// `evaluated` is the one that failed when they were held.
pub fn unchanged(evaluated: &[Evaluated], before: &Holds) -> BTreeSet<String> {
    before
        .iter()
        .filter(|(name, hold)| {
            let mut drvs = evaluated
                .iter()
                .filter(|one| one.name == **name)
                .map(|one| one.found.as_ref().map(|found| &found.drv))
                .peekable();
            drvs.peek().is_some() && hold.failed.is_some() && drvs.all(|drv| drv == hold.failed.as_ref())
        })
        .map(|(name, _)| name.clone())
        .collect()
}

/// How the pre-build ended for one listed package.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Every output the home installs is in the store.
    Built,
    /// It didn't build, or didn't evaluate: `drv` is a derivation of it that failed, when it
    /// evaluated.
    Failed { drv: Option<String> },
    /// It is held, and its derivation is the one that failed then, so it wasn't built.
    Unchanged,
}

/// What the pre-build found of one listed package, over every home configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prebuilt {
    pub name: String,
    pub outcome: Outcome,
    /// The package's name without its version, when it evaluated.
    pub pname: Option<String>,
}

/// Each listed package of `evaluated` with how its pre-build ended: those in `skipped` weren't
/// built, and a package built when every configuration's derivation of it evaluated and none of
/// their outputs is in `invalid`, the paths still missing after the build.
pub fn prebuilt(evaluated: &[Evaluated], invalid: &BTreeSet<String>, skipped: &BTreeSet<String>) -> Vec<Prebuilt> {
    let mut by_name: BTreeMap<&str, Prebuilt> = BTreeMap::new();
    for one in evaluated {
        let entry = by_name.entry(&one.name).or_insert_with(|| Prebuilt {
            name: one.name.clone(),
            outcome: if skipped.contains(&one.name) {
                Outcome::Unchanged
            } else {
                Outcome::Built
            },
            pname: None,
        });
        if entry.pname.is_none() {
            entry.pname = one.found.as_ref().map(|found| found.pname.clone());
        }
        if entry.outcome != Outcome::Built {
            continue;
        }
        match &one.found {
            None => entry.outcome = Outcome::Failed { drv: None },
            Some(found) if found.outputs.values().any(|path| invalid.contains(path)) => {
                entry.outcome = Outcome::Failed {
                    drv: Some(found.drv.clone()),
                };
            }
            Some(_) => {}
        }
    }
    by_name.into_values().collect()
}

/// The listed packages the home has now.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum CurrentHome {
    /// No home-manager generation yet.
    #[default]
    Nothing,
    /// The generation's record: each listed package to the store path it installed.
    Recorded(BTreeMap<String, String>),
    /// A generation built before the record: the store paths its packages' environment refers
    /// to, matched to a listed package by name.
    Unrecorded(Vec<String>),
}

impl CurrentHome {
    /// The store path the home has of the package `name`, whose name without its version is
    /// `pname`. A generation without the record has it when exactly one of its paths is that
    /// package at some version.
    fn path_of(&self, name: &str, pname: Option<&str>) -> Option<String> {
        match self {
            Self::Nothing => None,
            Self::Recorded(paths) => paths.get(name).cloned(),
            Self::Unrecorded(paths) => {
                let pname = pname?;
                let mut matching = paths
                    .iter()
                    .filter(|path| store_name(path).is_some_and(|name| package_name(name) == pname));
                let first = matching.next()?;
                matching.next().is_none().then(|| first.clone())
            }
        }
    }
}

/// Which packages a switch holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Decision {
    /// The packages that didn't build and keep the store path the home has.
    pub held: Holds,
    /// The packages that didn't build and that the home doesn't have: nothing to keep, so the
    /// switch fails on them.
    pub unkept: Vec<String>,
}

/// Holds each package of `prebuilt` that failed at the store path `current` has of it, and keeps
/// the hold of `before` of each that wasn't built because it was unchanged.
pub fn decide(prebuilt: &[Prebuilt], current: &CurrentHome, before: &Holds) -> Decision {
    let mut decision = Decision::default();
    for package in prebuilt {
        let failed = match &package.outcome {
            Outcome::Built => continue,
            Outcome::Unchanged => match before.get(&package.name) {
                Some(hold) => {
                    decision.held.insert(package.name.clone(), hold.clone());
                    continue;
                }
                None => None,
            },
            Outcome::Failed { drv } => drv.clone(),
        };
        match current.path_of(&package.name, package.pname.as_deref()) {
            Some(path) => {
                decision.held.insert(package.name.clone(), Hold { path, failed });
            }
            None => decision.unkept.push(package.name.clone()),
        }
    }
    decision
}

/// The name part of a store path, after its hash: `discord-1.0.159`.
fn store_name(path: &str) -> Option<&str> {
    let base = Path::new(path).file_name()?.to_str()?;
    let (hash, name) = base.split_once('-')?;
    (hash.len() == 32).then_some(name)
}

/// A store name without its version, as `builtins.parseDrvName` reads it: up to the first `-`
/// that a letter doesn't follow.
fn package_name(name: &str) -> &str {
    let bytes = name.as_bytes();
    for (i, byte) in bytes.iter().enumerate() {
        if *byte == b'-' && bytes.get(i + 1).is_some_and(|next| !next.is_ascii_alphabetic()) {
            return &name[..i];
        }
    }
    name
}

/// `held.toml` in one state directory, and the lock its writes take.
#[derive(Clone, Debug)]
pub struct HeldFile {
    state: StateDir,
    lock: PathBuf,
}

/// Writing `held.toml` failed. The file is as it was, unless the failure is in git.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WriteError {
    /// The edit lock could not be taken.
    #[error("locking {}", path.display())]
    Lock {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The file could not be written or removed.
    #[error("writing {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The file is written, but git's index could not be brought in step with it, so Nix may not
    /// see it.
    #[error("recording held.toml in git")]
    Git(#[source] GitError),
}

#[derive(Default, Serialize, Deserialize)]
struct Table {
    #[serde(default)]
    held: Holds,
}

impl HeldFile {
    /// `held.toml` in `state`, with `lock` as the edit lock (`Paths::edit_lock`).
    #[must_use]
    pub fn new(state: &StateDir, lock: impl Into<PathBuf>) -> Self {
        Self {
            state: state.clone(),
            lock: lock.into(),
        }
    }

    /// The state directory the file is in.
    pub fn state(&self) -> &StateDir {
        &self.state
    }

    /// The held packages; none without the file.
    ///
    /// # Errors
    ///
    /// [`ReadError`] when the file exists but cannot be read, or is not a `[held]` table of
    /// strings.
    pub fn read(&self) -> Result<Holds, ReadError> {
        let path = self.state.held_file();
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Holds::new()),
            Err(e) => return Err(ReadError::read(&path)(e)),
        };
        let file: Table = toml::from_str(&text).map_err(ReadError::parse(&path))?;
        Ok(file.held)
    }

    /// Makes the file hold `holds`, removing it when there are none, and records it in git.
    /// Returns whether it changed; when it didn't, nothing is written.
    ///
    /// # Errors
    ///
    /// [`WriteError`] when the lock, the write or git fails. A file that doesn't parse is
    /// replaced.
    pub fn write(&self, holds: &Holds) -> Result<bool, WriteError> {
        let _lock = lock_file(&self.lock).map_err(|source| WriteError::Lock {
            path: self.lock.clone(),
            source,
        })?;
        let path = self.state.held_file();
        let exists = path.exists();
        if self.read().ok().as_ref() == Some(holds) && exists != holds.is_empty() {
            return Ok(false);
        }
        let write_err = |source| WriteError::Write {
            path: path.clone(),
            source,
        };
        if holds.is_empty() {
            match fs::remove_file(&path) {
                Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(write_err(e)),
                _ => {}
            }
        } else {
            fs::create_dir_all(self.state.root()).map_err(write_err)?;
            let table = toml::to_string(&Table { held: holds.clone() }).map_err(|e| write_err(io::Error::other(e)))?;
            write_atomic(&path, &format!("{HEADER}{table}")).map_err(write_err)?;
        }
        self.state.sync_git().map_err(WriteError::Git)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests;
