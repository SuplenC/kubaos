//! The tunnel as NetworkManager is given it: a WireGuard connection, in the settings dictionary
//! of its D-Bus API, made fresh at every `on`.
//!
//! It is in memory only and volatile (`AddAndActivateConnection2`, `persist=volatile`):
//! NetworkManager never writes it to `/etc`, deletes it when it goes down, and never starts it on
//! its own. The private and preshared keys are in the call, flagged agent-owned, so
//! NetworkManager keeps them for that activation and saves them nowhere; a profile it does write
//! (its in-memory copy under `/run`) has no keys. The tunnel's DNS is the connection's too:
//! NetworkManager hands the servers and routing domains to systemd-resolved for the tunnel's link
//! alone, and gives them again whenever it reconfigures that link.
//!
//! # Owns
//!
//! - [`Connection`], what [`connection`] puts in it, and the names KubaOS gives the tunnel ([`ID`],
//!   [`INTERFACE`], [`FWMARK`]).
//!
//! # Not here
//!
//! - Calling NetworkManager: [`crate::real`].

use std::collections::BTreeMap;
use std::fmt;
use std::net::IpAddr;

use crate::profile::Dns;
use crate::secrets::Keys;
use crate::wireguard::Cidr;
use crate::wireguard::Config;

/// The connection's id in NetworkManager.
pub const ID: &str = "kubaos-proxy";
/// The tunnel's network interface; the kill switch lets it through by this name.
pub const INTERFACE: &str = "wg-kubaos";
/// The firewall mark of the tunnel's own packets, and NetworkManager's routing table for a full
/// tunnel. The kill switch lets them out (`system/files/proxy/kubaos-proxy-write`, `FWMARK`):
/// keep the two the same.
pub const FWMARK: u32 = 0x6b6f;
/// `NM_SETTING_SECRET_FLAG_AGENT_OWNED`: NetworkManager saves the secret nowhere.
pub const AGENT_OWNED: u32 = 1;
/// The DNS priority of a full tunnel: below every other link's, so with `~.` every name goes to
/// the work servers and none leaks to the network's own.
pub const FULL_TUNNEL_DNS_PRIORITY: i32 = -50;

/// A value in NetworkManager's settings.
#[derive(Clone, PartialEq, Eq)]
pub enum Prop {
    Str(String),
    /// A secret: never printed.
    Secret(String),
    U32(u32),
    I32(i32),
    Bool(bool),
    Strs(Vec<String>),
    Dicts(Vec<BTreeMap<String, Prop>>),
}

impl fmt::Debug for Prop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Prop::Str(s) => write!(f, "{s:?}"),
            Prop::Secret(_) => f.write_str("<redacted>"),
            Prop::U32(n) => write!(f, "{n}u"),
            Prop::I32(n) => write!(f, "{n}i"),
            Prop::Bool(b) => write!(f, "{b}"),
            Prop::Strs(list) => f.debug_list().entries(list).finish(),
            Prop::Dicts(dicts) => f.debug_list().entries(dicts).finish(),
        }
    }
}

/// A connection's settings: setting name, then property, then value (`a{sa{sv}}`).
pub type Connection = BTreeMap<String, BTreeMap<String, Prop>>;

fn str(s: impl Into<String>) -> Prop {
    Prop::Str(s.into())
}

fn strs(items: impl IntoIterator<Item = String>) -> Prop {
    Prop::Strs(items.into_iter().collect())
}

fn setting(props: Vec<(&str, Prop)>) -> BTreeMap<String, Prop> {
    props.into_iter().map(|(k, v)| (k.to_owned(), v)).collect()
}

/// The connection for `config` with its `keys`, the work `dns` on its link, owned by `user` when
/// one is known (so it needs no admin to make).
pub fn connection(config: &Config, keys: &Keys, dns: &Dns, user: Option<&str>) -> Connection {
    let full = config.is_full();
    let mut connection = vec![
        ("id", str(ID)),
        ("type", str("wireguard")),
        ("interface-name", str(INTERFACE)),
        ("autoconnect", Prop::Bool(false)),
    ];
    if let Some(user) = user {
        connection.push(("permissions", strs([format!("user:{user}")])));
    }
    let mut wireguard = vec![
        ("private-key", Prop::Secret(keys.private.expose().to_owned())),
        ("private-key-flags", Prop::U32(AGENT_OWNED)),
        ("fwmark", Prop::U32(FWMARK)),
        ("peers", Prop::Dicts(peers(config, keys))),
    ];
    if let Some(port) = config.interface.listen_port {
        wireguard.push(("listen-port", Prop::U32(u32::from(port))));
    }
    if let Some(mtu) = config.interface.mtu {
        wireguard.push(("mtu", Prop::U32(mtu)));
    }
    let mut settings = Connection::new();
    settings.insert("connection".to_owned(), setting(connection));
    settings.insert("wireguard".to_owned(), setting(wireguard));
    settings.insert("ipv4".to_owned(), ip(config, dns, full, true));
    settings.insert("ipv6".to_owned(), ip(config, dns, full, false));
    settings
}

fn peers(config: &Config, keys: &Keys) -> Vec<BTreeMap<String, Prop>> {
    config
        .peers
        .iter()
        .map(|peer| {
            let mut props = vec![
                ("public-key", str(peer.public_key.to_string())),
                ("allowed-ips", strs(peer.allowed_ips.iter().map(ToString::to_string))),
            ];
            if let Some(endpoint) = &peer.endpoint {
                props.push(("endpoint", str(endpoint.to_string())));
            }
            if let Some(keepalive) = peer.keepalive {
                props.push(("persistent-keepalive", Prop::U32(u32::from(keepalive))));
            }
            if let Some(psk) = keys.preshared_for(&peer.public_key) {
                props.push(("preshared-key", Prop::Secret(psk.expose().to_owned())));
                props.push(("preshared-key-flags", Prop::U32(AGENT_OWNED)));
            }
            setting(props)
        })
        .collect()
}

/// The `ipv4` or `ipv6` setting: the `.conf`'s addresses of that family, and the DNS servers of
/// that family with the routing domains on the first family that has servers.
fn ip(config: &Config, dns: &Dns, full: bool, v4: bool) -> BTreeMap<String, Prop> {
    let family = |addr: &IpAddr| addr.is_ipv4() == v4;
    let addresses: Vec<&Cidr> = config.interface.addresses.iter().filter(|c| family(&c.addr)).collect();
    if addresses.is_empty() {
        return setting(vec![("method", str("disabled"))]);
    }
    let address_data = addresses
        .iter()
        .map(|cidr| {
            setting(vec![
                ("address", str(cidr.addr.to_string())),
                ("prefix", Prop::U32(u32::from(cidr.prefix))),
            ])
        })
        .collect();
    let mut props = vec![("method", str("manual")), ("address-data", Prop::Dicts(address_data))];
    let servers: Vec<String> = dns
        .servers
        .iter()
        .filter(|s| family(s))
        .map(ToString::to_string)
        .collect();
    if !servers.is_empty() {
        props.push(("dns-data", Prop::Strs(servers)));
    }
    if domains_here(config, dns, v4) {
        let mut search: Vec<String> = dns.domains.iter().map(|d| format!("~{d}")).collect();
        if full && !dns.servers.is_empty() {
            search.push("~.".to_owned());
            props.push(("dns-priority", Prop::I32(FULL_TUNNEL_DNS_PRIORITY)));
        }
        if !search.is_empty() {
            props.push(("dns-search", Prop::Strs(search)));
        }
    }
    setting(props)
}

/// The routing domains go on one family only: the first with both an address and a server, else
/// IPv4 when it has an address.
fn domains_here(config: &Config, dns: &Dns, v4: bool) -> bool {
    let has_address = |four: bool| config.interface.addresses.iter().any(|c| c.addr.is_ipv4() == four);
    let has_server = |four: bool| dns.servers.iter().any(|s| s.is_ipv4() == four);
    let chosen = if has_address(true) && has_server(true) {
        true
    } else if has_address(false) && has_server(false) {
        false
    } else {
        has_address(true)
    };
    chosen == v4
}

#[cfg(test)]
mod tests;
