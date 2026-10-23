//! Scanning for devices on the shell's behalf, so a scan can always be started again.
//!
//! bluetoothd keeps one discovery session per D-Bus connection and refuses a second
//! `StartDiscovery` from the same connection as `InProgress`, even while it has the session's
//! discovery suspended for a pairing and shows `Discovering` false. A client that only stops what
//! it sees running, as Quickshell's adapter does, then can't scan again until it reconnects. Here
//! starting always stops first.
//!
//! # Owns
//!
//! - [`scan`]: turning the agent's discovery session on and off, and what the shell shows when
//!   bluetoothd refuses.
//!
//! # Not here
//!
//! - The calls: [`crate::bluez`].

use kotodama_extension::log;

use crate::bluez::Bluez;
use crate::pairing::NOT_READY;

/// Starts or stops scanning on `adapter`, an object path.
///
/// # Errors
///
/// When bluetoothd won't start scanning: the reason, as the shell shows it.
pub async fn scan(bluez: &impl Bluez, adapter: &str, on: bool) -> Result<(), String> {
    // Stopping a session this connection doesn't have is refused, and that's fine either way.
    if let Err(err) = bluez.stop_discovery(adapter).await
        && !on
    {
        log!("stopping the scan on {adapter}: {err}");
    }
    if !on {
        return Ok(());
    }
    bluez.start_discovery(adapter).await.map_err(|err| {
        log!("starting a scan on {adapter}: {err}");
        if err.is("NotReady") {
            NOT_READY.to_owned()
        } else if err.message.is_empty() {
            "Couldn't scan".to_owned()
        } else {
            format!("Couldn't scan: {}", err.message)
        }
    })
}

#[cfg(test)]
mod tests;
