//! Whether `[contrast]` covers every colour role the front ends draw with.
//!
//! The pairs themselves are judged by `kubaos-tokens`. What it can't know is whether a front end
//! draws a role the list never mentions: this reads every QML and iced file the lint reads and
//! finds each role named as a colour, `Tokens.color.<role>` in QML and `c.<role>` or
//! `colors.<role>` in Rust. A role that is in no pair, as foreground or background, and isn't
//! listed as decoration, is drawn with a contrast nobody checked.
//!
//! # Owns
//!
//! - Finding the roles drawn and not listed ([`unlisted`]).
//!
//! # Not here
//!
//! - The ratios and their minimums: `kubaos_tokens::contrast`. Which files are read:
//!   [`crate::lint::files`].

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use kubaos_tokens::Tokens;

use crate::Error;
use crate::lint;
use crate::lint::source::Language;

/// The specimen's colour page, which draws every role as a swatch: it shows the roles, it doesn't
/// draw with them.
const SWATCHES: &str = "crates/kubaos-specimen/src/pages/colour.rs";

/// A role a front end draws with that no contrast pair covers, and the first place it is named.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unlisted {
    pub role: String,
    /// `path:line`.
    pub at: String,
}

/// Every role named as a colour under `root` that is in no pair of `tokens` and isn't decoration.
///
/// # Errors
///
/// When a file the lint reads can't be read.
pub fn unlisted(root: &Path, tokens: &Tokens) -> Result<Vec<Unlisted>, Error> {
    let covered: BTreeSet<&str> = tokens
        .contrast
        .pairs
        .iter()
        .flat_map(|pair| [pair.fg.as_str(), pair.bg.as_str()])
        .chain(tokens.contrast.decorative.iter().map(String::as_str))
        .collect();
    let roles: BTreeSet<&str> = tokens.color.source.keys().map(String::as_str).collect();
    let mut first: BTreeMap<String, String> = BTreeMap::new();
    for (path, language) in lint::files(root)? {
        if path == SWATCHES {
            continue;
        }
        let full = root.join(&path);
        let text = fs::read_to_string(&full).map_err(|source| Error::Read { path: full, source })?;
        for (line, role) in named_roles(&text, language) {
            if roles.contains(role) && !covered.contains(role) {
                first.entry(role.to_owned()).or_insert_with(|| format!("{path}:{line}"));
            }
        }
    }
    Ok(first.into_iter().map(|(role, at)| Unlisted { role, at }).collect())
}

/// Each name that follows a colour prefix in `text`, with its line, from 1.
fn named_roles(text: &str, language: Language) -> Vec<(usize, &str)> {
    let prefixes: &[&str] = match language {
        Language::Qml => &["Tokens.color."],
        Language::Rust => &["c.", "colors."],
    };
    let mut found = Vec::new();
    for (number, line) in text.lines().enumerate() {
        for prefix in prefixes {
            for (at, _) in line.match_indices(prefix) {
                let before = line[..at].chars().next_back();
                if language == Language::Rust && before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                    continue;
                }
                let rest = &line[at + prefix.len()..];
                let end = rest
                    .find(|c: char| !(c.is_ascii_lowercase() || c == '_'))
                    .unwrap_or(rest.len());
                if end > 0 {
                    found.push((number + 1, &rest[..end]));
                }
            }
        }
    }
    found
}

#[cfg(test)]
mod tests;
