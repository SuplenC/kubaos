//! KubaOS's Proxy: one switch for the work network. A profile in the user layer
//! (`desktop/network/proxy.toml`) names up to four parts, each optional, and the switch takes all
//! of them up together or none: the WireGuard tunnel IT hands out, the work DNS servers and the
//! domains they answer for, an HTTP proxy or a PAC file, and the company's root certificate.
//!
//! ```text
//! control center, bar, menu, CLI ─▶ kubaos-proxy on|off ─▶ NetworkManager (the tunnel, its DNS)
//!     ▲                                  │               ─▶ pkexec kubaos-proxy-write (apt, Docker,
//!     │                                  │                  Nix, Chrome, the CA, the kill switch)
//!     │                                  │               ─▶ systemd --user env, GNOME's proxy,
//!     │                                  │                  ~/.docker/config.json, git's include
//!     └── $XDG_RUNTIME_DIR/kubaos-proxy.json ◀── kubaos-proxy serve (at login, after a resume)
//! ```
//!
//! The tunnel's private and preshared keys live in the Bitwarden vault, and a copy in the GNOME
//! keyring lets the switch work without the master password. They reach NetworkManager in the
//! call that brings the tunnel up, as agent-owned secrets of an in-memory connection: they are
//! never written to disk in plain text.
//!
//! # Owns
//!
//! - The `.conf` ([`wireguard`]) and the profile ([`profile`]), where their files are ([`paths`]),
//!   what a profile comes to ([`plan`]), putting it in place and taking it away ([`apply`]), the
//!   keys ([`secrets`]), the tunnel as NetworkManager is given it ([`tunnel`]), and what the shell,
//!   the menu and `status` read ([`status`]), and when the service acts ([`watch`]).
//!
//! # Not here
//!
//! - The command line and the service: `main.rs`. The machine for real: [`real`].
//! - Writing as root: `system/files/proxy/kubaos-proxy-write`.
//! - The control center's tile and the bar's indicator: `config/quickshell/services/Proxy.qml`. The
//!   menu's Items: `crates/kotodama-kubaos/src/proxy.rs` and `config/kotodama/kubaos.toml`.

pub mod apply;
pub mod machine;
pub mod paths;
pub mod plan;
pub mod profile;
pub mod real;
pub mod secrets;
pub mod status;
pub mod tunnel;
pub mod watch;
pub mod wireguard;

#[cfg(test)]
mod fake;

pub use self::machine::Machine;
pub use self::paths::Paths;
pub use self::plan::Plan;
pub use self::profile::Profile;
