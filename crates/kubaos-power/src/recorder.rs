//! When the battery's history gets an entry, and when a night that went wrong gets a toast.
//! [`Recorder`] is pure: it takes what happened and says what to write and what to say, so every
//! path is tested without a clock, a battery or a bus. [`Log`] is what the service holds: the
//! recorder with the file and the machine.
//!
//! ```text
//! on battery ──every minute──▶ sample          logind's sleep ──▶ Edge before
//! plug, stop, lid, Keep awake ▶ event          logind's wake, 5 s ▶ Edge after ▶ sleep ▶ toast?
//! lid opens after an hour closed on battery ─▶ toast
//! ```
//!
//! No timer runs while plugged in or asleep: [`Recorder::wait`] is `None` then, and the service's
//! loop goes on waiting for its bus. A write that fails is logged once and costs nothing else.
//!
//! # Owns
//!
//! - [`Recorder`], [`Log`], the thresholds of what went wrong, and the toasts' words ([`Note`]).
//!
//! # Not here
//!
//! - The file: [`crate::history`]. The reads: [`crate::reading`]. Showing a toast and opening the
//!   report: `main.rs`.

use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use crate::history;
use crate::history::Entry;
use crate::history::Event;
use crate::history::Sample;
use crate::history::Sleep;
use crate::history::What;
use crate::history::round;
use crate::machine::BatteryNow;
use crate::machine::Machine;
use crate::reading;
use crate::reading::Edge;
use crate::reading::Reader;
use crate::reading::Reading;

/// How often a sample comes on battery: fine enough to see a stretch's edges, and one wakeup a
/// minute next to the machine's hundreds a second.
pub const SAMPLE_EVERY: Duration = Duration::from_mins(1);
/// How long after a wake the battery is read. It is past [`crate::control::SETTLE`], and a charger
/// plugged in during the sleep can take a moment after the resume to show in `status`.
pub const WAKE_READING: Duration = Duration::from_secs(5);
/// How often the history is asked whether it needs pruning, which reads its first line.
pub const PRUNE_EVERY: Duration = Duration::from_hours(24);
/// A sleep over this long ought to reach hardware sleep: a shorter one may be a lid closed for a
/// walk down the corridor, whose drain doesn't show, and the kernel's entry takes part of it.
pub const LONG_SLEEP_SECONDS: i64 = 30 * 60;
/// Under this share of a long sleep in hardware sleep the battery drains many times faster than it
/// should; above it, a late entry or an early wake explains the rest.
pub const DEEP_SLEEP_SHARE: f64 = 0.5;
/// Longer than this awake with the lid closed on battery, nobody meant it: a move between rooms or
/// a build left to finish takes less.
pub const LONG_CLOSED_SECONDS: i64 = 60 * 60;
/// How much the energy must rise across a sleep to count as a charger rather than the gauge's
/// noise, in watt-hours.
const CHARGED_WH: f64 = 0.05;

/// A toast about something that went wrong. The percents are whole, as the battery reports them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Note {
    /// A long sleep that stayed out of hardware sleep; `share` is what it reached, from 0 to 1.
    Slept { seconds: i64, lost: f64, share: f64 },
    /// A long stretch awake with the lid closed on battery.
    AwakeClosed { seconds: i64, lost: f64, keep_awake: bool },
}

impl Note {
    /// The toast's title.
    pub const TITLE: &'static str = "Battery";
    /// The toast's button.
    pub const ACTION: &'static str = "Battery report";

    /// The toast's body.
    pub fn text(&self) -> String {
        match *self {
            Note::Slept { seconds, lost, share } if share > 0.0 => format!(
                "Slept {} and lost {lost:.0} %: only {:.0} % of it in deep sleep.",
                rough(seconds),
                share * 100.0
            ),
            Note::Slept { seconds, lost, .. } => {
                format!(
                    "Slept {} and lost {lost:.0} %: it didn't reach deep sleep.",
                    rough(seconds)
                )
            }
            Note::AwakeClosed {
                seconds,
                lost,
                keep_awake: true,
            } => format!(
                "Awake {} with the lid closed: Keep awake was on. Lost {lost:.0} %.",
                rough(seconds)
            ),
            Note::AwakeClosed {
                seconds,
                lost,
                keep_awake: false,
            } => format!("Awake {} with the lid closed. Lost {lost:.0} %.", rough(seconds)),
        }
    }
}

/// A length as a person says it: to the minute under 3 h, to the hour above.
pub fn rough(seconds: i64) -> String {
    if seconds >= 3 * 3600 {
        format!("{} h", (seconds + 1800) / 3600)
    } else {
        exact(seconds)
    }
}

/// A length to the minute: `45 min`, `2 h`, `1 h 25 min`.
pub fn exact(seconds: i64) -> String {
    let minutes = (seconds + 30) / 60;
    match (minutes / 60, minutes % 60) {
        (0, minutes) => format!("{minutes} min"),
        (hours, 0) => format!("{hours} h"),
        (hours, minutes) => format!("{hours} h {minutes} min"),
    }
}

/// Whether a sleep went wrong: long, on battery, and under half of it in hardware sleep. A machine
/// that can't tell never counts, and neither does a charger: the battery paid nothing, and a dock
/// keeps the machine out of hardware sleep night after night.
pub fn sleep_went_wrong(sleep: &Sleep) -> bool {
    !sleep.charger
        && sleep.end - sleep.at > LONG_SLEEP_SECONDS
        && sleep.hw_share.is_some_and(|share| share < DEEP_SLEEP_SHARE)
}

/// Whether a stretch awake with the lid closed on battery went wrong.
pub fn closed_went_wrong(seconds: i64) -> bool {
    seconds > LONG_CLOSED_SECONDS
}

/// The toast a sleep gets, if any.
pub fn sleep_note(sleep: &Sleep) -> Option<Note> {
    if !sleep_went_wrong(sleep) {
        return None;
    }
    Some(Note::Slept {
        seconds: sleep.end - sleep.at,
        lost: (sleep.percent_before? - sleep.percent_after?).max(0.0),
        share: sleep.hw_share?,
    })
}

/// A sleep's entry, from the machine at either end. It ended at `end`, by logind's word; `after`
/// comes a little later.
pub fn sleep_entry(before: &Edge, after: &Edge, end: i64, trigger: Option<&str>) -> Sleep {
    let seconds = end - before.at;
    let hours = seconds as f64 / 3600.0;
    let percent_before = before.battery.and_then(|battery| battery.percent);
    let percent_after = after.battery.and_then(|battery| battery.percent);
    let wh_before = before.battery.and_then(|battery| battery.wh);
    let wh_after = after.battery.and_then(|battery| battery.wh);
    let on_charger = |edge: &Edge| edge.battery.is_some_and(|battery| !battery.discharging);
    let rose = percent_before.zip(percent_after).is_some_and(|(from, to)| to > from)
        || wh_before.zip(wh_after).is_some_and(|(from, to)| to - from > CHARGED_WH);
    let charger = on_charger(before) || on_charger(after) || rose;
    let rate = |from: Option<f64>, to: Option<f64>| {
        let (from, to) = from.zip(to)?;
        (!charger && hours > 0.0).then(|| round((from - to) / hours, 2))
    };
    let hw_sleep_us = reading::hw_sleep_between(before.hw, after.hw, seconds);
    let hw_share = hw_sleep_us
        .filter(|_| seconds > 0)
        .map(|us| round((us as f64 / 1e6 / seconds as f64).min(1.0), 3));
    Sleep {
        at: before.at,
        end,
        trigger: trigger.map(str::to_owned),
        percent_before,
        percent_after,
        wh_before: wh_before.map(|wh| round(wh, 2)),
        wh_after: wh_after.map(|wh| round(wh, 2)),
        charger,
        watts: rate(wh_before, wh_after),
        percent_per_hour: rate(percent_before, percent_after),
        hw_sleep_us,
        last_hw_sleep_us: after.hw.last_us,
        hw_share,
    }
}

/// What the service follows, and the history with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Facts<'a> {
    pub on_battery: bool,
    /// The stop's id.
    pub stop: &'a str,
    /// The stop a low-battery drop left, while one is on.
    pub dropped_from: Option<&'a str>,
}

/// What to write, and what to say.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Out {
    pub entries: Vec<Entry>,
    pub notes: Vec<Note>,
    /// A charge or a sleep came between two samples: the reader's differences are no use.
    pub forget: bool,
}

/// A stretch awake with the lid closed on battery, under way.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Closed {
    since: i64,
    percent: Option<f64>,
    keep_awake: bool,
}

/// A wake whose battery reading waits for [`WAKE_READING`].
#[derive(Clone, Debug, PartialEq)]
struct Waking {
    before: Edge,
    trigger: Option<String>,
    end: i64,
    due: Instant,
}

/// What is due at a moment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Due {
    /// A wake's battery reading.
    Wake,
    Sample,
}

/// The history's decisions.
#[derive(Clone, Debug, Default)]
pub struct Recorder {
    started: bool,
    on_battery: bool,
    stop: String,
    dropped_from: Option<String>,
    lid_closed: Option<bool>,
    keep_awake: bool,
    next_sample: Option<Instant>,
    closed: Option<Closed>,
    /// A long closed stretch that ended with the lid still closed (a plug or a sleep), waiting for
    /// someone to come back to it.
    waiting: Option<Note>,
    asleep: Option<(Edge, Option<String>)>,
    waking: Option<Waking>,
}

impl Recorder {
    /// Whether `facts` differ from what the recorder knows.
    pub fn changes(&self, facts: &Facts<'_>) -> bool {
        !self.started
            || self.on_battery != facts.on_battery
            || self.stop != facts.stop
            || self.dropped_from.as_deref() != facts.dropped_from
    }

    /// The service's stop, source or drop as they stand; the first call marks the service's start.
    pub fn follow(&mut self, facts: &Facts<'_>, at: i64, percent: Option<f64>, keep_awake: bool, now: Instant) -> Out {
        let mut out = Out::default();
        let event = |what, detail: Option<&str>| {
            Entry::Event(Event {
                at,
                what,
                detail: detail.map(str::to_owned),
                percent,
            })
        };
        if self.started {
            if self.on_battery != facts.on_battery {
                let what = if facts.on_battery { What::Unplug } else { What::Plug };
                out.entries.push(event(what, None));
                out.forget = true;
            }
            if self.stop != facts.stop {
                out.entries.push(event(What::Stop, Some(facts.stop)));
            }
            if self.dropped_from.is_none()
                && let Some(from) = facts.dropped_from
            {
                out.entries.push(event(What::LowBattery, Some(from)));
            }
        } else {
            self.started = true;
            // Where Keep awake stands at the start is no change, and the samples say it.
            self.keep_awake = keep_awake;
            out.entries.push(event(What::ServiceStart, Some(facts.stop)));
        }
        self.on_battery = facts.on_battery;
        facts.stop.clone_into(&mut self.stop);
        self.dropped_from = facts.dropped_from.map(str::to_owned);
        self.heard_keep_awake(keep_awake, at, percent, &mut out);
        self.settle_closed(at, percent);
        self.settle_timer(now);
        out
    }

    /// The lid, as UPower has it. Its first word only says where things start.
    pub fn lid(&mut self, closed: bool, at: i64, percent: Option<f64>, keep_awake: bool) -> Out {
        let mut out = Out::default();
        let known = self.lid_closed;
        self.lid_closed = Some(closed);
        self.heard_keep_awake(keep_awake, at, percent, &mut out);
        self.settle_closed(at, percent);
        if known.is_some_and(|known| known != closed) {
            out.entries.push(Entry::Event(Event {
                at,
                what: if closed { What::LidClosed } else { What::LidOpen },
                detail: None,
                percent,
            }));
        }
        if !closed {
            out.notes.extend(self.waiting.take());
        }
        out
    }

    /// Keep awake as a sample or an event read it.
    fn heard_keep_awake(&mut self, on: bool, at: i64, percent: Option<f64>, out: &mut Out) {
        if on && let Some(closed) = &mut self.closed {
            closed.keep_awake = true;
        }
        if on != self.keep_awake {
            self.keep_awake = on;
            out.entries.push(Entry::Event(Event {
                at,
                what: if on { What::KeepAwakeOn } else { What::KeepAwakeOff },
                detail: None,
                percent,
            }));
        }
    }

    /// Starts or ends the closed stretch, as the lid, the power source and the sleep now say.
    fn settle_closed(&mut self, at: i64, percent: Option<f64>) {
        let holds = self.lid_closed == Some(true) && self.on_battery && self.asleep.is_none() && self.waking.is_none();
        match (holds, self.closed) {
            (true, None) => {
                self.closed = Some(Closed {
                    since: at,
                    percent,
                    keep_awake: self.keep_awake,
                });
            }
            (false, Some(closed)) => {
                self.closed = None;
                let seconds = at - closed.since;
                if closed_went_wrong(seconds)
                    && let Some((from, to)) = closed.percent.zip(percent)
                {
                    self.waiting = Some(Note::AwakeClosed {
                        seconds,
                        lost: (from - to).max(0.0),
                        keep_awake: closed.keep_awake || self.keep_awake,
                    });
                }
            }
            _ => {}
        }
    }

    /// A timer only while discharging and awake.
    fn settle_timer(&mut self, now: Instant) {
        if self.on_battery && self.asleep.is_none() && self.waking.is_none() {
            self.next_sample.get_or_insert(now + SAMPLE_EVERY);
        } else {
            self.next_sample = None;
        }
    }

    /// logind says the machine is going to sleep; `edge` was read as it said so.
    pub fn going_to_sleep(&mut self, edge: Edge) -> Out {
        let mut out = self.finish_waking(&edge, edge.at);
        let percent = edge.battery.and_then(|battery| battery.percent);
        let trigger = (self.lid_closed == Some(true)).then(|| "lid".to_owned());
        self.asleep = Some((edge, trigger));
        self.settle_closed(edge.at, percent);
        self.next_sample = None;
        out.forget = true;
        out
    }

    /// logind says the machine woke at `at`.
    pub fn woke(&mut self, at: i64, now: Instant) {
        let Some((before, trigger)) = self.asleep.take() else {
            return;
        };
        self.waking = Some(Waking {
            before,
            trigger,
            end: at,
            due: now + WAKE_READING,
        });
    }

    /// The reading after the wake came: the sleep's entry, and a toast if it went wrong or a closed
    /// stretch before it did.
    pub fn after_wake(&mut self, after: &Edge, now: Instant) -> Out {
        let mut out = self.finish_waking(after, after.at);
        self.settle_timer(now);
        out.forget = true;
        out
    }

    fn finish_waking(&mut self, after: &Edge, at: i64) -> Out {
        let mut out = Out::default();
        let Some(waking) = self.waking.take() else {
            return out;
        };
        let sleep = sleep_entry(&waking.before, after, waking.end, waking.trigger.as_deref());
        out.notes.extend(self.waiting.take());
        out.notes.extend(sleep_note(&sleep));
        out.entries.push(Entry::Sleep(sleep));
        self.settle_closed(at, after.battery.and_then(|battery| battery.percent));
        out
    }

    /// How long until something is due; `None` while plugged in or asleep, when no timer runs.
    pub fn wait(&self, now: Instant) -> Option<Duration> {
        let wake = self.waking.as_ref().map(|waking| waking.due);
        wake.or(self.next_sample).map(|due| due.saturating_duration_since(now))
    }

    /// What is due by `now`.
    pub fn due(&self, now: Instant) -> Option<Due> {
        if let Some(waking) = &self.waking {
            return (now >= waking.due).then_some(Due::Wake);
        }
        self.next_sample.filter(|due| now >= *due).map(|_| Due::Sample)
    }

    /// A sample's `reading`, at `at` by the wall clock and `now` by the monotonic one. A battery
    /// not discharging writes nothing: the charger's on its way to counting.
    pub fn sample(&mut self, reading: Reading, at: i64, now: Instant) -> Out {
        self.next_sample = Some(now + SAMPLE_EVERY);
        let mut out = Out::default();
        let percent = reading.battery.and_then(|battery| battery.percent);
        self.heard_keep_awake(reading.keep_awake, at, percent, &mut out);
        let Some(BatteryNow {
            watts,
            percent,
            wh,
            discharging: true,
        }) = reading.battery
        else {
            return out;
        };
        out.entries.push(Entry::Sample(Sample {
            at,
            percent,
            wh: wh.map(|wh| round(wh, 2)),
            watts: watts.map(|watts| round(watts, 2)),
            stop: self.stop.clone(),
            lid_closed: self.lid_closed,
            panel_on: reading.panel_on,
            brightness: reading.brightness,
            keep_awake: reading.keep_awake,
            docked: reading.docked,
            cpu: reading.cpu,
            wakers: reading.wakers,
        }));
        out
    }
}

/// The recorder with the file and the machine: what the service holds. Nothing here can disturb the
/// service: a write that fails is logged, once until one works again.
#[derive(Debug)]
pub struct Log {
    recorder: Recorder,
    reader: Reader,
    path: Option<PathBuf>,
    write_failed: bool,
    next_prune: Option<Instant>,
}

impl Log {
    /// The log of `machine`, written to `path`, with Keep awake's state file at `keep_awake`.
    pub fn new(machine: Machine, path: Option<PathBuf>, keep_awake: Option<PathBuf>) -> Log {
        Log {
            recorder: Recorder::default(),
            reader: Reader::new(machine, keep_awake),
            path,
            write_failed: false,
            next_prune: None,
        }
    }

    fn percent(&self) -> Option<f64> {
        self.reader.machine().battery().and_then(|battery| battery.percent)
    }

    /// Hears the service's stop, source and drop; cheap when none changed.
    pub fn follow(&mut self, facts: &Facts<'_>, now: Instant) {
        if !self.recorder.changes(facts) {
            return;
        }
        let at = history::now();
        let (percent, keep_awake) = (self.percent(), self.reader.keep_awake(at));
        let out = self.recorder.follow(facts, at, percent, keep_awake, now);
        self.prune_when_due(now);
        self.write(out);
    }

    /// Hears the lid; what comes back is for the toasts.
    pub fn lid(&mut self, closed: bool) -> Vec<Note> {
        let at = history::now();
        let (percent, keep_awake) = (self.percent(), self.reader.keep_awake(at));
        let out = self.recorder.lid(closed, at, percent, keep_awake);
        self.write(out)
    }

    /// Hears that the machine goes to sleep; `edge` was read as logind said so.
    pub fn going_to_sleep(&mut self, edge: Edge) -> Vec<Note> {
        let out = self.recorder.going_to_sleep(edge);
        self.write(out)
    }

    /// Hears that the machine woke.
    pub fn woke(&mut self, now: Instant) {
        self.recorder.woke(history::now(), now);
    }

    /// How long the service's loop may wait for the log.
    pub fn wait(&self, now: Instant) -> Option<Duration> {
        self.recorder.wait(now)
    }

    /// Does what is due by `now`; what comes back is for the toasts.
    pub fn tick(&mut self, now: Instant) -> Vec<Note> {
        let at = history::now();
        match self.recorder.due(now) {
            Some(Due::Wake) => {
                let edge = Edge::read(self.reader.machine(), at);
                let out = self.recorder.after_wake(&edge, now);
                self.write(out)
            }
            Some(Due::Sample) => {
                let reading = self.reader.read(at, now);
                let out = self.recorder.sample(reading, at, now);
                self.prune_when_due(now);
                self.write(out)
            }
            None => Vec::new(),
        }
    }

    /// Ends a line a crash cut and drops what is old: at the start, then once a day.
    fn prune_when_due(&mut self, now: Instant) {
        if self.next_prune.is_some_and(|due| now < due) {
            return;
        }
        let first = self.next_prune.is_none();
        self.next_prune = Some(now + PRUNE_EVERY);
        let Some(path) = &self.path else {
            return;
        };
        let at = history::now();
        let result = if first {
            history::end_last_line(path).and_then(|()| history::prune(path, at))
        } else {
            history::prune(path, at)
        };
        if let Err(err) = result {
            log::warn!("couldn't prune {}: {err}", path.display());
        }
    }

    fn write(&mut self, out: Out) -> Vec<Note> {
        if out.forget {
            self.reader.forget();
        }
        if let Some(path) = &self.path {
            for entry in &out.entries {
                match history::append(path, entry) {
                    Ok(()) => self.write_failed = false,
                    Err(err) => {
                        if !self.write_failed {
                            log::warn!("couldn't write {}: {err}", path.display());
                        }
                        self.write_failed = true;
                    }
                }
            }
        }
        out.notes
    }
}

#[cfg(test)]
mod tests;
