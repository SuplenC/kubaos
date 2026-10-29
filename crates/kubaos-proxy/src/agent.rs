//! The VPN's secret agent: what `kubaos-proxy` tells NetworkManager when it asks for the VPN's
//! secrets. They are agent-owned ([`crate::tunnel`]), so NetworkManager keeps none of them: as it
//! activates the VPN it asks the user's registered secret agents, and `kubaos-proxy` is one for as
//! long as it brings its own connection up ([`crate::real`]).
//!
//! Any program on the system bus can call an agent, and NetworkManager asks every agent of the
//! user for every connection that needs a secret, so the agent gives the secrets to NetworkManager
//! alone, for the one connection it is bringing up, and nothing to any other request.
//!
//! # Owns
//!
//! - [`Agent`], made from the connection being brought up, and [`Agent::answer`]: which requests
//!   get the secrets ([`Request`], [`Answer`], [`Refusal`]).
//!
//! # Not here
//!
//! - The agent on the bus, and registering it with NetworkManager: [`crate::real`].

use std::collections::BTreeMap;
use std::fmt;

use crate::tunnel::Connection;
use crate::tunnel::Prop;

/// The setting a WireGuard connection's keys are in.
pub const WIREGUARD: &str = "wireguard";
/// The setting a VPN plugin's connection (OpenVPN's) has its secrets in, under `secrets`.
pub const VPN: &str = "vpn";

/// A `GetSecrets` request, as far as its answer depends on it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    /// Whether NetworkManager sent it, rather than another program on the bus.
    pub from_network_manager: bool,
    /// The connection's `connection.id`.
    pub id: Option<String>,
    /// The connection's `connection.uuid`.
    pub uuid: Option<String>,
    /// The setting whose secrets are asked for.
    pub setting: String,
}

/// Why a request gets nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    NotNetworkManager,
    OtherConnection,
    OtherSetting,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Refusal::NotNetworkManager => "the caller isn't NetworkManager",
            Refusal::OtherConnection => "it's for another connection",
            Refusal::OtherSetting => "it's for a setting without the VPN's secrets",
        })
    }
}

/// What the agent says to a request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// The secrets, in NetworkManager's secrets dictionary: WireGuard's private key, and each
    /// peer's preshared key by its public key; a VPN plugin's `secrets`. Printed, they are
    /// redacted.
    Keys(Connection),
    Nothing(Refusal),
}

/// The agent for one connection: its id and uuid, the setting its secrets are in, and its
/// secrets.
#[derive(Clone, Debug)]
pub struct Agent {
    id: Option<String>,
    uuid: String,
    setting: String,
    keys: Connection,
}

impl Agent {
    /// The agent for `connection`, which NetworkManager is given with the uuid `uuid`.
    #[must_use]
    pub fn new(connection: &Connection, uuid: impl Into<String>) -> Agent {
        let text = |key: &str| match connection.get("connection").and_then(|c| c.get(key)) {
            Some(Prop::Str(value)) => Some(value.clone()),
            _ => None,
        };
        let setting = if text("type").as_deref() == Some(VPN) {
            VPN
        } else {
            WIREGUARD
        };
        Agent {
            id: text("id"),
            uuid: uuid.into(),
            setting: setting.to_owned(),
            keys: keys(connection, setting),
        }
    }

    /// What NetworkManager gets for `request`: the secrets when it asks for this connection's
    /// setting that has them, nothing for any other request.
    #[must_use]
    pub fn answer(&self, request: &Request) -> Answer {
        if !request.from_network_manager {
            return Answer::Nothing(Refusal::NotNetworkManager);
        }
        if request.uuid.as_deref() != Some(self.uuid.as_str()) || request.id != self.id {
            return Answer::Nothing(Refusal::OtherConnection);
        }
        if request.setting != self.setting {
            return Answer::Nothing(Refusal::OtherSetting);
        }
        Answer::Keys(self.keys.clone())
    }
}

/// The secrets of `connection`'s `setting`, as NetworkManager takes them back: its secret
/// properties (a VPN plugin's `secrets` among them), and each WireGuard peer that has one with its
/// public key, which names the peer.
fn keys(connection: &Connection, setting: &str) -> Connection {
    let mut secrets = BTreeMap::new();
    for (key, prop) in connection.get(setting).into_iter().flatten() {
        match prop {
            Prop::Secret(_) | Prop::Secrets(_) => {
                secrets.insert(key.clone(), prop.clone());
            }
            Prop::Dicts(peers) if key == "peers" => {
                let peers: Vec<_> = peers.iter().filter_map(peer_keys).collect();
                if !peers.is_empty() {
                    secrets.insert(key.clone(), Prop::Dicts(peers));
                }
            }
            _ => {}
        }
    }
    Connection::from([(setting.to_owned(), secrets)])
}

fn peer_keys(peer: &BTreeMap<String, Prop>) -> Option<BTreeMap<String, Prop>> {
    let mut keys: BTreeMap<String, Prop> = peer
        .iter()
        .filter(|(_, prop)| matches!(prop, Prop::Secret(_)))
        .map(|(key, prop)| (key.clone(), prop.clone()))
        .collect();
    if keys.is_empty() {
        return None;
    }
    keys.insert("public-key".to_owned(), peer.get("public-key")?.clone());
    Some(keys)
}

#[cfg(test)]
mod tests;
