//! What KubaOS installed, `/var/lib/kubaos/apt-installed`: one `<package><TAB><group>` line per
//! apt package an install or an update installed because a group listed it and the machine
//! lacked it. A package the user had first is never in it, so KubaOS never removes one.
//!
//! # Owns
//!
//! - [`Record`] and the file's format.
//!
//! # Not here
//!
//! - Writing it: the updater, and `group_install` in `system/manifest.sh`.

use std::collections::BTreeMap;
use std::fmt::Write as _;

/// The record, on the machine.
pub const RECORD: &str = "var/lib/kubaos/apt-installed";

/// The comment the file starts with.
const HEADER: &str = "\
# The apt packages KubaOS installed, and the group of system/packages.toml each was for.
# kubaos update removes one only when no group lists it any more.
";

/// The packages KubaOS installed, each with its group.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Record(BTreeMap<String, String>);

impl Record {
    /// Reads the file's lines. A package listed twice keeps its first group, as the scripts
    /// append.
    pub fn parse(text: &str) -> Self {
        let mut packages = BTreeMap::new();
        for line in text.lines().map(str::trim_end) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (package, group) = line.split_once('\t').unwrap_or((line, ""));
            packages
                .entry(package.trim().to_owned())
                .or_insert_with(|| group.trim().to_owned());
        }
        Self(packages)
    }

    /// The file's text, sorted by package.
    pub fn to_text(&self) -> String {
        let mut text = HEADER.to_owned();
        for (package, group) in &self.0 {
            let _ = writeln!(text, "{package}\t{group}");
        }
        text
    }

    /// Whether KubaOS installed `package`.
    pub fn has(&self, package: &str) -> bool {
        self.0.contains_key(package)
    }

    /// The group KubaOS installed `package` for.
    pub fn group_of(&self, package: &str) -> Option<&str> {
        self.0.get(package).map(String::as_str)
    }

    /// The packages, sorted, each with its group.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(p, g)| (p.as_str(), g.as_str()))
    }

    /// Records that KubaOS installed `package` for `group`.
    pub fn add(&mut self, package: impl Into<String>, group: impl Into<String>) {
        self.0.insert(package.into(), group.into());
    }

    /// Forgets `package`, which is gone.
    pub fn remove(&mut self, package: &str) {
        self.0.remove(package);
    }
}

#[cfg(test)]
mod tests;
