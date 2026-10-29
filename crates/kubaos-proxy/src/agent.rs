//! The tunnel's secret agent: what `kubaos-proxy` tells NetworkManager when it asks for the
//! tunnel's keys. They are agent-owned ([`crate::tunnel`]), so NetworkManager keeps none of them:
//! as it activates the tunnel it asks the user's registered secret agents, and `kubaos-proxy` is
//! one for as long as it brings its own connection up ([`crate::real`]).
//!
//! Any program on the system bus can call an agent, and NetworkManager asks every agent of the
//! user for every connection that needs a secret, so the agent gives the keys to NetworkManager
//! alone, for the one connection it is bringing up, and nothing to any other request.
//!
//! # Owns
//!
//! - [`Agent`], made from the connection being brought up, and [`Agent::answer`]: which requests
//!   get the keys ([`Request`], [`Answer`], [`Refusal`]).
//!
//! # Not here
//!
//! - The agent on the bus, and registering it with NetworkManager: [`crate::real`].

use std::collections::BTreeMap;
use std::fmt;

use crate::tunnel::Connection;
use crate::tunnel::Prop;

/// The setting the tunnel's keys are in.
pub const SETTING: &str = "wireguard";

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
            Refusal::OtherSetting => "it's for a setting without the tunnel's keys",
        })
    }
}

/// What the agent says to a request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// The keys, in NetworkManager's secrets dictionary: the private key, and each peer's
    /// preshared key by its public key. Printed, they are redacted.
    Keys(Connection),
    Nothing(Refusal),
}

/// The agent for one connection: its id and uuid, and its keys.
#[derive(Clone, Debug)]
pub struct Agent {
    id: Option<String>,
    uuid: String,
    keys: Connection,
}

impl Agent {
    /// The agent for `connection`, which NetworkManager is given with the uuid `uuid`.
    #[must_use]
    pub fn new(connection: &Connection, uuid: impl Into<String>) -> Agent {
        let id = match connection.get("connection").and_then(|c| c.get("id")) {
            Some(Prop::Str(id)) => Some(id.clone()),
            _ => None,
        };
        Agent {
            id,
            uuid: uuid.into(),
            keys: keys(connection),
        }
    }

    /// What NetworkManager gets for `request`: the keys when it asks for this connection's
    /// WireGuard setting, nothing for any other request.
    #[must_use]
    pub fn answer(&self, request: &Request) -> Answer {
        if !request.from_network_manager {
            return Answer::Nothing(Refusal::NotNetworkManager);
        }
        if request.uuid.as_deref() != Some(self.uuid.as_str()) || request.id != self.id {
            return Answer::Nothing(Refusal::OtherConnection);
        }
        if request.setting != SETTING {
            return Answer::Nothing(Refusal::OtherSetting);
        }
        Answer::Keys(self.keys.clone())
    }
}

/// The secrets of `connection`'s WireGuard setting, as NetworkManager takes them back: its secret
/// properties, and each peer that has one with its public key, which names the peer.
fn keys(connection: &Connection) -> Connection {
    let mut wireguard = BTreeMap::new();
    for (key, prop) in connection.get(SETTING).into_iter().flatten() {
        match prop {
            Prop::Secret(_) => {
                wireguard.insert(key.clone(), prop.clone());
            }
            Prop::Dicts(peers) if key == "peers" => {
                let peers: Vec<_> = peers.iter().filter_map(peer_keys).collect();
                if !peers.is_empty() {
                    wireguard.insert(key.clone(), Prop::Dicts(peers));
                }
            }
            _ => {}
        }
    }
    Connection::from([(SETTING.to_owned(), wireguard)])
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
