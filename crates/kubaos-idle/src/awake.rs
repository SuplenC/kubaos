//! Keep awake: no lock, screens off or suspend after a time idle, and no lock or suspend when the
//! lid closes, for a while, until turned off, or until the battery falls to a level. It is an
//! `idle` inhibitor of logind's, held by `systemd-inhibit` around `kubaos-idle awake hold`, which
//! hypridle honours (`ignore_systemd_inhibit` stays off); the lid's holder ([`crate::lid`]) keeps
//! the lid from logind meanwhile. hypridle keeps running, so the lock before a suspend and a lock
//! asked for still work, and a scheduled lock, suspend, shut down or reboot still fires.
//!
//! Its state is a file in the runtime dir the bar watches, replaced whole on every change:
//!
//! ```json
//! {"active": true, "until": 1793000000, "battery": 0}
//! ```
//!
//! `until` is when it ends, in seconds since the epoch, 0 until turned off. `battery` is the level
//! it ends at, in percent, once on battery; 0 for none. Plugged in it never ends for the battery,
//! and a Keep awake for a while or until turned off never ends for it at all. The holder runs as
//! the systemd user unit [`UNIT`], so it outlives the menu and the shell, and stopping the unit
//! ends it.
//!
//! # Owns
//!
//! - The state and its file, what a Keep awake is asked for ([`Length`]), and when the battery ends
//!   it.
//!
//! # Not here
//!
//! - Starting and stopping the holder: `main.rs`. Reading the battery: [`crate::machine`]. The
//!   bar's marker: `config/quickshell/modules/bar/IdleIndicator.qml`.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use jiff::SignedDuration;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::settings::Power;

/// The holder's transient unit.
pub const UNIT: &str = "kubaos-keep-awake";

/// The state file in the runtime dir.
pub const FILE: &str = "kubaos-keep-awake.json";

/// The state file under the runtime dir `runtime`.
pub fn path(runtime: &Path) -> PathBuf {
    runtime.join(FILE)
}

/// Keep awake's state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub active: bool,
    /// When it ends, in seconds since the epoch; 0 until turned off.
    #[serde(default)]
    pub until: i64,
    /// The battery level it ends at, in percent, once on battery; 0 for none.
    #[serde(default)]
    pub battery: u8,
}

impl State {
    /// Kept awake until `until`, 0 for until turned off.
    #[must_use]
    pub fn on(until: i64) -> Self {
        State {
            active: true,
            until,
            battery: 0,
        }
    }

    /// Kept awake until the battery falls to `level`.
    #[must_use]
    pub fn until_battery(level: Level) -> Self {
        State {
            active: true,
            until: 0,
            battery: level.percent(),
        }
    }

    /// Whether it holds at `now`, in seconds since the epoch: on, and not past its end.
    pub fn holds(&self, now: i64) -> bool {
        self.active && (self.until == 0 || self.until > now)
    }

    /// The battery level it ends at, if it ends at one.
    pub fn level(&self) -> Option<Level> {
        Level::new(self.battery)
    }

    /// Whether the battery ends it: it ends at a level, and the battery has reached it
    /// ([`Level::reached`]).
    pub fn battery_ends(&self, power: Power, percent: Option<u8>) -> bool {
        self.level().is_some_and(|level| level.reached(power, percent))
    }

    /// Reads the state file; a missing or unreadable one is off.
    pub fn read(path: &Path) -> State {
        fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    /// Writes the state file through a temporary file and a rename, so the bar never reads half.
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

/// A battery level a Keep awake ends at: 1 % to 99 %. At 0 % the laptop is off before it ends,
/// and 100 % would end it at once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Level(u8);

impl Level {
    /// The lowest level.
    pub const MIN: u8 = 1;
    /// The highest level.
    pub const MAX: u8 = 99;

    /// `percent` as a level; `None` outside [`Level::MIN`] to [`Level::MAX`].
    pub fn new(percent: u8) -> Option<Level> {
        (Self::MIN..=Self::MAX).contains(&percent).then_some(Level(percent))
    }

    pub fn percent(self) -> u8 {
        self.0
    }

    /// Whether a battery at `percent` has reached the level on `power`: on battery, at the level or
    /// below. Plugged in it never has, and a battery that can't be read hasn't.
    pub fn reached(self, power: Power, percent: Option<u8>) -> bool {
        power == Power::Battery && percent.is_some_and(|percent| percent <= self.0)
    }
}

/// What a Keep awake is asked for, as typed: a while, as the clock's timer reads it (`30m`,
/// `1h30m`, a number alone in minutes), or a battery level (`20%`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Length {
    For(SignedDuration),
    UntilBattery(Level),
}

/// Why a typed Keep awake isn't one.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum LengthError {
    #[error("`{0}` isn't a time to keep awake for or a battery level: try 1h, 30m or 20%")]
    Unreadable(String),
    #[error("Keep awake lasts until the battery is at 1 % to 99 %, not {0} %")]
    LevelOutOfRange(u64),
}

impl Length {
    /// Reads a typed Keep awake: a level when it ends with `%`, else a while.
    ///
    /// # Errors
    ///
    /// When it is neither, or the level is outside 1 % to 99 %.
    pub fn parse(typed: &str) -> Result<Length, LengthError> {
        let typed = typed.trim();
        let unreadable = || LengthError::Unreadable(typed.to_owned());
        let Some(number) = typed.strip_suffix('%') else {
            return kubaos_clock::duration(typed).map(Length::For).ok_or_else(unreadable);
        };
        let percent: u64 = number.trim().parse().map_err(|_| unreadable())?;
        u8::try_from(percent)
            .ok()
            .and_then(Level::new)
            .map(Length::UntilBattery)
            .ok_or(LengthError::LevelOutOfRange(percent))
    }
}

#[cfg(test)]
mod tests;
