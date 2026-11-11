//! Project pins: a folder's choice of a Suite's version, for that folder and everything below it.
//!
//! A pin is the Suite's main tool in the `[tools]` table of the folder's `mise.toml`
//! (`node = "22"`). mise's shell hook puts the version it selects on PATH, ahead of the installed
//! Suite's: the nearest config that names the tool wins, and every folder below it inherits it.
//! Only a Suite whose toolchain comes from mise, a mise tool that takes the Suite's version, can
//! be pinned.
//!
//! # Owns
//!
//! - The pins in effect in a folder, from mise's own resolution ([`Pins::resolve`]), and every pin
//!   under the Projects root ([`Pins::all`]).
//! - Writing and removing a pin, keeping the rest of the file ([`Pins::pin`], [`Pins::unpin`]).
//! - The `.envrc` lines direnv used to read, `use suite` and `use bwenv`: finding the ones left
//!   ([`Leftover`]) and moving them to the folder's `mise.toml` ([`Pins::migrate`]).
//!
//! # Not here
//!
//! - The Suites and their installed versions: `suites.rs`.
//! - Running mise: `mise.rs`.
//! - The script a `use bwenv` becomes, which reads the keyring: `home/secrets.nix`.

use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use std::time::SystemTime;

use toml_edit::DocumentMut;
use toml_edit::Item;
use toml_edit::TableLike;

use crate::mise;
use crate::mise::ListedVersion;
use crate::mise::Mise;
use crate::mise::MiseError;
use crate::mise::ToolRequest;
use crate::state::Paths;
use crate::suites::Request;
use crate::suites::Suite;
use crate::suites::SuiteConfig;
use crate::suites::SuiteError;
use crate::suites::Suites;
use crate::suites::catalog;
use crate::suites::is_suite_name;

/// The file a new pin is written to.
pub const MISE_TOML: &str = "mise.toml";

/// The configs of a folder a person writes and mise reads, in the order a pin is looked for in
/// them. A pin goes into the first that names the tool, or into [`MISE_TOML`].
pub const CONFIG_NAMES: &[&str] = &[MISE_TOML, ".mise.toml", "mise.local.toml", ".mise.local.toml"];

/// The file direnv loaded, whose `use suite` and `use bwenv` lines nothing reads any more.
pub const ENVRC: &str = ".envrc";

/// The variable that names a folder's Bitwarden projects for the script its `mise.toml` sources
/// ([`Paths::bwenv_script`]).
pub const BWENV_VAR: &str = "KUBAOS_BWENV";

/// How deep below the Projects root [`Pins::all`] and [`Pins::migrate`] look: a pin goes on a
/// group of projects or on one, rarely deeper.
pub const SCAN_DEPTH: usize = 4;

/// Folders [`Pins::all`] never enters, besides hidden ones: dependencies and build output, which
/// hold no pins of the user's and can be huge.
const SKIPPED: &[&str] = &["node_modules", "target", "vendor", "dist", "build"];

/// One pin: `<tool> = "<version>"` in `config`, where `tool` is the Suite's main tool.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pin {
    pub suite: String,
    /// mise's name for the Suite's main tool.
    pub tool: String,
    /// The version asked for, as the file says it.
    pub version: String,
    /// The config the pin is in.
    pub config: PathBuf,
}

/// What a pin stands for on this machine.
#[derive(Clone, Debug)]
pub struct PinStatus {
    pub pin: Pin,
    pub suite: &'static Suite,
    /// The Suite's options, when it is installed.
    pub installed: Option<SuiteConfig>,
    /// The exact version mise selects for the pin, or `None` when another config's pin of the
    /// tool wins in the pin's own folder.
    pub selected: Option<String>,
    /// Whether mise has that version installed.
    pub fetched: bool,
}

impl PinStatus {
    /// The version installed for the whole home, if the Suite is installed with one.
    pub fn global_version(&self) -> Option<&str> {
        self.installed.as_ref()?.version.as_deref()
    }
}

/// An `.envrc` line from before pins moved to mise, which nothing reads now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Leftover {
    pub envrc: PathBuf,
    /// The line's number, from 1.
    pub line: usize,
    pub kind: LeftoverKind,
}

/// What a [`Leftover`] line asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LeftoverKind {
    /// `use suite <suite> <version>`.
    Suite { suite: String, version: String },
    /// `use bwenv <project>…`.
    Bwenv { projects: Vec<String> },
}

/// The pins in effect in a folder.
#[derive(Clone, Debug, Default)]
pub struct Resolution {
    /// One per Suite, the one that wins, by tool.
    pub pins: Vec<PinStatus>,
    /// `use suite` and `use bwenv` lines in the `.envrc` files of the folder and those above it.
    pub leftovers: Vec<Leftover>,
}

/// Every pin under the Projects root.
#[derive(Clone, Debug, Default)]
pub struct Found {
    /// By config, then by tool.
    pub pins: Vec<Pin>,
    /// `use suite` and `use bwenv` lines in an `.envrc` under the root.
    pub leftovers: Vec<Leftover>,
}

/// What [`Pins::pin`] did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pinned {
    pub config: PathBuf,
    /// Whether the config is new.
    pub created: bool,
    /// The version the config pinned before, if any.
    pub previous: Option<String>,
    /// The pin of the tool mise selects in the folder now, which is another config's when one of
    /// higher precedence in the folder names the tool too.
    pub applies: Option<Pin>,
}

/// What [`Pins::unpin`] did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unpinned {
    pub config: PathBuf,
    /// The version that was pinned.
    pub version: String,
    /// Whether the config held nothing else, and so is deleted.
    pub deleted: bool,
}

/// What [`Pins::migrate`] did to one `.envrc`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Migrated {
    pub envrc: PathBuf,
    pub config: PathBuf,
    /// The pins moved, as `(suite, version)`.
    pub pins: Vec<(String, String)>,
    /// The `use suite` lines whose Suite the config already pinned, dropped for the config's pin,
    /// which mise put first on PATH all along.
    pub kept: Vec<(String, String)>,
    /// The Bitwarden projects moved.
    pub bwenv: Vec<String>,
    /// Whether the `.envrc` held nothing else, and so is deleted.
    pub envrc_deleted: bool,
}

/// Reading, writing or applying a pin failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PinError {
    #[error("{} is not a folder", path.display())]
    NotAFolder { path: PathBuf },
    #[error("no Suite is called {suite} (`kubaos suite list`)")]
    UnknownSuite { suite: String },
    /// The Suite's toolchain doesn't come from mise yet, so mise can't pin it.
    #[error("the {suite} Suite's toolchain doesn't come from mise yet, so it can't be pinned")]
    NotOnMise { suite: String },
    #[error("the {suite} Suite has no versions to pin")]
    Unversioned { suite: String },
    #[error("{version:?} is not a version: use letters, digits, `.`, `_` and `-`")]
    InvalidVersion { version: String },
    /// The folder's own configs don't pin the Suite. `applies_from` is the config whose pin of it
    /// is in effect there, if one is.
    #[error("{} pins no {suite} version", folder.display())]
    NotPinned {
        suite: String,
        folder: PathBuf,
        applies_from: Option<PathBuf>,
    },
    #[error("reading {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{} is not TOML", path.display())]
    Parse {
        path: PathBuf,
        #[source]
        source: toml_edit::TomlError,
    },
    #[error("writing {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The config names the tool in a way only the user should change; nothing is written.
    #[error("{} pins {tool} in a way kubaos leaves to you: {why}; edit it by hand", config.display())]
    Tangled { tool: String, config: PathBuf, why: Tangle },
    /// mise could not install the version; nothing is written.
    #[error("installing {tool}@{version} with mise")]
    Install {
        tool: String,
        version: String,
        #[source]
        source: MiseError,
    },
    /// The config is written, but mise could not say which versions apply in the folder.
    #[error("asking mise which versions apply in {}", folder.display())]
    Resolve {
        folder: PathBuf,
        #[source]
        source: MiseError,
    },
    /// The config is written, but mise could not delete the versions nothing selects any more.
    #[error("pruning mise's unused versions")]
    Prune(#[source] MiseError),
    /// The config is written, but mise could not trust it.
    #[error("trusting {} in mise", path.display())]
    Trust {
        path: PathBuf,
        #[source]
        source: MiseError,
    },
    /// The installed Suites could not be read.
    #[error("reading the installed Suites")]
    Suites(#[source] SuiteError),
}

/// Why a pin is left to the user.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Tangle {
    /// The tool's entry is a table or a list, with options or several versions.
    #[error("its entry is not a plain version")]
    NotAString,
    #[error("`tools` is not a table")]
    ToolsNotATable,
}

/// One line of an `.envrc`, as far as the old pins go.
#[derive(Debug, PartialEq, Eq)]
enum Line {
    Pin { suite: String, version: String },
    Bwenv { projects: Vec<String> },
    Other,
}

/// The Project pins of one user: the configs they live in, mise, and the installed Suites they
/// override.
#[derive(Clone, Debug)]
pub struct Pins {
    suites: Suites,
    projects: PathBuf,
    backups: PathBuf,
    mise_config: PathBuf,
    /// [`Paths::bwenv_script`], as a `mise.toml` names it: under `~` when it is in the home, so
    /// a project's file reads the same for everyone who uses KubaOS.
    bwenv_script: String,
    mise: Arc<dyn Mise>,
}

impl Pins {
    /// The Project pins of the user `paths` describes, with the `mise` on `PATH`.
    #[must_use]
    pub fn new(paths: &Paths) -> Self {
        Self::with_mise(paths, Arc::new(mise::Cli::new()))
    }

    /// The Project pins of the user `paths` describes, with `mise` reading and installing them.
    #[must_use]
    pub fn with_mise(paths: &Paths, mise: Arc<dyn Mise>) -> Self {
        Self {
            suites: Suites::with_mise(paths, Arc::clone(&mise)),
            projects: paths.projects_root(),
            backups: paths.backups(),
            mise_config: paths.mise_config().to_path_buf(),
            bwenv_script: match paths.bwenv_script().strip_prefix(paths.home()) {
                Ok(rest) => format!("~/{}", rest.display()),
                Err(_) => paths.bwenv_script().display().to_string(),
            },
            mise,
        }
    }

    /// The pins in effect in `folder`, as mise selects them there, and the `.envrc` lines left in
    /// the folder and above it.
    ///
    /// # Errors
    ///
    /// [`PinError::NotAFolder`], [`PinError::Resolve`] when mise fails (an untrusted config in
    /// the folder, say), or a failed read.
    pub fn resolve(&self, folder: &Path) -> Result<Resolution, PinError> {
        if !folder.is_dir() {
            return Err(PinError::NotAFolder {
                path: folder.to_path_buf(),
            });
        }
        let listed = self.current(folder)?;
        let mut pins = Vec::new();
        for entry in listed {
            if let Some(pin) = self.project_pin(&entry) {
                pins.push(self.status(pin, Some(&entry))?);
            }
        }
        let mut leftovers = Vec::new();
        for envrc in folder.ancestors().map(|dir| dir.join(ENVRC)).filter(|p| p.is_file()) {
            leftovers.extend(read_leftovers(&envrc)?);
        }
        Ok(Resolution { pins, leftovers })
    }

    /// What each of `pins` stands for here: its Suite, the installed version, and the version
    /// mise selects for it in its folder. mise runs once per folder.
    ///
    /// # Errors
    ///
    /// [`PinError::Resolve`] when mise fails, or [`PinError::Suites`].
    pub fn statuses(&self, pins: Vec<Pin>) -> Result<Vec<PinStatus>, PinError> {
        let mut by_folder: BTreeMap<PathBuf, Vec<ListedVersion>> = BTreeMap::new();
        let mut statuses = Vec::new();
        for pin in pins {
            let folder = pin.config.parent().unwrap_or(&pin.config).to_path_buf();
            if !by_folder.contains_key(&folder) {
                let listed = self.current(&folder)?;
                by_folder.insert(folder.clone(), listed);
            }
            let entry = by_folder[&folder]
                .iter()
                .find(|entry| entry.tool == pin.tool && entry.source.as_deref() == Some(pin.config.as_path()))
                .cloned();
            statuses.push(self.status(pin, entry.as_ref())?);
        }
        Ok(statuses)
    }

    /// Pins `suite` at `version` in `folder`: installs that version with mise first, then sets the
    /// Suite's main tool in the folder's config that names it already, or in its `mise.toml`,
    /// created when missing. The file's other entries, comments and layout stay. Then mise reads
    /// the folder, which also tracks the config, so pruning keeps the version while it exists.
    ///
    /// # Errors
    ///
    /// A Suite that isn't on mise or has no versions, an invalid version,
    /// [`PinError::NotAFolder`], [`PinError::Tangled`], a failed read, parse or write, or
    /// [`PinError::Install`], all with nothing written; or, with the file written,
    /// [`PinError::Resolve`].
    pub fn pin(&self, suite: &Suite, version: &str, folder: &Path) -> Result<Pinned, PinError> {
        let tool = main_tool(suite)?;
        if !is_version(version) {
            return Err(PinError::InvalidVersion {
                version: version.to_owned(),
            });
        }
        if !folder.is_dir() {
            return Err(PinError::NotAFolder {
                path: folder.to_path_buf(),
            });
        }
        let config = config_for(folder, tool)?;
        let current = read_config(&config)?;
        let created = current.is_none();
        let text = current.unwrap_or_default();
        let mut doc = parse_config(&config, &text)?;
        let previous = set_tool(&mut doc, &config, tool, version)?;

        self.mise
            .install(&[ToolRequest {
                tool: tool.to_owned(),
                request: version.to_owned(),
            }])
            .map_err(|source| PinError::Install {
                tool: tool.to_owned(),
                version: version.to_owned(),
                source,
            })?;
        let rendered = doc.to_string();
        if rendered != text {
            write_config(&config, &rendered)?;
        }
        let applies = self
            .current(folder)?
            .iter()
            .filter(|entry| entry.tool == tool)
            .find_map(|entry| self.project_pin(entry));
        Ok(Pinned {
            config,
            created,
            previous,
            applies,
        })
    }

    /// Removes `folder`'s pin of the Suite called `suite` from the folder's config that holds it,
    /// then prunes, so mise deletes the version when nothing else selects it. A config left with
    /// nothing in it is deleted.
    ///
    /// # Errors
    ///
    /// An unknown Suite or one not on mise, [`PinError::NotPinned`] when none of the folder's
    /// configs pins it, [`PinError::Tangled`], a failed read, parse or write, or, with the file
    /// written, [`PinError::Prune`].
    pub fn unpin(&self, suite: &str, folder: &Path) -> Result<Unpinned, PinError> {
        let known = catalog::find(suite).ok_or_else(|| PinError::UnknownSuite {
            suite: suite.to_owned(),
        })?;
        let tool = main_tool(known)?;
        if !folder.is_dir() {
            return Err(PinError::NotAFolder {
                path: folder.to_path_buf(),
            });
        }
        let config = config_for(folder, tool)?;
        let text = read_config(&config)?.unwrap_or_default();
        let mut doc = parse_config(&config, &text)?;
        let Some(version) = remove_tool(&mut doc, &config, tool)? else {
            let applies_from = self
                .resolve(folder)?
                .pins
                .into_iter()
                .find(|status| status.pin.tool == tool)
                .map(|status| status.pin.config);
            return Err(PinError::NotPinned {
                suite: suite.to_owned(),
                folder: folder.to_path_buf(),
                applies_from,
            });
        };
        let rendered = doc.to_string();
        let deleted = rendered.trim().is_empty();
        if deleted {
            fs::remove_file(&config).map_err(|source| PinError::Write {
                path: config.clone(),
                source,
            })?;
        } else {
            write_config(&config, &rendered)?;
        }
        self.mise.prune().map_err(PinError::Prune)?;
        Ok(Unpinned {
            config,
            version,
            deleted,
        })
    }

    /// Every pin in a config under the Projects root, [`SCAN_DEPTH`] folders deep at most, and
    /// every `.envrc` line left from before. Hidden folders, dependency and build folders, and
    /// links are not entered.
    ///
    /// # Errors
    ///
    /// A config or `.envrc` that can't be read, or a config that isn't TOML. A folder that can't
    /// be listed is skipped.
    pub fn all(&self) -> Result<Found, PinError> {
        let mut files = Vec::new();
        find_files(&self.projects, SCAN_DEPTH, &mut files);
        files.sort();
        let mut found = Found::default();
        for file in files {
            if file.file_name().is_some_and(|name| name == ENVRC) {
                found.leftovers.extend(read_leftovers(&file)?);
            } else {
                found.pins.extend(read_pins(&file)?);
            }
        }
        Ok(found)
    }

    /// Moves every `use suite` and `use bwenv` line in an `.envrc` under the Projects root to the
    /// folder's config: a pin becomes the Suite's main tool in `[tools]`, and `use bwenv
    /// <project>…` becomes `KUBAOS_BWENV` and a `_.source` of [`Paths::bwenv_script`] in
    /// `[env]`, trusted in mise since the user wrote the line. Only a lone, top-level line
    /// moves, of a Suite on mise; any other stays where it is, for [`Pins::all`] to report.
    /// Each file is backed up under [`Paths::backups`] before it changes, and an `.envrc` left
    /// with nothing but comments, blank lines and `source_up` is deleted. Running it again
    /// changes nothing.
    ///
    /// # Errors
    ///
    /// A failed read, parse, backup, write or trust; the files before it are migrated.
    pub fn migrate(&self) -> Result<Vec<Migrated>, PinError> {
        let mut files = Vec::new();
        find_files(&self.projects, SCAN_DEPTH, &mut files);
        files.retain(|file| file.file_name().is_some_and(|name| name == ENVRC));
        files.sort();
        let stamp = backup_stamp();
        let mut migrated = Vec::new();
        for envrc in files {
            if let Some(done) = self.migrate_envrc(&envrc, &stamp)? {
                migrated.push(done);
            }
        }
        Ok(migrated)
    }

    fn migrate_envrc(&self, envrc: &Path, stamp: &str) -> Result<Option<Migrated>, PinError> {
        let text = read_config(envrc)?.unwrap_or_default();
        let lines: Vec<&str> = text.lines().collect();
        let movable = movable_lines(&lines);
        if movable.is_empty() {
            return Ok(None);
        }
        let folder = envrc.parent().unwrap_or(envrc);
        let mut configs: BTreeMap<PathBuf, (String, DocumentMut)> = BTreeMap::new();
        let mut done = Migrated {
            envrc: envrc.to_path_buf(),
            config: folder.join(MISE_TOML),
            pins: Vec::new(),
            kept: Vec::new(),
            bwenv: Vec::new(),
            envrc_deleted: false,
        };
        let mut moved = Vec::new();
        for (at, line) in movable {
            match line {
                Line::Pin { suite, version } => {
                    let Some(tool) = catalog::find(&suite).and_then(|suite| main_tool(suite).ok()) else {
                        continue;
                    };
                    let config = config_for(folder, tool)?;
                    let (_, doc) = load_doc(&mut configs, &config)?;
                    if tool_entry(doc, &config, tool)?.is_some() {
                        done.kept.push((suite, version));
                    } else {
                        set_tool(doc, &config, tool, &version)?;
                        done.pins.push((suite, version));
                        done.config = config;
                    }
                }
                Line::Bwenv { projects } => {
                    let config = folder.join(MISE_TOML);
                    let (_, doc) = load_doc(&mut configs, &config)?;
                    if !set_bwenv(doc, &projects, &self.bwenv_script) {
                        continue;
                    }
                    done.bwenv.extend(projects);
                    done.config = config;
                }
                Line::Other => continue,
            }
            moved.push(at);
        }
        if moved.is_empty() {
            return Ok(None);
        }

        backup(&self.backups, stamp, envrc)?;
        for (config, (text, doc)) in &configs {
            let rendered = doc.to_string();
            if rendered != *text {
                backup(&self.backups, stamp, config)?;
                write_config(config, &rendered)?;
            }
        }
        let rest: Vec<&str> = lines
            .iter()
            .enumerate()
            .filter(|(at, _)| !moved.contains(at))
            .map(|(_, line)| *line)
            .collect();
        done.envrc_deleted = rest.iter().all(|line| {
            let line = line.trim();
            line.is_empty() || line.starts_with('#') || matches!(line, "source_up" | "source_up_if_exists")
        });
        if done.envrc_deleted {
            fs::remove_file(envrc).map_err(|source| PinError::Write {
                path: envrc.to_path_buf(),
                source,
            })?;
        } else {
            let mut kept = rest.join("\n");
            kept.push('\n');
            write_config(envrc, &kept)?;
        }
        for (suite, _) in done.pins.iter().chain(&done.kept) {
            remove_roots(folder, suite);
        }
        if !done.bwenv.is_empty() {
            self.mise.trust(&done.config).map_err(|source| PinError::Trust {
                path: done.config.clone(),
                source,
            })?;
        }
        Ok(Some(done))
    }

    /// The versions mise selects in `folder`.
    fn current(&self, folder: &Path) -> Result<Vec<ListedVersion>, PinError> {
        self.mise.current(folder).map_err(|source| PinError::Resolve {
            folder: folder.to_path_buf(),
            source,
        })
    }

    /// The pin `entry` of `mise ls --current` stands for: a Suite's main tool from a project's
    /// config, not mise's global configs or KubaOS's fragments.
    fn project_pin(&self, entry: &ListedVersion) -> Option<Pin> {
        let config = entry.source.as_ref()?;
        if config.starts_with(&self.mise_config) || config.starts_with("/etc") {
            return None;
        }
        let suite = suite_of_tool(&entry.tool)?;
        Some(Pin {
            suite: suite.name.to_owned(),
            tool: entry.tool.clone(),
            version: entry.requested.clone()?,
            config: config.clone(),
        })
    }

    fn status(&self, pin: Pin, entry: Option<&ListedVersion>) -> Result<PinStatus, PinError> {
        let suite = catalog::find(&pin.suite).ok_or_else(|| PinError::UnknownSuite {
            suite: pin.suite.clone(),
        })?;
        let installed = self.suites.config(suite).map_err(PinError::Suites)?;
        Ok(PinStatus {
            pin,
            suite,
            installed,
            selected: entry.map(|entry| entry.version.clone()),
            fetched: entry.is_some_and(|entry| entry.installed),
        })
    }
}

/// The tool a pin of `suite` sets: its main toolchain, the mise tool that takes the Suite's
/// version.
///
/// # Errors
///
/// [`PinError::NotOnMise`] when the Suite has versions but no mise tool takes them, or
/// [`PinError::Unversioned`] when it has no versions.
pub fn main_tool(suite: &Suite) -> Result<&'static str, PinError> {
    if let Some(tool) = suite.mise.iter().find(|tool| tool.request == Request::Suite) {
        return Ok(tool.name);
    }
    let name = suite.name.to_owned();
    if suite.versions.is_empty() {
        Err(PinError::Unversioned { suite: name })
    } else {
        Err(PinError::NotOnMise { suite: name })
    }
}

/// The Suite whose main tool is `tool`.
fn suite_of_tool(tool: &str) -> Option<&'static Suite> {
    catalog::all().find(|suite| main_tool(suite).is_ok_and(|main| main == tool))
}

/// Whether `version` is one plain word mise can take as a request: letters, digits, `.`, `_` and
/// `-`.
fn is_version(version: &str) -> bool {
    !version.is_empty()
        && !version.starts_with(['.', '-'])
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// The config of `folder` that names `tool`, in [`CONFIG_NAMES`] order, or its `mise.toml`.
fn config_for(folder: &Path, tool: &str) -> Result<PathBuf, PinError> {
    for name in CONFIG_NAMES {
        let config = folder.join(name);
        let Some(text) = read_config(&config)? else {
            continue;
        };
        if tool_entry(&parse_config(&config, &text)?, &config, tool)?.is_some() {
            return Ok(config);
        }
    }
    Ok(folder.join(MISE_TOML))
}

/// The pins in `config`'s `[tools]`, by tool. An entry that isn't a plain version is mise's to
/// read and is left out.
fn read_pins(config: &Path) -> Result<Vec<Pin>, PinError> {
    let text = read_config(config)?.unwrap_or_default();
    let doc = parse_config(config, &text)?;
    let Some(tools) = doc.get("tools").and_then(Item::as_table_like) else {
        return Ok(Vec::new());
    };
    let mut pins: Vec<Pin> = tools
        .iter()
        .filter_map(|(tool, item)| {
            let suite = suite_of_tool(tool)?;
            Some(Pin {
                suite: suite.name.to_owned(),
                tool: tool.to_owned(),
                version: item.as_str()?.to_owned(),
                config: config.to_path_buf(),
            })
        })
        .collect();
    pins.sort_by(|a, b| a.tool.cmp(&b.tool));
    Ok(pins)
}

/// The version `doc` pins `tool` at, or `None` when it doesn't name the tool.
///
/// # Errors
///
/// [`PinError::Tangled`] when `tools` isn't a table or the tool's entry isn't a plain version.
fn tool_entry(doc: &DocumentMut, config: &Path, tool: &str) -> Result<Option<String>, PinError> {
    let Some(tools) = doc.get("tools") else {
        return Ok(None);
    };
    let tools = tools
        .as_table_like()
        .ok_or_else(|| tangled(tool, config, Tangle::ToolsNotATable))?;
    let Some(item) = tools.get(tool) else {
        return Ok(None);
    };
    match item.as_str() {
        Some(version) => Ok(Some(version.to_owned())),
        None => Err(tangled(tool, config, Tangle::NotAString)),
    }
}

/// Sets `tool` to `version` in `doc`'s `[tools]`, keeping the entry's comments; the version it
/// pinned before, if any.
fn set_tool(doc: &mut DocumentMut, config: &Path, tool: &str, version: &str) -> Result<Option<String>, PinError> {
    let previous = tool_entry(doc, config, tool)?;
    let tools = tools_of(doc, config, tool)?;
    match tools.get_mut(tool).and_then(Item::as_value_mut) {
        Some(value) => {
            let decor = value.decor().clone();
            *value = version.into();
            *value.decor_mut() = decor;
        }
        None => {
            tools.insert(tool, toml_edit::value(version));
        }
    }
    Ok(previous)
}

/// Removes `tool` from `doc`'s `[tools]`, and the table when that leaves it empty; the version it
/// pinned, or `None` when it named none.
fn remove_tool(doc: &mut DocumentMut, config: &Path, tool: &str) -> Result<Option<String>, PinError> {
    let Some(version) = tool_entry(doc, config, tool)? else {
        return Ok(None);
    };
    let tools = tools_of(doc, config, tool)?;
    tools.remove(tool);
    if tools.is_empty() {
        doc.remove("tools");
    }
    Ok(Some(version))
}

/// `doc`'s `[tools]`, made when missing.
fn tools_of<'a>(doc: &'a mut DocumentMut, config: &Path, tool: &str) -> Result<&'a mut dyn TableLike, PinError> {
    doc.entry("tools")
        .or_insert_with(toml_edit::table)
        .as_table_like_mut()
        .ok_or_else(|| tangled(tool, config, Tangle::ToolsNotATable))
}

fn tangled(tool: &str, config: &Path, why: Tangle) -> PinError {
    PinError::Tangled {
        tool: tool.to_owned(),
        config: config.to_path_buf(),
        why,
    }
}

/// Sets `doc`'s `[env]` to source `script` for `projects`, as a `use bwenv` line did.
/// `false`, with nothing changed, when `[env]` already sources a script or names
/// [`BWENV_VAR`], which the user's own config then decides.
fn set_bwenv(doc: &mut DocumentMut, projects: &[String], script: &str) -> bool {
    if let Some(env) = doc.get("env") {
        let Some(env) = env.as_table_like() else {
            return false;
        };
        let sources = env
            .get("_")
            .is_some_and(|directives| directives.as_table_like().is_none_or(|d| d.contains_key("source")));
        if sources || env.contains_key(BWENV_VAR) {
            return false;
        }
    }
    let env = doc
        .entry("env")
        .or_insert_with(toml_edit::table)
        .as_table_like_mut()
        .expect("`env` was just checked to be a table");
    env.insert(BWENV_VAR, toml_edit::value(projects.join(" ")));
    let mut source = toml_edit::InlineTable::new();
    source.insert("path", script.into());
    source.insert("redact", true.into());
    let directives = env.entry("_").or_insert_with(|| {
        let mut directives = toml_edit::Table::new();
        directives.set_dotted(true);
        Item::Table(directives)
    });
    let Some(directives) = directives.as_table_like_mut() else {
        return false;
    };
    directives.insert("source", toml_edit::value(source));
    true
}

/// The config at `path` from `configs`, read and parsed the first time it is asked for.
fn load_doc<'a>(
    configs: &'a mut BTreeMap<PathBuf, (String, DocumentMut)>,
    path: &Path,
) -> Result<&'a mut (String, DocumentMut), PinError> {
    if !configs.contains_key(path) {
        let text = read_config(path)?.unwrap_or_default();
        let doc = parse_config(path, &text)?;
        configs.insert(path.to_path_buf(), (text, doc));
    }
    Ok(configs.get_mut(path).expect("the config was just loaded"))
}

fn parse_config(path: &Path, text: &str) -> Result<DocumentMut, PinError> {
    text.parse().map_err(|source| PinError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

/// The `use suite` and `use bwenv` lines of `envrc`.
fn read_leftovers(envrc: &Path) -> Result<Vec<Leftover>, PinError> {
    let text = read_config(envrc)?.unwrap_or_default();
    Ok(text
        .lines()
        .enumerate()
        .filter_map(|(at, line)| {
            let kind = match parse(line) {
                Line::Pin { suite, version } => LeftoverKind::Suite { suite, version },
                Line::Bwenv { projects } => LeftoverKind::Bwenv { projects },
                Line::Other => return None,
            };
            Some(Leftover {
                envrc: envrc.to_path_buf(),
                line: at + 1,
                kind,
            })
        })
        .collect())
}

/// The `use suite` and `use bwenv` lines of an `.envrc` that can move, by index: lone, top-level
/// lines, not indented or inside a block, and not a Suite pinned twice (an `if` and its `else`).
fn movable_lines(lines: &[&str]) -> Vec<(usize, Line)> {
    let mut movable = Vec::new();
    let mut seen = BTreeMap::<String, usize>::new();
    let mut depth = 0_i32;
    for (at, line) in lines.iter().enumerate() {
        let parsed = parse(line);
        if let Line::Pin { suite, .. } = &parsed {
            *seen.entry(suite.clone()).or_default() += 1;
        }
        let top_level = depth == 0 && !line.starts_with(char::is_whitespace);
        if top_level && parsed != Line::Other {
            movable.push((at, parsed));
        }
        depth = (depth + nesting(line)).max(0);
    }
    movable.retain(|(_, line)| match line {
        Line::Pin { suite, .. } => seen.get(suite) == Some(&1),
        _ => true,
    });
    movable
}

/// Every config named in [`CONFIG_NAMES`] and every `.envrc` under `dir`, `depth` folders deep at
/// most.
fn find_files(dir: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if kind.is_file() && (name == ENVRC || CONFIG_NAMES.contains(&name.as_ref())) {
            found.push(entry.path());
        } else if kind.is_dir() && depth > 0 && !name.starts_with('.') && !SKIPPED.contains(&name.as_ref()) {
            find_files(&entry.path(), depth - 1, found);
        }
    }
}

/// What one `.envrc` line does, as far as the old pins go. The line is split into words as the
/// shell would split a simple command: on blanks, quotes removed, a word starting with `#` ending
/// the line.
fn parse(line: &str) -> Line {
    let words: Vec<&str> = line
        .split_whitespace()
        .take_while(|word| !word.starts_with('#'))
        .map(unquote)
        .collect();
    match words.as_slice() {
        ["use", "suite", suite, version] if is_version(version) => Line::Pin {
            suite: (*suite).to_owned(),
            version: (*version).to_owned(),
        },
        ["use", "bwenv", projects @ ..] if !projects.is_empty() && projects.iter().all(|p| is_bwenv_project(p)) => {
            Line::Bwenv {
                projects: projects.iter().map(|p| (*p).to_owned()).collect(),
            }
        }
        _ => Line::Other,
    }
}

/// Whether `project` is a plain word a `KUBAOS_BWENV` list can hold.
fn is_bwenv_project(project: &str) -> bool {
    !project.is_empty()
        && project
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'))
}

/// How many shell blocks `line` opens, less how many it closes, counting its words as the
/// shell's keywords would be: `if`, `for`, `while`, `until`, `case`, `select` and `{` open,
/// `fi`, `done`, `esac` and `}` close. A function's `name() {` opens one.
fn nesting(line: &str) -> i32 {
    let code = line.replace(';', " ");
    code.split_whitespace()
        .take_while(|word| !word.starts_with('#'))
        .map(|word| match word {
            "if" | "for" | "while" | "until" | "case" | "select" | "{" => 1,
            "fi" | "done" | "esac" | "}" => -1,
            _ if word.ends_with("(){") => 1,
            _ => 0,
        })
        .sum()
}

/// `word` without the single or double quotes around it.
fn unquote(word: &str) -> &str {
    for quote in ['"', '\''] {
        if let Some(inner) = word.strip_prefix(quote).and_then(|w| w.strip_suffix(quote)) {
            return inner;
        }
    }
    word
}

/// Deletes the GC roots `use suite` made for `suite` in `folder`'s `.direnv/suites/<suite>/`, and
/// the folders it leaves empty. They only kept toolchains from Nix's garbage collector, so one
/// that can't be deleted is left.
fn remove_roots(folder: &Path, suite: &str) {
    if !is_suite_name(suite) {
        return;
    }
    let direnv = folder.join(".direnv");
    let suites = direnv.join("suites");
    let _ = fs::remove_dir_all(suites.join(suite));
    let _ = fs::remove_dir(&suites);
    let _ = fs::remove_dir(&direnv);
}

/// The text of `path`, or `None` when there is none.
fn read_config(path: &Path) -> Result<Option<String>, PinError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(PinError::Read {
            path: path.to_path_buf(),
            source,
        }),
    }
}

/// The folder a migration's backups go under: the time, in seconds since 1970.
fn backup_stamp() -> String {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
        .to_string()
}

/// Copies `file`, when it exists, to `backups/<stamp>/<its absolute path>`, as `kubaos_backup`
/// in `migrations/lib.sh` lays them out.
fn backup(backups: &Path, stamp: &str, file: &Path) -> Result<(), PinError> {
    if !file.exists() {
        return Ok(());
    }
    let relative = file.strip_prefix("/").unwrap_or(file);
    let dest = backups.join(stamp).join(relative);
    let write_err = |source| PinError::Write {
        path: dest.clone(),
        source,
    };
    fs::create_dir_all(dest.parent().unwrap_or(backups)).map_err(write_err)?;
    fs::copy(file, &dest).map(drop).map_err(write_err)
}

/// Writes `text` to `path`, through a temporary file and a rename. A linked file is written where
/// it points, so the link stays, and an existing file keeps its mode.
fn write_config(path: &Path, text: &str) -> Result<(), PinError> {
    let write_err = |source| PinError::Write {
        path: path.to_path_buf(),
        source,
    };
    let target = match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_symlink() => fs::canonicalize(path).map_err(write_err)?,
        _ => path.to_path_buf(),
    };
    let mode = fs::metadata(&target).ok().map(|meta| meta.permissions().mode());
    let mut tmp = target.as_os_str().to_owned();
    tmp.push(format!(".tmp-{}", process::id()));
    let tmp = PathBuf::from(tmp);
    replace_with(&target, &tmp, text, mode).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        write_err(e)
    })
}

/// Writes `text` to `tmp`, with `mode` when given, and renames it over `target`.
fn replace_with(target: &Path, tmp: &Path, text: &str, mode: Option<u32>) -> io::Result<()> {
    let mut options = File::options();
    options.write(true).create(true).truncate(true);
    if let Some(mode) = mode {
        options.mode(mode);
    }
    let mut file = options.open(tmp)?;
    // HAZARD: set the mode before the text goes in. A private config would otherwise be readable
    // by others, with whatever secrets it holds, until the rename.
    if let Some(mode) = mode {
        file.set_permissions(fs::Permissions::from_mode(mode))?;
    }
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    fs::rename(tmp, target)
}

#[cfg(test)]
mod tests;
