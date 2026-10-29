//! The VPN's servers as addresses. NetworkManager is given an address, never a name: with the kill
//! switch up (a VPN coming back after a resume, or a second `on`), a lookup would go out of the
//! network in the clear, which the kill switch rightly blocks. So each name is resolved here while
//! nothing blocks it, kept in [`Paths::endpoints`], and that address is used when the kill switch
//! is up or the name doesn't resolve. The kill switch lets an OpenVPN server out by the same
//! address ([`servers`]).
//!
//! # Owns
//!
//! - [`resolved`], [`servers`] and the endpoints file (`HOST PORT ADDRESS` lines).

use std::collections::BTreeMap;
use std::fs;
use std::net::IpAddr;

use super::StepError;
use crate::machine::Machine;
use crate::paths::Paths;
use crate::plan::Server;
use crate::profile::write_atomic;
use crate::vpn;

fn cache(paths: &Paths) -> BTreeMap<(String, u16), IpAddr> {
    fs::read_to_string(paths.endpoints())
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let host = words.next()?.to_owned();
            let port = words.next()?.parse().ok()?;
            let addr = words.next()?.parse().ok()?;
            Some(((host, port), addr))
        })
        .collect()
}

/// `config` with each server's name replaced by its address. `guarded`: the kill switch is up, so
/// the address kept from the last lookup goes first.
///
/// # Errors
///
/// A name that doesn't resolve and has no address kept.
pub(super) fn resolved(
    m: &mut dyn Machine,
    paths: &Paths,
    config: &vpn::Config,
    guarded: bool,
) -> Result<vpn::Config, StepError> {
    let mut known = cache(paths);
    let before = known.clone();
    let wants_v4 = config.prefers_v4();
    let mut out = config.clone();
    for endpoint in out.endpoints_mut() {
        if endpoint.host.parse::<IpAddr>().is_ok() {
            continue;
        }
        let key = (endpoint.host.clone(), endpoint.port);
        let kept = known.get(&key).copied();
        let addr = match kept {
            Some(addr) if guarded => addr,
            _ => match m.resolve(&endpoint.host, endpoint.port) {
                Ok(addrs) if !addrs.is_empty() => {
                    let addr = addrs
                        .iter()
                        .find(|addr| addr.is_ipv4() == wants_v4)
                        .copied()
                        .unwrap_or(addrs[0]);
                    known.insert(key, addr);
                    addr
                }
                Ok(_) => kept.ok_or_else(|| {
                    StepError::Endpoint(crate::machine::ResolveError {
                        host: endpoint.host.clone(),
                        said: crate::machine::Said("no address".to_owned()),
                    })
                })?,
                Err(err) => kept.ok_or(StepError::Endpoint(err))?,
            },
        };
        endpoint.host = addr.to_string();
    }
    if known != before {
        let text = known
            .iter()
            .map(|((host, port), addr)| format!("{host} {port} {addr}"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        write_atomic(&paths.endpoints(), &text, 0o644).map_err(|err| StepError::File(paths.endpoints(), err))?;
    }
    Ok(out)
}

/// The servers the kill switch lets out to for `config`, by the addresses NetworkManager was
/// given: OpenVPN's remotes, a name by the address kept for it. None for WireGuard, whose packets
/// carry the tunnel's mark.
pub(super) fn servers(paths: &Paths, config: &vpn::Config) -> Vec<Server> {
    let vpn::Config::OpenVpn(config) = config else {
        return Vec::new();
    };
    let known = cache(paths);
    let mut servers: Vec<Server> = config
        .remotes
        .iter()
        .filter_map(|remote| {
            let host = &remote.endpoint.host;
            let port = remote.endpoint.port;
            let addr = host
                .parse::<IpAddr>()
                .ok()
                .or_else(|| known.get(&(host.clone(), port)).copied())?;
            Some(Server {
                tcp: remote.proto.is_tcp(),
                addr,
                port,
            })
        })
        .collect();
    servers.sort();
    servers.dedup();
    servers
}
