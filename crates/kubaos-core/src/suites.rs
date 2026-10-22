//! Suites: installing, configuring and removing a whole programming setup, and scaffolding
//! projects with it.
//!
//! A Suite has two halves. Its Nix module, `home/suites/<name>.nix`, declares what it installs:
//! packages, environment variables, PATH entries and its editor settings in the Defaults. Its
//! metadata, a [`Suite`] in [`catalog`], says what the tool needs to know: its options (a version,
//! extras), where its toolchain comes from, its caches, the user-layer files it seeds and the
//! projects it scaffolds.
//!
//! Installing a Suite writes `kubaos.d/suites/<name>.toml` with its options, and removing it
//! deletes the file; the next switch applies either. The tool never writes to a user-layer file
//! that exists: it only creates a missing one, from the Suite's commented example. Removal deletes
//! caches only when asked, and never touches a project or a user-layer file.
//!
//! # Owns
//!
//! - The Suite metadata ([`Suite`]) and the catalog of every Suite.
//! - Writing, reading and deleting Suite files, under the state directory's edit lock.
//! - Seeding user-layer files, sizing and purging caches, and finding the user-layer files that
//!   mention a Suite.
//! - Running a Suite's scaffolders in the Projects root.
//!
//! # Not here
//!
//! - Reading the state directory's packages: `state.rs`.
//! - The switch that applies a Suite file: `switch.rs`.
//! - The Nix side: `home/suites.nix`, `home/state.nix` and `home/suites/<name>.nix`.

pub mod catalog;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use serde::Deserialize;
use serde::Serialize;

use crate::state::GitError;
use crate::state::Paths;
use crate::state::ReadError;
use crate::state::StateDir;
use crate::state::lock_file;
use crate::state::write_atomic;

/// A user-layer file larger than this is not searched for mentions: it is data, not settings.
const MENTION_SCAN_LIMIT: u64 = 1024 * 1024;

/// What the tool knows about one Suite. Its Nix module, `home/suites/<name>.nix`, holds the rest.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct Suite {
    /// The name: the Suite file's name, the Nix module's and the one commands take. Lowercase
    /// letters, digits and `-`, starting with a letter.
    pub name: &'static str,
    /// The name people read, as in "New Go project".
    pub title: &'static str,
    /// One sentence on what it sets up.
    pub description: &'static str,
    /// The versions to choose from, the default first. Empty when there is no choice.
    pub versions: &'static [&'static str],
    /// The parts that can be turned on and off.
    pub extras: &'static [Extra],
    /// Where its toolchain comes from.
    pub toolchain: Toolchain,
    /// What its tools download or build outside the Nix store, which removal deletes only when
    /// asked.
    pub caches: &'static [Cache],
    /// The "New project" actions it gives once installed.
    pub scaffolders: &'static [Scaffolder],
    /// The user-layer files it creates, when missing, with a commented example.
    pub seeds: &'static [Seed],
    /// The words that mark a user-layer file as being about the Suite, matched as whole words
    /// and ignoring case.
    pub mentions: &'static [&'static str],
}

/// A part of a Suite that can be turned on and off.
#[derive(Clone, Copy, Debug)]
pub struct Extra {
    pub name: &'static str,
    pub description: &'static str,
    /// Whether a new install turns it on.
    pub default: bool,
}

/// Where a Suite's toolchain comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Toolchain {
    /// nixpkgs, through the Suite's Nix module.
    Nix,
    /// The language's own manager, which the Nix module installs and the Suite drives, where
    /// that is the norm (rustup for Rust).
    Upstream { manager: &'static str },
}

/// A folder a Suite's tools fill outside the Nix store.
#[derive(Clone, Copy, Debug)]
pub struct Cache {
    /// The folder, relative to the home.
    pub path: &'static str,
    pub description: &'static str,
}

/// A "New <Suite> project" action.
#[derive(Clone, Copy, Debug)]
pub struct Scaffolder {
    /// Its name among the Suite's scaffolders.
    pub id: &'static str,
    /// What a menu shows: "New Go project".
    pub label: &'static str,
    /// The command that creates the project, run in the Projects root with the Suite installed.
    /// `{path}` becomes the new project's folder, which must not exist yet, and `{name}` its name.
    pub argv: &'static [&'static str],
}

/// A user-layer file a Suite creates when it is missing.
#[derive(Clone, Copy, Debug)]
pub struct Seed {
    /// The file, relative to the user layer (`~/.config/kubaos`).
    pub path: &'static str,
    /// A comment on how the file is loaded, and a commented example.
    pub contents: &'static str,
}

/// One Suite's options, as its file `suites/<name>.toml` holds them.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuiteConfig {
    /// The chosen version, for a Suite that has versions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The extras turned on.
    #[serde(default)]
    pub extras: BTreeSet<String>,
}

/// A change of options for [`Suites::add`] or [`Suites::configure`]. What it leaves out stays as
/// it was, or takes its default on an add.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OptionsChange {
    pub version: Option<String>,
    /// Extras to turn on.
    pub enable: Vec<String>,
    /// Extras to turn off, applied after `enable`.
    pub disable: Vec<String>,
}

/// What removal does with a Suite's caches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CachePolicy {
    Keep,
    Purge,
}

/// What seeding the user layer did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Seeded {
    /// The files it created.
    pub created: Vec<PathBuf>,
    /// The files that existed and were left as they are.
    pub kept: Vec<PathBuf>,
}

/// What [`Suites::add`] did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Added {
    pub config: SuiteConfig,
    pub seeds: Seeded,
}

/// What [`Suites::configure`] did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Configured {
    pub config: SuiteConfig,
    /// Whether the file changed, and so a switch has something to do.
    pub changed: bool,
    pub seeds: Seeded,
}

/// One cache folder and how much it holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheUse {
    pub path: PathBuf,
    pub description: &'static str,
    /// The bytes in it, or `None` when it does not exist.
    pub bytes: Option<u64>,
}

/// What [`Suites::remove`] did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Removed {
    /// The Suite's caches, sized before any was deleted.
    pub caches: Vec<CacheUse>,
    /// The caches it deleted.
    pub purged: Vec<PathBuf>,
    /// The user-layer files that mention the Suite, which stay as they are.
    pub mentions: Vec<PathBuf>,
}

/// Installing, configuring or removing a Suite failed. The Suite file is as it was, unless the
/// error says otherwise.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SuiteError {
    #[error("the {suite} Suite is not installed")]
    NotInstalled { suite: String },
    #[error("the {suite} Suite is already installed; change it with `kubaos suite configure {suite}`")]
    AlreadyInstalled { suite: String },
    #[error("the {suite} Suite has no version {version:?} (it has: {})", known.join(", "))]
    UnknownVersion {
        suite: String,
        version: String,
        known: Vec<String>,
    },
    #[error("the {suite} Suite has no extra {extra:?} (it has: {})", known.join(", "))]
    UnknownExtra {
        suite: String,
        extra: String,
        known: Vec<String>,
    },
    /// The edit lock could not be taken.
    #[error("locking {}", path.display())]
    Lock {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// A Suite file could not be read or parsed.
    #[error("reading the Suite files")]
    Read(#[source] ReadError),
    /// The Suite file could not be written or deleted.
    #[error("writing {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The Suite file is written, but a missing user-layer file could not be created.
    #[error("creating {}", path.display())]
    Seed {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The Suite file is deleted, but a cache could not be.
    #[error("deleting {}", path.display())]
    Purge {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The Suite file is written or deleted, but git's index could not be brought in step with it,
    /// so Nix may not see the change.
    #[error("recording the Suite file in git")]
    Git(#[source] GitError),
}

/// Scaffolding a project failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ScaffoldError {
    #[error("the {suite} Suite is not installed; add it first with `kubaos suite add {suite}`")]
    NotInstalled { suite: String },
    /// The name is not a plain folder name: letters, digits, `.`, `_` and `-`, starting with a
    /// letter or digit.
    #[error("{0:?} is not a project name")]
    InvalidName(String),
    /// Something already has the project's path; it is left alone.
    #[error("{} exists", path.display())]
    Exists { path: PathBuf },
    /// The Suite files could not be read.
    #[error("reading the Suite files")]
    Read(#[source] ReadError),
    /// The Projects root could not be made.
    #[error("creating {}", path.display())]
    Root {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The scaffolder could not be started.
    #[error("starting {program}")]
    Spawn {
        program: String,
        #[source]
        source: io::Error,
    },
    /// The scaffolder exited with an error, or was killed (no code).
    #[error("{program} exited with {}", code.map_or_else(|| "a signal".to_owned(), |c| format!("code {c}")))]
    Failed { program: String, code: Option<i32> },
    /// The scaffolder succeeded but made no folder at the project's path.
    #[error("the scaffolder made nothing at {}", path.display())]
    NothingMade { path: PathBuf },
}

/// The Suites of one user: their files in the state directory, the user layer they seed, the
/// caches in the home and the Projects root they scaffold into.
#[derive(Clone, Debug)]
pub struct Suites {
    state: StateDir,
    lock: PathBuf,
    home: PathBuf,
    user_layer: PathBuf,
    projects: PathBuf,
}

impl Suites {
    /// The Suites of the user `paths` describes.
    #[must_use]
    pub fn new(paths: &Paths) -> Self {
        Self {
            state: paths.state_dir(),
            lock: paths.edit_lock(),
            home: paths.home().to_path_buf(),
            user_layer: paths.user_layer(),
            projects: paths.projects_root(),
        }
    }

    /// Every Suite file, by name, KubaOS's Suites or not.
    ///
    /// # Errors
    ///
    /// [`SuiteError::Read`] when a file cannot be read or parsed.
    pub fn installed(&self) -> Result<BTreeMap<String, SuiteConfig>, SuiteError> {
        let mut installed = BTreeMap::new();
        for (name, path) in self.state.suite_files().map_err(SuiteError::Read)? {
            if let Some(config) = read_config(&path).map_err(SuiteError::Read)? {
                installed.insert(name, config);
            }
        }
        Ok(installed)
    }

    /// The options of `suite`, or `None` when it is not installed.
    ///
    /// # Errors
    ///
    /// [`SuiteError::Read`] when its file cannot be read or parsed.
    pub fn config(&self, suite: &Suite) -> Result<Option<SuiteConfig>, SuiteError> {
        read_config(&self.state.suite_file(suite.name)).map_err(SuiteError::Read)
    }

    /// Installs `suite` with its default options changed by `change`: writes its file and seeds
    /// its missing user-layer files. The switch that applies it is the caller's.
    ///
    /// # Errors
    ///
    /// [`SuiteError::AlreadyInstalled`], an unknown version or extra, or a failed lock, read,
    /// write, seed or git sync.
    pub fn add(&self, suite: &Suite, change: &OptionsChange) -> Result<Added, SuiteError> {
        let _lock = self.lock()?;
        if self.config(suite)?.is_some() {
            return Err(SuiteError::AlreadyInstalled {
                suite: suite.name.to_owned(),
            });
        }
        let config = apply(suite, default_config(suite), change)?;
        self.write(suite, &config)?;
        let seeds = self.seed(suite)?;
        Ok(Added { config, seeds })
    }

    /// Changes the options of the installed `suite` by `change`, and seeds its missing user-layer
    /// files.
    ///
    /// # Errors
    ///
    /// [`SuiteError::NotInstalled`], an unknown version or extra, or a failed lock, read, write,
    /// seed or git sync.
    pub fn configure(&self, suite: &Suite, change: &OptionsChange) -> Result<Configured, SuiteError> {
        let _lock = self.lock()?;
        let Some(current) = self.config(suite)? else {
            return Err(SuiteError::NotInstalled {
                suite: suite.name.to_owned(),
            });
        };
        let config = apply(suite, current.clone(), change)?;
        let changed = config != current;
        if changed {
            self.write(suite, &config)?;
        }
        let seeds = self.seed(suite)?;
        Ok(Configured { config, changed, seeds })
    }

    /// Uninstalls `suite`: deletes its file, and its caches as `caches` says. The user layer and
    /// the Projects root stay as they are; the user-layer files that mention the Suite are listed.
    ///
    /// # Errors
    ///
    /// [`SuiteError::NotInstalled`], or a failed lock, read, delete, purge or git sync.
    pub fn remove(&self, suite: &Suite, caches: CachePolicy) -> Result<Removed, SuiteError> {
        let _lock = self.lock()?;
        if self.config(suite)?.is_none() {
            return Err(SuiteError::NotInstalled {
                suite: suite.name.to_owned(),
            });
        }
        let path = self.state.suite_file(suite.name);
        fs::remove_file(&path).map_err(|source| SuiteError::Write { path, source })?;
        self.state.sync_git().map_err(SuiteError::Git)?;

        let sized = self.caches(suite);
        let mut purged = Vec::new();
        if caches == CachePolicy::Purge {
            for cache in sized.iter().filter(|c| c.bytes.is_some()) {
                remove_path(&cache.path).map_err(|source| SuiteError::Purge {
                    path: cache.path.clone(),
                    source,
                })?;
                purged.push(cache.path.clone());
            }
        }
        Ok(Removed {
            caches: sized,
            purged,
            mentions: self.mentions(suite),
        })
    }

    /// The caches of `suite`, with their sizes. A cache inside the Projects root, or holding it,
    /// is left out, so that nothing deletes a project, and so is one whose path leaves the home.
    pub fn caches(&self, suite: &Suite) -> Vec<CacheUse> {
        suite
            .caches
            .iter()
            .filter(|cache| is_plain_relative(cache.path))
            .map(|cache| (cache, self.home.join(cache.path)))
            .filter(|(_, path)| !path.starts_with(&self.projects) && !self.projects.starts_with(path))
            .map(|(cache, path)| CacheUse {
                bytes: size_of(&path),
                description: cache.description,
                path,
            })
            .collect()
    }

    /// The user-layer files that mention `suite`, sorted. The state directory is the tool's and
    /// is not searched, nor is a `.git` folder, a link or a file over 1 MiB; a folder or file that
    /// cannot be read is skipped.
    pub fn mentions(&self, suite: &Suite) -> Vec<PathBuf> {
        let mut found = Vec::new();
        self.scan(&self.user_layer, suite, &mut found);
        found.sort();
        found
    }

    /// Creates a project called `name` in the Projects root with `scaffolder`, and returns its
    /// folder. The scaffolder's output goes to this process's.
    ///
    /// # Errors
    ///
    /// [`ScaffoldError`] when the Suite is not installed, the name is not a folder name, the
    /// folder exists, or the scaffolder fails or makes nothing.
    pub fn scaffold(&self, suite: &Suite, scaffolder: &Scaffolder, name: &str) -> Result<PathBuf, ScaffoldError> {
        if !is_project_name(name) {
            return Err(ScaffoldError::InvalidName(name.to_owned()));
        }
        let installed = read_config(&self.state.suite_file(suite.name)).map_err(ScaffoldError::Read)?;
        if installed.is_none() {
            return Err(ScaffoldError::NotInstalled {
                suite: suite.name.to_owned(),
            });
        }
        let path = self.projects.join(name);
        if fs::symlink_metadata(&path).is_ok() {
            return Err(ScaffoldError::Exists { path });
        }
        fs::create_dir_all(&self.projects).map_err(|source| ScaffoldError::Root {
            path: self.projects.clone(),
            source,
        })?;
        let command = expand(scaffolder.argv, name, &path);
        let Some((program, args)) = command.split_first() else {
            return Err(ScaffoldError::Spawn {
                program: String::new(),
                source: io::Error::other("the scaffolder has no command"),
            });
        };
        let program_name = program.to_string_lossy().into_owned();
        let status = Command::new(program)
            .args(args)
            .current_dir(&self.projects)
            .stdin(Stdio::null())
            .status()
            .map_err(|source| ScaffoldError::Spawn {
                program: program_name.clone(),
                source,
            })?;
        if !status.success() {
            return Err(ScaffoldError::Failed {
                program: program_name,
                code: status.code(),
            });
        }
        if !path.is_dir() {
            return Err(ScaffoldError::NothingMade { path });
        }
        Ok(path)
    }

    fn lock(&self) -> Result<File, SuiteError> {
        lock_file(&self.lock).map_err(|source| SuiteError::Lock {
            path: self.lock.clone(),
            source,
        })
    }

    /// Writes the Suite file, with a header saying who writes it, and records it in git.
    fn write(&self, suite: &Suite, config: &SuiteConfig) -> Result<(), SuiteError> {
        let path = self.state.suite_file(suite.name);
        let write_err = |source| SuiteError::Write {
            path: path.clone(),
            source,
        };
        let body = toml::to_string(config).map_err(|e| write_err(io::Error::other(e)))?;
        let header = format!(
            "# The {title} Suite, as `kubaos suite` installed it. The kubaos command rewrites this file:\n\
             # change it with `kubaos suite configure {name}`, and remove it with `kubaos suite remove {name}`.\n",
            title = suite.title,
            name = suite.name,
        );
        fs::create_dir_all(self.state.suites_dir()).map_err(write_err)?;
        write_atomic(&path, &format!("{header}{body}")).map_err(write_err)?;
        self.state.sync_git().map_err(SuiteError::Git)?;
        Ok(())
    }

    /// Creates each of the Suite's user-layer files that is missing; a path that exists, even as
    /// a dangling link, is the user's and is not written.
    fn seed(&self, suite: &Suite) -> Result<Seeded, SuiteError> {
        let mut seeded = Seeded::default();
        for seed in suite.seeds.iter().filter(|seed| is_plain_relative(seed.path)) {
            let path = self.user_layer.join(seed.path);
            let seed_err = |source| SuiteError::Seed {
                path: path.clone(),
                source,
            };
            if fs::symlink_metadata(&path).is_ok() {
                seeded.kept.push(path);
                continue;
            }
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir).map_err(seed_err)?;
            }
            match File::options().write(true).create_new(true).open(&path) {
                Ok(mut file) => file.write_all(seed.contents.as_bytes()).map_err(seed_err)?,
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                    seeded.kept.push(path);
                    continue;
                }
                Err(e) => return Err(seed_err(e)),
            }
            seeded.created.push(path);
        }
        Ok(seeded)
    }

    /// Adds the files under `dir` that mention `suite` to `found`.
    fn scan(&self, dir: &Path, suite: &Suite, found: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                if entry.file_name() != ".git" && path != self.state.root() {
                    self.scan(&path, suite, found);
                }
            } else if kind.is_file() && mentions(&path, suite.mentions) {
                found.push(path);
            }
        }
    }
}

/// A scaffolder's command with `{name}` and `{path}` filled in.
fn expand(argv: &[&str], name: &str, path: &Path) -> Vec<OsString> {
    argv.iter()
        .map(|arg| {
            let arg = arg.replace("{name}", name);
            let mut out = OsString::new();
            let mut parts = arg.split("{path}");
            out.push(parts.next().unwrap_or_default());
            for part in parts {
                out.push(path);
                out.push(part);
            }
            out
        })
        .collect()
}

/// Whether `name` is a Suite name: lowercase letters, digits and `-`, starting with a letter.
pub fn is_suite_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Whether `name` is a plain folder name for a new project.
fn is_project_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphanumeric())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// Whether `path` is relative and has only plain parts, so it stays below the folder it is joined
/// to.
fn is_plain_relative(path: &str) -> bool {
    !path.is_empty() && Path::new(path).components().all(|c| matches!(c, Component::Normal(_)))
}

fn read_config(path: &Path) -> Result<Option<SuiteConfig>, ReadError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(ReadError::read(path)(e)),
    };
    toml::from_str(&text).map(Some).map_err(ReadError::parse(path))
}

/// The options a new install of `suite` starts from.
fn default_config(suite: &Suite) -> SuiteConfig {
    SuiteConfig {
        version: suite.versions.first().map(|v| (*v).to_owned()),
        extras: suite
            .extras
            .iter()
            .filter(|e| e.default)
            .map(|e| e.name.to_owned())
            .collect(),
    }
}

/// `config` with `change` applied, checked against what `suite` offers.
fn apply(suite: &Suite, mut config: SuiteConfig, change: &OptionsChange) -> Result<SuiteConfig, SuiteError> {
    let known_extra = |name: &str| suite.extras.iter().any(|e| e.name == name);
    let unknown_extra = |extra: &str| SuiteError::UnknownExtra {
        suite: suite.name.to_owned(),
        extra: extra.to_owned(),
        known: suite.extras.iter().map(|e| e.name.to_owned()).collect(),
    };
    if let Some(version) = &change.version {
        config.version = Some(version.clone());
    }
    for extra in &change.enable {
        if !known_extra(extra) {
            return Err(unknown_extra(extra));
        }
        config.extras.insert(extra.clone());
    }
    for extra in &change.disable {
        if !known_extra(extra) && !config.extras.contains(extra) {
            return Err(unknown_extra(extra));
        }
        config.extras.remove(extra);
    }
    if let Some(version) = &config.version
        && !suite.versions.contains(&version.as_str())
    {
        return Err(SuiteError::UnknownVersion {
            suite: suite.name.to_owned(),
            version: version.clone(),
            known: suite.versions.iter().map(|v| (*v).to_owned()).collect(),
        });
    }
    if let Some(extra) = config.extras.iter().find(|e| !known_extra(e)) {
        return Err(unknown_extra(extra));
    }
    Ok(config)
}

/// The bytes under `path`, not following links, or `None` when nothing is there.
fn size_of(path: &Path) -> Option<u64> {
    let meta = fs::symlink_metadata(path).ok()?;
    if !meta.is_dir() {
        return Some(meta.len());
    }
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            total += size_of(&entry.path()).unwrap_or(0);
        }
    }
    Some(total)
}

/// Deletes the file, link or folder at `path`. A link is removed, not what it points to.
fn remove_path(path: &Path) -> io::Result<()> {
    if fs::symlink_metadata(path)?.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

/// Whether the file at `path` holds one of `words` as a whole word, ignoring case.
fn mentions(path: &Path, words: &[&str]) -> bool {
    let Ok(file) = File::open(path) else {
        return false;
    };
    if file.metadata().map_or(true, |m| m.len() > MENTION_SCAN_LIMIT) {
        return false;
    }
    let mut bytes = Vec::new();
    if file.take(MENTION_SCAN_LIMIT).read_to_end(&mut bytes).is_err() {
        return false;
    }
    let text = String::from_utf8_lossy(&bytes).to_ascii_lowercase();
    words.iter().any(|word| has_word(&text, &word.to_ascii_lowercase()))
}

/// Whether `word` appears in `text` with no word character (a letter, digit, `_` or `-`) on
/// either side.
fn has_word(text: &str, word: &str) -> bool {
    let is_word = |c: char| c.is_alphanumeric() || c == '_' || c == '-';
    text.match_indices(word).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + word.len()..].chars().next();
        !before.is_some_and(is_word) && !after.is_some_and(is_word)
    })
}

#[cfg(test)]
mod tests;
