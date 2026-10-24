//! The pieces a date query is made of, each read from the front of a list of words: a date, a
//! time of day, a moment (a date, a time, or both), and a span (`3 weeks 2 days`, `1h30m`). Each
//! reader says how many words it took, so [`crate::query`] can put them together.
//!
//! # Owns
//!
//! - Every date, time and span the engine reads, and which year a date without one falls in.
//! - The durations of timers and the times of alarms.
//!
//! # Not here
//!
//! - Whole queries and their answers: [`crate::query`]. Zone names: [`crate::zones`].

use jiff::SignedDuration;
use jiff::Span;
use jiff::ToSpan;
use jiff::Zoned;
use jiff::civil::Date;
use jiff::civil::Time;

use crate::holidays;
use crate::holidays::Holidays;
use crate::words;
use crate::zones::Zones;

/// What the readers read against.
#[derive(Debug)]
pub(crate) struct Ctx<'a> {
    pub now: &'a Zoned,
    pub today: Date,
    pub holidays: &'a Holidays,
    pub zones: &'a Zones,
}

/// Which way a date without a year, or a bare weekday, leans from the day it is read from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Lean {
    /// That day or the next one like it.
    Next,
    /// That day or the last one like it.
    Last,
}

/// A point a query names: a whole day, or an instant.
#[derive(Clone, Debug)]
pub(crate) enum Moment {
    Day(Date),
    At(Zoned),
}

impl Moment {
    /// The moment as an instant in `ctx`'s zone; a day is its start.
    pub fn zoned(&self, ctx: &Ctx) -> Option<Zoned> {
        match self {
            Moment::Day(date) => date.to_zoned(ctx.now.time_zone().clone()).ok(),
            Moment::At(zoned) => Some(zoned.clone()),
        }
    }
}

/// A read moment, and whether it was a time of day alone, which a query on its own doesn't
/// answer (`15:00` says nothing the clock doesn't).
#[derive(Clone, Debug)]
pub(crate) struct ReadMoment {
    pub moment: Moment,
    pub used: usize,
    pub time_only: bool,
}

/// Days named the same in every country, as month and day, or days after Easter.
enum Named {
    Fixed(i8, i8),
    Easter(i32),
}

const NAMED: &[(&str, Named)] = &[
    ("christmas", Named::Fixed(12, 25)),
    ("christmas day", Named::Fixed(12, 25)),
    ("xmas", Named::Fixed(12, 25)),
    ("christmas eve", Named::Fixed(12, 24)),
    ("xmas eve", Named::Fixed(12, 24)),
    ("boxing day", Named::Fixed(12, 26)),
    ("new year", Named::Fixed(1, 1)),
    ("new years", Named::Fixed(1, 1)),
    ("new years day", Named::Fixed(1, 1)),
    ("new year day", Named::Fixed(1, 1)),
    ("new years eve", Named::Fixed(12, 31)),
    ("new year eve", Named::Fixed(12, 31)),
    ("nye", Named::Fixed(12, 31)),
    ("halloween", Named::Fixed(10, 31)),
    ("valentines", Named::Fixed(2, 14)),
    ("valentines day", Named::Fixed(2, 14)),
    ("st patricks day", Named::Fixed(3, 17)),
    ("easter", Named::Easter(0)),
    ("easter sunday", Named::Easter(0)),
    ("easter monday", Named::Easter(1)),
    ("good friday", Named::Easter(-2)),
];

/// A date at the front of `w`, read from `anchor`, and how many words it took. Of the ways the
/// words can be read, the longest wins: `christmas eve` over `christmas`.
pub(crate) fn date(w: &[&str], anchor: Date, lean: Lean, ctx: &Ctx) -> Option<(Date, usize)> {
    if w.is_empty() {
        return None;
    }
    [
        relative_day(w, anchor),
        iso(w),
        dotted(w),
        day_month(w, anchor, lean),
        month_day(w, anchor, lean),
        weekday(w, anchor, lean),
        in_span(w, anchor),
        ago(w, anchor),
        named(w, anchor, lean, ctx),
        end_of(w, anchor),
    ]
    .into_iter()
    .flatten()
    .max_by_key(|&(_, used)| used)
}

fn relative_day(w: &[&str], anchor: Date) -> Option<(Date, usize)> {
    match w {
        ["day", "after", "tomorrow", ..] => Some((anchor.checked_add(2.days()).ok()?, 3)),
        ["day", "before", "yesterday", ..] => Some((anchor.checked_sub(2.days()).ok()?, 3)),
        ["today", ..] => Some((anchor, 1)),
        ["tomorrow" | "tmrw" | "tmr", ..] => Some((anchor.tomorrow().ok()?, 1)),
        ["yesterday", ..] => Some((anchor.yesterday().ok()?, 1)),
        _ => None,
    }
}

fn iso(w: &[&str]) -> Option<(Date, usize)> {
    let first = w.first()?;
    if !words::is_iso_date(first) {
        return None;
    }
    Some((first.parse().ok()?, 1))
}

/// `24.12.2026`, as most of Europe writes a date.
fn dotted(w: &[&str]) -> Option<(Date, usize)> {
    let mut parts = w.first()?.split('.');
    let (day, month, year) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() || year.len() != 4 {
        return None;
    }
    Some((
        Date::new(year.parse().ok()?, month.parse().ok()?, day.parse().ok()?).ok()?,
        1,
    ))
}

/// `24 dec`, `24th of december 2027`.
fn day_month(w: &[&str], anchor: Date, lean: Lean) -> Option<(Date, usize)> {
    let day = day_number(w.first()?)?;
    let mut used = 1;
    if w.get(used) == Some(&"of") {
        used += 1;
    }
    let month = words::month(w.get(used)?)?;
    used += 1;
    with_year(w, used, anchor, lean, month, day)
}

/// `dec 24`, `december 24th 2027`.
fn month_day(w: &[&str], anchor: Date, lean: Lean) -> Option<(Date, usize)> {
    let month = words::month(w.first()?)?;
    let day = day_number(w.get(1)?)?;
    with_year(w, 2, anchor, lean, month, day)
}

/// The date of `month` and `day` with the year at `w[used]` if there is one, or else the year
/// `lean` gives from `anchor`.
fn with_year(w: &[&str], used: usize, anchor: Date, lean: Lean, month: i8, day: i8) -> Option<(Date, usize)> {
    if let Some(year) = w.get(used).and_then(|word| year_number(word)) {
        return Some((Date::new(year, month, day).ok()?, used + 1));
    }
    Some((leaning(anchor, month, day, lean)?, used))
}

/// `month`/`day` in `anchor`'s year, or the year after or before it as `lean` says, so the date
/// isn't on the wrong side of `anchor`.
fn leaning(anchor: Date, month: i8, day: i8, lean: Lean) -> Option<Date> {
    let date = Date::new(anchor.year(), month, day).ok()?;
    match lean {
        Lean::Next if date < anchor => Date::new(anchor.year() + 1, month, day).ok(),
        Lean::Last if date > anchor => Date::new(anchor.year() - 1, month, day).ok(),
        _ => Some(date),
    }
}

/// A day of the month: `24`, `24th`, `1st`.
fn day_number(word: &str) -> Option<i8> {
    let digits = word.trim_end_matches(|c: char| c.is_ascii_alphabetic());
    let suffix = &word[digits.len()..];
    if !matches!(suffix, "" | "st" | "nd" | "rd" | "th") || digits.is_empty() || digits.len() > 2 {
        return None;
    }
    let day: i8 = digits.parse().ok()?;
    (1..=31).contains(&day).then_some(day)
}

fn year_number(word: &str) -> Option<i16> {
    (word.len() == 4).then(|| word.parse().ok()).flatten()
}

/// `friday`, `next friday`, `this friday`, `last friday`. A bare weekday or `this` is that day if
/// `anchor` is one; `next` is always after it, and `last` always before.
fn weekday(w: &[&str], anchor: Date, lean: Lean) -> Option<(Date, usize)> {
    let (which, at) = match *w.first()? {
        "next" | "coming" => (Some(Lean::Next), 1),
        "last" | "previous" | "past" => (Some(Lean::Last), 1),
        "this" => (None, 1),
        _ => (None, 0),
    };
    let weekday = words::weekday(w.get(at)?)?;
    let date = match which {
        None if anchor.weekday() == weekday => anchor,
        Some(Lean::Next) => anchor.nth_weekday(1, weekday).ok()?,
        None if lean == Lean::Next => anchor.nth_weekday(1, weekday).ok()?,
        Some(Lean::Last) | None => anchor.nth_weekday(-1, weekday).ok()?,
    };
    Some((date, at + 1))
}

/// `in 3 weeks`, `in a month`: whole days only.
fn in_span(w: &[&str], anchor: Date) -> Option<(Date, usize)> {
    if w.first() != Some(&"in") {
        return None;
    }
    let (units, used) = span(&w[1..])?;
    if units.has_time() {
        return None;
    }
    Some((anchor.checked_add(units.span()).ok()?, used + 1))
}

/// `3 days ago`, `2 weeks from now`.
fn ago(w: &[&str], anchor: Date) -> Option<(Date, usize)> {
    let (units, used) = span(w)?;
    if units.has_time() {
        return None;
    }
    match &w[used..] {
        ["ago", ..] => Some((anchor.checked_sub(units.span()).ok()?, used + 1)),
        ["from", "now" | "today", ..] => Some((anchor.checked_add(units.span()).ok()?, used + 2)),
        _ => None,
    }
}

/// A day named the same everywhere (`christmas`, `easter`), or one of the country's holidays by
/// the start of its name (`corpus christi`, `independence day`). Four words at most.
fn named(w: &[&str], anchor: Date, lean: Lean, ctx: &Ctx) -> Option<(Date, usize)> {
    (1..=w.len().min(4)).rev().find_map(|n| {
        let name = words::plain(&w[..n].join(" "));
        if let Some((_, day)) = NAMED.iter().find(|(known, _)| *known == name) {
            return Some((named_date(day, anchor, lean)?, n));
        }
        // Short words (`may`, `day`, `all`) would start too many names.
        if name.len() < 4 {
            return None;
        }
        let found = match lean {
            Lean::Next => ctx.holidays.next_named(&name, anchor),
            Lean::Last => ctx.holidays.last_named(&name, anchor),
        };
        Some((found?, n))
    })
}

fn named_date(day: &Named, anchor: Date, lean: Lean) -> Option<Date> {
    match *day {
        Named::Fixed(month, day) => leaning(anchor, month, day, lean),
        Named::Easter(offset) => {
            let in_year = |year: i16| holidays::easter(year).checked_add(offset.days()).ok();
            let date = in_year(anchor.year())?;
            match lean {
                Lean::Next if date < anchor => in_year(anchor.year() + 1),
                Lean::Last if date > anchor => in_year(anchor.year() - 1),
                _ => Some(date),
            }
        }
    }
}

/// `end of month`, `end of the year`.
fn end_of(w: &[&str], anchor: Date) -> Option<(Date, usize)> {
    let (what, used) = match w {
        ["end", "of", "the", what, ..] => (*what, 4),
        ["end", "of", what, ..] => (*what, 3),
        _ => return None,
    };
    let date = match what {
        "month" => anchor.last_of_month(),
        "year" => Date::new(anchor.year(), 12, 31).ok()?,
        _ => return None,
    };
    Some((date, used))
}

/// A time of day at the front of `w`: `15:00`, `9am`, `9:30 pm`, `noon`, `midnight`. A number
/// alone isn't one.
pub(crate) fn time(w: &[&str]) -> Option<(Time, usize)> {
    let first = *w.first()?;
    match first {
        "noon" | "midday" => return Some((Time::constant(12, 0, 0, 0), 1)),
        "midnight" => return Some((Time::constant(0, 0, 0, 0), 1)),
        _ => {}
    }
    let (clock, suffix) = match first.find(|c: char| c.is_ascii_alphabetic()) {
        Some(at) => (&first[..at], Some(&first[at..])),
        None => (first, None),
    };
    let (meridiem, used) = match suffix {
        Some(suffix) => (Some(suffix), 1),
        None => match w.get(1) {
            Some(&next) if matches!(next, "am" | "pm" | "a.m." | "p.m.") => (Some(next), 2),
            _ => (None, 1),
        },
    };
    let (hour, minute) = match clock.split_once([':', '.']) {
        Some((hour, minute)) if minute.len() == 2 => (hour.parse::<i8>().ok()?, minute.parse::<i8>().ok()?),
        None if meridiem.is_some() => (clock.parse::<i8>().ok()?, 0),
        _ => return None,
    };
    let hour = match meridiem.map(|m| m.replace('.', "")) {
        None if (0..24).contains(&hour) => hour,
        Some(m) if m == "am" && (1..=12).contains(&hour) => hour % 12,
        Some(m) if m == "pm" && (1..=12).contains(&hour) => hour % 12 + 12,
        _ => return None,
    };
    Some((Time::new(hour, minute, 0, 0).ok()?, used))
}

/// A moment at the front of `w`: `now`, a date, a date and a time (`friday 17:00`,
/// `tomorrow at 9am`), a time and a date (`9am tomorrow`), or a time alone, today, or the next
/// or last time it comes round as `lean` says.
pub(crate) fn moment(w: &[&str], lean: Lean, ctx: &Ctx) -> Option<ReadMoment> {
    if w.first() == Some(&"now") {
        return Some(ReadMoment {
            moment: Moment::At(ctx.now.clone()),
            used: 1,
            time_only: false,
        });
    }
    let tz = ctx.now.time_zone().clone();
    if let Some((date, used)) = date(w, ctx.today, lean, ctx) {
        let at = usize::from(w.get(used) == Some(&"at"));
        if let Some((time, more)) = time(&w[used + at..]) {
            let zoned = date.to_datetime(time).to_zoned(tz).ok()?;
            return Some(ReadMoment {
                moment: Moment::At(zoned),
                used: used + at + more,
                time_only: false,
            });
        }
        return Some(ReadMoment {
            moment: Moment::Day(date),
            used,
            time_only: false,
        });
    }
    let at = usize::from(w.first() == Some(&"at"));
    let (time, used) = time(&w[at..])?;
    let used = used + at;
    if let Some((date, more)) = date(&w[used..], ctx.today, lean, ctx) {
        let zoned = date.to_datetime(time).to_zoned(tz).ok()?;
        return Some(ReadMoment {
            moment: Moment::At(zoned),
            used: used + more,
            time_only: false,
        });
    }
    let mut zoned = ctx.today.to_datetime(time).to_zoned(tz).ok()?;
    if lean == Lean::Next && zoned < *ctx.now {
        zoned = zoned.checked_add(1.day()).ok()?;
    } else if lean == Lean::Last && zoned > *ctx.now {
        zoned = zoned.checked_sub(1.day()).ok()?;
    }
    Some(ReadMoment {
        moment: Moment::At(zoned),
        used,
        time_only: true,
    })
}

/// A span's parts, as read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Units {
    pub years: i64,
    pub months: i64,
    pub weeks: i64,
    pub days: i64,
    pub hours: i64,
    pub minutes: i64,
    pub seconds: i64,
}

impl Units {
    /// Whether any part is shorter than a day.
    pub fn has_time(&self) -> bool {
        self.hours != 0 || self.minutes != 0 || self.seconds != 0
    }

    pub fn span(&self) -> Span {
        Span::new()
            .years(self.years)
            .months(self.months)
            .weeks(self.weeks)
            .days(self.days)
            .hours(self.hours)
            .minutes(self.minutes)
            .seconds(self.seconds)
    }

    /// Adds `n` of the unit named `unit`; false if `unit` isn't one.
    fn add(&mut self, n: i64, unit: &str) -> bool {
        let part = match unit {
            "y" | "yr" | "yrs" | "year" | "years" => &mut self.years,
            "mo" | "mon" | "mos" | "month" | "months" => &mut self.months,
            "w" | "wk" | "wks" | "week" | "weeks" => &mut self.weeks,
            "fortnight" | "fortnights" => {
                self.weeks += 2 * n;
                return true;
            }
            "d" | "day" | "days" => &mut self.days,
            "h" | "hr" | "hrs" | "hour" | "hours" => &mut self.hours,
            "m" | "min" | "mins" | "minute" | "minutes" => &mut self.minutes,
            "s" | "sec" | "secs" | "second" | "seconds" => &mut self.seconds,
            _ => return false,
        };
        *part += n;
        true
    }
}

/// A span at the front of `w`: one or more of `3 weeks`, `90d`, `1h30m`, `a month`, joined by
/// nothing or `and`.
pub(crate) fn span(w: &[&str]) -> Option<(Units, usize)> {
    let mut units = Units::default();
    let mut used = 0;
    let mut terms = 0;
    loop {
        let rest = &w[used..];
        let start = usize::from(terms > 0 && rest.first() == Some(&"and"));
        let Some(took) = term(&rest[start..], &mut units) else {
            break;
        };
        used += start + took;
        terms += 1;
    }
    (terms > 0).then_some((units, used))
}

/// One part of a span, added to `units`; how many words it took.
fn term(w: &[&str], units: &mut Units) -> Option<usize> {
    let first = *w.first()?;
    if matches!(first, "a" | "an" | "one") {
        let mut scratch = *units;
        return scratch.add(1, w.get(1)?).then(|| {
            *units = scratch;
            2
        });
    }
    if first.starts_with(|c: char| c.is_ascii_digit()) {
        if let Ok(n) = first.parse::<i64>() {
            let mut scratch = *units;
            return scratch.add(n, w.get(1)?).then(|| {
                *units = scratch;
                2
            });
        }
        return compact(first, units).then_some(1);
    }
    None
}

/// `90d`, `1h30m`, `2w3d`: numbers each followed by a unit, in one word.
fn compact(word: &str, units: &mut Units) -> bool {
    let mut scratch = *units;
    let mut rest = word;
    while !rest.is_empty() {
        let digits = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
        let letters = rest[digits..]
            .find(|c: char| !c.is_ascii_alphabetic())
            .map_or(rest.len(), |at| at + digits);
        let (Ok(n), unit) = (rest[..digits].parse::<i64>(), &rest[digits..letters]) else {
            return false;
        };
        if unit.is_empty() || !scratch.add(n, unit) {
            return false;
        }
        rest = &rest[letters..];
    }
    *units = scratch;
    true
}

/// How long a timer runs: a span of days or less (`5m`, `1h30m`, `90 seconds`), a number of
/// minutes alone (`25`), or `m:ss` or `h:mm:ss`.
pub fn duration(text: &str) -> Option<SignedDuration> {
    let text = text.trim().to_lowercase();
    if let Ok(minutes) = text.parse::<i64>() {
        return (minutes > 0).then(|| SignedDuration::from_mins(minutes));
    }
    if let Some(seconds) = clock_duration(&text) {
        return (seconds > 0).then(|| SignedDuration::from_secs(seconds));
    }
    let split = words::split(&text);
    let w: Vec<&str> = split.iter().map(String::as_str).collect();
    let (units, used) = span(&w)?;
    if used != w.len() || units.years != 0 || units.months != 0 {
        return None;
    }
    let seconds = ((units.weeks * 7 + units.days) * 24 + units.hours) * 3600 + units.minutes * 60 + units.seconds;
    (seconds > 0).then(|| SignedDuration::from_secs(seconds))
}

/// `1:30` as 90 seconds, `1:00:00` as an hour.
fn clock_duration(text: &str) -> Option<i64> {
    let parts: Vec<&str> = text.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 || parts[1..].iter().any(|part| part.len() != 2) {
        return None;
    }
    let numbers: Vec<i64> = parts.iter().map(|part| part.parse().ok()).collect::<Option<_>>()?;
    Some(numbers.iter().fold(0, |total, n| total * 60 + n))
}

/// When an alarm set for `text` rings: the next time a time of day comes round (`7:30`, `7am`),
/// or a moment with a date (`friday 17:00`). Never in the past.
pub fn alarm(text: &str, now: &Zoned, zones: &Zones) -> Option<Zoned> {
    let holidays = Holidays::none();
    let ctx = Ctx {
        now,
        today: now.date(),
        holidays: &holidays,
        zones,
    };
    let split = words::split(text);
    let w: Vec<&str> = split.iter().map(String::as_str).collect();
    let read = moment(&w, Lean::Next, &ctx)?;
    if read.used != w.len() {
        return None;
    }
    let at = read.moment.zoned(&ctx)?;
    (at > *now).then_some(at)
}

#[cfg(test)]
mod tests;
