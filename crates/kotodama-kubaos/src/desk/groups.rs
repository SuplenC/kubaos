//! The workspace groups file, `~/.config/kubaos/desktop/workspaces/groups.toml`: the screens the
//! user pinned to each group, by description, and the groups' names. A table a group:
//!
//! ```toml
//! [group.1]
//! name = "Left"
//! screens = ["BNQ ZOWIE XL LCD G4L01663SL0", "Dell Inc. DELL U2724D 5D6E7F8"]
//! ```
//!
//! # Owns
//!
//! - Reading the file as Hyprland reads it, and the menu's edits to it: pinning a screen,
//!   unpinning it, naming a group. An edit keeps the user's comments and writes the whole file
//!   through a temporary one and a rename.
//!
//! # Not here
//!
//! - Where each group goes, and reading the file in the session: `parse_groups` and `positions`
//!   in `config/hypr/workspace_blocks.lua`. Hyprland is told of a change by the bar, which
//!   watches the file (`config/quickshell/services/Hypr.qml`).
//! - The menu's Items: [`crate::desk`].

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use kubaos_user_layer::App;
use toml_edit::Array;
use toml_edit::DocumentMut;
use toml_edit::Item;
use toml_edit::Table;
use toml_edit::Value;
use toml_edit::value;

/// The file's name in the workspaces folder of the user layer.
pub const FILE: &str = "groups.toml";
/// The table holding one table a group.
const GROUP: &str = "group";
const NAME: &str = "name";
const SCREENS: &str = "screens";

/// A group as the file has it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    /// 1, 2 and on: the desk position the group is.
    pub number: u32,
    /// What the menu and the bar call it, when the user named it.
    pub name: Option<String>,
    /// The screens pinned to it by description, the first listed winning a clash.
    pub screens: Vec<String>,
}

/// Why the file couldn't be read or changed. The messages are what the menu says.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GroupsError {
    #[error("reading the workspace groups")]
    Read(#[source] io::Error),
    #[error("the workspace groups file isn't TOML")]
    Toml(#[source] toml_edit::TomlError),
    #[error("in the workspace groups file, {0}")]
    Invalid(String),
    #[error("writing the workspace groups")]
    Write(#[source] io::Error),
}

/// The file: `~/.config/kubaos/desktop/workspaces/groups.toml`; none without a config home.
pub fn path() -> Option<PathBuf> {
    kubaos_user_layer::config_home().map(|config| kubaos_user_layer::dir(&config, App::Workspaces).join(FILE))
}

/// The groups in the file at `path`, in number order; none without the file.
///
/// # Errors
///
/// When the file exists and can't be read, or isn't a groups file (as [`parse`]).
pub fn read(path: &Path) -> Result<Vec<Group>, GroupsError> {
    groups(&document(path)?)
}

/// The groups in `text`, in number order.
///
/// # Errors
///
/// [`GroupsError::Toml`] for text that isn't TOML, [`GroupsError::Invalid`] for TOML that isn't a
/// groups file: anything but `[group.N]` tables with a string `name` and a list of strings
/// `screens`, or a screen in two groups. Hyprland refuses the same files.
pub fn parse(text: &str) -> Result<Vec<Group>, GroupsError> {
    groups(&text.parse().map_err(GroupsError::Toml)?)
}

/// Pins `screen` to group `number`, last in its list, so a screen already there keeps winning a
/// clash; it leaves any group it was in. Makes the group when it is new.
///
/// # Errors
///
/// As [`read`], and when the file can't be written.
pub fn pin(path: &Path, screen: &str, number: u32) -> Result<(), GroupsError> {
    let mut doc = document(path)?;
    groups(&doc)?;
    remove_screen(&mut doc, screen);
    let screens = group_table(&mut doc, number)
        .entry(SCREENS)
        .or_insert_with(|| value(Array::new()));
    let Some(screens) = screens.as_array_mut() else {
        return Err(invalid(format!("group {number}'s screens aren't a list")));
    };
    push_like_the_rest(screens, screen);
    save(path, &doc)
}

/// Takes `screen` out of every group, so it takes its place left to right again. The group stays,
/// with its name and comments, even with no screen left.
///
/// # Errors
///
/// As [`read`], and when the file can't be written.
pub fn unpin(path: &Path, screen: &str) -> Result<(), GroupsError> {
    let mut doc = document(path)?;
    groups(&doc)?;
    remove_screen(&mut doc, screen);
    save(path, &doc)
}

/// Names group `number`, or takes its name away when `name` is blank. Makes the group when it is
/// new.
///
/// # Errors
///
/// As [`read`], and when the file can't be written.
pub fn name(path: &Path, number: u32, name: &str) -> Result<(), GroupsError> {
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut doc = document(path)?;
    groups(&doc)?;
    let table = group_table(&mut doc, number);
    if name.is_empty() {
        table.remove(NAME);
    } else if let Some(written) = table.get_mut(NAME).and_then(Item::as_value_mut) {
        // The value is replaced in place, so a comment after it stays on its line.
        let decor = written.decor().clone();
        *written = Value::from(name);
        *written.decor_mut() = decor;
    } else {
        table.insert(NAME, value(name));
        table.sort_values_by(|left, _, right, _| (left.get() != NAME).cmp(&(right.get() != NAME)));
    }
    save(path, &doc)
}

/// The file as a document to edit, empty when missing.
fn document(path: &Path) -> Result<DocumentMut, GroupsError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(GroupsError::Read(err)),
    };
    text.parse().map_err(GroupsError::Toml)
}

/// Writes `doc` over the file at `path` through a temporary file and a rename, once it reads as
/// groups.
fn save(path: &Path, doc: &DocumentMut) -> Result<(), GroupsError> {
    let text = doc.to_string();
    parse(&text)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(GroupsError::Write)?;
    }
    let tmp = path.with_extension("toml.new");
    fs::write(&tmp, text).map_err(GroupsError::Write)?;
    fs::rename(&tmp, path).map_err(GroupsError::Write)
}

fn invalid(problem: impl Into<String>) -> GroupsError {
    GroupsError::Invalid(problem.into())
}

/// The groups `doc` holds, held to what `parse_groups` in `config/hypr/workspace_blocks.lua`
/// takes.
fn groups(doc: &DocumentMut) -> Result<Vec<Group>, GroupsError> {
    let mut groups = Vec::new();
    for (key, item) in doc.iter() {
        let Some(parent) = item.as_table().filter(|table| key == GROUP && table.is_implicit()) else {
            return Err(invalid(format!(
                "`{key}` isn't a group; groups are [group.1], [group.2] and on"
            )));
        };
        for (number, item) in parent {
            let number = number
                .parse::<u32>()
                .ok()
                .filter(|number| *number >= 1)
                .ok_or_else(|| invalid(format!("[group.{number}] isn't a group; groups count from 1")))?;
            let table = item
                .as_table()
                .ok_or_else(|| invalid(format!("group {number} isn't a [group.{number}] table")))?;
            groups.push(group(number, table)?);
        }
    }
    groups.sort_by_key(|group| group.number);

    let mut pinned_to: BTreeMap<&str, u32> = BTreeMap::new();
    for group in &groups {
        for screen in &group.screens {
            if screen.is_empty() {
                return Err(invalid(format!("group {} lists an empty screen", group.number)));
            }
            match pinned_to.insert(screen, group.number) {
                Some(other) if other == group.number => {
                    return Err(invalid(format!("group {other} lists {screen} twice")));
                }
                Some(other) => {
                    return Err(invalid(format!(
                        "{screen} is in group {other} and group {}; a screen is pinned to one group",
                        group.number
                    )));
                }
                None => {}
            }
        }
    }
    Ok(groups)
}

fn group(number: u32, table: &Table) -> Result<Group, GroupsError> {
    let mut group = Group {
        number,
        name: None,
        screens: Vec::new(),
    };
    for (key, item) in table {
        match key {
            NAME => {
                let name = item
                    .as_str()
                    .ok_or_else(|| invalid(format!("group {number}'s name isn't text in quotes")))?;
                let name = name.trim();
                group.name = (!name.is_empty()).then(|| name.to_owned());
            }
            SCREENS => {
                let list = item
                    .as_array()
                    .ok_or_else(|| invalid(format!("group {number}'s screens aren't a list")))?;
                for screen in list {
                    let screen = screen
                        .as_str()
                        .ok_or_else(|| invalid(format!("group {number} lists a screen that isn't text in quotes")))?;
                    group.screens.push(screen.trim().to_owned());
                }
            }
            _ => {
                return Err(invalid(format!("`{key}` isn't a setting of a group: name or screens")));
            }
        }
    }
    Ok(group)
}

/// Takes `screen` out of the list of every group.
fn remove_screen(doc: &mut DocumentMut, screen: &str) {
    let Some(parent) = doc.get_mut(GROUP).and_then(Item::as_table_mut) else {
        return;
    };
    for (_, item) in parent.iter_mut() {
        if let Some(screens) = item.get_mut(SCREENS).and_then(Item::as_array_mut) {
            screens.retain(|written| written.as_str().map(str::trim) != Some(screen));
        }
    }
}

/// The table of group `number`, made at the end of the file when it is new.
fn group_table(doc: &mut DocumentMut, number: u32) -> &mut Table {
    let key = number.to_string();
    let exists = doc
        .get(GROUP)
        .and_then(Item::as_table)
        .is_some_and(|parent| parent.contains_key(&key));
    if !exists {
        // A new table goes at the end, below the comments there, which are the document's
        // trailing text until a table takes them as its own.
        let mut table = Table::new();
        let trailing = doc.trailing().as_str().unwrap_or_default().to_owned();
        if !trailing.trim().is_empty() {
            table.decor_mut().set_prefix(trailing + "\n");
            doc.set_trailing("");
        } else if doc.get(GROUP).is_some() {
            table.decor_mut().set_prefix("\n");
        }
        let parent = doc
            .entry(GROUP)
            .or_insert_with(|| {
                let mut parent = Table::new();
                parent.set_implicit(true);
                Item::Table(parent)
            })
            .as_table_mut()
            .expect("[group] that read is a table");
        parent.insert(&key, Item::Table(table));
    }
    doc[GROUP][&key].as_table_mut().expect("a group that read is a table")
}

/// Adds `screen` to `screens`, laid out like the entry before it: on a line of its own in a list
/// written one a line, below a comment on the line before.
fn push_like_the_rest(screens: &mut Array, screen: &str) {
    let prefix = screens
        .iter()
        .last()
        .and_then(|last| last.decor().prefix())
        .and_then(|prefix| prefix.as_str())
        .filter(|prefix| prefix.contains('\n'))
        .map(str::to_owned);
    let Some(prefix) = prefix else {
        screens.push(screen);
        return;
    };
    // After the last entry's comma, the list's trailing text starts with the rest of its line,
    // a comment on that entry, which stays with it.
    let trailing = screens.trailing().as_str().unwrap_or_default().to_owned();
    let (same_line, rest) = trailing.split_at(trailing.find('\n').unwrap_or(trailing.len()));
    let prefix = format!("{same_line}{prefix}");
    screens.set_trailing(rest);
    screens.push(screen);
    if let Some(added) = screens.iter_mut().last() {
        added.decor_mut().set_prefix(prefix);
    }
}

#[cfg(test)]
mod tests;
