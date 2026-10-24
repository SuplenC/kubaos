//! Small readers for the kernel's text files: a whole file, or one number in it.
//!
//! # Owns
//!
//! - Reading a file under `/proc` or `/sys` to a string or a number, `None` when it isn't there.
//!
//! # Not here
//!
//! - What any file means: the module that reads it.

use std::fs;
use std::path::Path;
use std::str::FromStr;

/// The file's text, or `None` when it can't be read.
pub(crate) fn text(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

/// The file's text, trimmed, parsed as a `T`.
pub(crate) fn number<T: FromStr>(path: &Path) -> Option<T> {
    text(path)?.trim().parse().ok()
}

/// The file's first line, trimmed; `None` when missing or empty.
pub(crate) fn line(path: &Path) -> Option<String> {
    let text = text(path)?;
    let line = text.lines().next()?.trim();
    (!line.is_empty()).then(|| line.to_owned())
}

/// The names of the entries of `dir`, sorted, so a scan reads them in the same order each time.
pub(crate) fn entries(dir: &Path) -> Vec<String> {
    let Ok(read) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = read
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .collect();
    names.sort_unstable();
    names
}
