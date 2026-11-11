//! The idle settings in the menu: "Lock after…", "Screens off after…", "Suspend after…" and "When
//! the lid closes…". Enter lists the usual choices, the ones in use marked, each for both power
//! sources or, from its Actions, one; or a time typed after the word, `lock-after 2m`,
//! `suspend-after 1h30m ac`, sets it at once.
//!
//! A choice is checked here, so a refusal reads in the menu, then launched as `kubaos-idle set`,
//! which writes the file, restarts hypridle and says what it came to in a toast.
//!
//! "Keep awake for…" takes what is typed after `keep-awake` the same way: a while (`30m`) or a
//! battery level (`25%`), checked here, then launched as `kubaos-idle awake for` or `until`. "Keep
//! awake with the lid closed for…" does the same after `keep-awake-lid`, as `kubaos-idle awake lid
//! for` or `until`.
//!
//! # Owns
//!
//! - The four Items, their choice lists, and what picking one runs.
//! - "Keep awake for…" and "Keep awake with the lid closed for…", and what typing after them runs.
//!
//! # Not here
//!
//! - The settings, their limits and their words, and what a typed Keep awake reads as:
//!   `kubaos_idle`. The other Keep awake Items and the scheduled actions: actions of the registry
//!   (`config/kotodama/kubaos.toml`).

use std::collections::BTreeMap;
use std::time::Duration;

use kotodama_extension::Chain;
use kotodama_protocol::Action;
use kotodama_protocol::Argument;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;
use kotodama_protocol::View;
use kotodama_protocol::ViewContent;
use kubaos_idle::awake::Length;
use kubaos_idle::settings::LidAction;
use kubaos_idle::settings::Power;
use kubaos_idle::settings::Settings;
use kubaos_idle::settings::Sources;
use kubaos_idle::settings::Timer;
use kubaos_idle::settings::Wait;
use kubaos_idle::words;

use crate::run::Runner;
use crate::toggles::Ran;

/// The Item kind of the four settings.
pub const KIND: &str = "kubaos.idle";
/// The Item kind of a choice in a setting's list.
pub const CHOICE_KIND: &str = "kubaos.idle-choice";

/// A setting's one Action: the list, or the typed value.
pub const CHOOSE: &str = "choose";
/// A choice's Actions: for both power sources, the primary one, or one of them.
pub const BOTH: &str = "both";
pub const BATTERY: &str = "battery";
pub const AC: &str = "ac";

/// The Item kind of "Keep awake for…" and "Keep awake with the lid closed for…".
pub const KEEP_AWAKE_KIND: &str = "kubaos.keep-awake";
/// "Keep awake for…"'s id, which `hide` names it by.
pub const KEEP_AWAKE_ID: &str = "keep-awake-for";
/// "Keep awake with the lid closed for…"'s id.
pub const KEEP_AWAKE_LID_ID: &str = "keep-awake-lid-for";
/// Their one Action: keep awake as typed.
pub const KEEP: &str = "keep";

/// The Argument typed after a setting's word, or Keep awake's.
const VALUE: &str = "value";

const ICON: &str = "preferences-desktop-screensaver";

/// One of the four settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    Timer(Timer),
    Lid,
}

impl Setting {
    pub const ALL: [Setting; 4] = [
        Setting::Timer(Timer::Lock),
        Setting::Timer(Timer::ScreensOff),
        Setting::Timer(Timer::Suspend),
        Setting::Lid,
    ];

    /// Its Item's id, which `hide` names it by.
    pub fn id(self) -> &'static str {
        match self {
            Setting::Timer(Timer::Lock) => "idle-lock",
            Setting::Timer(Timer::ScreensOff) => "idle-screens-off",
            Setting::Timer(Timer::Suspend) => "idle-suspend",
            Setting::Lid => "idle-lid",
        }
    }

    fn from_id(id: &str) -> Option<Setting> {
        Setting::ALL.into_iter().find(|setting| setting.id() == id)
    }

    /// Its name to `kubaos-idle set`.
    fn name(self) -> &'static str {
        match self {
            Setting::Timer(timer) => timer.name(),
            Setting::Lid => "lid",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Setting::Timer(Timer::Lock) => "Lock after…",
            Setting::Timer(Timer::ScreensOff) => "Screens off after…",
            Setting::Timer(Timer::Suspend) => "Suspend after…",
            Setting::Lid => "When the lid closes…",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            Setting::Timer(Timer::Lock) => "How long idle before the screen locks: pick, or type lock-after 2m",
            Setting::Timer(Timer::ScreensOff) => {
                "How long idle before the screens go dark: pick, or type screens-off-after 10m"
            }
            Setting::Timer(Timer::Suspend) => {
                "How long idle before it suspends: pick, or type suspend-after 1h30m battery"
            }
            Setting::Lid => "Suspend, lock or nothing, on battery and plugged in; docked, always nothing",
        }
    }

    fn keyword(self) -> &'static str {
        match self {
            Setting::Timer(Timer::Lock) => "lock-after",
            Setting::Timer(Timer::ScreensOff) => "screens-off-after",
            Setting::Timer(Timer::Suspend) => "suspend-after",
            Setting::Lid => "lid",
        }
    }

    fn keywords(self) -> &'static [&'static str] {
        match self {
            Setting::Timer(Timer::Lock) => &["auto lock", "idle", "lock screen", "timeout", "hypridle"],
            Setting::Timer(Timer::ScreensOff) => &["screen off", "dpms", "display off", "idle", "blank"],
            Setting::Timer(Timer::Suspend) => &["sleep", "auto suspend", "idle", "battery"],
            Setting::Lid => &["lid close", "laptop lid", "clamshell", "suspend on lid"],
        }
    }

    fn placeholder(self) -> &'static str {
        match self {
            Setting::Timer(_) => "2m, 1h30m or never; then battery or ac for one",
            Setting::Lid => "suspend, lock or nothing; then battery or ac for one",
        }
    }

    fn item(self) -> Item {
        let choose = Action::new(CHOOSE, "Choose")
            .arguments(vec![Argument::new(VALUE).placeholder(self.placeholder()).optional()]);
        Item {
            id: self.id().to_owned(),
            kind: kind(KIND),
            identity: None,
            title: self.title().to_owned(),
            subtitle: Some(self.subtitle().to_owned()),
            icon: Some(ICON.to_owned()),
            keyword: Some(self.keyword().to_owned()),
            keywords: self.keywords().iter().map(|&word| word.to_owned()).collect(),
            hidden: false,
            app_only: false,
            weak_keywords: Vec::new(),
            yields: false,
            actions: vec![choose],
        }
    }

    /// The choices as `kubaos-idle set` takes them, with their titles and what each is now for.
    fn choices(self, settings: &Settings) -> Vec<(String, String, Option<&'static str>)> {
        match self {
            Setting::Timer(timer) => timer
                .choices()
                .iter()
                .map(|&wait| {
                    let title = if wait.is_never() {
                        "Never".to_owned()
                    } else {
                        words::wait(wait)
                    };
                    let (battery, ac) = settings.waits(timer);
                    (wait.canonical(), title, now_for(battery == wait, ac == wait))
                })
                .collect(),
            Setting::Lid => LidAction::ALL
                .into_iter()
                .map(|action| {
                    let title = match action {
                        LidAction::Suspend => "Suspend",
                        LidAction::Lock => "Lock",
                        LidAction::Nothing => "Do nothing",
                    };
                    let now = now_for(settings.battery.lid == action, settings.ac.lid == action);
                    (action.name().to_owned(), title.to_owned(), now)
                })
                .collect(),
        }
    }
}

/// The words saying a choice is in use, and where.
fn now_for(battery: bool, ac: bool) -> Option<&'static str> {
    match (battery, ac) {
        (true, true) => Some("Now, on battery and plugged in"),
        (true, false) => Some("Now, on battery"),
        (false, true) => Some("Now, plugged in"),
        (false, false) => None,
    }
}

/// The four settings' Items and the two typed Keep awakes.
pub fn items() -> Vec<Item> {
    Setting::ALL
        .into_iter()
        .map(Setting::item)
        .chain([keep_awake_item(), keep_awake_lid_item()])
        .collect()
}

/// Every id of an Item this module sends, which no action of the registry may take.
pub fn ids() -> impl Iterator<Item = &'static str> {
    Setting::ALL
        .map(Setting::id)
        .into_iter()
        .chain([KEEP_AWAKE_ID, KEEP_AWAKE_LID_ID])
}

fn keep_awake_item() -> Item {
    typed_keep_awake(
        KEEP_AWAKE_ID,
        "Keep awake for…",
        "No auto-lock, screens off or suspend, as long as you say (30m, 3h) or until the battery is at a \
         level (25%); the lid does as set",
        "keep-awake",
        "30m, 3h, or a battery level: 25%",
        &["caffeine", "keep awake", "no sleep", "battery level"],
    )
}

fn keep_awake_lid_item() -> Item {
    typed_keep_awake(
        KEEP_AWAKE_LID_ID,
        "Keep awake with the lid closed for…",
        "As Keep awake, and closing the lid does nothing, as long as you say (2h) or until the battery is at \
         a level (25%)",
        "keep-awake-lid",
        "2h, or a battery level: 25%",
        &[
            "caffeine",
            "keep awake",
            "no sleep",
            "lid closed",
            "clamshell",
            "battery level",
        ],
    )
}

fn typed_keep_awake(
    id: &str,
    title: &str,
    subtitle: &str,
    keyword: &str,
    placeholder: &str,
    keywords: &[&str],
) -> Item {
    let keep = Action::new(KEEP, "Keep awake").arguments(vec![Argument::new(VALUE).placeholder(placeholder)]);
    Item {
        id: id.to_owned(),
        kind: kind(KEEP_AWAKE_KIND),
        identity: None,
        title: title.to_owned(),
        subtitle: Some(subtitle.to_owned()),
        icon: Some(ICON.to_owned()),
        keyword: Some(keyword.to_owned()),
        keywords: keywords.iter().map(|&word| word.to_owned()).collect(),
        hidden: false,
        app_only: false,
        weak_keywords: Vec::new(),
        yields: false,
        actions: vec![keep],
    }
}

/// Keeps awake as typed after the word of the Item `id`: a while, or until the battery is at a
/// level; "Keep awake with the lid closed for…" holds the lid too.
///
/// # Errors
///
/// When nothing or neither was typed, the level is outside 1 % to 99 %, or `kubaos-idle` can't
/// start; the message is what the menu shows.
pub fn keep_awake(id: &str, arguments: &BTreeMap<String, String>, runner: &impl Runner) -> Result<Ran, String> {
    let typed = arguments.get(VALUE).map_or("", |typed| typed.trim());
    if typed.is_empty() {
        return Err("Type how long to keep awake, 30m or 3h, or a battery level, 25%".to_owned());
    }
    let mut argv = vec!["kubaos-idle".to_owned(), "awake".to_owned()];
    if id == KEEP_AWAKE_LID_ID {
        argv.push("lid".to_owned());
    }
    match Length::parse(typed).map_err(|err| Chain(&err).to_string())? {
        Length::For(_) => argv.extend(["for".to_owned(), typed.to_owned()]),
        Length::UntilBattery(level) => argv.extend(["until".to_owned(), format!("{}%", level.percent())]),
    }
    runner
        .launch(&argv, Duration::ZERO)
        .map_err(|err| Chain(&err).to_string())?;
    Ok(Ran::Close)
}

/// Runs a setting's Action: what was typed after its word, or else its list of choices, marked
/// as `settings` has them.
///
/// # Errors
///
/// When there is no such setting, the typed value isn't one it takes, or `kubaos-idle` can't
/// start; the message is what the menu shows.
pub fn run(
    id: &str,
    arguments: &BTreeMap<String, String>,
    settings: &Settings,
    runner: &impl Runner,
) -> Result<Ran, String> {
    let setting = Setting::from_id(id).ok_or_else(|| format!("`{id}` isn't an idle setting"))?;
    let typed = arguments.get(VALUE).map_or("", |typed| typed.trim());
    if typed.is_empty() {
        return Ok(Ran::Push(list(setting, settings)));
    }
    let mut words: Vec<&str> = typed.split_whitespace().collect();
    let sources = match words.as_slice() {
        [_, .., last] => Sources::from_word(last),
        _ => None,
    };
    if sources.is_some() {
        words.pop();
    }
    set(setting, &words.join(" "), sources.unwrap_or(Sources::Both), runner)?;
    Ok(Ran::Close)
}

/// Runs a choice from a setting's list, for the power sources its Action names.
///
/// # Errors
///
/// When the choice or the Action isn't one, or `kubaos-idle` can't start.
pub fn choose(id: &str, action: &str, runner: &impl Runner) -> Result<Ran, String> {
    let not_a_choice = || format!("`{id}` isn't an idle choice");
    let (setting, value) = id.split_once(':').ok_or_else(not_a_choice)?;
    let setting = Setting::from_id(setting).ok_or_else(not_a_choice)?;
    let sources = match action {
        BOTH => Sources::Both,
        BATTERY => Sources::Only(Power::Battery),
        AC => Sources::Only(Power::Ac),
        _ => return Err(format!("`{id}` has no Action `{action}`")),
    };
    set(setting, value, sources, runner)?;
    Ok(Ran::Pop(setting.id().to_owned()))
}

/// Checks `value` for `setting`, then launches `kubaos-idle set`.
fn set(setting: Setting, value: &str, sources: Sources, runner: &impl Runner) -> Result<(), String> {
    match setting {
        Setting::Timer(timer) => {
            Wait::parse(value)
                .and_then(|wait| timer.check(wait))
                .map_err(|err| Chain(&err).to_string())?;
        }
        Setting::Lid => {
            LidAction::from_name(value)
                .ok_or_else(|| format!("`{value}` isn't something the lid can do: suspend, lock or nothing"))?;
        }
    }
    let argv: Vec<String> = ["kubaos-idle", "set", setting.name(), value, sources.name()]
        .map(str::to_owned)
        .to_vec();
    runner
        .launch(&argv, Duration::ZERO)
        .map_err(|err| Chain(&err).to_string())
}

/// The list View of `setting`'s choices.
fn list(setting: Setting, settings: &Settings) -> View {
    let items = setting
        .choices(settings)
        .into_iter()
        .map(|(value, title, now)| Item {
            id: format!("{}:{value}", setting.id()),
            kind: kind(CHOICE_KIND),
            identity: None,
            title,
            subtitle: now.map(str::to_owned),
            icon: Some(ICON.to_owned()),
            keyword: None,
            keywords: Vec::new(),
            hidden: false,
            app_only: false,
            weak_keywords: Vec::new(),
            yields: false,
            actions: vec![
                Action::new(BOTH, "On battery and plugged in"),
                Action::new(BATTERY, "On battery only"),
                Action::new(AC, "Plugged in only"),
            ],
        })
        .collect();
    View {
        id: setting.id().to_owned(),
        title: setting.title().trim_end_matches('…').to_owned(),
        content: ViewContent::List {
            items,
            filters_itself: false,
        },
    }
}

fn kind(name: &str) -> ItemKind {
    ItemKind::Namespaced(NamespacedKind::new(name).expect("the Extension's kinds are valid"))
}

#[cfg(test)]
mod tests;
