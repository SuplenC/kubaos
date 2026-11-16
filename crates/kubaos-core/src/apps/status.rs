//! Where each app of the catalog stands on the machine and in the user's picks: what `kubaos app
//! status` and the Command menu show, and which picked apps an update installs.
//!
//! An app is installed when its user journal says its install is complete, in the setup that
//! journal names, and its journals own something; one they own nothing of is found: on the
//! machine, KubaOS's to leave. An app with no journal whose own packages dpkg has (installed by
//! hand, or before KubaOS kept journals) is installed when it is picked, as taking it in will
//! make it KubaOS's, and found when not. That is two small files an app at most and the one dpkg
//! query the caller made, with no check run, so a status read stays cheap; a version is its
//! first package's.
//!
//! # Owns
//!
//! - [`AppState`], [`Status`], [`AppStatus`], [`Listed`] and [`listing`], and the setup an app is
//!   picked in ([`picked_setup`]).
//!
//! # Not here
//!
//! - Installing and removing: `engine.rs`. The journal's format: `journal.rs`.

use std::path::Path;

use serde::Serialize;

use super::catalog::Catalog;
use super::folder::Category;
use super::folder::Folder;
use super::folder::FolderSetup;
use super::journal::Journal;
use super::journal::JournalError;
use super::picks::Picks;
use crate::apt::Dpkg;

/// Where one app stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppState {
    /// Its install is complete, and KubaOS put it there: a removal undoes it.
    Installed,
    /// On the machine, put there by other means: KubaOS owns nothing of it and leaves it.
    Found,
    /// Picked, and not installed: the next update installs it.
    Missing,
    /// Neither picked nor installed.
    Available,
    /// A step of it has nothing for this machine's architecture, and it isn't installed.
    Unavailable,
}

impl AppState {
    /// The state's word, as the table shows it.
    pub fn word(self) -> &'static str {
        match self {
            Self::Installed => "installed",
            Self::Found => "found",
            Self::Missing => "missing",
            Self::Available => "available",
            Self::Unavailable => "unavailable",
        }
    }

    /// Whether the app is on the machine, KubaOS's or found.
    pub fn is_installed(self) -> bool {
        matches!(self, Self::Installed | Self::Found)
    }

    /// Whether a removal would undo something: the app is KubaOS's.
    pub fn is_removable(self) -> bool {
        self == Self::Installed
    }
}

/// One way an app can be set up, as lists show it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SetupInfo {
    pub id: String,
    pub name: String,
    pub summary: String,
}

impl From<&FolderSetup> for SetupInfo {
    fn from(setup: &FolderSetup) -> Self {
        Self {
            id: setup.id.clone(),
            name: setup.name.clone(),
            summary: setup.summary.clone(),
        }
    }
}

/// One app of the status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AppStatus {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub category: Category,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub picked: bool,
    pub state: AppState,
    /// The installed version of its first apt or `.deb` package, when dpkg was asked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The setup it is installed in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup: Option<String>,
    /// The setups it offers, by id; none for an app with one way only.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub setups: Vec<SetupInfo>,
    /// It is installed, KubaOS's, with a configure stage in its setup, which `kubaos app
    /// configure` runs alone again.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub configurable: bool,
    /// Whether each setup, or the app without setups, has something for this machine: by
    /// `setups`' order, or one value for an app without setups.
    #[serde(skip)]
    pub available: Vec<bool>,
}

impl AppStatus {
    /// Its setup `id`.
    pub fn setup_info(&self, id: &str) -> Option<&SetupInfo> {
        self.setups.iter().find(|s| s.id == id)
    }

    /// Whether `setup` (by id), or the app without setups, has something for this machine.
    pub fn offers(&self, setup: Option<&str>) -> bool {
        match setup {
            None => self.available.first().copied().unwrap_or(false),
            Some(id) => self
                .setups
                .iter()
                .position(|s| s.id == id)
                .and_then(|i| self.available.get(i).copied())
                .unwrap_or(false),
        }
    }
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
    /// The status of the apps of `catalog` with `picks`, installed as the user journals in
    /// `journals` and root's in `root_journals` say, on a machine of `arch`, with the packages
    /// and versions `dpkg` has when given.
    ///
    /// # Errors
    ///
    /// [`JournalError`] for a journal that is there and can't be read.
    pub fn read(
        catalog: &Catalog,
        picks: &Picks,
        journals: &Path,
        root_journals: &Path,
        arch: &str,
        dpkg: Option<&Dpkg>,
    ) -> Result<Self, JournalError> {
        let mut apps = Vec::new();
        for app in catalog.apps() {
            let journal = Journal::read(journals, &app.id)?;
            let installed = journal.as_ref().filter(|j| j.complete);
            let available: Vec<bool> = if app.has_setups() {
                app.setups.iter().map(|s| app.available(arch, Some(s))).collect()
            } else {
                vec![app.available(arch, None)]
            };
            let picked = picks.has(&app.id);
            let owns = match installed {
                Some(j) if j.owns_anything() => true,
                Some(_) => Journal::read(root_journals, &app.id)?.is_some_and(|root| root.owns_anything()),
                None => false,
            };
            let state = match installed {
                Some(_) if owns => AppState::Installed,
                Some(_) => AppState::Found,
                None if journal.is_none() && dpkg.is_some_and(|dpkg| has_packages(app, arch, dpkg)) => {
                    if picked {
                        AppState::Installed
                    } else {
                        AppState::Found
                    }
                }
                None if !available.iter().any(|a| *a) => AppState::Unavailable,
                None if picked => AppState::Missing,
                None => AppState::Available,
            };
            let version = dpkg.and_then(|dpkg| {
                let package = app.packages().into_iter().next()?;
                dpkg.get(package).map(|i| i.version.clone())
            });
            let configurable = state == AppState::Installed
                && app.configurable(installed.and_then(|j| j.setup.as_deref()).and_then(|s| app.setup(s)));
            apps.push(AppStatus {
                id: app.id.clone(),
                name: app.name.clone(),
                summary: app.summary.clone(),
                category: app.category,
                tags: app.tags.clone(),
                icon: app.icon.clone(),
                picked,
                state,
                version: version.filter(|_| state.is_installed()),
                setup: installed.and_then(|j| j.setup.clone()),
                setups: app.setups.iter().map(SetupInfo::from).collect(),
                configurable,
                available,
            });
        }
        Ok(Self {
            arch: arch.to_owned(),
            apps,
            unknown: unknown(catalog, picks),
        })
    }

    /// The app `id`.
    pub fn get(&self, id: &str) -> Option<&AppStatus> {
        self.apps.iter().find(|a| a.id == id)
    }
}

/// Whether dpkg has every package of `app`'s own actions for `arch`, and it has any.
fn has_packages(app: &Folder, arch: &str, dpkg: &Dpkg) -> bool {
    let packages = app.own_packages(arch);
    !packages.is_empty() && packages.iter().all(|p| dpkg.has(p))
}

/// One app of the catalog as `kubaos app list` shows it, with no fact of the machine, so the list
/// runs no program.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Listed<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub summary: &'a str,
    pub category: Category,
    pub tags: &'a [String],
    pub picked: bool,
    /// The setup the picks name for it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup: Option<&'a str>,
    /// The setups it offers, by id; none for an app with one way only.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub setups: Vec<SetupInfo>,
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
            category: app.category,
            tags: &app.tags,
            picked: picks.has(&app.id),
            setup: picks.setup(&app.id),
            setups: app.setups.iter().map(SetupInfo::from).collect(),
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

/// The setup a picked `app` is installed in: the one the picks name, else the one its journal
/// says the machine has. None for an app without setups, and for one with setups that neither
/// names, which nothing installs until one is picked.
pub fn picked_setup<'a>(app: &'a Folder, picks: &Picks, journal: Option<&Journal>) -> Option<&'a FolderSetup> {
    picks
        .setup(&app.id)
        .or_else(|| journal.and_then(|j| j.setup.as_deref()))
        .and_then(|id| app.setup(id))
}
