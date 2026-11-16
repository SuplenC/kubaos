//! The catalog: every app's manifest in one folder, `apps/` in the release's checkout, which is
//! `/usr/share/kubaos/apps/` on a machine.
//!
//! # Owns
//!
//! - [`Catalog`]: loading the folder, finding an app, the packages its apps install, and checking
//!   each app's vendor repository against the package manifest.
//!
//! # Not here
//!
//! - One manifest's format: `manifest.rs`.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use super::manifest::App;
use super::manifest::ManifestError;
use super::manifest::Source;
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
        source: ManifestError,
    },
    #[error("app {id} takes its packages from repo {repo}, which the package manifest doesn't define")]
    UnknownRepo { id: String, repo: String },
}

/// Every app KubaOS knows how to install, by id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Catalog {
    /// Sorted by id.
    apps: Vec<App>,
}

impl Catalog {
    /// The catalog of these apps.
    #[must_use]
    pub fn new(mut apps: Vec<App>) -> Self {
        apps.sort_by(|a, b| a.id.cmp(&b.id));
        Self { apps }
    }

    /// Reads every `<id>.toml` in `dir`; an empty catalog when the folder is missing, as in a
    /// release from before it. Anything else in the folder is passed over.
    ///
    /// # Errors
    ///
    /// [`CatalogError`] when the folder or a manifest can't be read, or a manifest isn't valid.
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
            let path = entry.map_err(read_err(dir))?.path();
            let Some(id) = path
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.strip_suffix(".toml"))
            else {
                continue;
            };
            let text = fs::read_to_string(&path).map_err(read_err(&path))?;
            let app = App::parse(id, &text).map_err(|source| CatalogError::App {
                id: id.to_owned(),
                source,
            })?;
            apps.push(app);
        }
        Ok(Self::new(apps))
    }

    /// Checks that every apt app's vendor repository is one of `manifest`'s.
    ///
    /// # Errors
    ///
    /// [`CatalogError::UnknownRepo`] for the first app whose repository isn't there.
    pub fn check_repos(&self, manifest: &Manifest) -> Result<(), CatalogError> {
        for app in &self.apps {
            if let Source::Apt { repo: Some(repo), .. } = &app.source
                && !manifest.repos().contains_key(repo)
            {
                return Err(CatalogError::UnknownRepo {
                    id: app.id.clone(),
                    repo: repo.clone(),
                });
            }
        }
        Ok(())
    }

    /// The apps, sorted by id.
    pub fn apps(&self) -> &[App] {
        &self.apps
    }

    /// The app `id`.
    pub fn get(&self, id: &str) -> Option<&App> {
        self.apps.iter().find(|a| a.id == id)
    }

    /// The dpkg packages its apt and `.deb` apps install: the catalog's, which the package
    /// manifest leaves alone.
    pub fn packages(&self) -> BTreeSet<String> {
        self.apps
            .iter()
            .flat_map(|app| match &app.source {
                Source::Apt { packages, .. } => packages.clone(),
                Source::Deb(deb) => vec![deb.package.clone()],
                Source::Nix { .. } => Vec::new(),
            })
            .collect()
    }

    /// The apps that install any of `packages`, in id order.
    pub fn installing<'a>(&'a self, packages: &'a [String]) -> impl Iterator<Item = &'a App> {
        self.apps.iter().filter(|app| match &app.source {
            Source::Apt { packages: own, .. } => own.iter().any(|p| packages.contains(p)),
            Source::Deb(deb) => packages.contains(&deb.package),
            Source::Nix { .. } => false,
        })
    }
}
