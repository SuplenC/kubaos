//! The outputs written from the tokens, one per toolkit, and checking that the checked-in ones
//! are current.
//!
//! Every output is a [`Target`]: a file in the repo and a pure function from the tokens to its
//! text. Writing, checking for staleness and the test that fails on a stale file all go through
//! [`TARGETS`], so an output added there is written by `kubaos-tokens write` and guarded from that
//! commit on.
//!
//! # Owns
//!
//! - The list of outputs, and comparing or replacing each one on disk.
//!
//! # Not here
//!
//! - What an output says: each target's own module ([`crate::rust`], [`crate::qml`]).

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use crate::model::Tokens;
use crate::qml::QmlSymbols;
use crate::qml::QmlTokens;
use crate::rust::RustModule;

/// One generated file.
pub trait Target: Sync {
    /// Where the file lives, relative to the repo's root.
    fn path(&self) -> &'static str;

    /// The file's whole text for `tokens`, reading any drawing it embeds from the repo at `root`.
    ///
    /// # Errors
    ///
    /// When a file the output embeds can't be read.
    fn render(&self, tokens: &Tokens, root: &Path) -> io::Result<String>;
}

/// Every output `kubaos-tokens` writes.
// TODO(#152): add the matugen templates from `color.source`; the terminals still use
// config/matugen. TODO(#153): add Plymouth's and GRUB's values; they still carry their own.
pub static TARGETS: &[&dyn Target] = &[&RustModule, &QmlTokens, &QmlSymbols];

/// The checked-in output at `path` doesn't match the tokens.
#[derive(Debug)]
pub struct Stale {
    pub path: PathBuf,
}

/// Each output under `root` whose text differs from what `tokens` render to, or that is missing.
///
/// # Errors
///
/// When an output exists but can't be read, or a file it embeds can't be.
pub fn stale(root: &Path, tokens: &Tokens) -> io::Result<Vec<Stale>> {
    let mut found = Vec::new();
    for target in TARGETS {
        let path = root.join(target.path());
        if read(&path)?.as_deref() != Some(target.render(tokens, root)?.as_str()) {
            found.push(Stale { path });
        }
    }
    Ok(found)
}

/// Writes every output under `root` that differs from what `tokens` render to, and returns the
/// paths it wrote.
///
/// # Errors
///
/// When an output can't be read or written.
pub fn write(root: &Path, tokens: &Tokens) -> io::Result<Vec<PathBuf>> {
    let mut written = Vec::new();
    for target in TARGETS {
        let path = root.join(target.path());
        let text = target.render(tokens, root)?;
        if read(&path)?.as_deref() == Some(text.as_str()) {
            continue;
        }
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(&path, text)?;
        written.push(path);
    }
    Ok(written)
}

/// The file's text, or `None` when there is no file.
fn read(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}
