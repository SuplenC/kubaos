//! Keep awake: no lock, screens off or suspend after a time idle, for a while or until turned
//! off. It is an `idle` inhibitor of logind's, held by `systemd-inhibit` around `kubaos-idle awake
//! hold`, which hypridle honours (`ignore_systemd_inhibit` stays off). hypridle keeps running, so
//! the lock before a suspend and a lock asked for still work, and a scheduled lock, suspend, shut down or reboot
//! still fires.
//!
//! Its state is a file in the runtime dir the bar watches, replaced whole on every change:
//!
//! ```json
//! {"active": true, "until": 1793000000}
//! ```
//!
//! `until` is when it ends, in seconds since the epoch, 0 until turned off. The holder runs as the
//! systemd user unit [`UNIT`], so it outlives the menu and the shell, and stopping the unit ends it.
//!
//! # Owns
//!
//! - The state and its file.
//!
//! # Not here
//!
//! - Starting and stopping the holder: `main.rs`. The bar's marker:
//!   `config/quickshell/modules/bar/IdleIndicator.qml`.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

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
}

impl State {
    /// Kept awake until `until`, 0 for until turned off.
    #[must_use]
    pub fn on(until: i64) -> Self {
        State { active: true, until }
    }

    /// Whether it holds at `now`, in seconds since the epoch: on, and not past its end.
    pub fn holds(&self, now: i64) -> bool {
        self.active && (self.until == 0 || self.until > now)
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

#[cfg(test)]
mod tests;
