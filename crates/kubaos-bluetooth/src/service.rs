//! The agent's own interface on the session bus, which the shell calls through the
//! `kubaos-bluetooth` command, and the daemon that serves it next to the agent.
//!
//! The session bus, not a socket of its own: it already checks the caller is the user, names the
//! agent so a second copy can't start, and reports a missing agent as such.
//!
//! # Owns
//!
//! - [`Service`], `org.kubaos.Bluetooth1`: pair, cancel a pairing, answer the prompt, scan.
//! - [`call`], the client side, and [`run`], which puts the agent and the service on their buses.
//!
//! # Not here
//!
//! - Pairing, the agent's answers and scanning: [`crate::pairing`], [`crate::agent`],
//!   [`crate::scan`].
//! - The unit that runs [`run`]: `home/bluetooth.nix`.

use std::sync::Arc;
use std::time::Duration;

use zbus::fdo;
use zbus::zvariant::DynamicType;
use zbus::zvariant::OwnedObjectPath;

use crate::agent::Agent;
use crate::bluez;
use crate::bluez::AgentObject;
use crate::bluez::SystemBluez;
use crate::hub::Answer;
use crate::hub::Hub;
use crate::pairing::Pairing;
use crate::scan;
use crate::state::StateFile;

/// The service's bus name.
pub const NAME: &str = "org.kubaos.Bluetooth";
pub const PATH: &str = "/org/kubaos/Bluetooth";
pub const INTERFACE: &str = "org.kubaos.Bluetooth1";

/// The longest any call to bluetoothd may take before it fails as timed out. Past the pairing's
/// own timeout, which ends `Pair` first; this catches `Connect` and the rest when bluetoothd is
/// stuck, so no task waits on it for good.
const METHOD_TIMEOUT: Duration = Duration::from_mins(2);

/// What the shell can ask of the agent.
#[derive(Debug)]
pub struct Service {
    hub: Arc<Hub<StateFile>>,
    pairing: Arc<Pairing<StateFile, SystemBluez>>,
    bluez: Arc<SystemBluez>,
    /// The system bus, whose executor runs the pairings.
    system: zbus::Connection,
}

#[zbus::interface(name = "org.kubaos.Bluetooth1")]
impl Service {
    /// Starts pairing `device`, and returns at once: the state file follows the pairing.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "zbus passes a method its arguments by value"
    )]
    fn pair(&self, device: OwnedObjectPath) -> fdo::Result<()> {
        let device = bluez_object(&device)?;
        let pairing = Arc::clone(&self.pairing);
        self.system
            .executor()
            .spawn(async move { pairing.pair(&device).await }, "pair")
            .detach();
        Ok(())
    }

    async fn cancel_pairing(&self, device: OwnedObjectPath) -> fdo::Result<()> {
        let device = bluez_object(&device)?;
        self.pairing.cancel(&device).await;
        Ok(())
    }

    /// Answers the prompt `id`. Cancel on a code to type cancels its pairing, as nothing in
    /// bluetoothd waits on that prompt.
    async fn answer(&self, id: u32, accept: bool, value: String) -> fdo::Result<()> {
        let answer = if accept {
            Answer::accept(value)
        } else {
            Answer::reject()
        };
        let prompt = self
            .hub
            .answer(id, answer)
            .map_err(|err| fdo::Error::Failed(err.to_string()))?;
        if !accept && prompt.kind.is_display() {
            self.pairing.cancel(&prompt.device).await;
        }
        Ok(())
    }

    /// Starts or stops scanning on `adapter`.
    async fn scan(&self, adapter: OwnedObjectPath, on: bool) -> fdo::Result<()> {
        let adapter = bluez_object(&adapter)?;
        scan::scan(self.bluez.as_ref(), &adapter, on)
            .await
            .map_err(fdo::Error::Failed)
    }
}

/// `path` if it is one of bluetoothd's objects, so the service only ever calls those.
fn bluez_object(path: &str) -> fdo::Result<String> {
    if path.starts_with("/org/bluez/") {
        Ok(path.to_owned())
    } else {
        Err(fdo::Error::InvalidArgs(format!(
            "{path} isn't a Bluetooth adapter or device"
        )))
    }
}

/// The agent couldn't start.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    /// There's no `$XDG_RUNTIME_DIR` to put the state file in.
    #[error("XDG_RUNTIME_DIR isn't set, so there's nowhere to put the pairing state")]
    NoRuntimeDir,
    #[error("connecting to the system bus")]
    System(#[source] zbus::Error),
    #[error("taking {NAME} on the session bus")]
    Session(#[source] zbus::Error),
    #[error("following bluetoothd on the system bus")]
    Follow(#[source] zbus::Error),
}

/// Runs the agent: its object on the system bus, the service on the session bus, and the
/// registration with bluetoothd, until a bus goes.
///
/// # Errors
///
/// When either bus can't be reached, or another agent has the service's name.
pub async fn run() -> Result<(), RunError> {
    let file = StateFile::in_runtime_dir().ok_or(RunError::NoRuntimeDir)?;
    let hub = Arc::new(Hub::new(file));
    let system = zbus::connection::Builder::system()
        .map_err(RunError::System)?
        .method_timeout(METHOD_TIMEOUT)
        .build()
        .await
        .map_err(RunError::System)?;
    let bluez = Arc::new(SystemBluez::new(system.clone()));
    let agent = AgentObject::new(Agent::new(Arc::clone(&hub)), Arc::clone(&bluez));
    system
        .object_server()
        .at(bluez::AGENT_PATH, agent)
        .await
        .map_err(RunError::System)?;

    let service = Service {
        hub: Arc::clone(&hub),
        pairing: Arc::new(Pairing::new(Arc::clone(&hub), Arc::clone(&bluez))),
        bluez: Arc::clone(&bluez),
        system,
    };
    let _session = zbus::connection::Builder::session()
        .and_then(|builder| builder.name(NAME))
        .and_then(|builder| builder.serve_at(PATH, service))
        .map_err(RunError::Session)?
        .build()
        .await
        .map_err(RunError::Session)?;

    hub.publish();
    bluez::keep_registered(&bluez, &hub)
        .await
        .map_err(RunError::Follow)
}

/// The agent is unreachable or turned the call down.
#[derive(Debug, thiserror::Error)]
pub enum CallError {
    #[error("The pairing agent isn't running: systemctl --user status kubaos-bluetooth")]
    NotRunning,
    /// Its own message, as the shell shows it.
    #[error("{0}")]
    Refused(String),
    #[error("calling the pairing agent")]
    Bus(#[source] zbus::Error),
}

/// Calls `method` of the running agent with `body`.
///
/// # Errors
///
/// When there's no agent, or it refuses.
pub fn call<B>(method: &str, body: &B) -> Result<(), CallError>
where
    B: serde::Serialize + DynamicType,
{
    let conn = zbus::blocking::Connection::session().map_err(CallError::Bus)?;
    match conn.call_method(Some(NAME), PATH, Some(INTERFACE), method, body) {
        Ok(_) => Ok(()),
        Err(zbus::Error::MethodError(name, _, _))
            if matches!(
                name.as_str(),
                "org.freedesktop.DBus.Error.ServiceUnknown"
                    | "org.freedesktop.DBus.Error.NameHasNoOwner"
            ) =>
        {
            Err(CallError::NotRunning)
        }
        Err(zbus::Error::MethodError(_, message, _)) => Err(CallError::Refused(
            message.unwrap_or_else(|| "The pairing agent refused".to_owned()),
        )),
        Err(err) => Err(CallError::Bus(err)),
    }
}
