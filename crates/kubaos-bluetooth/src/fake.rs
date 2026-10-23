//! Stand-ins for the shell's state file and for bluetoothd, for the unit tests.

use std::collections::VecDeque;
use std::sync::Mutex;

use crate::bluez::Bluez;
use crate::bluez::BluezError;
use crate::hub::Device;
use crate::state::Sink;
use crate::state::State;

/// The keyboard the tests pair.
pub fn keyboard() -> Device {
    Device {
        path: "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".into(),
        name: "K380".into(),
    }
}

/// Keeps every state published.
#[derive(Debug, Default)]
pub struct Recorder {
    states: Mutex<Vec<State>>,
}

impl Recorder {
    pub fn states(&self) -> Vec<State> {
        self.states.lock().unwrap().clone()
    }
}

impl Sink for Recorder {
    fn publish(&self, state: &State) {
        self.states.lock().unwrap().push(state.clone());
    }
}

/// How `Pair` answers.
#[derive(Clone, Debug)]
pub enum PairReply {
    Ok,
    Err(BluezError),
    /// Never: the pairing hangs, as it does with no agent and a keyboard waiting for a passkey.
    Never,
}

/// bluetoothd, answering from a script and logging each call as `Method path`.
#[derive(Debug)]
pub struct FakeBluez {
    pub pair: PairReply,
    pub paired: bool,
    /// What `CancelPairing` answers.
    pub cancel: Result<(), BluezError>,
    /// What each `StartDiscovery` answers, in turn; Ok once they run out.
    pub start: Mutex<VecDeque<Result<(), BluezError>>>,
    pub calls: Mutex<Vec<String>>,
}

impl Default for FakeBluez {
    fn default() -> Self {
        FakeBluez {
            pair: PairReply::Ok,
            paired: false,
            cancel: Ok(()),
            start: Mutex::default(),
            calls: Mutex::default(),
        }
    }
}

impl FakeBluez {
    pub fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    fn log(&self, method: &str, path: &str) {
        self.calls.lock().unwrap().push(format!("{method} {path}"));
    }
}

impl Bluez for FakeBluez {
    async fn pair(&self, device: &str) -> Result<(), BluezError> {
        self.log("Pair", device);
        match &self.pair {
            PairReply::Ok => Ok(()),
            PairReply::Err(err) => Err(err.clone()),
            PairReply::Never => futures_lite::future::pending().await,
        }
    }

    async fn cancel_pairing(&self, device: &str) -> Result<(), BluezError> {
        self.log("CancelPairing", device);
        self.cancel.clone()
    }

    async fn set_trusted(&self, device: &str) -> Result<(), BluezError> {
        self.log("Trust", device);
        Ok(())
    }

    async fn connect(&self, device: &str) -> Result<(), BluezError> {
        self.log("Connect", device);
        Ok(())
    }

    async fn paired(&self, device: &str) -> Result<bool, BluezError> {
        self.log("Paired?", device);
        Ok(self.paired)
    }

    async fn remove(&self, device: &str) -> Result<(), BluezError> {
        self.log("RemoveDevice", device);
        Ok(())
    }

    async fn start_discovery(&self, adapter: &str) -> Result<(), BluezError> {
        self.log("StartDiscovery", adapter);
        self.start.lock().unwrap().pop_front().unwrap_or(Ok(()))
    }

    async fn stop_discovery(&self, adapter: &str) -> Result<(), BluezError> {
        self.log("StopDiscovery", adapter);
        Err(BluezError::new(
            "org.bluez.Error.Failed",
            "No discovery started",
        ))
    }
}
