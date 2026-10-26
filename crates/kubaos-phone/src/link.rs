//! The machine the phone link runs on, as one trait: the setting, the user unit, KDE Connect on
//! the session bus and the clipboard. [`crate::kdeconnect::KdeConnect`] is the real one; the
//! tests use `crate::fake::Fake`.
//!
//! # Owns
//!
//! - The seam ([`Link`]), what can be asked of a device ([`Action`]) and how asking fails
//!   ([`LinkError`]).
//!
//! # Not here
//!
//! - The rules on top (on before anything else, which phone): [`crate::phone`].

use std::io;
use std::time::Duration;

use thiserror::Error;

use crate::device::Device;

/// What can be asked of one device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Answer its pairing request with yes.
    Accept,
    Unpair,
    /// Send it the laptop's clipboard.
    SendClipboard,
    /// Send it a file, by its `file://` URL.
    Share(String),
    /// Ring it, even when it is on silent.
    Ring,
}

/// Why the machine didn't do what was asked.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum LinkError {
    #[error("{0} couldn't be started")]
    Spawn(String, #[source] io::Error),
    #[error("{program} failed: {message}")]
    Failed { program: String, message: String },
    #[error("{0} answered something unexpected")]
    Answer(String),
    #[error("changing the setting")]
    Setting(#[source] io::Error),
    #[error("no home folder to keep the setting in")]
    NoHome,
    #[error("{0:?} isn't a KDE Connect device id")]
    BadId(String),
}

/// The machine behind the phone link.
pub trait Link {
    /// Whether the user turned the phone link on.
    fn is_on(&self) -> bool;

    /// Turns the setting on or off; kdeconnectd is left as it is.
    ///
    /// # Errors
    ///
    /// When the setting can't be written.
    fn set_on(&mut self, on: bool) -> Result<(), LinkError>;

    /// Starts the user unit that runs kdeconnectd.
    ///
    /// # Errors
    ///
    /// When systemd refuses.
    fn start(&mut self) -> Result<(), LinkError>;

    /// Stops the user unit.
    ///
    /// # Errors
    ///
    /// When systemd refuses.
    fn stop(&mut self) -> Result<(), LinkError>;

    /// kdeconnectd owns its name on the session bus.
    fn running(&mut self) -> bool;

    /// The name KDE Connect announces this laptop by.
    ///
    /// # Errors
    ///
    /// When kdeconnectd doesn't answer.
    fn name(&mut self) -> Result<String, LinkError>;

    /// The machine's host name, the name KDE Connect starts with.
    fn hostname(&mut self) -> Option<String>;

    /// Every device KDE Connect knows.
    ///
    /// # Errors
    ///
    /// When kdeconnectd doesn't answer.
    fn devices(&mut self) -> Result<Vec<Device>, LinkError>;

    /// This laptop's IPv4 addresses on its networks.
    fn addresses(&mut self) -> Vec<String>;

    /// The plugins `device` (an id) supports, each with whether it is turned on for it.
    ///
    /// # Errors
    ///
    /// When kdeconnectd doesn't answer.
    fn plugins(&mut self, device: &str) -> Result<Vec<(String, bool)>, LinkError>;

    /// Turns `plugin` on or off for `device`. kdeconnectd keeps it in the device's config.
    ///
    /// # Errors
    ///
    /// When kdeconnectd refuses or doesn't answer.
    fn set_plugin(&mut self, device: &str, plugin: &str, on: bool) -> Result<(), LinkError>;

    /// The plugins the user allows on top of the phone link's own.
    fn extra_plugins(&self) -> Vec<String>;

    /// Asks `device` (an id) for `action`.
    ///
    /// # Errors
    ///
    /// When kdeconnectd refuses or doesn't answer.
    fn call(&mut self, device: &str, action: &Action) -> Result<(), LinkError>;

    /// Puts `text` on the laptop's clipboard.
    ///
    /// # Errors
    ///
    /// When the clipboard can't be written.
    fn copy(&mut self, text: &str) -> Result<(), LinkError>;

    /// The text on the laptop's clipboard.
    ///
    /// # Errors
    ///
    /// When the clipboard can't be read, or holds no text.
    fn paste(&mut self) -> Result<String, LinkError>;

    /// Waits, while kdeconnectd starts or a pairing settles.
    fn pause(&mut self, how_long: Duration);
}
