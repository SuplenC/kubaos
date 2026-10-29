//! The local networks the kill switch lets out beside the VPN while the profile allows them: the
//! ranges of the machine's own interfaces, as they are when the kill switch goes up, so the phone
//! link, a printer or SSH on the network you're on keep working.
//!
//! A range the VPN routes never goes out beside it: a work network in 192.168.x behind the VPN
//! and a home network in the same range can't both be reached, and the VPN wins. What it routes is
//! what its config names, and what its interface carries once it is up: an OpenVPN server pushes
//! its routes then.
//!
//! # Owns
//!
//! - Reading the interfaces' ranges ([`read`], from `ip -json address`), the ranges the VPN's
//!   interface carries ([`routed`], from `ip -json route`), and which of the first the kill switch
//!   lets through ([`let_through`]).
//!
//! # Not here
//!
//! - The rules that let them through, link-local and discovery's multicast among them:
//!   `system/files/proxy/kubaos-proxy-write`. When they are read again: [`crate::apply`].

use std::net::IpAddr;

use serde_json::Value;

use crate::machine::Cmd;
use crate::machine::Machine;
use crate::net::Cidr;
use crate::vpn::INTERFACES;

/// The most ranges the kill switch takes, as the root helper does: more than any laptop's
/// interfaces hold.
pub const MAX_NETWORKS: usize = 32;

/// The interfaces whose ranges never count: loopback, the VPN's, and the container bridges, which
/// the kill switch lets through by name.
fn counts(interface: &str) -> bool {
    !(interface == "lo"
        || INTERFACES.contains(&interface)
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

/// The ranges `ip -json route show table all dev IFACE` gives, the default and the kernel's own
/// local and broadcast routes left out. A route read wrong is left out too.
pub fn parse_routes(json: &str) -> Vec<Cidr> {
    let Ok(Value::Array(routes)) = serde_json::from_str::<Value>(json) else {
        return Vec::new();
    };
    routes
        .iter()
        .filter(|route| route["type"].as_str().is_none_or(|kind| kind == "unicast"))
        .filter_map(|route| Cidr::parse(route["dst"].as_str()?))
        .filter(|cidr| !cidr.is_default())
        .map(Cidr::network)
        .collect()
}

/// The ranges the VPN's interface carries now; none when it isn't up or `ip` can't say.
pub fn routed(m: &mut dyn Machine, interface: &str) -> Vec<Cidr> {
    let cmd = Cmd::new("ip").args(["-json", "route", "show", "table", "all", "dev", interface]);
    match m.run(&cmd) {
        Ok(ran) if ran.ok() => parse_routes(&ran.stdout),
        _ => Vec::new(),
    }
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

/// Which of the `found` ranges the kill switch lets out beside a VPN that routes `routes`: each
/// once, by its first address, the widest of those inside each other, and none that overlaps a
/// range the VPN routes, so work traffic never leaves outside it. At most [`MAX_NETWORKS`].
pub fn let_through(found: &[Cidr], routes: &[Cidr]) -> Vec<Cidr> {
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
