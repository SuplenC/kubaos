//! The catalog: every app's folder, `apps/<id>/app.toml`, in the release's checkout, which is
//! `/usr/share/kubaos/apps/` on a machine.
//!
//! # Owns
//!
//! - [`Catalog`]: loading the folders, finding an app, the packages its apps install, checking each
//!   app's requirements (apps of the catalog with one way to be set up, no cycle) and its vendor
//!   repositories against the package manifest, and the architectures an app installs on.
//!
//! # Not here
//!
//! - One folder's format: `folder.rs`.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use super::action::ARCHES;
use super::action::Kind;
use super::folder::APP_FILE;
use super::folder::Folder;
use super::folder::FolderError;
use super::folder::FolderSetup;
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
    #[error("app {id} requires {required}, which the catalog doesn't have")]
    UnknownRequirement { id: String, required: String },
    #[error("app {id} requires {required}, which can be set up more than one way: it can't tell which")]
    RequiresSetups { id: String, required: String },
    #[error("apps require each other in a circle: {}", circle.join(" → "))]
    Cycle { circle: Vec<String> },
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
        let catalog = Self::new(apps);
        catalog.check_requires()?;
        Ok(catalog)
    }

    /// Checks that every app an app requires, in any setup, is one of the catalog's with one way
    /// to be set up, and that no app requires itself through others.
    ///
    /// # Errors
    ///
    /// [`CatalogError::UnknownRequirement`], [`CatalogError::RequiresSetups`] or
    /// [`CatalogError::Cycle`], for the first app it finds.
    pub fn check_requires(&self) -> Result<(), CatalogError> {
        for app in &self.apps {
            for required in all_requires(app) {
                let Some(found) = self.get(required) else {
                    return Err(CatalogError::UnknownRequirement {
                        id: app.id.clone(),
                        required: required.to_owned(),
                    });
                };
                if found.has_setups() {
                    return Err(CatalogError::RequiresSetups {
                        id: app.id.clone(),
                        required: required.to_owned(),
                    });
                }
            }
        }
        for app in &self.apps {
            let mut path = vec![app.id.as_str()];
            self.walk(&mut path)?;
        }
        Ok(())
    }

    /// Follows every requirement from the last app of `path`, refusing one back to an app on it.
    fn walk<'a>(&'a self, path: &mut Vec<&'a str>) -> Result<(), CatalogError> {
        let Some(app) = path.last().and_then(|id| self.get(id)) else {
            return Ok(());
        };
        for required in all_requires(app) {
            if let Some(start) = path.iter().position(|id| *id == required) {
                let mut circle: Vec<String> = path[start..].iter().map(|id| (*id).to_owned()).collect();
                circle.push(required.to_owned());
                return Err(CatalogError::Cycle { circle });
            }
            path.push(required);
            self.walk(path)?;
            path.pop();
        }
        Ok(())
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

    /// Whether an install of `app` in `setup` has something for a machine of `arch`: every step
    /// of its own, and of every app it requires, through what those require.
    pub fn available(&self, app: &Folder, setup: Option<&FolderSetup>, arch: &str) -> bool {
        app.available(arch, setup)
            && app.requires(setup).all(|r| {
                self.get(r.id())
                    .is_some_and(|required| self.available(required, None, arch))
            })
    }

    /// The architectures `app` installs on, in one setup at least, of the ones the catalog knows.
    pub fn arches(&self, app: &Folder) -> Vec<&'static str> {
        ARCHES
            .into_iter()
            .filter(|arch| {
                if app.has_setups() {
                    app.setups.iter().any(|s| self.available(app, Some(s), arch))
                } else {
                    self.available(app, None, arch)
                }
            })
            .collect()
    }

    /// The apps that require `id` in any of their setups, in id order.
    pub fn requiring<'a>(&'a self, id: &'a str) -> impl Iterator<Item = &'a Folder> {
        self.apps.iter().filter(move |app| all_requires(app).any(|r| r == id))
    }

    /// The dpkg packages its apps' `apt` and `deb` actions install: the catalog's, which the
    /// package manifest leaves alone.
    pub fn packages(&self) -> BTreeSet<String> {
        self.apps.iter().flat_map(Folder::packages).map(str::to_owned).collect()
    }
}

/// The ids `app` requires in any setup.
fn all_requires(app: &Folder) -> impl Iterator<Item = &str> {
    app.requires
        .iter()
        .chain(app.setups.iter().flat_map(|s| &s.requires))
        .map(super::folder::Requirement::id)
}
