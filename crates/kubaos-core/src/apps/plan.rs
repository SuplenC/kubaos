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
//! # Owns
//!
//! - [`Facts`], [`AppState`], [`Status`], [`listing`], and the choices [`installs`], [`removals`]
//!   and [`nix_released`].
//!
//! # Not here
//!
//! - Reading the facts and running the helper, apt or the switch: the updater's `apps.rs`.

use std::collections::BTreeSet;

use serde::Serialize;

use super::added::Added;
use super::catalog::Catalog;
use super::manifest::App;
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
        let installed = match &app.source {
            Source::Apt { packages, .. } => packages.iter().all(|p| self.dpkg.has(p)),
            Source::Nix { packages } => packages.iter().all(|p| self.nix.contains(p)),
            Source::Deb(deb) => match self.dpkg.get(&deb.package) {
                Some(installed) if !deb.self_updates && installed.version != deb.version => {
                    return AppState::Outdated;
                }
                Some(_) => true,
                None => false,
            },
        };
        match (installed, picked) {
            (true, _) => AppState::Installed,
            (false, _) if !app.source.available(self.arch) => AppState::Unavailable,
            (false, true) => AppState::Missing,
            (false, false) => AppState::Available,
        }
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

/// The picked apps to install, in the catalog's order: those the machine lacks, and those whose
/// pinned `.deb` is at another version.
pub fn installs<'a>(facts: &Facts<'a>) -> Vec<&'a App> {
    facts
        .catalog
        .apps()
        .iter()
        .filter(|app| facts.picks.has(&app.id))
        .filter(|app| matches!(facts.state(app), AppState::Missing | AppState::Outdated))
        .collect()
}

/// The apps of `ids` that are on the machine, in the catalog's order.
pub fn removals<'a>(facts: &Facts<'a>, ids: &[String]) -> Vec<&'a App> {
    facts
        .catalog
        .apps()
        .iter()
        .filter(|app| ids.contains(&app.id))
        .filter(|app| facts.state(app).is_installed())
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
}

/// Every app of `catalog`, by id, and whether it is picked.
pub fn listing<'a>(catalog: &'a Catalog, picks: &Picks) -> Vec<Listed<'a>> {
    catalog
        .apps()
        .iter()
        .map(|app| Listed {
            id: &app.id,
            name: &app.name,
            summary: &app.summary,
            source: app.source.kind(),
            picked: picks.has(&app.id),
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
