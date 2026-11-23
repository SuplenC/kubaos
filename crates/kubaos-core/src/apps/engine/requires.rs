//! Runs of more than one app: an install with the apps it requires, first and in order, each
//! through its own folder and journal; a removal refused while an installed app needs the app,
//! with the apps it brought that nothing needs any more after it; and the configure stage alone.
//!
//! # Owns
//!
//! - [`Planned`], [`Ran`], the plan of an install, whether a required app's source meets a `min`,
//!   "brought by" in the dependencies' journals, and [`Engine::configure`].
//!
//! # Not here
//!
//! - One app's install and removal: `engine.rs`. What a valid requirement is: `folder.rs` and,
//!   across apps, `catalog.rs`.

use std::cmp::Ordering;

use super::Engine;
use super::EngineError;
use super::Finished;
use super::Outcome;
use super::StepError;
use super::Undoable;
use crate::apps::action::Action;
use crate::apps::action::Kind;
use crate::apps::action::Side;
use crate::apps::action::Stage;
use crate::apps::action::perform::PerformError;
use crate::apps::folder::Folder;
use crate::apps::folder::FolderError;
use crate::apps::host::HostError;
use crate::apps::journal::Journal;
use crate::apps::journal::JournalError;
use crate::apps::picks::Picks;
use crate::apps::version;
use crate::update::event::ActionState;
use crate::update::event::AppEvent;
use crate::update::event::UpdateEvent;

/// One app of an install, in the order it goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Planned {
    pub id: String,
    /// The setup it installs in; none for a required app, which has one way only.
    pub setup: Option<String>,
    /// Its name, or its setup's.
    pub name: String,
    /// The apps of the install that require it.
    pub required_by: Vec<String>,
}

/// One app a run installed, removed or configured, in the order it went.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ran {
    pub id: String,
    /// Its name as the run lists it: an install's, its setup's; a removal's and a configure's, the
    /// app's.
    pub name: String,
    pub finished: Finished,
}

impl Engine<'_> {
    /// The apps an install of `id` in `setup` installs, in order: what it requires first, each
    /// after what that requires, every app once, then the app. Nothing runs but the version
    /// queries of a `min`.
    ///
    /// # Errors
    ///
    /// [`EngineError::Refused`] for a required app the catalog doesn't have, one that requires
    /// the app back, or a `min` the required app's source can't be shown to meet; else the
    /// folder that can't be read.
    pub fn plan(&mut self, id: &str, setup: Option<&str>) -> Result<Vec<Planned>, EngineError> {
        let mut plan = Vec::new();
        self.plan_into(id, setup, None, &mut plan, &mut Vec::new())?;
        Ok(plan)
    }

    fn plan_into(
        &mut self,
        id: &str,
        setup: Option<&str>,
        by: Option<&str>,
        plan: &mut Vec<Planned>,
        visiting: &mut Vec<String>,
    ) -> Result<(), EngineError> {
        if let Some(planned) = plan.iter_mut().find(|p| p.id == id) {
            if let Some(by) = by
                && !planned.required_by.iter().any(|b| b == by)
            {
                planned.required_by.push(by.to_owned());
            }
            return Ok(());
        }
        if visiting.iter().any(|v| v == id) {
            return Err(EngineError::Refused {
                app: id.to_owned(),
                why: vec![format!("it requires itself through {}", visiting.join(", "))],
                yours: false,
            });
        }
        let folder = Folder::load(&self.user.catalog, id).map_err(EngineError::Folder)?;
        let picked = folder.pick(setup).map_err(EngineError::Folder)?;
        let requirements: Vec<_> = folder.requires(picked).cloned().collect();
        visiting.push(id.to_owned());
        for requirement in requirements {
            let required = match Folder::load(&self.user.catalog, requirement.id()) {
                Ok(required) => required,
                Err(FolderError::Read { .. }) => {
                    return Err(EngineError::Refused {
                        app: folder.name.clone(),
                        why: vec![format!(
                            "it requires {}, which the catalog doesn't have",
                            requirement.id()
                        )],
                        yours: false,
                    });
                }
                Err(e) => return Err(EngineError::Folder(e)),
            };
            if let Some(min) = requirement.min() {
                self.meets(&folder, &required, min)?;
            }
            self.plan_into(requirement.id(), None, Some(id), plan, visiting)?;
        }
        visiting.pop();
        plan.push(Planned {
            id: id.to_owned(),
            setup: setup.map(str::to_owned),
            name: picked.map_or(&folder.name, |s| &s.name).clone(),
            required_by: by.map(str::to_owned).into_iter().collect(),
        });
        Ok(())
    }

    /// Refuses `app`'s install when the version `required`'s source offers isn't `min` or newer.
    fn meets(&mut self, app: &Folder, required: &Folder, min: &str) -> Result<(), EngineError> {
        let offered = self.offered(required)?;
        let why = match offered {
            Some(offered) if version::compare(&offered, min) != Ordering::Less => return Ok(()),
            Some(offered) => format!(
                "it needs {} {min} or newer, and this machine would get {offered}",
                required.name
            ),
            None => format!(
                "it needs {} {min} or newer, and which version this machine would get can't be told",
                required.name
            ),
        };
        Err(EngineError::Refused {
            app: app.name.clone(),
            why: vec![why],
            yours: false,
        })
    }

    /// The version `folder`'s source offers this machine: its first own package's, a pinned
    /// `.deb`'s or download's version (the installed one, for a `.deb` that updates itself), an
    /// `apt` package's installed version, else the one apt would install. None for a nixpkgs
    /// package, or an app with no package.
    fn offered(&mut self, folder: &Folder) -> Result<Option<String>, EngineError> {
        let asked = |e: HostError| EngineError::Step {
            name: format!("Asking which version of {} this machine gets", folder.name),
            source: Box::new(StepError::Perform(PerformError::Host(e))),
        };
        for action in folder.actions.iter().filter(|a| a.stage == Stage::Install) {
            let host = self.user.host();
            match &action.kind {
                Kind::Deb {
                    package,
                    version,
                    self_updates,
                    ..
                } => {
                    let installed = if *self_updates {
                        host.version(package).map_err(asked)?
                    } else {
                        None
                    };
                    return Ok(Some(installed.unwrap_or_else(|| version.clone())));
                }
                Kind::Download { version, .. } => return Ok(Some(version.clone())),
                Kind::Apt { packages, .. } => {
                    let Some(first) = packages.first() else {
                        continue;
                    };
                    if let Some(installed) = host.version(first).map_err(asked)? {
                        return Ok(Some(installed));
                    }
                    return host.candidate(first).map_err(asked);
                }
                Kind::Nix { .. } => return Ok(None),
                _ => {}
            }
        }
        Ok(None)
    }

    /// Installs the app `id` in `setup` with every app it requires, those first, in the plan's
    /// order, each through [`Engine::install`] and reported by an [`AppEvent`], holding the run
    /// lock from the plan to the last of them. A required app the machine has with no journal is
    /// taken in first, owned when `picks` has it. Once all of them stand, each required app the
    /// user hasn't picked records the apps that brought it; a record that can't be written is a
    /// warning, as every app stands. A failure takes back the required apps this run installed,
    /// newest first, as well as the app's own steps. An app the machine has in another setup is
    /// checked ready to leave it before anything runs, and once it stands in `setup`, the apps it
    /// brought for the setup it left go, as [`Engine::leave`] says.
    ///
    /// # Errors
    ///
    /// As [`Engine::plan`] and the ready checks, before anything runs, and [`Engine::install`].
    pub fn add(&mut self, id: &str, setup: Option<&str>, picks: &Picks) -> Result<Vec<Ran>, EngineError> {
        self.locked(|engine| engine.add_locked(id, setup, picks))
    }

    fn add_locked(&mut self, id: &str, setup: Option<&str>, picks: &Picks) -> Result<Vec<Ran>, EngineError> {
        let plan = self.plan(id, setup)?;
        let left = self.leaving(id, setup, &plan)?;
        let of = plan.len();
        let mut ran = Vec::new();
        let mut brought = Vec::new();
        for (n, planned) in plan.iter().enumerate() {
            self.sink.event(&UpdateEvent::App(AppEvent {
                app: planned.id.clone(),
                name: planned.name.clone(),
                n: n + 1,
                of,
            }));
            let dependency = planned.id != id;
            let mut there = false;
            let done = if dependency {
                self.adopt(&planned.id, picks.has(&planned.id), picks.setup(&planned.id))
                    .and_then(|_| self.complete(&planned.id))
                    .map(|complete| there = complete)
            } else {
                Ok(())
            };
            let done = done.and_then(|()| self.install(&planned.id, planned.setup.as_deref()));
            match done {
                Ok(finished) => {
                    if dependency && !there {
                        brought.push(planned.id.clone());
                    }
                    ran.push(Ran {
                        id: planned.id.clone(),
                        name: planned.name.clone(),
                        finished,
                    });
                }
                Err(e) => {
                    self.take_back(&brought);
                    return Err(e);
                }
            }
        }
        for planned in plan.iter().filter(|p| p.id != id && !picks.has(&p.id)) {
            if let Err(e) = self.bring(&planned.id, &planned.required_by) {
                self.warn(format!(
                    "{} stays when the apps that need it go: {}",
                    planned.name,
                    crate::update::event::chain(&e)
                ));
            }
        }
        ran.extend(self.leave(id, &left, picks));
        Ok(ran)
    }

    /// The apps the setup the machine has the app `id` in requires and an install of it in
    /// `setup`, planned as `plan`, doesn't: none when the machine hasn't it in another setup. The
    /// app is checked ready in both setups, as the switch undoes the old one's steps.
    fn leaving(&mut self, id: &str, setup: Option<&str>, plan: &[Planned]) -> Result<Vec<String>, EngineError> {
        let Some(journal) = Journal::read(self.user.journals(), id).map_err(EngineError::Journal)? else {
            return Ok(Vec::new());
        };
        if journal.setup.as_deref() == setup {
            return Ok(Vec::new());
        }
        let folder = Folder::load(&self.user.catalog, id).map_err(EngineError::Folder)?;
        self.ready(&folder, setup)?;
        self.ready(&folder, journal.setup.as_deref())?;
        let had = journal.setup.as_deref().and_then(|s| folder.setup(s));
        Ok(folder
            .requires(had)
            .map(|r| r.id().to_owned())
            .filter(|required| plan.iter().all(|p| &p.id != required))
            .collect())
    }

    /// Lets go of the apps in `left`, which the setup the app `id` left required: each forgets
    /// that `id` brought it, and one that nothing with a journal brought besides, no installed app
    /// requires and `picks` hasn't goes, through [`Engine::remove_with`], each reported removed.
    /// One that can't be removed is a warning, and keeps `id` as what brought it, so the app's
    /// own removal takes it.
    fn leave(&mut self, id: &str, left: &[String], picks: &Picks) -> Vec<Ran> {
        let mut ran = Vec::new();
        for gone in left {
            let journals = match Journal::all(self.user.journals()) {
                Ok(journals) => journals,
                Err(e) => {
                    self.warn(format!("the apps left aside stay: {}", crate::update::event::chain(&e)));
                    return ran;
                }
            };
            let Some(mut journal) = journals.iter().find(|j| &j.app == gone).cloned() else {
                continue;
            };
            if !journal.brought_by.iter().any(|by| by == id) {
                continue;
            }
            // An app whose install stopped part way still needs what it brought: its next run
            // finishes the install on it.
            let journaled = |app: &str| journals.iter().any(|j| j.app == app);
            let kept = picks.has(gone)
                || journal.brought_by.iter().any(|by| by != id && journaled(by))
                || self.needing(gone, &[]).is_ok_and(|n| !n.is_empty());
            if kept {
                journal.brought_by.retain(|by| by != id);
                if let Err(e) = journal.save(self.user.journals()) {
                    self.warn(format!(
                        "{} stays when the apps that need it go: {}",
                        self.name_of(gone),
                        crate::update::event::chain(&e)
                    ));
                }
                continue;
            }
            match self.remove_with(gone, picks) {
                Ok(removed) => ran.extend(removed.into_iter().map(|r| Ran {
                    finished: Finished {
                        removed: true,
                        ..r.finished
                    },
                    ..r
                })),
                Err(e) => self.warn(format!(
                    "{} was there only for {}, and couldn't be removed: {}; kubaos app remove {gone} tries again",
                    self.name_of(gone),
                    self.name_of(id),
                    crate::update::event::chain(&e)
                )),
            }
        }
        ran
    }

    /// Records in the app `id`'s journal that `by` brought it.
    fn bring(&self, id: &str, by: &[String]) -> Result<(), JournalError> {
        let Some(mut journal) = Journal::read(self.user.journals(), id)? else {
            return Ok(());
        };
        let before = journal.brought_by.len();
        for app in by {
            if !journal.brought_by.contains(app) {
                journal.brought_by.push(app.clone());
            }
        }
        if journal.brought_by.len() == before {
            return Ok(());
        }
        journal.save(self.user.journals())
    }

    /// Removes the required apps of a failed install that the install put there, newest first;
    /// one that can't be removed is a warning.
    fn take_back(&mut self, brought: &[String]) {
        for id in brought.iter().rev() {
            if let Err(e) = self.remove(id) {
                self.warn(format!(
                    "{id}, installed for the app that failed, couldn't be removed: {}",
                    crate::update::event::chain(&e)
                ));
            }
        }
    }

    /// Whether the app `id`'s install is complete.
    fn complete(&self, id: &str) -> Result<bool, EngineError> {
        Ok(Journal::read(self.user.journals(), id)
            .map_err(EngineError::Journal)?
            .is_some_and(|j| j.complete))
    }

    /// Removes the app `id`, then each app it brought that no installed app requires any more
    /// and `picks` (the user's picks without `id`) doesn't have, after the apps that brought it,
    /// each reported by an [`AppEvent`].
    ///
    /// Once `id` is gone, an app it brought that can't be removed is a warning naming how to
    /// remove it, and the apps that app brought stay: the removal asked for is done.
    ///
    /// # Errors
    ///
    /// [`EngineError::Needed`] while an installed app requires `id`, before anything runs; else as
    /// [`Engine::remove`] of `id`.
    pub fn remove_with(&mut self, id: &str, picks: &Picks) -> Result<Vec<Ran>, EngineError> {
        self.locked(|engine| engine.remove_with_locked(id, picks))
    }

    fn remove_with_locked(&mut self, id: &str, picks: &Picks) -> Result<Vec<Ran>, EngineError> {
        let needing = self.needing(id, &[])?;
        if !needing.is_empty() {
            return Err(EngineError::Needed {
                app: self.name_of(id),
                by: needing,
            });
        }
        let going = self.going(id, picks)?;
        let of = going.len();
        let mut ran = Vec::new();
        let mut stayed: Vec<String> = Vec::new();
        for (n, gone) in going.iter().enumerate() {
            let name = self.name_of(gone);
            let brought_by_stayed = Journal::read(self.user.journals(), gone)
                .ok()
                .flatten()
                .is_some_and(|j| j.brought_by.iter().any(|by| stayed.contains(by)));
            if brought_by_stayed {
                stayed.push(gone.clone());
                continue;
            }
            self.sink.event(&UpdateEvent::App(AppEvent {
                app: gone.clone(),
                name: name.clone(),
                n: n + 1,
                of,
            }));
            let finished = match self.remove(gone) {
                Ok(finished) => finished,
                Err(e) if gone == id => return Err(e),
                Err(e) => {
                    self.warn(format!(
                        "{name} was there only for {}, and couldn't be removed: {}; kubaos app remove {gone} tries again",
                        self.name_of(id),
                        crate::update::event::chain(&e)
                    ));
                    stayed.push(gone.clone());
                    continue;
                }
            };
            if let Err(e) = self.forget_bringer(gone, &going) {
                self.warn(format!(
                    "the apps {name} brought couldn't forget it: {}",
                    crate::update::event::chain(&e)
                ));
            }
            ran.push(Ran {
                id: gone.clone(),
                name,
                finished,
            });
        }
        Ok(ran)
    }

    /// Takes the app `gone` out of the `brought_by` of every app staying, those not in `going`.
    fn forget_bringer(&self, gone: &str, going: &[String]) -> Result<(), JournalError> {
        for mut journal in Journal::all(self.user.journals())? {
            if going.contains(&journal.app) || !journal.brought_by.iter().any(|by| by == gone) {
                continue;
            }
            journal.brought_by.retain(|by| by != gone);
            journal.save(self.user.journals())?;
        }
        Ok(())
    }

    /// The apps a removal of `id` takes, in order: `id`, then each app that only apps going
    /// brought (or apps no longer installed), which `picks` doesn't have and no installed app
    /// staying requires.
    fn going(&self, id: &str, picks: &Picks) -> Result<Vec<String>, EngineError> {
        let journals = Journal::all(self.user.journals()).map_err(EngineError::Journal)?;
        let installed = |app: &str| journals.iter().any(|j| j.app == app && j.complete);
        let mut going = vec![id.to_owned()];
        loop {
            let next = journals.iter().find(|j| {
                !going.contains(&j.app)
                    && !picks.has(&j.app)
                    && !j.brought_by.is_empty()
                    && j.brought_by.iter().all(|by| going.contains(by) || !installed(by))
                    && self.needing(&j.app, &going).is_ok_and(|n| n.is_empty())
            });
            match next {
                Some(journal) => going.push(journal.app.clone()),
                None => return Ok(going),
            }
        }
    }

    /// The names of the installed apps, other than `id` and those of `going`, that require `id`
    /// in the setup they are installed in.
    fn needing(&self, id: &str, going: &[String]) -> Result<Vec<String>, EngineError> {
        let mut needing = Vec::new();
        for journal in Journal::all(self.user.journals()).map_err(EngineError::Journal)? {
            if !journal.complete || journal.app == id || going.contains(&journal.app) {
                continue;
            }
            let Ok(folder) = Folder::load(&self.user.catalog, &journal.app) else {
                continue;
            };
            let setup = journal.setup.as_deref().and_then(|s| folder.setup(s));
            if folder.requires(setup).any(|r| r.id() == id) {
                needing.push(setup.map_or(&folder.name, |s| &s.name).clone());
            }
        }
        Ok(needing)
    }

    /// The app `id`'s name, else its id.
    fn name_of(&self, id: &str) -> String {
        Folder::load(&self.user.catalog, id).map_or_else(|_| id.to_owned(), |folder| folder.name)
    }

    /// Runs the configure stage of the installed app `id` alone again: each of its configure
    /// steps checked, one not done done, as an install does them. A failure undoes the steps this
    /// run added; the app stays installed.
    ///
    /// # Errors
    ///
    /// [`EngineError::NotInstalled`] for an app whose install isn't complete,
    /// [`EngineError::NothingToConfigure`] for one with no configure stage,
    /// [`EngineError::Outdated`] for one whose journals record a step the folder has changed or
    /// taken out since, or miss an install step it added (an install brings it up to date
    /// first), else the step that failed.
    pub fn configure(&mut self, id: &str) -> Result<Ran, EngineError> {
        self.locked(|engine| engine.configure_locked(id))
    }

    fn configure_locked(&mut self, id: &str) -> Result<Ran, EngineError> {
        let folder = Folder::load(&self.user.catalog, id).map_err(EngineError::Folder)?;
        let journal =
            self.journal(Side::User, id)?
                .filter(|j| j.complete)
                .ok_or_else(|| EngineError::NotInstalled {
                    app: folder.name.clone(),
                })?;
        let setup = journal.setup.as_deref();
        let name = folder.name.clone();
        let steps = folder.steps(setup).map_err(EngineError::Folder)?;
        let of = steps.len();
        let root = self.journal(Side::Root, id)?;
        let words = self.user.words().with_version(folder.version());
        // A `run` with `undo` alone a release took out stays recorded after the last step until
        // the removal forgets it: it changes nothing the configure stage reads.
        let changed = std::iter::once(&journal)
            .chain(root.as_ref())
            .flat_map(|j| &j.entries)
            .filter(|entry| entry.step <= of || !entry.action.removal_only())
            .any(|entry| {
                let now = entry.step.checked_sub(1).and_then(|i| steps.get(i));
                now.is_none_or(|action| !action.resolve(&words, &folder.dir).is_ok_and(|a| a == entry.action))
            });
        let added = steps.iter().enumerate().any(|(i, action)| {
            let side = match action.side() {
                Side::User => Some(&journal),
                Side::Root => root.as_ref(),
            };
            action.stage == Stage::Install && side.is_none_or(|j| j.entry(i + 1).is_none())
        });
        let outdated = changed || added;
        if outdated {
            return Err(EngineError::Outdated { app: name });
        }
        let stage: Vec<(usize, Action)> = steps
            .into_iter()
            .enumerate()
            .filter(|(_, a)| a.stage == Stage::Configure)
            .map(|(i, a)| (i + 1, a.clone()))
            .collect();
        if stage.is_empty() {
            return Err(EngineError::NothingToConfigure { app: name });
        }
        self.sink.event(&UpdateEvent::App(AppEvent {
            app: id.to_owned(),
            name: name.clone(),
            n: 1,
            of: 1,
        }));
        let total = stage.len();
        let mut this_run = Vec::new();
        for (done, (step, action)) in stage.iter().enumerate() {
            self.sink.event(&UpdateEvent::Step {
                text: action.name.clone(),
            });
            self.sink.event(&UpdateEvent::Progress { done, total });
            let new = journal.entry(*step).is_none();
            let outcome = self.user.step(id, setup, *step, self.windows);
            self.user_notes();
            match outcome {
                Ok(Outcome::Found) => self.report(id, *step, of, &action.name, ActionState::Found),
                Ok(Outcome::Skipped { says }) => {
                    self.report(id, *step, of, &action.name, ActionState::Skipped);
                    self.sink.event(&UpdateEvent::Note { text: says });
                }
                Ok(outcome) => {
                    if new && matches!(outcome, Outcome::Done) {
                        this_run.push(undoable(*step, action));
                    }
                    self.report(id, *step, of, &action.name, ActionState::Done);
                }
                Err(source) => {
                    if new {
                        this_run.push(undoable(*step, action));
                    }
                    self.report(id, *step, of, &action.name, ActionState::Failed);
                    self.undo_all(id, of, this_run, false);
                    return Err(EngineError::Step {
                        name: action.name.clone(),
                        source: Box::new(source),
                    });
                }
            }
        }
        self.sink.event(&UpdateEvent::Progress { done: total, total });
        Ok(Ran {
            id: id.to_owned(),
            name,
            finished: Finished::default(),
        })
    }
}

/// A configure step to undo, which is the user's.
fn undoable(step: usize, action: &Action) -> Undoable {
    Undoable {
        step,
        side: action.side(),
        name: action.name.clone(),
        kind_is_nix: matches!(action.kind, Kind::Nix { .. }),
        kind_is_run: matches!(action.kind, Kind::Run { .. }),
        removal_only: action.removal_only(),
    }
}
