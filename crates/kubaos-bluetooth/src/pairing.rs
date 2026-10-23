//! Pairing a device the user picked in the shell, so the attempt always ends: in a paired,
//! trusted and connected device, or in a short reason and a device BlueZ forgets again.
//!
//! bluetoothd suspends discovery for as long as a pairing runs and resumes it only when the
//! pairing ends, so a pairing that never ends leaves the adapter unable to scan. The timeout here
//! calls `CancelPairing` to end it.
//!
//! # Owns
//!
//! - [`Pairing`]: the attempt, its timeout, what it does on success, and the cleanup after a
//!   failure.
//! - [`describe`]: BlueZ's errors as the shell shows them.
//!
//! # Not here
//!
//! - The calls themselves: [`crate::bluez`].
//! - The state the shell reads: [`crate::hub`].

use std::sync::Arc;
use std::time::Duration;

use async_io::Timer;
use futures_lite::future;
use kotodama_extension::log;

use crate::bluez::Bluez;
use crate::bluez::BluezError;
use crate::hub::Hub;
use crate::state::Sink;

/// How long a pairing may take before it's cancelled. Typing a passkey on a keyboard takes
/// a few seconds, and bluetoothd gives up on an unanswered agent request after 60 s, so a
/// pairing still running past both is stuck.
pub const DEFAULT_PAIR_TIMEOUT: Duration = Duration::from_secs(90);

/// How long a device whose pairing failed keeps its row and reason before BlueZ forgets it: 30 s,
/// BlueZ's own `TemporaryTimeout` for a device it found and didn't pair. `Pair` makes a device
/// permanent before it tries, so without this a failed device stays listed for good.
pub const DEFAULT_LINGER: Duration = Duration::from_secs(30);

/// What the shell shows for a pairing that ran out of time.
pub const TIMED_OUT: &str = "Timed out";
/// What the shell shows for a pairing the user cancelled.
pub const CANCELLED: &str = "Cancelled";
/// What the shell shows when bluetoothd refuses for want of a powered adapter. Only bluetoothd
/// restarting gets it out of that state when the adapter is on, and that needs root.
pub const NOT_READY: &str =
    "Bluetooth isn't responding. Restart it: sudo systemctl restart bluetooth";

/// Pairs devices, one attempt per device at a time.
#[derive(Debug)]
pub struct Pairing<S, B> {
    hub: Arc<Hub<S>>,
    bluez: Arc<B>,
    timeout: Duration,
    linger: Duration,
}

impl<S: Sink, B: Bluez> Pairing<S, B> {
    #[must_use]
    pub fn new(hub: Arc<Hub<S>>, bluez: Arc<B>) -> Self {
        Pairing {
            hub,
            bluez,
            timeout: DEFAULT_PAIR_TIMEOUT,
            linger: DEFAULT_LINGER,
        }
    }

    /// How long an attempt may take.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// How long a failed device stays before BlueZ forgets it.
    #[must_use]
    pub fn linger(mut self, linger: Duration) -> Self {
        self.linger = linger;
        self
    }

    /// Pairs `device`, an object path. A device already pairing is left to that attempt.
    pub async fn pair(&self, device: &str) {
        if !self.hub.begin_pairing(device) {
            return;
        }
        match self.attempt(device).await {
            Ok(()) => self.paired(device).await,
            Err(reason) => {
                log!("pairing {device} failed: {reason}");
                self.hub.end_pairing(device, Err(reason));
                Timer::after(self.linger).await;
                self.forget_if_unpaired(device).await;
            }
        }
    }

    /// `Pair`, cancelled once it takes longer than the timeout.
    async fn attempt(&self, device: &str) -> Result<(), String> {
        let paired = async { Some(self.bluez.pair(device).await) };
        let timed_out = async {
            Timer::after(self.timeout).await;
            None
        };
        match future::or(paired, timed_out).await {
            Some(Ok(())) => Ok(()),
            Some(Err(err)) if err.is("AlreadyExists") => Ok(()),
            Some(Err(err)) => Err(describe(&err)),
            None => {
                if let Err(err) = self.bluez.cancel_pairing(device).await {
                    log!("cancelling the pairing of {device} after the timeout: {err}");
                }
                Err(TIMED_OUT.to_owned())
            }
        }
    }

    /// Trusts the new device, so it reconnects on its own, then connects it, as a keyboard
    /// isn't usable until it is.
    async fn paired(&self, device: &str) {
        if let Err(err) = self.bluez.set_trusted(device).await {
            log!("trusting {device}: {err}");
        }
        self.hub.end_pairing(device, Ok(()));
        match self.bluez.connect(device).await {
            Ok(()) => {}
            Err(err) if err.is("AlreadyConnected") => {}
            Err(err) => log!("connecting {device} after pairing it: {err}"),
        }
    }

    /// Removes `device` once its failure has been shown, unless it got paired or is being paired
    /// again meanwhile, so BlueZ treats it as any device it found: listed while it's around.
    async fn forget_if_unpaired(&self, device: &str) {
        if self.hub.is_pairing(device) {
            return;
        }
        match self.bluez.paired(device).await {
            Ok(false) => {
                if let Err(err) = self.bluez.remove(device).await {
                    log!("forgetting {device} after its pairing failed: {err}");
                }
            }
            Ok(true) => {}
            Err(err) => log!("reading whether {device} is paired: {err}"),
        }
        if !self.hub.is_pairing(device) {
            self.hub.clear_error(device);
        }
    }

    /// Cancels the pairing of `device`. When bluetoothd has no pairing of it to cancel, the
    /// attempt is stuck short of bluetoothd, and ends here instead.
    pub async fn cancel(&self, device: &str) {
        if let Err(err) = self.bluez.cancel_pairing(device).await {
            log!("cancelling the pairing of {device}: {err}");
            if self.hub.is_pairing(device) {
                self.hub.end_pairing(device, Err(CANCELLED.to_owned()));
            }
        }
    }
}

/// `err` as the shell shows it under the device: a few words, and what to do when there's
/// something to do.
pub fn describe(err: &BluezError) -> String {
    let reason = match err.short_name() {
        "AuthenticationFailed" => "Wrong code, or the device said no",
        "AuthenticationCanceled" => CANCELLED,
        "AuthenticationRejected" => "The device said no",
        "AuthenticationTimeout" | "Timeout" | "NoReply" => TIMED_OUT,
        "ConnectionAttemptFailed" => "Couldn't reach it. Is it in pairing mode?",
        "NotReady" => NOT_READY,
        "InProgress" => "Already pairing",
        "DoesNotExist" | "UnknownObject" => "It's gone. Scan again",
        _ if err.message.is_empty() => "Pairing failed",
        _ => return format!("Pairing failed: {}", err.message),
    };
    reason.to_owned()
}

#[cfg(test)]
mod tests;
