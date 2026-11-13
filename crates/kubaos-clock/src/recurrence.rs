//! When something repeats: "every day at 9", "mondays 8am", "1st of the month", "every 2 hours".
//! A [`Recurrence`] is read from what a person types, says when it next comes after a moment, and
//! says itself back in words.
//!
//! ```text
//! every day at 9 · daily · 9am             Every day at 9:00
//! mondays 8am · every monday · weekly      Every Monday at 8:00 (a week alone is a Monday)
//! 1st of the month · every month on the 15th · monthly
//!                                          Every month on the 15th at 0:00
//! every 2 hours · every 30 minutes · hourly
//!                                          Every 2 hours, on the clock from midnight
//! ```
//!
//! Times are on the wall clock, in the zone of the moment asked about: across a change to summer
//! time a time that doesn't exist that day comes an hour later, and one that comes twice counts
//! once, the first time. A day of the month past the month's end is its last day: the 31st is the
//! 30th in April and the 28th or 29th in February.
//!
//! # Owns
//!
//! - The phrases it reads, the next time after a moment, and the words it says itself back in.
//!
//! # Not here
//!
//! - Times of day, spans and names of days: `crate::parse` and `crate::words`. What repeats (a
//!   counter's auto reset): `kubaos-counters`.

use std::fmt;
use std::str::FromStr;

use jiff::ToSpan;
use jiff::Zoned;
use jiff::civil::Date;
use jiff::civil::Time;
use jiff::civil::Weekday;

use crate::parse;
use crate::settings::TimeFormat;
use crate::words;

/// Phrases it reads, for a person who typed one it can't.
pub const EXAMPLES: [&str; 4] = ["every day at 9", "mondays 8am", "every 2 hours", "1st of the month"];

/// The minutes in a day, the longest an interval may be.
const DAY_MINUTES: u16 = 24 * 60;

/// When something repeats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Recurrence {
    /// Every day at a time.
    Daily(Time),
    /// Every week, on a weekday at a time.
    Weekly(Weekday, Time),
    /// Every month, on a day (1 to 31) at a time.
    Monthly(i8, Time),
    /// Every so many minutes (1 to a day's), on the clock from each midnight: every 2 hours is
    /// 0:00, 2:00, 4:00…, and every 5 hours 0:00, 5:00… 20:00, then 0:00 again.
    Every(u16),
}

/// What was typed isn't a time [`Recurrence`] reads.
#[derive(Debug, thiserror::Error)]
#[error(
    "`{0}` isn't a time it can repeat at; try \"every day at 9\", \"mondays 8am\", \"every 2 hours\" or \"1st of the month\""
)]
pub struct RecurrenceError(String);

impl RecurrenceError {
    fn unread(text: &str) -> Self {
        RecurrenceError(text.trim().to_owned())
    }
}

/// The parts of a phrase as they are read, before they make a [`Recurrence`].
#[derive(Debug, Default)]
struct Parts {
    time: Option<Time>,
    weekday: Option<Weekday>,
    day: Option<i8>,
    minutes: Option<u16>,
    day_word: bool,
    week_word: bool,
    month_word: bool,
}

impl Recurrence {
    /// Reads `text`: a day (`every day`, `daily`), a weekday (`mondays`, `every monday`, `weekly`),
    /// a day of the month (`1st of the month`, `every month on the 15th`, `monthly`), each with a
    /// time or at midnight; a time alone, every day; or an interval up to a day (`every 2 hours`,
    /// `every 30 minutes`, `hourly`).
    ///
    /// # Errors
    ///
    /// When `text` isn't one of those, or says two of them at once.
    pub fn parse(text: &str) -> Result<Recurrence, RecurrenceError> {
        let split = words::split(text);
        let w: Vec<&str> = split.iter().map(String::as_str).collect();
        read(&w)
            .and_then(Parts::into_recurrence)
            .ok_or_else(|| RecurrenceError::unread(text))
    }

    /// The first time it comes strictly after `after`, in `after`'s zone; `None` only past the
    /// end of jiff's calendar, in the year 9999.
    pub fn next_after(&self, after: &Zoned) -> Option<Zoned> {
        let tz = after.time_zone().clone();
        let at = |date: Date, time: Time| date.to_datetime(time).to_zoned(tz.clone()).ok();
        let today = after.date();
        let tomorrow = today.tomorrow().ok();
        match *self {
            Recurrence::Daily(time) => [Some(today), tomorrow]
                .into_iter()
                .filter_map(|date| at(date?, time))
                .find(|z| z > after),
            Recurrence::Weekly(weekday, time) => (0..=7)
                .filter_map(|days| today.checked_add(days.days()).ok())
                .filter(|date| date.weekday() == weekday)
                .filter_map(|date| at(date, time))
                .find(|z| z > after),
            Recurrence::Monthly(day, time) => (0..=1)
                .filter_map(|months| in_month(today, months, day))
                .filter_map(|date| at(date, time))
                .find(|z| z > after),
            Recurrence::Every(minutes) => (0..DAY_MINUTES)
                .step_by(usize::from(minutes))
                .filter_map(|start| at(today, minutes_time(start)))
                .chain(tomorrow.and_then(|date| at(date, Time::midnight())))
                .find(|z| z > after),
        }
    }

    /// Itself in words to start a sentence, the time as `format` writes it (24-hour without one):
    /// `Every day at 9:00`, `Every Monday at 8:00`, `Every month on the 1st at 0:00`, `Every 2
    /// hours`.
    pub fn words(&self, format: Option<TimeFormat>) -> String {
        match *self {
            Recurrence::Daily(time) => format!("Every day at {}", clock(time, format)),
            Recurrence::Weekly(weekday, time) => {
                format!("Every {} at {}", words::weekday_name(weekday), clock(time, format))
            }
            Recurrence::Monthly(day, time) => {
                format!("Every month on the {} at {}", ordinal(day), clock(time, format))
            }
            Recurrence::Every(60) => "Every hour".to_owned(),
            Recurrence::Every(1) => "Every minute".to_owned(),
            Recurrence::Every(minutes) if minutes % 60 == 0 => format!("Every {} hours", minutes / 60),
            Recurrence::Every(minutes) => format!("Every {minutes} minutes"),
        }
    }

    /// Itself in words mid-sentence: `every day at 9:00`, as in "Resets every day at 9:00".
    pub fn phrase(&self, format: Option<TimeFormat>) -> String {
        let words = self.words(format);
        let mut chars = words.chars();
        chars
            .next()
            .map(|first| first.to_lowercase().chain(chars).collect())
            .unwrap_or_default()
    }
}

/// The 24-hour words, which [`Recurrence::parse`] reads back to the same rule: what a file keeps.
impl fmt::Display for Recurrence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.phrase(None))
    }
}

impl FromStr for Recurrence {
    type Err = RecurrenceError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Recurrence::parse(text)
    }
}

/// Reads the words of a phrase into its parts, in any order; `None` at a word it doesn't know or a
/// part said twice.
fn read(w: &[&str]) -> Option<Parts> {
    let mut parts = Parts::default();
    let mut at = 0;
    while at < w.len() {
        let rest = &w[at..];
        let after_the = at > 0 && w[at - 1] == "the";
        at += if let Some((time, used)) = time_at(rest) {
            set(&mut parts.time, time)?;
            used
        } else if let Some((minutes, used)) = interval(rest) {
            set(&mut parts.minutes, minutes)?;
            used
        } else {
            word(rest[0], after_the, &mut parts)?;
            1
        };
    }
    Some(parts)
}

/// One word that is neither a time nor an interval, into `parts`; `None` if it isn't one it reads.
fn word(word: &str, after_the: bool, parts: &mut Parts) -> Option<()> {
    match word {
        "every" | "each" | "on" | "the" | "of" | "at" | "a" | "per" => {}
        "day" | "days" | "daily" | "everyday" => parts.day_word = true,
        "week" | "weeks" | "weekly" => parts.week_word = true,
        "month" | "months" | "monthly" => parts.month_word = true,
        "hour" | "hourly" => set(&mut parts.minutes, 60)?,
        "minute" => set(&mut parts.minutes, 1)?,
        _ => {
            if let Some(weekday) = weekday(word) {
                set(&mut parts.weekday, weekday)?;
            } else {
                set(&mut parts.day, day_of_month(word, after_the)?)?;
            }
        }
    }
    Some(())
}

/// Puts `value` in `slot`; `None` if it holds one already.
fn set<T>(slot: &mut Option<T>, value: T) -> Option<()> {
    if slot.is_some() {
        return None;
    }
    *slot = Some(value);
    Some(())
}

impl Parts {
    fn into_recurrence(self) -> Option<Recurrence> {
        let time = self.time.unwrap_or(Time::midnight());
        if let Some(minutes) = self.minutes {
            let alone = self.time.is_none()
                && self.weekday.is_none()
                && self.day.is_none()
                && !self.day_word
                && !self.week_word
                && !self.month_word;
            return alone.then_some(Recurrence::Every(minutes));
        }
        match (self.weekday, self.day) {
            (Some(weekday), None) if !self.day_word && !self.month_word => Some(Recurrence::Weekly(weekday, time)),
            (None, Some(day)) if !self.day_word && !self.week_word => Some(Recurrence::Monthly(day, time)),
            (None, None) if self.month_word && !self.day_word && !self.week_word => Some(Recurrence::Monthly(1, time)),
            (None, None) if self.week_word && !self.day_word => Some(Recurrence::Weekly(Weekday::Monday, time)),
            (None, None) if self.day_word || self.time.is_some() => Some(Recurrence::Daily(time)),
            _ => None,
        }
    }
}

/// A time of day at the front of `w`: one the clock's reader takes (`9am`, `15:30`, `noon`), after
/// `at` or not, or `at` and an hour alone (`at 9`).
fn time_at(w: &[&str]) -> Option<(Time, usize)> {
    let at = usize::from(w.first() == Some(&"at"));
    if let Some((time, used)) = parse::time(&w[at..]) {
        return Some((time, at + used));
    }
    if at == 1 {
        let hour: i8 = w.get(1)?.parse().ok()?;
        return Some((Time::new(hour, 0, 0, 0).ok()?, 2));
    }
    None
}

/// An interval at the front of `w`, in minutes: a span of hours and minutes (`2 hours`, `30m`,
/// `1h30m`) up to a day.
fn interval(w: &[&str]) -> Option<(u16, usize)> {
    let (units, used) = parse::span(w)?;
    if units.years != 0 || units.months != 0 || units.weeks != 0 || units.days != 0 || units.seconds != 0 {
        return None;
    }
    let minutes = units.hours.checked_mul(60)?.checked_add(units.minutes)?;
    let minutes = u16::try_from(minutes).ok()?;
    (1..=DAY_MINUTES).contains(&minutes).then_some((minutes, used))
}

/// A weekday, its plural too: `monday`, `mondays`, `mon`.
fn weekday(word: &str) -> Option<Weekday> {
    words::weekday(word).or_else(|| words::weekday(word.strip_suffix('s')?))
}

/// A day of the month: `1st`, `15th`, or a number alone after `the`.
fn day_of_month(word: &str, after_the: bool) -> Option<i8> {
    let digits = word.trim_end_matches(|c: char| c.is_ascii_alphabetic());
    let suffix = &word[digits.len()..];
    let ordinal = matches!(suffix, "st" | "nd" | "rd" | "th");
    if !(ordinal || suffix.is_empty() && after_the) || digits.is_empty() || digits.len() > 2 {
        return None;
    }
    let day: i8 = digits.parse().ok()?;
    (1..=31).contains(&day).then_some(day)
}

/// The `day` of the month `months` after `date`'s, or that month's last day if it is shorter.
fn in_month(date: Date, months: i64, day: i8) -> Option<Date> {
    let first = date.first_of_month().checked_add(months.months()).ok()?;
    first.with().day(day.min(first.days_in_month())).build().ok()
}

/// The time `minutes` after midnight, for minutes within a day.
fn minutes_time(minutes: u16) -> Time {
    Time::constant((minutes / 60) as i8, (minutes % 60) as i8, 0, 0)
}

/// `9:00`, or `9:00 AM` in the 12-hour format.
fn clock(time: Time, format: Option<TimeFormat>) -> String {
    match format {
        Some(TimeFormat::TwelveHour) => time.strftime("%-I:%M %p").to_string(),
        _ => time.strftime("%-H:%M").to_string(),
    }
}

/// `1st`, `2nd`, `3rd`, `11th`, `22nd`.
fn ordinal(day: i8) -> String {
    let suffix = match (day % 10, day % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{day}{suffix}")
}

#[cfg(test)]
mod tests;
