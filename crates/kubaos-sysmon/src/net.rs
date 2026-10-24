//! The network: bytes in and out from `/proc/net/dev`, the active interface from the default route
//! in `/proc/net/route`, and its address from `/proc/net/fib_trie` (IPv4) or `/proc/net/if_inet6`.
//!
//! The kernel lists IPv4 addresses without their interface, so the address shown is the local one
//! inside a network the active interface routes to.
//!
//! # Owns
//!
//! - Parsing those four files, and picking the active interface and its address.
//!
//! # Not here
//!
//! - Speeds, which need two reads: [`crate::sampler`].

use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

/// The bytes an interface has received and sent since it came up.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Counter {
    pub name: String,
    pub received: u64,
    pub sent: u64,
}

/// Parses `/proc/net/dev`.
pub fn parse_net_dev(text: &str) -> Vec<Counter> {
    text.lines()
        .skip(2)
        .filter_map(|line| {
            let (name, rest) = line.split_once(':')?;
            let fields: Vec<u64> = rest
                .split_ascii_whitespace()
                .map(|value| value.parse().unwrap_or(0))
                .collect();
            Some(Counter {
                name: name.trim().to_owned(),
                received: *fields.first()?,
                sent: *fields.get(8)?,
            })
        })
        .collect()
}

/// A line of the IPv4 routing table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    pub interface: String,
    pub destination: Ipv4Addr,
    pub mask: Ipv4Addr,
    pub metric: u32,
    up: bool,
}

/// Parses `/proc/net/route`.
pub fn parse_routes(text: &str) -> Vec<Route> {
    text.lines()
        .skip(1)
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_ascii_whitespace().collect();
            let flags = u32::from_str_radix(fields.get(3)?, 16).ok()?;
            Some(Route {
                interface: (*fields.first()?).to_owned(),
                destination: hex_address(fields.get(1)?)?,
                mask: hex_address(fields.get(7)?)?,
                metric: fields.get(6)?.parse().ok()?,
                up: flags & 1 == 1,
            })
        })
        .collect()
}

/// An address as the routing table writes it: hex, in the machine's byte order, which on the
/// little-endian machines KubaOS runs on puts the first octet last.
fn hex_address(hex: &str) -> Option<Ipv4Addr> {
    let value = u32::from_str_radix(hex, 16).ok()?;
    Some(Ipv4Addr::from(value.to_le_bytes()))
}

/// The interface of the default route with the lowest metric.
pub fn default_interface(routes: &[Route]) -> Option<&str> {
    routes
        .iter()
        .filter(|route| route.up && route.destination.is_unspecified() && route.mask.is_unspecified())
        .min_by_key(|route| route.metric)
        .map(|route| route.interface.as_str())
}

/// The machine's own IPv4 addresses, from `/proc/net/fib_trie`: each `|-- a.b.c.d` followed by a
/// `/32 host LOCAL` line, once each.
pub fn parse_fib_locals(text: &str) -> Vec<Ipv4Addr> {
    let mut locals = Vec::new();
    let mut last: Option<Ipv4Addr> = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some(address) = line.strip_prefix("|-- ") {
            last = address.parse().ok();
        } else if line.starts_with("/32 host LOCAL")
            && let Some(address) = last.take()
            && !locals.contains(&address)
        {
            locals.push(address);
        }
    }
    locals
}

/// The local address inside a network `interface` routes to, loopback left out.
pub fn address(routes: &[Route], locals: &[Ipv4Addr], interface: &str) -> Option<Ipv4Addr> {
    let networks: Vec<&Route> = routes
        .iter()
        .filter(|route| route.interface == interface && !route.mask.is_unspecified())
        .collect();
    locals.iter().copied().find(|local| {
        !local.is_loopback()
            && networks
                .iter()
                .any(|route| local.to_bits() & route.mask.to_bits() == route.destination.to_bits())
    })
}

/// The global IPv6 address of `interface` in `/proc/net/if_inet6`, written short.
pub fn parse_if_inet6(text: &str, interface: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let fields: Vec<&str> = line.split_ascii_whitespace().collect();
        if fields.len() < 6 || fields[5] != interface || fields[3] != "00" {
            return None;
        }
        let bits = u128::from_str_radix(fields[0], 16).ok()?;
        Some(Ipv6Addr::from(bits).to_string())
    })
}

#[cfg(test)]
mod tests;
