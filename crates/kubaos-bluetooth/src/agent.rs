//! How the agent answers bluetoothd: each `org.bluez.Agent1` request as a prompt in the shell,
//! and the user's answer, or its absence, as the reply.
//!
//! The interface: <https://github.com/bluez/bluez/blob/master/doc/org.bluez.Agent.rst>
//!
//! # Owns
//!
//! - [`Agent`]: what each request shows and waits for, how long it waits, and when it says yes
//!   without asking.
//! - [`Refusal`], the two errors bluetoothd takes back.
//!
//! # Not here
//!
//! - The D-Bus object bluetoothd calls, and reading a device's name: [`crate::bluez`].
//! - The prompt itself: [`crate::hub`].

use std::sync::Arc;
use std::time::Duration;

use async_io::Timer;
use futures_lite::future;

use crate::hub::Device;
use crate::hub::Hub;
use crate::state::Kind;
use crate::state::Sink;

/// How long a prompt waits for the user. Under bluetoothd's own 60 s limit on an agent request,
/// so the agent says it was cancelled before bluetoothd gives up on it, and the prompt never
/// outlives the request.
pub const DEFAULT_PROMPT_TIMEOUT: Duration = Duration::from_secs(50);

/// The longest PIN BlueZ takes.
const MAX_PIN: usize = 16;
/// The largest passkey: six digits.
const MAX_PASSKEY: u32 = 999_999;

/// Why the agent turned a request down, as bluetoothd's errors name it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The user said no, or typed something that isn't a PIN or passkey.
    Rejected,
    /// Nobody answered: the prompt timed out, or bluetoothd or the next request closed it.
    Canceled,
}

/// The agent's side of each request.
#[derive(Debug)]
pub struct Agent<S> {
    hub: Arc<Hub<S>>,
    timeout: Duration,
}

impl<S: Sink> Agent<S> {
    #[must_use]
    pub fn new(hub: Arc<Hub<S>>) -> Self {
        Agent {
            hub,
            timeout: DEFAULT_PROMPT_TIMEOUT,
        }
    }

    /// How long a prompt waits for the user.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Asks the user, and returns what they typed once they accept.
    async fn wait(
        &self,
        kind: Kind,
        device: &Device,
        code: Option<String>,
    ) -> Result<String, Refusal> {
        let (id, reply) = self.hub.ask(kind, device, code);
        let answered = async { reply.recv().await.ok() };
        let timed_out = async {
            Timer::after(self.timeout).await;
            None
        };
        match future::or(answered, timed_out).await {
            Some(answer) if answer.accept => Ok(answer.value),
            Some(_) => Err(Refusal::Rejected),
            None => {
                self.hub.close(id);
                Err(Refusal::Canceled)
            }
        }
    }

    /// `RequestPinCode`: the PIN the user types, 1 to 16 letters or digits.
    ///
    /// # Errors
    ///
    /// When the user cancels or types no PIN, or nobody answers.
    pub async fn request_pin_code(&self, device: &Device) -> Result<String, Refusal> {
        let pin = self.wait(Kind::PinCode, device, None).await?;
        let pin = pin.trim();
        if pin.is_empty() || pin.len() > MAX_PIN || !pin.chars().all(|c| c.is_ascii_alphanumeric())
        {
            return Err(Refusal::Rejected);
        }
        Ok(pin.to_owned())
    }

    /// `RequestPasskey`: the passkey the user types, up to six digits.
    ///
    /// # Errors
    ///
    /// When the user cancels or types no passkey, or nobody answers.
    pub async fn request_passkey(&self, device: &Device) -> Result<u32, Refusal> {
        let typed = self.wait(Kind::Passkey, device, None).await?;
        let typed = typed.trim();
        if typed.is_empty() || !typed.chars().all(|c| c.is_ascii_digit()) {
            return Err(Refusal::Rejected);
        }
        match typed.parse() {
            Ok(passkey) if passkey <= MAX_PASSKEY => Ok(passkey),
            _ => Err(Refusal::Rejected),
        }
    }

    /// `DisplayPinCode`: shows the PIN to type on the device. bluetoothd waits for the device,
    /// not for this reply, and calls `Cancel` once the PIN is no longer needed.
    pub fn display_pin_code(&self, device: &Device, pin: &str) {
        self.hub
            .show(Kind::DisplayPinCode, device, pin.to_owned(), None);
    }

    /// `DisplayPasskey`: shows the passkey to type on the device, with how much of it was typed.
    /// Called again as the user types.
    pub fn display_passkey(&self, device: &Device, passkey: u32, entered: u16) {
        self.hub.show(
            Kind::DisplayPasskey,
            device,
            passkey_code(passkey),
            Some(entered),
        );
    }

    /// `RequestConfirmation`: asks whether the passkey matches the one the device shows. Asked
    /// even for a device the user chose to pair, since the match is what proves it's that device.
    ///
    /// # Errors
    ///
    /// When the user says no, or nobody answers.
    pub async fn request_confirmation(&self, device: &Device, passkey: u32) -> Result<(), Refusal> {
        self.wait(Kind::Confirm, device, Some(passkey_code(passkey)))
            .await
            .map(drop)
    }

    /// `RequestAuthorization`: a device asks to pair with no code. A device the user is pairing
    /// from the shell needs no second yes.
    ///
    /// # Errors
    ///
    /// When the user says no, or nobody answers.
    pub async fn request_authorization(&self, device: &Device) -> Result<(), Refusal> {
        if self.hub.is_pairing(&device.path) {
            return Ok(());
        }
        self.wait(Kind::Authorize, device, None).await.map(drop)
    }

    /// `AuthorizeService`: a device connects to a service. Allowed for a trusted device and for
    /// the one being paired, which the pairing trusts once it's done; any other is turned down
    /// without asking, as a device that isn't trusted shouldn't open a profile on its own.
    ///
    /// # Errors
    ///
    /// When the device is neither trusted nor being paired.
    pub fn authorize_service(&self, device: &str, trusted: bool) -> Result<(), Refusal> {
        if trusted || self.hub.is_pairing(device) {
            Ok(())
        } else {
            Err(Refusal::Rejected)
        }
    }

    /// `Cancel`: bluetoothd gave up on the request; the prompt goes.
    pub fn cancel(&self) {
        self.hub.dismiss();
    }

    /// `Release`: bluetoothd let the agent go, as when it stops.
    pub fn release(&self) {
        self.hub.set_agent(false);
    }
}

/// A passkey as the device shows it: six digits, zero-padded.
pub fn passkey_code(passkey: u32) -> String {
    format!("{passkey:06}")
}

#[cfg(test)]
mod tests;
