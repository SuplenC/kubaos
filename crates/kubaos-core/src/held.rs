//! Held packages: a package the state directory lists that couldn't be built at a switch keeps the
//! store path the home has, so one package that won't build never holds back the rest of the home.
//!
//! Before every switch, the switch job builds the listed packages as the user's nixpkgs has them
//! in the home configuration the switch builds (`kubaos.listedPackages`, `home/state.nix`),
//! keeping going past a failure. A package Nix says failed ([`failed_derivations`]) and that the
//! home has now, by the record its generation keeps (`~/.local/share/kubaos/installed.json`), is
//! held: `held.toml` in the state directory names it with that store path and the derivation that
//! failed, and `home/state.nix` installs the path in place of the attribute. A package that failed
//! and that the home doesn't have has no version to keep, and the job fails before the switch. A
//! package that doesn't evaluate is never held: the switch fails on it, saying why. Outputs
//! missing that Nix didn't say failed (the build was killed, the daemon went away) hold nothing
//! new ([`Unexplained`]).
//!
//! A held package whose derivation is still the one that failed isn't built again: its build would
//! fail the same way, and a download that times out costs minutes on every switch. It stays held
//! as it was ([`unchanged`]), until its outputs turn up in the store. The switch an update runs,
//! and `kubaos switch --retry-held`, build every held package again ([`Retry`]). A held package
//! that builds loses its hold, and the file goes with the last hold.
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
//! - What a person reads about a held package ([`message`]) and one it can't keep
//!   ([`unkept_message`]).
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
    /// The derivation whose build failed; a switch builds the package again only once its
    /// derivation is another.
    pub failed: String,
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

/// One listed package as the pre-build's evaluation found it.
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

/// The held packages of `before` the pre-build doesn't build again: those whose derivation is
/// still the one that failed when they were held.
pub fn unchanged(evaluated: &[Evaluated], before: &Holds) -> BTreeSet<String> {
    evaluated
        .iter()
        .filter(|one| {
            let drv = one.found.as_ref().map(|found| &found.drv);
            before.get(&one.name).is_some_and(|hold| drv == Some(&hold.failed))
        })
        .map(|one| one.name.clone())
        .collect()
}

/// The derivations `nix build`'s messages, `stderr`, say failed: each `.drv` an `error:` line
/// names, as in `error: Cannot build '<drv>'.` and `error: builder for '<drv>' failed`. A
/// package whose fetch failed is named too, since its own build then can't run.
pub fn failed_derivations(stderr: &str) -> BTreeSet<String> {
    let mut failed = BTreeSet::new();
    for line in stderr.lines().map(str::trim_start) {
        let Some(rest) = line.strip_prefix("error:") else {
            continue;
        };
        for quoted in rest.split('\'').skip(1).step_by(2) {
            if quoted.starts_with('/') && Path::new(quoted).extension().is_some_and(|e| e == "drv") {
                failed.insert(quoted.to_owned());
            }
        }
    }
    failed
}

/// What the pre-build's `nix build` left: the outputs still missing from the store, and the
/// derivations it said failed ([`failed_derivations`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Build {
    pub missing: BTreeSet<String>,
    pub failed: BTreeSet<String>,
}

/// How the pre-build ended for one listed package.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Every output the home installs is in the store.
    Built,
    /// Nix said its derivation failed, and its outputs are missing.
    Failed,
    /// It doesn't evaluate: the user's nixpkgs has no such package, or refuses it. It is never
    /// held, so the switch says why.
    Unevaluated,
    /// It is held, its derivation is the one that failed then, and its outputs are still
    /// missing, so it wasn't built.
    Unchanged,
}

/// What the pre-build found of one listed package.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prebuilt {
    pub name: String,
    pub outcome: Outcome,
    /// Its derivation, when it evaluated.
    pub found: Option<Found>,
}

/// Outputs of the listed packages are missing after the build, and Nix didn't say their builds
/// failed: the build stopped for another reason (a signal, the daemon, the disk), so nothing it
/// left says which packages can't be built.
#[derive(Debug, thiserror::Error)]
#[error("the build stopped before it built {}, without saying why", .0.join(", "))]
pub struct Unexplained(Vec<String>);

/// Each listed package of `evaluated` with how its pre-build ended: those in `skipped` weren't
/// built, and `build` is what the build of the others left.
///
/// # Errors
///
/// [`Unexplained`] when a package that was built has outputs missing that Nix didn't say failed.
pub fn prebuilt(
    evaluated: &[Evaluated],
    build: &Build,
    skipped: &BTreeSet<String>,
) -> Result<Vec<Prebuilt>, Unexplained> {
    let mut prebuilt = Vec::new();
    let mut unexplained = Vec::new();
    for one in evaluated {
        let outcome = match &one.found {
            None => Outcome::Unevaluated,
            Some(found) if found.outputs.values().all(|path| !build.missing.contains(path)) => Outcome::Built,
            Some(_) if skipped.contains(&one.name) => Outcome::Unchanged,
            Some(found) if build.failed.contains(&found.drv) => Outcome::Failed,
            Some(_) => {
                unexplained.push(one.name.clone());
                continue;
            }
        };
        prebuilt.push(Prebuilt {
            name: one.name.clone(),
            outcome,
            found: one.found.clone(),
        });
    }
    if unexplained.is_empty() {
        Ok(prebuilt)
    } else {
        Err(Unexplained(unexplained))
    }
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
    /// The store path the home has of the package `name`, whose derivation is `found`. A
    /// generation without the record has the path named as the derivation is; else the one path
    /// of the same package at another version, an output other than `out` left out
    /// (`postgresql-17.12-man`).
    fn path_of(&self, name: &str, found: &Found) -> Option<String> {
        let paths = match self {
            Self::Nothing => return None,
            Self::Recorded(paths) => return paths.get(name).cloned(),
            Self::Unrecorded(paths) => paths,
        };
        let drv_name = store_name(&found.drv).and_then(|name| name.strip_suffix(".drv"));
        if let Some(exact) = paths.iter().find(|path| store_name(path) == drv_name) {
            return Some(exact.clone());
        }
        let other_output = |name: &str| {
            found
                .outputs
                .keys()
                .any(|output| output != "out" && name.ends_with(&format!("-{output}")))
        };
        let mut matching = paths.iter().filter(|path| {
            store_name(path).is_some_and(|name| package_name(name) == found.pname && !other_output(name))
        });
        let first = matching.next()?;
        matching.next().is_none().then(|| first.clone())
    }
}

/// Which packages a switch holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Decision {
    /// The packages that failed and keep the store path the home has.
    pub held: Holds,
    /// The packages that failed and that the home doesn't have: nothing to keep, so the switch
    /// can't go on.
    pub unkept: Vec<String>,
}

/// Holds each package of `prebuilt` that failed at the store path `current` has of it, and keeps
/// the hold of `before` of each that wasn't built because it was unchanged. A package that built
/// or doesn't evaluate is held no more.
pub fn decide(prebuilt: &[Prebuilt], current: &CurrentHome, before: &Holds) -> Decision {
    let mut decision = Decision::default();
    for package in prebuilt {
        let (Outcome::Failed | Outcome::Unchanged, Some(found)) = (&package.outcome, &package.found) else {
            continue;
        };
        if package.outcome == Outcome::Unchanged
            && let Some(hold) = before.get(&package.name)
        {
            decision.held.insert(package.name.clone(), hold.clone());
            continue;
        }
        match current.path_of(&package.name, found) {
            Some(path) => {
                let hold = Hold {
                    path,
                    failed: found.drv.clone(),
                };
                decision.held.insert(package.name.clone(), hold);
            }
            None => decision.unkept.push(package.name.clone()),
        }
    }
    decision
}

/// What a person reads about the failed package `name` that the home has no version of.
pub fn unkept_message(name: &str) -> String {
    format!("{name} couldn't be built, and your home has no version of it to keep")
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

/// The held packages of the state directory `state`; none without `held.toml`.
///
/// # Errors
///
/// [`ReadError`] when the file exists but cannot be read, or is not a `[held]` table of holds.
pub fn read(state: &StateDir) -> Result<Holds, ReadError> {
    let path = state.held_file();
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Holds::new()),
        Err(e) => return Err(ReadError::read(&path)(e)),
    };
    let file: Table = toml::from_str(&text).map_err(ReadError::parse(&path))?;
    Ok(file.held)
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
        read(&self.state)
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
