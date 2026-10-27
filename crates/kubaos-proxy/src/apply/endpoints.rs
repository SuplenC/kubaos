//! The tunnel's endpoints as addresses. NetworkManager is given an address, never a name: with
//! the kill switch up (a tunnel coming back after a resume, or a second `on`), a lookup would go
//! out of the network in the clear, which the kill switch rightly blocks. So each name is
//! resolved here while nothing blocks it, kept in [`Paths::endpoints`], and that address is used
//! when the kill switch is up or the name doesn't resolve.
//!
//! # Owns
//!
//! - [`resolved`] and the endpoints file (`HOST PORT ADDRESS` lines).

use std::collections::BTreeMap;
use std::fs;
use std::net::IpAddr;

use super::StepError;
use crate::machine::Machine;
use crate::paths::Paths;
use crate::profile::write_atomic;
use crate::wireguard::Config;

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

/// `config` with each peer's endpoint name replaced by its address. `guarded`: the kill switch is
/// up, so the address kept from the last lookup goes first.
///
/// # Errors
///
/// A name that doesn't resolve and has no address kept.
pub(super) fn resolved(
    m: &mut dyn Machine,
    paths: &Paths,
    config: &Config,
    guarded: bool,
) -> Result<Config, StepError> {
    let mut known = cache(paths);
    let before = known.clone();
    let wants_v4 = config.interface.addresses.iter().any(|cidr| cidr.addr.is_ipv4());
    let mut out = config.clone();
    for peer in &mut out.peers {
        let Some(endpoint) = &mut peer.endpoint else {
            continue;
        };
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
