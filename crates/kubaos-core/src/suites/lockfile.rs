//! The Suites' entries in mise's global lockfile, which `kubaos update` saves before it moves
//! the Suites' tools and `kubaos rollback` puts back.
//!
//! Only the entries of the tools a Suite's fragment asks for are saved, and only those a
//! fragment still asks for, at a version its request still selects, are put back: mise leaves
//! an entry in the lockfile after the last request for it goes, and selects a locked version
//! even when the request has moved past it. Fragments and records are never touched here.
//!
//! # Owns
//!
//! - [`Suites::save_locked`] and [`Suites::restore_locked`], and the saved entries' file.
//!
//! # Not here
//!
//! - Where the saved entries go and when: `update/updater.rs`.
//! - The lockfile itself: mise writes it, [`crate::state::Paths::mise_lock`].

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use toml_edit::DocumentMut;
use toml_edit::Item;
use toml_edit::Table;

use super::SuiteError;
use super::Suites;
use super::catalog;
use super::on_line;
use crate::mise::ToolRequest;
use crate::state::write_atomic;

/// The header of a file of saved entries.
const SAVED_HEADER: &str = "# The lockfile entries of the Suites' tools as they were before a `kubaos update`, which\n\
                            # `kubaos rollback` puts back in ~/.config/mise/mise.lock.\n";

impl Suites {
    /// Copies the lockfile entries of the tools the Suites' fragments ask for to `to`. Writes
    /// nothing and returns `false` when no fragment asks for a tool the lockfile holds.
    ///
    /// # Errors
    ///
    /// A fragment, the lockfile or `to` can't be read, parsed or written.
    pub fn save_locked(&self, to: &Path) -> Result<bool, SuiteError> {
        let asked = self.asked()?;
        if asked.is_empty() {
            return Ok(false);
        }
        let Some(lockfile) = read_doc(&self.paths.mise_lock())? else {
            return Ok(false);
        };
        let mut saved = Table::new();
        saved.set_implicit(true);
        if let Some(tools) = lockfile.get("tools").and_then(Item::as_table_like) {
            for (tool, entry) in tools.iter().filter(|(tool, _)| asked.contains_key(*tool)) {
                saved.insert(tool, entry.clone());
            }
        }
        if saved.is_empty() {
            return Ok(false);
        }
        let mut doc = DocumentMut::new();
        doc.insert("tools", Item::Table(saved));
        let write_err = |source| SuiteError::Write {
            path: to.to_path_buf(),
            source,
        };
        if let Some(dir) = to.parent() {
            fs::create_dir_all(dir).map_err(write_err)?;
        }
        write_atomic(to, &format!("{SAVED_HEADER}{doc}")).map_err(write_err)?;
        Ok(true)
    }

    /// Puts the entries saved at `from` back in the lockfile, each only while a fragment still
    /// asks for its tool at a request one of its versions is on; installs what the lockfile then
    /// names and prunes the versions nothing selects any more. Returns the tools put back, at
    /// their locked versions, with the name of the Suite that asks for each; none, running no
    /// mise, when nothing was saved or none fits.
    ///
    /// # Errors
    ///
    /// A failed lock, read, parse, write or mise call.
    pub fn restore_locked(&self, from: &Path) -> Result<Vec<(&'static str, ToolRequest)>, SuiteError> {
        let _lock = self.lock()?;
        let Some(saved) = read_doc(from)? else {
            return Ok(Vec::new());
        };
        let asked = self.asked()?;
        let path = self.paths.mise_lock();
        let mut lockfile = read_doc(&path)?.unwrap_or_default();
        let mut restored = Vec::new();
        for (tool, entry) in saved
            .get("tools")
            .and_then(Item::as_table_like)
            .into_iter()
            .flat_map(|t| t.iter())
        {
            let Some((suite, request)) = asked.get(tool) else {
                continue;
            };
            let Some(version) = versions(entry).into_iter().find(|v| on_line(v, request)) else {
                continue;
            };
            lockfile["tools"][tool] = entry.clone();
            restored.push((
                *suite,
                ToolRequest {
                    tool: tool.to_owned(),
                    request: version,
                },
            ));
        }
        if restored.is_empty() {
            return Ok(restored);
        }
        write_atomic(&path, &lockfile.to_string()).map_err(|source| SuiteError::Write { path, source })?;
        self.mise.install_locked().map_err(SuiteError::Upgrade)?;
        self.mise.prune().map_err(SuiteError::Prune)?;
        Ok(restored)
    }

    /// Every tool a Suite's fragment asks for, with the Suite's name and the request.
    fn asked(&self) -> Result<BTreeMap<String, (&'static str, String)>, SuiteError> {
        let mut asked = BTreeMap::new();
        for suite in catalog::all() {
            for tool in self.fragment(suite)? {
                asked.insert(tool.tool, (suite.name, tool.request));
            }
        }
        Ok(asked)
    }
}

/// The TOML document at `path`, or `None` when there is no file.
fn read_doc(path: &Path) -> Result<Option<DocumentMut>, SuiteError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(SuiteError::Read(crate::state::ReadError::read(path)(e))),
    };
    text.parse().map(Some).map_err(|source| SuiteError::Lockfile {
        path: path.to_path_buf(),
        source,
    })
}

/// The versions a tool's lockfile entry names: one table per version (`[[tools.node]]`), or
/// one table.
fn versions(entry: &Item) -> Vec<String> {
    let version = |table: &dyn toml_edit::TableLike| table.get("version").and_then(Item::as_str).map(str::to_owned);
    match entry {
        Item::ArrayOfTables(tables) => tables.iter().filter_map(|t| version(t)).collect(),
        Item::Table(table) => version(table).into_iter().collect(),
        Item::Value(toml_edit::Value::InlineTable(table)) => version(table).into_iter().collect(),
        Item::Value(toml_edit::Value::Array(array)) => array
            .iter()
            .filter_map(|v| v.as_inline_table().and_then(|t| version(t)))
            .collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests;
