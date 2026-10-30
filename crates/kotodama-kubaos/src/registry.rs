//! The registry of KubaOS actions: what the Extension's config table holds
//! under `actions`, one table per action, and the ids `hide` leaves out.
//! KubaOS's own actions are its Defaults (`config/kotodama/kubaos.toml`); the
//! user adds theirs, changes KubaOS's key by key, and hides any, in the user
//! layer's `user.toml`, which the Daemon merges over them.
//!
//! ```toml
//! [extensions.kubaos]
//! terminal = ["ghostty", "-e"]
//! hide = ["color-picker"]
//!
//! [extensions.kubaos.actions.ssh-key]
//! title = "New SSH key on the security key"
//! keyword = "ssh-key"
//! run = ["dot-ssh-sk-new", "{name}"]
//! arguments = [{ name = "name", placeholder = "github" }]
//! terminal = true
//!
//! [extensions.kubaos.actions.proxy]
//! title = "Proxy"
//! state = "proxy"
//! subtitle = { true = "On", false = "Off" }
//! order = ["on", "off", "status", "import"]
//!
//! [extensions.kubaos.actions.proxy.actions.on]
//! title = "Turn on"
//! keywords = ["on", "connect"]
//! run = ["kubaos-proxy", "on"]
//! primary_when = { is = false }
//!
//! [extensions.kubaos.actions.proxy.actions.off]
//! title = "Turn off"
//! keywords = ["off", "disconnect"]
//! run = ["kubaos-proxy", "off"]
//! primary_when = { is = true }
//!
//! [extensions.kubaos.actions.proxy.actions.status]
//! title = "Status"
//! keywords = ["status"]
//! view = "proxy"
//!
//! [extensions.kubaos.actions.proxy.actions.import]
//! title = "Import VPN config…"
//! keywords = ["import"]
//! run = ["kubaos-proxy", "import", "{path}"]
//! arguments = [{ name = "path", kind = "file", accept = ["conf", "ovpn"] }]
//!
//! [extensions.kubaos.actions.packages-chrome]
//! title = "Google Chrome"
//! identity = "app:google-chrome.desktop"
//! state = "packages.chrome"
//! remove = { run = ["kubaos", "packages", "disable", "chrome"], terminal = true }
//!
//! [extensions.kubaos.actions.packages-chrome.actions.install]
//! title = "Install and open"
//! keywords = ["install"]
//! run = ["sh", "-c", "kubaos packages enable chrome && google-chrome-stable"]
//! ```
//!
//! An action is one Item. With `run` it has one Action, which `role = "destructive"` marks as one
//! that removes something. With `actions` it has an Action a table, each with its own command, its
//! `keywords` (the verbs that choose it from the query, as in `proxy off`) and its `role`: the
//! things one thing can have done to it are its Actions, never Items of their own.
//!
//! In place of `run`, an Action may show one of the Extension's own Views, read live when it runs:
//! `view = "proxy"` ([`Shows`]). The Actions come in id order, or in the order `order` lists their
//! ids, the ones it leaves out after it.
//!
//! `state` names the state such an action follows: its `subtitle` may be a table of one by the
//! state's value, and the Actions whose `primary_when` holds come first, so Enter runs what applies
//! next. While the state can't be read there is no subtitle by state, and the Actions with a
//! `primary_when` come first, each part in id order.
//!
//! `identity` makes the Item a placeholder for a thing not there yet, such as an app not
//! installed: it steps aside while another Item of that Identity is indexed. Its `remove` is what
//! the Contributed action [`REMOVE`] runs on that Item, aimed at it while `state` reads true or
//! can't be read.
//!
//! `shown_when` sends an action only while the state it names reads as `is`, or as anything but
//! `is_not`. While the state can't be read the action shows all the same.
//!
//! # Owns
//!
//! - Reading the table, and what is wrong with it, as problems in words for the user.
//! - The actions' Items, which of them the states show, their Actions' order, and the command each
//!   Action runs or the View it shows.
//! - The Identities [`REMOVE`] is aimed at, and what it runs on each.
//!
//! # Not here
//!
//! - Starting the command: [`crate::run`]. Reading the states: [`crate::states`]. The Views:
//!   [`crate::proxy`] and [`crate::power`].
//! - The system toggles, whose ids an action may not take: [`crate::toggles`].

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::time::Duration;

use kotodama_extension::config;
use kotodama_protocol::Action;
use kotodama_protocol::ActionRole;
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
/// The one Action of an action with one command.
pub const RUN: &str = "run";
/// The Contributed action the Manifest declares on the things a placeholder stands for, which runs
/// the placeholder's `remove`.
pub const REMOVE: &str = "remove";
/// What a `terminal = true` action runs in when `terminal` isn't set: the
/// freedesktop default terminal's launcher.
pub const DEFAULT_TERMINAL: &[&str] = &["xdg-terminal-exec"];

/// The keys an action's table may hold.
const ITEM_KEYS: &[&str] = &[
    "title",
    "subtitle",
    "icon",
    "keywords",
    "keyword",
    "shown_when",
    "state",
    "actions",
    "identity",
    "remove",
    "order",
];

/// The keys of what an Action does, which an action with one command takes beside [`ITEM_KEYS`],
/// and each of an action's `actions` takes beside [`ACTION_KEYS`].
const COMMAND_KEYS: &[&str] = &["run", "view", "arguments", "terminal", "delay_ms", "confirm", "role"];

/// The keys of [`COMMAND_KEYS`] only an Action that runs a command takes, not one with a `view`.
const RUN_KEYS: &[&str] = &["arguments", "terminal", "delay_ms"];

/// The keys each of an action's `actions` may hold, beside [`COMMAND_KEYS`].
const ACTION_KEYS: &[&str] = &["title", "keywords", "primary_when"];

/// The keys `remove` may hold.
const REMOVE_KEYS: &[&str] = &["run", "terminal", "delay_ms"];

/// The keys `shown_when` may hold.
const SHOWN_WHEN_KEYS: &[&str] = &["state", "is", "is_not"];

/// The keys `primary_when` may hold: its state is the action's.
const PRIMARY_WHEN_KEYS: &[&str] = &["is", "is_not"];

/// KubaOS's actions and the user's, as the config table has them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Registry {
    /// In id order, without the ones `hide` names.
    actions: Vec<Entry>,
    hide: BTreeSet<String>,
    terminal: Vec<String>,
}

/// One action: one Item, with one Action or several.
#[derive(Clone, Debug, PartialEq)]
struct Entry {
    id: String,
    title: String,
    subtitle: Subtitle,
    icon: Option<String>,
    keywords: Vec<String>,
    keyword: Option<String>,
    /// The state its subtitle and its first Action follow.
    state: Option<String>,
    /// In id order: [`RUN`] alone for an action with one command.
    verbs: Vec<Verb>,
    shown_when: Option<ShownWhen>,
    /// The thing it stands for while no other Item of the Identity is indexed.
    identity: Option<String>,
    /// What [`REMOVE`] runs on the thing of its Identity.
    remove: Option<Runs>,
}

/// An action's subtitle.
#[derive(Clone, Debug, PartialEq)]
enum Subtitle {
    None,
    Fixed(String),
    /// One for each value of the action's state, and none while it can't be read.
    ByState(Vec<(Reading, String)>),
}

/// One Action of an action.
#[derive(Clone, Debug, PartialEq)]
struct Verb {
    id: String,
    title: String,
    keywords: Vec<String>,
    role: ActionRole,
    confirm: bool,
    does: Does,
    /// The values of the action's state in which it comes first.
    primary_when: Option<Condition>,
}

/// What an Action does: run a command, or show a View of the Extension's own.
#[derive(Clone, Debug, PartialEq)]
enum Does {
    Run(Runs),
    Show(Shows),
}

/// A View of the Extension's own an Action may show, by its name in `view`: what the module that
/// sends it reads live when the Action runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shows {
    /// `proxy`: each part of the work network and whether it is up.
    Proxy,
    /// `power`: every layer of the power stop the machine is on.
    PowerStop,
    /// `cpu-scheduler`: which CPU scheduler runs, and why.
    CpuScheduler,
}

impl Shows {
    const ALL: [Shows; 3] = [Shows::Proxy, Shows::PowerStop, Shows::CpuScheduler];

    fn name(self) -> &'static str {
        match self {
            Shows::Proxy => "proxy",
            Shows::PowerStop => "power",
            Shows::CpuScheduler => "cpu-scheduler",
        }
    }

    fn from_name(name: &str) -> Option<Shows> {
        Shows::ALL.into_iter().find(|shows| shows.name() == name)
    }
}

/// What an Action runs.
#[derive(Clone, Debug, PartialEq)]
struct Runs {
    run: Vec<String>,
    arguments: Vec<Argument>,
    terminal: bool,
    delay: Duration,
}

/// The state an action is shown in.
#[derive(Clone, Debug, PartialEq)]
struct ShownWhen {
    state: String,
    condition: Condition,
}

/// A test of a state's reading.
#[derive(Clone, Debug, PartialEq)]
struct Condition {
    reading: Reading,
    /// Whether it holds while the state reads as `reading` (`is`), or while it doesn't (`is_not`).
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

    /// The Items of the actions `hide` leaves in that show in `states`, each with the Action that
    /// applies next in them first.
    pub fn items<'a>(&'a self, states: &'a States) -> impl Iterator<Item = Item> + 'a {
        self.actions
            .iter()
            .filter(|entry| entry.shown_when.as_ref().is_none_or(|when| when.holds(states)))
            .map(|entry| entry.item(states))
    }

    /// The Identities [`REMOVE`] is aimed at in `states`: each placeholder's with a `remove`,
    /// while its state says the thing is there or can't be read.
    pub fn targets(&self, states: &States) -> Vec<String> {
        self.actions
            .iter()
            .filter(|entry| entry.remove.is_some())
            .filter(|entry| {
                entry
                    .state
                    .as_ref()
                    .and_then(|state| states.get(state))
                    .is_none_or(|reading| *reading == Reading::Bool(true))
            })
            .filter_map(|entry| entry.identity.clone())
            .collect()
    }

    /// What the Action `action` of the action `id` runs with `arguments`; `None` for an id that is
    /// no action, one `hide` names, or an Action it doesn't have.
    pub fn command(&self, id: &str, action: &str, arguments: &BTreeMap<String, String>) -> Option<Command> {
        match &self.verb(id, action)?.does {
            Does::Run(runs) => Some(self.command_of(runs, arguments)),
            Does::Show(_) => None,
        }
    }

    /// The View the Action `action` of the action `id` shows; `None` for an id that is no action,
    /// one `hide` names, an Action it doesn't have, or one that runs a command.
    pub fn view(&self, id: &str, action: &str) -> Option<Shows> {
        match self.verb(id, action)?.does {
            Does::Show(shows) => Some(shows),
            Does::Run(_) => None,
        }
    }

    fn verb(&self, id: &str, action: &str) -> Option<&Verb> {
        let entry = self.actions.iter().find(|entry| entry.id == id)?;
        entry.verbs.iter().find(|verb| verb.id == action)
    }

    /// What [`REMOVE`] runs on the thing of `identity`; `None` when no placeholder `hide` leaves
    /// in stands for it with a `remove`.
    pub fn removal(&self, identity: &str) -> Option<Command> {
        let entry = self
            .actions
            .iter()
            .find(|entry| entry.identity.as_deref() == Some(identity))?;
        Some(self.command_of(entry.remove.as_ref()?, &BTreeMap::new()))
    }

    fn command_of(&self, runs: &Runs, arguments: &BTreeMap<String, String>) -> Command {
        let mut argv = if runs.terminal {
            self.terminal.clone()
        } else {
            Vec::new()
        };
        argv.extend(
            runs.run
                .iter()
                .filter_map(|word| fill(word, &runs.arguments, arguments)),
        );
        Command {
            argv,
            delay: runs.delay,
        }
    }
}

impl Entry {
    fn from_value(id: &str, value: &Value) -> Result<Entry, String> {
        let Value::Table(table) = value else {
            return Err("should be a table".to_owned());
        };
        if let Some(key) = table
            .keys()
            .find(|key| !ITEM_KEYS.contains(&key.as_str()) && !COMMAND_KEYS.contains(&key.as_str()))
        {
            return Err(format!("`{key}` isn't a setting of an action"));
        }
        let title = string(table, "title")?.ok_or("needs a `title`")?;
        let state = string(table, "state")?;
        let values = state
            .as_deref()
            .map(|state| states::values(state).ok_or_else(|| format!("`state` names `{state}`, which is no state")))
            .transpose()?;
        let named = state.as_deref().zip(values);
        let mut verbs: Vec<Verb> = match table.get("actions") {
            None => vec![Verb::from_table(RUN, title.clone(), Vec::new(), table)?],
            Some(_) if table.contains_key("run") => return Err("takes `run` or `actions`, not both".to_owned()),
            Some(Value::Table(actions)) if !actions.is_empty() => {
                if let Some(key) = table.keys().find(|key| COMMAND_KEYS.contains(&key.as_str())) {
                    return Err(format!("`{key}` is a setting of each of its `actions`"));
                }
                actions
                    .iter()
                    .map(|(id, value)| {
                        Verb::from_value(id, value, named).map_err(|problem| format!("`actions.{id}`: {problem}"))
                    })
                    .collect::<Result<_, _>>()?
            }
            Some(_) => return Err("`actions` should be a table of at least one action, by id".to_owned()),
        };
        if table.contains_key("order") {
            if !table.contains_key("actions") {
                return Err("`order` is for an item with `actions`".to_owned());
            }
            order(&mut verbs, &strings(table, "order")?)?;
        }
        let identity = match table.get("identity") {
            None => None,
            Some(Value::String(identity)) if !identity.is_empty() => Some(identity.clone()),
            Some(_) => return Err("`identity` should be a string that isn't empty".to_owned()),
        };
        let remove = table
            .get("remove")
            .map(|value| {
                if identity.is_none() {
                    return Err("`remove` is for an item with an `identity`".to_owned());
                }
                if values.is_some_and(|values| values != Values::Bool) {
                    return Err("`remove` needs a `state` that is true or false".to_owned());
                }
                Runs::from_remove(value)
            })
            .transpose()?;
        Ok(Entry {
            id: id.to_owned(),
            title,
            subtitle: Subtitle::from_table(table, named)?,
            icon: string(table, "icon")?,
            keywords: strings(table, "keywords")?,
            keyword: string(table, "keyword")?,
            state,
            verbs,
            shown_when: table.get("shown_when").map(ShownWhen::from_value).transpose()?,
            identity,
            remove,
        })
    }

    fn item(&self, states: &States) -> Item {
        let reading = self.state.as_deref().and_then(|state| states.get(state));
        let mut verbs: Vec<&Verb> = self.verbs.iter().collect();
        // A stable sort: the Actions that apply next keep their order, and so do the rest.
        verbs.sort_by_key(|verb| !verb.applies(reading));
        Item {
            id: self.id.clone(),
            kind: ItemKind::Namespaced(NamespacedKind::new(KIND).expect("the action kind is valid")),
            identity: self.identity.clone(),
            title: self.title.clone(),
            subtitle: self.subtitle.read(reading),
            icon: self.icon.clone(),
            keyword: self.keyword.clone(),
            keywords: self.keywords.clone(),
            hidden: false,
            app_only: false,
            weak_keywords: Vec::new(),
            yields: self.identity.is_some(),
            actions: verbs.into_iter().map(Verb::action).collect(),
        }
    }
}

impl Subtitle {
    /// The `subtitle` of `table`, by the values of the action's state `named` if it is a table.
    fn from_table(table: &Table, named: Option<(&str, Values)>) -> Result<Subtitle, String> {
        match table.get("subtitle") {
            None => Ok(Subtitle::None),
            Some(Value::String(subtitle)) => Ok(Subtitle::Fixed(subtitle.clone())),
            Some(Value::Table(by_state)) => {
                let (state, values) = named.ok_or("a `subtitle` by state needs a `state`")?;
                by_state
                    .iter()
                    .map(|(key, value)| {
                        let reading = match key.as_str() {
                            "true" if values == Values::Bool => Reading::Bool(true),
                            "false" if values == Values::Bool => Reading::Bool(false),
                            word => Reading::Word(word.to_owned()),
                        };
                        if !values.allow(&reading) {
                            return Err(format!("`subtitle`: {}", values_problem(state, values)));
                        }
                        let Value::String(subtitle) = value else {
                            return Err(SUBTITLE_SHAPE.to_owned());
                        };
                        Ok((reading, subtitle.clone()))
                    })
                    .collect::<Result<_, _>>()
                    .map(Subtitle::ByState)
            }
            Some(_) => Err(SUBTITLE_SHAPE.to_owned()),
        }
    }

    fn read(&self, reading: Option<&Reading>) -> Option<String> {
        match self {
            Subtitle::None => None,
            Subtitle::Fixed(subtitle) => Some(subtitle.clone()),
            Subtitle::ByState(by_state) => by_state
                .iter()
                .find(|(value, _)| Some(value) == reading)
                .map(|(_, subtitle)| subtitle.clone()),
        }
    }
}

const SUBTITLE_SHAPE: &str = "`subtitle` should be a string, or a table of strings by the `state`'s value";

impl Verb {
    /// The Action `id` of an action whose state is `named`, from its table under `actions`.
    fn from_value(id: &str, value: &Value, named: Option<(&str, Values)>) -> Result<Verb, String> {
        let Value::Table(table) = value else {
            return Err("should be a table".to_owned());
        };
        if let Some(key) = table
            .keys()
            .find(|key| !ACTION_KEYS.contains(&key.as_str()) && !COMMAND_KEYS.contains(&key.as_str()))
        {
            return Err(format!("`{key}` isn't a setting of an action"));
        }
        let title = string(table, "title")?.ok_or("needs a `title`")?;
        let mut verb = Verb::from_table(id, title, strings(table, "keywords")?, table)?;
        verb.primary_when = table
            .get("primary_when")
            .map(|value| {
                let (state, values) = named.ok_or("`primary_when` needs the item's `state`")?;
                Condition::from_value(value, "primary_when", PRIMARY_WHEN_KEYS, state, values)
            })
            .transpose()?;
        Ok(verb)
    }

    /// The Action `id`, titled `title` and known by `keywords`, with what `table` says it runs.
    fn from_table(id: &str, title: String, keywords: Vec<String>, table: &Table) -> Result<Verb, String> {
        let role = match string(table, "role")?.as_deref() {
            None | Some("normal") => ActionRole::Normal,
            Some("destructive") => ActionRole::Destructive,
            Some(_) => return Err("`role` should be `normal` or `destructive`".to_owned()),
        };
        let does = match string(table, "view")? {
            None => Does::Run(Runs::from_table(table)?),
            Some(_) if table.contains_key("run") => return Err("takes `run` or `view`, not both".to_owned()),
            Some(name) => {
                if let Some(key) = RUN_KEYS.iter().find(|key| table.contains_key(**key)) {
                    return Err(format!("`{key}` is a setting of an action that runs a command"));
                }
                let names: Vec<&str> = Shows::ALL.iter().map(|shows| shows.name()).collect();
                let (last, rest) = names.split_last().expect("there are Views");
                Does::Show(
                    Shows::from_name(&name)
                        .ok_or_else(|| format!("`view` should be one of `{}` and `{last}`", rest.join("`, `")))?,
                )
            }
        };
        Ok(Verb {
            id: id.to_owned(),
            title,
            keywords,
            role,
            confirm: boolean(table, "confirm")?,
            does,
            primary_when: None,
        })
    }

    /// Whether it comes first while the action's state reads as `reading`.
    fn applies(&self, reading: Option<&Reading>) -> bool {
        self.primary_when.as_ref().is_some_and(|when| when.holds(reading))
    }

    fn action(&self) -> Action {
        let arguments = match &self.does {
            Does::Run(runs) => runs.arguments.clone(),
            Does::Show(_) => Vec::new(),
        };
        let mut action = Action::new(&self.id, &self.title)
            .arguments(arguments)
            .keywords(self.keywords.clone())
            .role(self.role);
        action.confirm = self.confirm;
        action
    }
}

impl Runs {
    fn from_table(table: &Table) -> Result<Runs, String> {
        let run = match table.get("run").map(config::strings) {
            Some(Some(run)) if !run.is_empty() => run,
            _ => return Err("needs `run`, a list of strings, the program first".to_owned()),
        };
        let delay = match table.get("delay_ms") {
            None => Duration::ZERO,
            Some(Value::Integer(ms)) if *ms >= 0 => Duration::from_millis(ms.unsigned_abs()),
            Some(_) => return Err("`delay_ms` should be a number of milliseconds".to_owned()),
        };
        Ok(Runs {
            run,
            arguments: arguments(table)?,
            terminal: boolean(table, "terminal")?,
            delay,
        })
    }

    fn from_remove(value: &Value) -> Result<Runs, String> {
        let Value::Table(table) = value else {
            return Err("`remove` should be a table of `run`, and `terminal` or `delay_ms`".to_owned());
        };
        if let Some(key) = table.keys().find(|key| !REMOVE_KEYS.contains(&key.as_str())) {
            return Err(format!("`remove`: `{key}` isn't a setting of `remove`"));
        }
        Runs::from_table(table).map_err(|problem| format!("`remove`: {problem}"))
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
        if !table.contains_key("is") && !table.contains_key("is_not") {
            return Err("`shown_when` needs one of `is` and `is_not`".to_owned());
        }
        let values =
            states::values(&state).ok_or_else(|| format!("`shown_when` names `{state}`, which is no state"))?;
        let condition = Condition::from_value(value, "shown_when", SHOWN_WHEN_KEYS, &state, values)?;
        Ok(ShownWhen { state, condition })
    }

    /// Whether the action shows in `states`: always while its state can't be read.
    fn holds(&self, states: &States) -> bool {
        self.condition.holds(states.get(&self.state))
    }
}

impl Condition {
    /// The `is` or `is_not` of the table `value`, the setting `name` that may hold `keys`, of the
    /// state `state` that reads as `values`.
    fn from_value(value: &Value, name: &str, keys: &[&str], state: &str, values: Values) -> Result<Condition, String> {
        let Value::Table(table) = value else {
            return Err(format!("`{name}` should be a table of `is` or `is_not`"));
        };
        if let Some(key) = table.keys().find(|key| !keys.contains(&key.as_str())) {
            return Err(format!("`{key}` isn't a setting of `{name}`"));
        }
        let (key, value, equal) = match (table.get("is"), table.get("is_not")) {
            (Some(value), None) => ("is", value, true),
            (None, Some(value)) => ("is_not", value, false),
            _ => return Err(format!("`{name}` needs one of `is` and `is_not`")),
        };
        let reading = match value {
            Value::Bool(on) => Reading::Bool(*on),
            Value::String(word) => Reading::Word(word.clone()),
            _ => return Err(format!("`{name}`'s `{key}` should be true, false or a string")),
        };
        if !values.allow(&reading) {
            return Err(format!("`{name}`: {}", values_problem(state, values)));
        }
        Ok(Condition { reading, equal })
    }

    /// Whether it holds for `reading`: always while the state can't be read.
    fn holds(&self, reading: Option<&Reading>) -> bool {
        reading.is_none_or(|reading| (*reading == self.reading) == self.equal)
    }
}

/// Puts `verbs` in the order of the ids `order` names, the rest after them in id order.
fn order(verbs: &mut [Verb], order: &[String]) -> Result<(), String> {
    for (n, id) in order.iter().enumerate() {
        if !verbs.iter().any(|verb| verb.id == *id) {
            return Err(format!("`order` names `{id}`, which is no action of it"));
        }
        if order[..n].contains(id) {
            return Err(format!("`order` names `{id}` twice"));
        }
    }
    verbs.sort_by_key(|verb| order.iter().position(|id| *id == verb.id).unwrap_or(order.len()));
    Ok(())
}

/// What the state `state` of `values` may read as, for a problem.
fn values_problem(state: &str, values: Values) -> String {
    match values {
        Values::Bool => format!("`{state}` is true or false"),
        Values::Words(words) => format!("`{state}` is one of `{}`", words.join("`, `")),
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

fn strings(table: &Table, key: &str) -> Result<Vec<String>, String> {
    match table.get(key).map(config::strings) {
        None => Ok(Vec::new()),
        Some(Some(strings)) => Ok(strings),
        Some(None) => Err(format!("`{key}` should be a list of strings")),
    }
}

fn boolean(table: &Table, key: &str) -> Result<bool, String> {
    match table.get(key) {
        None => Ok(false),
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(format!("`{key}` should be true or false")),
    }
}

/// The `arguments` list: tables of `name`, and optionally `placeholder`,
/// `optional`, `kind` (`text`, `number`, `file` or `folder`) and, for a
/// file, `accept`: the extensions or MIME types it takes.
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
            let kind = match string(argument, "kind")? {
                None => ArgumentKind::Text,
                Some(kind) => ArgumentKind::try_from(kind).map_err(|error| error.to_string())?,
            };
            let accept = match argument.get("accept").map(config::strings) {
                None => Vec::new(),
                Some(Some(accept)) if kind == ArgumentKind::File => accept,
                Some(Some(_)) => return Err("`accept` is for an Argument of kind `file`".to_owned()),
                Some(None) => return Err("`accept` should be a list of extensions or MIME types".to_owned()),
            };
            Ok(Argument {
                name: string(argument, "name")?.ok_or_else(shape)?,
                placeholder: string(argument, "placeholder")?,
                optional: boolean(argument, "optional")?,
                kind,
                accept,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
