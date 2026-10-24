//! Date search and maths: what a query typed in the clock popup or the Command menu comes to.
//!
//! ```text
//! next friday · 24 dec · in 3 weeks · 2027-03-01 · christmas      a date
//! days until christmas · days since 2020-01-01 · until friday 17:00  a count
//! age 1993-05-12                                                     an age
//! 12 mar to 4 jul                                                    a range, with working days
//! today + 90d · now + 36h · friday 17:00 - 3h · 3 weeks before x     a date or time moved
//! 15:00 tokyo · 9am NY in Warsaw · time in tokyo                     a time in other zones
//! ```
//!
//! # Owns
//!
//! - The forms above, tried in a fixed order, and the [`Answer`] each comes to.
//!
//! # Not here
//!
//! - Reading dates, times and spans: `crate::parse`. Zone names: [`crate::zones`]. Holidays:
//!   [`crate::holidays`].

use jiff::Span;
use jiff::Unit;
use jiff::Zoned;
use jiff::ZonedDifference;
use jiff::civil::Date;
use jiff::civil::Time;
use jiff::civil::Weekday;

use crate::holidays::Holidays;
use crate::parse;
use crate::parse::Ctx;
use crate::parse::Lean;
use crate::parse::Moment;
use crate::words;
use crate::zones::Zone;
use crate::zones::Zones;

/// What the engine reads a query against.
#[derive(Debug)]
pub struct Context<'a> {
    now: &'a Zoned,
    holidays: &'a Holidays,
    zones: &'a Zones,
}

impl<'a> Context<'a> {
    /// Queries read as of `now`, in its zone, with `holidays` left out of working days.
    #[must_use]
    pub fn new(now: &'a Zoned, holidays: &'a Holidays, zones: &'a Zones) -> Self {
        Context { now, holidays, zones }
    }

    fn ctx(&self) -> Ctx<'a> {
        Ctx {
            now: self.now,
            today: self.now.date(),
            holidays: self.holidays,
            zones: self.zones,
        }
    }
}

/// What a query comes to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    /// The answer, shown large and copied by Enter: `62 days`, `Friday 30 October 2026`.
    pub title: String,
    /// The line under it: `in 6 days, a Friday · week 44`.
    pub detail: String,
    /// The answer in ISO 8601, copied by Ctrl+Enter: `2026-10-30`, `P62D`.
    pub iso: String,
    /// The day the calendar shows for it.
    pub date: Option<Date>,
}

/// Answers `query`, or `None` when it isn't a date query the engine reads.
pub fn evaluate(query: &str, context: &Context) -> Option<Answer> {
    let query = query.trim().trim_end_matches('?');
    let split = words::split(query);
    let mut w: Vec<&str> = split.iter().map(String::as_str).collect();
    if let ["when", "is" | "was", ..] = w.as_slice() {
        w.drain(..2);
    }
    if w.is_empty() {
        return None;
    }
    let ctx = context.ctx();
    zones(&w, &ctx)
        .or_else(|| age(&w, &ctx))
        .or_else(|| count(&w, &ctx))
        .or_else(|| range(&w, &ctx))
        .or_else(|| arithmetic(&w, &ctx))
        .or_else(|| single(&w, &ctx))
}

/// `15:00 tokyo`, `9am NY in Warsaw`, `17:00 in new york`, `time in tokyo`, `tokyo time`.
fn zones(w: &[&str], ctx: &Ctx) -> Option<Answer> {
    let here = Zones::of(ctx.now.time_zone());
    if let ["time" | "now", "in", rest @ ..] = w {
        return Some(now_in(&whole_zone(rest, ctx)?, ctx));
    }
    if let [rest @ .., "time" | "now"] = w {
        return Some(now_in(&whole_zone(rest, ctx)?, ctx));
    }
    let (date, time, used) = clock(w, ctx)?;
    let rest = &w[used..];
    let (from, to) = if let ["in" | "to" | "into", target @ ..] = rest {
        (here, whole_zone(target, ctx)?)
    } else {
        let (from, took) = ctx.zones.find(rest)?;
        match &rest[took..] {
            [] => (from, here),
            ["in" | "to" | "into", target @ ..] => (from, whole_zone(target, ctx)?),
            _ => return None,
        }
    };
    let date = date.unwrap_or_else(|| ctx.now.with_time_zone(from.tz.clone()).date());
    let source = date.to_datetime(time).to_zoned(from.tz.clone()).ok()?;
    let target = source.with_time_zone(to.tz.clone());
    Some(Answer {
        title: format!("{} in {}", hh_mm(&target), to.label),
        detail: format!(
            "{} in {} · {} · {}",
            hh_mm(&source),
            from.label,
            day_month(target.date()),
            compared(&to, &target, &source)
        ),
        iso: iso_zoned(&target),
        date: Some(target.date()),
    })
}

/// The zone `w` names whole.
fn whole_zone(w: &[&str], ctx: &Ctx) -> Option<Zone> {
    let (zone, used) = ctx.zones.find(w)?;
    (used == w.len()).then_some(zone)
}

/// A time with an optional date before or after it, and how many words they took.
fn clock(w: &[&str], ctx: &Ctx) -> Option<(Option<Date>, Time, usize)> {
    if let Some((date, used)) = parse::date(w, ctx.today, Lean::Next, ctx) {
        let at = usize::from(w.get(used) == Some(&"at"));
        let (time, more) = parse::time(&w[used + at..])?;
        return Some((Some(date), time, used + at + more));
    }
    let (time, used) = parse::time(w)?;
    // A zone name can read as a date (`christmas` is a zone too), so a date after the time is
    // taken only when no zone follows at once.
    if ctx.zones.find(&w[used..]).is_none()
        && let Some((date, more)) = parse::date(&w[used..], ctx.today, Lean::Next, ctx)
    {
        return Some((Some(date), time, used + more));
    }
    Some((None, time, used))
}

fn now_in(zone: &Zone, ctx: &Ctx) -> Answer {
    let there = ctx.now.with_time_zone(zone.tz.clone());
    Answer {
        title: format!("{} in {}", hh_mm(&there), zone.label),
        detail: format!("{} · {}", day_month(there.date()), compared(zone, &there, ctx.now)),
        iso: iso_zoned(&there),
        date: Some(there.date()),
    }
}

/// `Tokyo is 7 h ahead`: how `zone`'s clock at `there` compares with `here`'s, the same instant.
fn compared(zone: &Zone, there: &Zoned, here: &Zoned) -> String {
    let diff = there.offset().seconds() - here.offset().seconds();
    if diff == 0 {
        return format!("{} has the same time", zone.label);
    }
    let hours = words::offset_label(diff.abs());
    let hours = hours.trim_start_matches('+');
    format!("{} is {hours} {}", zone.label, if diff > 0 { "ahead" } else { "behind" })
}

/// `age 1993-05-12`, `how old is 12 may 1993`.
fn age(w: &[&str], ctx: &Ctx) -> Option<Answer> {
    let (["age", "of", rest @ ..] | ["how", "old", "is", rest @ ..] | ["age" | "born", rest @ ..]) = w else {
        return None;
    };
    let (birth, used) = parse::date(rest, ctx.today, Lean::Last, ctx)?;
    if used != rest.len() || birth > ctx.today {
        return None;
    }
    let lived = birth.until((Unit::Year, ctx.today)).ok()?;
    let (years, months, days) = (i64::from(lived.get_years()), i64::from(lived.get_months()), i64::from(lived.get_days()));
    let mut parts = vec![words::count(years, "year")];
    if months > 0 {
        parts.push(words::count(months, "month"));
    }
    if days > 0 {
        parts.push(words::count(days, "day"));
    }
    let birthday = next_birthday(birth, ctx.today)?;
    let to_birthday = days_between(ctx.today, birthday)?;
    let next = if to_birthday == 0 {
        "birthday today".to_owned()
    } else {
        format!("next birthday {}", words::relative_days(to_birthday))
    };
    Some(Answer {
        title: words::count(years, "year"),
        detail: format!("{} · born on {} · {next}", parts.join(" "), words::a_weekday(birth.weekday())),
        iso: format!("P{years}Y{months}M{days}D"),
        date: Some(birth),
    })
}

/// The first birthday on or after `today`; one born on 29 February has it on the 28th in other
/// years.
fn next_birthday(birth: Date, today: Date) -> Option<Date> {
    let in_year = |year: i16| {
        Date::new(year, birth.month(), birth.day())
            .or_else(|_| Date::new(year, birth.month(), birth.day() - 1))
            .ok()
    };
    let this_year = in_year(today.year())?;
    if this_year >= today { Some(this_year) } else { in_year(today.year() + 1) }
}

/// How a count is shown: in days, in weeks and days, or in the units that fit.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CountIn {
    Days,
    Weeks,
    Fitting,
}

/// `days until christmas`, `days since 2020-01-01`, `weeks until x`, `how long until friday
/// 17:00`, `until christmas`.
fn count(w: &[&str], ctx: &Ctx) -> Option<Answer> {
    let (unit, rest) = match w {
        ["how", "many", "weeks", rest @ ..] | ["weeks", rest @ ..] => (CountIn::Weeks, rest),
        ["how", "long", rest @ ..] | ["time", rest @ ..] => (CountIn::Fitting, rest),
        ["how", "many", "days", rest @ ..] | ["days", rest @ ..] | rest => (CountIn::Days, rest),
    };
    let (lean, rest) = match rest {
        ["until" | "till" | "til" | "to" | "left" | "before", rest @ ..] => (Lean::Next, rest),
        ["since" | "from", rest @ ..] => (Lean::Last, rest),
        _ => return None,
    };
    let read = parse::moment(rest, lean, ctx)?;
    if read.used != rest.len() {
        return None;
    }
    let toward = if lean == Lean::Next { "until" } else { "since" };
    match read.moment {
        Moment::Day(date) => {
            let days = days_between(ctx.today, date)?.abs();
            let title = match unit {
                CountIn::Weeks => weeks_and_days(days),
                CountIn::Days | CountIn::Fitting => words::count(days, "day"),
            };
            let other = if unit == CountIn::Weeks { words::count(days, "day") } else { weeks_and_days(days) };
            let detail = if days >= 7 || unit == CountIn::Weeks {
                format!("{toward} {} · {other}", words::long_date(date))
            } else {
                format!("{toward} {}", words::long_date(date))
            };
            Some(Answer { title, detail, iso: format!("P{days}D"), date: Some(date) })
        }
        Moment::At(at) => {
            let (from, to) = if lean == Lean::Next { (ctx.now, &at) } else { (&at, ctx.now) };
            let span = between(from, to)?;
            Some(Answer {
                title: fitting(&span),
                detail: format!("{toward} {}, {}", words::long_date(at.date()), hh_mm(&at)),
                iso: iso_span(&span),
                date: Some(at.date()),
            })
        }
    }
}

/// `12 mar to 4 jul`, `from x to y`, `between x and y`, `x - y`: days, weeks, months and working
/// days, both ends counted for working days.
fn range(w: &[&str], ctx: &Ctx) -> Option<Answer> {
    let (["from" | "between", w @ ..] | w) = w;
    let (start, used) = parse::date(w, ctx.today, Lean::Next, ctx)?;
    let ["to" | "until" | "till" | "-" | "\u{2013}" | "and" | "through" | "thru", rest @ ..] = &w[used..] else {
        return None;
    };
    let (end, used) = parse::date(rest, start, Lean::Next, ctx)?;
    if used != rest.len() || end < start {
        return None;
    }
    let days = days_between(start, end)?;
    let mut parts = vec![format!("{} to {}", short_date(start), short_date(end))];
    if days >= 7 {
        parts.push(weeks_and_days(days));
    }
    let months = start.until((Unit::Month, end)).ok()?;
    if months.get_months() > 0 {
        parts.push(months_and_days(i64::from(months.get_months()), i64::from(months.get_days())));
    }
    parts.push(words::count(working_days(start, end, ctx.holidays), "working day"));
    Some(Answer {
        title: words::count(days, "day"),
        detail: parts.join(" · "),
        iso: format!("{start}/{end}"),
        date: Some(start),
    })
}

/// The weekdays from `start` to `end`, both counted, that aren't holidays.
fn working_days(start: Date, end: Date, holidays: &Holidays) -> i64 {
    let off: Vec<Date> = (start.year()..=end.year())
        .flat_map(|year| holidays.in_year(year))
        .map(|holiday| holiday.date)
        .collect();
    let mut working = 0;
    for day in start.series(Span::new().days(1)).take_while(|day| *day <= end) {
        if !matches!(day.weekday(), Weekday::Saturday | Weekday::Sunday) && !off.contains(&day) {
            working += 1;
        }
    }
    working
}

/// `today + 90d`, `now + 36h`, `friday 17:00 - 3h`, `3 weeks before christmas`.
fn arithmetic(w: &[&str], ctx: &Ctx) -> Option<Answer> {
    if let Some((units, used)) = parse::span(w)
        && let Some(&word) = w.get(used)
        && matches!(word, "before" | "after" | "from")
        // `2 weeks from now` is a day, which the date reader takes.
        && !matches!(&w[used + 1..], ["now" | "today"])
    {
        let rest = &w[used + 1..];
        let read = parse::moment(rest, Lean::Next, ctx)?;
        if read.used != rest.len() {
            return None;
        }
        let sign = if word == "before" { -1 } else { 1 };
        return moved(&read.moment, &[(sign, units)], ctx);
    }
    let read = parse::moment(w, Lean::Next, ctx)?;
    let mut rest = &w[read.used..];
    let mut steps = Vec::new();
    while let [op, more @ ..] = rest {
        let sign = match *op {
            "+" | "plus" | "add" => 1,
            "-" | "minus" | "\u{2212}" => -1,
            _ => return None,
        };
        let (units, used) = parse::span(more)?;
        steps.push((sign, units));
        rest = &more[used..];
    }
    if steps.is_empty() {
        return None;
    }
    moved(&read.moment, &steps, ctx)
}

/// `base` moved by each signed span in turn.
fn moved(base: &Moment, steps: &[(i64, parse::Units)], ctx: &Ctx) -> Option<Answer> {
    let span = |sign: i64, units: &parse::Units| if sign < 0 { units.span().negate() } else { units.span() };
    if let Moment::Day(date) = base
        && steps.iter().all(|(_, units)| !units.has_time())
    {
        let mut date = *date;
        for (sign, units) in steps {
            date = date.checked_add(span(*sign, units)).ok()?;
        }
        return Some(date_answer(date, ctx));
    }
    let mut at = base.zoned(ctx)?;
    for (sign, units) in steps {
        at = at.checked_add(span(*sign, units)).ok()?;
    }
    Some(zoned_answer(&at, ctx))
}

/// A date or a moment alone: `next friday`, `24 dec`, `friday 17:00`.
fn single(w: &[&str], ctx: &Ctx) -> Option<Answer> {
    let read = parse::moment(w, Lean::Next, ctx)?;
    if read.used != w.len() || read.time_only {
        return None;
    }
    match read.moment {
        Moment::Day(date) => Some(date_answer(date, ctx)),
        Moment::At(at) => Some(zoned_answer(&at, ctx)),
    }
}

/// `Friday 30 October 2026`, then `in 6 days, a Friday · week 44`, and the holiday on the day.
pub(crate) fn date_answer(date: Date, ctx: &Ctx) -> Answer {
    let days = days_between(ctx.today, date).unwrap_or_default();
    let mut detail = format!(
        "{}, {} · week {}",
        words::relative_days(days),
        words::a_weekday(date.weekday()),
        date.iso_week_date().week()
    );
    if let Some(holiday) = ctx.holidays.on(date) {
        detail.push_str(" · ");
        detail.push_str(&holiday);
    }
    Answer {
        title: words::long_date(date),
        detail,
        iso: date.to_string(),
        date: Some(date),
    }
}

/// `Sunday 25 October 2026, 21:00`, then how far it is from now.
fn zoned_answer(at: &Zoned, ctx: &Ctx) -> Answer {
    let detail = match between(ctx.now, at) {
        Some(span) if span.is_zero() => "now".to_owned(),
        Some(span) if span.is_negative() => format!("{} ago", fitting(&span.abs())),
        Some(span) => format!("in {}", fitting(&span)),
        None => String::new(),
    };
    Answer {
        title: format!("{}, {}", words::long_date(at.date()), hh_mm(at)),
        detail,
        iso: iso_zoned(at),
        date: Some(at.date()),
    }
}

/// The days, hours and minutes from `from` to `to`, days as the calendar counts them, so a day
/// across a DST change is still one day.
fn between(from: &Zoned, to: &Zoned) -> Option<Span> {
    from.until(ZonedDifference::new(to).largest(Unit::Day).smallest(Unit::Minute)).ok()
}

/// The whole days from `from` to `to`.
pub(crate) fn days_between(from: Date, to: Date) -> Option<i64> {
    Some(i64::from(from.until((Unit::Day, to)).ok()?.get_days()))
}

/// `8 weeks 6 days`, `2 weeks`, `5 days`.
fn weeks_and_days(days: i64) -> String {
    let (weeks, days) = (days / 7, days % 7);
    match (weeks, days) {
        (0, days) => words::count(days, "day"),
        (weeks, 0) => words::count(weeks, "week"),
        (weeks, days) => format!("{} {}", words::count(weeks, "week"), words::count(days, "day")),
    }
}

fn months_and_days(months: i64, days: i64) -> String {
    if days == 0 {
        words::count(months, "month")
    } else {
        format!("{} {}", words::count(months, "month"), words::count(days, "day"))
    }
}

/// A span of days, hours and minutes in its two largest units: `6 days 7 hours`,
/// `8 hours 30 minutes`.
fn fitting(span: &Span) -> String {
    let parts = [
        (i64::from(span.get_days()), "day"),
        (i64::from(span.get_hours()), "hour"),
        (span.get_minutes(), "minute"),
    ];
    let shown: Vec<String> = parts
        .iter()
        .skip_while(|(n, _)| *n == 0)
        .take(2)
        .filter(|(n, _)| *n != 0)
        .map(|&(n, unit)| words::count(n, unit))
        .collect();
    if shown.is_empty() { "0 minutes".into() } else { shown.join(" ") }
}

fn iso_span(span: &Span) -> String {
    let (days, hours, minutes) = (span.get_days(), span.get_hours(), span.get_minutes());
    if hours == 0 && minutes == 0 { format!("P{days}D") } else { format!("P{days}DT{hours}H{minutes}M") }
}

fn hh_mm(at: &Zoned) -> String {
    at.strftime("%H:%M").to_string()
}

fn iso_zoned(at: &Zoned) -> String {
    at.strftime("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

/// `Saturday 24 October`.
fn day_month(date: Date) -> String {
    format!("{} {} {}", words::weekday_name(date.weekday()), date.day(), words::month_name(date.month()))
}

/// `12 March 2027`.
fn short_date(date: Date) -> String {
    format!("{} {} {}", date.day(), words::month_name(date.month()), date.year())
}

#[cfg(test)]
mod tests;
