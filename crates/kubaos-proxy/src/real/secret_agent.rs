//! [`Agent`] on the system bus: NetworkManager's `SecretAgent` interface on a bus connection of its
//! own, registered with NetworkManager's agent manager while one activation of the VPN takes.
//!
//! # Owns
//!
//! - [`Registered`]: the agent registered, until it is dropped.
//!
//! # Not here
//!
//! - Which requests get the secrets: [`crate::agent`].

#![expect(
    clippy::used_underscore_binding,
    reason = "The interface's methods take every argument of NetworkManager's calls, and the code zbus writes \
              around them reads the ones the agent ignores."
)]

use std::collections::HashMap;

use zbus::blocking::Connection as Bus;
use zbus::message::Header;
use zbus::names::BusName;
use zbus::zvariant::OwnedObjectPath;
use zbus::zvariant::OwnedValue;
use zbus::zvariant::Value;

use super::NM;
use super::refused;
use super::settings;
use super::unreachable;
use crate::agent::Agent;
use crate::agent::Answer;
use crate::agent::Request;
use crate::machine::TunnelError;

const AGENT_MANAGER: &str = "org.freedesktop.NetworkManager.AgentManager";
const AGENT_MANAGER_PATH: &str = "/org/freedesktop/NetworkManager/AgentManager";
/// Where NetworkManager calls every agent, by its API.
const AGENT_PATH: &str = "/org/freedesktop/NetworkManager/SecretAgent";
/// The agent's identifier in NetworkManager, which takes one agent of an identifier per user. The
/// session lock keeps a second `kubaos-proxy` from bringing the VPN up at the same time.
const IDENTIFIER: &str = "org.kubaos.proxy";

/// A connection's settings as they come over the bus (`a{sa{sv}}`).
type Settings = HashMap<String, HashMap<String, OwnedValue>>;

/// The agent registered with NetworkManager; dropping it unregisters it and closes its bus
/// connection, and NetworkManager forgets an agent whose connection goes in any case.
#[derive(Debug)]
pub struct Registered {
    bus: Bus,
}

impl Registered {
    /// Serves `agent` and registers it with NetworkManager.
    ///
    /// # Errors
    ///
    /// The system bus couldn't be reached, or NetworkManager refused the agent.
    pub fn new(agent: Agent) -> Result<Registered, TunnelError> {
        let bus = Bus::system().map_err(unreachable)?;
        bus.object_server()
            .at(AGENT_PATH, SecretAgent { agent })
            .map_err(unreachable)?;
        bus.call_method(
            Some(NM),
            AGENT_MANAGER_PATH,
            Some(AGENT_MANAGER),
            "Register",
            &(IDENTIFIER,),
        )
        .map_err(refused)?;
        Ok(Registered { bus })
    }
}

impl Drop for Registered {
    fn drop(&mut self) {
        let unregistered = self
            .bus
            .call_method(Some(NM), AGENT_MANAGER_PATH, Some(AGENT_MANAGER), "Unregister", &());
        if let Err(err) = unregistered {
            log::debug!("unregistering the VPN's secret agent: {err}");
        }
    }
}

/// NetworkManager's name for a request that gets nothing; it then asks the next agent.
#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "org.freedesktop.NetworkManager.SecretAgent")]
enum AgentError {
    #[zbus(error)]
    ZBus(zbus::Error),
    NoSecrets(String),
}

struct SecretAgent {
    agent: Agent,
}

#[zbus::interface(name = "org.freedesktop.NetworkManager.SecretAgent")]
impl SecretAgent {
    #[expect(
        clippy::too_many_arguments,
        reason = "NetworkManager's call has five arguments, and zbus passes the header and the bus beside them."
    )]
    async fn get_secrets(
        &self,
        connection: Settings,
        _connection_path: OwnedObjectPath,
        setting_name: String,
        _hints: Vec<String>,
        _flags: u32,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] bus: &zbus::Connection,
    ) -> Result<HashMap<String, HashMap<String, Value<'static>>>, AgentError> {
        let request = Request {
            from_network_manager: from_network_manager(bus, &header).await,
            id: text(&connection, "connection", "id"),
            uuid: text(&connection, "connection", "uuid"),
            setting: setting_name,
        };
        match self.agent.answer(&request) {
            Answer::Keys(keys) => {
                log::info!("gave NetworkManager the VPN's secrets for {}", request.setting);
                Ok(settings(&keys))
            }
            Answer::Nothing(refusal) => {
                log::info!("gave no secrets to a request for {:?}: {refusal}", request.id);
                Err(AgentError::NoSecrets(refusal.to_string()))
            }
        }
    }

    /// The agent answers at once, so there is never a request to cancel.
    #[expect(
        clippy::unused_self,
        reason = "NetworkManager calls it with these arguments, and zbus calls it on the object."
    )]
    fn cancel_get_secrets(&self, _connection_path: OwnedObjectPath, _setting_name: String) {}

    /// The secrets are saved nowhere, so NetworkManager's request to save them is answered with
    /// nothing done.
    #[expect(
        clippy::unused_self,
        reason = "NetworkManager calls it with these arguments, and zbus calls it on the object."
    )]
    fn save_secrets(&self, _connection: Settings, _connection_path: OwnedObjectPath) {}

    /// The secrets are kept nowhere, so there is nothing to delete.
    #[expect(
        clippy::unused_self,
        reason = "NetworkManager calls it with these arguments, and zbus calls it on the object."
    )]
    fn delete_secrets(&self, _connection: Settings, _connection_path: OwnedObjectPath) {}
}

/// Whether the call came from NetworkManager: the bus lets only root own its name.
async fn from_network_manager(bus: &zbus::Connection, header: &Header<'_>) -> bool {
    let Some(sender) = header.sender() else {
        return false;
    };
    let Ok(dbus) = zbus::fdo::DBusProxy::new(bus).await else {
        return false;
    };
    let Ok(name) = BusName::try_from(NM) else {
        return false;
    };
    dbus.get_name_owner(name)
        .await
        .is_ok_and(|owner| owner.as_str() == sender.as_str())
}

fn text(connection: &Settings, setting: &str, key: &str) -> Option<String> {
    match &**connection.get(setting)?.get(key)? {
        Value::Str(s) => Some(s.to_string()),
        _ => None,
    }
}
