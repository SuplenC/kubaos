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
//! - Keeping git's index in step with the directory when the user flake is a git repository, in
//!   `git.rs`.
//! - The edit lock and the atomic write every edit of the directory uses.
//!
//! # Not here
//!
//! - Editing `packages.toml`: `packages.rs`.
//! - Writing and deleting Suite files: `suites.rs`.
//! - Running the switch that applies the directory: `switch.rs`.
//! - The Nix side: `home/state.nix`.

mod git;
pub mod migrate;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process;

use serde::Deserialize;

pub use self::git::GitError;
pub use self::git::GitSync;

/// The state directory's name, inside the user flake.
pub const DIR_NAME: &str = "kubaos.d";
/// The file of packages the user added, inside the state directory.
pub const PACKAGES_FILE: &str = "packages.toml";
/// The folder of Suite files, inside the state directory.
pub const SUITES_DIR: &str = "suites";
/// Where a home-manager generation keeps its copy of the state directory, relative to the
/// generation. `home/state.nix` links it as `~/.local/share/kubaos/kubaos.d`.
pub const GENERATION_COPY: &str = "home-files/.local/share/kubaos/kubaos.d";

/// The Projects root, relative to the home, unless [`PROJECTS_VAR`] names another.
pub const PROJECTS_ROOT: &str = "Projects";
/// The environment variable that moves the Projects root: an absolute path, or one relative to
/// the home.
pub const PROJECTS_VAR: &str = "KUBAOS_PROJECTS";

/// Where the user flake, its state directory, the user layer and the switch jobs live for one
/// user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paths {
    home: PathBuf,
    config: PathBuf,
    flake: PathBuf,
    state: PathBuf,
    projects: PathBuf,
}

/// `HOME` is not set, so no path under it can be found.
#[derive(Debug, thiserror::Error)]
#[error("HOME is not set")]
pub struct NoHomeError(());

impl Paths {
    /// The paths for the current user, from `HOME`, `XDG_CONFIG_HOME`, `XDG_STATE_HOME` and
    /// [`PROJECTS_VAR`].
    ///
    /// # Errors
    ///
    /// [`NoHomeError`] when `HOME` is not set.
    pub fn from_env() -> Result<Self, NoHomeError> {
        let var = |name| std::env::var_os(name).filter(|v| !v.is_empty());
        let paths = Self::from_vars(var("HOME"), var("XDG_CONFIG_HOME"), var("XDG_STATE_HOME"))?;
        Ok(match var(PROJECTS_VAR) {
            Some(projects) => paths.with_projects_root(projects),
            None => paths,
        })
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
            projects: home.join(PROJECTS_ROOT),
            config,
            home,
        })
    }

    /// The same paths with the Projects root at `root`: absolute, or relative to the home.
    #[must_use]
    pub fn with_projects_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.projects = self.home.join(root.into());
        self
    }

    /// The user's home.
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// The user layer, `~/.config/kubaos`.
    pub fn user_layer(&self) -> PathBuf {
        kubaos_user_layer::root(&self.config)
    }

    /// The Projects root, where scaffolded projects go: `~/Projects` unless [`PROJECTS_VAR`]
    /// moved it.
    pub fn projects_root(&self) -> PathBuf {
        self.projects.clone()
    }

    /// Where a migration backs up a user file before changing it: KubaOS's state folder, outside
    /// the user flake, so a backup never becomes part of the flake's source.
    pub fn backups(&self) -> PathBuf {
        self.state.join("backups")
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

    /// The user flake's `flake.lock`.
    pub fn flake_lock(&self) -> PathBuf {
        self.flake.join("flake.lock")
    }

    /// A marker per user migration that ran, `~/.local/state/kubaos/migrations/`.
    pub fn migration_markers(&self) -> PathBuf {
        self.state.join("migrations")
    }

    /// The user flake's `flake.lock` as each update found it, for a rollback.
    pub fn saved_locks(&self) -> PathBuf {
        self.state.join("locks")
    }

    /// What the daily check found to install (`update::notice`).
    pub fn update_available(&self) -> PathBuf {
        self.state.join("update-available")
    }

    /// The package lists the daily check keeps to count pending apt security updates
    /// (`update::security`); the next check rebuilds them when removed.
    pub fn apt_lists(&self) -> PathBuf {
        self.state.join("apt")
    }

    /// The updates the daily check notified, one per line.
    pub fn notified(&self) -> PathBuf {
        self.state.join("notified")
    }

    /// The full output of each `kubaos update`, `upgrade` and `rollback`, one log per run,
    /// `~/.local/state/kubaos/update/`.
    pub fn update_logs(&self) -> PathBuf {
        self.state.join("update")
    }

    /// The lock `kubaos update`, `upgrade` and `rollback` hold while they run.
    pub fn update_lock(&self) -> PathBuf {
        self.state.join("update.lock")
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
    pub(crate) fn read(path: &Path) -> impl FnOnce(io::Error) -> Self {
        let path = path.to_path_buf();
        move |source| Self::Read { path, source }
    }

    pub(crate) fn parse(path: &Path) -> impl FnOnce(toml::de::Error) -> Self {
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
    /// The directory is restored, but git's index could not be brought in step with it.
    #[error("recording the restored state directory in git")]
    Git(#[source] GitError),
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

    /// The user flake the directory is in: its parent.
    pub fn flake(&self) -> &Path {
        self.root.parent().unwrap_or(&self.root)
    }

    /// `packages.toml`.
    pub fn packages_file(&self) -> PathBuf {
        self.root.join(PACKAGES_FILE)
    }

    /// `suites/`.
    pub fn suites_dir(&self) -> PathBuf {
        self.root.join(SUITES_DIR)
    }

    /// `suites/<name>.toml`, the file of one Suite.
    pub fn suite_file(&self, name: &str) -> PathBuf {
        self.suites_dir().join(format!("{name}.toml"))
    }

    /// Makes git's index list the directory as it is on disk, when the user flake is in a git
    /// work tree: every edit of the directory ends with this, or Nix would not see a new file.
    ///
    /// # Errors
    ///
    /// [`GitError`] when a git command fails.
    pub fn sync_git(&self) -> Result<GitSync, GitError> {
        git::sync(self.flake(), &self.root)
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
    pub(crate) fn suite_files(&self) -> Result<Vec<(String, PathBuf)>, ReadError> {
        let dir = self.suites_dir();
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
        remove_dir_if_present(&old).map_err(swap_err)?;
        self.sync_git().map_err(RestoreError::Git)?;
        Ok(())
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

/// Takes the lock at `path`, made with its folder when missing, held until the returned file is
/// dropped.
pub(crate) fn lock_file(path: &Path) -> io::Result<File> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let file = File::options().create(true).truncate(false).write(true).open(path)?;
    file.lock()?;
    Ok(file)
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
