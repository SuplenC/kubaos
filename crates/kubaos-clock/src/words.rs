//! The English the engine reads and writes: weekday and month names, how a query splits into
//! words, and how dates, spans and offsets are written.
//!
//! # Owns
//!
//! - Names in both directions, the query's words, and every phrase an answer is made of.
//!
//! # Not here
//!
//! - What the words mean together: `crate::parse` and [`crate::query`].

use jiff::civil::Date;
use jiff::civil::Weekday;

const WEEKDAYS: [(&str, Weekday); 7] = [
    ("monday", Weekday::Monday),
    ("tuesday", Weekday::Tuesday),
    ("wednesday", Weekday::Wednesday),
    ("thursday", Weekday::Thursday),
    ("friday", Weekday::Friday),
    ("saturday", Weekday::Saturday),
    ("sunday", Weekday::Sunday),
];

const MONTHS: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

/// A weekday by its name, or the first three letters of it or more (`fri`, `thurs`).
pub fn weekday(word: &str) -> Option<Weekday> {
    let word = word.trim_end_matches('.');
    if word.len() < 3 {
        return None;
    }
    WEEKDAYS.iter().find(|(name, _)| name.starts_with(word)).map(|&(_, weekday)| weekday)
}

/// A month, 1 to 12, by its name, or the first three letters of it or more (`dec`, `sept`).
pub fn month(word: &str) -> Option<i8> {
    let word = word.trim_end_matches('.');
    if word.len() < 3 {
        return None;
    }
    let found = MONTHS.iter().position(|name| name.starts_with(word))?;
    Some(found as i8 + 1)
}

/// The weekday's English name.
pub fn weekday_name(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Monday => "Monday",
        Weekday::Tuesday => "Tuesday",
        Weekday::Wednesday => "Wednesday",
        Weekday::Thursday => "Thursday",
        Weekday::Friday => "Friday",
        Weekday::Saturday => "Saturday",
        Weekday::Sunday => "Sunday",
    }
}

/// The month's English name, for 1 to 12.
pub fn month_name(month: i8) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        _ => "December",
    }
}

/// `Friday 25 December 2026`.
pub fn long_date(date: Date) -> String {
    format!(
        "{} {} {} {}",
        weekday_name(date.weekday()),
        date.day(),
        month_name(date.month()),
        date.year()
    )
}

/// `today`, `tomorrow`, `yesterday`, `in 61 days` or `3 days ago`, for a date `days` from today.
pub fn relative_days(days: i64) -> String {
    match days {
        0 => "today".into(),
        1 => "tomorrow".into(),
        -1 => "yesterday".into(),
        n if n > 0 => format!("in {}", count(n, "day")),
        n => format!("{} ago", count(-n, "day")),
    }
}

/// `1 day`, `3 days`.
pub fn count(n: i64, unit: &str) -> String {
    if n == 1 { format!("1 {unit}") } else { format!("{n} {unit}s") }
}

/// `a Thursday`.
pub fn a_weekday(weekday: Weekday) -> String {
    format!("a {}", weekday_name(weekday))
}

/// An offset from local time, as the world clocks show it: `same time`, `+7 h`, `−6 h`, `+5:30 h`.
/// The minus is U+2212, so it lines up with the plus.
pub fn offset_label(seconds: i32) -> String {
    if seconds == 0 {
        return "same time".into();
    }
    let sign = if seconds < 0 { '\u{2212}' } else { '+' };
    let minutes = seconds.unsigned_abs() / 60;
    let (hours, minutes) = (minutes / 60, minutes % 60);
    if minutes == 0 { format!("{sign}{hours} h") } else { format!("{sign}{hours}:{minutes:02} h") }
}

/// The words of a query, in lowercase, with `+` and `-` words of their own except inside an ISO
/// date (`2027-03-01`) or a word (`port-au-prince`), and commas dropped.
pub fn split(query: &str) -> Vec<String> {
    let lower = query.to_lowercase().replace(',', " ");
    let mut words = Vec::new();
    for word in lower.split_whitespace() {
        let mut rest = word;
        while !rest.is_empty() {
            if let Some(after) = rest.strip_prefix(['+', '-']) {
                words.push(rest[..1].to_owned());
                rest = after;
                continue;
            }
            if is_iso_date(rest) {
                words.push(rest.to_owned());
                break;
            }
            // A sign after a letter belongs to a name; after a digit or a unit, it is an operator.
            let at = rest.char_indices().skip(1).find(|&(at, c)| {
                (c == '+' || c == '-')
                    && !(c == '-' && rest[..at].ends_with(|p: char| p.is_alphabetic()) && rest[at + 1..].starts_with(char::is_alphabetic))
            });
            let Some((at, _)) = at else {
                words.push(rest.to_owned());
                break;
            };
            words.push(rest[..at].to_owned());
            rest = &rest[at..];
        }
    }
    words
}

/// Whether `word` looks like `2027-03-01`.
pub fn is_iso_date(word: &str) -> bool {
    let bytes = word.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes.iter().enumerate().all(|(at, b)| at == 4 || at == 7 || b.is_ascii_digit())
}

/// `name` in lowercase, with apostrophes and full stops gone and spaces single, for comparing
/// names as people type them: `St. Stephen's Day` and `st stephens day` are the same.
pub fn plain(name: &str) -> String {
    let kept: String = name
        .chars()
        .filter(|c| !matches!(c, '\'' | '\u{2019}' | '.'))
        .map(|c| if c == '_' || c == '-' { ' ' } else { c })
        .collect::<String>()
        .to_lowercase();
    kept.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests;
