//! The migration that wires the user's `home.nix` to the state directory: one setting,
//! `kubaos.stateDir = ./kubaos.d;`, which KubaOS's home module reads.
//!
//! It follows the rules for changing a user file: it only adds, running it again changes nothing,
//! and it backs the file up before writing. The backup goes to KubaOS's state folder
//! (`~/.local/state/kubaos/backups/`), outside the user flake, so that it never becomes part of the
//! flake's source in the Nix store. A `home.nix` whose last line is not the closing `}` of
//! its attribute set is left alone, with an error asking the user to add the line by hand.
//!
//! # Owns
//!
//! - Deciding whether `home.nix` is wired, and where the line goes.
//! - The backup and the atomic rewrite.
//!
//! # Not here
//!
//! - What the setting does: `home/state.nix`.
//! - New user flakes, which get the line from `user-flake/home.nix`.

use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use super::write_atomic;

/// The setting the migration adds, indented as the body of `home.nix`.
pub const STATE_DIR_LINE: &str = "  kubaos.stateDir = ./kubaos.d;";
/// The comment above it, so the user knows what the line is for.
const STATE_DIR_COMMENT: &str = "  # What `kubaos package add`, the Command menu and Kōbō install (kubaos.d/).";
/// The text that marks a `home.nix` as wired, on a line that is not a comment.
const MARKER: &str = "kubaos.stateDir";

/// What [`wire_home_nix`] did.
#[derive(Debug, PartialEq, Eq)]
pub enum Migration {
    /// The file already had the setting and was not touched.
    AlreadyWired,
    /// The setting was added, after copying the file to `backup`.
    Wired { backup: PathBuf },
}

/// Wiring `home.nix` failed. The file is as it was.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MigrateError {
    /// `home.nix` could not be read.
    #[error("reading {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// `home.nix` does not end with the `}` that closes its attribute set, so there is no safe
    /// place for the line.
    #[error("{} does not end with a lone `}}`; add `kubaos.stateDir = ./kubaos.d;` to it by hand", path.display())]
    NoClosingBrace { path: PathBuf },
    /// The backup could not be written.
    #[error("backing up {}", path.display())]
    Backup {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The wired file could not be written.
    #[error("writing {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

/// Whether `text` sets `kubaos.stateDir` on a line that is not a comment.
pub fn is_wired(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim_start();
        !line.starts_with('#') && line.contains(MARKER)
    })
}

/// `text` with the setting added above its closing `}`, or `None` when the last line that is
/// neither blank nor a comment is not a lone `}`.
pub fn wire(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let closing = lines.iter().rposition(|line| {
        let line = line.trim();
        !line.is_empty() && !line.starts_with('#')
    })?;
    if lines[closing].trim() != "}" {
        return None;
    }
    let mut wired = String::with_capacity(text.len() + 128);
    for line in &lines[..closing] {
        wired.push_str(line);
    }
    if !wired.is_empty() && !wired.ends_with('\n') {
        wired.push('\n');
    }
    wired.push('\n');
    wired.push_str(STATE_DIR_COMMENT);
    wired.push('\n');
    wired.push_str(STATE_DIR_LINE);
    wired.push('\n');
    for line in &lines[closing..] {
        wired.push_str(line);
    }
    Some(wired)
}

/// Adds the setting to the `home.nix` at `path` unless it is already there, backing the file up
/// first into `backups` (`Paths::backups`) as `home.nix.bak` (or `.1`, `.2`, … when that exists).
///
/// # Errors
///
/// [`MigrateError`] when the file cannot be read, has no place for the line, or the backup or the
/// write fails. The file is unchanged in every case.
pub fn wire_home_nix(path: &Path, backups: &Path) -> Result<Migration, MigrateError> {
    let text = fs::read_to_string(path).map_err(|source| MigrateError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if is_wired(&text) {
        return Ok(Migration::AlreadyWired);
    }
    let Some(wired) = wire(&text) else {
        return Err(MigrateError::NoClosingBrace {
            path: path.to_path_buf(),
        });
    };
    let backup = back_up(path, backups, &text)?;
    write_atomic(path, &wired).map_err(|source| MigrateError::Write {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(Migration::Wired { backup })
}

/// Writes `text` to the first free backup name for `path` in `backups`, never replacing an earlier
/// backup.
fn back_up(path: &Path, backups: &Path, text: &str) -> Result<PathBuf, MigrateError> {
    let backup_err = |source| MigrateError::Backup {
        path: path.to_path_buf(),
        source,
    };
    let mut name = path.file_name().unwrap_or("home.nix".as_ref()).to_owned();
    name.push(".bak");
    let base = backups.join(name);
    fs::create_dir_all(backups).map_err(backup_err)?;
    for n in 0u32.. {
        let candidate = if n == 0 {
            base.clone()
        } else {
            let mut name = base.as_os_str().to_owned();
            name.push(format!(".{n}"));
            PathBuf::from(name)
        };
        match OpenOptions::new().write(true).create_new(true).open(&candidate) {
            Ok(mut file) => {
                file.write_all(text.as_bytes())
                    .and_then(|()| file.sync_all())
                    .map_err(backup_err)?;
                return Ok(candidate);
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(backup_err(e)),
        }
    }
    Err(backup_err(io::Error::other("no free backup name")))
}

#[cfg(test)]
mod tests;
