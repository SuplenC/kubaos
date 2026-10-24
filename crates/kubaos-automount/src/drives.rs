//! The drives file: the removable drives with something mounted, which the Command menu lists as
//! "Eject" Items. The service rewrites it on every change udisks2 reports.
//!
//! One line a drive, its fields split by tabs: the drive's udisks2 object path, its name, then
//! each mount point. A name's tabs and newlines become spaces; a mount point holding either is
//! left out, as a reader couldn't split it back.
//!
//! # Owns
//!
//! - [`render`], the file's text, and [`write`], which replaces the file atomically.
//!
//! # Not here
//!
//! - Which drives: [`crate::block::mounted_drives`].
//! - Reading it for the menu: `crates/kotodama-kubaos/src/drives.rs`.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use crate::block::MountedDrive;

/// The file's name in `$XDG_RUNTIME_DIR`. Straight in it, so the menu's watch has a folder to
/// follow before the service first writes.
pub const FILE_NAME: &str = "kubaos-drives";

/// The drives file in `$XDG_RUNTIME_DIR`; none when it isn't set.
pub fn path() -> Option<PathBuf> {
    std::env::var_os("XDG_RUNTIME_DIR")
        .filter(|dir| !dir.is_empty())
        .map(|dir| PathBuf::from(dir).join(FILE_NAME))
}

/// The file's text for `drives`.
pub fn render(drives: &[MountedDrive]) -> String {
    let mut text = String::new();
    for drive in drives {
        let mounts: Vec<&str> = drive
            .mount_points
            .iter()
            .filter_map(|mount| mount.to_str())
            .filter(|mount| !mount.contains(['\t', '\n']))
            .collect();
        if mounts.is_empty() {
            continue;
        }
        text.push_str(&drive.drive);
        text.push('\t');
        text.push_str(&drive.name.replace(['\t', '\n'], " "));
        for mount in mounts {
            text.push('\t');
            text.push_str(mount);
        }
        text.push('\n');
    }
    text
}

/// Replaces `file` with the text for `drives`, through a temporary file and a rename, so a reader
/// never sees half of it.
///
/// # Errors
///
/// When the file can't be written.
pub fn write(file: &Path, drives: &[MountedDrive]) -> io::Result<()> {
    let text = render(drives);
    if fs::read_to_string(file).is_ok_and(|old| old == text) {
        return Ok(());
    }
    let temporary = file.with_extension("new");
    fs::write(&temporary, text)?;
    fs::rename(&temporary, file)
}

#[cfg(test)]
mod tests;
