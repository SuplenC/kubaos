//! The idle settings, in the user layer: `~/.config/kubaos/desktop/idle/idle.toml`.
//!
//! ```toml
//! [battery]
//! lock = "5m"
//! screens_off = "10m"
//! suspend = "30m"
//! lid = "suspend"
//!
//! [ac]
//! lock = "15m"
//! suspend = "never"
//! lid = "nothing"
//! ```
//!
//! A wait is a duration as the clock reads a timer's (`90s`, `2m`, `1h30m`, `25` for minutes) or
//! `never`. The Command menu changes the file through `kubaos-idle set`, which edits it in place,
//! so the user's comments and layout stay.
//!
//! # Owns
//!
//! - The settings, their defaults and limits, reading the file and every change made to it.
//!
//! # Not here
//!
//! - The seed with its comments: `user-layer/desktop/idle/idle.toml`. What the settings come to:
//!   [`crate::hypridle`], [`crate::lid`]. Their words: [`crate::words`].

use std::fmt;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use kubaos_user_layer::App;
use serde::Deserialize;
use toml_edit::DocumentMut;
use toml_edit::value;

/// The settings file in the idle folder of the user layer.
pub const FILE: &str = "idle.toml";

/// The screen locks after 5 min idle, on battery and plugged in: long enough to read a page,
/// short enough that a laptop left on a café table locks before anyone sits down at it.
pub const DEFAULT_LOCK: Wait = Wait::from_secs(5 * 60);
/// The screens go off 5 min after the lock, so the lock screen shows for a while first.
pub const DEFAULT_SCREENS_OFF: Wait = Wait::from_secs(10 * 60);
/// On battery, suspend after 30 min idle, which saves the battery of a laptop forgotten open.
pub const DEFAULT_SUSPEND_ON_BATTERY: Wait = Wait::from_secs(30 * 60);
/// Plugged in, never suspend: a desk machine runs builds and downloads while nobody looks.
pub const DEFAULT_SUSPEND_ON_AC: Wait = Wait::NEVER;
/// Closing the lid on battery suspends, as logind's file says (`system/files/logind.conf.d/lid.conf`).
pub const DEFAULT_LID_ON_BATTERY: LidAction = LidAction::Suspend;
/// Closing the lid plugged in does nothing, as logind's file says.
pub const DEFAULT_LID_ON_AC: LidAction = LidAction::Nothing;

/// The longest wait: a day. hypridle counts in seconds, and a longer one is `never` in practice.
pub const MAX_WAIT: Wait = Wait::from_secs(24 * 3600);

/// One of the three things that happen after a time idle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timer {
    Lock,
    ScreensOff,
    Suspend,
}

impl Timer {
    pub const ALL: [Timer; 3] = [Timer::Lock, Timer::ScreensOff, Timer::Suspend];

    /// Its key in the file: `screens_off`.
    pub fn key(self) -> &'static str {
        match self {
            Timer::Lock => "lock",
            Timer::ScreensOff => "screens_off",
            Timer::Suspend => "suspend",
        }
    }

    /// Its name on the command line: `screens-off`.
    pub fn name(self) -> &'static str {
        match self {
            Timer::Lock => "lock",
            Timer::ScreensOff => "screens-off",
            Timer::Suspend => "suspend",
        }
    }

    /// The timer named `name`, as the command line or the file spells it.
    pub fn from_name(name: &str) -> Option<Timer> {
        match name {
            "lock" => Some(Timer::Lock),
            "screens-off" | "screens_off" | "screen-off" | "dpms" => Some(Timer::ScreensOff),
            "suspend" | "sleep" => Some(Timer::Suspend),
            _ => None,
        }
    }

    /// The shortest wait it takes. Under 30 s the screen would lock or go dark while someone
    /// reads, and a suspend under a minute would come before the lock screen has been seen.
    pub fn minimum(self) -> Wait {
        match self {
            Timer::Lock | Timer::ScreensOff => Wait::from_secs(30),
            Timer::Suspend => Wait::from_secs(60),
        }
    }

    /// The waits the menu offers, `never` last.
    pub fn choices(self) -> &'static [Wait] {
        const MIN: u32 = 60;
        const SCREEN: [Wait; 8] = [
            Wait::from_secs(MIN),
            Wait::from_secs(2 * MIN),
            Wait::from_secs(5 * MIN),
            Wait::from_secs(10 * MIN),
            Wait::from_secs(15 * MIN),
            Wait::from_secs(30 * MIN),
            Wait::from_secs(60 * MIN),
            Wait::NEVER,
        ];
        const SUSPEND: [Wait; 6] = [
            Wait::from_secs(10 * MIN),
            Wait::from_secs(15 * MIN),
            Wait::from_secs(30 * MIN),
            Wait::from_secs(60 * MIN),
            Wait::from_secs(120 * MIN),
            Wait::NEVER,
        ];
        match self {
            Timer::Lock | Timer::ScreensOff => &SCREEN,
            Timer::Suspend => &SUSPEND,
        }
    }

    /// The wait checked against the limits.
    ///
    /// # Errors
    ///
    /// [`SettingsError::TooSoon`] under [`Timer::minimum`], [`SettingsError::TooLong`] over
    /// [`MAX_WAIT`]. `never` is always fine.
    pub fn check(self, wait: Wait) -> Result<Wait, SettingsError> {
        if wait.is_never() {
            return Ok(wait);
        }
        if wait < self.minimum() {
            return Err(SettingsError::TooSoon { timer: self, wait });
        }
        if wait > MAX_WAIT {
            return Err(SettingsError::TooLong { timer: self, wait });
        }
        Ok(wait)
    }
}

/// How long idle before something happens, in seconds; [`Wait::NEVER`] for never.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Wait(u32);

impl Wait {
    pub const NEVER: Wait = Wait(0);

    #[must_use]
    pub const fn from_secs(secs: u32) -> Self {
        Wait(secs)
    }

    /// The seconds; 0 for never.
    pub fn secs(self) -> u32 {
        self.0
    }

    pub fn is_never(self) -> bool {
        self.0 == 0
    }

    /// Reads a wait as typed: `never` (or `off`, `0`), or a duration as the clock reads a timer's:
    /// `90s`, `2m`, `1h30m`, `5h`, `25` for minutes.
    ///
    /// # Errors
    ///
    /// [`SettingsError::NotAWait`] for anything else, a fraction of a second included.
    pub fn parse(text: &str) -> Result<Wait, SettingsError> {
        let trimmed = text.trim().to_lowercase();
        if matches!(trimmed.as_str(), "never" | "off" | "0" | "no") {
            return Ok(Wait::NEVER);
        }
        let not_a_wait = || SettingsError::NotAWait(text.trim().to_owned());
        let duration = kubaos_clock::duration(&trimmed).ok_or_else(not_a_wait)?;
        if duration.subsec_nanos() != 0 {
            return Err(not_a_wait());
        }
        // A wait past `u32` is past [`MAX_WAIT`] too, which [`Timer::check`] refuses.
        Ok(Wait(u32::try_from(duration.as_secs()).unwrap_or(u32::MAX)))
    }

    /// As the file writes it: `never`, `90s` as `1m30s`, `1h30m`.
    pub fn canonical(self) -> String {
        if self.is_never() {
            return "never".to_owned();
        }
        let (hours, minutes, seconds) = (self.0 / 3600, self.0 / 60 % 60, self.0 % 60);
        let mut out = String::new();
        for (amount, unit) in [(hours, "h"), (minutes, "m"), (seconds, "s")] {
            if amount > 0 {
                let _ = write!(out, "{amount}{unit}");
            }
        }
        out
    }
}

/// What closing the lid does, when the laptop isn't docked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LidAction {
    Suspend,
    Lock,
    Nothing,
}

impl LidAction {
    pub const ALL: [LidAction; 3] = [LidAction::Suspend, LidAction::Lock, LidAction::Nothing];

    /// Its name in the file and on the command line.
    pub fn name(self) -> &'static str {
        match self {
            LidAction::Suspend => "suspend",
            LidAction::Lock => "lock",
            LidAction::Nothing => "nothing",
        }
    }

    /// The action named `name`; `ignore` and `none` are `nothing`, as logind spells it.
    pub fn from_name(name: &str) -> Option<LidAction> {
        match name.trim().to_lowercase().as_str() {
            "suspend" | "sleep" => Some(LidAction::Suspend),
            "lock" => Some(LidAction::Lock),
            "nothing" | "ignore" | "none" | "do-nothing" => Some(LidAction::Nothing),
            _ => None,
        }
    }
}

/// Where the power comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Power {
    Battery,
    Ac,
}

impl Power {
    pub const ALL: [Power; 2] = [Power::Battery, Power::Ac];

    /// Its table in the file.
    pub fn key(self) -> &'static str {
        match self {
            Power::Battery => "battery",
            Power::Ac => "ac",
        }
    }
}

/// Which power sources a change is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sources {
    Both,
    Only(Power),
}

impl Sources {
    /// The word typed after a value: `battery`, or `ac` (`plugged`, `plugged-in`); `both`.
    pub fn from_word(word: &str) -> Option<Sources> {
        match word.trim().to_lowercase().as_str() {
            "both" | "always" => Some(Sources::Both),
            "battery" | "bat" => Some(Sources::Only(Power::Battery)),
            "ac" | "plugged" | "plugged-in" | "plugged_in" | "charging" => Some(Sources::Only(Power::Ac)),
            _ => None,
        }
    }

    /// Its name on the command line.
    pub fn name(self) -> &'static str {
        match self {
            Sources::Both => "both",
            Sources::Only(power) => power.key(),
        }
    }

    fn powers(self) -> &'static [Power] {
        match self {
            Sources::Both => &Power::ALL,
            Sources::Only(Power::Battery) => &[Power::Battery],
            Sources::Only(Power::Ac) => &[Power::Ac],
        }
    }
}

/// The settings for one power source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OnPower {
    pub lock: Wait,
    pub screens_off: Wait,
    pub suspend: Wait,
    pub lid: LidAction,
}

impl OnPower {
    pub fn wait(&self, timer: Timer) -> Wait {
        match timer {
            Timer::Lock => self.lock,
            Timer::ScreensOff => self.screens_off,
            Timer::Suspend => self.suspend,
        }
    }
}

/// The idle settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub battery: OnPower,
    pub ac: OnPower,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            battery: OnPower {
                lock: DEFAULT_LOCK,
                screens_off: DEFAULT_SCREENS_OFF,
                suspend: DEFAULT_SUSPEND_ON_BATTERY,
                lid: DEFAULT_LID_ON_BATTERY,
            },
            ac: OnPower {
                lock: DEFAULT_LOCK,
                screens_off: DEFAULT_SCREENS_OFF,
                suspend: DEFAULT_SUSPEND_ON_AC,
                lid: DEFAULT_LID_ON_AC,
            },
        }
    }
}

/// A wait as written: a string, or a number of minutes.
#[derive(Deserialize)]
#[serde(untagged)]
enum WaitToml {
    Minutes(i64),
    Text(String),
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct OnPowerToml {
    lock: Option<WaitToml>,
    screens_off: Option<WaitToml>,
    suspend: Option<WaitToml>,
    lid: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsToml {
    #[serde(default)]
    battery: OnPowerToml,
    #[serde(default)]
    ac: OnPowerToml,
}

/// Why the settings couldn't be read or changed. The messages are what the toast says.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SettingsError {
    #[error("reading the idle settings")]
    Read(#[source] io::Error),
    #[error("parsing the idle settings")]
    Parse(#[source] toml::de::Error),
    #[error("the idle settings aren't TOML that can be edited")]
    Edit(#[source] toml_edit::TomlError),
    #[error("writing the idle settings")]
    Write(#[source] io::Error),
    #[error("`{0}` isn't a time to wait: try 90s, 2m, 1h30m or never")]
    NotAWait(String),
    #[error("{} can't come sooner than {} idle, not after {}", what(*.timer), crate::words::wait(.timer.minimum()), crate::words::wait(*.wait))]
    TooSoon { timer: Timer, wait: Wait },
    #[error("{} can wait a day at most, not {}; for longer, say never", what(*.timer), crate::words::wait(*.wait))]
    TooLong { timer: Timer, wait: Wait },
    #[error("`{0}` isn't something the lid can do: suspend, lock or nothing")]
    NotALidAction(String),
    #[error("`{key}` in [{power}]: {problem}")]
    Value {
        power: &'static str,
        key: &'static str,
        problem: Box<SettingsError>,
    },
}

/// The timer's name at the start of a sentence.
fn what(timer: Timer) -> &'static str {
    match timer {
        Timer::Lock => "The lock",
        Timer::ScreensOff => "Screens off",
        Timer::Suspend => "Suspend",
    }
}

impl fmt::Display for Wait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::words::wait(*self))
    }
}

impl Settings {
    /// The settings for `power`.
    pub fn on(&self, power: Power) -> &OnPower {
        match power {
            Power::Battery => &self.battery,
            Power::Ac => &self.ac,
        }
    }

    fn on_mut(&mut self, power: Power) -> &mut OnPower {
        match power {
            Power::Battery => &mut self.battery,
            Power::Ac => &mut self.ac,
        }
    }

    /// The waits of `timer`, on battery and plugged in.
    pub fn waits(&self, timer: Timer) -> (Wait, Wait) {
        (self.battery.wait(timer), self.ac.wait(timer))
    }

    /// Parses the settings file's text; a key left out takes its default.
    ///
    /// # Errors
    ///
    /// [`SettingsError::Parse`] for malformed TOML or an unknown key, [`SettingsError::Value`]
    /// for a wait or a lid action it can't take.
    pub fn parse(text: &str) -> Result<Self, SettingsError> {
        let read: SettingsToml = toml::from_str(text).map_err(SettingsError::Parse)?;
        let mut settings = Settings::default();
        for (power, table) in [(Power::Battery, read.battery), (Power::Ac, read.ac)] {
            let on = settings.on_mut(power);
            let in_table = |key: &'static str| {
                move |problem: SettingsError| SettingsError::Value {
                    power: power.key(),
                    key,
                    problem: Box::new(problem),
                }
            };
            for (timer, written) in [
                (Timer::Lock, table.lock),
                (Timer::ScreensOff, table.screens_off),
                (Timer::Suspend, table.suspend),
            ] {
                let Some(written) = written else { continue };
                let wait = match written {
                    WaitToml::Minutes(minutes) => u32::try_from(minutes)
                        .ok()
                        .and_then(|minutes| minutes.checked_mul(60))
                        .map(Wait)
                        .ok_or_else(|| SettingsError::NotAWait(minutes.to_string())),
                    WaitToml::Text(text) => Wait::parse(&text),
                }
                .and_then(|wait| timer.check(wait))
                .map_err(in_table(timer.key()))?;
                match timer {
                    Timer::Lock => on.lock = wait,
                    Timer::ScreensOff => on.screens_off = wait,
                    Timer::Suspend => on.suspend = wait,
                }
            }
            if let Some(lid) = table.lid {
                on.lid =
                    LidAction::from_name(&lid).ok_or_else(|| in_table("lid")(SettingsError::NotALidAction(lid)))?;
            }
        }
        Ok(settings)
    }

    /// Reads the settings file; a missing file is the defaults.
    ///
    /// # Errors
    ///
    /// [`SettingsError::Read`] when the file exists but can't be read, else as in
    /// [`Settings::parse`].
    pub fn load(path: &Path) -> Result<Self, SettingsError> {
        match fs::read_to_string(path) {
            Ok(text) => Self::parse(&text),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(SettingsError::Read(err)),
        }
    }
}

/// The idle folder in the user layer: `~/.config/kubaos/desktop/idle`.
pub fn dir() -> Option<PathBuf> {
    kubaos_user_layer::config_home().map(|config| kubaos_user_layer::dir(&config, App::Idle))
}

/// The settings file: `~/.config/kubaos/desktop/idle/idle.toml`.
pub fn path() -> Option<PathBuf> {
    dir().map(|dir| dir.join(FILE))
}

/// Sets `timer` to `wait` for `sources`, and returns the settings it comes to.
///
/// # Errors
///
/// As [`Timer::check`] for a wait out of its limits, else as reading and writing the file.
pub fn set_wait(path: &Path, timer: Timer, wait: Wait, sources: Sources) -> Result<Settings, SettingsError> {
    let wait = timer.check(wait)?;
    edit(path, sources, timer.key(), &wait.canonical())
}

/// Sets what closing the lid does for `sources`, and returns the settings it comes to.
///
/// # Errors
///
/// As reading and writing the file.
pub fn set_lid(path: &Path, action: LidAction, sources: Sources) -> Result<Settings, SettingsError> {
    edit(path, sources, "lid", action.name())
}

/// Takes every setting out of the file, so KubaOS's defaults apply; the comments stay.
///
/// # Errors
///
/// As reading and writing the file.
pub fn reset(path: &Path) -> Result<Settings, SettingsError> {
    let mut doc = document(path)?;
    // The comments above a table are its decor; they stay, at the end of the file.
    let mut kept = String::new();
    for power in Power::ALL {
        let Some(item) = doc.remove(power.key()) else {
            continue;
        };
        if let Some(prefix) = item
            .as_table()
            .and_then(|table| table.decor().prefix())
            .and_then(|p| p.as_str())
        {
            kept.push_str(prefix);
        }
    }
    kept.push_str(doc.trailing().as_str().unwrap_or_default());
    doc.set_trailing(kept);
    save(path, &doc)
}

fn edit(path: &Path, sources: Sources, key: &str, setting: &str) -> Result<Settings, SettingsError> {
    let mut doc = document(path)?;
    Settings::parse(&doc.to_string())?;
    for power in sources.powers() {
        if !doc.contains_key(power.key()) {
            // A new table goes at the end, below the comments there, which are the document's
            // trailing text until a table takes them as its own.
            let mut table = toml_edit::Table::new();
            let trailing = doc.trailing().as_str().unwrap_or_default().to_owned();
            if !trailing.trim().is_empty() {
                table.decor_mut().set_prefix(trailing + "\n");
                doc.set_trailing("");
            }
            doc.insert(power.key(), toml_edit::Item::Table(table));
        }
        let table = doc[power.key()]
            .as_table_like_mut()
            .expect("a [battery] or [ac] that parsed is a table");
        table.insert(key, value(setting));
    }
    save(path, &doc)
}

/// The settings file as a document to edit, empty when missing.
fn document(path: &Path) -> Result<DocumentMut, SettingsError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(SettingsError::Read(err)),
    };
    text.parse().map_err(SettingsError::Edit)
}

/// Writes `doc` over the file at `path` through a temporary file and a rename, once it parses.
fn save(path: &Path, doc: &DocumentMut) -> Result<Settings, SettingsError> {
    let text = doc.to_string();
    let settings = Settings::parse(&text)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(SettingsError::Write)?;
    }
    let tmp = path.with_extension("toml.new");
    fs::write(&tmp, text).map_err(SettingsError::Write)?;
    fs::rename(&tmp, path).map_err(SettingsError::Write)?;
    Ok(settings)
}

#[cfg(test)]
mod tests;
