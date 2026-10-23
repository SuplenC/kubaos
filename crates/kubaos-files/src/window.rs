//! The window: yazi in Ghostty, on the folders and files asked for. yazi opens a folder inside
//! it, and a file in its folder with the file hovered; several entries open as tabs.
//!
//! # Owns
//!
//! - The command line, and starting it in a scope of its own.
//!
//! # Not here
//!
//! - Which paths: [`crate::entry`].
//! - The scope: `kotodama_extension::launch`.

use std::io;
use std::path::Path;
use std::path::PathBuf;

use kotodama_extension::launch;

/// The terminal yazi runs in, and the flag that takes the command to run: the primary terminal
/// (`docs/CONTRACTS.md`, "Hyprland conventions"), as the SUPER + SHIFT + F bind starts it.
pub const TERMINAL: [&str; 2] = ["ghostty", "-e"];

/// The command line that opens yazi on `entries`.
pub fn argv(entries: &[PathBuf]) -> Vec<String> {
    let mut argv: Vec<String> = TERMINAL.iter().map(|word| (*word).to_owned()).collect();
    argv.push("yazi".to_owned());
    argv.push("--".to_owned());
    argv.extend(entries.iter().map(|entry| utf8(entry)));
    argv
}

/// Opens one window on `entries`, which stays when this process exits.
///
/// # Errors
///
/// When the terminal can't be started; the error says why in words for the user.
pub fn open(entries: &[PathBuf]) -> io::Result<()> {
    launch::launch(&argv(entries), None)
}

/// `path` as text, for the launcher, which takes UTF-8. A path that isn't UTF-8 becomes its nearest
/// folder that is, so yazi still opens close to it.
fn utf8(path: &Path) -> String {
    path.ancestors()
        .find_map(Path::to_str)
        .filter(|text| !text.is_empty())
        .unwrap_or("/")
        .to_owned()
}

#[cfg(test)]
mod tests;
