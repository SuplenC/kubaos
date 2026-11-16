//! One app's folder, `apps/<id>/app.toml`: what the app is, its setups, and the actions its
//! install does in order, beside the files those actions use.
//!
//! # Owns
//!
//! - [`Folder`], [`FolderSetup`], [`Ready`], and what a valid folder is: an install's list of
//!   steps, the app's actions then its setup's.
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

use super::action::Action;
use super::action::ActionError;
use super::action::is_argv;
use super::manifest::is_app_id;

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
    pub summary: String,
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
    summary: String,
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
            summary: file.summary,
            icon: file.icon,
            ready: file.ready,
            actions: shared,
            setups,
        };
        folder.check()?;
        Ok(folder)
    }

    fn check(&self) -> Result<(), FolderError> {
        for (key, value) in [("name", &self.name), ("summary", &self.summary)] {
            if value.trim().is_empty() {
                return Err(FolderError::Empty { key });
            }
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
            Action::try_from(table).map_err(|source| FolderError::Action {
                setup: setup.map(str::to_owned),
                position: i + 1,
                source,
            })
        })
        .collect()
}
