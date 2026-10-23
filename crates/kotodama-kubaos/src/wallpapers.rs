//! The wallpaper sets in the menu: one Item a set on this machine, to show it in the wallpaper
//! picker or hide it from there. What the picker shows is the sets file,
//! `~/.config/kubaos/wallpapers/sets`: a set's id for all of it, `<set>/<file>` for one of its
//! wallpapers, and every set without the file.
//!
//! A change is checked here, so a refusal reads in the menu, then made by `dot-theme show-set`
//! or `hide-set`, which own the file; its change comes back through the watch in `main.rs`.
//!
//! # Owns
//!
//! - Reading the sets on disk and the sets file, the set Items, and what their Actions run.
//!
//! # Not here
//!
//! - Writing the sets file, and what the picker shows from it: `scripts/dot-theme`.
//! - The picker itself: `config/quickshell/services/Wallpapers.qml`.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use kotodama_extension::Chain;
use kotodama_protocol::Action;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;

use crate::run::Runner;
use crate::toggles::Ran;

/// The Item kind of every set Item.
pub const KIND: &str = "kubaos.wallpaper-set";
/// Shows the whole set in the picker.
pub const SHOW: &str = "show";
/// Takes the set out of the picker.
pub const HIDE: &str = "hide";
/// A set's Item is this and its id, `wallpaper-set:shinjuku`.
const PREFIX: &str = "wallpaper-set:";

/// Where the sets are, one folder each, when `KUBAOS_WALLPAPERS` doesn't say; `scripts/dot-theme`
/// reads the same variable.
pub const DEFAULT_SETS_DIR: &str = "/usr/share/kubaos/wallpapers";
/// The sets file, under the config folder.
const SETS_FILE: &str = "kubaos/wallpapers/sets";
/// The picker's folder of the user's own wallpapers, which is no set.
const OWN_FOLDER: &str = "yours";

const ICON: &str = "folder-pictures";

/// The sets on this machine, by id, with how much of each the picker shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WallpaperSets {
    sets: BTreeMap<String, Shown>,
}

/// How much of a set the picker shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shown {
    Whole,
    /// This many of its wallpapers, picked one at a time.
    Picked(usize),
    Hidden,
}

impl WallpaperSets {
    /// Reads the set folders in `dir` and the sets file `file`. A missing `dir` leaves no sets,
    /// and a missing `file` shows every set.
    pub fn read(dir: &Path, file: &Path) -> Self {
        let on_disk = fs::read_dir(dir).into_iter().flatten().flatten().filter_map(|entry| {
            // `is_dir` follows the links, as the sets are a link into the Nix store.
            let is_set = entry.path().is_dir();
            is_set.then(|| entry.file_name().to_string_lossy().into_owned())
        });
        Self::parse(on_disk, fs::read_to_string(file).ok().as_deref())
    }

    /// The sets `on_disk`, shown as the sets file's text `file` says; all of each without it.
    pub fn parse(on_disk: impl IntoIterator<Item = String>, file: Option<&str>) -> Self {
        let entries: Vec<&str> = file
            .unwrap_or_default()
            .lines()
            .filter_map(|line| line.split_whitespace().next())
            .filter(|entry| !entry.starts_with('#'))
            .collect();
        let shown = |id: &str| {
            if file.is_none() || entries.contains(&id) {
                return Shown::Whole;
            }
            let picked = entries
                .iter()
                .filter(|entry| entry.split_once('/').is_some_and(|(set, _)| set == id))
                .count();
            if picked > 0 {
                Shown::Picked(picked)
            } else {
                Shown::Hidden
            }
        };
        WallpaperSets {
            sets: on_disk
                .into_iter()
                .filter(|id| id != OWN_FOLDER)
                .map(|id| {
                    let shown = shown(&id);
                    (id, shown)
                })
                .collect(),
        }
    }

    /// One Item a set, in id order.
    pub fn items(&self) -> Vec<Item> {
        self.sets.iter().map(|(id, shown)| item(id, *shown)).collect()
    }

    /// Runs `action` on the set Item `id`; the error is what the user reads.
    ///
    /// # Errors
    ///
    /// When there is no such Item or Action, the set is the last one the picker shows, or
    /// `dot-theme` fails.
    pub fn run(&self, id: &str, action: &str, runner: &impl Runner) -> Result<Ran, String> {
        let unknown = || format!("`{id}` has no Action `{action}`");
        let set = id.strip_prefix(PREFIX).ok_or_else(unknown)?;
        let shown = *self
            .sets
            .get(set)
            .ok_or_else(|| format!("`{set}` isn't a wallpaper set on this machine"))?;
        if !actions(shown).iter().any(|offered| offered.id == action) {
            return Err(unknown());
        }
        let word = if action == HIDE {
            if self.shown() == 1 {
                return Err("At least one wallpaper set stays in the picker. Nice try".to_owned());
            }
            "hide-set"
        } else {
            "show-set"
        };
        let argv = ["dot-theme", word, set].map(str::to_owned);
        runner.output(&argv).map_err(|err| Chain(&err).to_string())?;
        Ok(Ran::KeepOpen)
    }

    /// How many sets the picker shows, whole or in part.
    fn shown(&self) -> usize {
        self.sets.values().filter(|shown| **shown != Shown::Hidden).count()
    }
}

/// The folder of the sets: `KUBAOS_WALLPAPERS`, else [`DEFAULT_SETS_DIR`].
pub fn sets_dir() -> PathBuf {
    std::env::var_os("KUBAOS_WALLPAPERS")
        .filter(|value| !value.is_empty())
        .map_or_else(|| PathBuf::from(DEFAULT_SETS_DIR), PathBuf::from)
}

/// The sets file, `$XDG_CONFIG_HOME/kubaos/wallpapers/sets` or under `~/.config`.
pub fn sets_file() -> Option<PathBuf> {
    let var = |name| {
        std::env::var_os(name)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    };
    var("XDG_CONFIG_HOME")
        .or_else(|| var("HOME").map(|home| home.join(".config")))
        .map(|config| config.join(SETS_FILE))
}

fn actions(shown: Shown) -> Vec<Action> {
    match shown {
        Shown::Whole => vec![Action::new(HIDE, "Hide")],
        Shown::Picked(_) => vec![Action::new(SHOW, "Show all of it"), Action::new(HIDE, "Hide")],
        Shown::Hidden => vec![Action::new(SHOW, "Show")],
    }
}

fn item(id: &str, shown: Shown) -> Item {
    let subtitle = match shown {
        Shown::Whole => "In the wallpaper picker, all of it".to_owned(),
        Shown::Picked(count) => format!("In the wallpaper picker, {count} of its wallpapers"),
        Shown::Hidden => "Hidden from the wallpaper picker".to_owned(),
    };
    Item {
        id: format!("{PREFIX}{id}"),
        kind: ItemKind::Namespaced(NamespacedKind::new(KIND).expect("the Extension's kinds are valid")),
        identity: None,
        title: format!("Wallpaper set: {}", name(id)),
        subtitle: Some(subtitle),
        icon: Some(ICON.to_owned()),
        keyword: None,
        keywords: ["wallpaper sets", "choose wallpapers", "picker", "theme", "background"]
            .map(str::to_owned)
            .to_vec(),
        hidden: false,
        app_only: false,
        actions: actions(shown),
    }
}

/// A set's name from its id: `neo-tokyo` is "Neo tokyo".
fn name(id: &str) -> String {
    let spaced = id.replace(['-', '_'], " ");
    let mut chars = spaced.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
