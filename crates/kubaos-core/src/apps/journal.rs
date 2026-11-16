//! An app's journal: what its install did on this machine, step by step, with what each step
//! found there before, so a failure rolls back and a removal undoes exactly that.
//!
//! An app has one journal a side, `<dir>/<id>/journal.toml`: the user's in
//! `~/.local/state/kubaos/apps/`, root's in `/var/lib/kubaos/apps/`. Each is rewritten whole and
//! renamed into place, so a run killed at any point leaves the old journal or the new one. The
//! claims of every journal are who owns what: a path, a settings key, a package.
//!
//! # Owns
//!
//! - [`Journal`], [`Entry`], [`Claim`] and [`Prior`], reading and writing them, [`Owners`] (the
//!   claims of every journal) and the lock one run takes.
//!
//! # Not here
//!
//! - What a step claims and how it is undone: `action/perform.rs`. The order of a run: `engine.rs`.

use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::fs::TryLockError;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use super::action::Action;
use super::manifest::is_app_id;

/// The journal's file in an app's folder of a journals' folder.
pub const JOURNAL: &str = "journal.toml";

/// Where a user's journals are, under the home.
pub const USER_JOURNALS: &str = ".local/state/kubaos/apps";

/// The lock the user's side of a run holds, under the home.
pub const USER_LOCK: &str = ".local/state/kubaos/apps/.lock";

/// Where root's journals are.
pub const ROOT_JOURNALS: &str = "/var/lib/kubaos/apps";

/// How often [`lock`] tries before it says another run holds the lock, and how long it waits
/// between tries.
const LOCK_TRIES: u32 = 10;
const LOCK_WAIT: std::time::Duration = std::time::Duration::from_millis(20);

/// A journal that couldn't be read, written or locked.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum JournalError {
    #[error("reading {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{} is not a journal", path.display())]
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
    #[error("another run is changing this machine's apps; try again once it ends")]
    Busy,
}

/// One app's journal on one side.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Journal {
    pub app: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setup: Option<String>,
    /// Every step of the install is done. The user's journal says it for both sides.
    #[serde(default)]
    pub complete: bool,
    #[serde(default, rename = "entry")]
    pub entries: Vec<Entry>,
}

/// One step, as it was done.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    /// Its place in the install's list of actions, from 1.
    pub step: usize,
    pub state: State,
    /// What each thing it changes or found is, and whether it changed it.
    #[serde(default)]
    pub claims: Vec<Claim>,
    /// What the machine had before, which its undo puts back.
    #[serde(default)]
    pub prior: Prior,
    /// The action as it was done, its words filled in.
    pub action: Action,
    /// The step as it was before a release changed it, with the claims of it the changed action
    /// doesn't keep: undone once the changed action is done, so an update in place takes away
    /// what only the old one had (a package dropped from the list, the old version's download).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaced: Option<Box<Entry>>,
}

/// How far a step got.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    /// Started: the run stopped during it, or is in it.
    Doing,
    /// Done by this app.
    Done,
    /// Done already when the step came: nothing of it is the app's.
    Found,
    /// A step that opens a window, in a run that may not open one: its sentence was a note.
    Skipped,
    /// Being undone: the run stopped during its undo, or is in it.
    Undoing,
}

impl State {
    /// Whether the journal says the step needs nothing more, in a run that may open a window or
    /// not. A run checks a settled step all the same.
    pub fn settled(self, windows: bool) -> bool {
        match self {
            Self::Done | Self::Found => true,
            Self::Skipped => !windows,
            Self::Doing | Self::Undoing => false,
        }
    }

    /// Whether the app changed what the step claims: its prior state is what was there before
    /// the app came, kept as long as the entry is.
    pub fn ours(self) -> bool {
        matches!(self, Self::Doing | Self::Done | Self::Undoing)
    }
}

/// One thing a step changed or found: `path:/…`, `dir:/…` (a folder it made for a path),
/// `key:<file>#<key>`, `package:<name>`,
/// `nix:<name>`, `member:<group>:<user>`, `group:<name>`, `unit:<scope>:<name>`, `linger:<user>`
/// or `run`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    pub claim: String,
    /// The step changed it, so its undo takes it back; else it was there.
    pub owned: bool,
}

impl Claim {
    pub fn new(claim: impl Into<String>, owned: bool) -> Self {
        Self {
            claim: claim.into(),
            owned,
        }
    }

    /// The path of a `path:` claim.
    pub fn path(&self) -> Option<&Path> {
        self.claim.strip_prefix("path:").map(Path::new)
    }
}

/// What the machine had before a step, as much as its undo needs.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prior {
    /// A setting's old value, in its file's own format (a TOML or JSON value); none when the key
    /// wasn't there.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// A setting's file as it was, which its undo puts back whole while the file is as the step
    /// left it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Whether a setting's file was there.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<bool>,
    /// How many of the tables above a setting's key were there: the undo takes away the deeper
    /// ones the step made, once they are empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tables: Option<usize>,
    /// Whether a unit was enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Whether a unit was running.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Whether the user lingered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lingered: Option<bool>,
}

impl Prior {
    /// Takes what `other` recorded in place of this one's: the state before the app that hands
    /// a claim over is the state the claim goes back to.
    pub fn take_from(&mut self, other: &Self) {
        let Self {
            value,
            text,
            file,
            tables,
            enabled,
            active,
            lingered,
        } = other.clone();
        if file.is_some() {
            self.value = value;
            self.text = text;
            self.file = file;
            self.tables = tables;
        }
        self.enabled = enabled.or(self.enabled);
        self.active = active.or(self.active);
        self.lingered = lingered.or(self.lingered);
    }
}

impl Journal {
    /// A new journal of `app` in `setup`.
    pub fn new(app: &str, setup: Option<&str>) -> Self {
        Self {
            app: app.to_owned(),
            setup: setup.map(str::to_owned),
            complete: false,
            entries: Vec::new(),
        }
    }

    /// The file of the app `id`'s journal in `dir`.
    pub fn file(dir: &Path, id: &str) -> PathBuf {
        dir.join(id).join(JOURNAL)
    }

    /// The app `id`'s journal in `dir`; none when there is none.
    ///
    /// # Errors
    ///
    /// [`JournalError::Read`] or [`JournalError::Parse`] when it is there and can't be read.
    pub fn read(dir: &Path, id: &str) -> Result<Option<Self>, JournalError> {
        let path = Self::file(dir, id);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(source) => return Err(JournalError::Read { path, source }),
        };
        toml::from_str(&text)
            .map(Some)
            .map_err(|source| JournalError::Parse { path, source })
    }

    /// Writes it whole, through a file renamed into place; with no entries and not complete, takes
    /// it and its folder away instead.
    ///
    /// # Errors
    ///
    /// [`JournalError::Write`] when the file or its folder can't be written.
    pub fn save(&self, dir: &Path) -> Result<(), JournalError> {
        let path = Self::file(dir, &self.app);
        let write = |source| JournalError::Write {
            path: path.clone(),
            source,
        };
        if self.entries.is_empty() && !self.complete {
            return Self::forget(dir, &self.app);
        }
        let folder = dir.join(&self.app);
        fs::create_dir_all(&folder).map_err(write)?;
        let text = toml::to_string(self).map_err(|e| write(io::Error::other(e)))?;
        let tmp = folder.join(format!(".{JOURNAL}.new"));
        write_new(&tmp, text.as_bytes()).map_err(write)?;
        fs::rename(&tmp, &path).map_err(write)
    }

    /// Takes the app `id`'s journal in `dir` away, and its folder.
    ///
    /// # Errors
    ///
    /// [`JournalError::Write`] when it can't be removed.
    pub fn forget(dir: &Path, id: &str) -> Result<(), JournalError> {
        let folder = dir.join(id);
        match fs::remove_dir_all(&folder) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(JournalError::Write {
                path: folder,
                source: e,
            }),
            _ => Ok(()),
        }
    }

    /// The entry of `step`.
    pub fn entry(&self, step: usize) -> Option<&Entry> {
        self.entries.iter().find(|e| e.step == step)
    }

    /// Puts `entry` in, in place of the one of its step.
    pub fn put(&mut self, entry: Entry) {
        self.entries.retain(|e| e.step != entry.step);
        self.entries.push(entry);
        self.entries.sort_by_key(|e| e.step);
    }

    /// Takes out the entry of `step`.
    pub fn drop_step(&mut self, step: usize) {
        self.entries.retain(|e| e.step != step);
    }
}

/// Every claim of every journal in some folders, by app: who owns a path, and who else holds a
/// thing an app is about to take back.
#[derive(Clone, Debug, Default)]
pub struct Owners {
    /// App id to its claims.
    claims: BTreeMap<String, Vec<Claim>>,
}

impl Owners {
    /// The claims of every journal in `dirs`. A folder that isn't there has none.
    ///
    /// # Errors
    ///
    /// [`JournalError`] when a folder or a journal in it can't be read.
    pub fn load(dirs: &[&Path]) -> Result<Self, JournalError> {
        let mut claims: BTreeMap<String, Vec<Claim>> = BTreeMap::new();
        for dir in dirs {
            let entries = match fs::read_dir(dir) {
                Ok(entries) => entries,
                Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
                Err(source) => {
                    return Err(JournalError::Read {
                        path: dir.to_path_buf(),
                        source,
                    });
                }
            };
            for entry in entries {
                let entry = entry.map_err(|source| JournalError::Read {
                    path: dir.to_path_buf(),
                    source,
                })?;
                let Some(id) = entry.file_name().to_str().filter(|id| is_app_id(id)).map(str::to_owned) else {
                    continue;
                };
                if let Some(journal) = Journal::read(dir, &id)? {
                    claims
                        .entry(id)
                        .or_default()
                        .extend(journal.entries.into_iter().flat_map(|e| e.claims));
                }
            }
        }
        Ok(Self { claims })
    }

    /// The app other than `app` that owns `path`, a folder above it or a path in it.
    pub fn path_owner<'a>(&'a self, path: &Path, app: &'a str) -> Option<&'a str> {
        self.others(app)
            .find(|(_, claim)| {
                claim.owned
                    && claim
                        .path()
                        .is_some_and(|owned| owned.starts_with(path) || path.starts_with(owned))
            })
            .map(|(id, _)| id)
    }

    /// The app other than `app` that owns `claim`.
    pub fn owner<'a>(&'a self, claim: &str, app: &'a str) -> Option<&'a str> {
        self.others(app)
            .find(|(_, c)| c.owned && c.claim == claim)
            .map(|(id, _)| id)
    }

    /// The app other than `app` that holds `claim` without owning it: the one it passes to when
    /// `app` goes.
    pub fn holder<'a>(&'a self, claim: &str, app: &'a str) -> Option<&'a str> {
        self.others(app)
            .find(|(_, c)| !c.owned && c.claim == claim)
            .map(|(id, _)| id)
    }

    /// Whether `app`'s own claims cover `path`: the path or a folder above it.
    pub fn covers(&self, app: &str, path: &Path) -> bool {
        self.claims.get(app).is_some_and(|claims| {
            claims
                .iter()
                .any(|c| c.owned && c.path().is_some_and(|p| path.starts_with(p)))
        })
    }

    /// Adds claims to `app`'s, as a run makes them.
    pub fn add(&mut self, app: &str, claims: &[Claim]) {
        self.claims.entry(app.to_owned()).or_default().extend_from_slice(claims);
    }

    fn others<'a>(&'a self, app: &'a str) -> impl Iterator<Item = (&'a str, &'a Claim)> {
        self.claims
            .iter()
            .filter(move |(id, _)| id.as_str() != app)
            .flat_map(|(id, claims)| claims.iter().map(move |c| (id.as_str(), c)))
    }
}

/// Writes `bytes` to the new file `path`: whatever was at `path` (a link someone left there
/// included) is taken away first, and the file is created only if nothing took its place.
///
/// # Errors
///
/// When it can't be removed or written.
pub fn write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
        _ => {}
    }
    let mut file = File::options().write(true).create_new(true).open(path)?;
    io::Write::write_all(&mut file, bytes)
}

/// Hands `claim` from the app going over to `holder`, which held it without owning it, in
/// `holder`'s journal in `dir`, with `prior`, what the going app found before it changed it:
/// `holder` now owns it, and takes it back to that when it goes.
///
/// # Errors
///
/// [`JournalError`] when that journal can't be read or written.
pub fn hand_over(dir: &Path, holder: &str, claim: &str, prior: &Prior) -> Result<bool, JournalError> {
    let Some(mut journal) = Journal::read(dir, holder)? else {
        return Ok(false);
    };
    let mut found = false;
    for entry in &mut journal.entries {
        if let Some(c) = entry.claims.iter_mut().find(|c| c.claim == claim && !c.owned) {
            c.owned = true;
            entry.prior.take_from(prior);
            found = true;
            break;
        }
    }
    if found {
        journal.save(dir)?;
    }
    Ok(found)
}

/// Takes the lock at `path`, which one run holds while it changes the machine's apps.
///
/// # Errors
///
/// [`JournalError::Busy`] when another run holds it, [`JournalError::Write`] when the file can't
/// be opened.
pub fn lock(path: &Path) -> Result<File, JournalError> {
    let write = |source| JournalError::Write {
        path: path.to_path_buf(),
        source,
    };
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(write)?;
    }
    let file = File::options()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path)
        .map_err(write)?;
    // A child forked by this process keeps a copy of a just-dropped lock's descriptor until it
    // execs, and the lock lives as long as any copy does. A short wait outlasts that.
    for _ in 1..LOCK_TRIES {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(TryLockError::WouldBlock) => std::thread::sleep(LOCK_WAIT),
            Err(TryLockError::Error(e)) => return Err(write(e)),
        }
    }
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => Err(JournalError::Busy),
        Err(TryLockError::Error(e)) => Err(write(e)),
    }
}
