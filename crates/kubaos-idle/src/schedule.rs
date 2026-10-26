//! A lock scheduled for a time, whether idle or not: "Lock in 10m", "Lock at friday 17:00". It is
//! a set of systemd user transient timers with fixed names, so it outlives the menu and the shell,
//! `systemctl --user list-timers 'kubaos-scheduled-lock*'` lists it, and stopping them cancels it:
//!
//! ```text
//! kubaos-scheduled-lock-warn-1m.timer    a minute before: a heads-up toast
//! kubaos-scheduled-lock-warn-10s.timer   10 s before: another
//! kubaos-scheduled-lock.timer            then `kubaos-idle lock now`
//! ```
//!
//! One lock is scheduled at a time: a new one stops the timers first, which replaces the old. The
//! timers are on the calendar, not the time awake, so one that came due while suspended fires at
//! once on resume. Keep awake doesn't hold it back: a scheduled lock was asked for.
//!
//! Its state is a file in the runtime dir the bar watches: `{"at": 1793000000}`, 0 for none.
//!
//! # Owns
//!
//! - The timers' names, when each fires and what it runs, and the state file.
//!
//! # Not here
//!
//! - Running `systemd-run` and `systemctl`: `main.rs`. Reading `10m` and `friday 17:00`:
//!   `kubaos_clock::duration` and `kubaos_clock::alarm`.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use jiff::SignedDuration;
use jiff::Zoned;
use jiff::tz::TimeZone;
use serde::Deserialize;
use serde::Serialize;

/// The lock's unit, `.timer` and `.service`.
pub const LOCK_UNIT: &str = "kubaos-scheduled-lock";
/// The heads-up a minute before.
pub const WARN_MINUTE_UNIT: &str = "kubaos-scheduled-lock-warn-1m";
/// The heads-up 10 s before.
pub const WARN_SECONDS_UNIT: &str = "kubaos-scheduled-lock-warn-10s";
/// Every unit a scheduled lock may have, which cancelling stops.
pub const UNITS: [&str; 3] = [LOCK_UNIT, WARN_MINUTE_UNIT, WARN_SECONDS_UNIT];

/// The state file in the runtime dir.
pub const FILE: &str = "kubaos-scheduled-lock.json";

/// A heads-up comes only when there is at least this much more time than its own lead, so the
/// toast for "Lock in 1m" isn't the same moment as the one that set it.
const WARN_MARGIN: SignedDuration = SignedDuration::from_secs(5);

/// One transient timer: its unit, when it fires, and the program it runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transient {
    pub unit: &'static str,
    pub at: Zoned,
    pub argv: Vec<String>,
}

impl Transient {
    /// The `systemd-run` command line that makes it. `AccuracySec=1s`, since systemd's default
    /// minute would make a lock at 17:00 come at any time up to 17:01. `RemainAfterElapse=no`
    /// unloads a timer once it has fired, so its name is free for the next lock.
    pub fn systemd_run(&self) -> Vec<String> {
        let local = self.at.with_time_zone(TimeZone::system());
        let mut argv: Vec<String> = [
            "systemd-run",
            "--user",
            "--quiet",
            "--collect",
            "--timer-property=AccuracySec=1s",
            "--timer-property=RemainAfterElapse=no",
        ]
        .map(str::to_owned)
        .to_vec();
        argv.push(format!("--unit={}", self.unit));
        argv.push(format!("--description=KubaOS scheduled lock ({})", self.unit));
        argv.push(format!("--on-calendar={}", local.strftime("%Y-%m-%d %H:%M:%S")));
        argv.push("--".to_owned());
        argv.extend(self.argv.iter().cloned());
        argv
    }
}

/// The timers for a lock at `at`, made at `now`, each running `exe` (`kubaos-idle`): the
/// heads-ups that still have time, then the lock.
pub fn plan(at: &Zoned, now: &Zoned, exe: &str) -> Vec<Transient> {
    let left = now.duration_until(at);
    let run = |words: &[&str]| -> Vec<String> {
        std::iter::once(exe)
            .chain(words.iter().copied())
            .map(str::to_owned)
            .collect()
    };
    let mut timers = Vec::new();
    for (unit, lead, words) in [
        (WARN_MINUTE_UNIT, SignedDuration::from_secs(60), ["lock", "warn", "60"]),
        (WARN_SECONDS_UNIT, SignedDuration::from_secs(10), ["lock", "warn", "10"]),
    ] {
        if left < lead + WARN_MARGIN {
            continue;
        }
        if let Ok(warn_at) = at.checked_sub(lead) {
            timers.push(Transient {
                unit,
                at: warn_at,
                argv: run(&words),
            });
        }
    }
    timers.push(Transient {
        unit: LOCK_UNIT,
        at: at.clone(),
        argv: run(&["lock", "now"]),
    });
    timers
}

/// The timers a cancel stops: every one a scheduled lock may have, so a stale heads-up goes too.
pub fn timers() -> Vec<String> {
    UNITS.iter().map(|unit| format!("{unit}.timer")).collect()
}

/// The scheduled lock's state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// When it locks, in seconds since the epoch; 0 for no lock scheduled.
    pub at: i64,
}

impl State {
    /// Whether a lock is still to come at `now`, in seconds since the epoch.
    pub fn pending(&self, now: i64) -> bool {
        self.at > now
    }

    /// Reads the state file; a missing or unreadable one is none.
    pub fn read(path: &Path) -> State {
        fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    /// Writes the state file through a temporary file and a rename.
    ///
    /// # Errors
    ///
    /// When the file can't be written.
    pub fn write(&self, path: &Path) -> io::Result<()> {
        let text = serde_json::to_string(self).map_err(io::Error::other)?;
        let tmp = path.with_extension("json.new");
        fs::write(&tmp, text + "\n")?;
        fs::rename(&tmp, path)
    }
}

/// The state file under the runtime dir `runtime`.
pub fn path(runtime: &Path) -> PathBuf {
    runtime.join(FILE)
}

/// When "Lock in `text`" locks, from `now`: a duration as the clock reads a timer's (`10m`,
/// `1h30m`, `90s`, `25` for minutes).
pub fn lock_in(text: &str, now: &Zoned) -> Option<Zoned> {
    let duration = kubaos_clock::duration(text)?;
    now.checked_add(duration).ok()
}

/// When "Lock at `text`" locks, from `now`: the next time a time of day comes round (`20:00`,
/// `9pm`), or a moment with a day (`friday 17:00`, `tomorrow 8am`), never in the past.
pub fn lock_at(text: &str, now: &Zoned) -> Option<Zoned> {
    kubaos_clock::alarm(text, now, &kubaos_clock::zones::Zones::new())
}

#[cfg(test)]
mod tests;
