//! The Suites in the menu: one Item a Suite of the catalog, which installs it, or configures and
//! removes it once installed, and one "New <Suite> project…" Item a scaffolder of an installed
//! Suite. Whether a Suite is installed is its file in the state directory,
//! `kubaos.d/suites/<name>.toml`, read with the states, so its change comes back through their
//! watch in `main.rs`.
//!
//! Each Action runs `kubaos suite` in a terminal, which shows the switch, the caches a removal
//! keeps and the user-layer files that mention the Suite. What the menu can check, a version or
//! an extra that the Suite doesn't have, is checked here, so the refusal reads in the menu.
//!
//! # Owns
//!
//! - Reading which Suites are installed and their options, the Suite Items, and what their Actions
//!   run.
//!
//! # Not here
//!
//! - The catalog, the Suite files and the scaffolding: `kubaos_core::suites`.
//! - The commands themselves: `kubaos suite` (`crates/kubaos-cli/src/suite.rs`).
//! - The terminal the commands run in: the registry's `terminal`, [`crate::registry`].

use std::collections::BTreeMap;
use std::time::Duration;

use kotodama_extension::Chain;
use kotodama_protocol::Action;
use kotodama_protocol::Argument;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;
use kubaos_core::state::StateDir;
use kubaos_core::suites::Scaffolder;
use kubaos_core::suites::Suite;
use kubaos_core::suites::SuiteConfig;
use kubaos_core::suites::catalog;

use crate::run::Runner;
use crate::toggles::Ran;

/// The Item kind of every Suite Item.
pub const KIND: &str = "kubaos.suite";
/// Installs a Suite that isn't installed.
pub const INSTALL: &str = "install";
/// Changes an installed Suite's version and extras.
pub const CONFIGURE: &str = "configure";
/// Uninstalls an installed Suite.
pub const REMOVE: &str = "remove";
/// Creates a project with one of an installed Suite's scaffolders.
pub const NEW: &str = "new";

/// A Suite's Item is this and its name, `suite:go`.
const PREFIX: &str = "suite:";
/// A scaffolder's Item is this, its Suite's name and its id, `suite-new:go:module`.
const NEW_PREFIX: &str = "suite-new:";
/// The Argument of the version, in the Configure Action.
const VERSION: &str = "version";
/// The Argument of the project's name, in a scaffolder's Action.
const PROJECT: &str = "name";
/// What an extra's Argument takes, to turn it on or off.
const ON: &str = "on";
const OFF: &str = "off";

const ICON: &str = "applications-development";
const NEW_ICON: &str = "folder-new";

/// What the state directory says of each Suite of the catalog that is installed, by name: its
/// options, or `None` while its file doesn't read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Installed {
    suites: BTreeMap<&'static str, Option<SuiteConfig>>,
}

impl Installed {
    /// Reads the file of each Suite of the catalog in `state_dir`. A Suite without its file is
    /// not installed; one whose file doesn't read is, with options unknown.
    pub fn read(state_dir: &StateDir) -> Self {
        let suites = catalog::all()
            .filter_map(|suite| match state_dir.suite_config(suite.name) {
                Ok(None) => None,
                Ok(Some(config)) => Some((suite.name, Some(config))),
                Err(_) => Some((suite.name, None)),
            })
            .collect();
        Installed { suites }
    }

    /// These Suites with the catalog's Suite `name` installed with `config`, `None` for a file
    /// that doesn't read. A name the catalog doesn't know changes nothing.
    #[must_use]
    pub fn with(mut self, name: &str, config: Option<SuiteConfig>) -> Self {
        if let Some(suite) = catalog::find(name) {
            self.suites.insert(suite.name, config);
        }
        self
    }

    /// Whether the Suite `name` is installed: `None` while its file doesn't read.
    pub fn is_installed(&self, name: &str) -> Option<bool> {
        match self.suites.get(name) {
            None => Some(false),
            Some(config) => config.as_ref().map(|_| true),
        }
    }

    /// The Items of every Suite of the catalog, each followed by its scaffolders' while it is
    /// installed, in the catalog's order.
    pub fn items(&self) -> Vec<Item> {
        let mut items = Vec::new();
        for suite in catalog::all() {
            match self.suites.get(suite.name) {
                None => items.push(install_item(suite)),
                Some(config) => {
                    items.push(installed_item(suite, config.as_ref()));
                    items.extend(suite.scaffolders.iter().map(|scaffolder| new_item(suite, scaffolder)));
                }
            }
        }
        items
    }

    /// Runs `action` on the Suite Item `id` with `arguments`, in `terminal`; the error is what the
    /// user reads.
    ///
    /// # Errors
    ///
    /// When there is no such Item or Action, an Argument isn't one the Suite takes, or the
    /// terminal can't start.
    pub fn run(
        &self,
        id: &str,
        action: &str,
        arguments: &BTreeMap<String, String>,
        terminal: &[String],
        runner: &impl Runner,
    ) -> Result<Ran, String> {
        let unknown = || format!("`{id}` has no Action `{action}`");
        let command = if let Some(name) = id.strip_prefix(PREFIX) {
            let suite = catalog::find(name).ok_or_else(unknown)?;
            match (self.suites.contains_key(suite.name), action) {
                (false, INSTALL) => kubaos(&["add", suite.name]),
                (true, CONFIGURE) => configure(suite, arguments)?,
                (true, REMOVE) => kubaos(&["remove", suite.name]),
                _ => return Err(unknown()),
            }
        } else {
            let (suite, scaffolder) = self.scaffolder(id).ok_or_else(unknown)?;
            if action != NEW {
                return Err(unknown());
            }
            let project = arguments.get(PROJECT).map_or("", |name| name.trim());
            if project.is_empty() {
                return Err("Name the new project".to_owned());
            }
            kubaos(&["new", suite.name, project, "--with", scaffolder.id])
        };
        let argv: Vec<String> = terminal.iter().cloned().chain(command).collect();
        runner
            .launch(&argv, Duration::ZERO)
            .map_err(|err| Chain(&err).to_string())?;
        Ok(Ran::Close)
    }

    /// The installed Suite and the scaffolder of the Item `id`.
    fn scaffolder(&self, id: &str) -> Option<(&'static Suite, &'static Scaffolder)> {
        let (name, scaffolder) = id.strip_prefix(NEW_PREFIX)?.split_once(':')?;
        let suite = catalog::find(name).filter(|suite| self.suites.contains_key(suite.name))?;
        let scaffolder = suite.scaffolders.iter().find(|offered| offered.id == scaffolder)?;
        Some((suite, scaffolder))
    }
}

/// The id of every Item a Suite of the catalog may have, installed or not, which `hide` may name.
pub fn ids() -> Vec<String> {
    catalog::all()
        .flat_map(|suite| {
            let scaffolders = suite.scaffolders.iter().map(|scaffolder| new_id(suite, scaffolder));
            std::iter::once(format!("{PREFIX}{}", suite.name)).chain(scaffolders)
        })
        .collect()
}

/// `kubaos suite` with `words`.
fn kubaos(words: &[&str]) -> Vec<String> {
    ["kubaos", "suite"]
        .iter()
        .chain(words)
        .map(|&word| word.to_owned())
        .collect()
}

/// `kubaos suite configure` with the version and the extras the Arguments give; an Argument left
/// empty keeps what the Suite has.
fn configure(suite: &Suite, arguments: &BTreeMap<String, String>) -> Result<Vec<String>, String> {
    let given = |name: &str| {
        arguments
            .get(name)
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
    };
    let mut command = kubaos(&["configure", suite.name]);
    if let Some(version) = given(VERSION) {
        if !suite.versions.contains(&version) {
            return Err(format!(
                "The {} Suite has no version {version}. It has {}",
                suite.title,
                suite.versions.join(", ")
            ));
        }
        command.extend(["--version".to_owned(), version.to_owned()]);
    }
    for extra in suite.extras {
        let flag = match given(extra.name).map(str::to_lowercase).as_deref() {
            None => continue,
            Some(ON) => "--extra",
            Some(OFF) => "--no-extra",
            Some(other) => return Err(format!("`{other}` isn't on or off, for {}", extra.name)),
        };
        command.extend([flag.to_owned(), extra.name.to_owned()]);
    }
    Ok(command)
}

/// The Item of a Suite that isn't installed.
fn install_item(suite: &Suite) -> Item {
    let title = format!("Install the {} Suite", suite.title);
    let mut keywords = keywords(suite);
    keywords.push("install".to_owned());
    item(
        format!("{PREFIX}{}", suite.name),
        title.clone(),
        suite.description.to_owned(),
        ICON,
        keywords,
        vec![Action::new(INSTALL, title)],
    )
}

/// The Item of an installed Suite, whose options are `config`, `None` while its file doesn't
/// read.
fn installed_item(suite: &Suite, config: Option<&SuiteConfig>) -> Item {
    let subtitle = match config.map(|config| config.version.as_deref()) {
        None => format!("Installed, but kubaos.d/suites/{}.toml doesn't read", suite.name),
        Some(None) => "Installed".to_owned(),
        Some(Some(version)) => format!("Installed, version {version}"),
    };
    let mut keywords = keywords(suite);
    keywords.extend(["configure", "settings", "version", "extras", "remove", "uninstall"].map(str::to_owned));
    let remove = Action::new(REMOVE, format!("Remove the {} Suite", suite.title)).confirm();
    item(
        format!("{PREFIX}{}", suite.name),
        format!("{} Suite", suite.title),
        subtitle,
        ICON,
        keywords,
        vec![configure_action(suite), remove],
    )
}

/// Configure…, with the version when the Suite has a choice, then each extra, `on` or `off`.
fn configure_action(suite: &Suite) -> Action {
    let version = (!suite.versions.is_empty()).then(|| Argument {
        name: VERSION.to_owned(),
        placeholder: Some(suite.versions.join(", ")),
        optional: true,
    });
    let extras = suite.extras.iter().map(|extra| Argument {
        name: extra.name.to_owned(),
        placeholder: Some(format!("{ON} or {OFF}")),
        optional: true,
    });
    Action::new(CONFIGURE, "Configure…").arguments(version.into_iter().chain(extras).collect())
}

/// The Item of one scaffolder of an installed Suite.
fn new_item(suite: &Suite, scaffolder: &Scaffolder) -> Item {
    let title = format!("{}…", scaffolder.label);
    let mut keywords = keywords(suite);
    keywords.extend(["new project", "create", "scaffold", "template"].map(str::to_owned));
    let argument = Argument {
        name: PROJECT.to_owned(),
        placeholder: Some("hello".to_owned()),
        optional: false,
    };
    item(
        new_id(suite, scaffolder),
        title.clone(),
        format!("A project in your Projects folder, from the {} Suite", suite.title),
        NEW_ICON,
        keywords,
        vec![Action::new(NEW, title).arguments(vec![argument])],
    )
}

fn new_id(suite: &Suite, scaffolder: &Scaffolder) -> String {
    format!("{NEW_PREFIX}{}:{}", suite.name, scaffolder.id)
}

/// The words every Item of `suite` is found by: its name and title, "suite", and the words its
/// language goes by, which are the ones that mark a user-layer file as being about it.
fn keywords(suite: &Suite) -> Vec<String> {
    [suite.name, suite.title, "suite", "language", "toolchain"]
        .into_iter()
        .chain(suite.mentions.iter().copied())
        .map(str::to_owned)
        .collect()
}

fn item(id: String, title: String, subtitle: String, icon: &str, keywords: Vec<String>, actions: Vec<Action>) -> Item {
    Item {
        id,
        kind: ItemKind::Namespaced(NamespacedKind::new(KIND).expect("the Extension's kinds are valid")),
        identity: None,
        title,
        subtitle: Some(subtitle),
        icon: Some(icon.to_owned()),
        keyword: None,
        keywords,
        hidden: false,
        app_only: false,
        weak_keywords: Vec::new(),
        actions,
    }
}

#[cfg(test)]
mod tests;
