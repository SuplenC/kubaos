//! The workspace groups in the menu. One Item a desk position, "Workspace group 1: Left", saying
//! which workspaces the position owns and which screen stands there, that goes to the group; and
//! the Items that pin screens to groups and name them: "Pin <screen> to group 2" for each screen
//! plugged in, to each group and to a new one, "Unpin <screen>" for each screen pinned, and
//! "Name group 2…".
//!
//! The positions are the screens file Hyprland keeps, `$XDG_STATE_HOME/kubaos/hypr/
//! workspace-screens`; no file, as before Hyprland first ran, means no Items. The pins are the
//! user's groups file ([`groups`]); one that doesn't read gives no pin or name Items, as
//! Hyprland pins nothing then and says why in a toast.
//!
//! # Owns
//!
//! - Reading the screens file, the desk Items, and what their Actions run.
//!
//! # Not here
//!
//! - Writing the screens file, its format, and which group goes where: `config/hypr/workspaces.lua`
//!   and `config/hypr/workspace_blocks.lua`.
//! - Reading and editing the groups file: [`groups`].

pub mod groups;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use kotodama_extension::Chain;
use kotodama_protocol::Action;
use kotodama_protocol::Argument;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;

use crate::desk::groups::Group;
use crate::run::Runner;
use crate::toggles::Ran;
use crate::updates;

/// The Item kind of every desk Item.
pub const KIND: &str = "kubaos.desk";
/// Goes to the position's group.
pub const SHOW: &str = "show";
/// Pins a screen to a group.
pub const PIN: &str = "pin";
/// Takes a screen's pin away.
pub const UNPIN: &str = "unpin";
/// Names a group, with the name in [`NAME_ARGUMENT`].
pub const NAME: &str = "name";
/// The name a "Name group N…" Item asks for; blank takes the name away.
pub const NAME_ARGUMENT: &str = "name";
/// A position's Item is this and the position, `desk:panel`, `desk:1`.
const PREFIX: &str = "desk:";
/// The screens file, under the KubaOS state folder.
const FILE: &str = "hypr/workspace-screens";
/// The position the laptop panel has in the file.
const PANEL: &str = "panel";
/// Workspaces a block holds (`SLOTS` in `config/hypr/workspace_blocks.lua`).
const SLOTS: u32 = 10;

const ICON: &str = "video-display";

/// The desk positions Hyprland has seen, the panel first, then by number, and the user's groups.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Desk {
    places: Vec<Place>,
    /// The groups file, once given, and what it holds, or why it doesn't read.
    groups: Option<(PathBuf, Result<Vec<Group>, String>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Place {
    /// "panel", or "1", "2" and on.
    position: String,
    block: u32,
    /// The connector of the screen standing there now; empty when none is.
    connector: String,
    /// The screen there now or last.
    screen: String,
    /// The name the user gave the position's group; empty without one.
    name: String,
}

impl Place {
    /// The panel first, then the externals by number.
    fn order(&self) -> (bool, u32) {
        (self.position != PANEL, self.position.parse().unwrap_or(0))
    }
}

/// What a desk Item does.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Target {
    Show(String),
    Pin { screen: String, number: u32 },
    Unpin(String),
    Name(u32),
}

impl Desk {
    /// Reads the screens file `file`; none without it.
    pub fn read(file: &Path) -> Self {
        Self::parse(&fs::read_to_string(file).unwrap_or_default())
    }

    /// The positions in the file's text: a line each, its block, the connector of the screen
    /// there, the position, the screen and the group's name if it has one, split by tabs. A line
    /// of a file keyed by screen, which Hyprland migrates on its next start, is skipped.
    ///
    /// A screen gets one Item: the position it stands at, else the first it waited at. A file from
    /// before Hyprland repaired it can name a screen at a position and again waiting at another,
    /// or name a position twice; the first line for a position stands.
    pub fn parse(text: &str) -> Self {
        let mut positions = HashSet::new();
        let mut places: Vec<Place> = text
            .lines()
            .filter_map(|line| {
                let mut fields = line.split('\t');
                let block = fields.next()?.parse().ok()?;
                let connector = fields.next()?.to_owned();
                let position = fields.next().filter(|position| is_position(position))?.to_owned();
                let screen = fields.next().unwrap_or_default().to_owned();
                let name = fields.next().unwrap_or_default().to_owned();
                positions.insert(position.clone()).then_some(Place {
                    position,
                    block,
                    connector,
                    screen,
                    name,
                })
            })
            .collect();
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
        Desk { places, groups: None }
    }

    /// Reads the groups file `file` too, which the pin and name Items show and edit.
    #[must_use]
    pub fn with_groups_file(mut self, file: PathBuf) -> Self {
        let read = groups::read(&file).map_err(|err| Chain(&err).to_string());
        self.groups = Some((file, read));
        self
    }

    /// The Items: one a position, then the pins, the unpins and the names.
    pub fn items(&self) -> Vec<Item> {
        self.targets().into_iter().map(|(item, _)| item).collect()
    }

    /// Runs `action` on the desk Item `id`; the error is what the user reads.
    ///
    /// # Errors
    ///
    /// When there is no such Item or Action, `hyprctl` can't be started, or the groups file can't
    /// be changed.
    pub fn run(
        &self,
        id: &str,
        action: &str,
        arguments: &BTreeMap<String, String>,
        runner: &impl Runner,
    ) -> Result<Ran, String> {
        let unknown = || format!("`{id}` has no Action `{action}`");
        let target = self
            .targets()
            .into_iter()
            .find(|(item, _)| item.id == id)
            .map(|(_, target)| target)
            .ok_or_else(unknown)?;
        let file = || self.groups.as_ref().map(|(file, _)| file.as_path()).ok_or_else(unknown);
        let edited = match (target, action) {
            (Target::Show(position), SHOW) => return show(&position, runner),
            (Target::Pin { screen, number }, PIN) => groups::pin(file()?, &screen, number),
            (Target::Unpin(screen), UNPIN) => groups::unpin(file()?, &screen),
            (Target::Name(number), NAME) => {
                let name = arguments.get(NAME_ARGUMENT).map_or("", String::as_str);
                groups::name(file()?, number, name)
            }
            _ => return Err(unknown()),
        };
        edited.map_err(|err| Chain(&err).to_string())?;
        reload_groups(runner);
        Ok(Ran::Close)
    }

    /// Every Item with what it does.
    fn targets(&self) -> Vec<(Item, Target)> {
        let mut targets: Vec<(Item, Target)> = self
            .places
            .iter()
            .map(|place| (show_item(place), Target::Show(place.position.clone())))
            .collect();
        if let Some((_, Ok(groups))) = &self.groups {
            targets.extend(self.group_targets(groups));
        }
        targets
    }

    /// Whether the screen at `place` is plugged in and can be pinned. A pin names a description,
    /// so two screens reporting the same one, a pair without serials, can't be pinned apart:
    /// neither gets a Pin Item, and nor does an identity Hyprland numbered to tell them apart
    /// (`Generic Monitor #2`), which no screen reports.
    fn pinnable(&self, place: &Place) -> bool {
        let twin = |screen: &str| {
            self.places
                .iter()
                .filter(|other| !other.connector.is_empty() && other.screen == screen)
                .count()
                > 1
        };
        let numbered = place
            .screen
            .rsplit_once(" #")
            .is_some_and(|(_, count)| !count.is_empty() && count.bytes().all(|byte| byte.is_ascii_digit()));
        !place.connector.is_empty() && !numbered && !twin(&place.screen)
    }

    /// The pin, unpin and name Items for the screens plugged in and the groups in `groups` or on
    /// the desk.
    fn group_targets(&self, groups: &[Group]) -> Vec<(Item, Target)> {
        let names: BTreeMap<u32, &str> = groups
            .iter()
            .filter_map(|group| Some((group.number, group.name.as_deref()?)))
            .collect();
        let pinned: BTreeMap<&str, u32> = groups
            .iter()
            .flat_map(|group| group.screens.iter().map(|screen| (screen.as_str(), group.number)))
            .collect();
        let mut numbers: BTreeSet<u32> = groups.iter().map(|group| group.number).collect();
        numbers.extend(
            self.places
                .iter()
                .filter_map(|place| place.position.parse::<u32>().ok()),
        );
        let new = numbers.last().map_or(1, |last| last + 1);

        let mut targets = Vec::new();
        for place in self.places.iter().filter(|place| self.pinnable(place)) {
            for number in numbers.iter().copied().chain([new]) {
                if pinned.get(place.screen.as_str()) == Some(&number) {
                    continue;
                }
                let group = GroupName {
                    number,
                    name: names.get(&number).copied(),
                    new: number == new,
                };
                let target = Target::Pin {
                    screen: place.screen.clone(),
                    number,
                };
                targets.push((pin_item(place, &group), target));
            }
        }
        for (screen, number) in &pinned {
            let group = GroupName {
                number: *number,
                name: names.get(number).copied(),
                new: false,
            };
            targets.push((unpin_item(screen, &group), Target::Unpin((*screen).to_owned())));
        }
        for number in numbers {
            targets.push((name_item(number, names.get(&number).copied()), Target::Name(number)));
        }
        targets
    }
}

/// The screens file, `$XDG_STATE_HOME/kubaos/hypr/workspace-screens`, else under
/// `~/.local/state`; none without either.
pub fn file() -> Option<PathBuf> {
    updates::state_dir().map(|dir| dir.join(FILE))
}

/// Asks Hyprland to go to the group of `position`.
fn show(position: &str, runner: &impl Runner) -> Result<Ran, String> {
    let argv = [
        "hyprctl".to_owned(),
        "dispatch".to_owned(),
        format!("dot.workspaces.go_to_position(\"{position}\")"),
    ];
    runner
        .launch(&argv, Duration::ZERO)
        .map_err(|err| Chain(&err).to_string())?;
    Ok(Ran::Close)
}

/// Asks Hyprland to read the groups file again, so an edit from the menu applies even while the
/// shell, which watches the file for hand edits, is down. Hyprland moves nothing for a file it has
/// read already, so the shell's own reload after this one is harmless. The edit is saved either
/// way, so a `hyprctl` that can't start is logged, not shown as the Action failing.
fn reload_groups(runner: &impl Runner) {
    let argv = [
        "hyprctl".to_owned(),
        "dispatch".to_owned(),
        "dot.workspaces.reload_groups()".to_owned(),
    ];
    if let Err(err) = runner.launch(&argv, Duration::ZERO) {
        kotodama_extension::log!("telling Hyprland of the new workspace groups: {}", Chain(&err));
    }
}

fn is_position(field: &str) -> bool {
    field == PANEL || (!field.is_empty() && field.bytes().all(|byte| byte.is_ascii_digit()))
}

/// A group as an Item names it.
struct GroupName<'a> {
    number: u32,
    name: Option<&'a str>,
    /// The number after the last group: a group of the screen's own.
    new: bool,
}

impl GroupName<'_> {
    /// "group 2", "group 2: Centre", "a new group 3".
    fn title(&self) -> String {
        match (self.new, self.name) {
            (true, _) => format!("a new group {}", self.number),
            (false, Some(name)) => format!("group {}: {name}", self.number),
            (false, None) => format!("group {}", self.number),
        }
    }
}

fn show_title(place: &Place) -> String {
    if place.position == PANEL {
        "Workspace group: laptop panel".to_owned()
    } else if place.name.is_empty() {
        format!("Workspace group {}", place.position)
    } else {
        format!("Workspace group {}: {}", place.position, place.name)
    }
}

fn show_subtitle(place: &Place) -> String {
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

fn show_item(place: &Place) -> Item {
    item(
        format!("{PREFIX}{}", place.position),
        show_title(place),
        show_subtitle(place),
        &["position", "where are my windows"],
        Action::new(SHOW, "Go to it"),
    )
}

fn pin_item(place: &Place, group: &GroupName<'_>) -> Item {
    let subtitle = if group.new {
        format!(
            "On {}; it shows workspaces of its own whenever it is plugged in, on any port",
            place.connector
        )
    } else {
        format!(
            "On {}; it shows this group whenever it is plugged in, on any port, wherever it stands",
            place.connector
        )
    };
    item(
        format!("desk-pin:{}:{}", group.number, place.screen),
        format!("Pin {} to {}", place.screen, group.title()),
        subtitle,
        &["pin screen", "pin monitor", "assign screen"],
        Action::new(PIN, "Pin"),
    )
}

fn unpin_item(screen: &str, group: &GroupName<'_>) -> Item {
    item(
        format!("desk-unpin:{screen}"),
        format!("Unpin {screen}"),
        format!("From {}; it takes the next group left to right again", group.title()),
        &["unpin screen", "unpin monitor"],
        Action::new(UNPIN, "Unpin"),
    )
}

fn name_item(number: u32, name: Option<&str>) -> Item {
    let subtitle = match name {
        Some(name) => format!("Called {name} now; the menu and the bar show it. Blank takes it away"),
        None => "A name for the menu, and for the bar while the group waits on another screen".to_owned(),
    };
    let ask = Argument {
        name: NAME_ARGUMENT.to_owned(),
        placeholder: Some(name.unwrap_or("Left").to_owned()),
        optional: true,
    };
    item(
        format!("desk-name:{number}"),
        format!("Name group {number}…"),
        subtitle,
        &["rename group", "group name"],
        Action::new(NAME, "Name").arguments(vec![ask]),
    )
}

fn item(id: String, title: String, subtitle: String, keywords: &[&str], action: Action) -> Item {
    Item {
        id,
        kind: ItemKind::Namespaced(NamespacedKind::new(KIND).expect("the Extension's kinds are valid")),
        identity: None,
        title,
        subtitle: Some(subtitle),
        icon: Some(ICON.to_owned()),
        keyword: None,
        keywords: ["workspace group", "desk", "screen", "monitor", "display"]
            .iter()
            .chain(keywords)
            .map(|keyword| (*keyword).to_owned())
            .collect(),
        hidden: false,
        app_only: false,
        weak_keywords: Vec::new(),
        actions: vec![action],
    }
}

#[cfg(test)]
mod tests;
