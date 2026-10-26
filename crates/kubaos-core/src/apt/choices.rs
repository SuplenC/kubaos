//! The user's choices of optional package groups, `/etc/kubaos/apt-groups`: one `<group>=on` or
//! `<group>=off` line per group the user decided, the rest following the manifest's default. The
//! install's flags write it (`--no-docker`, `--piv`), and so does `kubaos packages enable|disable`.
//!
//! # Owns
//!
//! - [`Choices`], [`Switch`], and the file's format.
//!
//! # Not here
//!
//! - Writing it as root: the updater, and `choice_set` in `system/manifest.sh`.

use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Write as _;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;

/// The choices, on the machine.
pub const CHOICES: &str = "etc/kubaos/apt-groups";

/// The comment the file starts with.
const HEADER: &str = "\
# Optional apt package groups turned on or off (system/packages.toml).
# Change them with `kubaos packages enable|disable <group>`.
";

/// On or off.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Switch {
    On,
    Off,
}

impl fmt::Display for Switch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::On => "on",
            Self::Off => "off",
        })
    }
}

/// A word that is neither `on` nor `off`.
#[derive(Debug, thiserror::Error)]
#[error("{0:?} is neither on nor off")]
pub struct InvalidSwitch(String);

impl FromStr for Switch {
    type Err = InvalidSwitch;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            other => Err(InvalidSwitch(other.to_owned())),
        }
    }
}

/// The groups the user turned on or off.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Choices(BTreeMap<String, Switch>);

impl Choices {
    /// Reads the file's lines. A line that isn't `<group>=on|off` is skipped, so a damaged file
    /// leaves the groups at their defaults rather than stopping an update.
    pub fn parse(text: &str) -> Self {
        let choices = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .filter_map(|line| {
                let (name, switch) = line.split_once('=')?;
                Some((name.trim().to_owned(), switch.parse().ok()?))
            })
            .collect();
        Self(choices)
    }

    /// The file's text, the groups sorted.
    pub fn to_text(&self) -> String {
        let mut text = HEADER.to_owned();
        for (name, switch) in &self.0 {
            let _ = writeln!(text, "{name}={switch}");
        }
        text
    }

    /// The user's choice for `group`, if they made one.
    pub fn get(&self, group: &str) -> Option<Switch> {
        self.0.get(group).copied()
    }

    /// Records `switch` for `group`.
    pub fn set(&mut self, group: impl Into<String>, switch: Switch) {
        self.0.insert(group.into(), switch);
    }
}

#[cfg(test)]
mod tests;
