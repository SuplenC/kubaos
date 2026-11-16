//! How the machine compares with the user's picks: what `kubaos app status` shows, what an update
//! installs, and what a removal takes. Pure: the facts come in, read by the caller.
//!
//! An app is installed when everything its source names is on the machine: its apt packages or
//! its `.deb`'s package in dpkg, its nixpkgs names in the User flake's `packages.toml`. That is
//! the only record of it; KubaOS keeps no list of what is installed, only, beside it, which
//! nixpkgs names an app added, so a removal takes those and no name the user added. An update
//! installs each picked app the machine lacks, and a pinned `.deb` at another version than its pin,
//! unless the app updates itself. It removes nothing: an app goes only when the user removes it.
//!
//! An app with setups is installed in its picked setup when its source is and that setup's own
//! parts are, which their marker in the home says. The setup the machine has is the first whose
//! marker is there, else the one without parts of its own while the source is installed; a picked
//! app whose setup the picks don't name takes that one.
//!
//! # Owns
//!
//! - [`Facts`], [`AppState`], [`Status`], [`listing`], and the choices [`installs`], [`unsettled`],
//!   [`removals`], [`leaving`], [`parts_there`] and [`nix_released`].
//!
//! # Not here
//!
//! - Reading the facts and running the helper, apt or the switch: the updater's `apps.rs`.

use std::collections::BTreeSet;

use serde::Serialize;

use super::added::Added;
use super::catalog::Catalog;
use super::manifest::App;
use super::manifest::Setup;
use super::manifest::Source;
use super::picks::Picks;
use crate::apt::Dpkg;

/// What is known about one machine.
#[derive(Clone, Copy, Debug)]
pub struct Facts<'a> {
    pub catalog: &'a Catalog,
    pub picks: &'a Picks,
    /// As `dpkg --print-architecture` names it.
    pub arch: &'a str,
    pub dpkg: &'a Dpkg,
    /// The names in the User flake's `packages.toml`.
    pub nix: &'a BTreeSet<String>,
    /// The setups' markers that are there, as the manifests name them, under the home.
    pub markers: &'a BTreeSet<String>,
}

/// Where one app stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppState {
    Installed,
    /// Its pinned `.deb` is installed at another version, and it doesn't update itself: the next
    /// update installs the pin when it is picked.
    Outdated,
    /// Picked, and not on the machine: the next update installs it.
    Missing,
    /// Neither picked nor on the machine.
    Available,
    /// Its source has nothing for this machine's architecture, and it isn't there.
    Unavailable,
}

impl AppState {
    /// The state's word, as the table shows it.
    pub fn word(self) -> &'static str {
        match self {
            Self::Installed => "installed",
            Self::Outdated => "outdated",
            Self::Missing => "missing",
            Self::Available => "available",
            Self::Unavailable => "unavailable",
        }
    }

    /// Whether the app is on the machine, at whatever version.
    pub fn is_installed(self) -> bool {
        matches!(self, Self::Installed | Self::Outdated)
    }
}

impl Facts<'_> {
    /// Where `app` stands on the machine.
    pub fn state(&self, app: &App) -> AppState {
        let picked = self.picks.has(&app.id);
        let source = self.source(app);
        let setup_there = self.picked_setup(app).is_none_or(|setup| self.parts_there(setup));
        match source {
            Some(state) if setup_there => state,
            _ if !app.available(self.arch, self.picked_setup(app)) => AppState::Unavailable,
            _ if picked => AppState::Missing,
            _ => AppState::Available,
        }
    }

    /// Where `app`'s source stands, when it is on the machine: installed, or outdated.
    fn source(&self, app: &App) -> Option<AppState> {
        let installed = match &app.source {
            Source::Apt { packages, .. } => packages.iter().all(|p| self.dpkg.has(p)),
            Source::Nix { packages } => packages.iter().all(|p| self.nix.contains(p)),
            Source::Deb(deb) => match self.dpkg.get(&deb.package) {
                Some(installed) if !deb.self_updates && installed.version != deb.version => {
                    return Some(AppState::Outdated);
                }
                Some(_) => true,
                None => false,
            },
        };
        installed.then_some(AppState::Installed)
    }

    /// Whether `app`'s source is on the machine, at whatever version.
    pub fn source_there(&self, app: &App) -> bool {
        self.source(app).is_some()
    }

    /// Whether `setup`'s own parts are on the machine: its marker is there, or it has none.
    pub fn parts_there(&self, setup: &Setup) -> bool {
        setup
            .parts
            .as_ref()
            .is_none_or(|parts| self.markers.contains(&parts.marker))
    }

    /// The setup the machine has `app` in: the first whose own parts are there, else the one
    /// without parts while its source is. None while the app isn't on the machine.
    pub fn machine_setup<'b>(&self, app: &'b App) -> Option<&'b Setup> {
        if !self.source_there(app) {
            return None;
        }
        app.setups
            .iter()
            .find(|setup| setup.parts.is_some() && self.parts_there(setup))
            .or_else(|| app.setups.iter().find(|setup| setup.parts.is_none()))
    }

    /// The setup `app` is picked in: the one the picks name, else the one the machine has.
    pub fn picked_setup<'b>(&self, app: &'b App) -> Option<&'b Setup> {
        self.picks
            .setup(&app.id)
            .and_then(|id| app.setup(id))
            .or_else(|| self.machine_setup(app))
    }

    /// The installed version of `app`'s first apt or `.deb` package.
    fn version(&self, app: &App) -> Option<String> {
        let package = match &app.source {
            Source::Apt { packages, .. } => packages.first()?,
            Source::Deb(deb) => &deb.package,
            Source::Nix { .. } => return None,
        };
        self.dpkg.get(package).map(|i| i.version.clone())
    }
}

/// One picked app to install, and what of it the machine lacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Install<'a> {
    pub app: &'a App,
    /// The setup it is picked in, for an app with setups.
    pub setup: Option<&'a Setup>,
    /// Whether its source is to be installed: it isn't there, or a pinned `.deb` is at another
    /// version.
    pub source: bool,
    /// Whether its setup's own parts are to be installed.
    pub parts: bool,
}

impl Install<'_> {
    /// The name it is installed under: its setup's, else its own.
    pub fn name(&self) -> &str {
        self.setup.map_or(&self.app.name, |setup| &setup.name)
    }
}

/// The picked apps to install, in the catalog's order: those the machine lacks or lacks in their
/// picked setup, and those whose pinned `.deb` is at another version. An app with setups whose
/// setup isn't known is left out: [`unsettled`].
pub fn installs<'a>(facts: &Facts<'a>) -> Vec<Install<'a>> {
    facts
        .catalog
        .apps()
        .iter()
        .filter(|app| facts.picks.has(&app.id))
        .filter(|app| matches!(facts.state(app), AppState::Missing | AppState::Outdated))
        .filter_map(|app| {
            let setup = facts.picked_setup(app);
            if app.has_setups() && setup.is_none() {
                return None;
            }
            Some(Install {
                app,
                setup,
                source: facts.source(app) != Some(AppState::Installed),
                parts: setup.is_some_and(|setup| !facts.parts_there(setup)),
            })
        })
        .collect()
}

/// The picked apps with setups the machine lacks whose setup neither the picks nor the machine
/// say, which nothing installs until one is picked.
pub fn unsettled<'a>(facts: &Facts<'a>) -> Vec<&'a App> {
    facts
        .catalog
        .apps()
        .iter()
        .filter(|app| facts.picks.has(&app.id) && app.has_setups())
        .filter(|app| facts.picked_setup(app).is_none())
        .collect()
}

/// The apps of `ids` that are on the machine, any part of them, in the catalog's order.
pub fn removals<'a>(facts: &Facts<'a>, ids: &[String]) -> Vec<&'a App> {
    facts
        .catalog
        .apps()
        .iter()
        .filter(|app| ids.contains(&app.id))
        .filter(|app| facts.state(app).is_installed() || !parts_there(facts, app).is_empty())
        .collect()
}

/// The setups of `app` whose own parts are on the machine.
pub fn parts_there<'a>(facts: &Facts<'_>, app: &'a App) -> Vec<&'a Setup> {
    app.setups
        .iter()
        .filter(|setup| setup.parts.is_some() && facts.parts_there(setup))
        .collect()
}

/// The setups of `app` whose own parts go when it is set up as `setup` instead: every other
/// setup's that are on the machine. What the two share, the source, stays.
pub fn leaving<'a>(facts: &Facts<'_>, app: &'a App, setup: &Setup) -> Vec<&'a Setup> {
    parts_there(facts, app)
        .into_iter()
        .filter(|other| other.id != setup.id)
        .collect()
}

/// The nixpkgs names that go with `removed`: what each of them added, less a name another picked
/// app of the catalog needs, which then counts as that app's, so it goes with the last app that
/// needs it. A name the user had before any app added it was never recorded, and stays.
pub fn nix_released(catalog: &Catalog, picks: &Picks, added: &mut Added, removed: &[&App]) -> BTreeSet<String> {
    let mut released = BTreeSet::new();
    for app in removed {
        for name in added.take(&app.id) {
            let heir = catalog.apps().iter().find(|other| {
                picks.has(&other.id)
                    && !removed.iter().any(|r| r.id == other.id)
                    && matches!(&other.source, Source::Nix { packages } if packages.contains(&name))
            });
            match heir {
                Some(other) => added.record(&other.id, [name]),
                None => {
                    released.insert(name);
                }
            }
        }
    }
    released
}

/// One app of the status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AppStatus {
    pub id: String,
    pub name: String,
    pub summary: String,
    /// `apt`, `nix` or `deb`.
    pub source: &'static str,
    pub picked: bool,
    pub state: AppState,
    /// The installed version of its apt or `.deb` package.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The setup the machine has it in, while it is installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup: Option<String>,
    /// The setups it offers, by id; none for an app with one way only.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub setups: Vec<Setup>,
}

/// How the machine compares with the picks: `kubaos app status`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Status {
    pub arch: String,
    /// Every app of the catalog, by id.
    pub apps: Vec<AppStatus>,
    /// Picked ids the catalog doesn't have, such as an app a later release dropped.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unknown: Vec<String>,
}

impl Status {
    /// The status of the machine `facts` describes.
    pub fn new(facts: &Facts) -> Self {
        let apps = facts
            .catalog
            .apps()
            .iter()
            .map(|app| AppStatus {
                id: app.id.clone(),
                name: app.name.clone(),
                summary: app.summary.clone(),
                source: app.source.kind(),
                picked: facts.picks.has(&app.id),
                state: facts.state(app),
                version: facts.version(app),
                setup: facts.machine_setup(app).map(|setup| setup.id.clone()),
                setups: app.setups.clone(),
            })
            .collect();
        Self {
            arch: facts.arch.to_owned(),
            apps,
            unknown: unknown(facts.catalog, facts.picks),
        }
    }
}

/// One app of the catalog as `kubaos app list` shows it, with no fact of the machine, so the list
/// runs no program.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Listed<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub summary: &'a str,
    /// `apt`, `nix` or `deb`.
    pub source: &'static str,
    pub picked: bool,
    /// The setup the picks name for it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup: Option<&'a str>,
    /// The setups it offers, by id; none for an app with one way only.
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub setups: &'a [Setup],
}

/// Every app of `catalog`, by id, and whether it is picked.
pub fn listing<'a>(catalog: &'a Catalog, picks: &'a Picks) -> Vec<Listed<'a>> {
    catalog
        .apps()
        .iter()
        .map(|app| Listed {
            id: &app.id,
            name: &app.name,
            summary: &app.summary,
            source: app.source.kind(),
            picked: picks.has(&app.id),
            setup: picks.setup(&app.id),
            setups: &app.setups,
        })
        .collect()
}

/// The picked ids `catalog` doesn't have, sorted.
pub fn unknown(catalog: &Catalog, picks: &Picks) -> Vec<String> {
    picks
        .iter()
        .filter(|id| catalog.get(id).is_none())
        .map(str::to_owned)
        .collect()
}
