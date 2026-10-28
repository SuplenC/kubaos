//! The pairing state the agent, the pairings and the shell's answers share, and the one open
//! prompt's way back to whoever waits on it.
//!
//! # Owns
//!
//! - [`Hub`]: opening, answering and closing the prompt, which devices are pairing or failed, and
//!   how strongly each device was last heard. Every change goes to the [`Sink`] as it happens.
//!
//! # Not here
//!
//! - What a prompt asks and how its answer becomes BlueZ's reply: [`crate::agent`].
//! - Running a pairing: [`crate::pairing`].

use std::sync::Mutex;
use std::sync::MutexGuard;

use crate::state::Kind;
use crate::state::Prompt;
use crate::state::Sink;
use crate::state::State;

/// How far a device's RSSI moves, in dBm, from one advertisement to the next while it stays put.
/// A change under it is noise: publishing it would rewrite the state file many times a second
/// during a scan, and the shell orders devices only by steps of 15 dBm.
pub const RSSI_JITTER: i16 = 6;

/// The user's answer to a prompt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    /// Pair or Cancel.
    pub accept: bool,
    /// The PIN or passkey typed, for the prompts that ask for one; empty otherwise.
    pub value: String,
}

impl Answer {
    #[must_use]
    pub fn accept(value: impl Into<String>) -> Self {
        Answer {
            accept: true,
            value: value.into(),
        }
    }

    #[must_use]
    pub fn reject() -> Self {
        Answer {
            accept: false,
            value: String::new(),
        }
    }
}

/// A device as a prompt names it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device {
    /// Its object path, as `/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF`.
    pub path: String,
    /// Its alias, or its address without one.
    pub name: String,
}

/// The answer named a prompt that is no longer open.
#[derive(Debug, thiserror::Error)]
#[error("There's no prompt {0} to answer: it was answered, cancelled or timed out")]
pub struct NoPrompt(u32);

/// The shared pairing state. Each method holds the lock for itself alone, publishing included,
/// so the shell sees the changes in the order they were made.
///
/// Every method panics if the lock is poisoned, which takes a panic inside another method first.
#[derive(Debug)]
pub struct Hub<S> {
    sink: S,
    inner: Mutex<Inner>,
}

#[derive(Debug, Default)]
struct Inner {
    state: State,
    /// The last prompt id given out.
    last_id: u32,
    /// Where the open prompt's answer goes, when something waits on it. Dropping it tells the
    /// waiter the prompt was cancelled.
    reply: Option<async_channel::Sender<Answer>>,
}

impl Inner {
    fn next_id(&mut self) -> u32 {
        self.last_id = self.last_id.wrapping_add(1).max(1);
        self.last_id
    }

    fn clear_prompt(&mut self) {
        self.state.prompt = None;
        self.reply = None;
    }
}

impl<S: Sink> Hub<S> {
    #[must_use]
    pub fn new(sink: S) -> Self {
        Hub {
            sink,
            inner: Mutex::new(Inner::default()),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().expect("pairing state mutex poisoned")
    }

    /// Applies `change`, then publishes the state if it changed.
    fn change<R>(&self, change: impl FnOnce(&mut Inner) -> R) -> R {
        let mut inner = self.lock();
        let before = inner.state.clone();
        let result = change(&mut inner);
        if inner.state != before {
            self.sink.publish(&inner.state);
        }
        result
    }

    #[cfg(test)]
    pub fn sink(&self) -> &S {
        &self.sink
    }

    /// The state as it is.
    pub fn state(&self) -> State {
        self.lock().state.clone()
    }

    /// Publishes the state as it is, as when the agent starts.
    pub fn publish(&self) {
        self.sink.publish(&self.lock().state);
    }

    /// Opens a prompt that waits on the user, and returns its id and where its answer comes. It
    /// replaces any open prompt: bluetoothd asks one thing at a time, so a new question means it
    /// gave up on the old one, whose waiter is told it was cancelled.
    pub fn ask(&self, kind: Kind, device: &Device, code: Option<String>) -> (u32, async_channel::Receiver<Answer>) {
        let (sender, receiver) = async_channel::bounded(1);
        let id = self.change(|inner| {
            let id = inner.next_id();
            inner.state.prompt = Some(prompt(id, kind, device, code, None));
            inner.reply = Some(sender);
            id
        });
        (id, receiver)
    }

    /// Shows a code until the pairing ends or bluetoothd cancels it, and returns the prompt's id.
    /// The same code for the same device again only updates how much of it was typed.
    pub fn show(&self, kind: Kind, device: &Device, code: String, entered: Option<u16>) -> u32 {
        self.change(|inner| {
            if let Some(open) = &mut inner.state.prompt
                && open.kind == kind
                && open.device == device.path
                && open.code.as_deref() == Some(code.as_str())
            {
                open.entered = entered;
                return open.id;
            }
            let id = inner.next_id();
            inner.state.prompt = Some(prompt(id, kind, device, Some(code), entered));
            inner.reply = None;
            id
        })
    }

    /// Closes the prompt `id` with `answer`, passing it to whoever waits, and returns the prompt.
    ///
    /// # Errors
    ///
    /// When `id` isn't the open prompt.
    pub fn answer(&self, id: u32, answer: Answer) -> Result<Prompt, NoPrompt> {
        self.change(|inner| {
            if inner.state.prompt.as_ref().is_none_or(|open| open.id != id) {
                return Err(NoPrompt(id));
            }
            let reply = inner.reply.take();
            let closed = inner.state.prompt.take().ok_or(NoPrompt(id))?;
            if let Some(reply) = reply {
                // The waiter may have timed out in the meantime; then nobody needs the answer.
                let _ = reply.try_send(answer);
            }
            Ok(closed)
        })
    }

    /// Closes the prompt `id` if it is still open, as when its wait timed out.
    pub fn close(&self, id: u32) {
        self.change(|inner| {
            if inner.state.prompt.as_ref().is_some_and(|open| open.id == id) {
                inner.clear_prompt();
            }
        });
    }

    /// Closes whatever prompt is open: bluetoothd cancelled the request or let the agent go.
    pub fn dismiss(&self) {
        self.change(Inner::clear_prompt);
    }

    /// Marks `device` as pairing and forgets its last failure. Returns false when it already was.
    pub fn begin_pairing(&self, device: &str) -> bool {
        self.change(|inner| {
            inner.state.errors.remove(device);
            inner.state.pairing.insert(device.to_owned())
        })
    }

    /// Ends the pairing of `device`, keeping `result`'s failure for the shell, and closes the
    /// device's prompt.
    pub fn end_pairing(&self, device: &str, result: Result<(), String>) {
        self.change(|inner| {
            inner.state.pairing.remove(device);
            if let Err(message) = result {
                inner.state.errors.insert(device.to_owned(), message);
            }
            if inner.state.prompt.as_ref().is_some_and(|open| open.device == device) {
                inner.clear_prompt();
            }
        });
    }

    /// Forgets the last failure of `device`.
    pub fn clear_error(&self, device: &str) {
        self.change(|inner| {
            inner.state.errors.remove(device);
        });
    }

    /// Records that `device` was heard at `rssi` dBm, unless that is within [`RSSI_JITTER`] of what
    /// was last recorded.
    pub fn heard(&self, device: &str, rssi: i16) {
        self.change(|inner| {
            let last = inner.state.rssi.get(device).copied();
            if last.is_none_or(|last| (rssi - last).abs() >= RSSI_JITTER) {
                inner.state.rssi.insert(device.to_owned(), rssi);
            }
        });
    }

    /// Forgets the signal of `device`, which bluetoothd no longer has.
    pub fn gone(&self, device: &str) {
        self.change(|inner| {
            inner.state.rssi.remove(device);
        });
    }

    pub fn is_pairing(&self, device: &str) -> bool {
        self.lock().state.pairing.contains(device)
    }

    /// Records whether bluetoothd has the agent. Without it no prompt can be open.
    pub fn set_agent(&self, registered: bool) {
        self.change(|inner| {
            inner.state.agent = registered;
            if !registered {
                inner.clear_prompt();
            }
        });
    }
}

fn prompt(id: u32, kind: Kind, device: &Device, code: Option<String>, entered: Option<u16>) -> Prompt {
    Prompt {
        id,
        kind,
        device: device.path.clone(),
        name: device.name.clone(),
        code,
        entered,
    }
}

#[cfg(test)]
mod tests;
