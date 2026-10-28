//! The clock's settings, in the user layer: `~/.config/kubaos/desktop/clock/clock.toml`.
//!
//! ```toml
//! zones = ["America/New_York", "Asia/Tokyo"]
//! holidays = "pl"
//! first_weekday = "monday"
//! analog = false
//! bar_countdown = "Vacation"
//! time_format = "24h"
//! date_order = "day-month"
//!
//! [[countdowns]]
//! name = "Vacation"
//! at = "2026-12-19"
//! ```
//!
//! The popup and the Command menu change it through `kubaos-clock`, which edits the file in
//! place, so the user's comments and layout stay.
//!
//! # Owns
//!
//! - Reading the file, the defaults, and every change the popup and the menu make to it.
//! - Where the file and the user's holiday rules are.
//!
//! # Not here
//!
//! - The seed with its comments: `user-layer/desktop/clock/clock.toml`. What a countdown's `at` may
//!   say: [`crate::query`], through `crate::serve`.

use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use jiff::civil::Weekday;
use kubaos_user_layer::App;
use serde::Deserialize;
use toml_edit::DocumentMut;
use toml_edit::value;

use crate::holidays::Countries;
use crate::holidays::RulesError;
use crate::words;

/// The most extra clocks the popup shows; more crowd its header.
pub const MAX_ZONES: usize = 4;

/// The first day of the week by default: ISO 8601's, and most of Europe's.
pub const DEFAULT_FIRST_WEEKDAY: Weekday = Weekday::Monday;

/// How the shell writes the time. Without one, the locale's, as Qt reads `LC_TIME`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeFormat {
    /// `14:05`.
    TwentyFourHour,
    /// `2:05 PM`.
    TwelveHour,
}

impl TimeFormat {
    fn parse(text: &str) -> Option<Self> {
        match text {
            "24h" | "24" => Some(TimeFormat::TwentyFourHour),
            "12h" | "12" => Some(TimeFormat::TwelveHour),
            _ => None,
        }
    }

    /// The name in the file and in the popup's state.
    pub fn name(self) -> &'static str {
        match self {
            TimeFormat::TwentyFourHour => "24h",
            TimeFormat::TwelveHour => "12h",
        }
    }
}

/// Whether the shell writes the day or the month first. Without one, the locale's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateOrder {
    /// `Monday, 26 October`.
    DayMonth,
    /// `Monday, October 26`.
    MonthDay,
}

impl DateOrder {
    fn parse(text: &str) -> Option<Self> {
        match text {
            "day-month" | "day" => Some(DateOrder::DayMonth),
            "month-day" | "month" => Some(DateOrder::MonthDay),
            _ => None,
        }
    }

    /// The name in the file and in the popup's state.
    pub fn name(self) -> &'static str {
        match self {
            DateOrder::DayMonth => "day-month",
            DateOrder::MonthDay => "month-day",
        }
    }
}

/// A date or moment counted down to.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Countdown {
    pub name: String,
    /// `2026-12-19`, or an instant with its offset, `2026-12-19T17:00:00+01:00`.
    pub at: String,
}

/// The clock's settings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    /// Extra clocks, by IANA name or a name [`crate::zones::Zones`] finds.
    pub zones: Vec<String>,
    /// A country code for the holidays, `""` for none, or `None` for the locale's country.
    pub holidays: Option<String>,
    pub first_weekday: Weekday,
    /// The popup's analog face.
    pub analog: bool,
    /// The countdown shown next to the bar's clock, by name; `""` for none.
    pub bar_countdown: String,
    pub countdowns: Vec<Countdown>,
    /// `None` for the locale's.
    pub time_format: Option<TimeFormat>,
    /// `None` for the locale's.
    pub date_order: Option<DateOrder>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            zones: Vec::new(),
            holidays: None,
            first_weekday: DEFAULT_FIRST_WEEKDAY,
            analog: false,
            bar_countdown: String::new(),
            countdowns: Vec::new(),
            time_format: None,
            date_order: None,
        }
    }
}

/// The file as written; every key may be left out.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsToml {
    #[serde(default)]
    zones: Vec<String>,
    holidays: Option<String>,
    first_weekday: Option<String>,
    #[serde(default)]
    analog: bool,
    #[serde(default)]
    bar_countdown: String,
    #[serde(default)]
    countdowns: Vec<Countdown>,
    time_format: Option<String>,
    date_order: Option<String>,
}

/// Why the settings couldn't be read or changed.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("reading the clock settings")]
    Read(#[source] io::Error),
    #[error("parsing the clock settings")]
    Parse(#[source] toml::de::Error),
    #[error("the clock settings aren't TOML that can be edited")]
    Edit(#[source] toml_edit::TomlError),
    #[error("writing the clock settings")]
    Write(#[source] io::Error),
    #[error("`first_weekday` should be a weekday, not {0:?}")]
    Weekday(String),
    #[error("{MAX_ZONES} zones at most; remove one first")]
    TooManyZones,
    #[error("no zone {0:?} in the settings")]
    NoZone(String),
    #[error("no countdown {0:?} in the settings")]
    NoCountdown(String),
    #[error("no country {0:?} with holidays; one of {1}")]
    NoCountry(String, String),
    #[error("{key} can't be {value:?}")]
    Value { key: &'static str, value: String },
    #[error("no setting {0:?}; one of holidays, first-weekday, analog, bar-countdown, time-format, date-order")]
    NoSetting(String),
}

impl SettingsError {
    fn read(err: io::Error) -> Self {
        SettingsError::Read(err)
    }

    fn write(err: io::Error) -> Self {
        SettingsError::Write(err)
    }
}

impl Settings {
    /// Parses the settings file's text; a key left out takes its default.
    ///
    /// # Errors
    ///
    /// [`SettingsError::Parse`] for malformed TOML or an unknown key, [`SettingsError::Weekday`]
    /// for a `first_weekday` that isn't one, [`SettingsError::Value`] for a `time_format` or
    /// `date_order` it doesn't know.
    pub fn parse(text: &str) -> Result<Self, SettingsError> {
        let read: SettingsToml = toml::from_str(text).map_err(SettingsError::Parse)?;
        let first_weekday = match read.first_weekday {
            Some(name) => words::weekday(&name.to_lowercase()).ok_or(SettingsError::Weekday(name))?,
            None => DEFAULT_FIRST_WEEKDAY,
        };
        let time_format = read
            .time_format
            .map(|text| {
                TimeFormat::parse(&text).ok_or(SettingsError::Value {
                    key: "time_format",
                    value: text,
                })
            })
            .transpose()?;
        let date_order = read
            .date_order
            .map(|text| {
                DateOrder::parse(&text).ok_or(SettingsError::Value {
                    key: "date_order",
                    value: text,
                })
            })
            .transpose()?;
        Ok(Settings {
            zones: read.zones,
            holidays: read.holidays,
            first_weekday,
            analog: read.analog,
            bar_countdown: read.bar_countdown,
            countdowns: read.countdowns,
            time_format,
            date_order,
        })
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
            Err(err) => Err(SettingsError::read(err)),
        }
    }

    /// The country whose holidays count: the setting, or else the locale's region (`pl_PL.UTF-8`
    /// is `pl`) if there are rules for it. `None` for no holidays.
    pub fn holiday_country(&self, countries: &Countries) -> Option<String> {
        match self.holidays.as_deref() {
            Some("") => None,
            Some(code) => Some(code.to_lowercase()),
            None => locale_region().filter(|code| countries.holidays(code).is_some()),
        }
    }
}

/// The region of the locale, as a lowercase country code: `LC_ALL`, `LC_TIME` or `LANG`, the
/// first that is set.
fn locale_region() -> Option<String> {
    let locale = ["LC_ALL", "LC_TIME", "LANG"]
        .iter()
        .find_map(|name| env::var(name).ok().filter(|value| !value.is_empty()))?;
    let region = locale.split(['.', '@']).next()?.split_once('_')?.1;
    (region.len() == 2).then(|| region.to_lowercase())
}

/// The clock's folder in the user layer: `~/.config/kubaos/desktop/clock`.
pub fn dir() -> Option<PathBuf> {
    kubaos_user_layer::config_home().map(|config| kubaos_user_layer::dir(&config, App::Clock))
}

/// The settings file.
pub const FILE: &str = "clock.toml";
/// The user's holiday rules, beside it.
pub const HOLIDAYS_FILE: &str = "holidays.toml";

/// The bundled holiday rules, with the user's file in `dir` over them. A broken user file is left
/// out, and what is wrong with it comes back for the caller to log.
pub fn countries(dir: Option<&Path>) -> (Countries, Option<RulesError>) {
    let mut countries = Countries::bundled();
    let Some(text) = dir.and_then(|dir| fs::read_to_string(dir.join(HOLIDAYS_FILE)).ok()) else {
        return (countries, None);
    };
    let problem = countries.add_user(&text).err();
    (countries, problem)
}

/// The settings file as a document to edit, empty when missing.
fn document(path: &Path) -> Result<DocumentMut, SettingsError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(SettingsError::read(err)),
    };
    text.parse().map_err(SettingsError::Edit)
}

/// Writes `doc` over the file at `path` through a temporary file and a rename, so the popup's
/// watch never reads half a file.
fn save(path: &Path, doc: &DocumentMut) -> Result<(), SettingsError> {
    let text = doc.to_string();
    Settings::parse(&text)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(SettingsError::write)?;
    }
    let tmp = path.with_extension("toml.new");
    fs::write(&tmp, text).map_err(SettingsError::write)?;
    fs::rename(&tmp, path).map_err(SettingsError::write)
}

/// A copy of the `zones` array of `doc`, empty if missing; it goes back with `doc["zones"] = …`,
/// which keeps the key where it was.
fn zones_array(doc: &DocumentMut) -> toml_edit::Array {
    doc.get("zones")
        .and_then(toml_edit::Item::as_array)
        .cloned()
        .unwrap_or_default()
}

/// Adds the zone `name` (an IANA name) to the extra clocks; one already there stays as it is.
///
/// # Errors
///
/// [`SettingsError::TooManyZones`] past [`MAX_ZONES`], else as reading and writing the file.
pub fn add_zone(path: &Path, name: &str) -> Result<(), SettingsError> {
    let mut doc = document(path)?;
    let mut zones = zones_array(&doc);
    if zones.iter().any(|zone| zone.as_str() == Some(name)) {
        return Ok(());
    }
    if zones.len() >= MAX_ZONES {
        return Err(SettingsError::TooManyZones);
    }
    zones.push(name);
    doc["zones"] = value(zones);
    save(path, &doc)
}

/// Takes the zone `name` out of the extra clocks.
///
/// # Errors
///
/// [`SettingsError::NoZone`] when it isn't there, else as reading and writing the file.
pub fn remove_zone(path: &Path, name: &str) -> Result<(), SettingsError> {
    let mut doc = document(path)?;
    let mut zones = zones_array(&doc);
    let before = zones.len();
    zones.retain(|zone| zone.as_str() != Some(name));
    if zones.len() == before {
        return Err(SettingsError::NoZone(name.to_owned()));
    }
    doc["zones"] = value(zones);
    save(path, &doc)
}

/// Adds the countdown `name` to `at`, replacing one of the same name.
///
/// # Errors
///
/// As reading and writing the file.
pub fn add_countdown(path: &Path, name: &str, at: &str) -> Result<(), SettingsError> {
    let mut doc = document(path)?;
    let mut countdowns = doc
        .get("countdowns")
        .and_then(toml_edit::Item::as_array_of_tables)
        .cloned()
        .unwrap_or_default();
    if let Some(found) = countdowns
        .iter_mut()
        .find(|table| table.get("name").and_then(toml_edit::Item::as_str) == Some(name))
    {
        found["at"] = value(at);
    } else {
        let mut table = toml_edit::Table::new();
        table["name"] = value(name);
        table["at"] = value(at);
        countdowns.push(table);
    }
    doc["countdowns"] = toml_edit::Item::ArrayOfTables(countdowns);
    save(path, &doc)
}

/// Takes the countdown `name` away, and out of the bar if it was there.
///
/// # Errors
///
/// [`SettingsError::NoCountdown`] when there is none of that name, else as reading and writing
/// the file.
pub fn remove_countdown(path: &Path, name: &str) -> Result<(), SettingsError> {
    let mut doc = document(path)?;
    let Some(countdowns) = doc
        .get_mut("countdowns")
        .and_then(toml_edit::Item::as_array_of_tables_mut)
    else {
        return Err(SettingsError::NoCountdown(name.to_owned()));
    };
    let before = countdowns.len();
    countdowns.retain(|table| table.get("name").and_then(toml_edit::Item::as_str) != Some(name));
    if countdowns.len() == before {
        return Err(SettingsError::NoCountdown(name.to_owned()));
    }
    if doc.get("bar_countdown").and_then(toml_edit::Item::as_str) == Some(name) {
        doc["bar_countdown"] = value("");
    }
    save(path, &doc)
}

/// Sets one of the plain settings by the name the command line and the menu use:
/// `holidays` (a country code, or `none`), `first-weekday`, `analog` (`on`, `off` or `toggle`),
/// `bar-countdown` (a countdown's name, or `none`), `time-format` (`24h`, `12h`, or `locale`, which
/// takes the line out) and `date-order` (`day-month`, `month-day` or `locale`).
///
/// # Errors
///
/// [`SettingsError::NoSetting`] for another name; [`SettingsError::Value`],
/// [`SettingsError::NoCountry`] or [`SettingsError::NoCountdown`] for a value it can't take;
/// else as reading and writing the file.
pub fn set(path: &Path, key: &str, setting: &str, countries: &Countries) -> Result<(), SettingsError> {
    let mut doc = document(path)?;
    let current = Settings::parse(&doc.to_string())?;
    let bad = |key: &'static str| SettingsError::Value {
        key,
        value: setting.to_owned(),
    };
    match key {
        "holidays" => {
            let code = setting.to_lowercase();
            let code = if matches!(code.as_str(), "none" | "off" | "") {
                String::new()
            } else {
                code
            };
            if !code.is_empty() && countries.holidays(&code).is_none() {
                let known: Vec<&str> = countries.list().map(|(code, _)| code).collect();
                return Err(SettingsError::NoCountry(setting.to_owned(), known.join(", ")));
            }
            doc["holidays"] = value(code);
        }
        "first-weekday" | "first_weekday" => {
            let weekday = words::weekday(&setting.to_lowercase()).ok_or_else(|| bad("first-weekday"))?;
            doc["first_weekday"] = value(words::weekday_name(weekday).to_lowercase());
        }
        "analog" => {
            let on = match setting {
                "on" | "true" | "yes" => true,
                "off" | "false" | "no" => false,
                "toggle" => !current.analog,
                _ => return Err(bad("analog")),
            };
            doc["analog"] = value(on);
        }
        "bar-countdown" | "bar_countdown" => {
            let name = if matches!(setting, "none" | "off" | "") {
                ""
            } else {
                setting
            };
            if !name.is_empty() && !current.countdowns.iter().any(|countdown| countdown.name == name) {
                return Err(SettingsError::NoCountdown(name.to_owned()));
            }
            doc["bar_countdown"] = value(name);
        }
        "time-format" | "time_format" => {
            if setting == "locale" {
                doc.remove("time_format");
            } else {
                let format = TimeFormat::parse(setting).ok_or_else(|| bad("time-format"))?;
                doc["time_format"] = value(format.name());
            }
        }
        "date-order" | "date_order" => {
            if setting == "locale" {
                doc.remove("date_order");
            } else {
                let order = DateOrder::parse(setting).ok_or_else(|| bad("date-order"))?;
                doc["date_order"] = value(order.name());
            }
        }
        _ => return Err(SettingsError::NoSetting(key.to_owned())),
    }
    save(path, &doc)
}

#[cfg(test)]
mod tests;
