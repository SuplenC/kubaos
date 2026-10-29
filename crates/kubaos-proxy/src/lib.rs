//! KubaOS's Proxy: one switch for the work network. A profile in the user layer
//! (`desktop/network/proxy.toml`) names up to four parts, each optional, and the switch takes all
//! of them up together or none: the VPN IT hands out (WireGuard or OpenVPN), the work DNS servers
//! and the domains they answer for, an HTTP proxy or a PAC file, and the company's root
//! certificate.
//!
//! ```text
//! control center, bar, menu, CLI ─▶ kubaos-proxy on|off ─▶ NetworkManager (the VPN, its DNS)
//!     ▲                                  │               ─▶ pkexec kubaos-proxy-write (apt, Docker,
//!     │                                  │                  Nix, Chrome, the CA, the kill switch)
//!     │                                  │               ─▶ systemd --user env, GNOME's proxy,
//!     │                                  │                  ~/.docker/config.json, git's include
//!     └── $XDG_RUNTIME_DIR/kubaos-proxy.json ◀── kubaos-proxy serve (at login, after a resume)
//! ```
//!
//! The VPN's secrets (WireGuard's keys; OpenVPN's certificate, keys and password) live in the
//! Bitwarden vault, and a copy in the GNOME keyring lets the switch work without the master
//! password. WireGuard's keys and OpenVPN's password are agent-owned secrets of an in-memory
//! connection, so NetworkManager saves them nowhere: as it brings the VPN up it asks for them, and
//! `kubaos-proxy`, its secret agent for as long as that takes, answers ([`agent`]).
//! NetworkManager's OpenVPN plugin takes certificates and keys as files alone, so those are in the
//! session's memory only while the plugin reads them as the VPN comes up.
//!
//! # Owns
//!
//! - The VPN's config of either type ([`vpn`], [`wireguard`], [`openvpn`], their addresses in
//!   [`net`]) and the profile ([`profile`]), where their files are ([`paths`]), what a profile
//!   comes to ([`plan`]), putting it in place and taking it away ([`apply`]), the secrets
//!   ([`secrets`]), the VPN as NetworkManager is given it ([`tunnel`]), who gets the secrets when
//!   NetworkManager asks ([`agent`]), the local networks the kill switch lets through
//!   ([`local_network`]), what the shell, the menu and `status` read ([`status`]), and when the
//!   service acts ([`watch`]).
//!
//! # Not here
//!
//! - The command line and the service: `main.rs`. The machine for real: [`real`].
//! - Writing as root: `system/files/proxy/kubaos-proxy-write`.
//! - The control center's tile and the bar's indicator: `config/quickshell/services/Proxy.qml`. The
//!   menu's Items: `crates/kotodama-kubaos/src/proxy.rs` and `config/kotodama/kubaos.toml`.

pub mod agent;
pub mod apply;
pub mod local_network;
pub mod machine;
pub mod net;
pub mod openvpn;
pub mod paths;
pub mod plan;
pub mod profile;
pub mod real;
pub mod secrets;
pub mod status;
pub mod tunnel;
pub mod vpn;
pub mod watch;
pub mod wireguard;

#[cfg(test)]
mod fake;

pub use self::machine::Machine;
pub use self::paths::Paths;
pub use self::plan::Plan;
pub use self::profile::Profile;
