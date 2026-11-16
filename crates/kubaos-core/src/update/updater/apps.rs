//! The updater's side of the Apps catalog: `kubaos app add|remove|status|adopt`, and the phase of
//! every update that installs the picked apps.
//!
//! Each app goes through the engine (`apps::engine`): the user's steps in this process, root's
//! through the root helper, `kubaos-apps serve`, started once a run before its first step:
//! `kubaos app` starts it through pkexec, one polkit prompt a run, and an update through sudo,
//! which the update holds already. An update opens no window, so a step such as pairing is a
//! note for the user. An add or a remove that fails puts the picks back. An app the machine has
//! with no journal, installed before KubaOS kept journals, is taken in first, owned when it is
//! picked: KubaOS put it there.
//!
//! # Owns
//!
//! - Writing the picks as a run goes, running the engine with the root helper, and the Apps phase.
//!
//! # Not here
//!
//! - Installing, removing and taking in: `apps::engine`. Where each app stands: `apps::status`.

use std::ffi::OsString;

use super::UpdateError;
use super::Updater;
use super::release_of;
use crate::apps;
use crate::apps::Catalog;
use crate::apps::Folder;
use crate::apps::Picks;
use crate::apps::Status;
use crate::apps::action::Side;
use crate::apps::engine::Engine;
use crate::apps::engine::EngineError;
use crate::apps::engine::Keeper;
use crate::apps::journal::Journal;
use crate::apps::status;
use crate::packages::Packages;
use crate::update::changelog::Changelog;
use crate::update::event::AppChanges;
use crate::update::event::Command;
use crate::update::event::Phase;
use crate::update::event::Restart;
use crate::update::event::SuiteChanges;
use crate::update::event::Summary;
use crate::update::event::UpdateEvent;
use crate::update::event::chain;
use crate::update::firmware::Firmware;
use crate::update::machine::AppsHost;
use crate::update::machine::Cmd;
use crate::update::machine::Machine;

/// How the root helper gets root.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Root {
    /// pkexec, which asks through polkit: `kubaos app`, from a terminal or the Command menu.
    Polkit,
    /// sudo, which the update holds.
    Sudo,
}

/// Adding or removing.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Change {
    Add,
    Remove,
}

impl<M: Machine> Updater<'_, M> {
    /// Where each app of the checked-out release's catalog stands: picked or not, installed or
    /// not, from the picks and the journals, and its version from one dpkg query.
    ///
    /// # Errors
    ///
    /// [`UpdateError::Catalog`], [`UpdateError::Picks`] or [`UpdateError::AppJournal`] when the
    /// catalog, the picks or a journal can't be read, else the dpkg query that failed.
    pub fn app_status(&mut self) -> Result<Status, UpdateError> {
        let catalog = Catalog::load(&self.sys.catalog()).map_err(UpdateError::Catalog)?;
        let picks = Picks::read(&self.user.app_picks()).map_err(UpdateError::Picks)?;
        let dpkg = self.dpkg()?;
        Status::read(
            &catalog,
            &picks,
            &self.user.app_journals(),
            &self.sys.app_journals(),
            apps::arch(),
            Some(&dpkg),
        )
            .map_err(UpdateError::AppJournal)
    }

    /// Picks the apps `ids`, an app with setups in its setup `setup`, and installs them: every
    /// step checked, those not done done. An app the machine has in another setup switches.
    ///
    /// # Errors
    ///
    /// [`UpdateError::NoApp`] for an id the catalog doesn't have, [`UpdateError::NoSetup`] or
    /// [`UpdateError::UnknownSetup`] for an app with setups without that one,
    /// [`UpdateError::SetupWithout`] for a setup named for apps without setups,
    /// [`UpdateError::Unavailable`] for one with nothing for this machine, else the step that
    /// failed. The picks are as they were.
    pub fn add_apps(&mut self, ids: &[String], setup: Option<&str>) -> Result<(), UpdateError> {
        self.begin(Command::App);
        let result = self.try_change_apps(ids, Change::Add, setup);
        self.end(result)
    }

    /// Drops the apps `ids` from the picks and removes them: their journals undone.
    ///
    /// # Errors
    ///
    /// [`UpdateError::NoApp`] for an id the catalog doesn't have, else the step that failed. The
    /// picks are as they were.
    pub fn remove_apps(&mut self, ids: &[String]) -> Result<(), UpdateError> {
        self.begin(Command::App);
        let result = self.try_change_apps(ids, Change::Remove, None);
        self.end(result)
    }

    /// Takes in every app of the catalog the machine has with no journal, running nothing but
    /// checks and asking for no password: owned when it is picked, as KubaOS put a picked app
    /// there. Root's steps are recorded on the first run that starts root. Returns the apps taken
    /// in, by name. A user migration runs it once, so an updated machine counts what it has.
    ///
    /// # Errors
    ///
    /// [`UpdateError::Catalog`] or [`UpdateError::Picks`] when they can't be read, else the app
    /// that couldn't be taken in.
    pub fn adopt_apps(&mut self) -> Result<Vec<String>, UpdateError> {
        let catalog = Catalog::load(&self.sys.catalog()).map_err(UpdateError::Catalog)?;
        let picks = Picks::read(&self.user.app_picks()).map_err(UpdateError::Picks)?;
        let mut taken = Vec::new();
        self.with_engine(Root::Polkit, false, |engine| {
            for app in catalog.apps() {
                if let Some(adopted) = engine
                    .adopt(&app.id, picks.has(&app.id), picks.setup(&app.id))
                    .map_err(|source| app_error(format!("taking in {}", app.name), source))?
                {
                    taken.push(name_in(app, adopted.setup.as_deref()).to_owned());
                }
            }
            Ok(())
        })?;
        Ok(taken)
    }

    fn try_change_apps(&mut self, ids: &[String], change: Change, setup: Option<&str>) -> Result<(), UpdateError> {
        let (_lock, installed) = self.phase(Phase::Checking, |u| {
            let lock = u.lock()?;
            let installed = u.checked_installed()?;
            Ok((lock, installed))
        })?;
        let catalog = Catalog::load(&self.sys.catalog()).map_err(UpdateError::Catalog)?;
        let chosen = chosen_setups(&catalog, ids, change, setup)?;
        let path = self.user.app_picks();
        let before = Picks::read(&path).map_err(UpdateError::Picks)?;
        let mut picks = before.clone();
        for (id, setup) in ids.iter().zip(&chosen) {
            match (change, setup) {
                (Change::Add, Some(setup)) => picks.add_setup(id, setup),
                (Change::Add, None) => {
                    picks.add(id);
                }
                (Change::Remove, _) => {
                    picks.remove(id);
                }
            }
        }
        let changed = picks != before;
        if changed {
            picks.write(&path).map_err(UpdateError::Picks)?;
        }

        let apps = self.phase(Phase::Apps, |u| {
            let done = u.run_changes(&catalog, &before, ids, &chosen, change);
            if done.is_err() && changed {
                before.write(&path).map_err(UpdateError::Picks)?;
            }
            done
        })?;
        let summary = Summary {
            command: Command::App,
            from: release_of(&installed),
            to: release_of(&installed),
            channel: self.channel(&installed),
            changelog: Changelog::default(),
            packages: 0,
            upgraded: Vec::new(),
            apt: Vec::new(),
            apps,
            inputs: Vec::new(),
            nix_packages: None,
            migrations: Vec::new(),
            programs: Vec::new(),
            suites: SuiteChanges::default(),
            reloaded: self.j.reloaded.clone(),
            firmware: Firmware::default(),
            restart: Restart::None,
            restart_reason: None,
            undo: None,
            log: self.j.log_path(),
        };
        self.j.emit(&UpdateEvent::Summary(summary));
        Ok(())
    }

    /// Installs or removes each app of `ids`, in its `chosen` setup, through pkexec, taking in
    /// first an app the machine has with no journal (owned when `before` picks it). An app
    /// already as asked is listed too: the summary says what the machine has after the run, so a
    /// caller acting on a stale reading still learns the app is there or gone.
    fn run_changes(
        &mut self,
        catalog: &Catalog,
        before: &Picks,
        ids: &[String],
        chosen: &[Option<String>],
        change: Change,
    ) -> Result<AppChanges, UpdateError> {
        let mut changes = AppChanges::default();
        // A step that opens a window, pairing, waits for a session that has a display: over SSH
        // it is a note, and stays to do.
        let windows = self.m.has_display();
        let switch = self.with_engine(Root::Polkit, windows, |engine| {
            let mut switch = false;
            for (id, setup) in ids.iter().zip(chosen) {
                let Some(app) = catalog.get(id) else {
                    continue;
                };
                let name = name_in(app, setup.as_deref());
                engine
                    .adopt(id, before.has(id), before.setup(id))
                    .map_err(|source| app_error(format!("taking in {}", app.name), source))?;
                let finished = match change {
                    Change::Add => engine
                        .install(id, setup.as_deref())
                        .map_err(|source| app_error(format!("installing {name}"), source))?,
                    Change::Remove => engine
                        .remove(id)
                        .map_err(|source| app_error(format!("removing {}", app.name), source))?,
                };
                switch |= finished.switch;
                match change {
                    Change::Add => changes.installed.push(name.to_owned()),
                    Change::Remove if finished.left => changes.left.push(app.name.clone()),
                    Change::Remove => changes.removed.push(app.name.clone()),
                }
            }
            Ok(switch)
        })?;
        if switch {
            self.switch()?;
        }
        let names = match change {
            Change::Add => &changes.installed,
            Change::Remove => &changes.removed,
        };
        let verb = match change {
            Change::Add => "installed",
            Change::Remove => "removed",
        };
        if !names.is_empty() {
            self.j.detail(format!("{} {verb}", names.join(", ")));
        }
        for name in &changes.left {
            self.j.detail(crate::update::event::left_line(name));
        }
        Ok(changes)
    }

    /// Installs the picked apps, as an update's phase, before the switch that installs an app's
    /// nixpkgs packages: each step of each checked, a step not done (a picked app the machine
    /// lacks, a step a release added or changed, one the user undid) done, through sudo, with no
    /// window. No picks start no phase. A failure is a warning: the next update tries again.
    pub(super) fn reconcile_apps(&mut self) -> AppChanges {
        let picks = match Picks::read(&self.user.app_picks()) {
            Ok(picks) => picks,
            Err(e) => {
                self.j.warn(format!("no apps installed: {}", chain(&e)));
                return AppChanges::default();
            }
        };
        if picks.iter().next().is_none() {
            return AppChanges::default();
        }
        let catalog = match Catalog::load(&self.sys.catalog()) {
            Ok(catalog) => catalog,
            Err(e) => {
                self.j.warn(format!("no apps installed: {}", chain(&e)));
                return AppChanges::default();
            }
        };
        for id in status::unknown(&catalog, &picks) {
            self.j.warn(format!(
                "{id} is picked in apps.toml, but this release's catalog has no such app"
            ));
        }
        let journals = self.user.app_journals();
        let mut wanted: Vec<(&Folder, Option<String>, bool)> = Vec::new();
        for app in catalog.apps().iter().filter(|app| picks.has(&app.id)) {
            let journal = Journal::read(&journals, &app.id).ok().flatten();
            let setup = status::picked_setup(app, &picks, journal.as_ref());
            if app.has_setups() && setup.is_none() {
                let setups: Vec<&str> = app.setups.iter().map(|s| s.id.as_str()).collect();
                self.j.warn(format!(
                    "{id} is picked with no setup, so nothing installs it: kubaos app add {id} --setup <{setups}>",
                    id = app.id,
                    setups = setups.join("|"),
                ));
                continue;
            }
            if !app.available(apps::arch(), setup) {
                continue;
            }
            let complete = journal.is_some_and(|j| j.complete);
            wanted.push((app, setup.map(|s| s.id.clone()), complete));
        }
        if wanted.is_empty() {
            return AppChanges::default();
        }
        self.j.phase_start(Phase::Apps);
        let mut changes = AppChanges::default();
        let mut ok = true;
        let ran = self.with_engine(Root::Sudo, false, |engine| {
            for (app, setup, complete) in &wanted {
                let name = name_in(app, setup.as_deref()).to_owned();
                let done = engine
                    .adopt(&app.id, true, setup.as_deref())
                    .and_then(|adopted| {
                        let was = *complete || adopted.is_some();
                        engine.install(&app.id, setup.as_deref()).map(|f| (f, was))
                    })
                    .map_err(|source| app_error(format!("installing {name}"), source));
                match done {
                    Ok((_, was)) if !was => changes.installed.push(name),
                    Ok(_) => {}
                    Err(e) => {
                        ok = false;
                        engine_warning(engine, &e);
                    }
                }
            }
            Ok::<_, UpdateError>(())
        });
        if let Err(e) = ran {
            ok = false;
            self.j.warn(format!(
                "apps not installed, the next update tries again: {}",
                chain(&e)
            ));
        }
        if !changes.installed.is_empty() {
            self.j.detail(format!("{} installed", changes.installed.join(", ")));
        }
        self.j.phase_end_optional(ok);
        changes
    }

    /// Runs `work` with the engine: this user's side in this process, root's through the helper
    /// started the way `root` says, which may open a window when `windows` says so.
    fn with_engine<T>(
        &mut self,
        root: Root,
        windows: bool,
        work: impl FnOnce(&mut Engine<'_>) -> Result<T, UpdateError>,
    ) -> Result<T, UpdateError> {
        let helper = self.sys.apps_helper();
        let argv: Vec<OsString> = match root {
            Root::Polkit => vec!["pkexec".into(), helper.into_os_string(), "serve".into()],
            Root::Sudo => {
                let cmd = Cmd::root(&helper);
                self.authorise(&cmd)
                    .map_err(UpdateError::step("asking for the password"))?;
                vec!["sudo".into(), helper.into_os_string(), "serve".into()]
            }
        };
        let host = AppsHost {
            home: self.user.home().to_path_buf(),
            checkout: self.sys.checkout(),
            packages: Packages::new(&self.user.state_dir(), self.user.edit_lock()),
        };
        let (catalog, user_journals, root_journals, lock) = (
            self.sys.catalog(),
            self.user.app_journals(),
            self.sys.app_journals(),
            self.user.app_lock(),
        );
        let (mut host, mut root_side) = self.m.apps(&host, argv);
        let keeper = Keeper::new(&mut *host, Side::User, catalog, user_journals);
        let mut engine = Engine::new(keeper, &mut *root_side, root_journals, lock, &mut self.j, windows);
        work(&mut engine)
    }
}

/// The name `app` is installed under in `setup`: the setup's, else its own.
fn name_in<'a>(app: &'a Folder, setup: Option<&str>) -> &'a str {
    setup.and_then(|id| app.setup(id)).map_or(&app.name, |s| &s.name)
}

/// What a run of the engine that failed says: a declined password is the user's no.
fn app_error(what: String, source: EngineError) -> UpdateError {
    match source {
        EngineError::Declined => UpdateError::Declined,
        source => UpdateError::App {
            what,
            source: Box::new(source),
        },
    }
}

/// Reports an app the update couldn't install as a warning, through the engine's sink.
fn engine_warning(engine: &mut Engine<'_>, e: &UpdateError) {
    engine.warn(format!("{}; the next update tries again", chain(e)));
}

/// The setup each of `ids` is added in, by position: none for an app without setups. Checks
/// every id first, so nothing changes for a run that is refused.
fn chosen_setups(
    catalog: &Catalog,
    ids: &[String],
    change: Change,
    setup: Option<&str>,
) -> Result<Vec<Option<String>>, UpdateError> {
    let mut chosen = Vec::with_capacity(ids.len());
    for id in ids {
        let app = catalog.get(id).ok_or_else(|| UpdateError::NoApp { id: id.clone() })?;
        if change == Change::Remove {
            chosen.push(None);
            continue;
        }
        let setups = || app.setups.iter().map(|s| s.id.clone()).collect();
        let picked = match (app.has_setups(), setup) {
            (false, _) => None,
            (true, None) => {
                return Err(UpdateError::NoSetup {
                    id: id.clone(),
                    setups: setups(),
                });
            }
            (true, Some(wanted)) => Some(app.setup(wanted).ok_or_else(|| UpdateError::UnknownSetup {
                id: id.clone(),
                setup: wanted.to_owned(),
                setups: setups(),
            })?),
        };
        if !app.available(apps::arch(), picked) {
            return Err(UpdateError::Unavailable {
                id: id.clone(),
                arch: apps::arch().to_owned(),
            });
        }
        chosen.push(picked.map(|s| s.id.clone()));
    }
    if let Some(setup) = setup
        && chosen.iter().all(Option::is_none)
    {
        return Err(UpdateError::SetupWithout {
            setup: setup.to_owned(),
        });
    }
    Ok(chosen)
}
