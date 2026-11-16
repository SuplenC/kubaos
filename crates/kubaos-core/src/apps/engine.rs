//! Installing and removing an app of folders: its steps in order, each checked and journaled
//! before it runs, a failure rolled back, a removal undone from the journal, a stopped run
//! resumed, a step a release changed updated in place.
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

use std::collections::BTreeSet;
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
use super::action::perform::Recorded;
use super::folder::Folder;
use super::folder::FolderError;
use super::host::Host;
use super::journal;
use super::journal::Adoption;
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
    /// Its journal says it is done and its check agrees: nothing changed.
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
    #[error("{why}")]
    RootFailed { why: String },
    #[error("the root helper refused it: {}", why.join("; "))]
    RootRefused { why: Vec<String> },
    #[error("the root helper answered out of turn")]
    RootOutOfTurn,
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

    /// The words its actions take on its host.
    pub(crate) fn words(&self) -> Words {
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
    /// app's journal owns, or a path that is the user's own, and that every settings file a step
    /// changes is one it can read.
    ///
    /// # Errors
    ///
    /// The step that couldn't be resolved or the journals that couldn't be read; the refusals
    /// are the answer, one a step.
    pub fn preflight(&mut self, id: &str, setup: Option<&str>) -> Result<Vec<String>, StepError> {
        let (_, steps) = self.steps(id, setup)?;
        let owners = Owners::load(&[&self.journals]).map_err(StepError::Journal)?;
        let mut refused = Vec::new();
        for (_, action) in steps {
            let writes = action.path().is_some() || matches!(action.kind, Kind::Setting { .. } | Kind::WebApp { .. });
            if !writes {
                continue;
            }
            let checked = perform::look(&action, &mut *self.host, Recorded::Nothing)
                .map_err(StepError::Perform)
                .and_then(|look| self.may_write(&owners, id, &look.claims));
            if let Err(e) = checked {
                refused.push(format!("{}: {}", action.name, crate::update::event::chain(&e)));
            }
        }
        Ok(refused)
    }

    fn may_write(&self, owners: &Owners, id: &str, claims: &[Claim]) -> Result<(), StepError> {
        for claim in claims {
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

    /// Step `step` of the app `id`'s install in `setup`, checked whatever its journal says: done
    /// already is settled (or found, when the app never did it), not done is journaled, done and
    /// journaled again. A step a release changed is updated in place, or, when it now acts on
    /// another thing, undone first. A run that may not open a window skips a step that opens one
    /// and isn't done.
    ///
    /// # Errors
    ///
    /// [`StepError::WrongSide`] for a step of the other side, [`StepError::NoStep`] for one the
    /// install hasn't, the ownership refusals of [`Keeper::preflight`], else what failed. A step
    /// that failed stays in the journal as started, for the rollback or the next run.
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
        let mut old = journal.entry(step).cloned();
        if let Some(entry) = old.take_if(|e| !e.action.same_thing(&action)) {
            self.undo_journaled(&mut journal, id, &entry)?;
        }
        let look = perform::look(&action, &mut *self.host, recorded(old.as_ref())).map_err(StepError::Perform)?;
        if let Kind::Run { window: Some(says), .. } = &action.kind
            && !windows
            && !look.done
        {
            if old.as_ref().is_none_or(|e| e.state == State::Skipped) {
                journal.put(Entry {
                    step,
                    state: State::Skipped,
                    claims: Vec::new(),
                    prior: journal::Prior::default(),
                    action: action.clone(),
                    replaced: None,
                });
                journal.save(&self.journals).map_err(StepError::Journal)?;
            }
            return Ok(Outcome::Skipped { says: says.clone() });
        }
        let (claims, replaced) = carry(old.as_ref(), &look);
        let prior = match &old {
            Some(e) if e.state.ours() => e.prior.clone(),
            _ => look.prior.clone(),
        };
        if look.done {
            let owns = claims.iter().any(|c| c.owned);
            let entry = Entry {
                step,
                state: if owns { State::Done } else { State::Found },
                claims,
                prior,
                action,
                replaced,
            };
            if old.as_ref() != Some(&entry) {
                journal.put(entry);
                journal.save(&self.journals).map_err(StepError::Journal)?;
            }
            self.finish_replaced(&mut journal, id, step)?;
            return Ok(if old.is_some() {
                Outcome::Settled
            } else {
                Outcome::Found
            });
        }
        let owners = Owners::load(&[&self.journals]).map_err(StepError::Journal)?;
        self.may_write(&owners, id, &claims)?;
        let mut entry = Entry {
            step,
            state: State::Doing,
            claims,
            prior,
            action,
            replaced,
        };
        journal.put(entry.clone());
        journal.save(&self.journals).map_err(StepError::Journal)?;
        let journals = &self.journals;
        let action = entry.action.clone();
        perform::apply(&action, &mut *self.host, &look, &mut |claims| {
            for claim in claims {
                if !entry.claims.contains(claim) {
                    entry.claims.push(claim.clone());
                }
            }
            journal.put(entry.clone());
            journal.save(journals)
        })
        .map_err(StepError::Perform)?;
        entry.state = State::Done;
        journal.put(entry);
        journal.save(&self.journals).map_err(StepError::Journal)?;
        self.finish_replaced(&mut journal, id, step)?;
        Ok(Outcome::Done)
    }

    /// Records the steps of this side of the app `id` in `setup` that check done, without
    /// running anything else, for an app the machine had before it had a journal. With `owned`
    /// (KubaOS put the app there), a step owns what its undo can take back as a machine without
    /// the app had it: its packages, paths, units (neither enabled nor running), settings keys
    /// (absent), group membership and `run`s, never a group itself, which a package makes;
    /// lingering only when KubaOS's older mark, `<id>-linger-was-off`, says it was off; and a
    /// path that is there and holds an older release's content, which the next run brings to
    /// this one's. Without `owned`, every step is found and owns nothing. A step with an entry
    /// already, and one not done (but for such a path), is left to the next run. Returns how many
    /// steps it recorded.
    ///
    /// # Errors
    ///
    /// The step that couldn't be resolved or checked, or the journal that couldn't be written.
    pub fn adopt(&mut self, id: &str, setup: Option<&str>, owned: bool) -> Result<usize, StepError> {
        let (_, steps) = self.steps(id, setup)?;
        let mut journal = Journal::read(&self.journals, id)
            .map_err(StepError::Journal)?
            .unwrap_or_else(|| Journal::new(id, setup));
        journal.setup = setup.map(str::to_owned);
        let mark = self.journals.join(format!("{id}{LINGER_MARK}"));
        let lingered_off = owned && mark.is_file();
        let mut recorded = 0;
        for (step, action) in steps {
            if journal.entry(step).is_some() {
                continue;
            }
            let look = perform::look(&action, &mut *self.host, Recorded::Nothing).map_err(StepError::Perform)?;
            let stale = owned
                && !look.done
                && action.path().is_some()
                && look.claims.iter().any(|c| {
                    c.owned && c.path().is_some_and(|p| self.host.place(p).symlink_metadata().is_ok())
                });
            if !look.done && !stale {
                continue;
            }
            let (claims, prior) = if owned {
                adopted(&action, look, lingered_off)
            } else {
                (look.claims.into_iter().map(|c| Claim::new(c.claim, false)).collect(), look.prior)
            };
            let state = if claims.iter().any(|c| c.owned) {
                State::Done
            } else {
                State::Found
            };
            journal.put(Entry {
                step,
                state,
                claims,
                prior,
                action,
                replaced: None,
            });
            recorded += 1;
        }
        journal.save(&self.journals).map_err(StepError::Journal)?;
        if mark.is_file() {
            std::fs::remove_file(&mark).map_err(|e| {
                StepError::Journal(JournalError::Write {
                    path: mark.clone(),
                    source: e,
                })
            })?;
        }
        Ok(recorded)
    }

    /// Undoes what the step `step` replaced, once its changed action stands, and forgets it.
    fn finish_replaced(&mut self, journal: &mut Journal, id: &str, step: usize) -> Result<(), StepError> {
        let Some(mut entry) = journal.entry(step).cloned() else {
            return Ok(());
        };
        let Some(replaced) = entry.replaced.take() else {
            return Ok(());
        };
        self.undo_entry(id, &replaced)?;
        journal.put(entry);
        journal.save(&self.journals).map_err(StepError::Journal)
    }

    /// Undoes step `step` of the app `id`'s journal and takes it out; what another app holds
    /// passes to it instead of being undone.
    ///
    /// # Errors
    ///
    /// What failed; the step stays in the journal then, as being undone. A step the journal
    /// hasn't is undone already.
    pub fn undo(&mut self, id: &str, step: usize) -> Result<Outcome, StepError> {
        let Some(mut journal) = Journal::read(&self.journals, id).map_err(StepError::Journal)? else {
            return Ok(Outcome::Undone);
        };
        let Some(entry) = journal.entry(step).cloned() else {
            return Ok(Outcome::Undone);
        };
        self.undo_journaled(&mut journal, id, &entry)?;
        Ok(Outcome::Undone)
    }

    /// Marks `entry` as being undone, undoes it, and takes it out of `journal`; a run stopped
    /// during the undo leaves it marked, never looking settled.
    fn undo_journaled(&mut self, journal: &mut Journal, id: &str, entry: &Entry) -> Result<(), StepError> {
        let step = entry.step;
        journal.put(Entry {
            state: if entry.state == State::Skipped {
                State::Skipped
            } else {
                State::Undoing
            },
            ..entry.clone()
        });
        journal.save(&self.journals).map_err(StepError::Journal)?;
        self.undo_entry(id, entry)?;
        journal.drop_step(step);
        journal.save(&self.journals).map_err(StepError::Journal)
    }

    fn undo_entry(&mut self, id: &str, entry: &Entry) -> Result<(), StepError> {
        // A found step owns nothing, unless another app's removal has handed it a claim since.
        if entry.state != State::Skipped {
            let owners = Owners::load(&[&self.journals]).map_err(StepError::Journal)?;
            let mut mine = Vec::new();
            for claim in entry.claims.iter().filter(|c| c.owned) {
                // A package one step pulled in, which another app's step found, passes to it.
                let held = match claim.claim.strip_prefix("dep:") {
                    Some(package) => format!("package:{package}"),
                    None => claim.claim.clone(),
                };
                match owners.holder(&held, id) {
                    Some(holder) => {
                        journal::hand_over(&self.journals, holder, &held, &entry.prior).map_err(StepError::Journal)?;
                    }
                    None => mine.push(claim.clone()),
                }
            }
            perform::undo(&entry.action, &mut *self.host, &mine, &entry.prior).map_err(StepError::Perform)?;
        }
        match &entry.replaced {
            Some(replaced) => self.undo_entry(id, replaced),
            None => Ok(()),
        }
    }
}

/// The file KubaOS's older setups left beside the journals, `<id>-linger-was-off`, while
/// lingering was off before the app's service turned it on.
pub const LINGER_MARK: &str = "-linger-was-off";

/// The claims and prior state of a step of an app KubaOS put on the machine, adopted: owned as
/// [`Keeper::adopt`] says, with the prior state of a machine without the app.
fn adopted(action: &Action, look: perform::Look, lingered_off: bool) -> (Vec<Claim>, journal::Prior) {
    let claims = look
        .claims
        .into_iter()
        .map(|c| {
            let owned = if c.claim.starts_with("group:") || c.claim.starts_with("dep:") {
                false
            } else if c.claim.starts_with("linger:") {
                lingered_off
            } else {
                true
            };
            Claim::new(c.claim, owned)
        })
        .collect();
    let prior = match &action.kind {
        Kind::Unit { .. } => journal::Prior {
            enabled: Some(false),
            active: Some(false),
            ..journal::Prior::default()
        },
        Kind::Setting { .. } => journal::Prior {
            value: None,
            text: None,
            file: Some(true),
            ..look.prior
        },
        Kind::Linger {} => journal::Prior {
            lingered: Some(!lingered_off),
            ..journal::Prior::default()
        },
        _ => look.prior,
    };
    (claims, prior)
}

/// What the step's entry `old` says its path holds: what its action put there, when the app owns
/// the path and the step is done; unknown, when a run stopped during it.
fn recorded(old: Option<&Entry>) -> Recorded<'_> {
    let Some(old) = old else {
        return Recorded::Nothing;
    };
    if !old.claims.iter().any(|c| c.owned && c.claim.starts_with("path:")) {
        return Recorded::Nothing;
    }
    match old.state {
        State::Done => Recorded::Done(&old.action),
        State::Doing | State::Undoing => Recorded::Unsettled(&old.action),
        State::Found | State::Skipped => Recorded::Nothing,
    }
}

/// The claims of a step about to be recorded from `look`, keeping what its `old` entry owned:
/// a claim both have stays owned, as does a folder the old one made that the new one's path is
/// in. What the old entry owned and the new look doesn't claim is what it replaces, undone once
/// the new action stands.
fn carry(old: Option<&Entry>, look: &perform::Look) -> (Vec<Claim>, Option<Box<Entry>>) {
    let mut claims = look.claims.clone();
    let Some(old) = old else {
        return (claims, None);
    };
    let mut stale = Vec::new();
    for owned in old.claims.iter().filter(|c| c.owned) {
        if let Some(claim) = claims.iter_mut().find(|c| c.claim == owned.claim) {
            claim.owned = true;
            continue;
        }
        // What apt pulled in for the step's packages stays the step's while it stands: a check
        // never sees it, and only a removal can tell whether something else needs it.
        if owned.claim.starts_with("dep:") {
            claims.push(owned.clone());
            continue;
        }
        let dir = owned.claim.strip_prefix("dir:").map(Path::new);
        let holds = |c: &Claim| {
            let path = c.path().or_else(|| c.claim.strip_prefix("dir:").map(Path::new));
            dir.is_some_and(|dir| path.is_some_and(|p| p.starts_with(dir)))
        };
        if claims.iter().any(holds) {
            claims.insert(0, owned.clone());
        } else {
            stale.push(owned.clone());
        }
    }
    let replaced = if stale.is_empty() {
        old.replaced.clone()
    } else {
        Some(Box::new(Entry {
            claims: stale,
            ..old.clone()
        }))
    };
    (claims, replaced)
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

/// An app taken in by [`Engine::adopt`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Adopted {
    /// The setup the machine has it in.
    pub setup: Option<String>,
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

    /// Installs the app `id` in `setup`, from where its journals left it: every step is checked,
    /// a step done before is reported done, a step not done (stopped during, or undone since) is
    /// done, a step a release changed is updated in place, and a failure undoes what the install
    /// did. An app the machine has in another setup first loses that setup's own steps.
    ///
    /// # Errors
    ///
    /// [`EngineError`]; after a failure that wasn't a refusal before the first step, the machine
    /// is as it was before the install began, but for the steps of an installed app this run
    /// updated, which stay updated.
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
        let user_journal = Journal::read(self.user.journals(), id).map_err(EngineError::Journal)?;
        let root_journal = Journal::read(&self.root_journals, id).map_err(EngineError::Journal)?;
        let had = user_journal.as_ref().or(root_journal.as_ref()).map(|j| j.setup.clone());
        let switching = had.as_ref().is_some_and(|had| had.as_deref() != setup);
        let was_complete = !switching && user_journal.as_ref().is_some_and(|j| j.complete);
        let root_unsettled = self.unsettled(&folder, setup, root_journal.as_ref(), Side::Root);
        // An installed app whose every step stands changes nothing, so it needn't be ready: an
        // update checks it while the app is open.
        let stands = was_complete
            && user_journal.as_ref().is_some_and(|j| j.root_adoption.is_none())
            && !root_unsettled
            && !self.unsettled(&folder, setup, user_journal.as_ref(), Side::User);
        if !stands {
            self.ready(&folder, picked.map(|s| s.id.as_str()))?;
        }
        let journaled: BTreeSet<(Side, usize)> = [(Side::User, &user_journal), (Side::Root, &root_journal)]
            .into_iter()
            .flat_map(|(side, j)| {
                j.iter()
                    .flat_map(move |j| j.entries.iter().map(move |e| (side, e.step)))
            })
            .collect();
        let root_entries = root_journal.as_ref().is_some_and(|j| !j.entries.is_empty());
        let adopting = user_journal.as_ref().is_some_and(|j| j.root_adoption.is_some());
        // An app that isn't complete is rolled back whole on a failure, root's steps included.
        let needs_root = ((switching || !was_complete) && root_entries)
            || adopting
            || root_unsettled;
        if needs_root {
            self.start_root()?;
        }
        if adopting {
            self.adopt_root(id)?;
        }
        self.preflight(id, setup, needs_root)?;
        let mut finished = Finished::default();
        if switching {
            finished.switch |= self.switch_away(id, folder.actions.len())?;
        }
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
                // Root's steps were checked as settled before the run, as no root was asked for.
                Side::Root if !self.root_started => Ok(Outcome::Settled),
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
            let new = !journaled.contains(&(undoable.side, step));
            match outcome {
                Ok(Outcome::Settled) => self.report(id, step, total, &action.name, ActionState::Done),
                Ok(Outcome::Found) => self.report(id, step, total, &action.name, ActionState::Found),
                Ok(Outcome::Skipped { says }) => {
                    self.report(id, step, total, &action.name, ActionState::Skipped);
                    self.sink.event(&UpdateEvent::Note { text: says });
                }
                Ok(Outcome::Done | Outcome::Undone) => {
                    finished.switch |= undoable.kind_is_nix;
                    if new {
                        this_run.push(undoable);
                    }
                    self.report(id, step, total, &action.name, ActionState::Done);
                }
                Err(source) => {
                    if new {
                        this_run.push(undoable);
                    }
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

    /// Whether a step of `side` of the install isn't settled: its journal doesn't record it as
    /// settled with the same action, word for word, or its check, run from the user's side,
    /// doesn't say done. A root `run`'s check runs only as root, so such a step never is.
    fn unsettled(&mut self, folder: &Folder, setup: Option<&str>, journal: Option<&Journal>, side: Side) -> bool {
        let Ok(steps) = folder.steps(setup) else {
            return true;
        };
        let words = self.user.words();
        for (i, action) in steps.into_iter().enumerate() {
            if action.side() != side {
                continue;
            }
            let Ok(resolved) = action.resolve(&words, &folder.dir) else {
                return true;
            };
            let entry = journal.and_then(|j| j.entry(i + 1));
            let settled = entry
                .is_some_and(|e| e.action == resolved && e.state.settled(self.windows) && e.replaced.is_none());
            if !settled || (side == Side::Root && matches!(resolved.kind, Kind::Run { .. })) {
                return true;
            }
            // A step that opens a window, skipped in a run that may not, stays skipped.
            if entry.is_some_and(|e| e.state == State::Skipped) {
                continue;
            }
            if !perform::look(&resolved, self.user.host(), recorded(entry)).is_ok_and(|look| look.done) {
                return true;
            }
        }
        false
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
        if user_journal.as_ref().is_some_and(|j| j.root_adoption.is_some()) {
            self.adopt_root(id)?;
        }
        if root_journal.as_ref().is_some_and(|j| !j.entries.is_empty()) {
            self.start_root()?;
        }
        let user_journal = Journal::read(self.user.journals(), id).map_err(EngineError::Journal)?;
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

    /// Takes in the app `id` when the machine has it and no journal does: an app installed before
    /// KubaOS kept journals, by an older release of it or by hand. Nothing runs but checks. The
    /// app is there when every package step of its own actions (`apt`, `deb`, `nix`; every step,
    /// for an app with none) checks done; it is there in the first setup, by id, whose own steps
    /// that open no window all check done, else in the one with no steps of its own. Its user
    /// journal is then complete in that setup, its user steps recorded as [`Keeper::adopt`]
    /// says, `owned` when KubaOS put it there; root's steps are recorded by root on the first run
    /// that starts it, so taking an app in asks for no password. Returns the setup it was taken
    /// in, or none when it isn't there or has a journal already.
    ///
    /// # Errors
    ///
    /// [`EngineError`] when the folder or the journals can't be read, or a check can't run.
    pub fn adopt(&mut self, id: &str, owned: bool) -> Result<Option<Adopted>, EngineError> {
        let _lock = journal::lock(&self.lock).map_err(EngineError::Journal)?;
        if Journal::read(self.user.journals(), id).map_err(EngineError::Journal)?.is_some()
            || Journal::read(&self.root_journals, id).map_err(EngineError::Journal)?.is_some()
        {
            return Ok(None);
        }
        let folder = Folder::load(&self.user.catalog, id).map_err(EngineError::Folder)?;
        let words = self.user.words();
        let done = |actions: &[&Action], user: &mut Keeper<'_>| -> Result<bool, EngineError> {
            for action in actions {
                let resolved = action.resolve(&words, &folder.dir).map_err(|e| EngineError::Step {
                    name: action.name.clone(),
                    source: Box::new(StepError::Folder(FolderError::Action {
                        setup: None,
                        position: 0,
                        source: e,
                    })),
                })?;
                // A root `run`'s check runs only as root: from here it can't say done.
                let checkable = !(resolved.side() == Side::Root && matches!(resolved.kind, Kind::Run { .. }));
                let look = checkable
                    && perform::look(&resolved, user.host(), Recorded::Nothing)
                        .map_err(|e| EngineError::Step {
                            name: action.name.clone(),
                            source: Box::new(StepError::Perform(e)),
                        })?
                        .done;
                if !look {
                    return Ok(false);
                }
            }
            Ok(true)
        };
        let shared: Vec<&Action> = folder.actions.iter().collect();
        let packages: Vec<&Action> = shared
            .iter()
            .copied()
            .filter(|a| matches!(a.kind, Kind::Apt { .. } | Kind::Deb { .. } | Kind::Nix { .. }))
            .collect();
        let anchor = if packages.is_empty() { &shared } else { &packages };
        if !done(anchor, &mut self.user)? {
            return Ok(None);
        }
        let setup = if folder.has_setups() {
            let mut found = None;
            for setup in folder.setups.iter().filter(|s| !s.actions.is_empty()) {
                let own: Vec<&Action> = setup
                    .actions
                    .iter()
                    .filter(|a| !matches!(a.kind, Kind::Run { window: Some(_), .. }))
                    .collect();
                if done(&own, &mut self.user)? {
                    found = Some(setup.id.clone());
                    break;
                }
            }
            match found.or_else(|| folder.setups.iter().find(|s| s.actions.is_empty()).map(|s| s.id.clone())) {
                Some(setup) => Some(setup),
                None => return Ok(None),
            }
        } else {
            None
        };
        self.user
            .adopt(id, setup.as_deref(), owned)
            .map_err(|source| EngineError::Step {
                name: format!("Taking in {}", folder.name),
                source: Box::new(source),
            })?;
        let has_root = folder
            .steps(setup.as_deref())
            .map_err(EngineError::Folder)?
            .iter()
            .any(|a| a.side() == Side::Root);
        let mut journal = Journal::read(self.user.journals(), id)
            .map_err(EngineError::Journal)?
            .unwrap_or_else(|| Journal::new(id, setup.as_deref()));
        journal.setup.clone_from(&setup);
        journal.complete = true;
        journal.root_adoption = has_root.then_some(if owned { Adoption::Owned } else { Adoption::Found });
        journal.save(self.user.journals()).map_err(EngineError::Journal)?;
        Ok(Some(Adopted { setup }))
    }

    /// Has root record its steps of the app `id`, taken in on the user's side only so far,
    /// starting root when it isn't yet. Nothing to do for an app whose root half is recorded.
    fn adopt_root(&mut self, id: &str) -> Result<(), EngineError> {
        let Some(mut journal) = Journal::read(self.user.journals(), id).map_err(EngineError::Journal)? else {
            return Ok(());
        };
        let Some(adoption) = journal.root_adoption else {
            return Ok(());
        };
        self.start_root()?;
        let request = Request::Adopt {
            app: id.to_owned(),
            setup: journal.setup.clone(),
            owned: adoption.owned(),
        };
        let failed = |why: String| EngineError::Step {
            name: "Taking in what root has of the app".to_owned(),
            source: Box::new(StepError::RootFailed { why }),
        };
        match self.root.call(&request) {
            Ok(Reply::Ready) => {}
            Ok(Reply::Failed { why }) => return Err(failed(why)),
            Ok(Reply::Refused { why }) => return Err(failed(why.join("; "))),
            Ok(Reply::Step { .. }) => return Err(failed("the root helper answered out of turn".to_owned())),
            Err(e) => return Err(EngineError::Root(e)),
        }
        journal.root_adoption = None;
        journal.save(self.user.journals()).map_err(EngineError::Journal)
    }

    fn ready(&mut self, folder: &Folder, setup: Option<&str>) -> Result<(), EngineError> {
        let picked = setup.and_then(|s| folder.setup(s));
        let words = self.user.words();
        for ready in folder.readies(picked) {
            let check: Vec<String> = ready.check.iter().map(|a| words.fill(a, None)).collect();
            if self.user.host().run(&check, &folder.dir).is_err() {
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
            Err(RootError::Declined) => Err(EngineError::Declined),
            Err(e) => Err(EngineError::Root(e)),
        }
    }

    fn preflight(&mut self, id: &str, setup: Option<&str>, root: bool) -> Result<(), EngineError> {
        let refuse = |why: Vec<String>| EngineError::Refused {
            app: id.to_owned(),
            why,
        };
        let mut why = self.user.preflight(id, setup).map_err(|source| EngineError::Step {
            name: "Checking what the app would change".to_owned(),
            source: Box::new(source),
        })?;
        if root {
            match self.root.call(&Request::Preflight {
                app: id.to_owned(),
                setup: setup.map(str::to_owned),
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
        // Root is started here only for an undo of a root step the install found settled.
        if !self.root_started {
            self.root.start().map_err(StepError::RootLost)?;
            self.root_started = true;
        }
        match self.root.call(request) {
            Ok(Reply::Step { outcome }) => Ok(outcome),
            Ok(Reply::Failed { why }) => Err(StepError::RootFailed { why }),
            Ok(Reply::Refused { why }) => Err(StepError::RootRefused { why }),
            Ok(Reply::Ready) => Err(StepError::RootOutOfTurn),
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

    /// Reports `text` as a warning of the run.
    pub fn warn(&mut self, text: String) {
        self.sink.event(&UpdateEvent::Warning { text });
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
