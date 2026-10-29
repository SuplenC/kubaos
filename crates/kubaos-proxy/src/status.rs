//! What the Proxy is now, for `kubaos-proxy status`, the shell and the Command menu: the parts the
//! profile has and which are up, and the tunnel's endpoint, last handshake and bytes moved.
//!
//! The shell watches `$XDG_RUNTIME_DIR/kubaos-proxy.json`, which every change rewrites whole
//! ([`publish`]). The handshake needs root (WireGuard's netlink is admin-only), so it is read
//! through the helper only when asked live, while the control center's detail or the menu's View
//! is open.
//!
//! # Owns
//!
//! - [`Status`], reading it ([`read`]), publishing it ([`publish`]) and its rows for a person
//!   ([`rows`]).
//!
//! # Not here
//!
//! - What is up: [`crate::apply::applied`].

use std::fs;
use std::path::Path;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use serde::Serialize;

use crate::apply;
use crate::machine::Machine;
use crate::paths::Paths;
use crate::plan::Plan;
use crate::profile::LoadError;
use crate::profile::Profile;
use crate::profile::write_atomic;
use crate::tunnel::INTERFACE;

/// The tunnel, as the profile has it and NetworkManager runs it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "The shell reads each as a field of its own."
)]
#[serde(rename_all = "camelCase")]
pub struct Tunnel {
    pub full: bool,
    pub kill_switch: bool,
    /// The profile lets the local networks through the kill switch.
    pub allow_local_network: bool,
    /// The local networks the kill switch lets through now; none while it is down.
    pub local_networks: Vec<String>,
    pub endpoint: Option<String>,
    pub up: bool,
    /// The latest handshake with any peer, in seconds since the epoch; 0 for none yet. Only when
    /// read live.
    pub handshake: Option<u64>,
    pub rx_bytes: Option<u64>,
    pub tx_bytes: Option<u64>,
}

/// The Proxy now.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "The shell reads each as a field of its own."
)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// The profile has a part to switch.
    pub configured: bool,
    /// The user wants it on in this session.
    pub wanted: bool,
    /// Every part of the profile is up.
    pub on: bool,
    /// A change is under way.
    pub busy: bool,
    /// The profile's parts: `tunnel`, `dns`, `proxy`, `pac`, `ca`, `kill-switch`.
    pub parts: Vec<&'static str>,
    pub tunnel: Option<Tunnel>,
    pub dns_servers: Vec<String>,
    pub dns_domains: Vec<String>,
    pub proxy: Option<String>,
    pub pac: Option<String>,
    pub ca: Option<String>,
    pub warnings: Vec<String>,
    /// What is wrong with the profile, or what the last change ran into.
    pub problem: Option<String>,
}

fn parts(profile: &Profile) -> Vec<&'static str> {
    let mut parts = Vec::new();
    if profile.tunnel.is_some() {
        parts.push("tunnel");
    }
    if !profile.dns.is_empty() {
        parts.push("dns");
    }
    if profile.http.proxy.is_some() {
        parts.push("proxy");
    }
    if profile.http.pac.is_some() {
        parts.push("pac");
    }
    if profile.ca.is_some() {
        parts.push("ca");
    }
    if profile.kill_switch() {
        parts.push("kill-switch");
    }
    parts
}

fn counter(sys: &Path, name: &str) -> Option<u64> {
    fs::read_to_string(sys.join("class/net").join(INTERFACE).join("statistics").join(name))
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// The Proxy now, under `profile` as it loaded. `live` reads the handshake and the byte counts
/// too.
pub fn read(m: &mut dyn Machine, paths: &Paths, profile: &Result<Profile, LoadError>, live: bool) -> Status {
    let wanted = apply::wanted(paths);
    let applied = apply::applied(paths);
    let profile = match profile {
        Ok(profile) => profile,
        Err(err) => {
            return Status {
                wanted,
                on: !applied.is_empty(),
                problem: Some(apply::chain(err)),
                ..Status::default()
            };
        }
    };
    let plan = Plan::for_profile(profile);
    let tunnel_up = profile.tunnel.is_some() && m.tunnel_active().unwrap_or(false);
    let on = !plan.is_empty()
        && plan.kinds().iter().all(|kind| applied.contains(kind))
        && (profile.tunnel.is_none() || tunnel_up);
    let local_networks = apply::local_networks_let_through(paths)
        .iter()
        .map(ToString::to_string)
        .collect();
    let tunnel = profile.tunnel.as_ref().map(|tunnel| {
        let live = live && tunnel_up;
        Tunnel {
            full: tunnel.config.is_full(),
            kill_switch: profile.kill_switch(),
            allow_local_network: tunnel.allow_local_network,
            local_networks,
            endpoint: tunnel.config.endpoint().map(ToString::to_string),
            up: tunnel_up,
            handshake: if live { apply::handshake(m) } else { None },
            rx_bytes: if live { counter(&paths.sys, "rx_bytes") } else { None },
            tx_bytes: if live { counter(&paths.sys, "tx_bytes") } else { None },
        }
    });
    Status {
        configured: !profile.is_empty(),
        wanted,
        on,
        busy: false,
        parts: parts(profile),
        tunnel,
        dns_servers: profile.dns.servers.iter().map(ToString::to_string).collect(),
        dns_domains: profile.dns.domains.clone(),
        proxy: profile.http.proxy.as_ref().map(ToString::to_string),
        pac: profile.http.pac.clone(),
        ca: profile.ca.as_ref().map(|ca| ca.file.display().to_string()),
        warnings: plan.warnings,
        problem: (!wanted && !applied.is_empty())
            .then(|| "parts of the Proxy are still up: kubaos proxy off takes them away".to_owned()),
    }
}

/// Writes `status` where the shell watches it, whole.
///
/// # Errors
///
/// The runtime directory can't be written.
pub fn publish(paths: &Paths, status: &Status) -> std::io::Result<()> {
    let text = serde_json::to_string(status).unwrap_or_default();
    write_atomic(&paths.shell(), &text, 0o644)
}

/// A line of the status for a person: what, then how it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub label: &'static str,
    pub value: String,
}

/// `bytes` for a person: `12.4 MB`.
pub fn bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "kB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1000.0 && unit < UNITS.len() - 1 {
        value /= 1000.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// How long ago `epoch` was, for a person: `42 s ago`, `3 min ago`, `never`.
pub fn ago(epoch: u64, now: u64) -> String {
    if epoch == 0 {
        return "never".to_owned();
    }
    let secs = now.saturating_sub(epoch);
    match secs {
        0..60 => format!("{secs} s ago"),
        60..3600 => format!("{} min ago", secs / 60),
        _ => format!("{} h ago", secs / 3600),
    }
}

/// Seconds since the epoch now.
pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Whether the kill switch lets the local networks through, and which, for a person.
fn local_network(tunnel: &Tunnel) -> String {
    match (tunnel.allow_local_network, tunnel.local_networks.as_slice()) {
        (false, _) => "blocked".to_owned(),
        (true, []) => "allowed".to_owned(),
        (true, networks) => format!("allowed: {}", networks.join(", ")),
    }
}

/// The state first, then each part.
pub fn rows(status: &Status, now: u64) -> Vec<Row> {
    let mut rows = Vec::new();
    let state = match (status.configured, status.on, status.wanted) {
        (false, _, _) => "not set up: kubaos proxy import FILE, or edit proxy.toml".to_owned(),
        (true, true, _) => "on".to_owned(),
        (true, false, true) => "wanted on, but not every part is up".to_owned(),
        (true, false, false) => "off".to_owned(),
    };
    rows.push(Row {
        label: "State",
        value: state,
    });
    if let Some(tunnel) = &status.tunnel {
        let kind = if tunnel.full { "full tunnel" } else { "split tunnel" };
        let guard = if tunnel.kill_switch { ", kill switch" } else { "" };
        let endpoint = tunnel.endpoint.as_deref().unwrap_or("no endpoint");
        let state = if tunnel.up { "up" } else { "down" };
        rows.push(Row {
            label: "Tunnel",
            value: format!("{state}: {kind} to {endpoint}{guard}"),
        });
        if tunnel.kill_switch {
            rows.push(Row {
                label: "Local network",
                value: local_network(tunnel),
            });
        }
        if let Some(handshake) = tunnel.handshake {
            rows.push(Row {
                label: "Last handshake",
                value: ago(handshake, now),
            });
        }
        if let (Some(rx), Some(tx)) = (tunnel.rx_bytes, tunnel.tx_bytes) {
            rows.push(Row {
                label: "Moved",
                value: format!("{} in, {} out", bytes(rx), bytes(tx)),
            });
        }
    }
    if !status.dns_servers.is_empty() || !status.dns_domains.is_empty() {
        let servers = if status.dns_servers.is_empty() {
            "the network's".to_owned()
        } else {
            status.dns_servers.join(", ")
        };
        let domains = if status.dns_domains.is_empty() {
            String::new()
        } else {
            format!(" for {}", status.dns_domains.join(", "))
        };
        rows.push(Row {
            label: "DNS",
            value: format!("{servers}{domains}"),
        });
    }
    if let Some(proxy) = &status.proxy {
        rows.push(Row {
            label: "HTTP proxy",
            value: proxy.clone(),
        });
    }
    if let Some(pac) = &status.pac {
        rows.push(Row {
            label: "PAC file",
            value: pac.clone(),
        });
    }
    if let Some(ca) = &status.ca {
        rows.push(Row {
            label: "Company CA",
            value: ca.clone(),
        });
    }
    for warning in &status.warnings {
        rows.push(Row {
            label: "Note",
            value: warning.clone(),
        });
    }
    if let Some(problem) = &status.problem {
        rows.push(Row {
            label: "Problem",
            value: problem.clone(),
        });
    }
    rows
}

#[cfg(test)]
mod tests;
