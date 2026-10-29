//! The registry of KubaOS actions: what the Extension's config table holds
//! under `actions`, one table per action, and the ids `hide` leaves out.
//! KubaOS's own actions are its Defaults (`config/kotodama/kubaos.toml`); the
//! user adds theirs, changes KubaOS's key by key, and hides any, in the user
//! layer's `user.toml`, which the Daemon merges over them.
//!
//! ```toml
//! [extensions.kubaos]
//! terminal = ["ghostty", "-e"]
//! hide = ["theme-previous"]
//!
//! [extensions.kubaos.actions.ssh-key]
//! title = "New SSH key on the security key"
//! keyword = "ssh-key"
//! run = ["dot-ssh-sk-new", "{name}"]
//! arguments = [{ name = "name", placeholder = "github" }]
//! terminal = true
//!
//! [extensions.kubaos.actions.packages-chrome-off]
//! title = "Remove Google Chrome"
//! run = ["kubaos", "packages", "disable", "chrome"]
//! shown_when = { state = "packages.chrome", is = true }
//! ```
//!
//! `shown_when` sends an action only while the state it names reads as `is`, or as anything but
//! `is_not`, so of an on/off pair only the half that applies shows. While the state can't be read
//! the action shows all the same.
//!
//! # Owns
//!
//! - Reading the table, and what is wrong with it, as problems in words for the user.
//! - The actions' Items, which of them the states show, and the command each one runs.
//!
//! # Not here
//!
//! - Starting the command: [`crate::run`]. Reading the states: [`crate::states`].
//! - The system toggles, whose ids an action may not take: [`crate::toggles`].

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::time::Duration;

use kotodama_extension::config;
use kotodama_protocol::Action;
use kotodama_protocol::Argument;
use kotodama_protocol::ArgumentKind;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;
use kotodama_protocol::Table;
use kotodama_protocol::Value;

use crate::states;
use crate::states::Reading;
use crate::states::States;
use crate::states::Values;

/// The Item kind of every action.
pub const KIND: &str = "kubaos.action";
/// The one Action of every action's Item.
pub const RUN: &str = "run";
/// What a `terminal = true` action runs in when `terminal` isn't set: the
/// freedesktop default terminal's launcher.
pub const DEFAULT_TERMINAL: &[&str] = &["xdg-terminal-exec"];

/// The keys an action's table may hold.
const ACTION_KEYS: &[&str] = &[
    "title",
    "subtitle",
    "icon",
    "keywords",
    "keyword",
    "run",
    "arguments",
    "terminal",
    "confirm",
    "delay_ms",
    "shown_when",
];

/// The keys `shown_when` may hold.
const SHOWN_WHEN_KEYS: &[&str] = &["state", "is", "is_not"];

/// KubaOS's actions and the user's, as the config table has them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Registry {
    /// In id order, without the ones `hide` names.
    actions: Vec<Entry>,
    hide: BTreeSet<String>,
    terminal: Vec<String>,
}

/// One action.
#[derive(Clone, Debug, PartialEq)]
struct Entry {
    id: String,
    title: String,
    subtitle: Option<String>,
    icon: Option<String>,
    keywords: Vec<String>,
    keyword: Option<String>,
    run: Vec<String>,
    arguments: Vec<Argument>,
    terminal: bool,
    confirm: bool,
    delay: Duration,
    shown_when: Option<ShownWhen>,
}

/// The state an action is shown in.
#[derive(Clone, Debug, PartialEq)]
struct ShownWhen {
    state: String,
    reading: Reading,
    /// Whether it shows while the state reads as `reading` (`is`), or while it doesn't (`is_not`).
    equal: bool,
}

/// What an action runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    /// The program, then its arguments, with the Arguments filled in.
    pub argv: Vec<String>,
    /// How long to wait before it starts, so the menu is gone from the screen
    /// first, as a screenshot needs.
    pub delay: Duration,
}

impl Registry {
    /// Reads the Extension's config table. Ids in `reserved` belong to Items
    /// of the Extension's own, which `hide` may name but an action may not
    /// take. What is wrong comes back as problems, one line each; an action
    /// with a problem is left out, and the rest still load.
    pub fn from_config(config: &Table, reserved: &[&str]) -> (Registry, Vec<String>) {
        let mut problems = Vec::new();
        let default_terminal = || DEFAULT_TERMINAL.iter().map(|&word| word.to_owned()).collect();
        let terminal = match config.get("terminal").map(config::strings) {
            None => default_terminal(),
            Some(Some(terminal)) if !terminal.is_empty() => terminal,
            Some(_) => {
                problems.push("`terminal` should be a list of strings, the program first".to_owned());
                default_terminal()
            }
        };
        let hide: BTreeSet<String> = match config.get("hide").map(config::strings) {
            None => BTreeSet::new(),
            Some(Some(ids)) => ids.into_iter().collect(),
            Some(None) => {
                problems.push("`hide` should be a list of ids".to_owned());
                BTreeSet::new()
            }
        };

        let mut known: BTreeSet<&str> = reserved.iter().copied().collect();
        let mut actions = Vec::new();
        match config.get("actions") {
            None => {}
            Some(Value::Table(tables)) => {
                for (id, value) in tables {
                    known.insert(id);
                    if reserved.contains(&id.as_str()) {
                        problems.push(format!(
                            "`actions.{id}`: `{id}` is a toggle's id; give the action another"
                        ));
                        continue;
                    }
                    match Entry::from_value(id, value) {
                        Ok(entry) if !hide.contains(id) => actions.push(entry),
                        Ok(_) => {}
                        Err(problem) => problems.push(format!("`actions.{id}`: {problem}")),
                    }
                }
            }
            Some(_) => problems.push("`actions` should be a table of actions, by id".to_owned()),
        }
        for id in hide.iter().filter(|id| !known.contains(id.as_str())) {
            problems.push(format!("`hide` names `{id}`, which is no action or toggle"));
        }

        (
            Registry {
                actions,
                hide,
                terminal,
            },
            problems,
        )
    }

    /// Whether `hide` names `id`.
    pub fn hides(&self, id: &str) -> bool {
        self.hide.contains(id)
    }

    /// The Items of the actions `hide` leaves in that show in `states`.
    pub fn items<'a>(&'a self, states: &'a States) -> impl Iterator<Item = Item> + 'a {
        self.actions
            .iter()
            .filter(|entry| entry.shown_when.as_ref().is_none_or(|when| when.shows(states)))
            .map(Entry::item)
    }

    /// What the action `id` runs with `arguments`; `None` for an id that is
    /// no action, or one `hide` names.
    pub fn command(&self, id: &str, arguments: &BTreeMap<String, String>) -> Option<Command> {
        let entry = self.actions.iter().find(|entry| entry.id == id)?;
        let mut argv = if entry.terminal {
            self.terminal.clone()
        } else {
            Vec::new()
        };
        argv.extend(
            entry
                .run
                .iter()
                .filter_map(|word| fill(word, &entry.arguments, arguments)),
        );
        Some(Command {
            argv,
            delay: entry.delay,
        })
    }
}

impl Entry {
    fn from_value(id: &str, value: &Value) -> Result<Entry, String> {
        let Value::Table(table) = value else {
            return Err("should be a table".to_owned());
        };
        if let Some(key) = table.keys().find(|key| !ACTION_KEYS.contains(&key.as_str())) {
            return Err(format!("`{key}` isn't a setting of an action"));
        }
        let title = string(table, "title")?.ok_or("needs a `title`")?;
        let run = match table.get("run").map(config::strings) {
            Some(Some(run)) if !run.is_empty() => run,
            _ => return Err("needs `run`, a list of strings, the program first".to_owned()),
        };
        let keywords = match table.get("keywords").map(config::strings) {
            None => Vec::new(),
            Some(Some(keywords)) => keywords,
            Some(None) => return Err("`keywords` should be a list of strings".to_owned()),
        };
        let delay = match table.get("delay_ms") {
            None => Duration::ZERO,
            Some(Value::Integer(ms)) if *ms >= 0 => Duration::from_millis(ms.unsigned_abs()),
            Some(_) => return Err("`delay_ms` should be a number of milliseconds".to_owned()),
        };
        Ok(Entry {
            id: id.to_owned(),
            title,
            subtitle: string(table, "subtitle")?,
            icon: string(table, "icon")?,
            keywords,
            keyword: string(table, "keyword")?,
            run,
            arguments: arguments(table)?,
            terminal: boolean(table, "terminal")?,
            confirm: boolean(table, "confirm")?,
            delay,
            shown_when: table.get("shown_when").map(ShownWhen::from_value).transpose()?,
        })
    }

    fn item(&self) -> Item {
        let mut action = Action::new(RUN, self.title.clone()).arguments(self.arguments.clone());
        action.confirm = self.confirm;
        Item {
            id: self.id.clone(),
            kind: ItemKind::Namespaced(NamespacedKind::new(KIND).expect("the action kind is valid")),
            identity: None,
            title: self.title.clone(),
            subtitle: self.subtitle.clone(),
            icon: self.icon.clone(),
            keyword: self.keyword.clone(),
            keywords: self.keywords.clone(),
            hidden: false,
            app_only: false,
            weak_keywords: Vec::new(),
            yields: false,
            actions: vec![action],
        }
    }
}

impl ShownWhen {
    fn from_value(value: &Value) -> Result<ShownWhen, String> {
        let Value::Table(table) = value else {
            return Err("`shown_when` should be a table of `state` and `is` or `is_not`".to_owned());
        };
        if let Some(key) = table.keys().find(|key| !SHOWN_WHEN_KEYS.contains(&key.as_str())) {
            return Err(format!("`{key}` isn't a setting of `shown_when`"));
        }
        let state = string(table, "state")
            .map_err(|_| "`shown_when`'s `state` should be a string".to_owned())?
            .ok_or("`shown_when` needs a `state`")?;
        let (key, value, equal) = match (table.get("is"), table.get("is_not")) {
            (Some(value), None) => ("is", value, true),
            (None, Some(value)) => ("is_not", value, false),
            _ => return Err("`shown_when` needs one of `is` and `is_not`".to_owned()),
        };
        let reading = match value {
            Value::Bool(on) => Reading::Bool(*on),
            Value::String(word) => Reading::Word(word.clone()),
            _ => return Err(format!("`shown_when`'s `{key}` should be true, false or a string")),
        };
        let values =
            states::values(&state).ok_or_else(|| format!("`shown_when` names `{state}`, which is no state"))?;
        if !values.allow(&reading) {
            return Err(match values {
                Values::Bool => format!("`shown_when`: `{state}` is true or false"),
                Values::Words(words) => format!("`shown_when`: `{state}` is one of `{}`", words.join("`, `")),
            });
        }
        Ok(ShownWhen { state, reading, equal })
    }

    /// Whether the action shows in `states`: always while its state can't be read.
    fn shows(&self, states: &States) -> bool {
        states
            .get(&self.state)
            .is_none_or(|reading| (*reading == self.reading) == self.equal)
    }
}

/// `word` with each `{name}` of `declared` replaced by its value in `given`.
/// `None` drops the word: it is exactly the placeholder of an Argument left
/// out.
fn fill(word: &str, declared: &[Argument], given: &BTreeMap<String, String>) -> Option<String> {
    let mut filled = word.to_owned();
    for argument in declared {
        let placeholder = format!("{{{}}}", argument.name);
        let value = given.get(&argument.name).map_or("", String::as_str);
        if word == placeholder && value.is_empty() {
            return None;
        }
        filled = filled.replace(&placeholder, value);
    }
    Some(filled)
}

fn string(table: &Table, key: &str) -> Result<Option<String>, String> {
    match table.get(key) {
        None => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(format!("`{key}` should be a string")),
    }
}

fn boolean(table: &Table, key: &str) -> Result<bool, String> {
    match table.get(key) {
        None => Ok(false),
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(format!("`{key}` should be true or false")),
    }
}

/// The `arguments` list: tables of `name`, and optionally `placeholder` and
/// `optional`.
fn arguments(table: &Table) -> Result<Vec<Argument>, String> {
    let Some(value) = table.get("arguments") else {
        return Ok(Vec::new());
    };
    let shape = || "`arguments` should be a list of tables, each with a `name`".to_owned();
    let Value::Array(values) = value else {
        return Err(shape());
    };
    values
        .iter()
        .map(|value| {
            let Value::Table(argument) = value else {
                return Err(shape());
            };
            Ok(Argument {
                name: string(argument, "name")?.ok_or_else(shape)?,
                placeholder: string(argument, "placeholder")?,
                optional: boolean(argument, "optional")?,
                kind: ArgumentKind::Text,
                accept: Vec::new(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
