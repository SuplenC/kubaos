//! One app's folder, `apps/<id>/app.toml`: what the app is, its setups, and the actions its
//! install does in order, beside the files those actions use.
//!
//! # Owns
//!
//! - [`Folder`], [`FolderSetup`], [`Ready`], [`Category`], [`Requirement`], [`Source`], and what a
//!   valid folder is: what the app is (its category, tags, summary and description), the apps it
//!   requires, where its install takes it from, and an install's list of steps: the install stage's
//!   actions, the app's then its setup's, then the configure stage's, in the same order.
//!
//! # Not here
//!
//! - One action: `action.rs`. Running an install: `engine.rs`.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use super::action::Action;
use super::action::ActionError;
use super::action::Kind;
use super::action::RunSource;
use super::action::Side;
use super::action::Stage;
use super::action::is_argv;

/// The file of an app's folder.
pub const APP_FILE: &str = "app.toml";

/// A folder that isn't an app's.
/// The `description_from` of a description KubaOS wrote, as the app's own words aren't under a
/// licence that allows copying them.
pub const WRITTEN_BY_KUBAOS: &str = "kubaos";

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
    #[error(
        "{0} installs through a `run` and names no source: the `run` that installs the app says what from, `source = \"container_images\"`"
    )]
    NoSource(String),
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
    #[error("{0:?} is not where the description came from: an https:// URL, or \"kubaos\" for one KubaOS wrote")]
    InvalidSource(String),
    #[error("{0:?} is not an icon name: letters, digits, `-`, `_` and `.`, as the icon theme names them")]
    InvalidIcon(String),
    #[error("{0:?} is not an app this one can require: an app id, other than its own, named once")]
    InvalidRequirement(String),
    #[error("{0:?} is not a version: digits, letters and `.+~-:`, starting with a digit")]
    InvalidVersion(String),
    #[error(
        "{name}: a configure step writes the app's KubaOS defaults as the user, in ~/.config/kubaos/{id}/, and nowhere else"
    )]
    ConfigureOutside { id: String, name: String },
}

/// What kind of app it is, which groups it with others in lists, in this order.
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

    /// Its heading in a list.
    pub fn title(self) -> &'static str {
        match self {
            Self::Browsers => "Browsers",
            Self::Development => "Development",
            Self::Ai => "AI",
            Self::Communication => "Communication",
            Self::Media => "Media",
            Self::Graphics => "Graphics",
            Self::Office => "Office",
            Self::Security => "Security",
            Self::System => "System",
            Self::Games => "Games",
        }
    }
}

/// Where an install takes an app from, in the order the catalog prefers its sources. Exhaustive,
/// so a new source breaks every list that names them until it says the new one too.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Source {
    /// A vendor's apt repository, by its name in the package manifest.
    AptRepo { repo: String },
    /// Ubuntu's archive.
    Archive,
    /// The user's nixpkgs, through the User flake.
    Nixpkgs,
    /// A vendor's `.deb`, pinned by its sha256.
    Deb,
    /// A vendor's download, an `AppImage` or an archive, pinned by its sha256.
    Download,
    /// Container images, each pinned by its digest, which a `run` declaring them installs.
    ContainerImages,
    /// A website in a window of its own.
    WebApp,
}

impl Source {
    /// The source of `kind`, for the kinds that put the app itself on the machine: a `run` that
    /// declares one, and a web app on a website, not one of this machine's, which is a window on
    /// what another action installs.
    fn of(kind: &Kind) -> Option<Self> {
        match kind {
            Kind::Apt { repo: Some(repo), .. } => Some(Self::AptRepo { repo: repo.clone() }),
            Kind::Apt { repo: None, .. } => Some(Self::Archive),
            Kind::Nix { .. } => Some(Self::Nixpkgs),
            Kind::Deb { .. } => Some(Self::Deb),
            Kind::Download { .. } => Some(Self::Download),
            Kind::Run {
                source: Some(RunSource::ContainerImages),
                ..
            } => Some(Self::ContainerImages),
            Kind::WebApp { url, .. } if super::action::is_https(url) => Some(Self::WebApp),
            _ => None,
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

/// An app installed before the one that requires it.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Requirement {
    /// `"neovim"`.
    Id(String),
    /// `{ id = "neovim", min = "0.10" }`: at least that version, as dpkg compares them.
    Min {
        id: String,
        #[serde(default)]
        min: Option<String>,
    },
}

impl Requirement {
    /// The required app's id.
    pub fn id(&self) -> &str {
        match self {
            Self::Id(id) | Self::Min { id, .. } => id,
        }
    }

    /// The lowest version of it that will do, when any won't.
    pub fn min(&self) -> Option<&str> {
        match self {
            Self::Id(_) => None,
            Self::Min { min, .. } => min.as_deref(),
        }
    }
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
    /// The app's own words about itself, copied from its upstream, or KubaOS's plain factual
    /// ones where the upstream's licence doesn't allow copying.
    pub description: String,
    /// Where `description` came from: an https:// URL, or [`WRITTEN_BY_KUBAOS`] for KubaOS's own.
    pub description_from: String,
    pub icon: Option<String>,
    pub ready: Vec<Ready>,
    /// The apps every setup's install installs first, in order.
    pub requires: Vec<Requirement>,
    /// What a removal leaves on purpose, said as it ends: the user's own data.
    pub keeps: Option<String>,
    /// What every setup's install does first, in each stage.
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
    /// The apps it installs first, after the app's own.
    pub requires: Vec<Requirement>,
    /// What a removal of it leaves on purpose, said as it ends, after the app's own.
    pub keeps: Option<String>,
    /// What it does after the app's own actions, in each stage.
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
    requires: Vec<Requirement>,
    keeps: Option<String>,
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
    requires: Vec<Requirement>,
    keeps: Option<String>,
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
                    requires: setup.requires,
                    keeps: setup.keeps,
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
            requires: file.requires,
            keeps: file.keeps,
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
        let keeps = self.keeps.iter().chain(self.setups.iter().flat_map(|s| &s.keeps));
        if keeps.into_iter().any(|says| says.trim().is_empty()) {
            return Err(FolderError::Empty { key: "keeps" });
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
        if self.description_from != WRITTEN_BY_KUBAOS && !super::action::is_https(&self.description_from) {
            return Err(FolderError::InvalidSource(self.description_from.clone()));
        }
        if self.setups.len() == 1 {
            return Err(FolderError::OneSetup);
        }
        if self.setups.is_empty() && self.actions.is_empty() {
            return Err(FolderError::NoActions(self.id.clone()));
        }
        if self.setups.is_empty() && self.installs_unnamed(None) {
            return Err(FolderError::NoSource(self.id.clone()));
        }
        for setup in &self.setups {
            if setup.name.trim().is_empty() || setup.summary.trim().is_empty() {
                return Err(FolderError::Empty { key: "setup.name" });
            }
            if self.actions.is_empty() && setup.actions.is_empty() {
                return Err(FolderError::NoActions(format!("{}'s setup {}", self.id, setup.id)));
            }
            if self.installs_unnamed(Some(setup)) {
                return Err(FolderError::NoSource(format!("{}'s setup {}", self.id, setup.id)));
            }
        }
        let readies = self.ready.iter().chain(self.setups.iter().flat_map(|s| &s.ready));
        for ready in readies {
            if !is_argv(&ready.check) || ready.says.trim().is_empty() {
                return Err(FolderError::InvalidReady);
            }
        }
        self.check_requires()?;
        for action in self.all_actions().filter(|a| a.stage == Stage::Configure) {
            if !self.configures_inside(action) {
                return Err(FolderError::ConfigureOutside {
                    id: self.id.clone(),
                    name: action.name.clone(),
                });
            }
        }
        Ok(())
    }

    /// Checks each list of requirements, the app's and each setup's with it: an app id other
    /// than its own, each once, a `min` a version.
    fn check_requires(&self) -> Result<(), FolderError> {
        let lists = std::iter::once(None).chain(self.setups.iter().map(|s| Some(&s.requires)));
        for extra in lists {
            let all: Vec<&Requirement> = self.requires.iter().chain(extra.into_iter().flatten()).collect();
            for (i, requirement) in all.iter().enumerate() {
                let id = requirement.id();
                if !is_app_id(id) || id == self.id || all[..i].iter().any(|r| r.id() == id) {
                    return Err(FolderError::InvalidRequirement(id.to_owned()));
                }
                if let Some(min) = requirement.min()
                    && !super::version::is_version(min)
                {
                    return Err(FolderError::InvalidVersion(min.to_owned()));
                }
            }
        }
        Ok(())
    }

    /// Whether a configure step stays in the app's own folder of the user layer, as the user:
    /// what it writes is `~/.config/kubaos/<id>/…`, and it is a file, a link, a path a program
    /// writes, a settings key or the user's own `run`.
    fn configures_inside(&self, action: &Action) -> bool {
        let inside = |path: &str| {
            path.strip_prefix(&format!("~/.config/kubaos/{}/", self.id))
                .is_some_and(|rest| !rest.is_empty())
        };
        if action.side() != Side::User {
            return false;
        }
        match &action.kind {
            Kind::Setting { file, .. } => inside(file),
            Kind::Run { .. } => true,
            Kind::File { .. } | Kind::Symlink { .. } | Kind::Output { .. } | Kind::Download { .. } => {
                action.path().is_some_and(inside)
            }
            _ => false,
        }
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

    /// The install's steps in `setup`, in order: the install stage's, the app's actions then the
    /// setup's, then the configure stage's in the same order. Step `n` is the `n - 1`th. The app's
    /// own install-stage actions are the first steps in every setup.
    ///
    /// # Errors
    ///
    /// As [`Folder::pick`].
    pub fn steps(&self, setup: Option<&str>) -> Result<Vec<&Action>, FolderError> {
        let picked = self.pick(setup)?;
        let all = || self.actions.iter().chain(picked.into_iter().flat_map(|s| &s.actions));
        Ok(all()
            .filter(|a| a.stage == Stage::Install)
            .chain(all().filter(|a| a.stage == Stage::Configure))
            .collect())
    }

    /// How many steps every setup shares: the app's own install-stage actions, which come first.
    pub fn shared_steps(&self) -> usize {
        self.actions.iter().filter(|a| a.stage == Stage::Install).count()
    }

    /// The apps an install in `setup` requires, in order: the app's, then the setup's.
    pub fn requires<'a>(&'a self, setup: Option<&'a FolderSetup>) -> impl Iterator<Item = &'a Requirement> {
        self.requires.iter().chain(setup.into_iter().flat_map(|s| &s.requires))
    }

    /// What a removal of it in `setup` leaves on purpose, the app's then the setup's.
    pub fn keeps<'a>(&'a self, setup: Option<&'a FolderSetup>) -> impl Iterator<Item = &'a str> {
        self.keeps
            .iter()
            .chain(setup.into_iter().flat_map(|s| &s.keeps))
            .map(String::as_str)
    }

    /// Whether an install in `setup` has a configure stage.
    pub fn configurable(&self, setup: Option<&FolderSetup>) -> bool {
        self.actions
            .iter()
            .chain(setup.into_iter().flat_map(|s| &s.actions))
            .any(|a| a.stage == Stage::Configure)
    }

    /// Whether it can be set up more than one way.
    pub fn has_setups(&self) -> bool {
        !self.setups.is_empty()
    }

    /// Every action of it, the app's then each setup's.
    pub fn all_actions(&self) -> impl Iterator<Item = &Action> {
        self.actions.iter().chain(self.setups.iter().flat_map(|s| &s.actions))
    }

    /// Whether every step of its own install in `setup` has something for a machine of `arch`; the
    /// apps it requires are the catalog's to say ([`super::Catalog::available`]).
    pub fn available(&self, arch: &str, setup: Option<&FolderSetup>) -> bool {
        self.actions
            .iter()
            .chain(setup.into_iter().flat_map(|s| &s.actions))
            .all(|action| action.available(arch))
    }

    /// Whether an install in `setup` runs a program in its install stage and names no source:
    /// that `run` installs the app, so it says what from.
    fn installs_unnamed(&self, setup: Option<&FolderSetup>) -> bool {
        let runs = self
            .actions
            .iter()
            .chain(setup.into_iter().flat_map(|s| &s.actions))
            .any(|a| a.stage == Stage::Install && matches!(a.kind, Kind::Run { .. }));
        runs && self.sources(setup).is_empty()
    }

    /// Where an install in `setup` takes it from, its install stage's actions, the app's then
    /// the setup's, each source once, in the catalog's order of sources.
    pub fn sources(&self, setup: Option<&FolderSetup>) -> Vec<Source> {
        let sources: BTreeSet<Source> = self
            .actions
            .iter()
            .chain(setup.into_iter().flat_map(|s| &s.actions))
            .filter(|a| a.stage == Stage::Install)
            .filter_map(|a| Source::of(&a.kind))
            .collect();
        sources.into_iter().collect()
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

    /// Whether the description is the upstream's own words, copied, not KubaOS's.
    pub fn description_copied(&self) -> bool {
        self.description_from != WRITTEN_BY_KUBAOS
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
