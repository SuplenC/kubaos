//! The catalog: every app's folder, `apps/<id>/app.toml`, in the release's checkout, which is
//! `/usr/share/kubaos/apps/` on a machine.
//!
//! # Owns
//!
//! - [`Catalog`]: loading the folders, finding an app, the packages its apps install, and checking
//!   each app's vendor repositories against the package manifest.
//!
//! # Not here
//!
//! - One folder's format: `folder.rs`.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use super::action::Kind;
use super::folder::APP_FILE;
use super::folder::Folder;
use super::folder::FolderError;
use super::folder::is_app_id;
use crate::apt::Manifest;

/// The catalog's folder, inside the release's checkout.
pub const CATALOG: &str = "apps";

/// The catalog could not be read, or an app in it isn't valid.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CatalogError {
    #[error("reading {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("app {id}")]
    App {
        id: String,
        #[source]
        source: Box<FolderError>,
    },
    #[error("app {id} takes its packages from repo {repo}, which the package manifest doesn't define")]
    UnknownRepo { id: String, repo: String },
}

/// Every app KubaOS knows how to install, by id.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalog {
    /// Sorted by id.
    apps: Vec<Folder>,
}

impl Catalog {
    /// The catalog of these apps.
    #[must_use]
    pub fn new(mut apps: Vec<Folder>) -> Self {
        apps.sort_by(|a, b| a.id.cmp(&b.id));
        Self { apps }
    }

    /// Reads every folder of `dir` named as an app is that holds an `app.toml`; an empty catalog
    /// when `dir` is missing. Anything else in it is passed over.
    ///
    /// # Errors
    ///
    /// [`CatalogError`] when the folder or an app can't be read, or an app isn't valid.
    pub fn load(dir: &Path) -> Result<Self, CatalogError> {
        let read_err = |path: &Path| {
            let path = path.to_path_buf();
            move |source| CatalogError::Read { path, source }
        };
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(read_err(dir)(e)),
        };
        let mut apps = Vec::new();
        for entry in entries {
            let entry = entry.map_err(read_err(dir))?;
            let Some(id) = entry.file_name().to_str().filter(|id| is_app_id(id)).map(str::to_owned) else {
                continue;
            };
            if !entry.path().join(APP_FILE).is_file() {
                continue;
            }
            let app = Folder::load(dir, &id).map_err(|source| CatalogError::App {
                id,
                source: Box::new(source),
            })?;
            apps.push(app);
        }
        Ok(Self::new(apps))
    }

    /// Checks that every vendor repository an `apt` action names is one of `manifest`'s.
    ///
    /// # Errors
    ///
    /// [`CatalogError::UnknownRepo`] for the first app whose repository isn't there.
    pub fn check_repos(&self, manifest: &Manifest) -> Result<(), CatalogError> {
        for app in &self.apps {
            for action in app.all_actions() {
                if let Kind::Apt { repo: Some(repo), .. } = &action.kind
                    && !manifest.repos().contains_key(repo)
                {
                    return Err(CatalogError::UnknownRepo {
                        id: app.id.clone(),
                        repo: repo.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    /// The apps, sorted by id.
    pub fn apps(&self) -> &[Folder] {
        &self.apps
    }

    /// The app `id`.
    pub fn get(&self, id: &str) -> Option<&Folder> {
        self.apps.iter().find(|a| a.id == id)
    }

    /// The apps that install any of `packages`, in id order.
    pub fn installing<'a>(&'a self, packages: &'a [String]) -> impl Iterator<Item = &'a Folder> {
        self.apps
            .iter()
            .filter(|app| app.packages().iter().any(|p| packages.iter().any(|q| q == p)))
    }

    /// The dpkg packages its apps' `apt` and `deb` actions install: the catalog's, which the
    /// package manifest leaves alone.
    pub fn packages(&self) -> BTreeSet<String> {
        self.apps.iter().flat_map(Folder::packages).map(str::to_owned).collect()
    }
}
