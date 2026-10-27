//! Project pins: a folder's choice of a Suite's version, for that folder and everything below it.
//!
//! A pin is a line in the folder's `.envrc`, `use suite <name> <version>`, which the direnv
//! function KubaOS ships (`config/direnv/lib/suite.sh`) turns into that version's toolchain on
//! PATH. direnv loads only the nearest `.envrc` above the current folder, and a parent's only
//! where that file says `source_up`, so a pin covers the folders below it up to the next `.envrc`
//! that doesn't source it. Within what direnv loads, the last pin of a Suite wins.
//!
//! Every version a pin can name has a toolchain entry in the home (`home/pins.nix`): the
//! derivation, and the output its `bin/` is in. A version nobody uses costs nothing but the
//! derivation; the direnv function realises it, from the binary cache or by building it, on first
//! use.
//!
//! # Owns
//!
//! - Reading `.envrc` files the way direnv runs them, to the pins in effect in a folder
//!   ([`Pins::resolve`]), and listing the pins under the Projects root ([`Pins::all`]).
//! - What a pin stands for on this machine: the installed Suite's version, and whether the
//!   toolchain is in the store ([`Pins::status`]).
//! - Writing and removing pin lines, and allowing the result in direnv ([`Pins::pin`],
//!   [`Pins::unpin`]).
//!
//! # Not here
//!
//! - The Suites and their installed versions: `suites.rs`.
//! - The direnv function and the toolchain entries: `config/direnv/lib/suite.sh`, `home/pins.nix`.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::Command;
use std::process::Stdio;

use crate::state::Paths;
use crate::suites::Suite;
use crate::suites::SuiteConfig;
use crate::suites::SuiteError;
use crate::suites::Suites;
use crate::suites::catalog;
use crate::suites::is_suite_name;

/// The file direnv loads, and the one a pin is written to.
pub const ENVRC: &str = ".envrc";

/// How deep below the Projects root [`Pins::all`] looks: a pin goes on a group of projects or on
/// one, rarely deeper.
pub const SCAN_DEPTH: usize = 4;

/// Folders [`Pins::all`] never enters, besides hidden ones: dependencies and build output, which
/// hold no pins of the user's and can be huge.
const SKIPPED: &[&str] = &["node_modules", "target", "vendor", "dist", "build"];

/// One pin line: `use suite <suite> <version>` in `envrc`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pin {
    pub suite: String,
    pub version: String,
    /// The `.envrc` the line is in.
    pub envrc: PathBuf,
}

/// A pin in an `.envrc` above the folder that direnv never loads there, because a nearer `.envrc`
/// doesn't `source_up`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hidden {
    pub pin: Pin,
    /// The highest `.envrc` direnv loads for the folder, which would need `source_up`.
    pub by: PathBuf,
}

/// The pins in effect in a folder.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Resolution {
    /// The `.envrc` direnv loads for the folder, if any.
    pub nearest: Option<PathBuf>,
    /// One pin per Suite, the one that wins, by Suite name.
    pub pins: Vec<Pin>,
    /// Pins above that don't reach the folder, for Suites it has no pin of.
    pub hidden: Vec<Hidden>,
}

/// Where a pinned version's toolchain is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fetch {
    /// In the Nix store, at `path`.
    InStore { path: PathBuf },
    /// Not in the store yet: the direnv function fetches it the first time it loads the pin.
    OnFirstUse,
    /// The home has no toolchain for this Suite and version: the Suite has no such version, or
    /// the home predates it and needs a switch. The direnv function says so and changes nothing.
    Missing,
}

/// What a pin stands for on this machine.
#[derive(Clone, Debug)]
pub struct PinStatus {
    pub pin: Pin,
    /// The Suite, when this build of KubaOS knows it.
    pub suite: Option<&'static Suite>,
    /// The Suite's options, when it is installed.
    pub installed: Option<SuiteConfig>,
    pub toolchain: Fetch,
}

impl PinStatus {
    /// The version installed for the whole home, if the Suite is installed with one.
    pub fn global_version(&self) -> Option<&str> {
        self.installed.as_ref()?.version.as_deref()
    }

    /// Whether the pin names the version the whole home has anyway.
    pub fn is_global(&self) -> bool {
        self.global_version() == Some(self.pin.version.as_str())
    }
}

/// What [`Pins::pin`] did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pinned {
    pub envrc: PathBuf,
    /// Whether the `.envrc` is new.
    pub created: bool,
    /// The version the folder's `.envrc` pinned before, if any.
    pub previous: Option<String>,
}

/// What [`Pins::unpin`] did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unpinned {
    pub envrc: PathBuf,
    /// The version that was pinned.
    pub version: String,
    /// Whether the `.envrc` held nothing else, and so is deleted.
    pub deleted: bool,
}

/// Reading, writing or allowing a pin failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PinError {
    #[error("{} is not a folder", path.display())]
    NotAFolder { path: PathBuf },
    #[error("the {suite} Suite has no versions to pin")]
    Unversioned { suite: String },
    #[error("the {suite} Suite has no version {version:?} (it has: {})", known.join(", "))]
    UnknownVersion {
        suite: String,
        version: String,
        known: Vec<String>,
    },
    /// The folder's own `.envrc` has no pin of the Suite. `applies_from` is the `.envrc` whose
    /// pin of it is in effect there, if one is.
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
    #[error("writing {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The `.envrc` is written, but `direnv` could not be started to allow it.
    #[error("running `direnv allow {}`", path.display())]
    Allow {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The `.envrc` is written, but `direnv allow` failed, or was killed (no code).
    #[error("`direnv allow {}` exited with {}", path.display(), code.map_or_else(|| "a signal".to_owned(), |c| format!("code {c}")))]
    AllowFailed { path: PathBuf, code: Option<i32> },
    /// The installed Suites could not be read.
    #[error("reading the installed Suites")]
    Suites(#[source] SuiteError),
    /// The `.envrc`'s pin of the Suite is not a lone, top-level line, so changing it could break
    /// the file's shell. Nothing is written; `line` counts from 1.
    #[error("{}:{line} pins the {suite} Suite in a way kubaos leaves to you: {why}; edit it by hand", envrc.display())]
    Tangled {
        suite: String,
        envrc: PathBuf,
        line: usize,
        why: Tangle,
    },
}

/// Why a pin line is left to the user.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Tangle {
    /// The file pins the Suite more than once, as an `if` and its `else` do.
    #[error("the file pins it more than once")]
    Repeated,
    #[error("the line is indented")]
    Indented,
    /// The line is between an `if`, `for`, `while`, `until`, `case`, `select` or `{` and its end.
    #[error("the line is inside a block")]
    InBlock,
}

/// One line of an `.envrc`, as far as pins go.
#[derive(Debug, PartialEq, Eq)]
enum Line {
    Pin { suite: String, version: String },
    SourceUp,
    Other,
}

/// The Project pins of one user: the `.envrc` files they live in, the toolchain entries of the
/// home, and the installed Suites they override.
#[derive(Clone, Debug)]
pub struct Pins {
    suites: Suites,
    toolchains: PathBuf,
    projects: PathBuf,
    /// The `direnv` command: the program, then any arguments before `allow`.
    direnv: Vec<OsString>,
}

impl Pins {
    /// The Project pins of the user `paths` describes, allowed with the `direnv` on PATH.
    #[must_use]
    pub fn new(paths: &Paths) -> Self {
        Self {
            suites: Suites::new(paths),
            toolchains: paths.toolchains(),
            projects: paths.projects_root(),
            direnv: vec![OsString::from("direnv")],
        }
    }

    /// The same, allowing `.envrc` files with `command` (a program and its first arguments)
    /// instead of `direnv`.
    #[must_use]
    pub fn with_direnv(mut self, command: impl IntoIterator<Item = impl Into<OsString>>) -> Self {
        self.direnv = command.into_iter().map(Into::into).collect();
        self
    }

    /// The pins in effect in `folder`, as direnv loads them there.
    ///
    /// # Errors
    ///
    /// [`PinError::NotAFolder`], or [`PinError::Read`] when an `.envrc` direnv would load cannot
    /// be read.
    pub fn resolve(&self, folder: &Path) -> Result<Resolution, PinError> {
        if !folder.is_dir() {
            return Err(PinError::NotAFolder {
                path: folder.to_path_buf(),
            });
        }
        let Some(nearest) = nearest_envrc(folder) else {
            return Ok(Resolution::default());
        };
        let mut effective = BTreeMap::new();
        let mut loaded = Vec::new();
        load(&nearest, &mut effective, &mut loaded)?;

        let highest = loaded.iter().min_by_key(|path| path.components().count()).cloned();
        let mut hidden = Vec::new();
        if let Some(highest) = highest {
            let above = highest.parent().and_then(Path::parent);
            for envrc in above.into_iter().flat_map(envrcs_from) {
                for pin in read_pins(&envrc)? {
                    let shadowed =
                        effective.contains_key(&pin.suite) || hidden.iter().any(|h: &Hidden| h.pin.suite == pin.suite);
                    if !shadowed {
                        hidden.push(Hidden {
                            pin,
                            by: highest.clone(),
                        });
                    }
                }
            }
        }
        Ok(Resolution {
            nearest: Some(nearest),
            pins: effective.into_values().collect(),
            hidden,
        })
    }

    /// What `pin` stands for here: its Suite, the installed version, and where its toolchain is.
    ///
    /// # Errors
    ///
    /// [`PinError::Suites`] when the Suite's file cannot be read.
    pub fn status(&self, pin: Pin) -> Result<PinStatus, PinError> {
        let suite = catalog::find(&pin.suite);
        let installed = match suite {
            Some(suite) => self.suites.config(suite).map_err(PinError::Suites)?,
            None => None,
        };
        let toolchain = self.fetch(&pin.suite, &pin.version);
        Ok(PinStatus {
            pin,
            suite,
            installed,
            toolchain,
        })
    }

    /// Pins `suite` at `version` in `folder`: replaces the folder's `.envrc` line for the Suite,
    /// or adds one at the end, creating the file when missing, then allows it in direnv. The
    /// file's other lines stay as they are. A new file under another `.envrc` starts with
    /// `source_up`, so the folder keeps what that one sets. Only a lone, top-level pin line is
    /// replaced: one the file repeats, indents or puts in a block is left to the user.
    ///
    /// # Errors
    ///
    /// [`PinError::NotAFolder`], a version the Suite doesn't have, [`PinError::Tangled`], a
    /// failed read or write; or, with the file written, a failed `direnv allow`.
    pub fn pin(&self, suite: &Suite, version: &str, folder: &Path) -> Result<Pinned, PinError> {
        if suite.versions.is_empty() {
            return Err(PinError::Unversioned {
                suite: suite.name.to_owned(),
            });
        }
        if !suite.versions.contains(&version) {
            return Err(PinError::UnknownVersion {
                suite: suite.name.to_owned(),
                version: version.to_owned(),
                known: suite.versions.iter().map(|v| (*v).to_owned()).collect(),
            });
        }
        if !folder.is_dir() {
            return Err(PinError::NotAFolder {
                path: folder.to_path_buf(),
            });
        }
        let envrc = folder.join(ENVRC);
        let current = read_envrc(&envrc)?;
        let created = current.is_none();
        let text = current.unwrap_or_default();
        let line = format!("use suite {} {version}", suite.name);

        let found = pin_line(&envrc, &text, suite.name)?;
        let mut out: Vec<String> = text.lines().map(str::to_owned).collect();
        let previous = found.map(|(at, old)| {
            out[at] = replace_pin(&out[at], &line);
            old
        });
        let above = folder.parent().and_then(nearest_envrc);
        if created && above.is_some() {
            out.push("source_up".to_owned());
        }
        if previous.is_none() {
            out.push(line);
        }
        if created || out.join("\n") != text.trim_end_matches('\n') {
            write_envrc(&envrc, &out)?;
        }
        self.allow(&envrc)?;
        Ok(Pinned {
            envrc,
            created,
            previous,
        })
    }

    /// Removes `folder`'s pin of the Suite called `suite`: its line in the folder's `.envrc`,
    /// and the GC roots the direnv function made for it in the folder's `.direnv/`. A file left
    /// with nothing but blank lines and a bare `source_up` is deleted; otherwise it is allowed
    /// again.
    ///
    /// # Errors
    ///
    /// [`PinError::NotPinned`] when the folder's own `.envrc` has no such line,
    /// [`PinError::Tangled`], a failed read or write, or, with the file written, a failed
    /// `direnv allow`.
    pub fn unpin(&self, suite: &str, folder: &Path) -> Result<Unpinned, PinError> {
        if !folder.is_dir() {
            return Err(PinError::NotAFolder {
                path: folder.to_path_buf(),
            });
        }
        let envrc = folder.join(ENVRC);
        let text = read_envrc(&envrc)?.unwrap_or_default();
        let found = pin_line(&envrc, &text, suite)?;
        let Some((at, version)) = found else {
            let applies_from = self
                .resolve(folder)?
                .pins
                .into_iter()
                .find(|pin| pin.suite == suite)
                .map(|pin| pin.envrc);
            return Err(PinError::NotPinned {
                suite: suite.to_owned(),
                folder: folder.to_path_buf(),
                applies_from,
            });
        };
        let out: Vec<String> = text
            .lines()
            .enumerate()
            .filter(|&(i, _)| i != at)
            .map(|(_, line)| line.to_owned())
            .collect();
        // A file of a bare `source_up` alone loads what the folder would load without it.
        let deleted = out
            .iter()
            .all(|line| matches!(line.trim(), "" | "source_up" | "source_up_if_exists"));
        Self::remove_roots(folder, suite);
        if deleted {
            fs::remove_file(&envrc).map_err(|source| PinError::Write {
                path: envrc.clone(),
                source,
            })?;
        } else {
            write_envrc(&envrc, &out)?;
            self.allow(&envrc)?;
        }
        Ok(Unpinned {
            envrc,
            version,
            deleted,
        })
    }

    /// Every pin line in an `.envrc` under the Projects root, [`SCAN_DEPTH`] folders deep at
    /// most, by path. Hidden folders, dependency and build folders, and links are not entered.
    ///
    /// # Errors
    ///
    /// [`PinError::Read`] when an `.envrc` cannot be read. A folder that cannot be listed is
    /// skipped.
    pub fn all(&self) -> Result<Vec<Pin>, PinError> {
        let mut envrcs = Vec::new();
        find_envrcs(&self.projects, SCAN_DEPTH, &mut envrcs);
        envrcs.sort();
        let mut pins = Vec::new();
        for envrc in envrcs {
            pins.extend(read_pins(&envrc)?);
        }
        Ok(pins)
    }

    /// Deletes the GC roots of `suite` in `folder`'s `.direnv/suites/<suite>/`, which
    /// `config/direnv/lib/suite.sh` makes, and the folders it leaves empty. The roots only keep
    /// toolchains from the garbage collector, so one that can't be deleted is left.
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

    /// Where the toolchain of `suite` at `version` is, from the home's toolchain entry: the
    /// derivation on the first line, the output on the second.
    fn fetch(&self, suite: &str, version: &str) -> Fetch {
        if !is_suite_name(suite) || !is_version(version) {
            return Fetch::Missing;
        }
        let Ok(entry) = fs::read_to_string(self.toolchains.join(suite).join(version)) else {
            return Fetch::Missing;
        };
        let Some(out) = entry.lines().nth(1).filter(|out| !out.is_empty()) else {
            return Fetch::Missing;
        };
        let path = PathBuf::from(out);
        if fs::symlink_metadata(&path).is_ok() {
            Fetch::InStore { path }
        } else {
            Fetch::OnFirstUse
        }
    }

    /// Runs `direnv allow envrc`, so the next prompt in the folder loads the file without asking.
    fn allow(&self, envrc: &Path) -> Result<(), PinError> {
        let Some((program, args)) = self.direnv.split_first() else {
            return Ok(());
        };
        let status = Command::new(program)
            .args(args)
            .arg("allow")
            .arg(envrc)
            .stdin(Stdio::null())
            .status()
            .map_err(|source| PinError::Allow {
                path: envrc.to_path_buf(),
                source,
            })?;
        if !status.success() {
            return Err(PinError::AllowFailed {
                path: envrc.to_path_buf(),
                code: status.code(),
            });
        }
        Ok(())
    }
}

/// Whether `version` is one plain word a pin line can hold: letters, digits, `.`, `_` and `-`.
fn is_version(version: &str) -> bool {
    !version.is_empty()
        && !version.starts_with('.')
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// Adds `envrc`'s pins to `effective`, in the order direnv runs its lines, and the files it loads
/// to `loaded`. `source_up` loads the nearest `.envrc` above the file's folder at that point.
fn load(envrc: &Path, effective: &mut BTreeMap<String, Pin>, loaded: &mut Vec<PathBuf>) -> Result<(), PinError> {
    loaded.push(envrc.to_path_buf());
    let text = read_envrc(envrc)?.unwrap_or_default();
    for line in text.lines() {
        match parse(line) {
            Line::Pin { suite, version } => {
                let pin = Pin {
                    suite: suite.clone(),
                    version,
                    envrc: envrc.to_path_buf(),
                };
                effective.insert(suite, pin);
            }
            Line::SourceUp => {
                let above = envrc.parent().and_then(Path::parent).and_then(nearest_envrc);
                if let Some(above) = above {
                    load(&above, effective, loaded)?;
                }
            }
            Line::Other => {}
        }
    }
    Ok(())
}

/// The `.envrc` in `folder` or the nearest folder above it.
fn nearest_envrc(folder: &Path) -> Option<PathBuf> {
    envrcs_from(folder).next()
}

/// Every `.envrc` in `folder` and the folders above it, nearest first.
fn envrcs_from(folder: &Path) -> impl Iterator<Item = PathBuf> + '_ {
    folder
        .ancestors()
        .map(|dir| dir.join(ENVRC))
        .filter(|envrc| envrc.is_file())
}

/// The pin lines of `envrc`, in order.
fn read_pins(envrc: &Path) -> Result<Vec<Pin>, PinError> {
    let text = read_envrc(envrc)?.unwrap_or_default();
    Ok(text
        .lines()
        .filter_map(|line| match parse(line) {
            Line::Pin { suite, version } => Some(Pin {
                suite,
                version,
                envrc: envrc.to_path_buf(),
            }),
            _ => None,
        })
        .collect())
}

/// The `.envrc` files under `dir`, `depth` folders deep at most.
fn find_envrcs(dir: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if kind.is_file() && name == ENVRC {
            found.push(entry.path());
        } else if kind.is_dir() && depth > 0 && !name.starts_with('.') && !SKIPPED.contains(&name.as_ref()) {
            find_envrcs(&entry.path(), depth - 1, found);
        }
    }
}

/// What one `.envrc` line does for pins. The line is split into words as the shell would split a
/// simple command: on blanks, quotes removed, a word starting with `#` ending the line.
fn parse(line: &str) -> Line {
    let words: Vec<&str> = line
        .split_whitespace()
        .take_while(|word| !word.starts_with('#'))
        .map(unquote)
        .collect();
    match words.as_slice() {
        ["use", "suite", suite, version] => Line::Pin {
            suite: (*suite).to_owned(),
            version: (*version).to_owned(),
        },
        ["source_up" | "source_up_if_exists"]
        | ["source_up" | "source_up_if_exists", ".envrc" | "&&" | "||" | ";", ..] => Line::SourceUp,
        _ => Line::Other,
    }
}

/// The one line of `text` that pins `suite`, by index, with its version; `None` when there is
/// none.
///
/// # Errors
///
/// [`PinError::Tangled`] when the file pins the Suite more than once, or the line is indented
/// or inside a block, where replacing or removing it could break the shell around it.
fn pin_line(envrc: &Path, text: &str, suite: &str) -> Result<Option<(usize, String)>, PinError> {
    let tangled = |at: usize, why| PinError::Tangled {
        suite: suite.to_owned(),
        envrc: envrc.to_path_buf(),
        line: at + 1,
        why,
    };
    let mut found = None;
    let mut depth = 0_i32;
    for (at, line) in text.lines().enumerate() {
        if let Line::Pin { suite: name, version } = parse(line)
            && name == suite
        {
            if found.is_some() {
                return Err(tangled(at, Tangle::Repeated));
            }
            found = Some((at, version, line.starts_with(char::is_whitespace), depth > 0));
        }
        depth = (depth + nesting(line)).max(0);
    }
    match found {
        None => Ok(None),
        Some((at, _, true, _)) => Err(tangled(at, Tangle::Indented)),
        Some((at, _, _, true)) => Err(tangled(at, Tangle::InBlock)),
        Some((at, version, false, false)) => Ok(Some((at, version))),
    }
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

/// `original`, a pin line, as `line` with the original's indentation and trailing comment.
fn replace_pin(original: &str, line: &str) -> String {
    let indent = &original[..original.len() - original.trim_start().len()];
    let comment = original
        .char_indices()
        .find(|&(at, c)| c == '#' && original[..at].ends_with(char::is_whitespace))
        .map_or("", |(at, _)| &original[at..]);
    if comment.is_empty() {
        format!("{indent}{line}")
    } else {
        format!("{indent}{line} {comment}")
    }
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

/// The text of `envrc`, or `None` when there is none.
fn read_envrc(envrc: &Path) -> Result<Option<String>, PinError> {
    match fs::read_to_string(envrc) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(PinError::Read {
            path: envrc.to_path_buf(),
            source,
        }),
    }
}

/// Writes `lines` to `envrc`, through a temporary file and a rename. A linked `.envrc` is written
/// where it points, so the link stays, and an existing file keeps its mode.
fn write_envrc(envrc: &Path, lines: &[String]) -> Result<(), PinError> {
    let write_err = |source| PinError::Write {
        path: envrc.to_path_buf(),
        source,
    };
    let target = match fs::symlink_metadata(envrc) {
        Ok(meta) if meta.is_symlink() => fs::canonicalize(envrc).map_err(write_err)?,
        _ => envrc.to_path_buf(),
    };
    let mode = fs::metadata(&target).ok().map(|meta| meta.permissions().mode());
    let mut text = lines.join("\n");
    text.push('\n');
    let mut tmp = target.as_os_str().to_owned();
    tmp.push(format!(".tmp-{}", process::id()));
    let tmp = PathBuf::from(tmp);
    replace_with(&target, &tmp, &text, mode).map_err(|e| {
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
    // HAZARD: set the mode before the text goes in. A private `.envrc` would otherwise be
    // readable by others, with its secrets, until the rename.
    if let Some(mode) = mode {
        file.set_permissions(fs::Permissions::from_mode(mode))?;
    }
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    fs::rename(tmp, target)
}

#[cfg(test)]
mod tests;
