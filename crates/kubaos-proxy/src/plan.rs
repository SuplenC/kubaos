//! What a profile comes to on the machine: the parts to put in place, in the order they go up.
//!
//! ```text
//! VPN ─▶ root (kill switch, CA, apt, Docker, Nix, Chrome) ─▶ session env ─▶ GNOME ─▶ Docker client ─▶ git
//! ```
//!
//! The VPN goes first, so the kill switch never blocks the network before the tunnel is there,
//! and the proxies last, once what they reach is up. Taking them down goes the other way.
//!
//! # Owns
//!
//! - [`Plan`], its [`Part`]s and their [`Kind`]s, and the root helper's arguments ([`RootArg`]),
//!   the kill switch's among them ([`KillSwitch`], [`Server`]).
//!
//! # Not here
//!
//! - Putting them in place and taking them away: [`crate::apply`].

use std::fmt;
use std::net::IpAddr;

use crate::local_network;
use crate::net::Cidr;
use crate::profile::Dns;
use crate::profile::Profile;
use crate::vpn;

/// The variables the session gets, for the programs that read the proxy from the environment.
pub const ENV_NAMES: [&str; 6] = [
    "http_proxy",
    "https_proxy",
    "no_proxy",
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "NO_PROXY",
];

/// The root helper's parts, each of which `on` takes away when the plan no longer has it.
pub const ROOT_PARTS: [&str; 6] = ["killswitch", "ca", "apt", "docker", "nix", "chrome"];

/// A part of the Proxy, each put in place and taken away as one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    Tunnel,
    Root,
    Env,
    Gnome,
    DockerClient,
    Git,
}

impl Kind {
    /// In the order they go up.
    pub const ALL: [Kind; 6] = [
        Kind::Tunnel,
        Kind::Root,
        Kind::Env,
        Kind::Gnome,
        Kind::DockerClient,
        Kind::Git,
    ];

    /// Its name in the applied file.
    pub const fn name(self) -> &'static str {
        match self {
            Kind::Tunnel => "tunnel",
            Kind::Root => "root",
            Kind::Env => "env",
            Kind::Gnome => "gnome",
            Kind::DockerClient => "docker-client",
            Kind::Git => "git",
        }
    }

    pub fn from_name(name: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|kind| kind.name() == name)
    }

    /// What it is, for a person: "the session's proxy variables".
    pub const fn title(self) -> &'static str {
        match self {
            Kind::Tunnel => "the VPN",
            Kind::Root => "the system's parts (apt, Docker, Nix, Chrome, the CA, the kill switch)",
            Kind::Env => "the session's proxy variables",
            Kind::Gnome => "GNOME's proxy settings",
            Kind::DockerClient => "Docker's client proxy",
            Kind::Git => "git's proxy",
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.title())
    }
}

/// A VPN server the kill switch lets out to, by its address, port and protocol: how a VPN whose
/// packets carry no mark of their own (OpenVPN's) still reaches its server. WireGuard's packets
/// carry the tunnel's mark, which the kill switch lets out whatever their address.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Server {
    pub tcp: bool,
    pub addr: IpAddr,
    pub port: u16,
}

impl fmt::Display for Server {
    /// As the root helper takes it: `udp/198.51.100.7/1194`, `tcp/2001:db8::1/443`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let proto = if self.tcp { "tcp" } else { "udp" };
        write!(f, "{proto}/{}/{}", self.addr, self.port)
    }
}

/// What the kill switch lets out beside the tunnel, the tunnel's own packets aside.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KillSwitch {
    /// Nothing.
    Strict,
    /// The local networks: link-local ranges, discovery's multicast and broadcast, and these
    /// ranges of the machine's interfaces ([`crate::local_network::let_through`]).
    LocalNetwork(Vec<Cidr>),
}

/// One argument to the root helper, `system/files/proxy/kubaos-proxy-write`, which checks it
/// again.
#[derive(Clone, PartialEq, Eq)]
pub enum RootArg {
    /// What it lets through beside the tunnel, and the VPN's servers it lets out to.
    KillSwitch {
        through: KillSwitch,
        servers: Vec<Server>,
    },
    /// The certificate, in PEM, which goes on the helper's stdin.
    Ca(String),
    Apt(String),
    Docker {
        proxy: String,
        no_proxy: String,
    },
    Nix {
        proxy: String,
        no_proxy: String,
    },
    ChromeProxy {
        proxy: String,
        no_proxy: String,
    },
    ChromePac(String),
}

impl fmt::Debug for RootArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.arg())
    }
}

impl RootArg {
    /// Its part's name: `apt`.
    pub const fn name(&self) -> &'static str {
        match self {
            RootArg::KillSwitch { .. } => "killswitch",
            RootArg::Ca(_) => "ca",
            RootArg::Apt(_) => "apt",
            RootArg::Docker { .. } => "docker",
            RootArg::Nix { .. } => "nix",
            RootArg::ChromeProxy { .. } | RootArg::ChromePac(_) => "chrome",
        }
    }

    /// The argument that puts it in place: `apt=http://proxy:3128`, `killswitch=lan|192.168.1.0/24`,
    /// `killswitch=server:udp/198.51.100.7/1194|on`.
    pub fn arg(&self) -> String {
        let name = self.name();
        match self {
            RootArg::KillSwitch { through, servers } => {
                let to = if servers.is_empty() {
                    String::new()
                } else {
                    let servers: Vec<String> = servers.iter().map(ToString::to_string).collect();
                    format!("server:{}|", servers.join(","))
                };
                match through {
                    KillSwitch::Strict => format!("{name}={to}on"),
                    KillSwitch::LocalNetwork(networks) => {
                        let networks: Vec<String> = networks.iter().map(ToString::to_string).collect();
                        format!("{name}={to}lan|{}", networks.join(","))
                    }
                }
            }
            RootArg::Ca(_) => format!("{name}=-"),
            RootArg::Apt(proxy) => format!("{name}={proxy}"),
            RootArg::Docker { proxy, no_proxy } | RootArg::Nix { proxy, no_proxy } => {
                format!("{name}={proxy}|{no_proxy}")
            }
            RootArg::ChromeProxy { proxy, no_proxy } => format!("{name}=proxy|{proxy}|{no_proxy}"),
            RootArg::ChromePac(pac) => format!("{name}=pac|{pac}"),
        }
    }

    /// The argument that takes it away: `apt=off`.
    pub fn off(&self) -> String {
        format!("{}=off", self.name())
    }

    /// What goes on the helper's stdin.
    pub fn stdin(&self) -> Option<&str> {
        match self {
            RootArg::Ca(pem) => Some(pem),
            _ => None,
        }
    }
}

/// GNOME's proxy settings, which browsers and GIO programs follow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Gnome {
    Manual {
        host: String,
        port: u16,
        ignore: Vec<String>,
    },
    Auto {
        pac: String,
        ignore: Vec<String>,
    },
}

/// Docker's client proxies: what `docker run` and `docker build` hand their containers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DockerClient {
    pub proxy: String,
    pub no_proxy: String,
}

/// The VPN: its config without the secrets, and the work DNS on its link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tunnel {
    pub config: vpn::Config,
    pub dns: Dns,
}

/// One part, with what it puts in place.
#[derive(Clone, Debug, PartialEq, Eq)]
#[expect(
    clippy::large_enum_variant,
    reason = "A plan holds a handful of parts, made once a switch; boxing the VPN saves nothing."
)]
pub enum Part {
    Tunnel(Tunnel),
    Root(Vec<RootArg>),
    Env(Vec<(String, String)>),
    Gnome(Gnome),
    DockerClient(DockerClient),
    Git(String),
}

impl Part {
    pub const fn kind(&self) -> Kind {
        match self {
            Part::Tunnel(_) => Kind::Tunnel,
            Part::Root(_) => Kind::Root,
            Part::Env(_) => Kind::Env,
            Part::Gnome(_) => Kind::Gnome,
            Part::DockerClient(_) => Kind::DockerClient,
            Part::Git(_) => Kind::Git,
        }
    }
}

/// A profile's parts, in the order they go up.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    pub parts: Vec<Part>,
    /// What the profile asks that won't work as it may expect, for `status` and the import.
    pub warnings: Vec<String>,
}

impl Plan {
    /// What `profile` comes to.
    pub fn for_profile(profile: &Profile) -> Plan {
        let mut parts = Vec::new();
        let mut warnings = Vec::new();
        if let Some(tunnel) = &profile.tunnel {
            parts.push(Part::Tunnel(Tunnel {
                config: tunnel.config.clone(),
                dns: profile.dns.clone(),
            }));
            if tunnel.keys_on_disk {
                warnings.push(format!(
                    "{} holds the VPN's secrets in plain text: import it (kubaos proxy import) and delete \
                     that copy",
                    tunnel.file.display()
                ));
            }
            if tunnel.config.is_full() && profile.dns.servers.is_empty() {
                warnings.push(if profile.kill_switch() {
                    "a full tunnel without DNS servers: no name resolves, since the kill switch blocks \
                     the network's own resolver"
                        .to_owned()
                } else {
                    "a full tunnel without DNS servers: names resolve through the network you're on, \
                     outside the tunnel"
                        .to_owned()
                });
            }
        }
        let proxy = profile.http.proxy.as_ref().map(ToString::to_string);
        let no_proxy = profile.http.no_proxy.join(",");
        let mut root = Vec::new();
        if profile.kill_switch() {
            root.push(RootArg::KillSwitch {
                through: if profile.allow_local_network() {
                    KillSwitch::LocalNetwork(Vec::new())
                } else {
                    KillSwitch::Strict
                },
                servers: Vec::new(),
            });
        }
        if let Some(ca) = &profile.ca {
            root.push(RootArg::Ca(ca.pem.clone()));
        }
        if let Some(proxy) = &proxy {
            root.push(RootArg::Apt(proxy.clone()));
            root.push(RootArg::Docker {
                proxy: proxy.clone(),
                no_proxy: no_proxy.clone(),
            });
            root.push(RootArg::Nix {
                proxy: proxy.clone(),
                no_proxy: no_proxy.clone(),
            });
        }
        match (&profile.http.pac, &proxy) {
            (Some(pac), _) => root.push(RootArg::ChromePac(pac.clone())),
            (None, Some(proxy)) => root.push(RootArg::ChromeProxy {
                proxy: proxy.clone(),
                no_proxy: no_proxy.clone(),
            }),
            (None, None) => {}
        }
        if !root.is_empty() {
            parts.push(Part::Root(root));
        }
        if let Some(proxy) = &proxy {
            parts.push(Part::Env(env(proxy, &no_proxy)));
        }
        let ignore = profile.http.no_proxy.clone();
        match (&profile.http.pac, &profile.http.proxy) {
            (Some(pac), _) => parts.push(Part::Gnome(Gnome::Auto {
                pac: pac.clone(),
                ignore,
            })),
            (None, Some(url)) => parts.push(Part::Gnome(Gnome::Manual {
                host: url.host.clone(),
                port: url.port,
                ignore,
            })),
            (None, None) => {}
        }
        if profile.http.pac.is_some() && proxy.is_none() {
            warnings.push(
                "only a PAC file: GNOME and Chrome follow it, but apt, Docker, Nix, git and the shell \
                 need [http] proxy"
                    .to_owned(),
            );
        }
        if let Some(proxy) = &proxy {
            parts.push(Part::DockerClient(DockerClient {
                proxy: proxy.clone(),
                no_proxy,
            }));
            parts.push(Part::Git(proxy.clone()));
        }
        Plan { parts, warnings }
    }

    pub fn kinds(&self) -> Vec<Kind> {
        self.parts.iter().map(Part::kind).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// Whether the plan puts the kill switch up.
    pub fn has_kill_switch(&self) -> bool {
        self.kill_switch().is_some()
    }

    /// What the kill switch the plan puts up lets through beside the tunnel.
    pub fn kill_switch(&self) -> Option<&KillSwitch> {
        match self.kill_switch_arg()? {
            RootArg::KillSwitch { through, .. } => Some(through),
            _ => None,
        }
    }

    /// The root helper's argument for the kill switch the plan puts up.
    pub fn kill_switch_arg(&self) -> Option<&RootArg> {
        self.parts.iter().find_map(|part| match part {
            Part::Root(args) => args.iter().find(|arg| matches!(arg, RootArg::KillSwitch { .. })),
            _ => None,
        })
    }

    /// The VPN's config, when the plan has one.
    pub fn vpn(&self) -> Option<&vpn::Config> {
        self.parts.iter().find_map(|part| match part {
            Part::Tunnel(tunnel) => Some(&tunnel.config),
            _ => None,
        })
    }

    /// The kill switch's argument, changed by `change`.
    fn with_kill_switch(&self, change: impl Fn(&mut KillSwitch, &mut Vec<Server>)) -> Plan {
        let mut plan = self.clone();
        for part in &mut plan.parts {
            let Part::Root(args) = part else {
                continue;
            };
            for arg in args {
                if let RootArg::KillSwitch { through, servers } = arg {
                    change(through, servers);
                }
            }
        }
        plan
    }

    /// The plan with its kill switch letting through those of the `found` ranges that
    /// [`local_network::let_through`] allows beside its VPN, when it lets the local networks
    /// through at all. `routed` are the ranges the VPN's interface carries beside those its config
    /// names: the routes an OpenVPN server pushed.
    #[must_use]
    pub fn with_local_networks(&self, found: &[Cidr], routed: &[Cidr]) -> Plan {
        let Some(vpn) = self.vpn() else {
            return self.clone();
        };
        let mut behind = vpn.routes();
        behind.extend(routed.iter().filter(|cidr| !cidr.is_default()));
        self.with_kill_switch(|through, _| {
            if let KillSwitch::LocalNetwork(networks) = through {
                *networks = local_network::let_through(found, &behind);
            }
        })
    }

    /// The plan with its kill switch letting out to `servers`, the VPN's servers by address.
    #[must_use]
    pub fn with_servers(&self, servers: &[Server]) -> Plan {
        self.with_kill_switch(|_, to| *to = servers.to_vec())
    }
}

fn env(proxy: &str, no_proxy: &str) -> Vec<(String, String)> {
    ENV_NAMES
        .iter()
        .map(|name| {
            let value = if name.eq_ignore_ascii_case("no_proxy") {
                no_proxy
            } else {
                proxy
            };
            ((*name).to_owned(), value.to_owned())
        })
        .collect()
}

#[cfg(test)]
pub(crate) mod tests;
