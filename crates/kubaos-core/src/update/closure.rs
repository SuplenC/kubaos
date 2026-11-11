//! The home's Nix packages that a switch moved: `nix store diff-closures` of the home-manager
//! generation before the switch and the one after it.
//!
//! home-manager keeps a GC root to the generation it activated last,
//! `$XDG_STATE_HOME/home-manager/gcroots/current-home`; reading it before and after the switch
//! names both generations, so the diff needs no build of its own.
//!
//! # Owns
//!
//! - Reading `nix store diff-closures`' output ([`parse_diff`]).
//!
//! # Not here
//!
//! - Running it around the switch: the updater's `updater.rs`. Where the GC root is:
//!   `crate::state::Paths::home_generation`.

use super::event::PackageChange;

/// What `nix store diff-closures` writes for a version set that is empty: the package is new, or
/// gone.
const ABSENT: &str = "∅";

/// What it writes for a store path without a version.
const NO_VERSION: &str = "ε";

/// The packages whose versions `nix store diff-closures` says changed, in its order. A line that
/// only changes size (`kubaos: -858.7 KiB`) is a rebuild at the same version and left out, as is
/// anything that isn't a package's line. Nix colours the sizes even into a pipe, so escapes are
/// dropped first.
pub fn parse_diff(output: &str) -> Vec<PackageChange> {
    output
        .lines()
        .filter_map(|line| package_line(&strip_escapes(line)))
        .collect()
}

/// One `name: old → new[, size]` line. Versions are a comma-separated set, so the size is told
/// apart by its unit.
fn package_line(line: &str) -> Option<PackageChange> {
    let (name, rest) = line.split_once(": ")?;
    if name.is_empty() || name.contains(' ') {
        return None;
    }
    let (from, to) = rest.split_once(" → ")?;
    let mut to: Vec<&str> = to.split(", ").collect();
    if to.len() > 1 && to.last().is_some_and(|last| is_size(last)) {
        to.pop();
    }
    let change = PackageChange {
        name: name.to_owned(),
        from: versions(from),
        to: versions(&to.join(", ")),
    };
    (change.from.is_some() || change.to.is_some()).then_some(change)
}

/// A version set as the diff writes it, none for [`ABSENT`].
fn versions(text: &str) -> Option<String> {
    match text.trim() {
        ABSENT => None,
        text => Some(text.replace(NO_VERSION, "")),
    }
}

/// Whether `text` is a size the diff appends, as `+1.2 MiB` or `-68.3 KiB`.
fn is_size(text: &str) -> bool {
    text.ends_with("iB") && text.starts_with(|c: char| c.is_ascii_digit() || c == '+' || c == '-')
}

/// `line` without its ANSI escape sequences.
fn strip_escapes(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // A CSI sequence ends at its first letter.
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests;
