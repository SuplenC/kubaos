//! The updater's side of the Apps catalog: `kubaos app add|remove|configure|status|adopt`, and the
//! phase of every update that installs the picked apps.
//!
//! Each app goes through the engine (`apps::engine`): the user's steps in this process, root's
//! through the root helper, `kubaos-apps serve`, started once a run before its first step:
//! `kubaos app` starts it through pkexec, one polkit prompt a run, and an update through sudo,
//! which the update holds already. An update opens no window, so a step such as pairing is a
//! note for the user. An app is installed with the apps it requires, first, and removed with
//! those it brought that nothing needs any more; neither is in the picks. An add or a remove that
//! fails puts the picks back. An app the machine has
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
use crate::apps::Catalog;
use crate::apps::Folder;
use crate::apps::FolderSetup;
use crate::apps::Picks;
use crate::apps::Platform;
use crate::apps::Status;
use crate::apps::action::Side;
use crate::apps::engine::Engine;
use crate::apps::engine::EngineError;
use crate::apps::engine::Keeper;
use crate::apps::journal::Journal;
use crate::apps::status;
use crate::packages::Packages;
use crate::unfree::Unfree;
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

/// Adding, removing or configuring.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Change {
    Add,
    Remove,
    Configure,
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
            self.platform(),
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

    /// Runs the configure stage of the installed apps `ids` alone again: each configure step
    /// checked, one not done done. The picks don't change.
    ///
    /// # Errors
    ///
    /// [`UpdateError::NoApp`] for an id the catalog doesn't have, else the app that isn't
    /// installed, has nothing to configure, or the step that failed.
    pub fn configure_apps(&mut self, ids: &[String]) -> Result<(), UpdateError> {
        self.begin(Command::App);
        let result = self.try_change_apps(ids, Change::Configure, None);
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
        let chosen = chosen_setups(&catalog, ids, change, setup, self.platform())?;
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
                (Change::Configure, _) => {}
            }
        }
        let changed = picks != before;
        if changed {
            picks.write(&path).map_err(UpdateError::Picks)?;
        }

        let apps = self.phase(Phase::Apps, |u| {
            let done = u.run_changes(&catalog, (&before, &picks), ids, &chosen, change);
            if done.is_err() && changed {
                // The picks go back to what the machine has: an app whose removal was rolled
                // back is there and stays picked, and one removed before the failure stays out.
                let mut back = before.clone();
                if change == Change::Remove {
                    for id in ids {
                        if !u.has_journal(id) {
                            back.remove(id);
                        }
                    }
                }
                back.write(&path).map_err(UpdateError::Picks)?;
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
            held: self.j.held.clone(),
            firmware: Firmware::default(),
            restart: Restart::None,
            restart_reason: None,
            undo: None,
            log: self.j.log_path(),
        };
        self.j.emit(&UpdateEvent::Summary(summary));
        Ok(())
    }

    /// Whether either side has a journal of the app `id`: it is on the machine, or partly.
    fn has_journal(&self, id: &str) -> bool {
        [self.user.app_journals(), self.sys.app_journals()]
            .iter()
            .any(|dir| !matches!(Journal::read(dir, id), Ok(None)))
    }

    /// Installs, removes or configures each app of `ids`, in its `chosen` setup, through pkexec,
    /// taking in first an app the machine has with no journal (owned when the picks `before` the
    /// run have it). An install brings the apps it requires and a removal takes those it brought,
    /// each listed with it: `after` is the picks the run leaves, which say which apps the user
    /// picked. An app already as asked is listed too: the summary says what the machine has after
    /// the run, so a caller acting on a stale reading still learns the app is there or gone.
    fn run_changes(
        &mut self,
        catalog: &Catalog,
        (before, after): (&Picks, &Picks),
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
                let ran = match change {
                    Change::Add => engine
                        .add(id, setup.as_deref(), after)
                        .map_err(|source| app_error(format!("installing {name}"), source))?,
                    Change::Remove => engine
                        .remove_with(id, after)
                        .map_err(|source| app_error(format!("removing {}", app.name), source))?,
                    Change::Configure => vec![
                        engine
                            .configure(id)
                            .map_err(|source| app_error(format!("configuring {}", app.name), source))?,
                    ],
                };
                for ran in ran {
                    switch |= ran.finished.switch;
                    let list = match change {
                        Change::Add if ran.finished.removed => &mut changes.removed,
                        Change::Add => &mut changes.installed,
                        Change::Remove if ran.finished.left => &mut changes.left,
                        Change::Remove => &mut changes.removed,
                        Change::Configure => &mut changes.configured,
                    };
                    if !list.contains(&ran.name) {
                        list.push(ran.name);
                    }
                }
            }
            Ok(switch)
        })?;
        if switch {
            self.switch()?;
        }
        let (names, verb) = match change {
            Change::Add => (&changes.installed, "installed"),
            Change::Remove => (&changes.removed, "removed"),
            Change::Configure => (&changes.configured, "configured"),
        };
        if !names.is_empty() {
            self.j.detail(format!("{} {verb}", names.join(", ")));
        }
        if change == Change::Add && !changes.removed.is_empty() {
            self.j.detail(format!("{} removed", changes.removed.join(", ")));
        }
        for name in &changes.left {
            self.j.detail(crate::update::event::left_line(name));
        }
        Ok(changes)
    }

    /// Installs the picked apps, as an update's phase, before the switch that installs an app's
    /// nixpkgs packages: each step of each checked, a step not done (a picked app the machine
    /// lacks, a step a release added or changed, one the user undid) done, through sudo, with no
    /// window. An app a picked setup no longer requires, and nothing else needs, goes, listed
    /// removed. Then the key of each vendor repository the apps set up that nears its expiry is
    /// fetched again, in the same root session, which a run with none due never starts. No picks
    /// start no phase. A failure is a warning: the next update tries again.
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
        let wanted = self.picked_apps(&catalog, &picks);
        if wanted.is_empty() {
            return AppChanges::default();
        }
        self.j.phase_start(Phase::Apps);
        let mut changes = AppChanges::default();
        let mut ok = true;
        let mut refreshed = Vec::new();
        let ran = self.with_engine(Root::Sudo, false, |engine| {
            for (app, setup, complete) in &wanted {
                let name = name_in(app, setup.as_deref()).to_owned();
                let done = engine
                    .adopt(&app.id, true, setup.as_deref())
                    .and_then(|adopted| {
                        let was = *complete || adopted.is_some();
                        engine.add(&app.id, setup.as_deref(), &picks).map(|f| (f, was))
                    })
                    .map_err(|source| app_error(format!("installing {name}"), source));
                match done {
                    Ok((ran, was)) => {
                        if !was {
                            changes.installed.push(name);
                        }
                        for gone in ran.into_iter().filter(|r| r.finished.removed) {
                            if !changes.removed.contains(&gone.name) {
                                changes.removed.push(gone.name);
                            }
                        }
                    }
                    Err(e) => {
                        ok = false;
                        engine_warning(engine, &e);
                    }
                }
            }
            match engine.refresh_keys() {
                Ok(repos) => refreshed = repos,
                Err(e) => {
                    ok = false;
                    engine.warn(format!(
                        "the app repositories' keys weren't checked: {}; the next update tries again",
                        chain(&e)
                    ));
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
        let mut details = Vec::new();
        if !changes.installed.is_empty() {
            details.push(format!("{} installed", changes.installed.join(", ")));
        }
        if !changes.removed.is_empty() {
            details.push(format!("{} removed", changes.removed.join(", ")));
        }
        if !refreshed.is_empty() {
            details.push(format!("the signing key of {} fetched again", refreshed.join(", ")));
        }
        if !details.is_empty() {
            self.j.detail(details.join("; "));
        }
        self.j.phase_end_optional(ok);
        changes
    }

    /// The apps of `catalog` an update installs for `picks`, each with its setup and whether its
    /// journal says it is complete; a pick the catalog lacks, one with no setup and one this
    /// machine can't have are warnings.
    fn picked_apps<'c>(&mut self, catalog: &'c Catalog, picks: &Picks) -> Vec<(&'c Folder, Option<String>, bool)> {
        for id in status::unknown(catalog, picks) {
            self.j.warn(format!(
                "{id} is picked in apps.toml, but this release's catalog has no such app"
            ));
        }
        let journals = self.user.app_journals();
        let platform = self.platform();
        let mut wanted = Vec::new();
        for app in catalog.apps().iter().filter(|app| picks.has(&app.id)) {
            let journal = Journal::read(&journals, &app.id).ok().flatten();
            let setup = status::picked_setup(app, picks, journal.as_ref());
            if app.has_setups() && setup.is_none() {
                let setups: Vec<&str> = app.setups.iter().map(|s| s.id.as_str()).collect();
                self.j.warn(format!(
                    "{id} is picked with no setup, so nothing installs it: kubaos app add {id} --setup <{setups}>",
                    id = app.id,
                    setups = setups.join("|"),
                ));
                continue;
            }
            if let Some(why) = unavailable(catalog, app, setup, platform) {
                self.j.warn(format!("{why}, so the update leaves it"));
                continue;
            }
            let complete = journal.is_some_and(|j| j.complete);
            wanted.push((app, setup.map(|s| s.id.clone()), complete));
        }
        wanted
    }

    /// This machine, with the setting of the user's nixpkgs for unfree packages.
    fn platform(&self) -> Platform<'static> {
        Platform::this_machine(self.user.state_dir().root())
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

/// Reports an app the update couldn't install as a warning, through the engine's sink. A refusal
/// stands at every update until the user settles what it names, so it asks for that first.
fn engine_warning(engine: &mut Engine<'_>, e: &UpdateError) {
    let refused = matches!(e, UpdateError::App { source, .. } if matches!(**source, EngineError::Refused { .. }));
    let then = if refused {
        ", then run kubaos update"
    } else {
        "; the next update tries again"
    };
    engine.warn(format!("{}{then}", chain(e)));
}

/// The setup each of `ids` is added in, by position: none for an app without setups. Checks
/// every id first, so nothing changes for a run that is refused.
fn chosen_setups(
    catalog: &Catalog,
    ids: &[String],
    change: Change,
    setup: Option<&str>,
    platform: Platform<'_>,
) -> Result<Vec<Option<String>>, UpdateError> {
    let mut chosen = Vec::with_capacity(ids.len());
    for id in ids {
        let app = catalog.get(id).ok_or_else(|| UpdateError::NoApp { id: id.clone() })?;
        if change != Change::Add {
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
        if let Some(why) = unavailable(catalog, app, picked, platform) {
            return Err(why);
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

/// Why `app` in `setup` can't install on `platform`: [`UpdateError::Unfree`] when letting unfree
/// packages in would do, else [`UpdateError::Unavailable`]; `None` when it can.
fn unavailable(
    catalog: &Catalog,
    app: &Folder,
    setup: Option<&FolderSetup>,
    platform: Platform<'_>,
) -> Option<UpdateError> {
    if catalog.available(app, setup, platform) {
        return None;
    }
    Some(
        if catalog.available(app, setup, Platform::new(platform.arch, Unfree::Allowed)) {
            UpdateError::Unfree { id: app.id.clone() }
        } else {
            UpdateError::Unavailable {
                id: app.id.clone(),
                arch: platform.arch.to_owned(),
            }
        },
    )
}
