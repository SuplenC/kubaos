//! What the daily update check found, as the KubaOS update actions show it:
//! `kubaos update --check` writes `~/.local/state/kubaos/update-available`,
//! `key=value` lines, empty when there is nothing to install
//! (`kubaos_core::update::notice` owns the format).
//!
//! - `update` puts what `kubaos update` would install in its subtitle.
//! - `upgrade` is listed only when a next series is out, titled with its codename.
//!
//! # Owns
//!
//! - Reading the file, and how it changes those two Items.
//!
//! # Not here
//!
//! - The actions themselves: `config/kotodama/kubaos.toml`, through [`crate::registry`].
//! - Watching the file: `main.rs`.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use kotodama_protocol::Item;

/// The action that runs `kubaos update`.
pub const UPDATE: &str = "update";
/// The action that runs `kubaos upgrade`.
pub const UPGRADE: &str = "upgrade";
/// The file in [`state_dir`] the check writes.
pub const FILE: &str = "update-available";

/// What the check found waiting.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pending {
    /// "main at 1a2b3c4 (0.1.1)" or "0.1.2".
    update: Option<String>,
    /// The next series' version, "0.2.0".
    upgrade: Option<String>,
    upgrade_codename: Option<String>,
}

impl Pending {
    /// Reads the check's file; nothing is waiting when it isn't there.
    pub fn read(path: &Path) -> Self {
        fs::read_to_string(path)
            .map(|text| Self::parse(&text))
            .unwrap_or_default()
    }

    /// Parses the check's `key=value` lines, skipping keys it doesn't use.
    pub fn parse(text: &str) -> Self {
        let get = |key: &str| {
            text.lines()
                .filter_map(|line| line.split_once('='))
                .find(|(k, _)| *k == key)
                .map(|(_, value)| value.trim().to_owned())
                .filter(|value| !value.is_empty())
        };
        Pending {
            update: get("update"),
            upgrade: get("upgrade"),
            upgrade_codename: get("upgrade-codename"),
        }
    }

    /// `item` as the check makes it, or `None` for the upgrade Item when no
    /// next series is out.
    pub fn apply(&self, mut item: Item) -> Option<Item> {
        match item.id.as_str() {
            UPDATE => {
                if let Some(update) = &self.update {
                    item.subtitle = Some(format!("KubaOS {update} is ready. Update when you feel like it"));
                }
                Some(item)
            }
            UPGRADE => {
                let version = self.upgrade.as_ref()?;
                let name = self
                    .upgrade_codename
                    .as_deref()
                    .map_or_else(|| version.clone(), capitalise);
                item.title = format!("Upgrade to {name}");
                item.subtitle = Some(format!("The next KubaOS release, {version}"));
                Some(item)
            }
            _ => Some(item),
        }
    }
}

/// KubaOS's state folder, `$XDG_STATE_HOME/kubaos` or `~/.local/state/kubaos`.
pub fn state_dir() -> Option<PathBuf> {
    let var = |name| {
        std::env::var_os(name)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    };
    var("XDG_STATE_HOME")
        .or_else(|| var("HOME").map(|home| home.join(".local/state")))
        .map(|state| state.join("kubaos"))
}

fn capitalise(word: &str) -> String {
    let mut chars = word.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
