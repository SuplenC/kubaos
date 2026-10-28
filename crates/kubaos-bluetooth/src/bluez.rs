//! bluetoothd on the system bus: the calls the pairing and scanning make, behind [`Bluez`], and
//! the agent object bluetoothd calls back.
//!
//! The interfaces: <https://github.com/bluez/bluez/tree/master/doc> (`org.bluez.Device`,
//! `org.bluez.Adapter`, `org.bluez.AgentManager`, `org.bluez.Agent`).
//!
//! # Owns
//!
//! - [`Bluez`] and [`BluezError`], and [`SystemBluez`], which makes the calls for real.
//! - [`AgentObject`], the `org.bluez.Agent1` object, and registering it as the default agent
//!   whenever bluetoothd appears.
//!
//! # Not here
//!
//! - What the agent answers: [`crate::agent`].
//! - What a pairing does with the calls: [`crate::pairing`].

use std::io;
use std::sync::Arc;

use futures_lite::StreamExt;
use kotodama_extension::log;
use zbus::Connection;
use zbus::zvariant::DynamicType;
use zbus::zvariant::ObjectPath;
use zbus::zvariant::OwnedObjectPath;
use zbus::zvariant::OwnedValue;
use zbus::zvariant::Value;

use crate::agent::Agent;
use crate::agent::Refusal;
use crate::hub::Device;
use crate::hub::Hub;
use crate::state::StateFile;

/// bluetoothd's bus name.
pub const SERVICE: &str = "org.bluez";
/// Where the agent's object is.
pub const AGENT_PATH: &str = "/org/kubaos/bluetooth/agent";
/// What the agent can do: show a code and take one typed, so every pairing method works,
/// passkey entry on a keyboard included.
pub const CAPABILITY: &str = "KeyboardDisplay";

const DEVICE: &str = "org.bluez.Device1";
const ADAPTER: &str = "org.bluez.Adapter1";
const AGENT_MANAGER: &str = "org.bluez.AgentManager1";
const PROPERTIES: &str = "org.freedesktop.DBus.Properties";

/// A call to bluetoothd failed: the D-Bus error's name and message.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{name}: {message}")]
pub struct BluezError {
    /// As `org.bluez.Error.AuthenticationFailed`.
    pub name: String,
    pub message: String,
}

impl BluezError {
    #[must_use]
    pub fn new(name: impl Into<String>, message: impl Into<String>) -> Self {
        BluezError {
            name: name.into(),
            message: message.into(),
        }
    }

    /// The name's last part, as `AuthenticationFailed`.
    pub fn short_name(&self) -> &str {
        self.name.rsplit('.').next().unwrap_or(&self.name)
    }

    /// Whether the name's last part is `short`.
    pub fn is(&self, short: &str) -> bool {
        self.short_name() == short
    }

    fn zbus(err: zbus::Error) -> Self {
        match err {
            zbus::Error::MethodError(name, message, _) => BluezError::new(name.as_str(), message.unwrap_or_default()),
            zbus::Error::InputOutput(io) if io.kind() == io::ErrorKind::TimedOut => {
                BluezError::new("Timeout", io.to_string())
            }
            other => BluezError::new("zbus", other.to_string()),
        }
    }
}

/// The calls to bluetoothd the pairing and the scanning make, each on an object path.
pub trait Bluez: Send + Sync + 'static {
    /// `Device1.Pair`, which returns once the pairing ends.
    fn pair(&self, device: &str) -> impl Future<Output = Result<(), BluezError>> + Send;
    fn cancel_pairing(&self, device: &str) -> impl Future<Output = Result<(), BluezError>> + Send;
    /// Sets `Device1.Trusted`, so the device may reconnect on its own.
    fn set_trusted(&self, device: &str) -> impl Future<Output = Result<(), BluezError>> + Send;
    fn connect(&self, device: &str) -> impl Future<Output = Result<(), BluezError>> + Send;
    /// `Device1.Paired`.
    fn paired(&self, device: &str) -> impl Future<Output = Result<bool, BluezError>> + Send;
    /// `Adapter1.RemoveDevice` on the device's adapter.
    fn remove(&self, device: &str) -> impl Future<Output = Result<(), BluezError>> + Send;
    fn start_discovery(&self, adapter: &str) -> impl Future<Output = Result<(), BluezError>> + Send;
    fn stop_discovery(&self, adapter: &str) -> impl Future<Output = Result<(), BluezError>> + Send;
}

/// bluetoothd on the system bus. Its discovery session is this connection's, so it lasts as long
/// as the agent runs, and ends with it.
#[derive(Clone, Debug)]
pub struct SystemBluez {
    conn: Connection,
}

impl SystemBluez {
    #[must_use]
    pub fn new(conn: Connection) -> Self {
        SystemBluez { conn }
    }

    async fn call<B>(&self, path: &str, interface: &str, method: &str, body: &B) -> Result<zbus::Message, BluezError>
    where
        B: serde::Serialize + DynamicType,
    {
        let path = ObjectPath::try_from(path).map_err(|err| BluezError::new("InvalidArgs", err.to_string()))?;
        self.conn
            .call_method(Some(SERVICE), path, Some(interface), method, body)
            .await
            .map_err(BluezError::zbus)
    }

    async fn get(&self, path: &str, interface: &str, name: &str) -> Result<OwnedValue, BluezError> {
        let reply = self.call(path, PROPERTIES, "Get", &(interface, name)).await?;
        reply
            .body()
            .deserialize::<OwnedValue>()
            .map_err(|err| BluezError::new("InvalidReply", err.to_string()))
    }

    /// The device as a prompt names it: its alias, or its address without one.
    pub async fn device(&self, path: &str) -> Device {
        let alias = self
            .get(path, DEVICE, "Alias")
            .await
            .ok()
            .and_then(|alias| String::try_from(alias).ok())
            .filter(|alias| !alias.is_empty());
        Device {
            path: path.to_owned(),
            name: alias.unwrap_or_else(|| address(path)),
        }
    }

    /// `Device1.Trusted`, false when it can't be read.
    pub async fn trusted(&self, path: &str) -> bool {
        match self.get(path, DEVICE, "Trusted").await {
            Ok(value) => bool::try_from(value).unwrap_or(false),
            Err(err) => {
                log!("reading whether {path} is trusted: {err}");
                false
            }
        }
    }

    /// Registers the agent with bluetoothd and makes it the default, so every pairing uses it,
    /// the shell's and any other program's.
    ///
    /// # Errors
    ///
    /// When bluetoothd refuses either call.
    pub async fn register(&self) -> Result<(), BluezError> {
        let agent = ObjectPath::from_static_str_unchecked(AGENT_PATH);
        // A second registration on the same connection, after bluetoothd restarted, is refused
        // as AlreadyExists only when bluetoothd kept the first, which is as good.
        if let Err(err) = self
            .call("/org/bluez", AGENT_MANAGER, "RegisterAgent", &(&agent, CAPABILITY))
            .await
            && !err.is("AlreadyExists")
        {
            return Err(err);
        }
        self.call("/org/bluez", AGENT_MANAGER, "RequestDefaultAgent", &(&agent,))
            .await
            .map(drop)
    }
}

impl Bluez for SystemBluez {
    async fn pair(&self, device: &str) -> Result<(), BluezError> {
        self.call(device, DEVICE, "Pair", &()).await.map(drop)
    }

    async fn cancel_pairing(&self, device: &str) -> Result<(), BluezError> {
        self.call(device, DEVICE, "CancelPairing", &()).await.map(drop)
    }

    async fn set_trusted(&self, device: &str) -> Result<(), BluezError> {
        self.call(device, PROPERTIES, "Set", &(DEVICE, "Trusted", Value::from(true)))
            .await
            .map(drop)
    }

    async fn connect(&self, device: &str) -> Result<(), BluezError> {
        self.call(device, DEVICE, "Connect", &()).await.map(drop)
    }

    async fn paired(&self, device: &str) -> Result<bool, BluezError> {
        let value = self.get(device, DEVICE, "Paired").await?;
        bool::try_from(value).map_err(|err| BluezError::new("InvalidReply", err.to_string()))
    }

    async fn remove(&self, device: &str) -> Result<(), BluezError> {
        let adapter = device.rsplit_once('/').map_or(device, |(adapter, _)| adapter);
        let device = ObjectPath::try_from(device).map_err(|err| BluezError::new("InvalidArgs", err.to_string()))?;
        self.call(adapter, ADAPTER, "RemoveDevice", &(&device,)).await.map(drop)
    }

    async fn start_discovery(&self, adapter: &str) -> Result<(), BluezError> {
        self.call(adapter, ADAPTER, "StartDiscovery", &()).await.map(drop)
    }

    async fn stop_discovery(&self, adapter: &str) -> Result<(), BluezError> {
        self.call(adapter, ADAPTER, "StopDiscovery", &()).await.map(drop)
    }
}

/// The address in a device's object path: `/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF` is
/// `AA:BB:CC:DD:EE:FF`.
pub fn address(path: &str) -> String {
    let last = path.rsplit('/').next().unwrap_or(path);
    last.strip_prefix("dev_").unwrap_or(last).replace('_', ":")
}

/// The object bluetoothd calls with each request. It reads what the prompt names and whether a
/// device is trusted, and leaves the answer to [`Agent`].
#[derive(Debug)]
pub struct AgentObject {
    agent: Agent<StateFile>,
    bluez: Arc<SystemBluez>,
}

impl AgentObject {
    #[must_use]
    pub fn new(agent: Agent<StateFile>, bluez: Arc<SystemBluez>) -> Self {
        AgentObject { agent, bluez }
    }
}

/// [`Refusal`] as bluetoothd reads it.
#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "org.bluez.Error")]
pub enum AgentError {
    Rejected(String),
    Canceled(String),
}

impl From<Refusal> for AgentError {
    fn from(refusal: Refusal) -> Self {
        match refusal {
            Refusal::Rejected => AgentError::Rejected("The user said no".into()),
            Refusal::Canceled => AgentError::Canceled("Nobody answered".into()),
        }
    }
}

#[zbus::interface(name = "org.bluez.Agent1")]
impl AgentObject {
    fn release(&self) {
        self.agent.release();
    }

    async fn request_pin_code(&self, device: OwnedObjectPath) -> Result<String, AgentError> {
        let device = self.bluez.device(&device).await;
        Ok(self.agent.request_pin_code(&device).await?)
    }

    async fn display_pin_code(&self, device: OwnedObjectPath, pincode: String) {
        let device = self.bluez.device(&device).await;
        self.agent.display_pin_code(&device, &pincode);
    }

    async fn request_passkey(&self, device: OwnedObjectPath) -> Result<u32, AgentError> {
        let device = self.bluez.device(&device).await;
        Ok(self.agent.request_passkey(&device).await?)
    }

    async fn display_passkey(&self, device: OwnedObjectPath, passkey: u32, entered: u16) {
        let device = self.bluez.device(&device).await;
        self.agent.display_passkey(&device, passkey, entered);
    }

    async fn request_confirmation(&self, device: OwnedObjectPath, passkey: u32) -> Result<(), AgentError> {
        let device = self.bluez.device(&device).await;
        Ok(self.agent.request_confirmation(&device, passkey).await?)
    }

    async fn request_authorization(&self, device: OwnedObjectPath) -> Result<(), AgentError> {
        let device = self.bluez.device(&device).await;
        Ok(self.agent.request_authorization(&device).await?)
    }

    async fn authorize_service(&self, device: OwnedObjectPath, uuid: String) -> Result<(), AgentError> {
        let trusted = self.bluez.trusted(&device).await;
        let allowed = self.agent.authorize_service(&device, trusted);
        if allowed.is_err() {
            log!(
                "turned down service {uuid} for {}, which isn't trusted",
                device.as_str()
            );
        }
        Ok(allowed?)
    }

    fn cancel(&self) {
        self.agent.cancel();
    }
}

/// Registers the agent now and every time bluetoothd starts again, which forgets its agents, and
/// marks it gone while bluetoothd is. Returns only when the bus does.
///
/// # Errors
///
/// When the bus won't send bluetoothd's comings and goings.
pub async fn keep_registered(bluez: &SystemBluez, hub: &Hub<StateFile>) -> zbus::Result<()> {
    let dbus = zbus::fdo::DBusProxy::new(&bluez.conn).await?;
    let mut owners = dbus.receive_name_owner_changed_with_args(&[(0, SERVICE)]).await?;
    register(bluez, hub).await;
    while let Some(change) = owners.next().await {
        let appeared = change.args().is_ok_and(|args| args.new_owner().is_some());
        if appeared {
            register(bluez, hub).await;
        } else {
            log!("bluetoothd went away");
            hub.set_agent(false);
        }
    }
    Ok(())
}

async fn register(bluez: &SystemBluez, hub: &Hub<StateFile>) {
    match bluez.register().await {
        Ok(()) => hub.set_agent(true),
        Err(err) => {
            log!("registering the pairing agent with bluetoothd: {err}");
            hub.set_agent(false);
        }
    }
}

#[cfg(test)]
mod tests;
