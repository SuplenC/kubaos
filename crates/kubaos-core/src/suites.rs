//! Suites: installing, configuring and removing a whole programming setup, and scaffolding
//! projects with it.
//!
//! A Suite has two halves. Its Nix module, `home/suites/<name>.nix`, declares what it installs:
//! packages, environment variables, PATH entries and its editor settings in the Defaults. Its
//! metadata, a [`Suite`] in [`catalog`], says what the tool needs to know: its options (a version,
//! extras), its mise tools, where its toolchain comes from, its caches, the user-layer files it
//! seeds and the projects it scaffolds.
//!
//! Installing a Suite writes its record, `kubaos.d/suites/<name>.toml`, with its options, and
//! removing it deletes the record; the next switch applies either. A Suite with mise tools also
//! has a fragment of mise's config, `conf.d/kubaos-<name>.toml`, written whole: its tools are
//! installed first, with nothing written, and the fragment and the record are written only once
//! they are, so a failed install leaves nothing behind. Removal deletes the fragment and prunes
//! the versions nothing selects any more. The tool never writes to a user-layer file that exists:
//! it only creates a missing one, from the Suite's commented example. Removal deletes caches only
//! when asked, and never touches a project or a user-layer file.
//!
//! # Owns
//!
//! - The Suite metadata ([`Suite`]) and the catalog of every Suite.
//! - Writing, reading and deleting Suite records and their mise fragments, under the state
//!   directory's edit lock, and the mise calls each change makes.
//! - Seeding user-layer files, sizing and purging caches, and finding the user-layer files that
//!   mention a Suite.
//! - Running a Suite's scaffolders in the Projects root.
//!
//! # Not here
//!
//! - Reading the state directory's packages: `state.rs`.
//! - The switch that applies a Suite file: `switch.rs`.
//! - Running mise: `mise.rs`.
//! - The Nix side: `home/suites.nix`, `home/state.nix` and `home/suites/<name>.nix`.

pub mod catalog;
mod lockfile;
mod updates;

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
use std::sync::Arc;

use kubaos_user_layer::App;
use serde::Deserialize;
use serde::Serialize;

pub use self::updates::Available;
pub use self::updates::Reach;
pub use self::updates::SuiteUpdates;
pub use self::updates::Upgraded;
use crate::mise;
use crate::mise::ListedVersion;
use crate::mise::Mise;
use crate::mise::MiseError;
use crate::mise::ToolRequest;
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
    /// The tools mise installs for it, in its fragment. A Suite without any is installed by its
    /// Nix module alone.
    pub mise: &'static [Tool],
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

/// A tool mise installs for a Suite.
#[derive(Clone, Copy, Debug)]
pub struct Tool {
    /// mise's name for it: a registry name (`node`) or a backend's (`aqua:astral-sh/uv`).
    pub name: &'static str,
    /// The version a new install asks for.
    pub request: Request,
    /// The extra that installs it, or `None` when it is always installed.
    pub extra: Option<&'static str>,
}

/// The version a [`Tool`] asks mise for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// The Suite's chosen version: its main toolchain. The record keeps the line as chosen
    /// (`24`), and the fragment asks for the line's newest major.minor (`24.22`).
    Suite,
    /// This request, at major.minor precision (`0.13`), so a plain `mise upgrade` stays within
    /// its patch releases.
    Fixed(&'static str),
}

/// Where a Suite's toolchain comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Toolchain {
    /// nixpkgs, through the Suite's Nix module.
    Nix,
    /// mise, from the Suite's tools.
    Mise,
    /// The language's own manager, which the Suite installs (through its Nix module, or as one of
    /// its mise tools) and drives, where that is the norm (rustup for Rust, uv for Python, proto
    /// for moon).
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
    /// The app whose folder in the user layer holds the file.
    pub app: App,
    /// The file, relative to the app's folder.
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
    /// The tools its fragment asks mise for.
    pub tools: Vec<ToolRequest>,
    pub seeds: Seeded,
    /// The Suite as it was, for [`Suites::restore`] when the switch fails.
    pub before: Before,
}

/// What [`Suites::configure`] did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Configured {
    pub config: SuiteConfig,
    /// The tools its fragment asks mise for.
    pub tools: Vec<ToolRequest>,
    /// Whether the record changed, and so a switch has something to do.
    pub changed: bool,
    pub seeds: Seeded,
    /// The Suite as it was, for [`Suites::restore`] when the switch fails.
    pub before: Before,
}

/// A Suite's record and fragment as an add or configure found them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Before {
    /// Its options, or `None` when it was not installed.
    pub config: Option<SuiteConfig>,
    /// Its fragment's tools, empty when it had no fragment.
    pub tools: Vec<ToolRequest>,
}

/// One tool of an installed Suite, as its fragment asks for it and mise lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolState {
    pub tool: String,
    /// What the fragment asks for.
    pub request: String,
    /// The version mise selects for it, or `None` when mise lists none.
    pub version: Option<String>,
    /// Whether that version is installed.
    pub installed: bool,
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
    /// mise could not install the Suite's tools. Nothing is written, and the versions the
    /// install added are uninstalled.
    #[error("installing the {suite} Suite's tools with mise")]
    Install {
        suite: String,
        #[source]
        source: MiseError,
    },
    /// mise could not list its tools.
    #[error("listing mise's tools")]
    Tools(#[source] MiseError),
    /// The Suite's files are written or deleted, but mise could not delete the versions nothing
    /// selects any more.
    #[error("pruning mise's unused versions")]
    Prune(#[source] MiseError),
    /// The Suite's files are written or deleted, but mise could not write its lockfile.
    #[error("locking mise's tools")]
    MiseLock(#[source] MiseError),
    /// mise could not read which of the Suites' tools have newer releases.
    #[error("asking mise for newer releases")]
    Outdated(#[source] MiseError),
    /// mise could not move the Suites' tools; the tools it moved before failing stay moved.
    #[error("moving the Suites' tools with mise")]
    Upgrade(#[source] MiseError),
    /// mise's lockfile, or the entries an update saved from it, are not TOML.
    #[error("reading {}", path.display())]
    Lockfile {
        path: PathBuf,
        #[source]
        source: toml_edit::TomlError,
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
    /// The scaffolder's command isn't on `PATH`: the Suite's tools aren't installed.
    #[error("{program} isn't on PATH; `kubaos suite configure {suite}` installs the Suite's tools again")]
    NotOnPath { program: String, suite: String },
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

/// The Suites of one user: their records in the state directory, their fragments of mise's
/// config, the user layer they seed, the caches in the home and the Projects root they scaffold
/// into.
#[derive(Clone, Debug)]
pub struct Suites {
    paths: Paths,
    state: StateDir,
    lock: PathBuf,
    home: PathBuf,
    user_layer: PathBuf,
    projects: PathBuf,
    mise: Arc<dyn Mise>,
}

impl Suites {
    /// The Suites of the user `paths` describes, with the `mise` on `PATH`.
    #[must_use]
    pub fn new(paths: &Paths) -> Self {
        Self::with_mise(paths, Arc::new(mise::Cli::new()))
    }

    /// The Suites of the user `paths` describes, with `mise` running their tools.
    #[must_use]
    pub fn with_mise(paths: &Paths, mise: Arc<dyn Mise>) -> Self {
        Self {
            paths: paths.clone(),
            state: paths.state_dir(),
            lock: paths.edit_lock(),
            home: paths.home().to_path_buf(),
            user_layer: paths.user_layer(),
            projects: paths.projects_root(),
            mise,
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
        self.state.suite_config(suite.name).map_err(SuiteError::Read)
    }

    /// Installs `suite` with its default options changed by `change`. Its mise tools are
    /// installed first; only then are its fragment, mise's lockfile and its record written, and
    /// its missing user-layer files seeded. The switch that applies it is the caller's.
    ///
    /// # Errors
    ///
    /// [`SuiteError::AlreadyInstalled`], an unknown version or extra, a failed install (which
    /// leaves nothing behind), or a failed lock, read, write, seed, git sync or mise call.
    pub fn add(&self, suite: &Suite, change: &OptionsChange) -> Result<Added, SuiteError> {
        let _lock = self.lock()?;
        if self.config(suite)?.is_some() {
            return Err(SuiteError::AlreadyInstalled {
                suite: suite.name.to_owned(),
            });
        }
        let config = apply(suite, default_config(suite), change)?;
        let before = Before {
            config: None,
            tools: self.fragment(suite)?,
        };
        let tools = wanted_tools(suite, &config, &[], |tool, line| self.narrow(suite, tool, line))?;
        self.install(suite, &tools)?;
        if let Err(e) = self
            .write_fragment(suite, &tools)
            .and_then(|()| self.write(suite, &config))
        {
            self.take_out(suite);
            return Err(e);
        }
        if !tools.is_empty() {
            self.mise.lock().map_err(SuiteError::MiseLock)?;
        }
        let seeds = self.seed(suite)?;
        Ok(Added {
            config,
            tools,
            seeds,
            before,
        })
    }

    /// Changes the options of the installed `suite` by `change`, and seeds its missing user-layer
    /// files. Tools the change adds or moves are installed first, the fragment and the record
    /// written after, and the versions it no longer asks for pruned. A request `mise upgrade`
    /// moved in the fragment stays while it is still on the Suite's line.
    ///
    /// # Errors
    ///
    /// [`SuiteError::NotInstalled`], an unknown version or extra, a failed install (which
    /// changes nothing), or a failed lock, read, write, seed, git sync or mise call.
    pub fn configure(&self, suite: &Suite, change: &OptionsChange) -> Result<Configured, SuiteError> {
        let _lock = self.lock()?;
        let Some(current) = self.config(suite)? else {
            return Err(SuiteError::NotInstalled {
                suite: suite.name.to_owned(),
            });
        };
        let config = apply(suite, current.clone(), change)?;
        let changed = config != current;
        let before = Before {
            config: Some(current),
            tools: self.fragment(suite)?,
        };
        let tools = wanted_tools(suite, &config, &before.tools, |tool, line| {
            self.narrow(suite, tool, line)
        })?;
        let moved = !same_tools(&tools, &before.tools);
        if moved {
            self.install(suite, &tools)?;
            self.write_fragment(suite, &tools)?;
        }
        if changed {
            self.write(suite, &config)?;
        }
        if moved {
            self.mise.lock().map_err(SuiteError::MiseLock)?;
            self.mise.prune().map_err(SuiteError::Prune)?;
        }
        let seeds = self.seed(suite)?;
        Ok(Configured {
            config,
            tools,
            changed,
            seeds,
            before,
        })
    }

    /// Puts `suite` back as `before` found it, after the switch that followed an add or a
    /// configure failed: its record and fragment as they were, or gone when it was not
    /// installed, with the versions they no longer ask for pruned.
    ///
    /// # Errors
    ///
    /// A failed lock, write, git sync or mise call.
    pub fn restore(&self, suite: &Suite, before: &Before) -> Result<(), SuiteError> {
        let _lock = self.lock()?;
        if let Some(config) = &before.config {
            self.install(suite, &before.tools)?;
            self.write_fragment(suite, &before.tools)?;
            self.write(suite, config)?;
        } else {
            self.write_fragment(suite, &[])?;
            self.delete_record(suite)?;
        }
        if !suite.mise.is_empty() {
            self.mise.lock().map_err(SuiteError::MiseLock)?;
            self.mise.prune().map_err(SuiteError::Prune)?;
        }
        Ok(())
    }

    /// Uninstalls `suite`: deletes its record and its fragment, prunes the versions nothing
    /// selects any more, and deletes its caches as `caches` says. The user layer and the
    /// Projects root stay as they are; the user-layer files that mention the Suite are listed.
    ///
    /// # Errors
    ///
    /// [`SuiteError::NotInstalled`], or a failed lock, read, delete, purge, git sync or mise
    /// call.
    pub fn remove(&self, suite: &Suite, caches: CachePolicy) -> Result<Removed, SuiteError> {
        let _lock = self.lock()?;
        if self.config(suite)?.is_none() {
            return Err(SuiteError::NotInstalled {
                suite: suite.name.to_owned(),
            });
        }
        self.delete_record(suite)?;
        let had_fragment = self.paths.mise_fragment(suite.name).exists();
        self.write_fragment(suite, &[])?;
        // HAZARD: lock before pruning. While any global config is left (the user's
        // config.toml, a Suite's defaults in conf.d), mise reads the global lockfile with it,
        // and prune keeps every version the lockfile still lists.
        if had_fragment {
            self.mise.lock().map_err(SuiteError::MiseLock)?;
            self.mise.prune().map_err(SuiteError::Prune)?;
        }

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

    /// The tools of each of `suites` that has a fragment, as the fragment asks for them and mise
    /// lists them, by Suite name. mise runs once, and not at all when none has a fragment.
    ///
    /// # Errors
    ///
    /// [`SuiteError::Read`] when a fragment can't be read, or [`SuiteError::Tools`] when mise
    /// can't list its tools.
    pub fn tools<'a>(
        &self,
        suites: impl IntoIterator<Item = &'a Suite>,
    ) -> Result<BTreeMap<&'a str, Vec<ToolState>>, SuiteError> {
        let mut requested = BTreeMap::new();
        for suite in suites {
            let tools = self.fragment(suite)?;
            if !tools.is_empty() {
                requested.insert(suite.name, tools);
            }
        }
        if requested.is_empty() {
            return Ok(BTreeMap::new());
        }
        let listed = self.mise.list().map_err(SuiteError::Tools)?;
        Ok(requested
            .into_iter()
            .map(|(name, tools)| {
                let fragment = self.paths.mise_fragment(name);
                let states = tools.into_iter().map(|t| tool_state(t, &fragment, &listed)).collect();
                (name, states)
            })
            .collect())
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
            .map_err(|source| {
                if source.kind() == io::ErrorKind::NotFound {
                    ScaffoldError::NotOnPath {
                        program: program_name.clone(),
                        suite: suite.name.to_owned(),
                    }
                } else {
                    ScaffoldError::Spawn {
                        program: program_name.clone(),
                        source,
                    }
                }
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

    /// Installs `tools` with mise. When the install fails, the versions it added (a failed run
    /// keeps the tools that did install) are uninstalled again, as far as mise can, so a failure
    /// leaves mise as it found it.
    fn install(&self, suite: &Suite, tools: &[ToolRequest]) -> Result<(), SuiteError> {
        if tools.is_empty() {
            return Ok(());
        }
        let before = installed_versions(&self.mise.list().map_err(SuiteError::Tools)?);
        let Err(source) = self.mise.install(tools) else {
            return Ok(());
        };
        if let Ok(after) = self.mise.list() {
            let added: Vec<ToolRequest> = installed_versions(&after)
                .into_iter()
                .filter(|v| !before.contains(v))
                .collect();
            let _ = self.mise.uninstall(&added);
        }
        Err(SuiteError::Install {
            suite: suite.name.to_owned(),
            source,
        })
    }

    /// The request for the main tool `tool` of `suite` on the chosen `line`: the major.minor of
    /// the line's newest release (`24` asks for `24.22`), so a plain update takes only its patch
    /// releases. A line with no release mise knows of, or whose newest release isn't numbered
    /// major.minor.something, stays as chosen.
    fn narrow(&self, suite: &Suite, tool: &str, line: &str) -> Result<String, SuiteError> {
        let latest = self.mise.latest(tool, line).map_err(|source| SuiteError::Install {
            suite: suite.name.to_owned(),
            source,
        })?;
        Ok(latest
            .as_deref()
            .and_then(major_minor)
            .filter(|request| on_line(request, line))
            .unwrap_or_else(|| line.to_owned()))
    }

    /// The tools `suite`'s fragment asks for, in the order it lists them; none without a
    /// fragment. A request that isn't a plain string is mise's to read and is left out.
    fn fragment(&self, suite: &Suite) -> Result<Vec<ToolRequest>, SuiteError> {
        let path = self.paths.mise_fragment(suite.name);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(SuiteError::Read(ReadError::read(&path)(e))),
        };
        let fragment: Fragment = toml::from_str(&text).map_err(|e| SuiteError::Read(ReadError::parse(&path)(e)))?;
        Ok(fragment
            .tools
            .into_iter()
            .filter_map(|(tool, request)| match request {
                toml::Value::String(request) => Some(ToolRequest { tool, request }),
                _ => None,
            })
            .collect())
    }

    /// Writes `suite`'s fragment whole, with a header saying who writes it, or deletes it when
    /// `tools` is empty.
    fn write_fragment(&self, suite: &Suite, tools: &[ToolRequest]) -> Result<(), SuiteError> {
        let path = self.paths.mise_fragment(suite.name);
        let write_err = |source| SuiteError::Write {
            path: path.clone(),
            source,
        };
        if tools.is_empty() {
            return match fs::remove_file(&path) {
                Err(e) if e.kind() != io::ErrorKind::NotFound => Err(write_err(e)),
                _ => Ok(()),
            };
        }
        let fragment = Fragment {
            tools: tools
                .iter()
                .map(|t| (t.tool.clone(), toml::Value::String(t.request.clone())))
                .collect(),
        };
        let body = toml::to_string(&fragment).map_err(|e| write_err(io::Error::other(e)))?;
        let header = format!(
            "# The {title} Suite's tools, as `kubaos suite` installed them. The kubaos command rewrites this\n\
             # file: change it with `kubaos suite configure {name}`, and remove it with `kubaos suite remove {name}`.\n\
             # Your own ~/.config/mise/config.toml loads after it, so what you set there wins.\n",
            title = suite.title,
            name = suite.name,
        );
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(write_err)?;
        }
        write_atomic(&path, &format!("{header}{body}")).map_err(write_err)
    }

    /// Deletes `suite`'s record and records that in git.
    fn delete_record(&self, suite: &Suite) -> Result<(), SuiteError> {
        let path = self.state.suite_file(suite.name);
        match fs::remove_file(&path) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(SuiteError::Write { path, source: e }),
            _ => {}
        }
        self.state.sync_git().map(drop).map_err(SuiteError::Git)
    }

    /// Takes a half-written add back out: the fragment and the record, then prunes the versions
    /// the install added. Each step is tried whatever the last did, and their errors are dropped:
    /// the error that led here is the one to report.
    fn take_out(&self, suite: &Suite) {
        let _ = self.write_fragment(suite, &[]);
        let _ = self.delete_record(suite);
        if !suite.mise.is_empty() {
            let _ = self.mise.prune();
        }
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
            let path = self.user_layer.join(seed.app.relative()).join(seed.path);
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

/// A Suite's fragment of mise's config: only its tools. KubaOS writes no setting there, so
/// every setting stays the user's.
#[derive(Serialize, Deserialize)]
struct Fragment {
    #[serde(default)]
    tools: toml::Table,
}

/// The tools `suite` asks mise for with `config`. A request in `current`, the fragment as it is,
/// stays while it is on the line the Suite asks for, so a request `mise upgrade --bump` moved is
/// kept: for the main toolchain, while it starts with the chosen version; for any other tool,
/// always. The main toolchain's request on a new line is what `narrow` makes of the line.
fn wanted_tools(
    suite: &Suite,
    config: &SuiteConfig,
    current: &[ToolRequest],
    mut narrow: impl FnMut(&str, &str) -> Result<String, SuiteError>,
) -> Result<Vec<ToolRequest>, SuiteError> {
    let mut tools = Vec::new();
    for tool in suite.mise {
        if tool.extra.is_some_and(|extra| !config.extras.contains(extra)) {
            continue;
        }
        let kept = current.iter().find(|t| t.tool == tool.name).map(|t| t.request.clone());
        let request = match (tool.request, kept) {
            (Request::Fixed(_), Some(kept)) => kept,
            (Request::Fixed(request), None) => request.to_owned(),
            (Request::Suite, kept) => {
                let Some(line) = &config.version else {
                    continue;
                };
                match kept {
                    Some(kept) if on_line(&kept, line) => kept,
                    _ => narrow(tool.name, line)?,
                }
            }
        };
        tools.push(ToolRequest {
            tool: tool.name.to_owned(),
            request,
        });
    }
    Ok(tools)
}

/// `version`'s major.minor, `24.22` of `24.22.1`, when its first two parts are numbers and a
/// third follows.
fn major_minor(version: &str) -> Option<String> {
    let mut parts = version.splitn(3, '.');
    let (major, minor, _rest) = (parts.next()?, parts.next()?, parts.next()?);
    let numeric = |part: &str| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
    (numeric(major) && numeric(minor)).then(|| format!("{major}.{minor}"))
}

/// Whether `a` and `b` ask for the same tools at the same requests. A fragment reads back with
/// its tools sorted by name, the catalog lists them in its own order.
fn same_tools(a: &[ToolRequest], b: &[ToolRequest]) -> bool {
    a.iter().collect::<BTreeSet<_>>() == b.iter().collect::<BTreeSet<_>>()
}

/// Whether `request` is `line` or a narrower request within it: `24.22` is on `24`.
fn on_line(request: &str, line: &str) -> bool {
    request == line || request.strip_prefix(line).is_some_and(|rest| rest.starts_with('.'))
}

/// The versions in `listed` that are installed, as `tool@version`.
fn installed_versions(listed: &[ListedVersion]) -> BTreeSet<ToolRequest> {
    listed
        .iter()
        .filter(|v| v.installed)
        .map(|v| ToolRequest {
            tool: v.tool.clone(),
            request: v.version.clone(),
        })
        .collect()
}

/// `tool` as the fragment at `fragment` asks for it, with the version mise selects for it there.
fn tool_state(tool: ToolRequest, fragment: &Path, listed: &[ListedVersion]) -> ToolState {
    let selected = listed
        .iter()
        .find(|v| v.tool == tool.tool && v.source.as_deref().is_some_and(|source| same_file(source, fragment)));
    ToolState {
        version: selected.map(|v| v.version.clone()),
        installed: selected.is_some_and(|v| v.installed),
        tool: tool.tool,
        request: tool.request,
    }
}

/// Whether `a` and `b` name the same file, links resolved.
fn same_file(a: &Path, b: &Path) -> bool {
    a == b || matches!((fs::canonicalize(a), fs::canonicalize(b)), (Ok(a), Ok(b)) if a == b)
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

impl StateDir {
    /// The options of the Suite `name` as its file holds them, or `None` when it is not
    /// installed.
    ///
    /// # Errors
    ///
    /// [`ReadError`] when the file exists but cannot be read or parsed.
    pub fn suite_config(&self, name: &str) -> Result<Option<SuiteConfig>, ReadError> {
        read_config(&self.suite_file(name))
    }
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
