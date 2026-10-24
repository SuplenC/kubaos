//! What the popup's calendar and clocks show: a month as six weeks with their numbers, one day's
//! details, today's place in the year, and the extra clocks.
//!
//! # Owns
//!
//! - The month grid (first weekday, week numbers, holidays, today), a day's details, the clocks'
//!   offsets, and when a countdown ends.
//!
//! # Not here
//!
//! - Turning these into the popup's JSON: [`crate::serve`].

use jiff::Timestamp;
use jiff::ToSpan;
use jiff::Zoned;
use jiff::civil::Date;
use jiff::civil::Weekday;
use jiff::tz::TimeZone;

use crate::holidays::Holidays;
use crate::query;
use crate::words;
use crate::zones::Zone;
use crate::zones::Zones;

/// Rows in a month: every month fits in six weeks, and a fixed count keeps the popup's height.
pub const WEEKS: usize = 6;

/// One day of the grid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Day {
    pub date: Date,
    /// In the month shown, not the weeks around it.
    pub in_month: bool,
    pub today: bool,
    pub weekend: bool,
    pub holiday: Option<String>,
}

/// One row of the grid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Week {
    /// The ISO week number of the row's Thursday, so a row always has the number ISO gives most of
    /// its days.
    pub number: i8,
    pub days: Vec<Day>,
}

/// A month as the calendar shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Month {
    pub year: i16,
    pub month: i8,
    /// `October 2026`.
    pub title: String,
    /// The weekday headers in order, `Mon` first by default.
    pub weekdays: Vec<String>,
    pub weeks: Vec<Week>,
}

/// The month `month` of `year`, starting its weeks on `first`.
pub fn month(year: i16, month: i8, first: Weekday, today: Date, holidays: &Holidays) -> Option<Month> {
    let first_of_month = Date::new(year, month, 1).ok()?;
    let back = i64::from(first_of_month.weekday().since(first));
    let start = first_of_month.checked_sub(back.days()).ok()?;
    let mut off = holidays.in_year(year);
    if start.year() != year {
        off.extend(holidays.in_year(start.year()));
    }
    off.extend(
        holidays
            .in_year(year + 1)
            .into_iter()
            .filter(|day| day.date.month() == 1),
    );
    let mut weeks = Vec::with_capacity(WEEKS);
    let mut day = start;
    for _ in 0..WEEKS {
        let mut days = Vec::with_capacity(7);
        let mut thursday = day;
        for _ in 0..7 {
            if day.weekday() == Weekday::Thursday {
                thursday = day;
            }
            let names: Vec<&str> = off.iter().filter(|h| h.date == day).map(|h| h.name.as_str()).collect();
            days.push(Day {
                date: day,
                in_month: day.month() == month,
                today: day == today,
                weekend: matches!(day.weekday(), Weekday::Saturday | Weekday::Sunday),
                holiday: (!names.is_empty()).then(|| names.join(", ")),
            });
            day = day.tomorrow().ok()?;
        }
        weeks.push(Week {
            number: thursday.iso_week_date().week(),
            days,
        });
    }
    let weekdays = (0..7)
        .map(|n| words::weekday_name(first.wrapping_add(n))[..3].to_owned())
        .collect();
    Some(Month {
        year,
        month,
        title: format!("{} {year}", words::month_name(month)),
        weekdays,
        weeks,
    })
}

/// Today's place in the year: `Day 297 · Week 43`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Today {
    pub date: Date,
    pub week: i8,
    pub day_of_year: i16,
}

pub fn today(now: &Zoned) -> Today {
    let date = now.date();
    Today {
        date,
        week: date.iso_week_date().week(),
        day_of_year: date.day_of_year(),
    }
}

/// An extra clock: its zone and how far it is from local time now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clock {
    pub name: String,
    pub label: String,
    /// The zone's offset from UTC now, in seconds; the popup adds it to UTC each second.
    pub offset: i32,
    /// `−6 h` from local time.
    pub relative: String,
}

/// The clocks for the zone names in the settings, leaving out a name no zone has.
pub fn clocks(names: &[String], zones: &Zones, now: &Zoned) -> Vec<Clock> {
    let here = now.offset().seconds();
    names
        .iter()
        .filter_map(|name| zones.get(name))
        .map(|zone| {
            let offset = now.with_time_zone(zone.tz.clone()).offset().seconds();
            Clock {
                name: zone.name,
                label: zone.label,
                offset,
                relative: words::offset_label(offset - here),
            }
        })
        .collect()
}

/// A day's details, for the day clicked in the calendar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DayDetails {
    /// `Friday 25 December 2026`.
    pub title: String,
    /// `in 62 days, a Friday · week 52 · day 359`.
    pub detail: String,
    pub holiday: Option<String>,
    /// The extra clocks on that day, at the local time of day it is now.
    pub zones: Vec<ZoneTime>,
}

/// One clock's time on a day.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZoneTime {
    pub label: String,
    /// `18:00`, with `+1` or `−1` after it when it is another day there.
    pub time: String,
}

/// The details of `date`, as of `now`.
pub fn day(date: Date, now: &Zoned, holidays: &Holidays, zones: &[Zone]) -> DayDetails {
    let days = query::days_between(now.date(), date).unwrap_or_default();
    let detail = format!(
        "{}, {} · week {} · day {}",
        words::relative_days(days),
        words::a_weekday(date.weekday()),
        date.iso_week_date().week(),
        date.day_of_year()
    );
    let here = date.to_datetime(now.time()).to_zoned(now.time_zone().clone()).ok();
    let zone_times = here
        .map(|here| {
            zones
                .iter()
                .map(|zone| {
                    let there = here.with_time_zone(zone.tz.clone());
                    let shift = query::days_between(date, there.date()).unwrap_or_default();
                    let mark = match shift {
                        0 => String::new(),
                        n if n > 0 => format!(" +{n}"),
                        n => format!(" \u{2212}{}", -n),
                    };
                    ZoneTime {
                        label: zone.label.clone(),
                        time: format!("{}{mark}", there.strftime("%H:%M")),
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    DayDetails {
        title: words::long_date(date),
        detail,
        holiday: holidays.on(date),
        zones: zone_times,
    }
}

/// When the countdown to `at` ends, as a timestamp: a date ends at its start in `tz`.
pub fn countdown_end(at: &str, tz: &TimeZone) -> Option<Timestamp> {
    if let Ok(date) = at.parse::<Date>() {
        return date.to_zoned(tz.clone()).ok().map(|zoned| zoned.timestamp());
    }
    at.parse::<Timestamp>().ok()
}

#[cfg(test)]
mod tests;
