//! The workspace groups by desk position in the menu: one Item each, "Workspace group: position 1
//! from the left", saying which workspaces the position owns and which screen stands there, that
//! goes to the group. The list is the screens file Hyprland keeps,
//! `$XDG_STATE_HOME/kubaos/hypr/workspace-screens`; no file, as before Hyprland first ran, means
//! no Items.
//!
//! # Owns
//!
//! - Reading the screens file, the desk Items, and what their Action runs.
//!
//! # Not here
//!
//! - Writing the file, its format, and which group goes where: `config/hypr/workspaces.lua` and
//!   `config/hypr/workspace_blocks.lua`.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use kotodama_extension::Chain;
use kotodama_protocol::Action;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;

use crate::run::Runner;
use crate::toggles::Ran;
use crate::updates;

/// The Item kind of every desk Item.
pub const KIND: &str = "kubaos.desk";
/// Goes to the position's group.
pub const SHOW: &str = "show";
/// A position's Item is this and the position, `desk:panel`, `desk:1`.
const PREFIX: &str = "desk:";
/// The screens file, under the KubaOS state folder.
const FILE: &str = "hypr/workspace-screens";
/// The position the laptop panel has in the file.
const PANEL: &str = "panel";
/// Workspaces a block holds (`SLOTS` in `config/hypr/workspace_blocks.lua`).
const SLOTS: u32 = 10;

const ICON: &str = "video-display";

/// The desk positions Hyprland has seen, the panel first, then left to right.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Desk {
    places: Vec<Place>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Place {
    /// "panel", or "1", "2" and on from the left.
    position: String,
    block: u32,
    /// The connector of the screen standing there now; empty when none is.
    connector: String,
    /// The screen there now or last.
    screen: String,
}

impl Place {
    /// The panel first, then the externals from the left.
    fn order(&self) -> (bool, u32) {
        (self.position != PANEL, self.position.parse().unwrap_or(0))
    }
}

impl Desk {
    /// Reads the screens file `file`; none without it.
    pub fn read(file: &Path) -> Self {
        Self::parse(&fs::read_to_string(file).unwrap_or_default())
    }

    /// The positions in the file's text: a line each, its block, the connector of the screen
    /// there, the position and the screen, split by tabs. A line of a file keyed by screen, which
    /// Hyprland migrates on its next start, is skipped.
    ///
    /// A screen gets one Item: the position it stands at, else the first it waited at. A file from
    /// before Hyprland repaired it can name a screen at a position and again waiting at another,
    /// or name a position twice; the first line for a position stands.
    pub fn parse(text: &str) -> Self {
        let mut places: Vec<Place> = text
            .lines()
            .filter_map(|line| {
                let mut fields = line.split('\t');
                let block = fields.next()?.parse().ok()?;
                let connector = fields.next()?.to_owned();
                let position = fields.next().filter(|position| is_position(position))?.to_owned();
                let screen = fields.next().unwrap_or_default().to_owned();
                Some(Place {
                    position,
                    block,
                    connector,
                    screen,
                })
            })
            .collect();
        places.dedup_by(|later, earlier| later.position == earlier.position);
        places.sort_by_key(Place::order);
        let present: Vec<String> = places
            .iter()
            .filter(|place| !place.connector.is_empty())
            .map(|place| place.screen.clone())
            .collect();
        let mut listed: Vec<String> = Vec::new();
        places.retain(|place| {
            if place.connector.is_empty() && (present.contains(&place.screen) || listed.contains(&place.screen)) {
                return false;
            }
            listed.push(place.screen.clone());
            true
        });
        Desk { places }
    }

    /// One Item a position.
    pub fn items(&self) -> Vec<Item> {
        self.places.iter().map(item).collect()
    }

    /// Runs `action` on the desk Item `id`; the error is what the user reads.
    ///
    /// # Errors
    ///
    /// When there is no such Item or Action, or `hyprctl` can't be started.
    pub fn run(&self, id: &str, action: &str, runner: &impl Runner) -> Result<Ran, String> {
        let unknown = || format!("`{id}` has no Action `{action}`");
        if action != SHOW {
            return Err(unknown());
        }
        let place = self
            .places
            .iter()
            .find(|place| item_id(place) == id)
            .ok_or_else(unknown)?;
        let argv = [
            "hyprctl".to_owned(),
            "dispatch".to_owned(),
            format!("dot.workspaces.go_to_position(\"{}\")", place.position),
        ];
        runner
            .launch(&argv, Duration::ZERO)
            .map_err(|err| Chain(&err).to_string())?;
        Ok(Ran::Close)
    }
}

/// The screens file, `$XDG_STATE_HOME/kubaos/hypr/workspace-screens`, else under
/// `~/.local/state`; none without either.
pub fn file() -> Option<PathBuf> {
    updates::state_dir().map(|dir| dir.join(FILE))
}

fn is_position(field: &str) -> bool {
    field == PANEL || (!field.is_empty() && field.bytes().all(|byte| byte.is_ascii_digit()))
}

fn item_id(place: &Place) -> String {
    format!("{PREFIX}{}", place.position)
}

fn title(place: &Place) -> String {
    if place.position == PANEL {
        "Workspace group: laptop panel".to_owned()
    } else {
        format!("Workspace group: position {} from the left", place.position)
    }
}

fn subtitle(place: &Place) -> String {
    let first = place.block * SLOTS + 1;
    let last = first + SLOTS - 1;
    if place.connector.is_empty() {
        format!(
            "Workspaces {first}-{last}, waiting for a screen here; last on {}",
            place.screen
        )
    } else {
        format!("Workspaces {first}-{last}, on {} ({})", place.screen, place.connector)
    }
}

fn item(place: &Place) -> Item {
    Item {
        id: item_id(place),
        kind: ItemKind::Namespaced(NamespacedKind::new(KIND).expect("the Extension's kinds are valid")),
        identity: None,
        title: title(place),
        subtitle: Some(subtitle(place)),
        icon: Some(ICON.to_owned()),
        keyword: None,
        keywords: [
            "workspace group",
            "desk",
            "screen",
            "monitor",
            "display",
            "position",
            "where are my windows",
        ]
        .map(str::to_owned)
        .to_vec(),
        hidden: false,
        app_only: false,
        actions: vec![Action::new(SHOW, "Go to it")],
    }
}

#[cfg(test)]
mod tests;
