//! The popup's protocol: `kubaos-clock serve` reads one JSON request a line on stdin and writes
//! one JSON reply a line on stdout, the request's `id` in it. It lives as long as the shell, so a
//! keystroke in the search box costs a line each way, not a process.
//!
//! ```text
//! {"id":1,"op":"query","text":"next friday"}   {"id":1,"answer":{"title":...}}  or "answer":null
//! {"id":2,"op":"state"}                        {"id":2,"settings":...,"clocks":...,"countdowns":...}
//! {"id":3,"op":"month","year":2026,"month":10} {"id":3,"month":{"title":...,"weeks":[...]}}
//! {"id":4,"op":"day","date":"2026-12-25"}      {"id":4,"day":{"title":...}}
//! {"id":5,"op":"duration","text":"25m"}        {"id":5,"ms":1500000}            or "ms":null
//! {"id":6,"op":"alarm","text":"7:30"}          {"id":6,"at":1761370200000,"label":"..."}
//! {"id":7,"op":"zone-add","name":"tokyo"}      the state, as for "state"
//!   also zone-remove {name}, countdown-add {name, when}, countdown-remove {name}, set {key, value}
//! ```
//!
//! A request that fails gets `{"id":n,"error":"..."}`. The settings file is read again whenever
//! it changed since the last request.
//!
//! # Owns
//!
//! - The requests and their replies, the settings and holidays the server holds, and the changes to
//!   the settings the popup and the command line make.
//!
//! # Not here
//!
//! - What the answers say: [`crate::query`] and [`crate::calendar`]. The file: [`crate::settings`].

use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use jiff::Timestamp;
use jiff::Zoned;
use jiff::civil::Date;
use serde_json::Value;
use serde_json::json;

use crate::calendar;
use crate::holidays::Countries;
use crate::holidays::Holidays;
use crate::parse;
use crate::query;
use crate::query::Answer;
use crate::query::Context;
use crate::settings;
use crate::settings::Settings;
use crate::settings::SettingsError;
use crate::words;
use crate::zones::Zones;

/// Why a change to the settings was refused.
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("changing the clock settings")]
    Settings(#[source] SettingsError),
    #[error("no time zone called {0:?}")]
    NoZone(String),
    #[error("{0:?} isn't a date or time to count down to")]
    NotAMoment(String),
    #[error("neither XDG_CONFIG_HOME nor HOME is set, so there is no settings file")]
    NoSettingsFile,
}

impl CommandError {
    fn settings(err: SettingsError) -> Self {
        CommandError::Settings(err)
    }
}

/// An error and each of its sources, joined by `: `, as the popup and the terminal show it.
pub fn chain(err: &dyn Error) -> String {
    let mut text = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        let _ = write!(text, ": {cause}");
        source = cause.source();
    }
    text
}

/// The engine with the user's settings, as `serve` and the command line use it.
#[derive(Debug)]
pub struct Server {
    /// The settings file, if the environment names a home for it.
    path: Option<PathBuf>,
    settings: Settings,
    /// What was wrong with the settings or the user's holiday file when last read.
    problem: Option<String>,
    /// When the settings file was last read changed.
    modified: Option<SystemTime>,
    countries: Countries,
    holidays: Holidays,
    zones: Zones,
}

impl Server {
    /// A server on the settings in `dir`, the user layer's clock folder.
    #[must_use]
    pub fn new(dir: Option<PathBuf>) -> Self {
        let mut server = Server {
            path: dir.map(|dir| dir.join(settings::FILE)),
            settings: Settings::default(),
            problem: None,
            modified: None,
            countries: Countries::bundled(),
            holidays: Holidays::none(),
            zones: Zones::new(),
        };
        server.reload();
        server
    }

    /// The holidays the settings pick.
    pub fn holidays(&self) -> &Holidays {
        &self.holidays
    }

    pub fn zones(&self) -> &Zones {
        &self.zones
    }

    /// Reads the settings and the user's holiday rules again; a broken file keeps the defaults,
    /// and says why in the state.
    fn reload(&mut self) {
        let dir = self.path.as_deref().and_then(Path::parent);
        let (countries, rules_problem) = settings::countries(dir);
        self.countries = countries;
        self.modified = self.path.as_deref().and_then(modified);
        let loaded = self
            .path
            .as_deref()
            .map_or_else(|| Ok(Settings::default()), Settings::load);
        let settings_problem = loaded.as_ref().err().map(|err| chain(err));
        self.settings = loaded.unwrap_or_default();
        self.problem = settings_problem.or_else(|| rules_problem.map(|err| chain(&err)));
        self.holidays = self
            .settings
            .holiday_country(&self.countries)
            .and_then(|code| self.countries.holidays(&code))
            .unwrap_or_default();
    }

    /// Reads the settings again if the file changed since they were read: the menu's answers ask
    /// before each query, so a country picked in the popup counts its holidays at once.
    pub fn refresh(&mut self) {
        if self.path.as_deref().and_then(modified) != self.modified {
            self.reload();
        }
    }

    /// Answers one request line, as of `now`.
    pub fn handle(&mut self, line: &str, now: &Zoned) -> String {
        self.refresh();
        let request: Value = match serde_json::from_str(line) {
            Ok(request) => request,
            Err(err) => return json!({ "id": Value::Null, "error": format!("not JSON: {err}") }).to_string(),
        };
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let text = |key: &str| request.get(key).and_then(Value::as_str).unwrap_or_default().to_owned();
        let mut reply = match request.get("op").and_then(Value::as_str).unwrap_or_default() {
            "state" => Ok(self.state(now)),
            "query" => Ok(json!({ "answer": self.query(&text("text"), now).map(|answer| answer_json(&answer)) })),
            "month" => Ok(self.month(&request, now)),
            "day" => Ok(self.day(&text("date"), now)),
            "duration" => Ok(json!({ "ms": parse::duration(&text("text")).map(|duration| duration.as_millis()) })),
            "alarm" => Ok(self.alarm(&text("text"), now)),
            "zone-add" => self
                .add_zone(&text("name"))
                .map(drop)
                .map_err(|err| chain(&err))
                .map(|()| self.state(now)),
            "zone-remove" => self
                .remove_zone(&text("name"))
                .map_err(|err| chain(&err))
                .map(|()| self.state(now)),
            "countdown-add" => self
                .add_countdown(&text("name"), &text("when"), now)
                .map(drop)
                .map_err(|err| chain(&err))
                .map(|()| self.state(now)),
            "countdown-remove" => self
                .remove_countdown(&text("name"))
                .map_err(|err| chain(&err))
                .map(|()| self.state(now)),
            "set" => self
                .set(&text("key"), &text("value"))
                .map_err(|err| chain(&err))
                .map(|()| self.state(now)),
            other => Err(format!("no op {other:?}")),
        }
        .unwrap_or_else(|error| json!({ "error": error }));
        reply["id"] = id;
        reply.to_string()
    }

    /// Answers `query` as of `now`.
    pub fn query(&self, query: &str, now: &Zoned) -> Option<Answer> {
        query::evaluate(query, &Context::new(now, &self.holidays, &self.zones))
    }

    fn state(&self, now: &Zoned) -> Value {
        let today = calendar::today(now);
        let clocks: Vec<Value> = calendar::clocks(&self.settings.zones, &self.zones, now)
            .into_iter()
            .map(|clock| json!({ "name": clock.name, "label": clock.label, "offset": clock.offset, "relative": clock.relative }))
            .collect();
        let countdowns: Vec<Value> = self
            .settings
            .countdowns
            .iter()
            .filter_map(|countdown| {
                let end = calendar::countdown_end(&countdown.at, now.time_zone())?;
                Some(json!({ "name": countdown.name, "at": countdown.at, "end": end.as_millisecond() }))
            })
            .collect();
        let countries: Vec<Value> = self
            .countries
            .list()
            .map(|(code, name)| json!({ "code": code, "name": name }))
            .collect();
        let first = self.settings.first_weekday;
        json!({
            "settings": {
                "zones": self.settings.zones,
                "holidays": self.settings.holiday_country(&self.countries).unwrap_or_default(),
                "firstWeekday": first.to_monday_one_offset(),
                "firstWeekdayName": words::weekday_name(first),
                "analog": self.settings.analog,
                "barCountdown": self.settings.bar_countdown,
            },
            "country": self.holidays.country(),
            "here": { "label": Zones::of(now.time_zone()).label, "offset": now.offset().seconds() },
            "today": { "date": today.date.to_string(), "week": today.week, "dayOfYear": today.day_of_year },
            "clocks": clocks,
            "countdowns": countdowns,
            "countries": countries,
            "problem": self.problem,
        })
    }

    fn month(&self, request: &Value, now: &Zoned) -> Value {
        let number = |key: &str| request.get(key).and_then(Value::as_i64);
        let (year, month) = match (number("year"), number("month")) {
            (Some(year), Some(month)) => (year as i16, month as i8),
            _ => (now.date().year(), now.date().month()),
        };
        let Some(month) = calendar::month(year, month, self.settings.first_weekday, now.date(), &self.holidays) else {
            return json!({ "error": format!("no month {month} in {year}") });
        };
        let weeks: Vec<Value> = month
            .weeks
            .iter()
            .map(|week| {
                let days: Vec<Value> = week
                    .days
                    .iter()
                    .map(|day| {
                        json!({
                            "date": day.date.to_string(),
                            "day": day.date.day(),
                            "inMonth": day.in_month,
                            "today": day.today,
                            "weekend": day.weekend,
                            "holiday": day.holiday,
                        })
                    })
                    .collect();
                json!({ "number": week.number, "days": days })
            })
            .collect();
        json!({ "month": {
            "year": month.year,
            "month": month.month,
            "title": month.title,
            "weekdays": month.weekdays,
            "weeks": weeks,
        } })
    }

    fn day(&self, date: &str, now: &Zoned) -> Value {
        let Ok(date) = date.parse::<Date>() else {
            return json!({ "error": format!("{date:?} isn't a date") });
        };
        let zones: Vec<_> = self
            .settings
            .zones
            .iter()
            .filter_map(|name| self.zones.get(name))
            .collect();
        let details = calendar::day(date, now, &self.holidays, &zones);
        let zones: Vec<Value> = details
            .zones
            .iter()
            .map(|zone| json!({ "label": zone.label, "time": zone.time }))
            .collect();
        json!({ "day": {
            "date": date.to_string(),
            "title": details.title,
            "detail": details.detail,
            "holiday": details.holiday,
            "zones": zones,
        } })
    }

    fn alarm(&self, text: &str, now: &Zoned) -> Value {
        match parse::alarm(text, now, &self.zones) {
            Some(at) => {
                let days = query::days_between(now.date(), at.date()).unwrap_or_default();
                let label = format!("{} {}", at.strftime("%H:%M"), words::relative_days(days));
                json!({ "at": at.timestamp().as_millisecond(), "label": label })
            }
            None => json!({ "at": Value::Null }),
        }
    }

    fn path(&self) -> Result<&Path, CommandError> {
        self.path.as_deref().ok_or(CommandError::NoSettingsFile)
    }

    /// Adds the zone typed as `typed` to the extra clocks; its IANA name comes back.
    ///
    /// # Errors
    ///
    /// [`CommandError::NoZone`] when no zone has that name, else as [`settings::add_zone`].
    pub fn add_zone(&mut self, typed: &str) -> Result<String, CommandError> {
        let result = self
            .zones
            .get(typed)
            .ok_or_else(|| CommandError::NoZone(typed.to_owned()))
            .and_then(|zone| {
                settings::add_zone(self.path()?, &zone.name).map_err(CommandError::settings)?;
                Ok(zone.name)
            });
        self.after(result)
    }

    /// Takes the zone `name` out, by its IANA name or a name that finds it.
    ///
    /// # Errors
    ///
    /// As [`settings::remove_zone`].
    pub fn remove_zone(&mut self, name: &str) -> Result<(), CommandError> {
        let name = if self.settings.zones.iter().any(|zone| zone == name) {
            name.to_owned()
        } else {
            self.zones.get(name).map_or_else(|| name.to_owned(), |zone| zone.name)
        };
        let result = self
            .path()
            .and_then(|path| settings::remove_zone(path, &name).map_err(CommandError::settings));
        self.after(result)
    }

    /// Adds a countdown named `name` to what `when` says (`20 dec`, `friday 17:00`), as of
    /// `now`; what it was stored as comes back.
    ///
    /// # Errors
    ///
    /// [`CommandError::NotAMoment`] when `when` isn't a date or a moment, else as
    /// [`settings::add_countdown`].
    pub fn add_countdown(&mut self, name: &str, when: &str, now: &Zoned) -> Result<String, CommandError> {
        let result = self
            .moment(when, now)
            .ok_or_else(|| CommandError::NotAMoment(when.to_owned()))
            .and_then(|at| {
                let name = if name.trim().is_empty() {
                    when.trim()
                } else {
                    name.trim()
                };
                settings::add_countdown(self.path()?, name, &at).map_err(CommandError::settings)?;
                Ok(at)
            });
        self.after(result)
    }

    /// The date (`2026-12-19`) or instant (`2026-12-19T17:00:00+01:00`) a countdown's `when`
    /// comes to.
    fn moment(&self, when: &str, now: &Zoned) -> Option<String> {
        let answer = self.query(when, now)?;
        let moment = answer.iso.parse::<Date>().is_ok() || answer.iso.parse::<Timestamp>().is_ok();
        moment.then_some(answer.iso)
    }

    /// Takes the countdown `name` away.
    ///
    /// # Errors
    ///
    /// As [`settings::remove_countdown`].
    pub fn remove_countdown(&mut self, name: &str) -> Result<(), CommandError> {
        let result = self
            .path()
            .and_then(|path| settings::remove_countdown(path, name).map_err(CommandError::settings));
        self.after(result)
    }

    /// Sets one plain setting; see [`settings::set`].
    ///
    /// # Errors
    ///
    /// As [`settings::set`].
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), CommandError> {
        let result = self
            .path()
            .and_then(|path| settings::set(path, key, value, &self.countries).map_err(CommandError::settings));
        self.after(result)
    }

    /// Reads the settings again after a change.
    fn after<T>(&mut self, result: Result<T, CommandError>) -> Result<T, CommandError> {
        self.reload();
        result
    }
}

/// An answer as the popup reads it.
pub fn answer_json(answer: &Answer) -> Value {
    json!({
        "title": answer.title,
        "detail": answer.detail,
        "iso": answer.iso,
        "date": answer.date.map(|date| date.to_string()),
    })
}

fn modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).and_then(|meta| meta.modified()).ok()
}

/// Serves `input` until it ends, one reply line on `output` for each request line.
///
/// # Errors
///
/// When reading a request or writing a reply fails.
pub fn run(server: &mut Server, input: impl BufRead, mut output: impl Write) -> io::Result<()> {
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = server.handle(&line, &Zoned::now());
        writeln!(output, "{reply}")?;
        output.flush()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
