//! The VPN, whatever its type: a WireGuard `.conf` or an OpenVPN `.ovpn`, told apart by what the
//! file holds ([`detect`]), and what the rest of the Proxy asks of either ([`Config`]): its
//! server, whether it is a full tunnel, the ranges it routes, its DNS, its interface.
//!
//! # Owns
//!
//! - [`Type`], [`detect`], [`Config`] and [`ParseError`].
//!
//! # Not here
//!
//! - Each type's format: [`crate::wireguard`], [`crate::openvpn`]. What NetworkManager is given:
//!   [`crate::tunnel`].

use std::fmt;
use std::net::IpAddr;
use std::path::Path;

use crate::net::Cidr;
use crate::net::Endpoint;
use crate::openvpn;
use crate::wireguard;

/// A VPN's type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    WireGuard,
    OpenVpn,
}

impl Type {
    pub const ALL: [Type; 2] = [Type::WireGuard, Type::OpenVpn];

    /// Its name in `proxy.toml` and `status --json`: `wireguard`.
    pub const fn name(self) -> &'static str {
        match self {
            Type::WireGuard => "wireguard",
            Type::OpenVpn => "openvpn",
        }
    }

    pub fn from_name(name: &str) -> Option<Type> {
        Type::ALL.into_iter().find(|kind| kind.name() == name)
    }

    /// Its name for a person: `WireGuard`.
    pub const fn title(self) -> &'static str {
        match self {
            Type::WireGuard => "WireGuard",
            Type::OpenVpn => "OpenVPN",
        }
    }

    /// The network interface its tunnel takes. The kill switch lets each of these through by
    /// name (`system/files/proxy/kubaos-proxy-write`, `INTERFACES`): keep the two the same.
    pub const fn interface(self) -> &'static str {
        match self {
            Type::WireGuard => "wg-kubaos",
            Type::OpenVpn => "tun-kubaos",
        }
    }

    /// The file `kubaos-proxy import` writes beside `proxy.toml`.
    pub const fn file_name(self) -> &'static str {
        match self {
            Type::WireGuard => "vpn.conf",
            Type::OpenVpn => "vpn.ovpn",
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.title())
    }
}

/// The interfaces of every type, which the local networks never count.
pub const INTERFACES: [&str; 2] = [Type::WireGuard.interface(), Type::OpenVpn.interface()];

/// The type of the config in `text`: WireGuard's `[Interface]` or `[Peer]`, else OpenVPN's
/// `client`, `remote` or `<ca>`. `None` for neither.
pub fn detect(text: &str) -> Option<Type> {
    let lines = || {
        text.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with(';'))
    };
    if lines().any(|line| {
        let line = line.to_ascii_lowercase();
        line.starts_with("[interface]") || line.starts_with("[peer]")
    }) {
        return Some(Type::WireGuard);
    }
    let openvpn = lines().any(|line| {
        let word = line.split_whitespace().next().unwrap_or("").trim_start_matches("--");
        matches!(word, "client" | "remote" | "tls-client") || line.eq_ignore_ascii_case("<ca>")
    });
    openvpn.then_some(Type::OpenVpn)
}

/// A VPN's config, of either type.
#[derive(Clone, Debug, PartialEq, Eq)]
#[expect(
    clippy::large_enum_variant,
    reason = "A profile holds one VPN, read once a switch; boxing it saves nothing."
)]
pub enum Config {
    WireGuard(wireguard::Config),
    OpenVpn(openvpn::Config),
}

/// Why a VPN's config can't be used.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ParseError {
    #[error(transparent)]
    WireGuard(wireguard::ParseError),
    #[error(transparent)]
    OpenVpn(openvpn::ParseError),
}

impl Config {
    /// `text` as a config of `kind`, an OpenVPN one reading the files it names in `dir`.
    ///
    /// # Errors
    ///
    /// The text isn't a config of that type the Proxy can use.
    pub fn parse(kind: Type, text: &str, dir: &Path) -> Result<Config, ParseError> {
        match kind {
            Type::WireGuard => wireguard::Config::parse(text)
                .map(Config::WireGuard)
                .map_err(ParseError::WireGuard),
            Type::OpenVpn => openvpn::Config::parse(text, dir)
                .map(Config::OpenVpn)
                .map_err(ParseError::OpenVpn),
        }
    }

    pub const fn kind(&self) -> Type {
        match self {
            Config::WireGuard(_) => Type::WireGuard,
            Config::OpenVpn(_) => Type::OpenVpn,
        }
    }

    /// Everything goes through it.
    pub fn is_full(&self) -> bool {
        match self {
            Config::WireGuard(config) => config.is_full(),
            Config::OpenVpn(config) => config.is_full(),
        }
    }

    /// The server the shell shows: WireGuard's first peer's endpoint, OpenVPN's first remote.
    pub fn endpoint(&self) -> Option<&Endpoint> {
        match self {
            Config::WireGuard(config) => config.endpoint(),
            Config::OpenVpn(config) => config.endpoint(),
        }
    }

    /// Every server it reaches, to resolve before NetworkManager is given it.
    pub fn endpoints_mut(&mut self) -> Vec<&mut Endpoint> {
        match self {
            Config::WireGuard(config) => config
                .peers
                .iter_mut()
                .filter_map(|peer| peer.endpoint.as_mut())
                .collect(),
            Config::OpenVpn(config) => config.remotes.iter_mut().map(|remote| &mut remote.endpoint).collect(),
        }
    }

    /// Whether a server's name resolves to its IPv4 address first: the family the tunnel's own
    /// addresses take for WireGuard, IPv4 for OpenVPN unless it reaches its servers over IPv6
    /// alone.
    pub fn prefers_v4(&self) -> bool {
        match self {
            Config::WireGuard(config) => config.interface.addresses.iter().any(|cidr| cidr.addr.is_ipv4()),
            Config::OpenVpn(config) => !config.remotes.iter().all(|remote| remote.proto.is_v6_only()),
        }
    }

    /// The ranges it routes, as the config says, but a whole family, which is what makes it full:
    /// WireGuard's peers' allowed IPs and its own addresses, OpenVPN's `route` lines. A server
    /// pushes OpenVPN's other routes as the tunnel comes up.
    pub fn routes(&self) -> Vec<Cidr> {
        let routes: Vec<Cidr> = match self {
            Config::WireGuard(config) => config
                .peers
                .iter()
                .flat_map(|peer| &peer.allowed_ips)
                .chain(&config.interface.addresses)
                .copied()
                .collect(),
            Config::OpenVpn(config) => config.routes.clone(),
        };
        routes.into_iter().filter(|cidr| !cidr.is_default()).collect()
    }

    /// The DNS servers the config names.
    pub fn dns_servers(&self) -> &[IpAddr] {
        match self {
            Config::WireGuard(config) => &config.interface.dns_servers,
            Config::OpenVpn(config) => &config.dns_servers,
        }
    }

    /// The domains the config's DNS servers answer for.
    pub fn dns_domains(&self) -> &[String] {
        match self {
            Config::WireGuard(config) => &config.interface.dns_domains,
            Config::OpenVpn(config) => &config.dns_domains,
        }
    }

    /// Whether the file holds a secret in plain text, which `status` warns of: the import keeps
    /// them out.
    pub fn has_secrets(&self) -> bool {
        match self {
            Config::WireGuard(config) => {
                config.interface.private_key.is_some() || config.peers.iter().any(|peer| peer.preshared_key.is_some())
            }
            Config::OpenVpn(config) => !config.credentials.is_empty(),
        }
    }

    /// What the config has that KubaOS doesn't act on, for the import to name.
    pub fn ignored(&self) -> &[String] {
        match self {
            Config::WireGuard(config) => &config.ignored,
            Config::OpenVpn(config) => &config.ignored,
        }
    }

    /// The config without its secrets, as the user layer keeps it.
    #[must_use]
    pub fn sanitized(&self) -> Config {
        match self {
            Config::WireGuard(config) => Config::WireGuard(config.sanitized()),
            Config::OpenVpn(config) => Config::OpenVpn(config.sanitized()),
        }
    }

    /// The config in its own format.
    pub fn render(&self) -> String {
        match self {
            Config::WireGuard(config) => config.render(),
            Config::OpenVpn(config) => config.render(),
        }
    }
}

#[cfg(test)]
mod tests;
