//! Migrations: one script per change a release makes to an installed machine, run once each.
//!
//! A migration is `migrations/system/<unix timestamp>-<slug>.sh` (run as root) or
//! `migrations/user/<unix timestamp>-<slug>.sh` (run as the user) in KubaOS's checkout. They run in
//! timestamp order, each under `bash -euo pipefail`, and a marker of the same name, in
//! `/var/lib/kubaos/migrations/` or `~/.local/state/kubaos/migrations/`, records that it ran. The
//! install writes a marker for every migration of the release it installs, so a fresh machine does
//! not replay them.
//!
//! # Owns
//!
//! - Which files are migrations, their order, which are pending, and naming a new one.
//!
//! # Not here
//!
//! - Running them and writing markers: `updater.rs`. Creating one: `just migration`.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

/// One migration script.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Migration {
    /// When it was written, the number its name starts with. It orders the migrations.
    pub stamp: u64,
    /// The file name, `1761130000-move-the-bar.sh`, which its marker is named after too.
    pub name: String,
    pub path: PathBuf,
}

/// A slug that can't name a migration.
#[derive(Debug, thiserror::Error)]
#[error("{0:?} can't name a migration: use a-z, 0-9 and -, starting with a letter or digit")]
pub struct InvalidSlug(String);

/// The timestamp of a migration's file name, or `None` when the name is not a migration's.
pub fn stamp_of(name: &str) -> Option<u64> {
    let stem = name.strip_suffix(".sh")?;
    let (stamp, slug) = stem.split_once('-')?;
    let digits = !stamp.is_empty() && stamp.bytes().all(|b| b.is_ascii_digit());
    if !digits || !is_slug(slug) {
        return None;
    }
    stamp.parse().ok()
}

/// The file name of a new migration written at `stamp`.
///
/// # Errors
///
/// [`InvalidSlug`] unless `slug` is a-z, 0-9 and `-`, not starting with `-`.
pub fn file_name(stamp: u64, slug: &str) -> Result<String, InvalidSlug> {
    if is_slug(slug) {
        Ok(format!("{stamp}-{slug}.sh"))
    } else {
        Err(InvalidSlug(slug.to_owned()))
    }
}

fn is_slug(slug: &str) -> bool {
    !slug.is_empty()
        && !slug.starts_with('-')
        && slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// The migrations in `dir`, oldest first. A missing folder has none, and files that are not
/// migrations are left out.
///
/// # Errors
///
/// When the folder exists but can't be read.
pub fn list(dir: &Path) -> io::Result<Vec<Migration>> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let mut migrations = Vec::new();
    for entry in entries {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if let Some(stamp) = stamp_of(&name)
            && entry.path().is_file()
        {
            migrations.push(Migration {
                stamp,
                path: entry.path(),
                name,
            });
        }
    }
    migrations.sort();
    Ok(migrations)
}

/// The names of the markers in `markers`. A missing folder has none.
///
/// # Errors
///
/// When the folder exists but can't be read.
pub fn markers(markers: &Path) -> io::Result<BTreeSet<String>> {
    let entries = match fs::read_dir(markers) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(BTreeSet::new()),
        Err(e) => return Err(e),
    };
    let mut names = BTreeSet::new();
    for entry in entries {
        if let Some(name) = entry?.file_name().to_str() {
            names.insert(name.to_owned());
        }
    }
    Ok(names)
}

/// The migrations in `dir` without a marker in `markers`, oldest first.
///
/// # Errors
///
/// When either folder exists but can't be read.
pub fn pending(dir: &Path, markers_dir: &Path) -> io::Result<Vec<Migration>> {
    let done = markers(markers_dir)?;
    Ok(list(dir)?.into_iter().filter(|m| !done.contains(&m.name)).collect())
}

#[cfg(test)]
mod tests;
