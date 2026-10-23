//! What the shell shows of pairing: the agent's open prompt, the devices being paired, and the
//! last failure of each, written as JSON to a file the shell watches.
//!
//! A file, not a call into the shell (`qs ipc call`): the agent never waits on the shell or
//! depends on it running, a shell that restarts mid-pairing reads the prompt back, and Quickshell's
//! `FileView` follows the file without polling. The shell answers through the session bus
//! (`crate::service`).
//!
//! # Owns
//!
//! - [`State`] and its JSON, which `config/quickshell/services/Bt.qml` reads.
//! - [`Sink`], and [`StateFile`], which writes the state atomically.
//!
//! # Not here
//!
//! - Changing the state: [`crate::hub`].

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

/// The file's name in `$XDG_RUNTIME_DIR`. Straight in it, not in a folder of its own, so the
/// shell's watch has a folder to follow before the agent first writes.
pub const FILE_NAME: &str = "kubaos-bluetooth.json";

/// Everything the shell shows of pairing.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct State {
    /// Whether bluetoothd has the agent registered as its default one.
    pub agent: bool,
    pub prompt: Option<Prompt>,
    /// The object paths of the devices being paired.
    pub pairing: BTreeSet<String>,
    /// The last pairing failure of each device, by object path, as the shell shows it.
    pub errors: BTreeMap<String, String>,
}

/// What the agent asks or shows the user, for one device.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Prompt {
    /// Names the prompt in the shell's answer, so a late answer can't reach the next one.
    pub id: u32,
    pub kind: Kind,
    /// The device's object path.
    pub device: String,
    /// The device's alias, or its address without one.
    pub name: String,
    /// The PIN or passkey to show or confirm, zero-padded to six digits for a passkey.
    pub code: Option<String>,
    /// How many digits of the passkey the device has had typed, when it reports them.
    pub entered: Option<u16>,
}

/// The kinds of prompt, one per question the agent can be asked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// Type the passkey on the device, then Enter.
    DisplayPasskey,
    /// Type the PIN on the device, then Enter (keyboards older than Bluetooth 2.1).
    DisplayPinCode,
    /// Does the passkey match the one the device shows?
    Confirm,
    /// Allow the device's own pairing request?
    Authorize,
    /// Enter the passkey the device shows.
    Passkey,
    /// Enter the PIN the device shows or expects.
    PinCode,
}

impl Kind {
    /// Whether the prompt only shows a code, so nothing in BlueZ waits on its answer.
    pub fn is_display(self) -> bool {
        matches!(self, Kind::DisplayPasskey | Kind::DisplayPinCode)
    }
}

/// Where each new state goes.
pub trait Sink: Send + Sync {
    fn publish(&self, state: &State);
}

/// The state file in `$XDG_RUNTIME_DIR`.
#[derive(Debug)]
pub struct StateFile {
    path: PathBuf,
}

impl StateFile {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        StateFile { path: path.into() }
    }

    /// The file in `$XDG_RUNTIME_DIR`, or `None` without one.
    #[must_use]
    pub fn in_runtime_dir() -> Option<Self> {
        let dir = std::env::var_os("XDG_RUNTIME_DIR")?;
        Some(StateFile::new(Path::new(&dir).join(FILE_NAME)))
    }

    /// Writes `state` beside the file, then renames it over, so the shell never reads half a state.
    ///
    /// # Errors
    ///
    /// When the file can't be written.
    pub fn write(&self, state: &State) -> io::Result<()> {
        let json = serde_json::to_vec(state).map_err(io::Error::other)?;
        let mut partial = self.path.clone().into_os_string();
        partial.push(".partial");
        fs::write(&partial, json)?;
        fs::rename(&partial, &self.path)
    }
}

impl Sink for StateFile {
    fn publish(&self, state: &State) {
        if let Err(err) = self.write(state) {
            kotodama_extension::log!("writing {}: {err}", self.path.display());
        }
    }
}

#[cfg(test)]
mod tests;
