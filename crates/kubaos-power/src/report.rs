//! `kubaos power report`: what the battery's history comes to, as text a person can read or paste,
//! or as JSON, and "What's draining the battery", the Command menu's short answer.
//!
//! # Owns
//!
//! - Turning entries into [`Report`]'s stretches, averages and wakers, the report's text, and
//!   [`Drain`].
//!
//! # Not here
//!
//! - The entries: [`crate::history`]. What counts as wrong, and the lengths' words:
//!   [`crate::recorder`]. Printing it and opening it from a toast: `main.rs`. The menu's Items:
//!   `crates/kotodama-kubaos/src/power.rs`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use std::time::Duration;

use jiff::Timestamp;
use jiff::tz::TimeZone;
use serde::Serialize;

use crate::history;
use crate::history::DAY;
use crate::history::Entry;
use crate::history::RETENTION_DAYS;
use crate::history::Sample;
use crate::history::Sleep;
use crate::history::Wakers;
use crate::history::round;
use crate::machine::Machine;
use crate::reading;
use crate::recorder;
use crate::recorder::exact;
use crate::stop::Stop;

/// The days a report covers unless asked otherwise: a week shows a habit, and reads in one screen.
pub const DEFAULT_DAYS: i64 = 7;
/// A stretch shorter than this isn't one worth a line: the report is about nights and long
/// afternoons.
pub const LONG_STRETCH_SECONDS: i64 = 30 * 60;
/// A gap between two samples that means the stretch ended: a charge, a failed sleep's minutes, the
/// service down. A sample a minute that runs late by this much isn't late.
pub const SAMPLE_GAP_SECONDS: i64 = 3 * 60;
/// How many wakers of each sort the report names.
pub const REPORT_WAKERS: usize = 8;
/// How old the last count of the wakers may be for the menu's answer to use it: two scans' time.
pub const DRAIN_FRESH_SECONDS: i64 = 20 * 60;
/// How much of the history's end the menu's answer reads: hundreds of samples, tens of scans.
pub const DRAIN_TAIL_BYTES: u64 = 64 * 1024;
/// How long the menu's answer measures when no recent count is written down.
pub const DRAIN_WINDOW: Duration = Duration::from_secs(1);
/// How many wakers of each sort the menu's answer names.
pub const DRAIN_WAKERS: usize = 3;

/// What a stretch was.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// Asleep, at least half of it in hardware sleep.
    AsleepDeep,
    /// Asleep, under half of it in hardware sleep.
    AsleepShallow,
    /// Asleep on a machine that doesn't report hardware sleep.
    AsleepUnknown,
    AwakeLidClosed,
    AwakeLidOpen,
}

/// A long stretch.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Stretch {
    pub kind: Kind,
    pub start: i64,
    pub end: i64,
    pub percent_before: Option<f64>,
    pub percent_after: Option<f64>,
    /// What it lost by the gauge, which can jump at low charge; the energy is the truer number.
    pub lost_percent: Option<f64>,
    pub lost_wh: Option<f64>,
    pub watts: Option<f64>,
    pub percent_per_hour: Option<f64>,
    /// Asleep, the percent of it in hardware sleep.
    pub deep_percent: Option<f64>,
    pub keep_awake: bool,
    /// A charger made the drain meaningless.
    pub charger: bool,
    pub went_wrong: bool,
}

impl Stretch {
    fn seconds(&self) -> i64 {
        self.end - self.start
    }

    fn of_sleep(sleep: &Sleep) -> Stretch {
        let kind = match sleep.hw_share {
            None => Kind::AsleepUnknown,
            Some(share) if share < recorder::DEEP_SLEEP_SHARE => Kind::AsleepShallow,
            Some(_) => Kind::AsleepDeep,
        };
        Stretch {
            kind,
            start: sleep.at,
            end: sleep.end,
            percent_before: sleep.percent_before,
            percent_after: sleep.percent_after,
            lost_percent: lost(sleep.percent_before, sleep.percent_after).filter(|_| !sleep.charger),
            lost_wh: lost(sleep.wh_before, sleep.wh_after).filter(|_| !sleep.charger),
            watts: sleep.watts,
            percent_per_hour: sleep.percent_per_hour,
            deep_percent: sleep.hw_share.map(|share| round(share * 100.0, 0)),
            keep_awake: false,
            charger: sleep.charger,
            went_wrong: recorder::sleep_went_wrong(sleep),
        }
    }

    fn of_samples(samples: &[&Sample]) -> Option<Stretch> {
        let (first, last) = (samples.first()?, samples.last()?);
        let seconds = last.at - first.at;
        let closed = first.lid_closed == Some(true);
        let watts: Vec<f64> = samples.iter().filter_map(|sample| sample.watts).collect();
        let lost_percent = lost(first.percent, last.percent);
        Some(Stretch {
            kind: if closed {
                Kind::AwakeLidClosed
            } else {
                Kind::AwakeLidOpen
            },
            start: first.at,
            end: last.at,
            percent_before: first.percent,
            percent_after: last.percent,
            lost_percent,
            lost_wh: lost(first.wh, last.wh),
            watts: (!watts.is_empty()).then(|| round(watts.iter().sum::<f64>() / watts.len() as f64, 2)),
            percent_per_hour: lost_percent
                .filter(|_| seconds > 0)
                .map(|lost| round(lost * 3600.0 / seconds as f64, 2)),
            deep_percent: None,
            keep_awake: samples.iter().any(|sample| sample.keep_awake),
            charger: false,
            went_wrong: closed && recorder::closed_went_wrong(seconds),
        })
    }

    /// What it was, in words.
    fn words(&self) -> String {
        match self.kind {
            Kind::AsleepDeep | Kind::AsleepShallow => {
                format!("asleep, {:.0} % deep sleep", self.deep_percent.unwrap_or(0.0))
            }
            Kind::AsleepUnknown => "asleep, deep sleep unknown".to_owned(),
            Kind::AwakeLidClosed if self.keep_awake => "awake, lid closed, Keep awake on".to_owned(),
            Kind::AwakeLidClosed => "awake, lid closed".to_owned(),
            Kind::AwakeLidOpen if self.keep_awake => "awake, lid open, Keep awake on".to_owned(),
            Kind::AwakeLidOpen => "awake, lid open".to_owned(),
        }
    }

    /// What it cost, in words: the energy beside the percent, and watts where the energy or a draw
    /// gave them, else percent an hour.
    fn cost(&self) -> String {
        if self.charger {
            return match (self.percent_before, self.percent_after) {
                (Some(from), Some(to)) => format!("{from:.0} → {to:.0} %, with a charger in"),
                _ => "with a charger in".to_owned(),
            };
        }
        let (Some(from), Some(to)) = (self.percent_before, self.percent_after) else {
            return "the battery didn't say".to_owned();
        };
        let lost = self.lost_percent.unwrap_or(0.0);
        match (self.lost_wh, self.watts, self.percent_per_hour) {
            (Some(wh), Some(watts), _) => format!("{from:.0} → {to:.0} %, lost {lost:.0} % ({wh:.1} Wh), {watts:.1} W"),
            (None, Some(watts), _) => format!("{from:.0} → {to:.0} %, lost {lost:.0} %, {watts:.1} W"),
            (Some(wh), None, Some(rate)) => {
                format!("{from:.0} → {to:.0} %, lost {lost:.0} % ({wh:.1} Wh), {rate:.1} %/h")
            }
            (None, None, Some(rate)) => format!("{from:.0} → {to:.0} %, lost {lost:.0} %, {rate:.1} %/h"),
            (Some(wh), None, None) => format!("{from:.0} → {to:.0} %, lost {lost:.0} % ({wh:.1} Wh)"),
            (None, None, None) => format!("{from:.0} → {to:.0} %, lost {lost:.0} %"),
        }
    }
}

/// `from - to`, 0 at the least; a rise isn't a loss.
fn lost(from: Option<f64>, to: Option<f64>) -> Option<f64> {
    from.zip(to).map(|(from, to)| round((from - to).max(0.0), 2))
}

/// The average draw in a stop with the lid as it was.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Average {
    /// The stop's id.
    pub stop: String,
    pub lid_closed: bool,
    pub watts: f64,
    /// How many minutes of samples it is over.
    pub minutes: u32,
}

/// A waker over the period.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Waker {
    pub name: String,
    /// Wakeups a second, over the time counted.
    pub rate: f64,
}

/// The report on a period.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Report {
    pub days: i64,
    pub since: i64,
    pub until: i64,
    /// What went wrong first, then the rest, each newest first.
    pub stretches: Vec<Stretch>,
    /// Highest draw first.
    pub averages: Vec<Average>,
    pub irqs: Vec<Waker>,
    pub procs: Vec<Waker>,
}

/// When a report for `days` at `now` starts: never past what the file is meant to keep.
pub fn since(days: i64, now: i64) -> i64 {
    now - days.clamp(1, RETENTION_DAYS) * DAY
}

impl Report {
    /// The report on `entries`, oldest first, for the `days` up to `now`.
    pub fn of(entries: &[Entry], days: i64, now: i64) -> Report {
        let days = days.clamp(1, RETENTION_DAYS);
        let samples: Vec<&Sample> = entries
            .iter()
            .filter_map(|entry| match entry {
                Entry::Sample(sample) => Some(sample),
                _ => None,
            })
            .collect();
        let mut stretches: Vec<Stretch> = entries
            .iter()
            .filter_map(|entry| match entry {
                Entry::Sleep(sleep) => Some(Stretch::of_sleep(sleep)),
                _ => None,
            })
            .chain(awake_stretches(&samples))
            .filter(|stretch| stretch.seconds() >= LONG_STRETCH_SECONDS)
            .collect();
        stretches.sort_by_key(|stretch| (!stretch.went_wrong, -stretch.start));
        let (irqs, procs) = top_wakers(&samples);
        Report {
            days,
            since: since(days, now),
            until: now,
            stretches,
            averages: averages(&samples),
            irqs,
            procs,
        }
    }

    /// The report's text, its times in `zone`.
    pub fn text(&self, zone: &TimeZone) -> String {
        let mut out = format!(
            "Battery report: the last {} days, from {}\n",
            self.days,
            when(self.since, zone)
        );
        if self.stretches.is_empty() && self.averages.is_empty() {
            out.push_str("\nNothing written down yet: the history fills as the laptop runs on battery and sleeps.\n");
            return out;
        }
        let (wrong, fine): (Vec<&Stretch>, Vec<&Stretch>) =
            self.stretches.iter().partition(|stretch| stretch.went_wrong);
        section(&mut out, "Went wrong", &wrong, zone);
        section(&mut out, "Long stretches", &fine, zone);
        if !self.averages.is_empty() {
            out.push_str("\nAverage draw on battery\n");
            for average in &self.averages {
                let stop = Stop::from_id(&average.stop).map_or(average.stop.as_str(), |stop| stop.title());
                let text = if average.lid_closed {
                    format!(
                        "  {stop}, lid closed: {:.1} W over {}",
                        average.watts,
                        exact(i64::from(average.minutes) * 60)
                    )
                } else {
                    format!(
                        "  {stop}, lid open: {:.1} W over {}",
                        average.watts,
                        exact(i64::from(average.minutes) * 60)
                    )
                };
                let _ = writeln!(out, "{text}");
            }
        }
        wakers_section(&mut out, "Top interrupts (per second)", &self.irqs);
        wakers_section(&mut out, "Top processes waking (per second)", &self.procs);
        out
    }
}

fn section(out: &mut String, title: &str, stretches: &[&Stretch], zone: &TimeZone) {
    if stretches.is_empty() {
        return;
    }
    let _ = writeln!(out, "\n{title}");
    for stretch in stretches {
        let _ = writeln!(
            out,
            "  {} to {} ({}): {}\n    {}",
            when(stretch.start, zone),
            until(stretch.start, stretch.end, zone),
            exact(stretch.seconds()),
            stretch.words(),
            stretch.cost()
        );
    }
}

fn wakers_section(out: &mut String, title: &str, wakers: &[Waker]) {
    if wakers.is_empty() {
        return;
    }
    let _ = writeln!(out, "\n{title}");
    for waker in wakers {
        let _ = writeln!(out, "  {:>9.1}  {}", waker.rate, waker.name);
    }
}

fn zoned(at: i64, zone: &TimeZone) -> Option<jiff::Zoned> {
    Timestamp::from_second(at).ok().map(|at| at.to_zoned(zone.clone()))
}

/// A moment: `Fri Nov 6 00:50`.
fn when(at: i64, zone: &TimeZone) -> String {
    zoned(at, zone).map_or_else(|| at.to_string(), |at| at.strftime("%a %b %-d %H:%M").to_string())
}

/// A stretch's end: the time alone when it is the start's day.
fn until(start: i64, end: i64, zone: &TimeZone) -> String {
    match (zoned(start, zone), zoned(end, zone)) {
        (Some(start), Some(end)) if start.date() == end.date() => end.strftime("%H:%M").to_string(),
        _ => when(end, zone),
    }
}

/// The samples' stretches: unbroken runs with the lid the same.
fn awake_stretches(samples: &[&Sample]) -> Vec<Stretch> {
    samples
        .chunk_by(|a, b| b.at - a.at <= SAMPLE_GAP_SECONDS && a.lid_closed == b.lid_closed)
        .filter_map(Stretch::of_samples)
        .collect()
}

fn averages(samples: &[&Sample]) -> Vec<Average> {
    let mut summed: BTreeMap<(&str, bool), (f64, u32)> = BTreeMap::new();
    for sample in samples {
        if let Some(watts) = sample.watts {
            let sum = summed
                .entry((sample.stop.as_str(), sample.lid_closed == Some(true)))
                .or_default();
            sum.0 += watts;
            sum.1 += 1;
        }
    }
    let mut averages: Vec<Average> = summed
        .into_iter()
        .map(|((stop, lid_closed), (watts, minutes))| Average {
            stop: stop.to_owned(),
            lid_closed,
            watts: round(watts / f64::from(minutes), 2),
            minutes,
        })
        .collect();
    averages.sort_by(|a, b| b.watts.total_cmp(&a.watts).then_with(|| a.stop.cmp(&b.stop)));
    averages
}

/// The wakers of the samples that counted them, each weighted by how long its count ran.
fn top_wakers(samples: &[&Sample]) -> (Vec<Waker>, Vec<Waker>) {
    let counts: Vec<&Wakers> = samples.iter().filter_map(|sample| sample.wakers.as_ref()).collect();
    let seconds: f64 = counts.iter().map(|wakers| f64::from(wakers.secs)).sum();
    let top = |of: fn(&Wakers) -> &[(String, f64)]| {
        let mut summed: BTreeMap<&str, f64> = BTreeMap::new();
        for wakers in &counts {
            for (name, rate) in of(wakers) {
                *summed.entry(name).or_default() += rate * f64::from(wakers.secs);
            }
        }
        let mut wakers: Vec<Waker> = summed
            .into_iter()
            .map(|(name, count)| Waker {
                name: name.to_owned(),
                rate: round(count / seconds, 1),
            })
            .collect();
        wakers.sort_by(|a, b| b.rate.total_cmp(&a.rate).then_with(|| a.name.cmp(&b.name)));
        wakers.truncate(REPORT_WAKERS);
        wakers
    };
    (top(|wakers| &wakers.irqs), top(|wakers| &wakers.procs))
}

/// The menu's short answer to what is draining the battery.
#[derive(Clone, Debug, PartialEq)]
pub struct Drain {
    pub on_battery: bool,
    pub watts: Option<f64>,
    pub percent: Option<f64>,
    /// The stop's title.
    pub stop: String,
    pub wakers: Option<Wakers>,
}

impl Drain {
    /// The machine now in `stop`: the draw is live, and the wakers the last count in the history
    /// at `path` when one is recent, so the answer comes at once. With none, on battery, it
    /// measures for [`DRAIN_WINDOW`]; `sleep` waits. Plugged in, nothing is draining.
    pub fn now(machine: &Machine, stop: &str, path: Option<&Path>, now: i64, sleep: &mut dyn FnMut(Duration)) -> Drain {
        let battery = machine.battery();
        let on_battery = battery.is_some_and(|battery| battery.discharging);
        let written = path.and_then(|path| last_wakers(&history::read_tail(path, DRAIN_TAIL_BYTES), now));
        let wakers = if on_battery {
            written.or_else(|| reading::wakers_now(machine, DRAIN_WINDOW, sleep))
        } else {
            None
        };
        Drain {
            on_battery,
            watts: battery.and_then(|battery| battery.watts),
            percent: battery.and_then(|battery| battery.percent),
            stop: stop.to_owned(),
            wakers,
        }
    }

    /// The answer as rows: a title, and what it is.
    pub fn rows(&self) -> Vec<(String, String)> {
        let state = match self.percent {
            Some(percent) => format!("{}, {percent:.0} % left", self.stop),
            None => self.stop.clone(),
        };
        let first = match (self.on_battery, self.watts) {
            (false, _) => "Plugged in: nothing is draining the battery".to_owned(),
            (true, Some(watts)) => format!("Drawing {watts:.1} W now"),
            (true, None) => "The battery doesn't report its draw".to_owned(),
        };
        let mut rows = vec![(first, state)];
        if let Some(wakers) = &self.wakers {
            let row =
                |(name, rate): &(String, f64), what: &str| (format!("{name}: {rate:.0} a second"), what.to_owned());
            rows.extend(
                wakers
                    .procs
                    .iter()
                    .take(DRAIN_WAKERS)
                    .map(|waker| row(waker, "A process waking")),
            );
            rows.extend(
                wakers
                    .irqs
                    .iter()
                    .take(DRAIN_WAKERS)
                    .map(|waker| row(waker, "An interrupt line")),
            );
        }
        rows
    }
}

/// The newest count of the wakers in `entries`, oldest first, if it is recent at `now`.
fn last_wakers(entries: &[Entry], now: i64) -> Option<Wakers> {
    entries.iter().rev().find_map(|entry| match entry {
        Entry::Sample(sample) if now - sample.at <= DRAIN_FRESH_SECONDS => sample.wakers.clone(),
        _ => None,
    })
}

#[cfg(test)]
mod tests;
