//! The state directory, `kubaos.d/` in the user flake: what the KubaOS CLI, the Command menu and
//! Kōbō install, as files the tool owns.
//!
//! `packages.toml` lists packages from the user's nixpkgs, and each `suites/<name>.toml` holds one
//! Suite's options and version, plus any packages it lists. KubaOS's home module (`home/state.nix`)
//! reads the same files and installs the union of every `packages` list. Each home-manager
//! generation carries a copy of the directory, so a rollback can bring it back.
//!
//! # Owns
//!
//! - Where the user flake, the state directory and the switch jobs live ([`Paths`]).
//! - Reading every file of the state directory into what it declares ([`StateDir::declared`]).
//! - Restoring the directory from a home-manager generation.
//! - Wiring `home.nix` to the directory, in [`migrate`].
//!
//! # Not here
//!
//! - Editing `packages.toml`: `packages.rs`.
//! - Running the switch that applies the directory: `switch.rs`.
//! - The Nix side: `home/state.nix`.

pub mod migrate;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process;

use serde::Deserialize;

/// The state directory's name, inside the user flake.
pub const DIR_NAME: &str = "kubaos.d";
/// The file of packages the user added, inside the state directory.
pub const PACKAGES_FILE: &str = "packages.toml";
/// The folder of Suite files, inside the state directory.
pub const SUITES_DIR: &str = "suites";
/// Where a home-manager generation keeps its copy of the state directory, relative to the
/// generation. `home/state.nix` links it as `~/.local/share/kubaos/kubaos.d`.
pub const GENERATION_COPY: &str = "home-files/.local/share/kubaos/kubaos.d";

/// Where the user flake, its state directory and the switch jobs live for one user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paths {
    flake: PathBuf,
    state: PathBuf,
}

/// `HOME` is not set, so no path under it can be found.
#[derive(Debug, thiserror::Error)]
#[error("HOME is not set")]
pub struct NoHomeError(());

impl Paths {
    /// The paths for the current user, from `HOME`, `XDG_CONFIG_HOME` and `XDG_STATE_HOME`.
    ///
    /// # Errors
    ///
    /// [`NoHomeError`] when `HOME` is not set.
    pub fn from_env() -> Result<Self, NoHomeError> {
        let var = |name| std::env::var_os(name).filter(|v| !v.is_empty());
        Self::from_vars(var("HOME"), var("XDG_CONFIG_HOME"), var("XDG_STATE_HOME"))
    }

    /// The paths for the given `HOME`, `XDG_CONFIG_HOME` and `XDG_STATE_HOME`, each XDG one
    /// falling back to its default under the home.
    ///
    /// # Errors
    ///
    /// [`NoHomeError`] when `home` is `None`.
    pub fn from_vars(
        home: Option<OsString>,
        config: Option<OsString>,
        state: Option<OsString>,
    ) -> Result<Self, NoHomeError> {
        let home = PathBuf::from(home.ok_or(NoHomeError(()))?);
        let config = config.map_or_else(|| home.join(".config"), PathBuf::from);
        let state = state.map_or_else(|| home.join(".local/state"), PathBuf::from);
        Ok(Self {
            flake: config.join("kubaos/home"),
            state: state.join("kubaos"),
        })
    }

    /// The user flake, `~/.config/kubaos/home`.
    pub fn flake(&self) -> &Path {
        &self.flake
    }

    /// The user's `home.nix`, in the user flake.
    pub fn home_nix(&self) -> PathBuf {
        self.flake.join("home.nix")
    }

    /// The state directory, `kubaos.d/` in the user flake.
    pub fn state_dir(&self) -> StateDir {
        StateDir::new(self.flake.join(DIR_NAME))
    }

    /// The switch jobs' files: their status, logs and lock.
    pub fn jobs(&self) -> PathBuf {
        self.state.join("jobs")
    }

    /// The lock every edit of the state directory takes. It lives outside the directory, so that
    /// neither the flake nor a generation's copy picks it up.
    pub fn edit_lock(&self) -> PathBuf {
        self.state.join("kubaos.d.lock")
    }
}

/// The state directory at one path. Nothing is read until asked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateDir {
    root: PathBuf,
}

/// Where a declared package comes from.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    /// `packages.toml`: the user added it.
    Packages,
    /// `suites/<name>.toml`: a Suite lists it.
    Suite(String),
}

/// What the state directory declares: every package, with the files that list it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Declared {
    packages: BTreeMap<String, BTreeSet<Origin>>,
}

impl Declared {
    /// Whether no file lists a package.
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    /// Every declared package, sorted, each once: the union over the files.
    pub fn names(&self) -> Vec<&str> {
        self.packages.keys().map(String::as_str).collect()
    }

    /// The files that list `name`, `packages.toml` first, then the Suites by name.
    pub fn origins(&self, name: &str) -> Vec<Origin> {
        self.packages
            .get(name)
            .map(|o| o.iter().cloned().collect())
            .unwrap_or_default()
    }
}

/// The part of a state file this module reads. A Suite file has more keys, which the Suite
/// framework reads.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct StateFile {
    #[serde(default)]
    pub(crate) packages: Vec<String>,
}

/// Reading the state directory failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ReadError {
    /// A file or folder of it could not be read.
    #[error("reading {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// A file is not TOML, or its `packages` is not a list of strings.
    #[error("parsing {}", path.display())]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
}

impl ReadError {
    fn read(path: &Path) -> impl FnOnce(io::Error) -> Self {
        let path = path.to_path_buf();
        move |source| Self::Read { path, source }
    }

    fn parse(path: &Path) -> impl FnOnce(toml::de::Error) -> Self {
        let path = path.to_path_buf();
        move |source| Self::Parse { path, source }
    }
}

/// Restoring the state directory from a generation failed. The directory is as it was, unless
/// the failure is in the final swap, which says so.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RestoreError {
    /// The path is not a home-manager generation: it has no `home-files`.
    #[error("{} is not a home-manager generation", path.display())]
    NotAGeneration { path: PathBuf },
    /// Copying the generation's state directory failed.
    #[error("copying {} from the generation", path.display())]
    Copy {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// Swapping the copy in place of the state directory failed.
    #[error("replacing {} with the generation's copy", path.display())]
    Swap {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

impl StateDir {
    /// The state directory at `root`.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The directory itself.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// `packages.toml`.
    pub fn packages_file(&self) -> PathBuf {
        self.root.join(PACKAGES_FILE)
    }

    /// Reads `packages.toml` and every `suites/*.toml`. A missing directory or file declares
    /// nothing.
    ///
    /// # Errors
    ///
    /// [`ReadError`] when a file exists but cannot be read or parsed.
    pub fn declared(&self) -> Result<Declared, ReadError> {
        let mut declared = Declared::default();
        let mut add = |file: StateFile, origin: &Origin| {
            for name in file.packages {
                declared.packages.entry(name).or_default().insert(origin.clone());
            }
        };
        add(read_state_file(&self.packages_file())?, &Origin::Packages);
        for (name, path) in self.suite_files()? {
            add(read_state_file(&path)?, &Origin::Suite(name));
        }
        Ok(declared)
    }

    /// The Suite files, `suites/<name>.toml`, by name.
    fn suite_files(&self) -> Result<Vec<(String, PathBuf)>, ReadError> {
        let dir = self.root.join(SUITES_DIR);
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(ReadError::read(&dir)(e)),
        };
        let mut files = Vec::new();
        for entry in entries {
            let path = entry.map_err(ReadError::read(&dir))?.path();
            let is_toml = path.extension().is_some_and(|e| e == "toml");
            if let Some(name) = path.file_stem().and_then(|s| s.to_str())
                && is_toml
                && path.is_file()
            {
                files.push((name.to_owned(), path));
            }
        }
        files.sort();
        Ok(files)
    }

    /// Replaces the state directory with the copy `generation` carries, the way a rollback to that
    /// generation must. A generation without a copy predates the state directory or had nothing in
    /// it, so the directory becomes empty.
    ///
    /// # Errors
    ///
    /// [`RestoreError`] when `generation` is not a home-manager generation, or the copy or the
    /// swap fails.
    pub fn restore_from_generation(&self, generation: &Path) -> Result<(), RestoreError> {
        if !generation.join("home-files").is_dir() {
            return Err(RestoreError::NotAGeneration {
                path: generation.to_path_buf(),
            });
        }
        let fresh = self.sibling("restore");
        let copy_err = |source| RestoreError::Copy {
            path: fresh.clone(),
            source,
        };
        remove_dir_if_present(&fresh).map_err(copy_err)?;
        let saved = generation.join(GENERATION_COPY);
        if saved.is_dir() {
            copy_tree(&saved, &fresh).map_err(copy_err)?;
        } else {
            fs::create_dir_all(&fresh).map_err(copy_err)?;
        }

        let old = self.sibling("old");
        let swap_err = |source| RestoreError::Swap {
            path: self.root.clone(),
            source,
        };
        remove_dir_if_present(&old).map_err(swap_err)?;
        match fs::rename(&self.root, &old) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(swap_err(e)),
        }
        fs::rename(&fresh, &self.root).map_err(swap_err)?;
        remove_dir_if_present(&old).map_err(swap_err)
    }

    /// A working directory next to the state directory, hidden and outside it.
    fn sibling(&self, what: &str) -> PathBuf {
        let mut name = OsString::from(".");
        name.push(self.root.file_name().unwrap_or(DIR_NAME.as_ref()));
        name.push(format!(".{what}-{}", process::id()));
        self.root.with_file_name(name)
    }
}

/// Reads one state file; a missing one is empty.
pub(crate) fn read_state_file(path: &Path) -> Result<StateFile, ReadError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(StateFile::default()),
        Err(e) => return Err(ReadError::read(path)(e)),
    };
    toml::from_str(&text).map_err(ReadError::parse(path))
}

/// Writes `contents` to `path` through a temporary file in the same folder and a rename, so a
/// reader (the flake, another `kubaos`) sees the old file or the new one, never half of one.
pub(crate) fn write_atomic(path: &Path, contents: &str) -> io::Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(".tmp-{}", process::id()));
    let tmp = PathBuf::from(tmp);
    fs::write(&tmp, contents)?;
    fs::rename(&tmp, path).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })
}

fn remove_dir_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_dir_all(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

/// Copies the folders and files under `from` to `to`, following links, as the user's own writable
/// files: a generation's copy lives in the read-only Nix store.
fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if src.is_dir() {
            copy_tree(&src, &dst)?;
        } else {
            fs::write(&dst, fs::read(&src)?)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
