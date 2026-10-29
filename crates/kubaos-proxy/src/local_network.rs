//! The local networks the kill switch lets out beside the tunnel while the profile allows them:
//! the ranges of the machine's own interfaces, as they are when the kill switch goes up, so the
//! phone link, a printer or SSH on the network you're on keep working.
//!
//! A range the tunnel routes never goes out beside it: a work network in 192.168.x behind the
//! tunnel and a home network in the same range can't both be reached, and the tunnel wins.
//!
//! # Owns
//!
//! - Reading the interfaces' ranges ([`read`], from `ip -json address`) and which of them the kill
//!   switch lets through ([`let_through`]).
//!
//! # Not here
//!
//! - The rules that let them through, link-local and discovery's multicast among them:
//!   `system/files/proxy/kubaos-proxy-write`. When they are read again: [`crate::apply`].

use std::net::IpAddr;

use serde_json::Value;

use crate::machine::Cmd;
use crate::machine::Machine;
use crate::tunnel::INTERFACE;
use crate::wireguard::Cidr;
use crate::wireguard::Config;

/// The most ranges the kill switch takes, as the root helper does: more than any laptop's
/// interfaces hold.
pub const MAX_NETWORKS: usize = 32;

/// The interfaces whose ranges never count: loopback, the tunnel, and the container bridges, which
/// the kill switch lets through by name.
fn counts(interface: &str) -> bool {
    !(interface == "lo"
        || interface == INTERFACE
        || interface == "docker0"
        || interface == "virbr0"
        || interface.starts_with("br-")
        || interface.starts_with("veth"))
}

/// The ranges of the interfaces that are up, as `ip -json address show up` gives them. A range
/// read wrong is left out: the kill switch then blocks it, which is the safe side.
pub fn parse(json: &str) -> Vec<Cidr> {
    let Ok(Value::Array(interfaces)) = serde_json::from_str::<Value>(json) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for interface in &interfaces {
        if !interface["ifname"].as_str().is_some_and(counts) {
            continue;
        }
        let Some(addresses) = interface["addr_info"].as_array() else {
            continue;
        };
        for address in addresses {
            let addr = address["local"].as_str().and_then(|a| a.parse::<IpAddr>().ok());
            let prefix = address["prefixlen"].as_u64().and_then(|p| u8::try_from(p).ok());
            if let (Some(addr), Some(prefix)) = (addr, prefix)
                && let Some(cidr) = Cidr::parse(&format!("{addr}/{prefix}"))
            {
                found.push(cidr);
            }
        }
    }
    found
}

/// The ranges of the machine's interfaces now; none when `ip` can't say.
pub fn read(m: &mut dyn Machine) -> Vec<Cidr> {
    match m.run(&Cmd::new("ip").args(["-json", "address", "show", "up"])) {
        Ok(ran) if ran.ok() => parse(&ran.stdout),
        _ => Vec::new(),
    }
}

/// What the tunnel routes: its peers' allowed IPs but a whole family, which is what makes it full,
/// and its own addresses.
fn routes(tunnel: &Config) -> Vec<Cidr> {
    tunnel
        .peers
        .iter()
        .flat_map(|peer| &peer.allowed_ips)
        .chain(&tunnel.interface.addresses)
        .filter(|cidr| !cidr.is_default())
        .copied()
        .collect()
}

/// Whether the kill switch lets the range itself out: link-local ranges it lets through whole,
/// and loopback and multicast ranges aren't a network to reach. Nor is an IPv6 range of IPv4
/// addresses (`::ffff:a.b.c.d`), which the root helper refuses as it is written with dots.
fn reachable(cidr: &Cidr) -> bool {
    !cidr.is_default()
        && match cidr.addr {
            IpAddr::V4(v4) => !(v4.is_loopback() || v4.is_link_local() || v4.is_multicast() || v4.is_unspecified()),
            IpAddr::V6(v6) => {
                let embeds_v4 = v6.to_ipv4_mapped().is_some() || v6.segments()[..6] == [0; 6];
                !(embeds_v4 || v6.is_unicast_link_local() || v6.is_multicast())
            }
        }
}

/// Which of the `found` ranges the kill switch lets out beside `tunnel`: each once, by its first
/// address, the widest of those inside each other, and none that overlaps a range the tunnel
/// routes, so work traffic never leaves outside it. At most [`MAX_NETWORKS`].
pub fn let_through(found: &[Cidr], tunnel: &Config) -> Vec<Cidr> {
    let routes = routes(tunnel);
    let mut networks: Vec<Cidr> = found
        .iter()
        .map(|cidr| cidr.network())
        .filter(reachable)
        .filter(|cidr| !routes.iter().any(|route| route.overlaps(cidr)))
        .collect();
    networks.sort_by_key(|cidr| (cidr.addr.is_ipv6(), cidr.prefix, cidr.addr));
    let mut kept: Vec<Cidr> = Vec::new();
    for cidr in networks {
        if !kept.iter().any(|wider| wider.overlaps(&cidr)) {
            kept.push(cidr);
        }
    }
    kept.truncate(MAX_NETWORKS);
    kept
}

#[cfg(test)]
mod tests;
