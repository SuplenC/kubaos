//! Installing and removing an app of folders: its steps in order, each checked and journaled
//! before it runs, a failure rolled back, a removal undone from the journal and rolled back
//! when a step of it fails, a stopped run resumed, a step a release changed updated in place.
//!
//! A run has two sides. The user's [`Keeper`] runs in this process; root's runs in the root
//! helper (`kubaos-apps serve`), reached through a [`RootSide`], started once before the first
//! step so a declined password changes nothing. Both sides run the same [`Keeper`], each for its
//! own steps and its own journal.
//!
//! # Owns
//!
//! - [`Keeper`] (one side's steps, undos and who-owns-what checks), [`Engine`] (the order of an
//!   install and a removal, rollback, events), and their errors. Runs of more than one app, and the
//!   configure stage alone: `engine/requires.rs`.
//!
//! # Not here
//!
//! - The actions: `action.rs`. The journal's format: `journal.rs`. The root helper's requests:
//!   `root.rs`.

use std::collections::BTreeMap;
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
    #[error(
        "{} is yours, and KubaOS never writes over a file it didn't put there: delete or rename it",
        path.display()
    )]
    Yours { path: PathBuf },
    #[error("{key} belongs to {owner}")]
    KeyOwned { key: String, owner: String },
    #[error("step {step} of {app} is not one this side runs")]
    WrongSide { app: String, step: usize },
    #[error("{app} has no step {step}")]
    NoStep { app: String, step: usize },
    #[error("{name:?} is still a step of {app}'s install: only what a release took out of it is taken out")]
    StillAStep { app: String, name: String },
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
    #[error("{name}: it couldn't be undone, so the removal was rolled back and the app is as it was")]
    RemovalRolledBack {
        name: String,
        #[source]
        source: Box<StepError>,
    },
    #[error(
        "{name}: it couldn't be undone, and rolling the removal back left steps undone: the warnings name them, and kubaos update does them again"
    )]
    RemovalHalfRolledBack {
        name: String,
        #[source]
        source: Box<StepError>,
    },
    #[error("{app} can't be installed: {}", why.join("; "))]
    Refused {
        app: String,
        why: Vec<String>,
        /// One of `why` is a file of the user's own where a step writes, which stands until the
        /// user deletes or renames it.
        yours: bool,
    },
    #[error("{app} is needed by {}: remove that first", by.join(", "))]
    Needed { app: String, by: Vec<String> },
    #[error("{app} isn't installed")]
    NotInstalled { app: String },
    #[error("{app} has nothing to configure")]
    NothingToConfigure { app: String },
    #[error("{app} changed with this release: kubaos update brings it up to date, then configure it")]
    Outdated { app: String },
}

/// What stands in the way of one side's steps of an install, found before anything runs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Refusals {
    /// One sentence a refused step: the step's name, then why.
    pub why: Vec<String>,
    /// One of them is a file of the user's own where a step writes, which stands until the user
    /// deletes or renames it; the others (a path or key another app owns, a settings file that
    /// can't be read) are the catalog's or another app's.
    pub yours: bool,
}

/// One side's steps of the apps' installs: the user's in this process, root's in the root helper.
pub struct Keeper<'h> {
    host: &'h mut dyn Host,
    side: Side,
    catalog: PathBuf,
    journals: PathBuf,
    /// The entries this side undid in this run, as they were, newest last, with the claims each
    /// handed to another app: what a rolled-back removal does again. Kept here, never sent, so
    /// root's side trusts only its own record.
    undone: Vec<Undone>,
    /// The other side's journals, when this side may read them: root's, for the user's side.
    other_journals: Option<PathBuf>,
    /// What the user has to know of the steps this side ran or undid since the last
    /// [`Keeper::take_notes`], in order.
    notes: Vec<String>,
}

/// A step a removal undid, as its entry was, and the claims its undo handed to other apps.
#[derive(Clone, Debug, PartialEq)]
pub struct Undone {
    app: String,
    entry: Entry,
    handed: Vec<Handed>,
    /// A stale `run` with `undo` alone the removal forgot rather than ran, which a rollback
    /// leaves forgotten.
    forgotten: bool,
}

/// A claim an undo handed to the app that held it unowned, and that app's prior state before.
#[derive(Clone, Debug, PartialEq)]
struct Handed {
    holder: String,
    claim: String,
    prior: journal::Prior,
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
            undone: Vec::new(),
            other_journals: None,
            notes: Vec::new(),
        }
    }

    /// What the user has to know of the steps this side ran or undid since it was last asked:
    /// what a step that stands left to the user, what an undo left behind and why.
    pub fn take_notes(&mut self) -> Vec<String> {
        std::mem::take(&mut self.notes)
    }

    /// It, with `undone`, the steps this side undid earlier in the run, kept by a caller that
    /// makes a keeper a request.
    #[must_use]
    pub fn with_undone(mut self, undone: Vec<Undone>) -> Self {
        self.undone = undone;
        self
    }

    /// The steps this side undid in the run, for [`Keeper::with_undone`].
    pub fn take_undone(&mut self) -> Vec<Undone> {
        std::mem::take(&mut self.undone)
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
            version: None,
        }
    }

    /// This side's steps of the app `id` in `setup`, by number, resolved for this machine.
    fn steps(&self, id: &str, setup: Option<&str>) -> Result<(Folder, Vec<(usize, Action)>), StepError> {
        let folder = Folder::load(&self.catalog, id).map_err(StepError::Folder)?;
        let words = self.words().with_version(folder.version());
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

    /// This side's journal of the app `id`, numbered as the folder now numbers its steps, and
    /// written so when that renumbered it: a journal a release before wrote keeps its entries.
    fn journal(&self, id: &str) -> Result<Option<Journal>, StepError> {
        self.aligned_journal(id).map_err(StepError::Journal)
    }

    fn aligned_journal(&self, id: &str) -> Result<Option<Journal>, JournalError> {
        let was = Journal::read(&self.journals, id)?;
        let journal = aligned(&self.journals, id, &self.catalog, self.side)?;
        // Compared whole: a release that takes a step out can leave the same numbers, each now
        // another entry's.
        if let Some(journal) = &journal
            && was.as_ref() != Some(journal)
        {
            journal.save(&self.journals)?;
        }
        Ok(journal)
    }

    /// Checks, before anything runs, that no step of this side writes a path or a key another
    /// app's journal owns, or a path that is the user's own, and that every settings file a step
    /// changes is one it can read.
    ///
    /// # Errors
    ///
    /// The step that couldn't be resolved or the journals that couldn't be read; the refusals
    /// are the answer, one a step.
    pub fn preflight(&mut self, id: &str, setup: Option<&str>) -> Result<Refusals, StepError> {
        let (_, steps) = self.steps(id, setup)?;
        let owners = Owners::load(&[&self.journals]).map_err(StepError::Journal)?;
        let mut refused = Refusals::default();
        for (_, action) in steps {
            let writes = action.path().is_some()
                || matches!(
                    action.kind,
                    Kind::Setting { .. } | Kind::WebApp { .. } | Kind::Dkms { .. }
                );
            if !writes {
                continue;
            }
            let checked = perform::look(&action, &mut *self.host, Recorded::Nothing)
                .map_err(StepError::Perform)
                .and_then(|look| self.may_write(&owners, id, &look.claims));
            if let Err(e) = checked {
                refused.yours |= matches!(e, StepError::Yours { .. });
                refused
                    .why
                    .push(format!("{}: {}", action.name, crate::update::event::chain(&e)));
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
        let mut journal = self.journal(id)?.unwrap_or_else(|| Journal::new(id, setup));
        journal.setup = setup.map(str::to_owned);
        let mut old = journal.entry(step).cloned();
        if let Some(entry) = old.take_if(|e| !e.action.same_thing(&action)) {
            self.set_aside(&mut journal, id, &entry)?;
        }
        let recorded = self.recorded_for(&journal, &action, old.as_ref())?;
        let look = perform::look(&action, &mut *self.host, recorded).map_err(StepError::Perform)?;
        if let Kind::Run { window: Some(says), .. } = &action.kind
            && !windows
            && !look.done
        {
            self.skip(&mut journal, step, &action, old.as_ref())?;
            return Ok(Outcome::Skipped { says: says.clone() });
        }
        let (claims, replaced) = carry(old.as_ref(), &look);
        let prior = match &old {
            Some(e) if e.state.ours() => e.prior.clone(),
            _ => look.prior.clone(),
        };
        if look.done {
            let owns = claims.iter().any(|c| c.owned);
            // A step KubaOS did stays done though it owns nothing (a repository it set up for
            // packages that were there): the release's migrations read what it set up from it.
            let ours = owns || old.as_ref().is_some_and(|e| e.state == State::Done);
            let entry = Entry {
                step,
                state: if ours { State::Done } else { State::Found },
                claims,
                prior,
                action,
                replaced,
            };
            if old.as_ref() != Some(&entry) {
                journal.put(entry);
                journal.save(&self.journals).map_err(StepError::Journal)?;
            }
            self.say(&journal, step);
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
        let note = perform::apply(&action, &mut *self.host, &look, &mut |claims| {
            for claim in claims {
                if !entry.claims.contains(claim) {
                    entry.claims.push(claim.clone());
                }
            }
            journal.put(entry.clone());
            journal.save(journals)
        })
        .map_err(StepError::Perform)?;
        self.notes.extend(note);
        entry.state = State::Done;
        journal.put(entry);
        journal.save(&self.journals).map_err(StepError::Journal)?;
        self.say(&journal, step);
        Ok(Outcome::Done)
    }

    /// Records step `step`, `action`, which opens a window in a run that may not, as skipped,
    /// unless its entry `old` holds more.
    fn skip(&self, journal: &mut Journal, step: usize, action: &Action, old: Option<&Entry>) -> Result<(), StepError> {
        if old.is_none_or(|e| e.state == State::Skipped) {
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
        Ok(())
    }

    /// Keeps what the user has to know of step `step` of `journal`, which stands, for the run's
    /// notes. One the machine can't be asked for is left unsaid: the step stands all the same.
    fn say(&mut self, journal: &Journal, step: usize) {
        if let Some(entry) = journal.entry(step)
            && let Ok(Some(note)) = perform::says(&entry.action, &mut *self.host)
        {
            self.notes.push(note);
        }
    }

    /// Records the steps of this side of the app `id` in `setup` that check done, without
    /// running anything else, for an app the machine had before it had a journal. With `owned`
    /// (KubaOS put the app there), a step owns what its undo can take back as a machine without
    /// the app had it: its packages, paths, units (neither enabled nor running), settings keys
    /// (absent), group membership and `run`s, never a group itself, which a package makes;
    /// lingering only when KubaOS's older mark, `<id>-linger-was-off`, says it was off; and a
    /// path that is there and holds an older release's content, which the next run brings to
    /// this one's, as it sets up the repository of an `apt` step whose packages are there without
    /// it. Without `owned`, every step is found and owns nothing. A step with an entry already,
    /// and one not done (but for those two), is left to the next run. The mark goes
    /// once a `linger` step is recorded, whose prior state replaces it. Returns how many steps it
    /// recorded.
    ///
    /// # Errors
    ///
    /// The step that couldn't be resolved or checked, or the journal that couldn't be written.
    pub fn adopt(&mut self, id: &str, setup: Option<&str>, owned: bool) -> Result<usize, StepError> {
        let (_, steps) = self.steps(id, setup)?;
        let mut journal = self.journal(id)?.unwrap_or_else(|| Journal::new(id, setup));
        journal.setup = setup.map(str::to_owned);
        let mark = self.journals.join(format!("{id}{LINGER_MARK}"));
        let lingered_off = owned && mark.is_file();
        let mut recorded = 0;
        let mut lingering = false;
        for (step, action) in steps {
            if journal.entry(step).is_some() {
                continue;
            }
            let look = perform::look(&action, &mut *self.host, Recorded::Nothing).map_err(StepError::Perform)?;
            let stale = owned
                && !look.done
                && action.path().is_some()
                && look
                    .claims
                    .iter()
                    .any(|c| c.owned && c.path().is_some_and(|p| self.host.place(p).symlink_metadata().is_ok()));
            // A `run` that only undoes is the app's however it came: its undo is what a removal of
            // the app needs.
            if !stale && !perform::there(&action, &look) && !action.removal_only() {
                continue;
            }
            let (claims, prior) = if owned {
                adopted(&action, look, lingered_off)
            } else {
                (
                    look.claims.into_iter().map(|c| Claim::new(c.claim, false)).collect(),
                    look.prior,
                )
            };
            let settled = if claims.iter().any(|c| c.owned) {
                State::Done
            } else {
                State::Found
            };
            lingering |= matches!(action.kind, Kind::Linger {});
            journal.put(Entry {
                step,
                state: settled,
                claims,
                prior,
                action,
                replaced: None,
            });
            recorded += 1;
        }
        journal.save(&self.journals).map_err(StepError::Journal)?;
        // The mark is the only word on whether lingering was off until a linger entry holds it.
        if lingering && mark.is_file() {
            std::fs::remove_file(&mark).map_err(|e| {
                StepError::Journal(JournalError::Write {
                    path: mark.clone(),
                    source: e,
                })
            })?;
        }
        Ok(recorded)
    }

    /// Undoes what the step `step` replaced, once every step of the install stands, and forgets
    /// it; what another entry of the app holds of it passes to that entry ([`pass_within`]), so a
    /// package a release moved to a step of its own stays.
    fn finish_replaced(&mut self, journal: &mut Journal, id: &str, step: usize) -> Result<(), StepError> {
        let Some(mut entry) = journal.entry(step).cloned() else {
            return Ok(());
        };
        let Some(mut replaced) = entry.replaced.take() else {
            return Ok(());
        };
        pass_within(journal, &mut replaced);
        journal.put(Entry {
            replaced: Some(replaced.clone()),
            ..entry.clone()
        });
        journal.save(&self.journals).map_err(StepError::Journal)?;
        self.undo_entry(id, &replaced, &mut Vec::new(), true)?;
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
        let Some(mut journal) = self.journal(id)? else {
            return Ok(Outcome::Undone);
        };
        let Some(entry) = journal.entry(step).cloned() else {
            return Ok(Outcome::Undone);
        };
        // HAZARD: a `run` with `undo` alone the folder moved to the other side is forgotten, never
        // undone, here: the other side's entry runs it, and run twice it would be done twice. It
        // goes at the removal, as this side may not be started while the other records it.
        if self.moved_away(id, journal.setup.as_deref(), &entry) {
            journal.drop_step(step);
            journal.save(&self.journals).map_err(StepError::Journal)?;
            self.undone.push(Undone {
                app: id.to_owned(),
                entry,
                handed: Vec::new(),
                forgotten: true,
            });
            return Ok(Outcome::Undone);
        }
        let mut handed = Vec::new();
        let undone = self.undo_journaled(&mut journal, id, &entry, &mut handed, false);
        // Kept whether or not it went through: a claim handed over before a later part failed
        // goes back all the same.
        self.undone.push(Undone {
            app: id.to_owned(),
            entry,
            handed,
            forgotten: false,
        });
        undone?;
        Ok(Outcome::Undone)
    }

    /// Takes out of the app `id`'s journal what a release took out of its install, once every
    /// step of the install stands: the entry numbered `step`, when the install has no step of its
    /// name, undone and taken out; or, for a step the install has, what its changed action
    /// replaced. What another entry of the app holds passes to that entry instead of being undone
    /// ([`pass_within`]): a step a release merged into another leaves the machine as it is.
    ///
    /// # Errors
    ///
    /// [`StepError::StillAStep`] for an entry whose step the install still has, on either side,
    /// with nothing replaced: this side checks the folder itself, whatever the other side asked.
    /// Else as [`Keeper::undo`].
    pub fn take_out(&mut self, id: &str, step: usize) -> Result<Outcome, StepError> {
        let Some(mut journal) = self.journal(id)? else {
            return Ok(Outcome::Undone);
        };
        let Some(entry) = journal.entry(step).cloned() else {
            return Ok(Outcome::Undone);
        };
        let folder = Folder::load(&self.catalog, id).map_err(StepError::Folder)?;
        let steps = folder.steps(journal.setup.as_deref()).map_err(StepError::Folder)?;
        let still = || StepError::StillAStep {
            app: id.to_owned(),
            name: entry.action.name.clone(),
        };
        if step <= steps.len() {
            if entry.replaced.is_none() {
                return Err(still());
            }
            self.finish_replaced(&mut journal, id, step)?;
            return Ok(Outcome::Settled);
        }
        // HAZARD: a `run` with `undo` alone waits for the app's removal, whose undo it is.
        if entry.action.removal_only() || steps.iter().any(|a| a.name == entry.action.name) {
            return Err(still());
        }
        self.undo_journaled(&mut journal, id, &entry, &mut Vec::new(), true)?;
        Ok(Outcome::Undone)
    }

    /// Does step `step` of the app `id` in `setup` again after this run's [`Keeper::undo`] of
    /// it, for a removal rolled back: first the claims that undo handed to other apps go back to
    /// them, then the step's entry is put back as it was, still being undone, so the step done
    /// again owns what it owned; then it is done as [`Keeper::step`] does it, from the folder,
    /// with no window, so a step that opens one is left to the user as a note. A step this run
    /// didn't undo is only done.
    ///
    /// # Errors
    ///
    /// As [`Keeper::step`], or the journal that couldn't be written.
    pub fn redo(&mut self, id: &str, setup: Option<&str>, step: usize) -> Result<Outcome, StepError> {
        let recorded = self.undone.iter().rposition(|u| u.app == id && u.entry.step == step);
        let mut its_own = true;
        if let Some(at) = recorded {
            let undone = self.undone.remove(at);
            // A stale `run` with `undo` alone this side forgot: the other side's runs it now.
            if undone.forgotten {
                return Ok(Outcome::Settled);
            }
            for handed in undone.handed.iter().rev() {
                journal::give_back(&self.journals, &handed.holder, &handed.claim, &handed.prior)
                    .map_err(StepError::Journal)?;
            }
            let mut journal = self.journal(id)?.unwrap_or_else(|| Journal::new(id, setup));
            its_own = self.holds(id, setup, step, &undone.entry.action);
            if journal.entry(step).is_none() {
                // Being undone, the step done again keeps what it owned; one that owned nothing
                // keeps its state, so a step KubaOS did stays done.
                let owned = undone.entry.claims.iter().any(|c| c.owned);
                let state = match undone.entry.state {
                    State::Skipped => State::Skipped,
                    _ if owned => State::Undoing,
                    state => state,
                };
                journal.put(Entry { state, ..undone.entry });
                journal.save(&self.journals).map_err(StepError::Journal)?;
            }
        }
        // An entry the folder has no step for any more (a release took it out, renamed it or
        // changed it to another thing) is only put back, being undone: the next install takes
        // it out, or does its changed step over it.
        if !its_own {
            return Ok(Outcome::Settled);
        }
        self.step(id, setup, step, false)
    }

    /// Whether step `step` of the app `id`'s install in `setup` is this side's, of the name of
    /// `action`, an entry's, and acts on the same thing: the entry is the step's, not one a
    /// release took out, renamed or changed to another thing.
    fn holds(&self, id: &str, setup: Option<&str>, step: usize, action: &Action) -> bool {
        self.steps(id, setup).is_ok_and(|(_, steps)| {
            steps
                .iter()
                .any(|(n, now)| *n == step && now.name == action.name && now.same_thing(action))
        })
    }

    /// Sets the vendor repository of root's settled `apt` step `step` of the app `id` up again,
    /// as root's journal records it, so a key of it that nears its expiry is fetched again.
    ///
    /// # Errors
    ///
    /// [`StepError::NoStep`] when the journal has no such step, else the setup that failed.
    pub fn refresh(&mut self, id: &str, step: usize) -> Result<(), StepError> {
        let journal = Journal::read(&self.journals, id).map_err(StepError::Journal)?;
        let repo = journal
            .as_ref()
            .and_then(|j| j.entry(step))
            .and_then(set_up_repo)
            .map(str::to_owned)
            .ok_or_else(|| StepError::NoStep {
                app: id.to_owned(),
                step,
            })?;
        self.host
            .apt_repo(&repo)
            .map_err(|e| StepError::Perform(PerformError::Host(e)))
    }

    /// What `old`, the entry of `action`'s step in `journal`, says for its check; for a `run` with
    /// `undo` alone the journal doesn't record yet, found when its app is ([`Keeper::found`]).
    fn recorded_for<'e>(
        &self,
        journal: &Journal,
        action: &Action,
        old: Option<&'e Entry>,
    ) -> Result<Recorded<'e>, StepError> {
        if old.is_none() && action.removal_only() && self.found(journal)? {
            return Ok(Recorded::Found);
        }
        Ok(recorded(old))
    }

    /// Whether the app of `journal`, this side's, is found, as far as this side can read: every
    /// entry of it and of the other side's journal, when this side reads that, found and owning
    /// nothing, and at least one. A `run` with `undo` alone that a found app's journal doesn't
    /// record yet (a release added it, or moved it to this side) is found with it.
    fn found(&self, journal: &Journal) -> Result<bool, StepError> {
        let other = match &self.other_journals {
            Some(dir) => Journal::read(dir, &journal.app).map_err(StepError::Journal)?,
            None => None,
        };
        let entries: Vec<&Entry> = journal
            .entries
            .iter()
            .chain(other.iter().flat_map(|j| &j.entries))
            .collect();
        Ok(!entries.is_empty()
            && entries
                .iter()
                .all(|e| e.state == State::Found && e.claims.iter().all(|c| !c.owned)))
    }

    /// Whether `entry`, a `run` with `undo` alone of this side's journal of the app `id` in
    /// `setup`, is stale: the folder has its step, by its name, as the other side's `run` with
    /// `undo` alone, whose entry runs it, and no longer as this side's. Never when the folder
    /// can't be read.
    fn moved_away(&self, id: &str, setup: Option<&str>, entry: &Entry) -> bool {
        if !entry.action.removal_only() {
            return false;
        }
        let Ok(folder) = Folder::load(&self.catalog, id) else {
            return false;
        };
        let Ok(steps) = folder.steps(setup) else {
            return false;
        };
        let mine = steps.iter().find(|a| a.name == entry.action.name);
        mine.is_some_and(|a| a.side() != self.side && a.removal_only())
    }

    /// Takes `entry` out of `journal` for another thing its step now holds: undone, or, for a `run`
    /// that only undoes, forgotten.
    fn set_aside(&mut self, journal: &mut Journal, id: &str, entry: &Entry) -> Result<(), StepError> {
        if !entry.action.removal_only() {
            return self.undo_journaled(journal, id, entry, &mut Vec::new(), false);
        }
        // HAZARD: never undone here: the app stays, and its undo (Proton VPN's kill switch turned
        // off) is for a removal, which the step that now holds its name no longer asks for.
        journal.drop_step(entry.step);
        journal.save(&self.journals).map_err(StepError::Journal)
    }

    /// Marks `entry` as being undone, undoes it, and takes it out of `journal`; a run stopped
    /// during the undo leaves it marked, never looking settled. With `within`, what another entry
    /// of the journal holds passes to it first and isn't undone ([`pass_within`]).
    fn undo_journaled(
        &mut self,
        journal: &mut Journal,
        id: &str,
        entry: &Entry,
        handed: &mut Vec<Handed>,
        within: bool,
    ) -> Result<(), StepError> {
        let step = entry.step;
        let mut entry = entry.clone();
        if within {
            pass_within(journal, &mut entry);
        }
        journal.put(Entry {
            state: if entry.state == State::Skipped {
                State::Skipped
            } else {
                State::Undoing
            },
            ..entry.clone()
        });
        journal.save(&self.journals).map_err(StepError::Journal)?;
        self.undo_entry(id, &entry, handed, false)?;
        keep_brought(journal, &entry, handed);
        journal.drop_step(step);
        journal.save(&self.journals).map_err(StepError::Journal)
    }

    /// Undoes `entry` of the app `id`; a claim another app holds unowned passes to it instead,
    /// once the undo of the rest went through, each one handed over added to `handed`.
    fn undo_entry(
        &mut self,
        id: &str,
        entry: &Entry,
        handed: &mut Vec<Handed>,
        replacing: bool,
    ) -> Result<(), StepError> {
        // A found step owns nothing, unless another app's removal has handed it a claim since.
        if entry.state != State::Skipped {
            let owners = Owners::load(&[&self.journals]).map_err(StepError::Journal)?;
            let mut mine = Vec::new();
            let mut passing = Vec::new();
            for claim in entry.claims.iter().filter(|c| c.owned) {
                // A package one step pulled in, which another app's step found, passes to it.
                let held = match claim.claim.strip_prefix("dep:") {
                    Some(package) => format!("package:{package}"),
                    None => claim.claim.clone(),
                };
                match owners.holder(&held, id) {
                    Some(holder) => passing.push((holder.to_owned(), held)),
                    None => mine.push(claim.clone()),
                }
            }
            // HAZARD: handed over before an undo that then fails, a claim would be owned by both
            // apps, and the first to go would take it from the other.
            let brought = match entry.action.kind {
                Kind::Architecture { .. } => self.brought(id)?,
                _ => Vec::new(),
            };
            let undoing = perform::Undoing {
                brought: &brought,
                replacing,
            };
            let note = perform::undo(&entry.action, &mut *self.host, &mine, &entry.prior, undoing)
                .map_err(StepError::Perform)?;
            self.notes.extend(note);
            for (holder, held) in passing {
                if let Some(prior) =
                    journal::hand_over(&self.journals, &holder, &held, &entry.prior).map_err(StepError::Journal)?
                {
                    handed.push(Handed {
                        holder,
                        claim: held,
                        prior,
                    });
                }
            }
        }
        match &entry.replaced {
            Some(replaced) => self.undo_entry(id, replaced, handed, true),
            None => Ok(()),
        }
    }

    /// The packages the app `id`'s steps of this side installed or pulled in, by their owned
    /// `package:` and `dep:` claims and the `brought:` claims an undone step left on its
    /// `architecture` step: those its journal holds, and those of the steps this run undid, but
    /// the claims those undos handed to other apps.
    fn brought(&self, id: &str) -> Result<Vec<String>, StepError> {
        let journal = Journal::read(&self.journals, id).map_err(StepError::Journal)?;
        let handed: Vec<&str> = self
            .undone
            .iter()
            .filter(|u| u.app == id)
            .flat_map(|u| &u.handed)
            .map(|h| h.claim.as_str())
            .collect();
        let entries = journal
            .iter()
            .flat_map(|j| &j.entries)
            .chain(self.undone.iter().filter(|u| u.app == id).map(|u| &u.entry));
        let mut brought = Vec::new();
        for claim in entries
            .flat_map(|e| &e.claims)
            .filter(|c| c.owned || c.claim.starts_with("brought:"))
        {
            let Some(package) = claim
                .claim
                .strip_prefix("package:")
                .or_else(|| claim.claim.strip_prefix("dep:"))
                .or_else(|| claim.claim.strip_prefix("brought:"))
            else {
                continue;
            };
            if !handed.contains(&format!("package:{package}").as_str()) && !brought.iter().any(|b| b == package) {
                brought.push(package.to_owned());
            }
        }
        Ok(brought)
    }
}

/// Records on `journal`'s `architecture` steps the packages `entry`, a step about to leave the
/// journal, installed or pulled in and didn't hand to another app (`handed`), as `brought:<name>`
/// claims, which own nothing: the architecture's undo purges what of them is left, and the
/// journal keeps them, so a run stopped between the two undos leaves the next one knowing them.
fn keep_brought(journal: &mut Journal, entry: &Entry, handed: &[Handed]) {
    let packages: Vec<String> = entry
        .claims
        .iter()
        .filter(|c| {
            c.owned
                && !handed
                    .iter()
                    .any(|h| h.claim == c.claim.replacen("dep:", "package:", 1))
        })
        .filter_map(|c| {
            c.claim
                .strip_prefix("package:")
                .or_else(|| c.claim.strip_prefix("dep:"))
        })
        .map(|p| format!("brought:{p}"))
        .collect();
    if packages.is_empty() {
        return;
    }
    for arch in journal
        .entries
        .iter_mut()
        .filter(|e| matches!(e.action.kind, Kind::Architecture { .. }))
    {
        for claim in &packages {
            if !arch.claims.iter().any(|c| &c.claim == claim) {
                arch.claims.push(Claim::new(claim.clone(), false));
            }
        }
    }
}

/// The entry of `journal` other than step `step`'s that takes `claim` from a step a release took
/// out: one that holds it, or for a `dir:` claim, one whose path or folder is in that folder, as
/// its file stands in it.
fn taker(journal: &Journal, step: usize, claim: &str) -> Option<usize> {
    let dir = claim.strip_prefix("dir:").map(Path::new);
    let inside = |c: &Claim| {
        let path = c.path().or_else(|| c.claim.strip_prefix("dir:").map(Path::new));
        dir.is_some_and(|dir| path.is_some_and(|p| p.starts_with(dir)))
    };
    let others = || journal.entries.iter().filter(move |e| e.step != step);
    others()
        .find(|e| e.claims.iter().any(|c| c.claim == claim))
        .or_else(|| others().find(|e| e.claims.iter().any(inside)))
        .map(|e| e.step)
}

/// The claims of `entry` that another entry of `journal` may take: what it owns, but a `run`
/// claim, every run's own, and what apt pulled in, which goes with the package it came for.
fn passable(entry: &Entry) -> impl Iterator<Item = &Claim> {
    entry
        .claims
        .iter()
        .filter(|c| c.owned && c.claim != "run" && !c.claim.starts_with("dep:"))
}

/// Whether `entry`, a step a release took out, leaves `journal` with nothing to undo: another
/// entry takes everything it owns ([`pass_within`]).
fn folds(journal: &Journal, entry: &Entry) -> bool {
    let mut passable = passable(entry).peekable();
    passable.peek().is_some()
        && passable.all(|c| taker(journal, entry.step, &c.claim).is_some())
        && !entry.claims.iter().any(|c| c.owned && c.claim == "run")
}

/// Passes what `entry`, a step a release took out, owns and another entry of `journal` takes
/// ([`taker`]) to that entry, which owns it from then on with `entry`'s prior state, so the undo
/// leaves it: a package of two steps a release merged into one stays, as does the folder of a
/// file another step writes in it. What apt pulled in for the step (its `dep:` claims) goes to
/// the first entry that took a package, and only once every package it owned passed: a step of
/// which some package goes keeps them for its undo, which takes away those nothing needs.
fn pass_within(journal: &mut Journal, entry: &mut Entry) {
    let (step, prior) = (entry.step, entry.prior.clone());
    let takers: Vec<(String, usize)> = passable(entry)
        .filter_map(|c| taker(journal, step, &c.claim).map(|t| (c.claim.clone(), t)))
        .collect();
    let mut package_taker = None;
    for (claim, to) in &takers {
        let Some(other) = journal.entries.iter_mut().find(|e| e.step == *to) else {
            continue;
        };
        match other.claims.iter_mut().find(|c| c.claim == *claim) {
            Some(held) if !held.owned => {
                held.owned = true;
                other.prior.take_from(&prior);
            }
            Some(_) => {}
            None => other.claims.insert(0, Claim::new(claim.clone(), true)),
        }
        if let Some(c) = entry.claims.iter_mut().find(|c| c.claim == *claim) {
            c.owned = false;
        }
        if claim.starts_with("package:") {
            package_taker.get_or_insert(*to);
        }
    }
    let Some(taker) = package_taker else {
        return;
    };
    if entry.claims.iter().any(|c| c.owned && c.claim.starts_with("package:")) {
        return;
    }
    let deps: Vec<Claim> = entry
        .claims
        .iter_mut()
        .filter(|c| c.owned && c.claim.starts_with("dep:"))
        .map(|c| {
            c.owned = false;
            c.clone()
        })
        .collect();
    if let Some(other) = journal.entries.iter_mut().find(|e| e.step == taker) {
        for dep in deps {
            match other.claims.iter_mut().find(|c| c.claim == dep.claim) {
                Some(held) => held.owned = true,
                None => other.claims.push(Claim::new(dep.claim, true)),
            }
        }
    }
}

/// Whether a step of `kind` is what puts an app's program on the machine, whose being there says
/// the app is: a package, or a module DKMS builds.
fn puts_it_there(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::Apt { .. } | Kind::Deb { .. } | Kind::Nix { .. } | Kind::Dkms { .. }
    )
}

/// The vendor repository a journal's `entry` set up: a settled `apt` step's. Its check holds the
/// repository to being set up as KubaOS sets it up, so a step found done stands for one too.
fn set_up_repo(entry: &Entry) -> Option<&str> {
    match &entry.action.kind {
        Kind::Apt { repo: Some(repo), .. } if matches!(entry.state, State::Done | State::Found) => Some(repo),
        _ => None,
    }
}

/// The app `id`'s journal of `side` in `dir`, its entries numbered as the folder in `catalog` now
/// numbers the steps of their names in the setup it records ([`Journal::align`]); as it is
/// written when the folder or that setup can't be read. Every reader that matches entries to the
/// folder's steps by number reads it so, whichever side wrote it last.
///
/// # Errors
///
/// [`JournalError`] when the journal can't be read.
fn aligned(dir: &Path, id: &str, catalog: &Path, side: Side) -> Result<Option<Journal>, JournalError> {
    let Some(mut journal) = Journal::read(dir, id)? else {
        return Ok(None);
    };
    let Ok(folder) = Folder::load(catalog, id) else {
        return Ok(Some(journal));
    };
    let Ok(all) = folder.steps(journal.setup.as_deref()) else {
        return Ok(Some(journal));
    };
    let steps: Vec<(usize, &str)> = all
        .iter()
        .enumerate()
        .filter(|(_, action)| action.side() == side)
        .map(|(i, action)| (i + 1, action.name.as_str()))
        .collect();
    journal.align(&steps, all.len());
    Ok(Some(journal))
}

/// Whether `entry`, of an install whose steps are named `names`, numbered after its last step,
/// stays recorded until the app's removal rather than being taken out by an install: a `run`
/// with `undo` alone, whose undo is for the removal; one that owns nothing, found on the machine,
/// which has nothing to undo; and one of a step the install has on the other side, moved there.
fn lingers(entry: &Entry, names: &[&str]) -> bool {
    let owns = entry.claims.iter().any(|c| c.owned)
        || entry
            .replaced
            .as_ref()
            .is_some_and(|r| r.claims.iter().any(|c| c.owned));
    entry.action.removal_only() || !owns || names.contains(&entry.action.name.as_str())
}

/// The vendor repositories the root journals in `dir` record an `apt` step setting up, each with
/// the app and the step of the first that does.
///
/// # Errors
///
/// [`JournalError`] when a journal can't be read.
pub fn recorded_repos(dir: &Path) -> Result<BTreeMap<String, (String, usize)>, JournalError> {
    let mut repos = BTreeMap::new();
    for journal in Journal::all(dir)? {
        for entry in &journal.entries {
            if let Some(repo) = set_up_repo(entry) {
                repos
                    .entry(repo.to_owned())
                    .or_insert_with(|| (journal.app.clone(), entry.step));
            }
        }
    }
    Ok(repos)
}

/// The file KubaOS's older setups left beside the journals, `<id>-linger-was-off`, while
/// lingering was off before the app's service turned it on.
pub const LINGER_MARK: &str = "-linger-was-off";

/// The claims and prior state of a step of an app KubaOS put on the machine, adopted: owned as
/// [`Keeper::adopt`] says, with the prior state of a machine without the app. A foreign
/// architecture dpkg takes is never owned: no record says whether KubaOS added it, and a removal
/// that took it away from what the user installed for it would break their packages.
fn adopted(action: &Action, look: perform::Look, lingered_off: bool) -> (Vec<Claim>, journal::Prior) {
    let claims = look
        .claims
        .into_iter()
        .map(|c| {
            let owned = if ["group:", "dep:", "arch:"].iter().any(|p| c.claim.starts_with(p)) {
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
/// the path and the step is done; unknown, when a run stopped during it. A `run` that only undoes
/// has no path, and its entry is all its check reads: done and the app's, or found.
fn recorded(old: Option<&Entry>) -> Recorded<'_> {
    let Some(old) = old else {
        return Recorded::Nothing;
    };
    if old.action.removal_only() && old.state == State::Found {
        return Recorded::Found;
    }
    let undoes = old.action.removal_only() && old.claims.iter().any(|c| c.owned && c.claim == "run");
    if !undoes && !old.claims.iter().any(|c| c.owned && c.claim.starts_with("path:")) {
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
    /// Whether this run holds the lock already: a run of more than one app holds it from its plan
    /// to its last app, and each app's install or removal in it goes on under it.
    locked: bool,
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

/// The engine while a run holds the lock: dropping it, as the run ends or unwinds from a panic,
/// says the lock is no longer held, so a later run takes it again.
struct Held<'e, 'a>(&'e mut Engine<'a>);

impl<'a> std::ops::Deref for Held<'_, 'a> {
    type Target = Engine<'a>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl std::ops::DerefMut for Held<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl Drop for Held<'_, '_> {
    fn drop(&mut self) {
        self.0.locked = false;
    }
}

/// What a finished install or removal leaves for its caller to do.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Finished {
    /// The User flake's `packages.toml` changed: a switch installs or removes its names.
    pub switch: bool,
    /// A removal found the app owning nothing, as it was there before KubaOS: it forgot the app
    /// and left the machine as it is.
    pub left: bool,
    /// An install removed it: it was there only for the setup the app the run installed left.
    pub removed: bool,
}

/// Where an install starts from.
struct Begun {
    switching: bool,
    was_complete: bool,
    /// The steps either journal has, by side.
    journaled: BTreeSet<(Side, usize)>,
}

/// An app taken in by [`Engine::adopt`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Adopted {
    /// The setup the machine has it in.
    pub setup: Option<String>,
}

/// Where a failed removal is rolled back to.
struct Rollback<'f> {
    setup: Option<&'f str>,
    folder: Option<&'f Folder>,
    /// The highest step, for the report.
    total: usize,
    /// The user journal as the removal found it, before it marked it incomplete.
    header: Option<&'f Journal>,
}

/// One step of either side's journal, for an undo across both.
#[derive(Clone, Debug)]
struct Undoable {
    step: usize,
    side: Side,
    name: String,
    kind_is_nix: bool,
    kind_is_run: bool,
    /// A `run` that only undoes, which a rollback that keeps the app leaves recorded.
    removal_only: bool,
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
        let root_journals = root_journals.into();
        let mut user = user;
        user.other_journals = Some(root_journals.clone());
        Self {
            user,
            root,
            sink,
            root_journals,
            lock: lock.into(),
            windows,
            root_started: false,
            locked: false,
        }
    }

    /// Runs `go` holding the run lock, or under the lock this run holds already.
    fn locked<T>(&mut self, go: impl FnOnce(&mut Self) -> Result<T, EngineError>) -> Result<T, EngineError> {
        if self.locked {
            return go(self);
        }
        let _lock = journal::lock(&self.lock).map_err(EngineError::Journal)?;
        self.locked = true;
        let mut held = Held(self);
        go(&mut held)
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
        self.locked(|engine| engine.install_locked(id, setup))
    }

    fn install_locked(&mut self, id: &str, setup: Option<&str>) -> Result<Finished, EngineError> {
        let folder = Folder::load(&self.user.catalog, id).map_err(EngineError::Folder)?;
        folder.pick(setup).map_err(EngineError::Folder)?;
        let steps: Vec<Action> = folder
            .steps(setup)
            .map_err(EngineError::Folder)?
            .into_iter()
            .cloned()
            .collect();
        let Begun {
            switching,
            was_complete,
            journaled,
        } = self.begin(&folder, id, setup)?;
        let mut finished = Finished::default();
        if switching {
            finished.switch |= self.switch_away(id, &folder)?;
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
                Side::User => {
                    let outcome = self.user.step(id, setup, step, self.windows);
                    self.user_notes();
                    outcome
                }
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
                kind_is_run: matches!(action.kind, Kind::Run { .. }),
                removal_only: action.removal_only(),
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
        // After the steps, so what a step taken out shares with one that stands passes to it.
        let (switch, undid) = self.take_out(id, &steps)?;
        finished.switch |= switch;
        if undid {
            finished.switch |= self.check_again(id, setup, &steps)?;
        }
        let mut journal = Journal::read(self.user.journals(), id)
            .map_err(EngineError::Journal)?
            .unwrap_or_else(|| Journal::new(id, setup));
        journal.setup = setup.map(str::to_owned);
        journal.complete = true;
        journal.save(self.user.journals()).map_err(EngineError::Journal)?;
        self.forget_rollback(id);
        Ok(finished)
    }

    /// What an install starts from: whether it switches setups and whether the app was complete,
    /// with its ready checks passed when something will change, root started when a step needs
    /// it, and root's half of an adopted app recorded. Returns the steps either journal has.
    fn begin(&mut self, folder: &Folder, id: &str, setup: Option<&str>) -> Result<Begun, EngineError> {
        let user_journal = self.journal(Side::User, id)?;
        let root_journal = self.journal(Side::Root, id)?;
        let had = user_journal.as_ref().or(root_journal.as_ref()).map(|j| j.setup.clone());
        let switching = had.as_ref().is_some_and(|had| had.as_deref() != setup);
        // A switch undoes the old setup's own steps, so that setup has to be ready to go too.
        if switching {
            self.ready(folder, had.as_ref().and_then(Option::as_deref))?;
        }
        let was_complete = !switching && user_journal.as_ref().is_some_and(|j| j.complete);
        let root_unsettled = self.unsettled(folder, setup, root_journal.as_ref(), Side::Root, false);
        // An installed app whose every step stands changes nothing, so it needn't be ready: an
        // update checks it while the app is open. A step whose record only words the same thing
        // differently stands too: the step rewrites its record and leaves the machine as it is.
        let all = folder.steps(setup).map_err(EngineError::Folder)?;
        let names: Vec<&str> = all.iter().map(|a| a.name.as_str()).collect();
        let total = all.len();
        let taken_out = |journal: &Option<Journal>| {
            journal
                .iter()
                .flat_map(|j| &j.entries)
                .any(|e| e.replaced.is_some() || (e.step > total && !lingers(e, &names)))
        };
        let stands = was_complete
            && !taken_out(&user_journal)
            && !taken_out(&root_journal)
            && user_journal.as_ref().is_some_and(|j| j.root_adoption.is_none())
            && !self.unsettled(folder, setup, root_journal.as_ref(), Side::Root, true)
            && !self.unsettled(folder, setup, user_journal.as_ref(), Side::User, true);
        if !stands {
            self.ready(folder, setup)?;
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
        let needs_root =
            ((switching || !was_complete) && root_entries) || adopting || root_unsettled || taken_out(&root_journal);
        if needs_root {
            self.start_root()?;
        }
        if adopting {
            self.adopt_root(id)?;
        }
        self.preflight(id, setup, needs_root)?;
        Ok(Begun {
            switching,
            was_complete,
            journaled,
        })
    }

    /// Whether a step of `side` of the install isn't settled: its journal doesn't record it as
    /// settled with the same action, word for word, or its check, run from the user's side,
    /// doesn't say done. A root `run`'s check runs only as root, so such a step never is, but for
    /// one that only undoes, which its entry settles. With `same_thing`, a recorded action that
    /// acts on the same thing as the step's ([`Action::same_thing`]) will do in place of the same
    /// action word for word.
    fn unsettled(
        &mut self,
        folder: &Folder,
        setup: Option<&str>,
        journal: Option<&Journal>,
        side: Side,
        same_thing: bool,
    ) -> bool {
        let Ok(steps) = folder.steps(setup) else {
            return true;
        };
        let words = self.user.words().with_version(folder.version());
        for (i, action) in steps.into_iter().enumerate() {
            if action.side() != side {
                continue;
            }
            let Ok(resolved) = action.resolve(&words, &folder.dir) else {
                return true;
            };
            let entry = journal.and_then(|j| j.entry(i + 1));
            let recorded_as = |e: &Entry| {
                if same_thing {
                    e.action.same_thing(&resolved)
                } else {
                    e.action == resolved
                }
            };
            let settled =
                entry.is_some_and(|e| recorded_as(e) && e.state.settled(self.windows) && e.replaced.is_none());
            let root_check =
                side == Side::Root && matches!(resolved.kind, Kind::Run { .. }) && !resolved.removal_only();
            if !settled || root_check {
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
    /// journal is not installed, and nothing happens. An app whose journals own nothing (it was
    /// there by other means) is forgotten and the machine left as it is, which
    /// [`Finished::left`] says; with nothing run and no root asked for, unless root has a record
    /// of what it found, which only root can forget.
    ///
    /// # Errors
    ///
    /// [`EngineError`]. A removal is all or nothing, as an install is: a step that couldn't be
    /// undone stops it, and the steps it undid are done again, that step's with them, oldest
    /// first, as the current folder has them and with no window, so the app is as it was
    /// ([`EngineError::RemovalRolledBack`]; a mark beside the user journal records it, which
    /// [`journal::rolled_back`] reads). A step done again makes anew what it makes: a `run` runs
    /// its `do` again. One that can't be done again is a warning, the rest still go, and the app
    /// is left incomplete for the next update to finish ([`EngineError::RemovalHalfRolledBack`]).
    pub fn remove(&mut self, id: &str) -> Result<Finished, EngineError> {
        self.locked(|engine| engine.remove_locked(id))
    }

    fn remove_locked(&mut self, id: &str) -> Result<Finished, EngineError> {
        let user_journal = Journal::read(self.user.journals(), id).map_err(EngineError::Journal)?;
        let root_journal = Journal::read(&self.root_journals, id).map_err(EngineError::Journal)?;
        if user_journal.is_none() && root_journal.is_none() {
            self.forget_rollback(id);
            return Ok(Finished::default());
        }
        let owns = [&user_journal, &root_journal]
            .into_iter()
            .flatten()
            .any(Journal::owns_anything);
        // Root's record of what it found takes part in other apps' hand-overs, so only root can
        // forget it: an app with one goes through root, though nothing is undone.
        let root_records = root_journal.as_ref().is_some_and(|j| !j.entries.is_empty());
        if !owns && !root_records {
            Journal::forget(self.user.journals(), id).map_err(EngineError::Journal)?;
            self.forget_rollback(id);
            return Ok(Finished {
                left: true,
                ..Finished::default()
            });
        }
        let setup = user_journal
            .as_ref()
            .or(root_journal.as_ref())
            .and_then(|j| j.setup.clone());
        let folder = Folder::load(&self.user.catalog, id).ok();
        if let Some(folder) = &folder {
            self.ready(folder, setup.as_deref())?;
        }
        if user_journal.as_ref().is_some_and(|j| j.root_adoption.is_some()) {
            self.adopt_root(id)?;
        }
        if root_journal.as_ref().is_some_and(|j| !j.entries.is_empty()) {
            self.start_root()?;
        }
        let user_journal = Journal::read(self.user.journals(), id).map_err(EngineError::Journal)?;
        // What the user journal says of the app beyond its steps, which a rollback writes back:
        // saved with no steps and incomplete, as it is here for an app of root's steps alone, it
        // goes.
        let header = user_journal.clone();
        if let Some(mut journal) = user_journal {
            journal.complete = false;
            journal.save(self.user.journals()).map_err(EngineError::Journal)?;
        }
        let all = self.journaled(id)?;
        let total = all.iter().map(|u| u.step).max().unwrap_or(0);
        let count = all.len();
        let mut finished = Finished::default();
        let mut undone = Vec::new();
        for (done, undoable) in all.into_iter().enumerate() {
            self.sink.event(&UpdateEvent::Step {
                text: undoable.name.clone(),
            });
            self.sink.event(&UpdateEvent::Progress { done, total: count });
            self.report(id, undoable.step, total, &undoable.name, ActionState::Undoing);
            if let Err(source) = self.undo_one(id, &undoable) {
                self.report(id, undoable.step, total, &undoable.name, ActionState::Failed);
                let name = undoable.name.clone();
                undone.push(undoable);
                let back = Rollback {
                    setup: setup.as_deref(),
                    folder: folder.as_ref(),
                    total,
                    header: header.as_ref(),
                };
                return Err(self.roll_back_removal(id, &back, undone, name, source));
            }
            finished.switch |= undoable.kind_is_nix;
            self.report(id, undoable.step, total, &undoable.name, ActionState::Undone);
            undone.push(undoable);
        }
        self.sink.event(&UpdateEvent::Progress {
            done: count,
            total: count,
        });
        Journal::forget(self.user.journals(), id).map_err(EngineError::Journal)?;
        self.forget_rollback(id);
        finished.left = !owns;
        if let Some(folder) = &folder {
            let picked = setup.as_deref().and_then(|s| folder.setup(s));
            for keeps in folder.keeps(picked) {
                self.sink.event(&UpdateEvent::Note { text: keeps.to_owned() });
            }
        }
        Ok(finished)
    }

    /// Does again, oldest first, the steps a removal of the app `id` undid before the step
    /// `name` failed, that step with them, so the app is as it was before the removal. A mark
    /// beside the user journals records the rollback before the first step is done again, so a
    /// run stopped during it leaves the record. Each step is done again by its own side from that
    /// side's record of the undo ([`Keeper::redo`]), with no window; a step that can't be is a
    /// warning, and the rest still go. A `run` done again is a note, as what it makes is new.
    /// The user journal then gets back what it said beyond its steps, and is complete again when
    /// the app was and every step stands.
    fn roll_back_removal(
        &mut self,
        id: &str,
        back: &Rollback<'_>,
        mut undone: Vec<Undoable>,
        name: String,
        source: StepError,
    ) -> EngineError {
        let mut rolled = journal::RolledBack {
            whole: false,
            why: format!("{name}: {}", crate::update::event::chain(&source)),
        };
        // Marked as not whole until every step stands, so a run stopped during it says so.
        let mut recorded = journal::mark_rolled_back(self.user.journals(), id, Some(&rolled)).is_ok();
        self.sink.event(&UpdateEvent::Note {
            text: format!("{name} couldn't be undone, so I'm putting back what the removal took."),
        });
        undone.sort_by_key(|u| u.step);
        let mut whole = true;
        if back.folder.is_none() {
            whole = false;
            self.warn(format!(
                "{id} is gone from the catalog, so the steps its removal undid can't be done again"
            ));
        } else {
            for undoable in &undone {
                let outcome = match undoable.side {
                    Side::User => {
                        let outcome = self.user.redo(id, back.setup, undoable.step);
                        self.user_notes();
                        outcome
                    }
                    Side::Root => self.root_step(&Request::Redo {
                        app: id.to_owned(),
                        setup: back.setup.map(str::to_owned),
                        step: undoable.step,
                    }),
                };
                let state = match outcome {
                    Ok(Outcome::Found) => ActionState::Found,
                    Ok(Outcome::Skipped { says }) => {
                        self.sink.event(&UpdateEvent::Note { text: says });
                        ActionState::Skipped
                    }
                    // A `run` with `undo` alone ran nothing to be put back.
                    Ok(Outcome::Done) if undoable.kind_is_run && !undoable.removal_only => {
                        self.sink.event(&UpdateEvent::Note {
                            text: format!(
                                "{}: I ran it again to put it back, so what it makes is new.",
                                undoable.name
                            ),
                        });
                        ActionState::Done
                    }
                    Ok(_) => ActionState::Done,
                    Err(e) => {
                        whole = false;
                        self.warn(format!(
                            "{}: rolling the removal back, it couldn't be done again: {}",
                            undoable.name,
                            crate::update::event::chain(&e)
                        ));
                        ActionState::Failed
                    }
                };
                self.report(id, undoable.step, back.total, &undoable.name, state);
            }
        }
        let header = Journal::read(self.user.journals(), id).and_then(|journal| {
            let mut journal = journal.unwrap_or_else(|| Journal::new(id, back.setup));
            if let Some(header) = back.header {
                journal.setup.clone_from(&header.setup);
                journal.brought_by.clone_from(&header.brought_by);
                journal.root_adoption = header.root_adoption;
            }
            journal.complete = whole && back.header.is_some_and(|h| h.complete);
            journal.save(self.user.journals())
        });
        recorded &= header.is_ok();
        if whole {
            rolled.whole = true;
            recorded &= journal::mark_rolled_back(self.user.journals(), id, Some(&rolled)).is_ok();
        }
        if !recorded {
            self.warn(format!(
                "the rollback of {id}'s removal couldn't be recorded in its journal: kubaos app show {id} may not say it"
            ));
        }
        let source = Box::new(source);
        if whole {
            EngineError::RemovalRolledBack { name, source }
        } else {
            EngineError::RemovalHalfRolledBack { name, source }
        }
    }

    /// Takes away the mark that the app `id`'s last removal was rolled back, once an install or a
    /// removal went through; one that can't be is a warning, as the run itself is done.
    fn forget_rollback(&mut self, id: &str) {
        if let Err(e) = journal::mark_rolled_back(self.user.journals(), id, None) {
            self.warn(format!(
                "the mark that {id}'s last removal was rolled back couldn't be taken away: {}",
                crate::update::event::chain(&e)
            ));
        }
    }

    /// Takes in the app `id` when the machine has it and no journal does: an app installed before
    /// KubaOS kept journals, by an older release of it or by hand. Nothing runs but checks. The
    /// app is there when every package step of its own actions (`apt`, `deb`, `nix`, `dkms`; every
    /// step, for an app with none) checks done, an `apt` step's packages installed whether or
    /// not its repository is set up yet; it is there in the first setup, by id, whose own steps
    /// that open no window all check done, else in the one with no steps of its own. Its user
    /// journal is then complete in that setup, its user steps recorded as [`Keeper::adopt`]
    /// says, `owned` when KubaOS put it there; root's steps are recorded by root on the first run
    /// that starts it, so taking an app in asks for no password. An `owned` app whose `picked`
    /// setup the folder has is taken in that setup: the user picked it, and a step of it that
    /// doesn't check done is the next run's to do. Returns the setup it was taken in, or none
    /// when it isn't there or has a journal already.
    ///
    /// # Errors
    ///
    /// [`EngineError`] when the folder or the journals can't be read, or a check can't run.
    pub fn adopt(&mut self, id: &str, owned: bool, picked: Option<&str>) -> Result<Option<Adopted>, EngineError> {
        self.locked(|engine| engine.adopt_locked(id, owned, picked))
    }

    fn adopt_locked(&mut self, id: &str, owned: bool, picked: Option<&str>) -> Result<Option<Adopted>, EngineError> {
        if Journal::read(self.user.journals(), id)
            .map_err(EngineError::Journal)?
            .is_some()
            || Journal::read(&self.root_journals, id)
                .map_err(EngineError::Journal)?
                .is_some()
        {
            return Ok(None);
        }
        let folder = Folder::load(&self.user.catalog, id).map_err(EngineError::Folder)?;
        let words = self.user.words().with_version(folder.version());
        let done = |actions: &[&Action], user: &mut Keeper<'_>| -> Result<bool, EngineError> {
            for action in actions {
                // An app with nothing for this machine's architecture isn't on it.
                if !action.available(&words.arch) {
                    return Ok(false);
                }
                let resolved = action.resolve(&words, &folder.dir).map_err(|e| EngineError::Step {
                    name: action.name.clone(),
                    source: Box::new(StepError::Folder(FolderError::Action {
                        setup: None,
                        position: 0,
                        source: e,
                    })),
                })?;
                // A `run` that only undoes is no sign of the app either way, and the journal that
                // takes the app in makes it the app's.
                if resolved.removal_only() {
                    continue;
                }
                // A root `run`'s check runs only as root: from here it can't say done.
                let checkable = !(resolved.side() == Side::Root && matches!(resolved.kind, Kind::Run { .. }));
                let look = checkable && {
                    let look =
                        perform::look(&resolved, user.host(), Recorded::Nothing).map_err(|e| EngineError::Step {
                            name: action.name.clone(),
                            source: Box::new(StepError::Perform(e)),
                        })?;
                    perform::there(&resolved, &look)
                };
                if !look {
                    return Ok(false);
                }
            }
            Ok(true)
        };
        let shared: Vec<&Action> = folder.actions.iter().collect();
        let packages: Vec<&Action> = shared.iter().copied().filter(|a| puts_it_there(&a.kind)).collect();
        let anchor = if packages.is_empty() { &shared } else { &packages };
        if !done(anchor, &mut self.user)? {
            return Ok(None);
        }
        let picked = picked.filter(|setup| owned && folder.setup(setup).is_some());
        let setup = if let Some(picked) = picked {
            Some(picked.to_owned())
        } else if folder.has_setups() {
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
            match found.or_else(|| {
                folder
                    .setups
                    .iter()
                    .find(|s| s.actions.is_empty())
                    .map(|s| s.id.clone())
            }) {
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
            Ok(Reply::Refused { why, .. }) => return Err(failed(why.join("; "))),
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
        let refuse = |refused: Refusals| EngineError::Refused {
            app: id.to_owned(),
            why: refused.why,
            yours: refused.yours,
        };
        let mut refused = self.user.preflight(id, setup).map_err(|source| EngineError::Step {
            name: "Checking what the app would change".to_owned(),
            source: Box::new(source),
        })?;
        if root {
            match self.root.call(&Request::Preflight {
                app: id.to_owned(),
                setup: setup.map(str::to_owned),
            }) {
                Ok(Reply::Refused { why, yours }) => {
                    refused.why.extend(why);
                    refused.yours |= yours;
                }
                Ok(Reply::Failed { why }) => {
                    return Err(refuse(Refusals {
                        why: vec![why],
                        yours: false,
                    }));
                }
                Err(e) => return Err(EngineError::Root(e)),
                Ok(_) => {}
            }
        }
        if refused.why.is_empty() {
            Ok(())
        } else {
            Err(refuse(refused))
        }
    }

    fn root_step(&mut self, request: &Request) -> Result<Outcome, StepError> {
        // Root is started here only for an undo of a root step the install found settled.
        if !self.root_started {
            self.root.start().map_err(StepError::RootLost)?;
            self.root_started = true;
        }
        match self.root.call(request) {
            Ok(Reply::Step { outcome, notes }) => {
                for text in notes {
                    self.sink.event(&UpdateEvent::Note { text });
                }
                Ok(outcome)
            }
            Ok(Reply::Failed { why }) => Err(StepError::RootFailed { why }),
            Ok(Reply::Refused { why, .. }) => Err(StepError::RootRefused { why }),
            Ok(Reply::Ready) => Err(StepError::RootOutOfTurn),
            Err(e) => Err(StepError::RootLost(e)),
        }
    }

    /// Says what the user's side has to tell of the steps it ran or undid.
    fn user_notes(&mut self) {
        for text in self.user.take_notes() {
            self.sink.event(&UpdateEvent::Note { text });
        }
    }

    fn undo_one(&mut self, id: &str, undoable: &Undoable) -> Result<(), StepError> {
        match undoable.side {
            Side::User => {
                let undone = self.user.undo(id, undoable.step).map(|_| ());
                self.user_notes();
                undone
            }
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
        // HAZARD: a rollback that keeps the app keeps its `run` that only undoes recorded: its
        // undo is for the app's removal, and it changed nothing to take back.
        if !forget {
            undoables.retain(|u| !u.removal_only);
        }
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

    /// The app `id`'s journal of `side`, numbered as the folder now numbers its steps.
    fn journal(&self, side: Side, id: &str) -> Result<Option<Journal>, EngineError> {
        let dir = match side {
            Side::User => self.user.journals(),
            Side::Root => &self.root_journals,
        };
        aligned(dir, id, &self.user.catalog, side).map_err(EngineError::Journal)
    }

    /// Every step of both journals of the app `id`, newest first.
    fn journaled(&self, id: &str) -> Result<Vec<Undoable>, EngineError> {
        let mut all = Vec::new();
        for side in [Side::User, Side::Root] {
            if let Some(journal) = self.journal(side, id)? {
                all.extend(journal.entries.into_iter().map(|e| Undoable {
                    step: e.step,
                    side,
                    name: e.action.name.clone(),
                    kind_is_nix: matches!(e.action.kind, Kind::Nix { .. }),
                    kind_is_run: matches!(e.action.kind, Kind::Run { .. }),
                    removal_only: e.action.removal_only(),
                }));
            }
        }
        all.sort_by_key(|u| std::cmp::Reverse(u.step));
        Ok(all)
    }

    /// Takes away the steps of another setup than the one being installed: those after the app's
    /// own shared actions in `folder`. A `run` with `undo` alone a release took out stays, as its
    /// undo is for the app's removal. Whether `packages.toml` changed.
    fn switch_away(&mut self, id: &str, folder: &Folder) -> Result<bool, EngineError> {
        let mut switch = false;
        let shared = folder.shared_steps();
        let had = match self.journal(Side::User, id)? {
            Some(journal) => journal.setup,
            None => self.journal(Side::Root, id)?.and_then(|j| j.setup),
        };
        let had_total = folder.steps(had.as_deref()).map_or(usize::MAX, |steps| steps.len());
        let leaving: Vec<Undoable> = self
            .journaled(id)?
            .into_iter()
            .filter(|u| u.step > shared && !(u.removal_only && u.step > had_total))
            .collect();
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

    /// Takes out what a release took out of the app `id`'s install of `steps`, once its steps
    /// stand, each through [`Keeper::take_out`], so what a step that stands holds of it stays:
    /// first, newest first, the entries of names the install no longer has, but those that
    /// linger until the removal ([`lingers`]); then what each changed step replaced. A step taken
    /// out that passes everything it owned to steps that stand, a step a release merged into
    /// another, is no undo, and isn't reported as one; nor is what a changed step replaced, as
    /// the step stands. Whether `packages.toml` changed, and whether a step taken out was undone.
    fn take_out(&mut self, id: &str, steps: &[Action]) -> Result<(bool, bool), EngineError> {
        let names: Vec<&str> = steps.iter().map(|a| a.name.as_str()).collect();
        let total = steps.len();
        let mut gone = Vec::new();
        let mut replaced = Vec::new();
        for side in [Side::User, Side::Root] {
            let Some(journal) = self.journal(side, id)? else {
                continue;
            };
            for entry in &journal.entries {
                let undoable = Undoable {
                    step: entry.step,
                    side,
                    name: entry.action.name.clone(),
                    kind_is_nix: matches!(entry.action.kind, Kind::Nix { .. }),
                    kind_is_run: matches!(entry.action.kind, Kind::Run { .. }),
                    removal_only: entry.action.removal_only(),
                };
                if entry.step > total {
                    if !lingers(entry, &names) {
                        gone.push((undoable, !folds(&journal, entry)));
                    }
                } else if entry.replaced.is_some() {
                    replaced.push((undoable, false));
                }
            }
        }
        gone.sort_by_key(|(u, _)| std::cmp::Reverse(u.step));
        let of = gone.first().map_or(0, |(u, _)| u.step);
        let mut switch = false;
        let undid = gone.iter().any(|(_, reported)| *reported);
        for (undoable, reported) in gone.into_iter().chain(replaced) {
            if reported {
                self.report(id, undoable.step, of, &undoable.name, ActionState::Undoing);
            }
            let taken = match undoable.side {
                Side::User => {
                    let taken = self.user.take_out(id, undoable.step).map(drop);
                    self.user_notes();
                    taken
                }
                Side::Root => self
                    .root_step(&Request::TakeOut {
                        app: id.to_owned(),
                        step: undoable.step,
                    })
                    .map(drop),
            };
            taken.map_err(|source| EngineError::Undo {
                name: undoable.name.clone(),
                source: Box::new(source),
            })?;
            switch |= undoable.kind_is_nix;
            if reported {
                self.report(id, undoable.step, of, &undoable.name, ActionState::Undone);
            }
        }
        Ok((switch, undid))
    }

    /// Checks every step of the app `id`'s install in `setup` again once a step taken out was
    /// undone, and does again one whose check now fails: a step a release renamed was found done
    /// under its new name while its old entry still held it, and the old one's undo took it away.
    /// Root's steps are checked when root runs. Whether `packages.toml` changed.
    fn check_again(&mut self, id: &str, setup: Option<&str>, steps: &[Action]) -> Result<bool, EngineError> {
        let total = steps.len();
        let mut switch = false;
        for (i, action) in steps.iter().enumerate() {
            let step = i + 1;
            let outcome = match action.side() {
                Side::User => {
                    let outcome = self.user.step(id, setup, step, self.windows);
                    self.user_notes();
                    outcome
                }
                Side::Root if !self.root_started => continue,
                Side::Root => self.root_step(&Request::Step {
                    app: id.to_owned(),
                    setup: setup.map(str::to_owned),
                    step,
                    windows: self.windows,
                }),
            };
            match outcome {
                Ok(Outcome::Done) => {
                    switch |= matches!(action.kind, Kind::Nix { .. });
                    self.report(id, step, total, &action.name, ActionState::Done);
                }
                Ok(_) => {}
                Err(source) => {
                    self.report(id, step, total, &action.name, ActionState::Failed);
                    return Err(EngineError::Step {
                        name: action.name.clone(),
                        source: Box::new(source),
                    });
                }
            }
        }
        Ok(switch)
    }

    /// Fetches again the signing key of each vendor repository the apps' root journals record
    /// setting up whose key nears its expiry (`fetch_key`'s 30 days), by setting the repository up
    /// again through root. Which are due is read from the keyrings first, from the user's side,
    /// so a run with none due asks for no root. Returns the repositories whose key no longer
    /// nears its expiry; one that still does is a warning, for the next run to try again.
    ///
    /// # Errors
    ///
    /// [`EngineError::Journal`] when root's journals can't be read, [`EngineError::Declined`]
    /// when the password wasn't given, else the keyrings or the root helper that couldn't be
    /// reached.
    pub fn refresh_keys(&mut self) -> Result<Vec<String>, EngineError> {
        self.locked(Self::refresh_keys_locked)
    }

    fn refresh_keys_locked(&mut self) -> Result<Vec<String>, EngineError> {
        let recorded = recorded_repos(&self.root_journals).map_err(EngineError::Journal)?;
        let unread = |e| EngineError::Step {
            name: "Reading the apt repositories' keys".to_owned(),
            source: Box::new(StepError::Perform(PerformError::Host(e))),
        };
        let repos: Vec<String> = recorded.keys().cloned().collect();
        let due: Vec<String> = self
            .user
            .host()
            .apt_keys_due(&repos)
            .map_err(unread)?
            .into_iter()
            .collect();
        // The keys' script answers with repositories it was asked about; a line it printed
        // besides is no repository an app set up.
        let (due, stray): (Vec<String>, Vec<String>) = due.into_iter().partition(|r| recorded.contains_key(r));
        for line in stray {
            self.warn(format!(
                "the check of the apt repositories' keys answered with {line:?}, which no app's repository is"
            ));
        }
        if due.is_empty() {
            return Ok(Vec::new());
        }
        self.start_root()?;
        let mut failed = BTreeSet::new();
        for repo in &due {
            let Some((app, step)) = recorded.get(repo) else {
                continue;
            };
            let request = Request::Refresh {
                app: app.clone(),
                step: *step,
            };
            let why = match self.root.call(&request) {
                Ok(Reply::Ready) => continue,
                Ok(Reply::Failed { why }) => why,
                Ok(Reply::Refused { why, .. }) => why.join("; "),
                Ok(Reply::Step { .. }) => "the root helper answered out of turn".to_owned(),
                Err(e) => return Err(EngineError::Root(e)),
            };
            self.warn(format!(
                "the {repo} repository's signing key nears its expiry and couldn't be fetched again: {why}"
            ));
            failed.insert(repo.clone());
        }
        let still = self.user.host().apt_keys_due(&due).map_err(unread)?;
        for repo in still.iter().filter(|r| due.contains(*r) && !failed.contains(*r)) {
            self.warn(format!(
                "the {repo} repository's signing key nears its expiry, and its vendor has no newer one yet: \
                 the next update tries again"
            ));
        }
        Ok(due.into_iter().filter(|r| !still.contains(r)).collect())
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

mod requires;

pub use self::requires::Planned;
pub use self::requires::Ran;

#[cfg(test)]
mod tests;
