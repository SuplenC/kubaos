//! The updater's side of the Apps catalog: `kubaos app add|remove|status`, and the phase of every
//! update that installs the picked apps the machine lacks.
//!
//! An app's root half (apt, a pinned `.deb`) goes through the root helper, given only app ids:
//! `kubaos app` runs it through pkexec, one polkit prompt a run, and an update through sudo,
//! which the update holds already. Its nixpkgs half is the User flake's `packages.toml`, as
//! `kubaos package add` writes it, and a switch: `kubaos app` switches itself, an update's own
//! switch comes after the phase. A setup's own parts are the user's: the user helper sets them up
//! and takes them away as the user, never as root. An add or a remove that fails puts the picks
//! back, so a declined prompt changes nothing; the record of the nixpkgs names each app added
//! (`apps::added`) keeps what was written to `packages.toml`, which is the apps' still.
//!
//! # Owns
//!
//! - Writing the picks and the record of added names as a run goes, running the root helper and
//!   the user helper, switching an app's setup, and the Apps phase.
//!
//! # Not here
//!
//! - What to install and remove: `apps::plan`. The formats: `apps`. The helpers:
//!   `system/files/apps/kubaos-apps` and `apps/user-helper`.

use std::collections::BTreeSet;

use super::UpdateError;
use super::Updater;
use super::release_of;
use crate::apps;
use crate::apps::Added;
use crate::apps::App;
use crate::apps::Catalog;
use crate::apps::Picks;
use crate::apps::Setup;
use crate::apps::Source;
use crate::apps::Status;
use crate::apps::plan;
use crate::apps::plan::Facts;
use crate::apps::plan::Install;
use crate::apt::Dpkg;
use crate::packages::Packages;
use crate::state::migrate::wire_home_nix;
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
use crate::update::machine::Cmd;
use crate::update::machine::Machine;

/// What the apps' plan reads.
struct AppsState {
    catalog: Catalog,
    picks: Picks,
    dpkg: Dpkg,
    nix: BTreeSet<String>,
    markers: BTreeSet<String>,
}

impl AppsState {
    fn facts(&self) -> Facts<'_> {
        Facts {
            catalog: &self.catalog,
            picks: &self.picks,
            arch: apps::arch(),
            dpkg: &self.dpkg,
            nix: &self.nix,
            markers: &self.markers,
        }
    }
}

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
    /// How the machine compares with the user's picks: every app of the checked-out release's
    /// catalog, picked or not, installed or not.
    ///
    /// # Errors
    ///
    /// [`UpdateError::Catalog`] or [`UpdateError::Picks`] when the catalog or the picks can't be
    /// read, else the dpkg query or the read of `packages.toml` that failed.
    pub fn app_status(&mut self) -> Result<Status, UpdateError> {
        let catalog = Catalog::load(&self.sys.catalog()).map_err(UpdateError::Catalog)?;
        let picks = Picks::read(&self.user.app_picks()).map_err(UpdateError::Picks)?;
        let state = self.apps_state(catalog, picks)?;
        Ok(Status::new(&state.facts()))
    }

    /// Picks the apps `ids`, an app with setups in its setup `setup`, and installs what the
    /// machine lacks of them: `kubaos app add`. An app the machine has in another setup switches:
    /// that setup's own parts go first.
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

    /// Drops the apps `ids` from the picks and removes those on the machine: `kubaos app remove`.
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
            let state = u.apps_state(catalog, picks)?;
            let done = match change {
                Change::Add => u.add_in_setups(state, ids),
                Change::Remove => u.uninstall_apps(&state, &plan::removals(&state.facts(), ids)).map(|done| (done, state)),
            };
            if done.is_err() && changed {
                before.write(&path).map_err(UpdateError::Picks)?;
            }
            let (mut done, state) = done?;
            // An app asked for that is already as asked is listed too: the summary says what
            // the machine has after the run, so a caller acting on a stale reading still learns
            // the app is there or gone.
            let facts = state.facts();
            for app in state.catalog.apps().iter().filter(|app| ids.contains(&app.id)) {
                let (already, list, name) = match change {
                    Change::Add => (
                        facts.state(app) == apps::AppState::Installed,
                        &mut done.installed,
                        facts.picked_setup(app).map_or(&app.name, |setup| &setup.name),
                    ),
                    Change::Remove => (
                        !facts.state(app).is_installed() && plan::parts_there(&facts, app).is_empty(),
                        &mut done.removed,
                        &app.name,
                    ),
                };
                if already && !list.contains(name) {
                    list.push(name.clone());
                }
            }
            Ok(done)
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

    /// Installs the picked apps the machine lacks, as an update's phase, before the switch that
    /// installs an app's nixpkgs packages. No picks, or nothing missing, starts no phase and runs
    /// nothing as root. A failure is a warning: the next update tries again.
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
        let found = Catalog::load(&self.sys.catalog())
            .map_err(UpdateError::Catalog)
            .and_then(|catalog| self.apps_state(catalog, picks));
        let state = match found {
            Ok(state) => state,
            Err(e) => {
                self.j.warn(format!("no apps installed: {}", chain(&e)));
                return AppChanges::default();
            }
        };
        for id in plan::unknown(&state.catalog, &state.picks) {
            self.j.warn(format!(
                "{id} is picked in apps.toml, but this release's catalog has no such app"
            ));
        }
        for app in plan::unsettled(&state.facts()) {
            let setups: Vec<String> = app.setups.iter().map(|setup| setup.id.clone()).collect();
            self.j.warn(UpdateError::NoSetup {
                id: app.id.clone(),
                setups,
            }
            .to_string());
        }
        let wanted = plan::installs(&state.facts());
        if wanted.is_empty() {
            return AppChanges::default();
        }
        self.j.phase_start(Phase::Apps);
        let done = self.install_apps(&state, &wanted, Root::Sudo);
        self.j.phase_end_optional(done.is_ok());
        done.unwrap_or_else(|e| {
            self.j.warn(format!(
                "apps not installed, the next update tries again: {}",
                chain(&e)
            ));
            AppChanges::default()
        })
    }

    /// Installs what the machine lacks of the apps `ids` as the picks in `state` have them, after
    /// taking away the parts of the setups an app leaves. Returns what it installed, and the state
    /// the machine is then in.
    fn add_in_setups(&mut self, state: AppsState, ids: &[String]) -> Result<(AppChanges, AppsState), UpdateError> {
        let facts = state.facts();
        let leaving: Vec<(&App, &Setup)> = state
            .catalog
            .apps()
            .iter()
            .filter(|app| ids.contains(&app.id))
            .filter_map(|app| Some((app, facts.picked_setup(app)?)))
            .flat_map(|(app, setup)| plan::leaving(&facts, app, setup).into_iter().map(move |other| (app, other)))
            .collect();
        let state = if leaving.is_empty() {
            state
        } else {
            self.remove_parts(&leaving)?;
            self.apps_state(state.catalog.clone(), state.picks.clone())?
        };
        let wanted: Vec<Install> = plan::installs(&state.facts())
            .into_iter()
            .filter(|install| ids.contains(&install.app.id))
            .collect();
        let done = self.install_apps(&state, &wanted, Root::Polkit)?;
        let after = self.apps_state(state.catalog.clone(), state.picks.clone())?;
        Ok((done, after))
    }

    /// Takes away the own parts of each setup of `leaving`, through the user helper.
    fn remove_parts(&mut self, leaving: &[(&App, &Setup)]) -> Result<(), UpdateError> {
        for (app, setup) in leaving {
            self.j.step(format!("Removing {}", setup.name));
            let helper = self.user_helper("remove", app, setup);
            self.run(&helper)
                .map_err(UpdateError::step(format!("removing {}", setup.name)))?;
        }
        Ok(())
    }

    /// Installs `wanted`: the root half in one run of the helper, the nixpkgs half into
    /// `packages.toml`, switched to at once when `root` is polkit, recording the names each app
    /// added, then each setup's own parts through the user helper. The names installed are those
    /// the machine then has.
    fn install_apps(&mut self, state: &AppsState, wanted: &[Install], root: Root) -> Result<AppChanges, UpdateError> {
        if wanted.is_empty() {
            self.j.detail("already installed");
            return Ok(AppChanges::default());
        }
        let names: Vec<&str> = wanted.iter().map(Install::name).collect();
        self.j.step(format!("Installing {}", names.join(", ")));
        let as_root: Vec<&str> = wanted
            .iter()
            .filter(|i| i.source && i.app.source.needs_root())
            .map(|i| i.app.id.as_str())
            .collect();
        if !as_root.is_empty() {
            let helper = self.helper(root).arg("install").args(&as_root);
            self.run(&helper).map_err(UpdateError::step("installing the apps"))?;
        }
        let sources: Vec<&App> = wanted.iter().filter(|i| i.source).map(|i| i.app).collect();
        let nix = nix_packages(&sources);
        if !nix.is_empty() {
            let changed = self.edit_nix(&nix, Change::Add)?;
            let path = self.user.apps_added();
            let mut added = Added::read(&path).map_err(UpdateError::AppsAdded)?;
            let before = added.clone();
            for app in &sources {
                if let Source::Nix { packages } = &app.source {
                    added.record(&app.id, packages.iter().filter(|p| changed.contains(*p)).cloned());
                }
            }
            if added != before {
                added.write(&path).map_err(UpdateError::AppsAdded)?;
            }
            if root == Root::Polkit {
                self.switch()?;
            }
        }
        for install in wanted {
            if let (true, Some(setup)) = (install.parts, install.setup) {
                self.j.step(format!("Setting up {}", setup.name));
                let helper = self.user_helper("install", install.app, setup);
                self.run(&helper)
                    .map_err(UpdateError::step(format!("setting up {}", setup.name)))?;
            }
        }
        let after = self.apps_state(state.catalog.clone(), state.picks.clone())?;
        let mut changes = AppChanges::default();
        for install in wanted {
            if after.facts().state(install.app) == apps::AppState::Installed {
                changes.installed.push(install.name().to_owned());
            } else {
                self.j
                    .warn(format!("{} isn't on the machine after its install", install.name()));
            }
        }
        self.j.detail(format!("{} installed", changes.installed.join(", ")));
        Ok(changes)
    }

    /// Removes `apps`: each setup's own parts that are there through the user helper, the root
    /// half in one run of the helper, and from `packages.toml` the nixpkgs names they added that
    /// no other picked app needs, then a switch.
    fn uninstall_apps(&mut self, state: &AppsState, apps: &[&App]) -> Result<AppChanges, UpdateError> {
        if apps.is_empty() {
            self.j.detail("not installed");
            return Ok(AppChanges::default());
        }
        let names: Vec<&str> = apps.iter().map(|a| a.name.as_str()).collect();
        let facts = state.facts();
        let parts: Vec<(&App, &Setup)> = apps
            .iter()
            .flat_map(|app| plan::parts_there(&facts, app).into_iter().map(move |setup| (*app, setup)))
            .collect();
        self.remove_parts(&parts)?;
        self.j.step(format!("Removing {}", names.join(", ")));
        let as_root: Vec<&str> = apps
            .iter()
            .filter(|a| a.source.needs_root() && facts.source_there(a))
            .map(|a| a.id.as_str())
            .collect();
        if !as_root.is_empty() {
            let helper = self.helper(Root::Polkit).arg("remove").args(&as_root);
            self.run(&helper).map_err(UpdateError::step("removing the apps"))?;
        }
        let path = self.user.apps_added();
        let mut added = Added::read(&path).map_err(UpdateError::AppsAdded)?;
        let before = added.clone();
        let nix: Vec<String> = plan::nix_released(&state.catalog, &state.picks, &mut added, apps)
            .into_iter()
            .collect();
        if !nix.is_empty() {
            self.edit_nix(&nix, Change::Remove)?;
        }
        if added != before {
            added.write(&path).map_err(UpdateError::AppsAdded)?;
        }
        if !nix.is_empty() {
            self.switch()?;
        }
        self.j.detail(format!("{} removed", names.join(", ")));
        Ok(AppChanges {
            installed: Vec::new(),
            removed: names.into_iter().map(str::to_owned).collect(),
        })
    }

    /// The user helper's `verb` of `app`'s setup `setup`, run as the user, for the home the
    /// updater works on.
    fn user_helper(&self, verb: &str, app: &App, setup: &Setup) -> Cmd {
        Cmd::user("bash")
            .arg(self.sys.apps_user_helper())
            .args([verb, app.id.as_str(), setup.id.as_str()])
            .env("HOME", self.user.home())
    }

    /// The helper, run as root the way `root` says.
    fn helper(&self, root: Root) -> Cmd {
        let helper = self.sys.apps_helper();
        match root {
            Root::Polkit => Cmd::user("pkexec").arg(helper),
            Root::Sudo => Cmd::root(helper),
        }
    }

    /// Adds `names` to the User flake's `packages.toml`, or removes them, wiring `home.nix` to it
    /// first as `kubaos package add` does. Returns the names it changed: on an add, those that
    /// weren't there yet.
    fn edit_nix(&mut self, names: &[String], change: Change) -> Result<BTreeSet<String>, UpdateError> {
        let home_nix = self.user.home_nix();
        wire_home_nix(&home_nix, &self.user.backups()).map_err(UpdateError::Wire)?;
        let packages = Packages::new(&self.user.state_dir(), self.user.edit_lock());
        match change {
            Change::Add => packages.add(names),
            Change::Remove => packages.remove(names),
        }
        .map(|done| done.changed.into_iter().collect())
        .map_err(UpdateError::NixPackages)
    }

    /// The facts the plan reads: dpkg's packages, the names in `packages.toml`, and the setups'
    /// markers in the home.
    fn apps_state(&mut self, catalog: Catalog, picks: Picks) -> Result<AppsState, UpdateError> {
        let dpkg = self.dpkg()?;
        let nix = Packages::new(&self.user.state_dir(), self.user.edit_lock())
            .list()
            .map_err(UpdateError::NixList)?
            .into_iter()
            .collect();
        let markers = catalog.markers_there(self.user.home());
        Ok(AppsState {
            catalog,
            picks,
            dpkg,
            nix,
            markers,
        })
    }
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

/// The nixpkgs names of `apps`, each once.
fn nix_packages(apps: &[&App]) -> Vec<String> {
    let names: BTreeSet<&String> = apps
        .iter()
        .filter_map(|a| match &a.source {
            Source::Nix { packages } => Some(packages),
            _ => None,
        })
        .flatten()
        .collect();
    names.into_iter().cloned().collect()
}
