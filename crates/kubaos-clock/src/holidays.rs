//! Public holidays, offline: rules in `data/holidays.toml`, embedded, that give the days of any
//! year. A country of the user's own, in the same shape, replaces the bundled one.
//!
//! # Owns
//!
//! - Reading the rules, Easter, and the days a rule comes to in a year, observed days included.
//! - Finding a holiday by name.
//!
//! # Not here
//!
//! - Which country the user picked and where their file is: [`crate::settings`].

use std::collections::BTreeMap;

use jiff::ToSpan;
use jiff::civil::Date;
use jiff::civil::Weekday;
use serde::Deserialize;

use crate::words;

/// The bundled rules.
const BUNDLED: &str = include_str!("../data/holidays.toml");

/// Every country the engine has rules for, by lowercase ISO 3166 code.
#[derive(Clone, Debug)]
pub struct Countries {
    countries: BTreeMap<String, Country>,
}

/// One country's rules.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Country {
    name: String,
    days: Vec<Rule>,
}

/// One holiday's rule, as the data file writes it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Rule {
    name: String,
    /// `MM-DD`.
    date: Option<String>,
    /// Days after Easter Sunday.
    easter: Option<i32>,
    month: Option<i8>,
    weekday: Option<String>,
    nth: Option<i8>,
    from: Option<i16>,
    until: Option<i16>,
    #[serde(default)]
    observed: Observed,
}

/// What happens to a holiday that falls on a weekend.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Observed {
    /// Nothing.
    #[default]
    No,
    /// A Saturday is kept on the Friday too, a Sunday on the Monday.
    Nearest,
    /// The next weekday that isn't a holiday already is one too.
    Substitute,
    /// A Sunday moves to the Saturday before.
    Saturday,
}

/// A user's holidays file that can't be read as rules.
#[derive(Debug, thiserror::Error)]
#[error("reading the holiday rules")]
pub struct RulesError(#[source] RulesCause);

/// What is wrong with a rules file.
#[derive(Debug, thiserror::Error)]
pub enum RulesCause {
    #[error("the file isn't holiday rules")]
    Parse(#[source] toml::de::Error),
    #[error("`{0}` needs one of `date`, `easter`, or `month` with `weekday` and `nth`")]
    Shape(String),
}

impl RulesError {
    fn parse(err: toml::de::Error) -> Self {
        RulesError(RulesCause::Parse(err))
    }

    fn shape(name: &str) -> Self {
        RulesError(RulesCause::Shape(name.to_owned()))
    }
}

impl Countries {
    /// The rules shipped with KubaOS.
    ///
    /// # Panics
    ///
    /// When the embedded file is broken, which its tests catch.
    #[must_use]
    pub fn bundled() -> Self {
        let mut countries = Countries {
            countries: BTreeMap::new(),
        };
        countries.add_user(BUNDLED).expect("the bundled holiday rules parse");
        countries
    }

    /// Adds the countries of a rules file, each replacing a bundled one of the same code whole.
    ///
    /// # Errors
    ///
    /// When `text` isn't TOML in the shape of `data/holidays.toml`, or a day has no rule.
    pub fn add_user(&mut self, text: &str) -> Result<(), RulesError> {
        let read: BTreeMap<String, Country> = toml::from_str(text).map_err(RulesError::parse)?;
        for country in read.values() {
            if let Some(rule) = country.days.iter().find(|rule| !rule.has_shape()) {
                return Err(RulesError::shape(&rule.name));
            }
        }
        for (code, country) in read {
            self.countries.insert(code.to_lowercase(), country);
        }
        Ok(())
    }

    /// The countries, as code and name, by code.
    pub fn list(&self) -> impl Iterator<Item = (&str, &str)> {
        self.countries
            .iter()
            .map(|(code, country)| (code.as_str(), country.name.as_str()))
    }

    /// The holidays of the country `code`, in any case.
    #[must_use]
    pub fn holidays(&self, code: &str) -> Option<Holidays> {
        let country = self.countries.get(&code.to_lowercase())?;
        Some(Holidays {
            country: Some(country.clone()),
        })
    }
}

/// A day off, and what it is called.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Holiday {
    pub date: Date,
    pub name: String,
}

/// The holidays of the country the user picked, or none.
#[derive(Clone, Debug, Default)]
pub struct Holidays {
    country: Option<Country>,
}

impl Holidays {
    /// No holidays at all.
    #[must_use]
    pub fn none() -> Self {
        Holidays::default()
    }

    /// The country's name, if there is one.
    pub fn country(&self) -> Option<&str> {
        self.country.as_ref().map(|country| country.name.as_str())
    }

    /// The holidays of `year`, by date.
    pub fn in_year(&self, year: i16) -> Vec<Holiday> {
        let Some(country) = &self.country else {
            return Vec::new();
        };
        let mut days: Vec<Holiday> = country
            .days
            .iter()
            .filter(|rule| rule.from.is_none_or(|from| year >= from) && rule.until.is_none_or(|until| year <= until))
            .filter_map(|rule| {
                Some(Holiday {
                    date: rule.date_in(year)?,
                    name: rule.name.clone(),
                })
            })
            .collect();
        days.sort_by_key(|day| day.date);
        let mut observed = Vec::new();
        for day in &days {
            let Some(rule) = country.days.iter().find(|rule| rule.name == day.name) else {
                continue;
            };
            let taken = |date: Date| {
                days.iter()
                    .chain(&observed)
                    .any(|holiday: &Holiday| holiday.date == date)
            };
            if let Some(extra) = rule.observed_day(day.date, taken) {
                observed.push(extra);
            }
        }
        days.extend(observed);
        days.sort_by_key(|day| day.date);
        days
    }

    /// The holiday on `date`, if any; with two on one day, their names joined.
    pub fn on(&self, date: Date) -> Option<String> {
        let names: Vec<String> = self
            .in_year(date.year())
            .into_iter()
            .filter(|day| day.date == date)
            .map(|day| day.name)
            .collect();
        (!names.is_empty()).then(|| names.join(", "))
    }

    /// The first holiday whose name starts with `name` (case, apostrophes and full stops aside)
    /// on or after `from`, within the next two years.
    pub fn next_named(&self, name: &str, from: Date) -> Option<Date> {
        let wanted = words::plain(name);
        if wanted.is_empty() {
            return None;
        }
        (from.year()..=from.year() + 2).find_map(|year| {
            self.in_year(year)
                .into_iter()
                .find(|day| day.date >= from && words::plain(&day.name).starts_with(&wanted))
                .map(|day| day.date)
        })
    }

    /// The last holiday named like `name` on or before `from`, within the two years before.
    pub fn last_named(&self, name: &str, from: Date) -> Option<Date> {
        let wanted = words::plain(name);
        if wanted.is_empty() {
            return None;
        }
        (from.year() - 2..=from.year()).rev().find_map(|year| {
            self.in_year(year)
                .into_iter()
                .rev()
                .find(|day| day.date <= from && words::plain(&day.name).starts_with(&wanted))
                .map(|day| day.date)
        })
    }
}

impl Rule {
    fn has_shape(&self) -> bool {
        let fixed = self.date.as_deref().is_some_and(|date| month_day(date).is_some());
        let weekday =
            self.month.is_some() && self.nth.is_some() && self.weekday.as_deref().and_then(words::weekday).is_some();
        fixed || self.easter.is_some() || weekday
    }

    /// The day the rule falls on in `year`, before any observed day.
    fn date_in(&self, year: i16) -> Option<Date> {
        let date = if let Some((month, day)) = self.date.as_deref().and_then(month_day) {
            Date::new(year, month, day).ok()?
        } else if let Some(days) = self.easter {
            easter(year).checked_add(days.days()).ok()?
        } else {
            let weekday = words::weekday(self.weekday.as_deref()?)?;
            Date::new(year, self.month?, 1)
                .ok()?
                .nth_weekday_of_month(self.nth?, weekday)
                .ok()?
        };
        Some(match (self.observed, date.weekday()) {
            (Observed::Saturday, Weekday::Sunday) => date.yesterday().ok()?,
            _ => date,
        })
    }

    /// The extra day off for a holiday on `date` that falls on a weekend; `taken` says whether a
    /// day is a holiday already.
    fn observed_day(&self, date: Date, taken: impl Fn(Date) -> bool) -> Option<Holiday> {
        let (day, what) = match (self.observed, date.weekday()) {
            (Observed::Nearest, Weekday::Saturday) => (date.yesterday().ok()?, "observed"),
            (Observed::Nearest, Weekday::Sunday) => (date.tomorrow().ok()?, "observed"),
            (Observed::Substitute, Weekday::Saturday | Weekday::Sunday) => {
                let mut day = date.tomorrow().ok()?;
                while is_weekend(day) || taken(day) {
                    day = day.tomorrow().ok()?;
                }
                (day, "substitute day")
            }
            _ => return None,
        };
        Some(Holiday {
            date: day,
            name: format!("{} ({what})", self.name),
        })
    }
}

fn is_weekend(date: Date) -> bool {
    matches!(date.weekday(), Weekday::Saturday | Weekday::Sunday)
}

/// `MM-DD` as month and day.
fn month_day(text: &str) -> Option<(i8, i8)> {
    let (month, day) = text.split_once('-')?;
    Some((month.parse().ok()?, day.parse().ok()?))
}

/// Western Easter Sunday of `year`, by the anonymous Gregorian algorithm (Meeus/Jones/Butcher).
///
/// # Panics
///
/// Never for the years jiff takes: the algorithm always gives a day in March or April.
#[expect(
    clippy::many_single_char_names,
    reason = "the letters are the algorithm's own, so the code reads against its published form"
)]
pub fn easter(year: i16) -> Date {
    let y = i32::from(year);
    let a = y % 19;
    let b = y / 100;
    let c = y % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = (h + l - 7 * m + 114) % 31 + 1;
    Date::new(year, month as i8, day as i8).expect("Easter is a day in March or April")
}

#[cfg(test)]
mod tests;
