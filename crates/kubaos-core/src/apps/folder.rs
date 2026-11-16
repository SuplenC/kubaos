//! One app's folder, `apps/<id>/app.toml`: what the app is, its setups, and the actions its
//! install does in order, beside the files those actions use.
//!
//! # Owns
//!
//! - [`Folder`], [`FolderSetup`], [`Ready`], [`Category`], and what a valid folder is: what the app
//!   is (its category, tags, summary and description), and an install's list of steps, the app's
//!   actions then its setup's.
//!
//! # Not here
//!
//! - One action: `action.rs`. Running an install: `engine.rs`.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use super::action::Action;
use super::action::ActionError;
use super::action::Kind;
use super::action::is_argv;

/// The file of an app's folder.
pub const APP_FILE: &str = "app.toml";

/// A folder that isn't an app's.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FolderError {
    #[error("reading {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("app.toml is not valid TOML of its shape")]
    Parse(#[source] toml::de::Error),
    #[error("{0:?} is not an app id: lowercase letters, digits and dashes, starting with a letter or digit")]
    InvalidId(String),
    #[error("{}action {position}", setup.as_deref().map_or_else(String::new, |s| format!("setup {s}'s ")))]
    Action {
        setup: Option<String>,
        position: usize,
        #[source]
        source: ActionError,
    },
    #[error("an app with setups offers at least two, so there is a choice; this offers one")]
    OneSetup,
    #[error("{0:?} is not a setup id: lowercase letters, digits and dashes, starting with a letter or digit")]
    InvalidSetup(String),
    #[error("{key} is empty")]
    Empty { key: &'static str },
    #[error("an install does something: {0} has no actions")]
    NoActions(String),
    #[error("a ready check is an argv, a program on PATH or ./<script> in the app's folder, and what it says")]
    InvalidReady,
    #[error("{id} can be set up more than one way: name one of {}", setups.join(", "))]
    NoSetup { id: String, setups: Vec<String> },
    #[error("{id} has no setup {setup}: it has {}", setups.join(", "))]
    UnknownSetup {
        id: String,
        setup: String,
        setups: Vec<String>,
    },
    #[error("{id} can be set up one way only, so it takes no setup")]
    SetupWithout { id: String },
    #[error("{0:?} is not a tag: lowercase words people search by, each once")]
    InvalidTag(String),
    #[error("{0:?} is not where the description came from: an https:// URL")]
    InvalidSource(String),
    #[error("{0:?} is not an icon name: letters, digits, `-`, `_` and `.`, as the icon theme names them")]
    InvalidIcon(String),
}

/// What kind of app it is, which groups it with others in lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Category {
    Browsers,
    Development,
    Ai,
    Communication,
    Media,
    Graphics,
    Office,
    Security,
    System,
    Games,
}

impl Category {
    /// Its word in `app.toml`.
    pub fn word(self) -> &'static str {
        match self {
            Self::Browsers => "browsers",
            Self::Development => "development",
            Self::Ai => "ai",
            Self::Communication => "communication",
            Self::Media => "media",
            Self::Graphics => "graphics",
            Self::Office => "office",
            Self::Security => "security",
            Self::System => "system",
            Self::Games => "games",
        }
    }
}

/// A check that must pass before an install or a removal changes anything.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ready {
    /// An argv run as the user; 0 is ready.
    pub check: Vec<String>,
    /// What to tell the user when it isn't: "T3 Code is open: quit it, then try again".
    pub says: String,
}

/// One app's folder.
#[derive(Clone, Debug, PartialEq)]
pub struct Folder {
    pub id: String,
    /// The folder, where a `file`'s `from` and a `run`'s `./<script>` are.
    pub dir: PathBuf,
    pub name: String,
    pub category: Category,
    /// Words people search by ("browser", "containers"), each lowercase.
    pub tags: Vec<String>,
    /// One line of KubaOS's own about it.
    pub summary: String,
    /// The app's own words about itself, copied from its upstream.
    pub description: String,
    /// Where `description` came from: an https:// URL.
    pub description_from: String,
    pub icon: Option<String>,
    pub ready: Vec<Ready>,
    /// What every setup's install does first.
    pub actions: Vec<Action>,
    /// Sorted by id; none for an app with one way only.
    pub setups: Vec<FolderSetup>,
}

/// One way the app can be set up.
#[derive(Clone, Debug, PartialEq)]
pub struct FolderSetup {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub ready: Vec<Ready>,
    /// What it does after the app's own actions.
    pub actions: Vec<Action>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    name: String,
    category: Category,
    tags: Vec<String>,
    summary: String,
    description: String,
    description_from: String,
    icon: Option<String>,
    #[serde(default)]
    ready: Vec<Ready>,
    #[serde(default)]
    action: Vec<toml::Table>,
    #[serde(default)]
    setup: BTreeMap<String, SetupFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetupFile {
    name: String,
    summary: String,
    #[serde(default)]
    ready: Vec<Ready>,
    #[serde(default)]
    action: Vec<toml::Table>,
}

impl Folder {
    /// Reads the app `id`'s folder in the catalog `catalog`.
    ///
    /// # Errors
    ///
    /// [`FolderError`] when `id` can't be an app's, its `app.toml` can't be read or isn't valid.
    pub fn load(catalog: &Path, id: &str) -> Result<Self, FolderError> {
        if !is_app_id(id) {
            return Err(FolderError::InvalidId(id.to_owned()));
        }
        let dir = catalog.join(id);
        let path = dir.join(APP_FILE);
        let text = fs::read_to_string(&path).map_err(|source| FolderError::Read { path, source })?;
        Self::parse(id, &dir, &text)
    }

    /// Parses the app `id`'s `app.toml`, in the folder `dir`.
    ///
    /// # Errors
    ///
    /// [`FolderError`] when it isn't a valid app.
    pub fn parse(id: &str, dir: &Path, text: &str) -> Result<Self, FolderError> {
        if !is_app_id(id) {
            return Err(FolderError::InvalidId(id.to_owned()));
        }
        let file: File = toml::from_str(text).map_err(FolderError::Parse)?;
        let shared = actions(None, file.action)?;
        let setups = file
            .setup
            .into_iter()
            .map(|(id, setup)| {
                if !is_app_id(&id) {
                    return Err(FolderError::InvalidSetup(id));
                }
                Ok(FolderSetup {
                    actions: actions(Some(&id), setup.action)?,
                    id,
                    name: setup.name,
                    summary: setup.summary,
                    ready: setup.ready,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let folder = Self {
            id: id.to_owned(),
            dir: dir.to_path_buf(),
            name: file.name,
            category: file.category,
            tags: file.tags,
            summary: file.summary,
            description: file.description,
            description_from: file.description_from,
            icon: file.icon,
            ready: file.ready,
            actions: shared,
            setups,
        };
        folder.check()?;
        Ok(folder)
    }

    fn check(&self) -> Result<(), FolderError> {
        for (key, value) in [
            ("name", &self.name),
            ("summary", &self.summary),
            ("description", &self.description),
        ] {
            if value.trim().is_empty() {
                return Err(FolderError::Empty { key });
            }
        }
        if self.tags.is_empty() {
            return Err(FolderError::Empty { key: "tags" });
        }
        for (i, tag) in self.tags.iter().enumerate() {
            if !is_tag(tag) || self.tags[..i].contains(tag) {
                return Err(FolderError::InvalidTag(tag.clone()));
            }
        }
        if let Some(icon) = &self.icon
            && !is_icon_name(icon)
        {
            return Err(FolderError::InvalidIcon(icon.clone()));
        }
        if !super::action::is_https(&self.description_from) {
            return Err(FolderError::InvalidSource(self.description_from.clone()));
        }
        if self.setups.len() == 1 {
            return Err(FolderError::OneSetup);
        }
        if self.setups.is_empty() && self.actions.is_empty() {
            return Err(FolderError::NoActions(self.id.clone()));
        }
        for setup in &self.setups {
            if setup.name.trim().is_empty() || setup.summary.trim().is_empty() {
                return Err(FolderError::Empty { key: "setup.name" });
            }
            if self.actions.is_empty() && setup.actions.is_empty() {
                return Err(FolderError::NoActions(format!("{}'s setup {}", self.id, setup.id)));
            }
        }
        let readies = self.ready.iter().chain(self.setups.iter().flat_map(|s| &s.ready));
        for ready in readies {
            if !is_argv(&ready.check) || ready.says.trim().is_empty() {
                return Err(FolderError::InvalidReady);
            }
        }
        Ok(())
    }

    /// The setup `id`.
    pub fn setup(&self, id: &str) -> Option<&FolderSetup> {
        self.setups.iter().find(|s| s.id == id)
    }

    /// The setup an install of it in `setup` is, checked: one of its setups for an app that has
    /// them, none for one that hasn't.
    ///
    /// # Errors
    ///
    /// [`FolderError::NoSetup`], [`FolderError::UnknownSetup`] or [`FolderError::SetupWithout`].
    pub fn pick(&self, setup: Option<&str>) -> Result<Option<&FolderSetup>, FolderError> {
        let ids = || self.setups.iter().map(|s| s.id.clone()).collect();
        match (self.setups.is_empty(), setup) {
            (true, None) => Ok(None),
            (true, Some(_)) => Err(FolderError::SetupWithout { id: self.id.clone() }),
            (false, None) => Err(FolderError::NoSetup {
                id: self.id.clone(),
                setups: ids(),
            }),
            (false, Some(wanted)) => self.setup(wanted).map(Some).ok_or_else(|| FolderError::UnknownSetup {
                id: self.id.clone(),
                setup: wanted.to_owned(),
                setups: ids(),
            }),
        }
    }

    /// The install's steps in `setup`, in order: the app's actions, then the setup's. Step `n` is
    /// the `n - 1`th.
    ///
    /// # Errors
    ///
    /// As [`Folder::pick`].
    pub fn steps(&self, setup: Option<&str>) -> Result<Vec<&Action>, FolderError> {
        let picked = self.pick(setup)?;
        Ok(self
            .actions
            .iter()
            .chain(picked.into_iter().flat_map(|s| &s.actions))
            .collect())
    }

    /// Whether it can be set up more than one way.
    pub fn has_setups(&self) -> bool {
        !self.setups.is_empty()
    }

    /// Every action of it, the app's then each setup's.
    pub fn all_actions(&self) -> impl Iterator<Item = &Action> {
        self.actions.iter().chain(self.setups.iter().flat_map(|s| &s.actions))
    }

    /// Whether every step of an install in `setup` has something for a machine of `arch`.
    pub fn available(&self, arch: &str, setup: Option<&FolderSetup>) -> bool {
        self.actions
            .iter()
            .chain(setup.into_iter().flat_map(|s| &s.actions))
            .all(|action| action.available(arch))
    }

    /// The dpkg packages its `apt` and `deb` actions install, in any setup.
    pub fn packages(&self) -> Vec<&str> {
        self.all_actions()
            .flat_map(|action| match &action.kind {
                Kind::Apt { packages, .. } => packages.iter().map(String::as_str).collect(),
                Kind::Deb { package, .. } => vec![package.as_str()],
                _ => Vec::new(),
            })
            .collect()
    }

    /// The packages of the app's own `apt` and `.deb` actions for `arch`, which every setup
    /// installs; none when an `apt` action is for another architecture, or the app has none.
    pub fn own_packages(&self, arch: &str) -> Vec<&str> {
        let mut packages = Vec::new();
        for action in &self.actions {
            match &action.kind {
                Kind::Apt { arch: Some(only), .. } if only != arch => return Vec::new(),
                Kind::Apt { packages: own, .. } => packages.extend(own.iter().map(String::as_str)),
                Kind::Deb { package, .. } => packages.push(package.as_str()),
                _ => {}
            }
        }
        packages
    }

    /// The ready checks of an install or a removal in `setup`.
    pub fn readies<'a>(&'a self, setup: Option<&'a FolderSetup>) -> impl Iterator<Item = &'a Ready> {
        self.ready.iter().chain(setup.into_iter().flat_map(|s| &s.ready))
    }
}

fn actions(setup: Option<&str>, tables: Vec<toml::Table>) -> Result<Vec<Action>, FolderError> {
    tables
        .into_iter()
        .enumerate()
        .map(|(i, table)| {
            Action::from_folder(table).map_err(|source| FolderError::Action {
                setup: setup.map(str::to_owned),
                position: i + 1,
                source,
            })
        })
        .collect()
}

/// Whether `id` can be an app's: lowercase ASCII letters, digits and `-`, starting with a letter
/// or a digit, at most 64, so an id is never a path or an option. A repository's name, a setup's
/// and a web app's keep to it too.
pub fn is_app_id(id: &str) -> bool {
    id.len() <= 64
        && id
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Whether `value` can be an icon theme's name for an icon: no path, no extension.
/// <https://specifications.freedesktop.org/icon-naming-spec/latest/>
fn is_icon_name(value: &str) -> bool {
    value.len() <= 128
        && value.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Whether `tag` is lowercase words, one space between them.
fn is_tag(tag: &str) -> bool {
    !tag.is_empty()
        && tag
            .split(' ')
            .all(|word| !word.is_empty() && word.chars().all(|c| c.is_lowercase() || c.is_ascii_digit() || c == '-'))
}

#[cfg(test)]
mod tests;
