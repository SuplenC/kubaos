//! The VPN as NetworkManager is given it, in the settings dictionary of its D-Bus API, made fresh
//! at every `on`: a WireGuard connection ([`wireguard`]), or a VPN connection of the OpenVPN plugin
//! ([`openvpn`]).
//!
//! It is in memory only and volatile (`AddAndActivateConnection2`, `persist=volatile`):
//! NetworkManager never writes it to `/etc`, deletes it when it goes down, and never starts it on
//! its own. The secrets NetworkManager takes as values (WireGuard's private and preshared keys,
//! OpenVPN's password and the key's passphrase) are flagged agent-owned, so NetworkManager saves
//! them nowhere, not even in its in-memory profile under `/run`: it drops the ones in the call, and
//! asks a secret agent for them as it activates the VPN ([`crate::agent`]). The OpenVPN plugin
//! takes certificates and keys as files alone, so those are named by where they are for as long as
//! the plugin reads them ([`Files`]). The VPN's DNS is the connection's too: NetworkManager hands
//! the servers and routing domains to systemd-resolved for the VPN's link alone, and gives them
//! again whenever it reconfigures that link.
//!
//! # Owns
//!
//! - [`Connection`], what [`wireguard`] and [`openvpn`] put in it, and the names KubaOS gives the
//!   VPN ([`ID`], [`FWMARK`]).
//!
//! # Not here
//!
//! - The VPN's interface: [`crate::vpn::Type::interface`]. Calling NetworkManager: [`crate::real`].

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

use crate::net::Cidr;
use crate::openvpn;
use crate::openvpn::Credentials;
use crate::profile::Dns;
use crate::secrets::Keys;
use crate::vpn::Type;
use crate::wireguard;

/// The connection's id in NetworkManager.
pub const ID: &str = "kubaos-proxy";
/// The firewall mark of WireGuard's own packets, and NetworkManager's routing table for a full
/// tunnel. The kill switch lets them out (`system/files/proxy/kubaos-proxy-write`, `FWMARK`):
/// keep the two the same.
pub const FWMARK: u32 = 0x6b6f;
/// `NM_SETTING_SECRET_FLAG_AGENT_OWNED`: NetworkManager saves the secret nowhere and asks a secret
/// agent for it.
pub const AGENT_OWNED: u32 = 1;
/// The DNS priority of a full tunnel: below every other link's, so with `~.` every name goes to
/// the work servers and none leaks to the network's own.
pub const FULL_TUNNEL_DNS_PRIORITY: i32 = -50;
/// NetworkManager's OpenVPN plugin, by its D-Bus name.
pub const OPENVPN_SERVICE: &str = "org.freedesktop.NetworkManager.openvpn";

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
    /// A dictionary of strings (`a{ss}`): a VPN plugin's `data`.
    Map(BTreeMap<String, String>),
    /// A dictionary of secrets (`a{ss}`), a VPN plugin's `secrets`: never printed.
    Secrets(BTreeMap<String, String>),
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
            Prop::Map(map) => f.debug_map().entries(map).finish(),
            Prop::Secrets(map) => f
                .debug_map()
                .entries(map.keys().map(|key| (key, "<redacted>")))
                .finish(),
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

/// The `connection` setting of NetworkManager's type `nm_type` on `interface`, owned by `user` when
/// one is known (so it needs no admin to make).
fn base(nm_type: &str, interface: Option<&str>, user: Option<&str>) -> BTreeMap<String, Prop> {
    let mut connection = vec![
        ("id", str(ID)),
        ("type", str(nm_type)),
        ("autoconnect", Prop::Bool(false)),
    ];
    if let Some(interface) = interface {
        connection.push(("interface-name", str(interface)));
    }
    if let Some(user) = user {
        connection.push(("permissions", strs([format!("user:{user}")])));
    }
    setting(connection)
}

/// The WireGuard connection for `config` with its `keys`, the work `dns` on its link, owned by
/// `user` when one is known.
pub fn wireguard(config: &wireguard::Config, keys: &Keys, dns: &Dns, user: Option<&str>) -> Connection {
    let full = config.is_full();
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
    let addresses = &config.interface.addresses;
    let has_address = |v4: bool| addresses.iter().any(|c| c.addr.is_ipv4() == v4);
    let mut settings = Connection::new();
    settings.insert(
        "connection".to_owned(),
        base("wireguard", Some(Type::WireGuard.interface()), user),
    );
    settings.insert("wireguard".to_owned(), setting(wireguard));
    for v4 in [true, false] {
        let mut props = if has_address(v4) {
            manual(addresses, v4)
        } else {
            vec![("method", str("disabled"))]
        };
        if has_address(v4) && domains_here(has_address, dns, v4) {
            props.extend(dns_props(dns, full, v4));
        } else if has_address(v4) {
            props.extend(dns_servers(dns, v4));
        }
        settings.insert(family(v4).to_owned(), setting(props));
    }
    settings
}

fn family(v4: bool) -> &'static str {
    if v4 { "ipv4" } else { "ipv6" }
}

fn peers(config: &wireguard::Config, keys: &Keys) -> Vec<BTreeMap<String, Prop>> {
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

/// A family's `manual` method with the config's addresses of that family.
fn manual(addresses: &[Cidr], v4: bool) -> Vec<(&'static str, Prop)> {
    let address_data = addresses
        .iter()
        .filter(|cidr| cidr.addr.is_ipv4() == v4)
        .map(|cidr| {
            setting(vec![
                ("address", str(cidr.addr.to_string())),
                ("prefix", Prop::U32(u32::from(cidr.prefix))),
            ])
        })
        .collect();
    vec![("method", str("manual")), ("address-data", Prop::Dicts(address_data))]
}

/// The DNS servers of the family.
fn dns_servers(dns: &Dns, v4: bool) -> Vec<(&'static str, Prop)> {
    let servers: Vec<String> = dns
        .servers
        .iter()
        .filter(|s| s.is_ipv4() == v4)
        .map(ToString::to_string)
        .collect();
    if servers.is_empty() {
        Vec::new()
    } else {
        vec![("dns-data", Prop::Strs(servers))]
    }
}

/// The DNS servers of the family, with the routing domains: every name (`~.`, first in line) for a
/// full tunnel with servers.
fn dns_props(dns: &Dns, full: bool, v4: bool) -> Vec<(&'static str, Prop)> {
    let mut props = dns_servers(dns, v4);
    let mut search: Vec<String> = dns.domains.iter().map(|d| format!("~{d}")).collect();
    if full && !dns.servers.is_empty() {
        search.push("~.".to_owned());
        props.push(("dns-priority", Prop::I32(FULL_TUNNEL_DNS_PRIORITY)));
    }
    if !search.is_empty() {
        props.push(("dns-search", Prop::Strs(search)));
    }
    props
}

/// The routing domains go on one family only: the first with both an address and a server, else
/// IPv4 when it has an address.
fn domains_here(has_address: impl Fn(bool) -> bool, dns: &Dns, v4: bool) -> bool {
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

/// Where an OpenVPN VPN's certificates and keys are while the plugin reads them, by the name of
/// each: `ca`, `cert`, `key`, `tls-auth`, `tls-crypt`, `tls-crypt-v2`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Files(pub Vec<(&'static str, PathBuf)>);

impl Files {
    /// The file named `name`.
    pub fn get(&self, name: &str) -> Option<&PathBuf> {
        self.0.iter().find(|(n, _)| *n == name).map(|(_, path)| path)
    }
}

/// The plugin's key for each file, by its name.
const FILE_KEYS: [(&str, &str); 6] = [
    ("ca", "ca"),
    ("cert", "cert"),
    ("key", "key"),
    ("tls-auth", "ta"),
    ("tls-crypt", "tls-crypt"),
    ("tls-crypt-v2", "tls-crypt-v2"),
];

/// The OpenVPN connection for `config` with its `credentials`, the certificates and keys at
/// `files`, the work `dns` on its link, owned by `user` when one is known. The plugin opens the
/// tunnel as [`Type::OpenVpn`]'s interface, so the kill switch knows it by name.
pub fn openvpn(
    config: &openvpn::Config,
    credentials: &Credentials,
    files: &Files,
    dns: &Dns,
    user: Option<&str>,
) -> Connection {
    let pair = credentials.cert.is_some() && credentials.key.is_some();
    let kind = match (config.auth_user_pass, pair) {
        (true, true) => "password-tls",
        (true, false) => "password",
        (false, _) => "tls",
    };
    let mut data: BTreeMap<String, String> = config.data().into_iter().map(|(k, v)| (k.to_owned(), v)).collect();
    data.insert("connection-type".to_owned(), kind.to_owned());
    data.insert("dev".to_owned(), Type::OpenVpn.interface().to_owned());
    data.insert("dev-type".to_owned(), "tun".to_owned());
    for (name, key) in FILE_KEYS {
        if let Some(path) = files.get(name) {
            data.insert(key.to_owned(), path.display().to_string());
        }
    }
    if files.get("tls-auth").is_some()
        && let Some(direction) = config.key_direction
    {
        data.insert("ta-dir".to_owned(), direction.to_string());
    }
    let mut secrets = BTreeMap::new();
    if config.auth_user_pass {
        if let Some(username) = &credentials.username {
            data.insert("username".to_owned(), username.expose().to_owned());
        }
        if let Some(password) = &credentials.password {
            secrets.insert("password".to_owned(), password.expose().to_owned());
        }
        data.insert("password-flags".to_owned(), AGENT_OWNED.to_string());
    }
    if let Some(pass) = &credentials.cert_pass {
        secrets.insert("cert-pass".to_owned(), pass.expose().to_owned());
        data.insert("cert-pass-flags".to_owned(), AGENT_OWNED.to_string());
    }
    let mut vpn = vec![("service-type", str(OPENVPN_SERVICE)), ("data", Prop::Map(data))];
    if !secrets.is_empty() {
        vpn.push(("secrets", Prop::Secrets(secrets)));
    }
    let mut settings = Connection::new();
    settings.insert("connection".to_owned(), base("vpn", None, user));
    settings.insert("vpn".to_owned(), setting(vpn));
    let full = config.is_full();
    for v4 in [true, false] {
        let mut props = vec![("method", str("auto"))];
        // A split tunnel carries the server's routes and the config's, never the default route,
        // even when the server pushes it: what the profile says it is, the kill switch included.
        if !full {
            props.push(("never-default", Prop::Bool(true)));
        }
        let routes: Vec<BTreeMap<String, Prop>> = config
            .routes
            .iter()
            .filter(|cidr| cidr.addr.is_ipv4() == v4)
            .map(|cidr| {
                setting(vec![
                    ("dest", str(cidr.addr.to_string())),
                    ("prefix", Prop::U32(u32::from(cidr.prefix))),
                ])
            })
            .collect();
        if !routes.is_empty() {
            props.push(("route-data", Prop::Dicts(routes)));
        }
        if domains_here(|_| true, dns, v4) {
            props.extend(dns_props(dns, full, v4));
        } else {
            props.extend(dns_servers(dns, v4));
        }
        settings.insert(family(v4).to_owned(), setting(props));
    }
    settings
}

#[cfg(test)]
mod tests;
