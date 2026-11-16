//! Installing and removing an app of folders: its steps in order, each journaled before it runs,
//! a failure rolled back, a removal undone from the journal, a stopped run resumed.
//!
//! A run has two sides. The user's [`Keeper`] runs in this process; root's runs in the root
//! helper (`kubaos-apps serve`), reached through a [`RootSide`], started once before the first
//! step so a declined password changes nothing. Both sides run the same [`Keeper`], each for its
//! own steps and its own journal.
//!
//! # Owns
//!
//! - [`Keeper`] (one side's steps, undos and who-owns-what checks), [`Engine`] (the order of an
//!   install and a removal, rollback, events), and their errors.
//!
//! # Not here
//!
//! - The actions: `action.rs`. The journal's format: `journal.rs`. The root helper's requests:
//!   `root.rs`.

use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use super::action::Action;
use super::action::Kind;
use super::action::Side;
use super::action::Words;
use super::action::perform;
use super::action::perform::PerformError;
use super::folder::Folder;
use super::folder::FolderError;
use super::host::Host;
use super::journal;
use super::journal::Claim;
use super::journal::Entry;
use super::journal::Journal;
use super::journal::JournalError;
use super::journal::Owners;
use super::journal::State;
use super::root::Reply;
use super::root::Request;
use super::root::RootError;
use super::root::RootSide;
use crate::update::event::ActionEvent;
use crate::update::event::ActionState;
use crate::update::event::Sink;
use crate::update::event::UpdateEvent;

/// What happened to one step.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Outcome {
    /// Its journal says it is done: nothing ran.
    Settled,
    /// Done already, by something other than the app.
    Found,
    /// It opens a window, in a run that may not; `says` is the step left to the user.
    Skipped { says: String },
    /// Done now.
    Done,
    /// Undone now.
    Undone,
}

/// A step that didn't go through.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StepError {
    #[error(transparent)]
    Perform(PerformError),
    #[error(transparent)]
    Journal(JournalError),
    #[error(transparent)]
    Folder(FolderError),
    #[error("{} belongs to {owner}", path.display())]
    Owned { path: PathBuf, owner: String },
    #[error("{} is yours: KubaOS never writes over a file it didn't put there", path.display())]
    Yours { path: PathBuf },
    #[error("{key} belongs to {owner}")]
    KeyOwned { key: String, owner: String },
    #[error("step {step} of {app} is not one this side runs")]
    WrongSide { app: String, step: usize },
    #[error("{app} has no step {step}")]
    NoStep { app: String, step: usize },
    #[error("{0}")]
    Root(String),
    #[error("the root helper")]
    RootLost(#[source] RootError),
}

/// An install or a removal that didn't go through.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EngineError {
    #[error(transparent)]
    Folder(FolderError),
    #[error(transparent)]
    Journal(JournalError),
    #[error("{0}")]
    NotReady(String),
    #[error("you said no: nothing changed")]
    Declined,
    #[error("the root helper")]
    Root(#[source] RootError),
    #[error("{name}")]
    Step {
        name: String,
        #[source]
        source: Box<StepError>,
    },
    #[error("{name}: it couldn't be undone")]
    Undo {
        name: String,
        #[source]
        source: Box<StepError>,
    },
    #[error("{app} can't be installed: {}", why.join("; "))]
    Refused { app: String, why: Vec<String> },
}

/// One side's steps of the apps' installs: the user's in this process, root's in the root helper.
pub struct Keeper<'h> {
    host: &'h mut dyn Host,
    side: Side,
    catalog: PathBuf,
    journals: PathBuf,
}

impl std::fmt::Debug for Keeper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keeper")
            .field("side", &self.side)
            .field("catalog", &self.catalog)
            .field("journals", &self.journals)
            .finish_non_exhaustive()
    }
}

impl<'h> Keeper<'h> {
    /// The `side` of the catalog `catalog` on `host`, with its journals in `journals`.
    pub fn new(host: &'h mut dyn Host, side: Side, catalog: impl Into<PathBuf>, journals: impl Into<PathBuf>) -> Self {
        Self {
            host,
            side,
            catalog: catalog.into(),
            journals: journals.into(),
        }
    }

    /// Its journals' folder.
    pub fn journals(&self) -> &Path {
        &self.journals
    }

    /// Its host.
    pub fn host(&mut self) -> &mut dyn Host {
        &mut *self.host
    }

    fn words(&self) -> Words {
        Words {
            user: self.host.user().to_owned(),
            home: self.host.home().to_path_buf(),
            arch: self.host.arch().to_owned(),
        }
    }

    /// This side's steps of the app `id` in `setup`, by number, resolved for this machine.
    fn steps(&self, id: &str, setup: Option<&str>) -> Result<(Folder, Vec<(usize, Action)>), StepError> {
        let folder = Folder::load(&self.catalog, id).map_err(StepError::Folder)?;
        let words = self.words();
        let steps = folder
            .steps(setup)
            .map_err(StepError::Folder)?
            .into_iter()
            .enumerate()
            .filter(|(_, action)| action.side() == self.side)
            .map(|(i, action)| {
                action.resolve(&words, &folder.dir).map(|a| (i + 1, a)).map_err(|e| {
                    StepError::Folder(FolderError::Action {
                        setup: setup.map(str::to_owned),
                        position: i + 1,
                        source: e,
                    })
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok((folder, steps))
    }

    /// Checks, before anything runs, that no step of this side writes a path or a key another
    /// app's journal owns, or a path that is the user's own.
    ///
    /// # Errors
    ///
    /// [`StepError::Owned`], [`StepError::KeyOwned`] or [`StepError::Yours`] for the first such
    /// step's path, or the step that couldn't be looked at.
    pub fn preflight(&mut self, id: &str, setup: Option<&str>, windows: bool) -> Result<Vec<String>, StepError> {
        let (folder, steps) = self.steps(id, setup)?;
        let owners = Owners::load(&[&self.journals]).map_err(StepError::Journal)?;
        let journal = Journal::read(&self.journals, id).map_err(StepError::Journal)?;
        let mut refused = Vec::new();
        for (step, action) in steps {
            let settled = journal
                .as_ref()
                .and_then(|j| j.entry(step))
                .is_some_and(|e| e.action == action && e.state.settled(windows));
            let writes = action.path().is_some() || matches!(action.kind, Kind::Setting { .. });
            if settled || !writes {
                continue;
            }
            let look = perform::look(&action, &mut *self.host, &folder.dir).map_err(StepError::Perform)?;
            if let Err(e) = self.may_write(&owners, id, &look) {
                refused.push(format!("{}: {e}", action.name));
            }
        }
        Ok(refused)
    }

    fn may_write(&self, owners: &Owners, id: &str, look: &perform::Look) -> Result<(), StepError> {
        for claim in &look.claims {
            if let Some(path) = claim.path() {
                if let Some(owner) = owners.path_owner(path, id) {
                    return Err(StepError::Owned {
                        path: path.to_path_buf(),
                        owner: owner.to_owned(),
                    });
                }
                let there = self.host.place(path).symlink_metadata().is_ok();
                if claim.owned && there && !owners.covers(id, path) {
                    return Err(StepError::Yours {
                        path: path.to_path_buf(),
                    });
                }
            } else if claim.claim.starts_with("key:")
                && let Some(owner) = owners.owner(&claim.claim, id)
            {
                return Err(StepError::KeyOwned {
                    key: claim.claim["key:".len()..].to_owned(),
                    owner: owner.to_owned(),
                });
            }
        }
        Ok(())
    }

    /// Step `step` of the app `id`'s install in `setup`: settled when its journal says so, else
    /// looked at, journaled, done and journaled again. A run that may not open a window skips a
    /// step that opens one.
    ///
    /// # Errors
    ///
    /// [`StepError::WrongSide`] for a step of the other side, [`StepError::NoStep`] for one the
    /// install hasn't, the ownership refusals of [`Keeper::preflight`], else what failed. A step
    /// that failed stays in the journal as started, for the rollback to undo.
    pub fn step(&mut self, id: &str, setup: Option<&str>, step: usize, windows: bool) -> Result<Outcome, StepError> {
        let (folder, steps) = self.steps(id, setup)?;
        let Some((_, action)) = steps.into_iter().find(|(n, _)| *n == step) else {
            let total = folder.steps(setup).map_err(StepError::Folder)?.len();
            return Err(if (1..=total).contains(&step) {
                StepError::WrongSide {
                    app: id.to_owned(),
                    step,
                }
            } else {
                StepError::NoStep {
                    app: id.to_owned(),
                    step,
                }
            });
        };
        let mut journal = Journal::read(&self.journals, id)
            .map_err(StepError::Journal)?
            .unwrap_or_else(|| Journal::new(id, setup));
        journal.setup = setup.map(str::to_owned);
        if let Some(entry) = journal.entry(step).cloned() {
            if entry.action == action && entry.state.settled(windows) {
                return Ok(Outcome::Settled);
            }
            if entry.action == action && entry.state == State::Doing {
                return self.finish(&mut journal, entry, &folder.dir);
            }
            self.undo_entry(id, &entry, &folder.dir)?;
            journal.drop_step(step);
            journal.save(&self.journals).map_err(StepError::Journal)?;
        }
        if let Kind::Run { window: Some(says), .. } = &action.kind
            && !windows
        {
            journal.put(Entry {
                step,
                state: State::Skipped,
                claims: Vec::new(),
                prior: journal::Prior::default(),
                action: action.clone(),
            });
            journal.save(&self.journals).map_err(StepError::Journal)?;
            return Ok(Outcome::Skipped { says: says.clone() });
        }
        let look = perform::look(&action, &mut *self.host, &folder.dir).map_err(StepError::Perform)?;
        if look.done {
            journal.put(Entry {
                step,
                state: State::Found,
                claims: look.claims.into_iter().map(|c| Claim::new(c.claim, false)).collect(),
                prior: look.prior,
                action,
            });
            journal.save(&self.journals).map_err(StepError::Journal)?;
            return Ok(Outcome::Found);
        }
        let owners = Owners::load(&[&self.journals]).map_err(StepError::Journal)?;
        self.may_write(&owners, id, &look)?;
        let entry = Entry {
            step,
            state: State::Doing,
            claims: look.claims.clone(),
            prior: look.prior.clone(),
            action,
        };
        journal.put(entry.clone());
        journal.save(&self.journals).map_err(StepError::Journal)?;
        perform::apply(&entry.action, &mut *self.host, &look, &folder.dir).map_err(StepError::Perform)?;
        journal.put(Entry {
            state: State::Done,
            ..entry
        });
        journal.save(&self.journals).map_err(StepError::Journal)?;
        Ok(Outcome::Done)
    }

    /// A step a run stopped during: done when its check says so, else done again from the look
    /// recorded before it first ran.
    fn finish(&mut self, journal: &mut Journal, entry: Entry, folder: &Path) -> Result<Outcome, StepError> {
        if !perform::check(&entry.action, &mut *self.host, folder).map_err(StepError::Perform)? {
            let look = perform::Look {
                done: false,
                claims: entry.claims.clone(),
                prior: entry.prior.clone(),
            };
            perform::apply(&entry.action, &mut *self.host, &look, folder).map_err(StepError::Perform)?;
        }
        journal.put(Entry {
            state: State::Done,
            ..entry
        });
        journal.save(&self.journals).map_err(StepError::Journal)?;
        Ok(Outcome::Done)
    }

    /// Undoes step `step` of the app `id`'s journal and takes it out; what another app holds
    /// passes to it instead of being undone.
    ///
    /// # Errors
    ///
    /// What failed; the step stays in the journal then. A step the journal hasn't is undone
    /// already.
    pub fn undo(&mut self, id: &str, step: usize) -> Result<Outcome, StepError> {
        let Some(mut journal) = Journal::read(&self.journals, id).map_err(StepError::Journal)? else {
            return Ok(Outcome::Undone);
        };
        let Some(entry) = journal.entry(step).cloned() else {
            return Ok(Outcome::Undone);
        };
        self.undo_entry(id, &entry, &self.catalog.join(id))?;
        journal.drop_step(step);
        journal.save(&self.journals).map_err(StepError::Journal)?;
        Ok(Outcome::Undone)
    }

    fn undo_entry(&mut self, id: &str, entry: &Entry, folder: &Path) -> Result<(), StepError> {
        // A found step owns nothing, unless another app's removal has handed it a claim since.
        if entry.state == State::Skipped {
            return Ok(());
        }
        let owners = Owners::load(&[&self.journals]).map_err(StepError::Journal)?;
        let mut mine = Vec::new();
        for claim in entry.claims.iter().filter(|c| c.owned) {
            match owners.holder(&claim.claim, id) {
                Some(holder) => {
                    journal::hand_over(&self.journals, holder, &claim.claim).map_err(StepError::Journal)?;
                }
                None => mine.push(claim.clone()),
            }
        }
        perform::undo(&entry.action, &mut *self.host, &mine, &entry.prior, folder).map_err(StepError::Perform)
    }
}

/// Installs and removes apps of folders, reporting each step.
pub struct Engine<'a> {
    user: Keeper<'a>,
    root: &'a mut dyn RootSide,
    sink: &'a mut dyn Sink,
    root_journals: PathBuf,
    lock: PathBuf,
    windows: bool,
    root_started: bool,
}

impl std::fmt::Debug for Engine<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("user", &self.user)
            .field("root_journals", &self.root_journals)
            .field("windows", &self.windows)
            .finish_non_exhaustive()
    }
}

/// What a finished install or removal leaves for its caller to do.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Finished {
    /// The User flake's `packages.toml` changed: a switch installs or removes its names.
    pub switch: bool,
}

/// One step of either side's journal, for an undo across both.
#[derive(Clone, Debug)]
struct Undoable {
    step: usize,
    side: Side,
    name: String,
    kind_is_nix: bool,
}

impl<'a> Engine<'a> {
    /// An engine with the user's side `user`, root's behind `root`, root's journals readable at
    /// `root_journals`, one run at a time through the lock file `lock`, reporting to `sink`. With
    /// `windows`, a step may open a window.
    pub fn new(
        user: Keeper<'a>,
        root: &'a mut dyn RootSide,
        root_journals: impl Into<PathBuf>,
        lock: impl Into<PathBuf>,
        sink: &'a mut dyn Sink,
        windows: bool,
    ) -> Self {
        Self {
            user,
            root,
            sink,
            root_journals: root_journals.into(),
            lock: lock.into(),
            windows,
            root_started: false,
        }
    }

    /// Installs the app `id` in `setup`, from where its journals left it: a step done before is
    /// reported done, a step stopped during is finished, a failure undoes what the install did.
    /// An app the machine has in another setup first loses that setup's own steps.
    ///
    /// # Errors
    ///
    /// [`EngineError`]; after a failure that wasn't a refusal before the first step, the machine
    /// is as it was before the install began.
    pub fn install(&mut self, id: &str, setup: Option<&str>) -> Result<Finished, EngineError> {
        let _lock = journal::lock(&self.lock).map_err(EngineError::Journal)?;
        let folder = Folder::load(&self.user.catalog, id).map_err(EngineError::Folder)?;
        let picked = folder.pick(setup).map_err(EngineError::Folder)?;
        let steps: Vec<Action> = folder
            .steps(setup)
            .map_err(EngineError::Folder)?
            .into_iter()
            .cloned()
            .collect();
        self.ready(&folder, picked.map(|s| s.id.as_str()))?;
        let user_journal = Journal::read(self.user.journals(), id).map_err(EngineError::Journal)?;
        let root_journal = Journal::read(&self.root_journals, id).map_err(EngineError::Journal)?;
        let had = user_journal.as_ref().or(root_journal.as_ref()).map(|j| j.setup.clone());
        let switching = had.as_ref().is_some_and(|had| had.as_deref() != setup);
        let was_complete = !switching && user_journal.as_ref().is_some_and(|j| j.complete);
        let root_settled = |n: usize, action: &Action| {
            root_journal
                .as_ref()
                .and_then(|j| j.entry(n))
                .is_some_and(|e| e.state.settled(self.windows) && e.action.name == action.name)
        };
        let needs_root = steps
            .iter()
            .enumerate()
            .any(|(i, a)| a.side() == Side::Root && !root_settled(i + 1, a))
            || (switching && root_journal.as_ref().is_some_and(|j| !j.entries.is_empty()));
        if needs_root {
            self.start_root()?;
        }
        let mut finished = Finished::default();
        if switching {
            finished.switch |= self.switch_away(id, folder.actions.len())?;
        }
        self.preflight(id, setup, needs_root)?;
        let total = steps.len();
        let mut this_run = Vec::new();
        for (i, action) in steps.iter().enumerate() {
            let step = i + 1;
            self.sink.event(&UpdateEvent::Step {
                text: action.name.clone(),
            });
            self.sink.event(&UpdateEvent::Progress { done: i, total });
            let outcome = match action.side() {
                Side::User => self.user.step(id, setup, step, self.windows),
                Side::Root => self.root_step(&Request::Step {
                    app: id.to_owned(),
                    setup: setup.map(str::to_owned),
                    step,
                    windows: self.windows,
                }),
            };
            let undoable = Undoable {
                step,
                side: action.side(),
                name: action.name.clone(),
                kind_is_nix: matches!(action.kind, Kind::Nix { .. }),
            };
            match outcome {
                Ok(Outcome::Settled) => self.report(id, step, total, &action.name, ActionState::Done),
                Ok(Outcome::Found) => self.report(id, step, total, &action.name, ActionState::Found),
                Ok(Outcome::Skipped { says }) => {
                    self.report(id, step, total, &action.name, ActionState::Skipped);
                    self.sink.event(&UpdateEvent::Note { text: says });
                }
                Ok(Outcome::Done | Outcome::Undone) => {
                    finished.switch |= undoable.kind_is_nix;
                    this_run.push(undoable);
                    self.report(id, step, total, &action.name, ActionState::Done);
                }
                Err(source) => {
                    this_run.push(undoable);
                    self.report(id, step, total, &action.name, ActionState::Failed);
                    let rollback = if was_complete {
                        this_run
                    } else {
                        self.undoables(id, &steps)?
                    };
                    self.undo_all(id, total, rollback, !was_complete);
                    return Err(EngineError::Step {
                        name: action.name.clone(),
                        source: Box::new(source),
                    });
                }
            }
        }
        self.sink.event(&UpdateEvent::Progress { done: total, total });
        let mut journal = Journal::read(self.user.journals(), id)
            .map_err(EngineError::Journal)?
            .unwrap_or_else(|| Journal::new(id, setup));
        journal.setup = setup.map(str::to_owned);
        journal.complete = true;
        journal.save(self.user.journals()).map_err(EngineError::Journal)?;
        Ok(finished)
    }

    /// Removes the app `id`: its journals undone, newest step first, then gone. An app with no
    /// journal is not installed, and nothing happens.
    ///
    /// # Errors
    ///
    /// [`EngineError`]. A step that couldn't be undone stops the removal; the journal keeps the
    /// steps left, and the next removal goes on from them.
    pub fn remove(&mut self, id: &str) -> Result<Finished, EngineError> {
        let _lock = journal::lock(&self.lock).map_err(EngineError::Journal)?;
        let user_journal = Journal::read(self.user.journals(), id).map_err(EngineError::Journal)?;
        let root_journal = Journal::read(&self.root_journals, id).map_err(EngineError::Journal)?;
        if user_journal.is_none() && root_journal.is_none() {
            return Ok(Finished::default());
        }
        let setup = user_journal
            .as_ref()
            .or(root_journal.as_ref())
            .and_then(|j| j.setup.clone());
        if let Ok(folder) = Folder::load(&self.user.catalog, id) {
            self.ready(&folder, setup.as_deref())?;
        }
        if root_journal.as_ref().is_some_and(|j| !j.entries.is_empty()) {
            self.start_root()?;
        }
        if let Some(mut journal) = user_journal {
            journal.complete = false;
            journal.save(self.user.journals()).map_err(EngineError::Journal)?;
        }
        let all = self.journaled(id)?;
        let total = all.iter().map(|u| u.step).max().unwrap_or(0);
        let mut finished = Finished::default();
        for undoable in all {
            self.report(id, undoable.step, total, &undoable.name, ActionState::Undoing);
            self.undo_one(id, &undoable).map_err(|source| EngineError::Undo {
                name: undoable.name.clone(),
                source: Box::new(source),
            })?;
            finished.switch |= undoable.kind_is_nix;
            self.report(id, undoable.step, total, &undoable.name, ActionState::Undone);
        }
        Journal::forget(self.user.journals(), id).map_err(EngineError::Journal)?;
        Ok(finished)
    }

    fn ready(&mut self, folder: &Folder, setup: Option<&str>) -> Result<(), EngineError> {
        let picked = setup.and_then(|s| folder.setup(s));
        for ready in folder.readies(picked) {
            if self.user.host().run(&ready.check, &folder.dir).is_err() {
                return Err(EngineError::NotReady(ready.says.clone()));
            }
        }
        Ok(())
    }

    fn start_root(&mut self) -> Result<(), EngineError> {
        if self.root_started {
            return Ok(());
        }
        match self.root.start() {
            Ok(()) => {
                self.root_started = true;
                Ok(())
            }
            Err(RootError::Declined) => {
                self.sink.event(&UpdateEvent::Declined);
                Err(EngineError::Declined)
            }
            Err(e) => Err(EngineError::Root(e)),
        }
    }

    fn preflight(&mut self, id: &str, setup: Option<&str>, root: bool) -> Result<(), EngineError> {
        let refuse = |why: Vec<String>| EngineError::Refused {
            app: id.to_owned(),
            why,
        };
        let mut why = self
            .user
            .preflight(id, setup, self.windows)
            .map_err(|source| EngineError::Step {
                name: "Checking what the app would change".to_owned(),
                source: Box::new(source),
            })?;
        if root {
            match self.root.call(&Request::Preflight {
                app: id.to_owned(),
                setup: setup.map(str::to_owned),
                windows: self.windows,
            }) {
                Ok(Reply::Refused { why: root_why }) => why.extend(root_why),
                Ok(Reply::Failed { why: e }) => return Err(refuse(vec![e])),
                Err(e) => return Err(EngineError::Root(e)),
                Ok(_) => {}
            }
        }
        if why.is_empty() { Ok(()) } else { Err(refuse(why)) }
    }

    fn root_step(&mut self, request: &Request) -> Result<Outcome, StepError> {
        // A root step the journals' reading said was settled, and isn't, starts root here.
        if !self.root_started {
            self.root.start().map_err(StepError::RootLost)?;
            self.root_started = true;
        }
        match self.root.call(request) {
            Ok(Reply::Step { outcome }) => Ok(outcome),
            Ok(Reply::Failed { why }) => Err(StepError::Root(why)),
            Ok(Reply::Refused { why }) => Err(StepError::Root(why.join("; "))),
            Ok(Reply::Ready) => Err(StepError::Root("the root helper answered out of turn".to_owned())),
            Err(e) => Err(StepError::RootLost(e)),
        }
    }

    fn undo_one(&mut self, id: &str, undoable: &Undoable) -> Result<(), StepError> {
        match undoable.side {
            Side::User => self.user.undo(id, undoable.step).map(|_| ()),
            Side::Root => self
                .root_step(&Request::Undo {
                    app: id.to_owned(),
                    step: undoable.step,
                })
                .map(|_| ()),
        }
    }

    /// Undoes `undoables`, newest first, reporting each; one that fails is a warning, and the rest
    /// still go. With `forget`, the journals go too: the app wasn't installed when the run began.
    fn undo_all(&mut self, id: &str, total: usize, mut undoables: Vec<Undoable>, forget: bool) {
        undoables.sort_by_key(|u| std::cmp::Reverse(u.step));
        let mut clean = true;
        for undoable in &undoables {
            self.report(id, undoable.step, total, &undoable.name, ActionState::Undoing);
            match self.undo_one(id, undoable) {
                Ok(()) => self.report(id, undoable.step, total, &undoable.name, ActionState::Undone),
                Err(e) => {
                    clean = false;
                    self.sink.event(&UpdateEvent::Warning {
                        text: format!(
                            "{}: it couldn't be undone: {}",
                            undoable.name,
                            crate::update::event::chain(&e)
                        ),
                    });
                }
            }
        }
        if forget && clean {
            let _ = Journal::forget(self.user.journals(), id);
        }
    }

    /// Every step of both journals of the app `id`, newest first, named as `steps` names them.
    fn undoables(&self, id: &str, steps: &[Action]) -> Result<Vec<Undoable>, EngineError> {
        let mut all = self.journaled(id)?;
        for u in &mut all {
            if let Some(action) = steps.get(u.step - 1) {
                u.name.clone_from(&action.name);
            }
        }
        Ok(all)
    }

    /// Every step of both journals of the app `id`, newest first.
    fn journaled(&self, id: &str) -> Result<Vec<Undoable>, EngineError> {
        let mut all = Vec::new();
        for (dir, side) in [
            (self.user.journals().to_path_buf(), Side::User),
            (self.root_journals.clone(), Side::Root),
        ] {
            if let Some(journal) = Journal::read(&dir, id).map_err(EngineError::Journal)? {
                all.extend(journal.entries.into_iter().map(|e| Undoable {
                    step: e.step,
                    side,
                    name: e.action.name.clone(),
                    kind_is_nix: matches!(e.action.kind, Kind::Nix { .. }),
                }));
            }
        }
        all.sort_by_key(|u| std::cmp::Reverse(u.step));
        Ok(all)
    }

    /// Takes away the steps of another setup than the one being installed: those after the app's
    /// own `shared` actions. Whether `packages.toml` changed.
    fn switch_away(&mut self, id: &str, shared: usize) -> Result<bool, EngineError> {
        let mut switch = false;
        let leaving: Vec<Undoable> = self.journaled(id)?.into_iter().filter(|u| u.step > shared).collect();
        let total = leaving.first().map_or(0, |u| u.step);
        for undoable in leaving {
            self.report(id, undoable.step, total, &undoable.name, ActionState::Undoing);
            self.undo_one(id, &undoable).map_err(|source| EngineError::Undo {
                name: undoable.name.clone(),
                source: Box::new(source),
            })?;
            switch |= undoable.kind_is_nix;
            self.report(id, undoable.step, total, &undoable.name, ActionState::Undone);
        }
        if let Some(mut journal) = Journal::read(self.user.journals(), id).map_err(EngineError::Journal)? {
            journal.complete = false;
            journal.save(self.user.journals()).map_err(EngineError::Journal)?;
        }
        Ok(switch)
    }

    fn report(&mut self, app: &str, step: usize, of: usize, name: &str, state: ActionState) {
        self.sink.event(&UpdateEvent::Action(ActionEvent {
            app: app.to_owned(),
            step,
            of,
            name: name.to_owned(),
            state,
        }));
    }
}

#[cfg(test)]
mod tests;
