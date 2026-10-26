//! A power action scheduled for a time, whether idle or not: "Lock in 10m", "Suspend at 23:00",
//! "Shut down at friday 17:00", "Reboot in 5m". It is a set of systemd user transient timers with
//! fixed names, so it outlives the menu and the shell, `systemctl --user list-timers
//! 'kubaos-scheduled*'` lists it, and stopping them cancels it:
//!
//! ```text
//! kubaos-scheduled-warn-1m.timer    a minute before: a heads-up toast with Cancel
//! kubaos-scheduled-warn-10s.timer   10 s before: another
//! kubaos-scheduled-action.timer     then `kubaos-idle schedule fire ACTION AT`
//! ```
//!
//! One action is scheduled at a time, whichever it is: a new one stops the timers first, which
//! replaces the old. The timers are on the calendar, not the time awake, so one that came due while
//! suspended fires at once on resume. Keep awake doesn't hold any of them back: they were asked
//! for. Each runs in the Hyprland session, as a keybind would: the lock with `loginctl
//! lock-session`, suspend with `systemctl suspend` (hypridle's `before_sleep_cmd` locks first),
//! shut down and reboot with `dot-session leave`, so `graphical-session.target` stops cleanly.
//!
//! Its state is a file in the runtime dir the bar watches: `{"at": 1793000000, "action":
//! "shutdown"}`, `at` 0 for none. A heads-up and the action itself act only while the state still
//! names them, so a Cancel clicked on a stale toast never cancels the action that replaced it.
//!
//! # Owns
//!
//! - The actions and what each runs, the timers' names, when each fires and what it runs, and the
//!   state file.
//!
//! # Not here
//!
//! - Running `systemd-run`, `systemctl` and the actions: `main.rs`. Reading `10m` and
//!   `friday 17:00`: `kubaos_clock::duration` and `kubaos_clock::alarm`.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use jiff::SignedDuration;
use jiff::Zoned;
use jiff::tz::TimeZone;
use serde::Deserialize;
use serde::Serialize;

/// The action's unit, `.timer` and `.service`.
pub const ACTION_UNIT: &str = "kubaos-scheduled-action";
/// The heads-up a minute before.
pub const WARN_MINUTE_UNIT: &str = "kubaos-scheduled-warn-1m";
/// The heads-up 10 s before.
pub const WARN_SECONDS_UNIT: &str = "kubaos-scheduled-warn-10s";
/// Every unit a scheduled action may have, which cancelling stops.
pub const UNITS: [&str; 3] = [ACTION_UNIT, WARN_MINUTE_UNIT, WARN_SECONDS_UNIT];

/// The state file in the runtime dir.
pub const FILE: &str = "kubaos-scheduled.json";

/// A heads-up comes only when there is at least this much more time than its own lead, so the
/// toast for "Lock in 1m" isn't the same moment as the one that set it.
const WARN_MARGIN: SignedDuration = SignedDuration::from_secs(5);

/// What a scheduled action does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    #[default]
    Lock,
    Suspend,
    Shutdown,
    Reboot,
}

impl Action {
    pub const ALL: [Action; 4] = [Action::Lock, Action::Suspend, Action::Shutdown, Action::Reboot];

    /// Its name on the command line and in the state file.
    pub fn name(self) -> &'static str {
        match self {
            Action::Lock => "lock",
            Action::Suspend => "suspend",
            Action::Shutdown => "shutdown",
            Action::Reboot => "reboot",
        }
    }

    /// The action named `name`; `poweroff` and `shut-down` are `shutdown`, `restart` is `reboot`.
    pub fn from_name(name: &str) -> Option<Action> {
        match name {
            "lock" => Some(Action::Lock),
            "suspend" | "sleep" => Some(Action::Suspend),
            "shutdown" | "shut-down" | "poweroff" => Some(Action::Shutdown),
            "reboot" | "restart" => Some(Action::Reboot),
            _ => None,
        }
    }

    /// It as a noun, after "the": "the shut down at 17:00".
    pub fn noun(self) -> &'static str {
        match self {
            Action::Lock => "lock",
            Action::Suspend => "suspend",
            Action::Shutdown => "shut down",
            Action::Reboot => "reboot",
        }
    }

    /// What it does, to start a sentence: "Shuts down at 17:00".
    pub fn does(self) -> &'static str {
        match self {
            Action::Lock => "Locks",
            Action::Suspend => "Suspends",
            Action::Shutdown => "Shuts down",
            Action::Reboot => "Reboots",
        }
    }

    /// What it is about to do: "Shutting down in 1 min".
    pub fn doing(self) -> &'static str {
        match self {
            Action::Lock => "Locking",
            Action::Suspend => "Suspending",
            Action::Shutdown => "Shutting down",
            Action::Reboot => "Rebooting",
        }
    }

    /// The command it runs in the session.
    pub fn command(self) -> &'static [&'static str] {
        match self {
            Action::Lock => &["loginctl", "lock-session"],
            Action::Suspend => &["systemctl", "suspend"],
            Action::Shutdown => &["dot-session", "leave", "poweroff"],
            Action::Reboot => &["dot-session", "leave", "reboot"],
        }
    }
}

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
    /// unloads a timer once it has fired, so its name is free for the next action.
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
        argv.push(format!("--description=KubaOS scheduled action ({})", self.unit));
        argv.push(format!("--on-calendar={}", local.strftime("%Y-%m-%d %H:%M:%S")));
        argv.push("--".to_owned());
        argv.extend(self.argv.iter().cloned());
        argv
    }
}

/// The timers for `action` at `at`, made at `now`, each running `exe` (`kubaos-idle`): the
/// heads-ups that still have time, then the action. Each names the action and its time, which
/// [`State::is`] checks before it acts.
pub fn plan(action: Action, at: &Zoned, now: &Zoned, exe: &str) -> Vec<Transient> {
    let left = now.duration_until(at);
    let at_secs = at.timestamp().as_second().to_string();
    let run = |words: &[&str]| -> Vec<String> {
        std::iter::once(exe)
            .chain(words.iter().copied())
            .chain([action.name(), at_secs.as_str()])
            .map(str::to_owned)
            .collect()
    };
    let mut timers = Vec::new();
    for (unit, lead, words) in [
        (
            WARN_MINUTE_UNIT,
            SignedDuration::from_secs(60),
            ["schedule", "warn", "60"],
        ),
        (
            WARN_SECONDS_UNIT,
            SignedDuration::from_secs(10),
            ["schedule", "warn", "10"],
        ),
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
        unit: ACTION_UNIT,
        at: at.clone(),
        argv: run(&["schedule", "fire"]),
    });
    timers
}

/// The timers a cancel stops: every one a scheduled action may have, so a stale heads-up goes too.
pub fn timers() -> Vec<String> {
    UNITS.iter().map(|unit| format!("{unit}.timer")).collect()
}

/// The scheduled action's state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// When it comes, in seconds since the epoch; 0 for nothing scheduled.
    pub at: i64,
    #[serde(default)]
    pub action: Action,
}

impl State {
    /// `action` at `at`, in seconds since the epoch.
    #[must_use]
    pub fn new(action: Action, at: i64) -> Self {
        State { at, action }
    }

    /// Whether an action is still to come at `now`, in seconds since the epoch.
    pub fn pending(&self, now: i64) -> bool {
        self.at > now
    }

    /// Whether this is `action` at `at`: what a heads-up or the action itself checks, so one left
    /// from a schedule since replaced or cancelled does nothing.
    pub fn is(&self, action: Action, at: i64) -> bool {
        self.at != 0 && self.at == at && self.action == action
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

/// When "… in `text`" comes, from `now`: a duration as the clock reads a timer's (`10m`, `1h30m`,
/// `90s`, `25` for minutes).
pub fn after(text: &str, now: &Zoned) -> Option<Zoned> {
    let duration = kubaos_clock::duration(text)?;
    now.checked_add(duration).ok()
}

/// When "… at `text`" comes, from `now`: the next time a time of day comes round (`20:00`,
/// `9pm`), or a moment with a day (`friday 17:00`, `tomorrow 8am`), never in the past.
pub fn at_time(text: &str, now: &Zoned) -> Option<Zoned> {
    kubaos_clock::alarm(text, now, &kubaos_clock::zones::Zones::new())
}

#[cfg(test)]
mod tests;
