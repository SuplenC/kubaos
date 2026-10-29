//! Addresses and servers, as a VPN config writes them: an address with its prefix ([`Cidr`]), a
//! server's name or address and port ([`Endpoint`]), and DNS names ([`is_hostname`]).
//!
//! # Owns
//!
//! - [`Cidr`], [`Endpoint`] and [`is_hostname`].
//!
//! # Not here
//!
//! - The configs that hold them: [`crate::wireguard`] and [`crate::openvpn`].

use std::fmt;
use std::net::IpAddr;

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

    /// The range it is in, by its first address: `192.168.1.0/24` for `192.168.1.20/24`.
    #[must_use]
    pub fn network(self) -> Cidr {
        let (bits, width) = bits(self.addr);
        let first = bits & mask(self.prefix, width);
        let addr = match self.addr {
            IpAddr::V4(_) => IpAddr::V4(std::net::Ipv4Addr::from(u32::try_from(first).unwrap_or_default())),
            IpAddr::V6(_) => IpAddr::V6(std::net::Ipv6Addr::from(first)),
        };
        Cidr {
            addr,
            prefix: self.prefix,
        }
    }

    /// Whether the two ranges share an address: one holds the other.
    pub fn overlaps(&self, other: &Cidr) -> bool {
        if self.addr.is_ipv4() != other.addr.is_ipv4() {
            return false;
        }
        let (mine, width) = bits(self.addr);
        let (theirs, _) = bits(other.addr);
        let mask = mask(self.prefix.min(other.prefix), width);
        mine & mask == theirs & mask
    }
}

/// The address as a number, and how many bits its family has.
fn bits(addr: IpAddr) -> (u128, u8) {
    match addr {
        IpAddr::V4(v4) => (u128::from(u32::from(v4)), 32),
        IpAddr::V6(v6) => (u128::from(v6), 128),
    }
}

/// The mask of a `prefix` in a family of `width` bits.
fn mask(prefix: u8, width: u8) -> u128 {
    let family = if width == 128 { u128::MAX } else { (1u128 << width) - 1 };
    if prefix == 0 {
        return 0;
    }
    (u128::MAX << (width - prefix.min(width))) & family
}

impl fmt::Display for Cidr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.addr, self.prefix)
    }
}

/// Where a VPN server is reached: a name or an address, and a port.
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
