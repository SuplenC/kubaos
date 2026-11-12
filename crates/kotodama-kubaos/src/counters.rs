//! The counters in the menu: "New counter", whose query after `new-counter` is the name (empty is
//! "Counter"), and an Item a counter, titled by its name, whose Enter adds 1 and whose other
//! Actions take 1, set it to a number, reset it, rename it, set its auto reset, show its history or
//! remove it.
//!
//! "Auto reset…" opens a list that reads what is typed as the clock does, through
//! `kubaos_clock::recurrence`: the rule it read and its next reset, or examples when it reads
//! nothing; a number is "When it reaches N"; Never is always there. The list filters itself, so it
//! answers every keystroke (`ToExtension::FilterView`).
//!
//! A change is tried on a copy of the counters first, so a refusal (−1 at 0, a taken name) reads in
//! the menu; then it is launched as `kubaos-counters … --notify`, which writes the files and says
//! what it came to in a toast. The Items follow `counters.toml`, which `main.rs` watches.
//!
//! # Owns
//!
//! - The Items, their Actions, the Auto reset list and the History View, and what each runs.
//!
//! # Not here
//!
//! - The counters, their rules and their log: `kubaos_counters`. Watching the file: `main.rs`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::Duration;

use jiff::Zoned;
use jiff::tz::TimeZone;
use kotodama_extension::Chain;
use kotodama_protocol::Action;
use kotodama_protocol::ActionRole;
use kotodama_protocol::Argument;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::MetadataEntry;
use kotodama_protocol::NamespacedKind;
use kotodama_protocol::View;
use kotodama_protocol::ViewContent;
use kubaos_clock::recurrence;
use kubaos_clock::recurrence::Recurrence;
use kubaos_clock::settings::TimeFormat;
use kubaos_counters::Paths;
use kubaos_counters::history;
use kubaos_counters::history::Group;
use kubaos_counters::history::Sort;
use kubaos_counters::live;
use kubaos_counters::store::AutoReset;
use kubaos_counters::store::Counter;
use kubaos_counters::store::Counters;
use kubaos_counters::store::DEFAULT_NAME;

use crate::run::Runner;
use crate::toggles::Ran;

/// The Item kind of a counter.
pub const KIND: &str = "kubaos.counter";
/// The Item kind of "New counter".
pub const NEW_KIND: &str = "kubaos.counter-new";
/// The Item kind of a choice in the Auto reset list.
pub const RESET_KIND: &str = "kubaos.counter-reset";

/// "New counter"'s id, which `hide` names it by.
pub const NEW_ID: &str = "counter-new";
/// "New counter"'s Action.
pub const CREATE: &str = "create";

/// A counter's Actions; [`ADD`] is the primary one.
pub const ADD: &str = "add";
pub const SUB: &str = "sub";
pub const SET: &str = "set";
pub const RESET: &str = "reset";
pub const RENAME: &str = "rename";
pub const AUTO_RESET: &str = "auto-reset";
pub const HISTORY: &str = "history";
pub const REMOVE: &str = "remove";

/// An Auto reset choice's one Action.
pub const CHOOSE: &str = "choose";

/// The Argument holding a name.
const NAME: &str = "name";
/// The Argument holding a number.
const VALUE: &str = "value";

/// A counter's Item is this and its id, `counter:3`.
const PREFIX: &str = "counter:";
/// The Auto reset list of a counter is this and its id.
const RESET_VIEW: &str = "counter-reset:";
/// The History View of a counter is this and its id.
const HISTORY_VIEW: &str = "counter-history:";

/// The changes the History View shows, newest first; the bar's history has the rest.
const HISTORY_ROWS: usize = 200;

const ICON: &str = "accessories-calculator";
const PROGRAM: &str = "kubaos-counters";
const KEYWORDS: [&str; 4] = ["counter", "count", "tally", "tracker"];

/// The counters as `counters.toml` has them, and the clock's time format their words follow.
#[derive(Clone, Debug, Default)]
pub struct CounterList {
    counters: Counters,
    format: Option<TimeFormat>,
    /// The change log, which History reads when it opens.
    log: Option<PathBuf>,
}

impl CounterList {
    /// `counters`, written in `format`, with their log at `log`.
    #[must_use]
    pub fn new(counters: Counters, format: Option<TimeFormat>, log: Option<PathBuf>) -> Self {
        CounterList { counters, format, log }
    }

    /// The counters in their file, or none when it can't be read; the clock's time format from its
    /// settings.
    pub fn read() -> Self {
        let Some(paths) = Paths::from_env() else {
            return CounterList::default();
        };
        let counters = Counters::load(&paths.counters).unwrap_or_else(|err| {
            kotodama_extension::log!("not listing the counters: {}", Chain(&err));
            Counters::default()
        });
        CounterList::new(counters, time_format(), Some(paths.log))
    }

    /// "New counter", then an Item a counter, in their order.
    pub fn items(&self) -> Vec<Item> {
        std::iter::once(new_item())
            .chain(self.counters.all().iter().map(|counter| self.item(counter)))
            .collect()
    }

    fn item(&self, counter: &Counter) -> Item {
        let detail = live::detail(counter.reset, self.format);
        let subtitle = if detail.is_empty() {
            counter.value.to_string()
        } else {
            format!("{} · {detail}", counter.value)
        };
        let set = Action::new(SET, "Set to…")
            .arguments(vec![Argument::new(VALUE).number().placeholder("0 or more")])
            .role(ActionRole::Form);
        let rename = Action::new(RENAME, "Rename…")
            .arguments(vec![Argument::new(NAME).placeholder(counter.name.clone())])
            .role(ActionRole::Form);
        let remove = Action::new(REMOVE, "Remove")
            .keywords(["delete"])
            .confirm()
            .role(ActionRole::Destructive);
        Item {
            id: format!("{PREFIX}{}", counter.id),
            kind: kind(KIND),
            identity: None,
            title: counter.name.clone(),
            subtitle: Some(subtitle),
            icon: Some(ICON.to_owned()),
            keyword: None,
            keywords: KEYWORDS.map(str::to_owned).to_vec(),
            hidden: false,
            app_only: false,
            weak_keywords: Vec::new(),
            yields: false,
            actions: vec![
                Action::new(ADD, "+1").keywords(["add", "plus", "increment"]),
                Action::new(SUB, "\u{2212}1").keywords(["subtract", "minus", "decrement"]),
                set,
                Action::new(RESET, "Reset").keywords(["zero", "clear"]),
                rename,
                Action::new(AUTO_RESET, "Auto reset…").keywords(["schedule", "stop at", "limit"]),
                Action::new(HISTORY, "History").keywords(["log", "changes"]),
                remove,
            ],
        }
    }

    /// Runs "New counter": a counter named as typed after `new-counter`, or "Counter".
    ///
    /// # Errors
    ///
    /// When the name is taken or isn't one, or `kubaos-counters` can't start.
    pub fn create(&self, arguments: &BTreeMap<String, String>, runner: &impl Runner) -> Result<Ran, String> {
        let name = arguments.get(NAME).map_or("", |name| name.trim());
        self.counters
            .clone()
            .create(name, &Zoned::now())
            .map_err(|err| err.to_string())?;
        let name = if name.is_empty() { DEFAULT_NAME } else { name };
        launch(runner, &["new", name])?;
        Ok(Ran::Close)
    }

    /// Runs `action` on the counter Item `id`.
    ///
    /// # Errors
    ///
    /// When there's no such counter or Action, the change is refused (the message says why), or
    /// `kubaos-counters` can't start.
    pub fn run(
        &self,
        id: &str,
        action: &str,
        arguments: &BTreeMap<String, String>,
        runner: &impl Runner,
    ) -> Result<Ran, String> {
        let counter = self.by_item(id, PREFIX)?;
        let name = counter.name.as_str();
        let mut copy = self.counters.clone();
        let now = Zoned::now();
        let refused = |err: kubaos_counters::store::ChangeError| err.to_string();
        match action {
            ADD => {
                copy.add(name, &now).map_err(refused)?;
                launch(runner, &["add", name])?;
            }
            SUB => {
                copy.sub(name, &now).map_err(refused)?;
                launch(runner, &["sub", name])?;
            }
            SET => {
                let value = number(arguments, "Type the value, a whole number of 0 or more")?;
                launch(runner, &["set", name, &value.to_string()])?;
            }
            RESET => launch(runner, &["reset", name])?,
            RENAME => {
                let to = arguments.get(NAME).map_or("", |to| to.trim());
                copy.rename(name, to, &now).map_err(refused)?;
                launch(runner, &["rename", name, to])?;
            }
            AUTO_RESET => return Ok(Ran::Push(self.reset_view(counter, "", &now))),
            HISTORY => return Ok(Ran::Push(self.history_view(counter))),
            REMOVE => launch(runner, &["remove", name])?,
            _ => return Err(format!("`{id}` has no Action `{action}`")),
        }
        Ok(Ran::Close)
    }

    /// Runs a choice of an Auto reset list: a rule, never, or a stop.
    ///
    /// # Errors
    ///
    /// When the choice isn't one, the stop isn't a number of 1 or more, or `kubaos-counters`
    /// can't start.
    pub fn choose(
        &self,
        id: &str,
        action: &str,
        arguments: &BTreeMap<String, String>,
        runner: &impl Runner,
    ) -> Result<Ran, String> {
        let not_a_choice = || format!("`{id}` isn't an auto reset");
        if action != CHOOSE {
            return Err(format!("`{id}` has no Action `{action}`"));
        }
        let rest = id.strip_prefix(RESET_VIEW).ok_or_else(not_a_choice)?;
        let (counter_id, choice) = rest.split_once(':').ok_or_else(not_a_choice)?;
        let counter = self.by_id(counter_id)?;
        let name = counter.name.as_str();
        match choice.split_once(':') {
            Some(("rule", rule)) => launch(runner, &["rule", name, rule])?,
            Some(("stop", stop)) => launch(runner, &["stop", name, stop])?,
            None if choice == "never" => launch(runner, &["rule", name, "never"])?,
            None if choice == "stop" => {
                let stop = number(arguments, "Type where it stops, a whole number of 1 or more")?;
                if stop == 0 {
                    return Err("A counter can only stop at 1 or more".to_owned());
                }
                launch(runner, &["stop", name, &stop.to_string()])?;
            }
            _ => return Err(not_a_choice()),
        }
        Ok(Ran::Pop(format!("{RESET_VIEW}{counter_id}")))
    }

    /// The Auto reset list `view` as `text` reads, when it is one of this module's.
    pub fn filter(&self, view: &str, text: &str) -> Option<View> {
        let counter_id = view.strip_prefix(RESET_VIEW)?;
        let counter = self.by_id(counter_id).ok()?;
        Some(self.reset_view(counter, text, &Zoned::now()))
    }

    /// The Auto reset list of `counter` for `typed`: what it reads as (a rule, or a number for
    /// "When it reaches N"), or the examples; then Never and "When it reaches N…".
    fn reset_view(&self, counter: &Counter, typed: &str, now: &Zoned) -> View {
        let typed = typed.trim();
        let view = format!("{RESET_VIEW}{}", counter.id);
        let mut items = Vec::new();
        let mut title = format!("Auto reset for {}", counter.name);
        let stop = typed.parse::<u64>().ok().filter(|&stop| stop > 0);
        if let Some(stop) = stop {
            items.push(Self::choice(
                &view,
                &format!("stop:{stop}"),
                format!("When it reaches {stop}"),
                None,
            ));
        } else if typed.is_empty() {
            if let Some(now_item) = self.current(&view, counter, now) {
                items.push(now_item);
            }
            items.extend(self.examples(&view, now));
        } else if let Ok(rule) = Recurrence::parse(typed) {
            items.push(self.rule_choice(&view, rule, now, None));
        } else {
            title = format!(
                "Auto reset for {}: “{typed}” doesn't read; try one of these",
                counter.name
            );
            items.extend(self.examples(&view, now));
        }
        items.push(Self::choice(
            &view,
            "never",
            "Never".to_owned(),
            Some("No auto reset".to_owned()),
        ));
        if stop.is_none() {
            let mut reaches = Self::choice(
                &view,
                "stop",
                "When it reaches N…".to_owned(),
                Some("Stops at N, till you reset it; or type the number".to_owned()),
            );
            reaches.actions = vec![
                Action::new(CHOOSE, "Stop there")
                    .arguments(vec![Argument::new(VALUE).number().placeholder("1 or more")])
                    .role(ActionRole::Form),
            ];
            items.push(reaches);
        }
        View {
            id: view,
            title,
            content: ViewContent::List {
                items,
                filters_itself: true,
            },
        }
    }

    /// The counter's auto reset now, first in the list it opens with.
    fn current(&self, view: &str, counter: &Counter, now: &Zoned) -> Option<Item> {
        match counter.reset {
            AutoReset::Rule(rule) => Some(self.rule_choice(view, rule, now, Some("Now"))),
            AutoReset::Stop(stop) => Some(Self::choice(
                view,
                &format!("stop:{stop}"),
                format!("When it reaches {stop}"),
                Some("Now".to_owned()),
            )),
            AutoReset::Never => None,
        }
    }

    fn examples(&self, view: &str, now: &Zoned) -> Vec<Item> {
        recurrence::EXAMPLES
            .iter()
            .filter_map(|example| Recurrence::parse(example).ok())
            .map(|rule| self.rule_choice(view, rule, now, None))
            .collect()
    }

    /// A rule's choice: its words, and when it would next reset, after `now_label` when it is the
    /// counter's now.
    fn rule_choice(&self, view: &str, rule: Recurrence, now: &Zoned, now_label: Option<&str>) -> Item {
        let next = rule
            .next_after(now)
            .map(|at| format!("Next {}", live::when(&at, now, self.format)));
        let subtitle = match (now_label, next) {
            (Some(label), Some(next)) => Some(format!("{label} · {next}")),
            (Some(label), None) => Some(label.to_owned()),
            (None, next) => next,
        };
        Self::choice(view, &format!("rule:{rule}"), rule.words(self.format), subtitle)
    }

    fn choice(view: &str, choice: &str, title: String, subtitle: Option<String>) -> Item {
        Item {
            id: format!("{view}:{choice}"),
            kind: kind(RESET_KIND),
            identity: None,
            title,
            subtitle,
            icon: Some(ICON.to_owned()),
            keyword: None,
            keywords: Vec::new(),
            hidden: false,
            app_only: false,
            weak_keywords: Vec::new(),
            yields: false,
            actions: vec![Action::new(CHOOSE, "Reset this way")],
        }
    }

    /// The History View of `counter`: its changes, newest first, under the day of each.
    fn history_view(&self, counter: &Counter) -> View {
        let entries = self
            .log
            .as_deref()
            .map(history::read)
            .and_then(Result::ok)
            .unwrap_or_default();
        let tz = TimeZone::system();
        let rows = history::history(&entries, counter.id, Group::Change, Sort::Newest, &tz);
        View {
            id: format!("{HISTORY_VIEW}{}", counter.id),
            title: format!("History of {}", counter.name),
            content: ViewContent::Detail {
                markdown: history_markdown(&rows, &tz, self.format),
                metadata: vec![
                    MetadataEntry {
                        label: "Now".to_owned(),
                        value: counter.value.to_string(),
                    },
                    MetadataEntry {
                        label: "Auto reset".to_owned(),
                        value: match live::detail(counter.reset, self.format) {
                            detail if detail.is_empty() => "Never".to_owned(),
                            detail => detail,
                        },
                    },
                    MetadataEntry {
                        label: "Changes".to_owned(),
                        value: rows.len().to_string(),
                    },
                ],
            },
        }
    }

    /// The counter of the Item or View id `id`, made of `prefix` and the counter's id.
    fn by_item(&self, id: &str, prefix: &str) -> Result<&Counter, String> {
        let counter_id = id
            .strip_prefix(prefix)
            .ok_or_else(|| format!("`{id}` isn't a counter"))?;
        self.by_id(counter_id)
    }

    fn by_id(&self, counter_id: &str) -> Result<&Counter, String> {
        let gone = || "That counter isn't there any more".to_owned();
        let counter_id: u32 = counter_id.parse().map_err(|_| gone())?;
        self.counters
            .all()
            .iter()
            .find(|counter| counter.id == counter_id)
            .ok_or_else(gone)
    }
}

fn new_item() -> Item {
    Item {
        id: NEW_ID.to_owned(),
        kind: kind(NEW_KIND),
        identity: None,
        title: "New counter".to_owned(),
        subtitle: Some("A counter at 0 in the bar; type its name after new-counter".to_owned()),
        icon: Some(ICON.to_owned()),
        keyword: Some("new-counter".to_owned()),
        keywords: ["add counter", "create counter", "tally", "count", "tracker"]
            .map(str::to_owned)
            .to_vec(),
        hidden: false,
        app_only: false,
        weak_keywords: Vec::new(),
        yields: false,
        actions: vec![
            Action::new(CREATE, "Create").arguments(vec![Argument::new(NAME).placeholder(DEFAULT_NAME).optional()]),
        ],
    }
}

/// Every id of an Item this module always sends, which no action of the registry may take.
pub fn ids() -> impl Iterator<Item = &'static str> {
    [NEW_ID].into_iter()
}

/// The changes as Markdown: a heading a day, then a line a change, `9:05 · +1 → 13`.
fn history_markdown(rows: &[history::Row], tz: &TimeZone, format: Option<TimeFormat>) -> String {
    if rows.is_empty() {
        return "No changes yet.".to_owned();
    }
    let mut markdown = String::new();
    let mut day = None;
    for row in rows.iter().take(HISTORY_ROWS) {
        let at = row.at.to_zoned(tz.clone());
        if day != Some(at.date()) {
            day = Some(at.date());
            let _ = write!(markdown, "\n**{}**\n\n", at.strftime("%A %-d %B %Y"));
        }
        let time = match format {
            Some(TimeFormat::TwelveHour) => at.strftime("%-I:%M %p").to_string(),
            _ => at.strftime("%-H:%M").to_string(),
        };
        let label = row.kind.map_or("", |kind| kind.label());
        let _ = writeln!(markdown, "- {time} · {label} → {}", row.value);
    }
    if rows.len() > HISTORY_ROWS {
        let _ = write!(markdown, "\nThe {HISTORY_ROWS} newest of {} changes.\n", rows.len());
    }
    markdown.trim_start().to_owned()
}

/// A whole number typed into the Argument `value`.
fn number(arguments: &BTreeMap<String, String>, missing: &str) -> Result<u64, String> {
    let typed = arguments.get(VALUE).map_or("", |typed| typed.trim());
    if typed.is_empty() {
        return Err(missing.to_owned());
    }
    typed
        .parse()
        .map_err(|_| format!("`{typed}` isn't a whole number of 0 or more"))
}

/// Launches `kubaos-counters` with `args`, saying what it came to in a toast.
fn launch(runner: &impl Runner, words: &[&str]) -> Result<(), String> {
    let argv: Vec<String> = std::iter::once(PROGRAM)
        .chain(words.iter().copied())
        .chain(["--notify"])
        .map(str::to_owned)
        .collect();
    runner
        .launch(&argv, Duration::ZERO)
        .map_err(|err| Chain(&err).to_string())
}

/// The clock's time format, which the rules' words follow.
fn time_format() -> Option<TimeFormat> {
    let dir = kubaos_clock::settings::dir()?;
    kubaos_clock::settings::Settings::load(&dir.join(kubaos_clock::settings::FILE))
        .ok()?
        .time_format
}

/// The counters' file's name in KubaOS's state folder, which `main.rs` watches.
pub const FILE_NAME: &str = "counters.toml";

fn kind(name: &str) -> ItemKind {
    ItemKind::Namespaced(NamespacedKind::new(name).expect("the Extension's kinds are valid"))
}

#[cfg(test)]
mod tests;
