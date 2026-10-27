//! What the service tells the shell and the menu, in `$XDG_RUNTIME_DIR/kubaos-power.json`,
//! replaced whole on every change, and the socket commands reach the service on.
//!
//! ```json
//! {"stop":"super-saver","index":0,"title":"Super Saver","source":"battery",
//!  "dropped_from":null,"quiet":true,"pid":4242}
//! ```
//!
//! `config/quickshell/services/Power.qml` watches the file: the slider, the bar's icon, and the
//! system monitor's slower pace while `quiet`. A missing file means no service runs, and so does a
//! `pid` that is no longer the service's: the file of one killed outright stays behind.
//!
//! # Owns
//!
//! - [`Published`], its file, and the socket's path.
//!
//! # Not here
//!
//! - The socket's lines: `main.rs`.

use std::env;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::stop::Source;
use crate::stop::Stop;

/// The file in `$XDG_RUNTIME_DIR`.
pub const FILE: &str = "kubaos-power.json";
/// The service's socket in `$XDG_RUNTIME_DIR`.
pub const SOCKET: &str = "kubaos-power.sock";

/// What the shell reads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Published {
    pub stop: String,
    pub index: usize,
    pub title: String,
    pub source: String,
    pub dropped_from: Option<String>,
    /// Background work waits: the system monitor samples less often.
    pub quiet: bool,
    /// The service's process.
    #[serde(default)]
    pub pid: u32,
    /// The knobs the kernel refused for this stop, as the helper's arguments: `aspm=powersupersave`.
    #[serde(default)]
    pub refused: Vec<String>,
}

impl Published {
    pub fn new(stop: Stop, source: Source, dropped_from: Option<Stop>, quiet: bool) -> Published {
        Published {
            stop: stop.id().to_owned(),
            index: stop.index(),
            title: stop.title().to_owned(),
            source: source.key().to_owned(),
            dropped_from: dropped_from.map(|stop| stop.id().to_owned()),
            quiet,
            pid: std::process::id(),
            refused: Vec::new(),
        }
    }

    /// The file's text.
    ///
    /// # Panics
    ///
    /// Never: the JSON is strings, a number and booleans, which always serialise.
    pub fn text(&self) -> String {
        serde_json::to_string(self).expect("plain JSON") + "\n"
    }

    /// The file's contents, `None` when missing or unreadable.
    pub fn load(path: &Path) -> Option<Published> {
        serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
    }

    /// The stop it names.
    pub fn current(&self) -> Option<Stop> {
        Stop::from_id(&self.stop)
    }
}

/// `$XDG_RUNTIME_DIR`, `None` without it.
pub fn dir() -> Option<PathBuf> {
    env::var_os("XDG_RUNTIME_DIR")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
}

pub fn file() -> Option<PathBuf> {
    dir().map(|dir| dir.join(FILE))
}

pub fn socket() -> Option<PathBuf> {
    dir().map(|dir| dir.join(SOCKET))
}
