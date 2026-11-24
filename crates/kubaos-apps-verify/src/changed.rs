//! What a branch changed, for `--changed`: an outage on a pin the branch changed fails, since
//! nothing has checked that pin yet and a pull request that merges green would carry it unchecked.
//! A pin the branch didn't touch was checked when it came in, so its outage only warns.
//!
//! # Owns
//!
//! - [`Changed`]: the catalog and package manifest at the merge base, read from git, and which
//!   actions and vendor repositories differ from them.
//!
//! # Not here
//!
//! - Which apps a branch reaches, and the table: `main.rs`. How a row is escalated: `check.rs`.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use kubaos_core::apps::Catalog;
use kubaos_core::apps::action::Kind;
use kubaos_core::apps::catalog::CATALOG;
use kubaos_core::apt::Manifest;

/// Where the vendor repositories' sources files are, in a checkout.
const SOURCES: &str = "system/files/apt/sources.list.d";

/// The merge base's catalog and the repositories the branch changed.
#[derive(Debug)]
pub struct Changed {
    /// None when the merge base's catalog can't be read: every action then counts as changed,
    /// so an outage fails rather than letting an unchecked pin through.
    base: Option<Catalog>,
    /// The repositories whose pin or sources file differs from the merge base's.
    repos: BTreeSet<String>,
}

impl Changed {
    /// What the checkout at `root` changed since `merge_base`, a commit, given its `manifest`.
    pub fn read(root: &Path, merge_base: &str, manifest: &Manifest) -> Self {
        let base_manifest = show(root, merge_base, "system/packages.toml")
            .and_then(|text| Manifest::parse(&String::from_utf8_lossy(&text)).ok());
        let repos = manifest
            .repos()
            .iter()
            .filter(|(name, pin)| {
                let path = format!("{SOURCES}/{name}.sources");
                let same_pin = base_manifest
                    .as_ref()
                    .and_then(|base| base.repos().get(*name))
                    .is_some_and(|base| base == *pin);
                let same_sources = show(root, merge_base, &path)
                    .zip(fs::read(root.join(&path)).ok())
                    .is_some_and(|(base, head)| base == head);
                !(same_pin && same_sources)
            })
            .map(|(name, _)| name.clone())
            .collect();
        Self {
            base: base_catalog(root, merge_base),
            repos,
        }
    }

    /// Whether the app `id`'s action `kind` is new on the branch: the merge base's app has no
    /// action of the same kind with the same values.
    pub fn action(&self, id: &str, kind: &Kind) -> bool {
        let Some(app) = self.base.as_ref().and_then(|base| base.get(id)) else {
            return true;
        };
        !app.actions
            .iter()
            .chain(app.setups.iter().flat_map(|setup| &setup.actions))
            .any(|action| action.kind == *kind)
    }

    /// Whether the repository `name`'s pin or sources file changed on the branch.
    pub fn repo(&self, name: &str) -> bool {
        self.repos.contains(name)
    }
}

/// The file at `path` in `commit`, none when it isn't there or git fails.
fn show(root: &Path, commit: &str, path: &str) -> Option<Vec<u8>> {
    let output = kubaos_git::at(root)
        .args(["show", &format!("{commit}:{path}")])
        .output()
        .ok()?;
    output.status.success().then_some(output.stdout)
}

/// The catalog as it is in `commit`, read from a copy of its `apps/` in a temporary folder.
fn base_catalog(root: &Path, commit: &str) -> Option<Catalog> {
    let dir = tempfile::tempdir().ok()?;
    let archive = dir.path().join("apps.tar");
    let archived = kubaos_git::at(root)
        .args(["archive", "--format=tar", "-o"])
        .arg(&archive)
        .args([commit, "--", CATALOG])
        .status()
        .ok()?;
    if !archived.success() {
        return None;
    }
    let unpacked = std::process::Command::new("tar")
        .arg("-xf")
        .arg(&archive)
        .arg("-C")
        .arg(dir.path())
        .status()
        .ok()?;
    if !unpacked.success() {
        return None;
    }
    Catalog::load(&dir.path().join(CATALOG)).ok()
}

#[cfg(test)]
mod tests;
