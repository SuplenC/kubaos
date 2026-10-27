//! A WireGuard `.conf` as IT hands it out, in wg-quick's format: one `[Interface]`, then a
//! `[Peer]` for each server.
//!
//! The keys in it are secrets. They are parsed into [`Secret`]s, which never print, and an error
//! names the line and the key but never the value. [`Config::sanitized`] is the same file without
//! them: what the user layer keeps.
//!
//! # Owns
//!
//! - Parsing and checking a `.conf` ([`Config::parse`]), the keys ([`Key`], [`Secret`]), addresses
//!   ([`Cidr`]) and endpoints ([`Endpoint`]), and whether it is a full tunnel
//!   ([`Config::is_full`]).
//!
//! # Not here
//!
//! - Where the secrets go: [`crate::secrets`]. What NetworkManager is given: [`crate::tunnel`].

use std::fmt;
use std::net::IpAddr;

use kubaos_core::bitwarden::base64;

/// A WireGuard public key: 32 bytes, written as base64.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key([u8; 32]);

impl Key {
    /// The key written as base64, `None` for anything else.
    pub fn parse(text: &str) -> Option<Key> {
        key_bytes(text).map(Key)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&base64::encode(&self.0))
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Key({self})")
    }
}

/// A private or preshared key: 32 bytes like a [`Key`], but never printed.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    /// The key written as base64, `None` for anything else.
    pub fn parse(text: &str) -> Option<Secret> {
        key_bytes(text).map(|bytes| Secret(base64::encode(&bytes)))
    }

    /// The key as base64, for the vault, the keyring and NetworkManager alone.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

fn key_bytes(text: &str) -> Option<[u8; 32]> {
    let text = text.trim();
    // 32 bytes are 43 base64 characters and one `=`.
    if text.len() != 44 || !text.ends_with('=') {
        return None;
    }
    base64::decode(text)?.try_into().ok()
}

/// An address with its prefix length: `10.8.0.2/32`, `0.0.0.0/0`, `fd00::/64`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cidr {
    pub addr: IpAddr,
    pub prefix: u8,
}

impl Cidr {
    /// `addr/prefix`; a bare address is the host alone (`/32`, `/128`).
    pub fn parse(text: &str) -> Option<Cidr> {
        let (addr, prefix) = match text.split_once('/') {
            Some((addr, prefix)) => (addr, Some(prefix)),
            None => (text, None),
        };
        let addr: IpAddr = addr.parse().ok()?;
        let max = if addr.is_ipv4() { 32 } else { 128 };
        let prefix = match prefix {
            Some(prefix) if prefix.bytes().all(|b| b.is_ascii_digit()) && !prefix.is_empty() => {
                prefix.parse::<u8>().ok().filter(|&p| p <= max)?
            }
            Some(_) => return None,
            None => max,
        };
        Some(Cidr { addr, prefix })
    }

    /// Every address of its family: `0.0.0.0/0` or `::/0`.
    pub fn is_default(&self) -> bool {
        self.prefix == 0
    }
}

impl fmt::Display for Cidr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.addr, self.prefix)
    }
}

/// Where a peer is reached: a name or an address, and a port.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoint {
    pub host: String,
    pub port: u16,
}

impl Endpoint {
    /// `host:port`, `1.2.3.4:51820` or `[2001:db8::1]:51820`.
    pub fn parse(text: &str) -> Option<Endpoint> {
        let (host, port) = text.rsplit_once(':')?;
        let port: u16 = port.parse().ok().filter(|&p| p != 0)?;
        let host = match host.strip_prefix('[').and_then(|h| h.strip_suffix(']')) {
            Some(v6) => v6.parse::<std::net::Ipv6Addr>().ok()?.to_string(),
            None if host.parse::<std::net::Ipv4Addr>().is_ok() || is_hostname(host) => host.to_ascii_lowercase(),
            None => return None,
        };
        Some(Endpoint { host, port })
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.host.contains(':') {
            write!(f, "[{}]:{}", self.host, self.port)
        } else {
            write!(f, "{}:{}", self.host, self.port)
        }
    }
}

/// A DNS name: labels of letters, digits and hyphens, joined by dots. A trailing dot is not part
/// of it.
pub fn is_hostname(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 253
        && name.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}

/// The `[Interface]`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Interface {
    pub private_key: Option<Secret>,
    pub addresses: Vec<Cidr>,
    /// `DNS =`'s addresses.
    pub dns_servers: Vec<IpAddr>,
    /// `DNS =`'s names: wg-quick's search domains, the tunnel's routing domains here.
    pub dns_domains: Vec<String>,
    pub mtu: Option<u32>,
    pub listen_port: Option<u16>,
}

/// A `[Peer]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Peer {
    pub public_key: Key,
    pub preshared_key: Option<Secret>,
    pub endpoint: Option<Endpoint>,
    pub allowed_ips: Vec<Cidr>,
    pub keepalive: Option<u16>,
}

/// A whole `.conf`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub interface: Interface,
    pub peers: Vec<Peer>,
    /// wg-quick's own keys, which KubaOS doesn't act on (`PostUp`, `Table`, `SaveConfig`...): the
    /// import says it left them out.
    pub ignored: Vec<String>,
}

/// Why a `.conf` can't be used: the line, 0 for the whole file, and what is wrong with it, the
/// source. A secret's value is never in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub problem: Problem,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            0 => f.write_str("the WireGuard config"),
            line => write!(f, "line {line} of the WireGuard config"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.problem)
    }
}

/// What is wrong with a line of a `.conf`, or with the whole of it (line 0).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum Problem {
    #[error("`{0}` isn't a section of a WireGuard config ([Interface] or [Peer])")]
    Section(String),
    #[error("`{0}` isn't a key of a WireGuard config")]
    Key(String),
    #[error("{0} is outside a section")]
    Outside(String),
    #[error("this line isn't `Key = Value`")]
    NotKeyValue,
    #[error("{0} isn't a WireGuard key (44 characters of base64)")]
    BadKey(&'static str),
    #[error("{key} has `{value}`, which isn't {wanted}")]
    BadValue {
        key: &'static str,
        value: String,
        wanted: &'static str,
    },
    #[error("{0} is set twice")]
    Twice(&'static str),
    #[error("the config has no [Interface]")]
    NoInterface,
    #[error("the config has two [Interface] sections")]
    TwoInterfaces,
    #[error("the config has no [Peer]")]
    NoPeer,
    #[error("a [Peer] has no PublicKey")]
    NoPublicKey,
    #[error("a [Peer] has no AllowedIPs")]
    NoAllowedIps,
    #[error("two [Peer] sections have the same PublicKey")]
    SamePeer,
    #[error("the config has no Address")]
    NoAddress,
}

/// wg-quick's keys that only its own scripts act on.
const WG_QUICK_ONLY: [&str; 7] = [
    "PreUp",
    "PostUp",
    "PreDown",
    "PostDown",
    "SaveConfig",
    "Table",
    "FwMark",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    None,
    Interface,
    Peer,
}

#[derive(Default)]
struct PeerDraft {
    line: usize,
    public_key: Option<Key>,
    preshared_key: Option<Secret>,
    endpoint: Option<Endpoint>,
    allowed_ips: Option<Vec<Cidr>>,
    keepalive: Option<u16>,
}

impl Config {
    /// Parses a `.conf`. Keys and section names match whatever their case, `#` starts a comment,
    /// and a list (`Address`, `DNS`, `AllowedIPs`) may be split over several lines.
    ///
    /// # Errors
    ///
    /// The first line that isn't a WireGuard config, or what the whole lacks: an `[Interface]`
    /// with an address, a `[Peer]` with a public key and its allowed IPs.
    pub fn parse(text: &str) -> Result<Config, ParseError> {
        let mut section = Section::None;
        let mut interfaces = 0;
        let mut interface = Interface::default();
        let mut seen: Vec<&'static str> = Vec::new();
        let mut peers: Vec<PeerDraft> = Vec::new();
        let mut ignored = Vec::new();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let err = |problem| ParseError { line, problem };
            let content = raw.split('#').next().unwrap_or("").trim();
            if content.is_empty() {
                continue;
            }
            if let Some(name) = content.strip_prefix('[').and_then(|c| c.strip_suffix(']')) {
                section = match name.trim().to_ascii_lowercase().as_str() {
                    "interface" => {
                        interfaces += 1;
                        if interfaces > 1 {
                            return Err(err(Problem::TwoInterfaces));
                        }
                        seen.clear();
                        Section::Interface
                    }
                    "peer" => {
                        peers.push(PeerDraft {
                            line,
                            ..PeerDraft::default()
                        });
                        Section::Peer
                    }
                    _ => return Err(err(Problem::Section(name.trim().to_owned()))),
                };
                continue;
            }
            let Some((key, value)) = content.split_once('=') else {
                return Err(err(Problem::NotKeyValue));
            };
            let (key, value) = (key.trim(), value.trim());
            match section {
                Section::None => return Err(err(Problem::Outside(key.to_owned()))),
                Section::Interface => {
                    interface_key(&mut interface, &mut seen, &mut ignored, key, value).map_err(err)?;
                }
                Section::Peer => {
                    let Some(draft) = peers.last_mut() else {
                        return Err(err(Problem::Outside(key.to_owned())));
                    };
                    peer_key(draft, &mut ignored, key, value).map_err(err)?;
                }
            }
        }
        let whole = |problem| ParseError { line: 0, problem };
        if interfaces == 0 {
            return Err(whole(Problem::NoInterface));
        }
        if interface.addresses.is_empty() {
            return Err(whole(Problem::NoAddress));
        }
        if peers.is_empty() {
            return Err(whole(Problem::NoPeer));
        }
        let mut done: Vec<Peer> = Vec::with_capacity(peers.len());
        for draft in peers {
            let at = |problem| ParseError {
                line: draft.line,
                problem,
            };
            let public_key = draft.public_key.ok_or_else(|| at(Problem::NoPublicKey))?;
            let allowed_ips = draft
                .allowed_ips
                .filter(|ips| !ips.is_empty())
                .ok_or_else(|| at(Problem::NoAllowedIps))?;
            if done.iter().any(|peer| peer.public_key == public_key) {
                return Err(at(Problem::SamePeer));
            }
            done.push(Peer {
                public_key,
                preshared_key: draft.preshared_key,
                endpoint: draft.endpoint,
                allowed_ips,
                keepalive: draft.keepalive,
            });
        }
        Ok(Config {
            interface,
            peers: done,
            ignored,
        })
    }

    /// A peer takes every address of a family (`0.0.0.0/0` or `::/0`): a full tunnel.
    pub fn is_full(&self) -> bool {
        self.peers
            .iter()
            .flat_map(|peer| &peer.allowed_ips)
            .any(Cidr::is_default)
    }

    /// The first peer's endpoint, which the shell shows.
    pub fn endpoint(&self) -> Option<&Endpoint> {
        self.peers.iter().find_map(|peer| peer.endpoint.as_ref())
    }

    /// The config without its secrets, as the user layer keeps it.
    #[must_use]
    pub fn sanitized(&self) -> Config {
        let mut config = self.clone();
        config.interface.private_key = None;
        for peer in &mut config.peers {
            peer.preshared_key = None;
        }
        config.ignored.clear();
        config
    }

    /// The config in wg-quick's format, its secrets included where it has them.
    pub fn render(&self) -> String {
        let joined = |items: Vec<String>| items.join(", ");
        let mut lines = vec!["[Interface]".to_owned()];
        if let Some(key) = &self.interface.private_key {
            lines.push(["PrivateKey = ", key.expose()].concat());
        }
        lines.push(format!(
            "Address = {}",
            joined(self.interface.addresses.iter().map(ToString::to_string).collect())
        ));
        let dns: Vec<String> = self
            .interface
            .dns_servers
            .iter()
            .map(ToString::to_string)
            .chain(self.interface.dns_domains.iter().cloned())
            .collect();
        if !dns.is_empty() {
            lines.push(format!("DNS = {}", joined(dns)));
        }
        if let Some(mtu) = self.interface.mtu {
            lines.push(format!("MTU = {mtu}"));
        }
        if let Some(port) = self.interface.listen_port {
            lines.push(format!("ListenPort = {port}"));
        }
        for peer in &self.peers {
            lines.push(String::new());
            lines.push("[Peer]".to_owned());
            lines.push(format!("PublicKey = {}", peer.public_key));
            if let Some(key) = &peer.preshared_key {
                lines.push(["PresharedKey = ", key.expose()].concat());
            }
            if let Some(endpoint) = &peer.endpoint {
                lines.push(format!("Endpoint = {endpoint}"));
            }
            lines.push(format!(
                "AllowedIPs = {}",
                joined(peer.allowed_ips.iter().map(ToString::to_string).collect())
            ));
            if let Some(keepalive) = peer.keepalive {
                lines.push(format!("PersistentKeepalive = {keepalive}"));
            }
        }
        lines.push(String::new());
        lines.join("\n")
    }
}

/// `key`, matched whatever its case, as one of `known`'s names.
fn known(key: &str, names: &[&'static str]) -> Option<&'static str> {
    names.iter().copied().find(|name| name.eq_ignore_ascii_case(key))
}

fn once(seen: &mut Vec<&'static str>, key: &'static str) -> Result<(), Problem> {
    if seen.contains(&key) {
        return Err(Problem::Twice(key));
    }
    seen.push(key);
    Ok(())
}

fn bad(key: &'static str, value: &str, wanted: &'static str) -> Problem {
    Problem::BadValue {
        key,
        value: value.to_owned(),
        wanted,
    }
}

fn items(value: &str) -> impl Iterator<Item = &str> {
    value.split(',').map(str::trim).filter(|item| !item.is_empty())
}

fn number<T: std::str::FromStr>(key: &'static str, value: &str, wanted: &'static str) -> Result<T, Problem> {
    value.parse().map_err(|_| bad(key, value, wanted))
}

fn interface_key(
    interface: &mut Interface,
    seen: &mut Vec<&'static str>,
    ignored: &mut Vec<String>,
    key: &str,
    value: &str,
) -> Result<(), Problem> {
    let names = ["PrivateKey", "Address", "DNS", "MTU", "ListenPort"];
    if let Some(name) = known(key, &WG_QUICK_ONLY) {
        if !ignored.iter().any(|i| i == name) {
            ignored.push(name.to_owned());
        }
        return Ok(());
    }
    match known(key, &names) {
        Some("PrivateKey") => {
            once(seen, "PrivateKey")?;
            interface.private_key = Some(Secret::parse(value).ok_or(Problem::BadKey("PrivateKey"))?);
        }
        Some("Address") => {
            for item in items(value) {
                interface
                    .addresses
                    .push(Cidr::parse(item).ok_or_else(|| bad("Address", item, "an address like 10.8.0.2/32"))?);
            }
        }
        Some("DNS") => {
            for item in items(value) {
                if let Ok(addr) = item.parse::<IpAddr>() {
                    interface.dns_servers.push(addr);
                } else if is_hostname(item.trim_end_matches('.')) {
                    interface
                        .dns_domains
                        .push(item.trim_end_matches('.').to_ascii_lowercase());
                } else {
                    return Err(bad("DNS", item, "an address or a domain"));
                }
            }
        }
        Some("MTU") => {
            once(seen, "MTU")?;
            let mtu: u32 = number("MTU", value, "an MTU from 576 to 9000")?;
            if !(576..=9000).contains(&mtu) {
                return Err(bad("MTU", value, "an MTU from 576 to 9000"));
            }
            interface.mtu = Some(mtu);
        }
        Some("ListenPort") => {
            once(seen, "ListenPort")?;
            interface.listen_port = Some(number("ListenPort", value, "a port")?);
        }
        _ => return Err(Problem::Key(key.to_owned())),
    }
    Ok(())
}

fn peer_key(draft: &mut PeerDraft, ignored: &mut Vec<String>, key: &str, value: &str) -> Result<(), Problem> {
    let names = [
        "PublicKey",
        "PresharedKey",
        "Endpoint",
        "AllowedIPs",
        "PersistentKeepalive",
    ];
    if let Some(name) = known(key, &WG_QUICK_ONLY) {
        if !ignored.iter().any(|i| i == name) {
            ignored.push(name.to_owned());
        }
        return Ok(());
    }
    match known(key, &names) {
        Some("PublicKey") => {
            if draft.public_key.is_some() {
                return Err(Problem::Twice("PublicKey"));
            }
            draft.public_key = Some(Key::parse(value).ok_or(Problem::BadKey("PublicKey"))?);
        }
        Some("PresharedKey") => {
            if draft.preshared_key.is_some() {
                return Err(Problem::Twice("PresharedKey"));
            }
            draft.preshared_key = Some(Secret::parse(value).ok_or(Problem::BadKey("PresharedKey"))?);
        }
        Some("Endpoint") => {
            if draft.endpoint.is_some() {
                return Err(Problem::Twice("Endpoint"));
            }
            draft.endpoint =
                Some(Endpoint::parse(value).ok_or_else(|| bad("Endpoint", value, "a host and a port, host:51820"))?);
        }
        Some("AllowedIPs") => {
            let ips = draft.allowed_ips.get_or_insert_with(Vec::new);
            for item in items(value) {
                ips.push(Cidr::parse(item).ok_or_else(|| bad("AllowedIPs", item, "an address range like 10.0.0.0/8"))?);
            }
        }
        Some("PersistentKeepalive") => {
            if draft.keepalive.is_some() {
                return Err(Problem::Twice("PersistentKeepalive"));
            }
            draft.keepalive = if value.eq_ignore_ascii_case("off") {
                None
            } else {
                Some(number("PersistentKeepalive", value, "seconds or off")?)
            };
        }
        _ => return Err(Problem::Key(key.to_owned())),
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests;
