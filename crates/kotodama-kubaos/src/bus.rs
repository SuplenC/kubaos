//! The D-Bus properties the toggles read and set, behind [`Bus`], so the
//! toggles are tested without NetworkManager, BlueZ or power-profiles-daemon.
//!
//! # Owns
//!
//! - [`Bus`], and [`SystemBus`], which calls the system bus for real: the
//!   `org.freedesktop.DBus.Properties` calls and the introspection the toggles need, and reading
//!   their answers.
//!
//! # Not here
//!
//! - Connecting, authenticating and marshalling: `kotodama_extension::dbus`.
//! - Which properties the toggles use: [`crate::toggles`].

use kotodama_extension::dbus;
use kotodama_extension::dbus::Call;
use kotodama_extension::dbus::Marshal;
use kotodama_extension::dbus::Message;

const PROPERTIES: &str = "org.freedesktop.DBus.Properties";
const INTROSPECTABLE: &str = "org.freedesktop.DBus.Introspectable";

/// A property of an object a service on the system bus exports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Property<'a> {
    /// The service's bus name, as `org.freedesktop.NetworkManager`.
    pub service: &'a str,
    pub path: &'a str,
    pub interface: &'a str,
    pub name: &'a str,
}

/// What reads and sets the properties.
pub trait Bus {
    /// The boolean `property`.
    ///
    /// # Errors
    ///
    /// When the call fails, or the property isn't a boolean.
    fn get_bool(&mut self, property: &Property) -> Result<bool, BusError>;

    /// Sets the boolean `property` to `value`.
    ///
    /// # Errors
    ///
    /// When the call fails.
    fn set_bool(&mut self, property: &Property, value: bool) -> Result<(), BusError>;

    /// Sets the string `property` to `value`.
    ///
    /// # Errors
    ///
    /// When the call fails.
    fn set_string(&mut self, property: &Property, value: &str) -> Result<(), BusError>;

    /// The names of the objects right below `path` in `service`, as its
    /// introspection lists them.
    ///
    /// # Errors
    ///
    /// When the call fails, or answers with something other than XML.
    fn children(&mut self, service: &str, path: &str) -> Result<Vec<String>, BusError>;
}

/// Why a property couldn't be read or set.
#[derive(Debug, thiserror::Error)]
pub enum BusError {
    /// The call didn't go through: no system bus, no such service, or the
    /// service turned it down.
    #[error("calling {service}")]
    Call {
        service: String,
        #[source]
        source: dbus::Error,
    },
    /// The answer wasn't of the type asked for.
    #[error("{service} answered with something other than {expected}")]
    Answer { service: String, expected: &'static str },
}

impl BusError {
    fn call(service: &str) -> impl FnOnce(dbus::Error) -> BusError + '_ {
        move |source| BusError::Call {
            service: service.to_owned(),
            source,
        }
    }

    fn answer(service: &str, expected: &'static str) -> BusError {
        BusError::Answer {
            service: service.to_owned(),
            expected,
        }
    }
}

/// The system bus. Each call connects and hangs up, as toggles are rare and a
/// connection held open would cost a socket for the Extension's whole life.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemBus;

impl SystemBus {
    fn call(call: &Call) -> Result<Message, BusError> {
        let service = call.destination;
        let mut bus = dbus::Bus::system().map_err(BusError::call(service))?;
        bus.call(call, |_| {}).map_err(BusError::call(service))
    }

    fn set(property: &Property, signature: &str, value: impl FnOnce(&mut Marshal)) -> Result<(), BusError> {
        let mut body = Marshal::default();
        body.string(property.interface);
        body.string(property.name);
        body.signature(signature);
        value(&mut body);
        let call = Call::new(property.path, PROPERTIES, "Set", property.service).with_body("ssv", body);
        SystemBus::call(&call).map(drop)
    }
}

impl Bus for SystemBus {
    fn get_bool(&mut self, property: &Property) -> Result<bool, BusError> {
        let mut body = Marshal::default();
        body.string(property.interface);
        body.string(property.name);
        let call = Call::new(property.path, PROPERTIES, "Get", property.service).with_body("ss", body);
        let reply = SystemBus::call(&call)?;
        let mut read = reply.body();
        let (Some(signature), Some(value)) = (read.u32(), read.u32()) else {
            return Err(BusError::answer(property.service, "a boolean"));
        };
        bool_variant(signature, value).ok_or_else(|| BusError::answer(property.service, "a boolean"))
    }

    fn set_bool(&mut self, property: &Property, value: bool) -> Result<(), BusError> {
        SystemBus::set(property, "b", |body| body.u32(u32::from(value)))
    }

    fn set_string(&mut self, property: &Property, value: &str) -> Result<(), BusError> {
        SystemBus::set(property, "s", |body| body.string(value))
    }

    fn children(&mut self, service: &str, path: &str) -> Result<Vec<String>, BusError> {
        let reply = SystemBus::call(&Call::new(path, INTROSPECTABLE, "Introspect", service))?;
        let xml = reply
            .body()
            .string()
            .ok_or_else(|| BusError::answer(service, "introspection XML"))?;
        Ok(child_nodes(xml))
    }
}

/// The boolean in a variant whose first four bytes were read as the word
/// `signature` and the next four as `value`, or `None` if it holds another
/// type. A variant opens with its signature: a length byte, the type's
/// letters and a nul, so a boolean's is `1 b 0`, padded to the 4-byte
/// boundary its `u32` starts at. Each word was read in the message's own byte
/// order, so its bytes are checked in both.
fn bool_variant(signature: u32, value: u32) -> Option<bool> {
    let boolean = [1, b'b', 0];
    let is_boolean = signature.to_le_bytes()[..3] == boolean || signature.to_be_bytes()[..3] == boolean;
    match value {
        0 if is_boolean => Some(false),
        1 if is_boolean => Some(true),
        _ => None,
    }
}

/// The `name`s of the `<node>` elements in introspection `xml`, which are
/// the children of the object introspected. The outer `<node>` is the object
/// itself, named by its path if at all.
fn child_nodes(xml: &str) -> Vec<String> {
    xml.split("<node")
        .skip(1)
        .filter_map(|element| {
            let element = element.split('>').next()?;
            let name = element.split("name=\"").nth(1)?.split('"').next()?;
            (!name.contains('/')).then(|| name.to_owned())
        })
        .collect()
}

#[cfg(test)]
mod tests;
