//! What the service remembers across restarts, logins and reboots, in
//! `~/.local/state/kubaos/power/stops.json`: the stop last chosen on battery and plugged in, the
//! stop it last left the machine on, and the low-battery drop in progress.
//!
//! ```json
//! {"battery":"super-saver","ac":"balanced","last":{"source":"battery","stop":"power-saver"},
//!  "dropped_from":"super-saver","undone":false}
//! ```
//!
//! It is state, not a setting: the user layer holds what the user writes by hand, this what the
//! slider last did. A file that doesn't read is the defaults, never an error.
//!
//! # Owns
//!
//! - [`Memory`], its file and its path.
//!
//! # Not here
//!
//! - Deciding what to remember: [`crate::control`].

use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::stop::Source;
use crate::stop::Stop;

/// The file in the state folder.
pub const FILE: &str = "stops.json";

/// What the service remembers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Memory {
    /// The stop last chosen on battery.
    pub battery: Stop,
    /// The stop last chosen plugged in.
    pub ac: Stop,
    /// The source and the stop the machine was last left on.
    pub last: Option<(Source, Stop)>,
    /// The stop before a low-battery drop that is still on.
    pub dropped_from: Option<Stop>,
    /// The user undid this discharge's drop, or chose a stop below the level; no drop comes again
    /// until the laptop is plugged in.
    pub undone: bool,
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            battery: Source::Battery.default_stop(),
            ac: Source::Ac.default_stop(),
            last: None,
            dropped_from: None,
            undone: false,
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
struct LastJson {
    source: String,
    stop: String,
}

#[derive(Default, Deserialize, Serialize)]
struct MemoryJson {
    #[serde(default)]
    battery: String,
    #[serde(default)]
    ac: String,
    #[serde(default)]
    last: Option<LastJson>,
    #[serde(default)]
    dropped_from: Option<String>,
    #[serde(default)]
    undone: bool,
}

impl Memory {
    /// The stop remembered for `source`.
    pub fn stop_for(&self, source: Source) -> Stop {
        match source {
            Source::Battery => self.battery,
            Source::Ac => self.ac,
        }
    }

    /// Remembers `stop` as `source`'s choice.
    pub fn choose(&mut self, source: Source, stop: Stop) {
        match source {
            Source::Battery => self.battery = stop,
            Source::Ac => self.ac = stop,
        }
    }

    /// The memory in `text`; what doesn't read takes its default.
    pub fn parse(text: &str) -> Memory {
        let Ok(json) = serde_json::from_str::<MemoryJson>(text) else {
            return Memory::default();
        };
        let defaults = Memory::default();
        let last = json.last.and_then(|last| {
            let source = match last.source.as_str() {
                "battery" => Source::Battery,
                "ac" => Source::Ac,
                _ => return None,
            };
            Some((source, Stop::from_id(&last.stop)?))
        });
        Memory {
            battery: Stop::from_id(&json.battery).unwrap_or(defaults.battery),
            ac: Stop::from_id(&json.ac).unwrap_or(defaults.ac),
            last,
            dropped_from: json.dropped_from.as_deref().and_then(Stop::from_id),
            undone: json.undone,
        }
    }

    /// The file's text.
    ///
    /// # Panics
    ///
    /// Never: the JSON is strings and a boolean, which always serialise.
    pub fn text(&self) -> String {
        let json = MemoryJson {
            battery: self.battery.id().to_owned(),
            ac: self.ac.id().to_owned(),
            last: self.last.map(|(source, stop)| LastJson {
                source: source.key().to_owned(),
                stop: stop.id().to_owned(),
            }),
            dropped_from: self.dropped_from.map(|stop| stop.id().to_owned()),
            undone: self.undone,
        };
        serde_json::to_string(&json).expect("the memory is plain JSON") + "\n"
    }

    /// The memory in the file at `path`; a missing or broken file is the defaults.
    pub fn load(path: &Path) -> Memory {
        fs::read_to_string(path).map_or_else(|_| Memory::default(), |text| Memory::parse(&text))
    }

    /// Writes the memory to `path` through a temporary file and a rename.
    ///
    /// # Errors
    ///
    /// When the folder or the file can't be written.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        write_atomic(path, &self.text())
    }
}

/// Writes `text` to `path` through a temporary file beside it and a rename, so a reader never
/// sees half a file.
///
/// # Errors
///
/// When the folder or the file can't be written.
pub fn write_atomic(path: &Path, text: &str) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("new");
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path)
}

/// The state folder: `$XDG_STATE_HOME/kubaos/power`, else `~/.local/state/kubaos/power`.
pub fn dir() -> Option<PathBuf> {
    env::var_os("XDG_STATE_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .filter(|home| !home.is_empty())
                .map(|home| PathBuf::from(home).join(".local/state"))
        })
        .map(|state| state.join("kubaos/power"))
}

/// The memory's file.
pub fn path() -> Option<PathBuf> {
    dir().map(|dir| dir.join(FILE))
}

#[cfg(test)]
mod tests;
