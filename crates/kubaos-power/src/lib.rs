//! KubaOS's power stops: a slider of five fixed stops, Super Saver · Power Saver · Balanced ·
//! Performance · Max Performance, each defined across every layer of the machine, on top of
//! power-profiles-daemon, which stays the one profile switch other tools see.
//!
//! ```text
//! slider, menu, CLI ─set─▶ kubaos-power serve ─▶ power-profiles-daemon (ActiveProfile)
//!          ppd, UPower ───▶      │  (Control)  ─▶ pkexec kubaos-power-write (sysfs, as root)
//!                                │             ─▶ hyprctl eval dot.power_panel() (refresh, VRR)
//!                                └─▶ $XDG_RUNTIME_DIR/kubaos-power.json ─▶ the shell
//! ```
//!
//! # Owns
//!
//! - The stops ([`stop`]) and what each sets ([`plan`], the one place a stop becomes its
//!   settings), the user's settings ([`settings`]), what the service remembers ([`memory`]) and
//!   tells the shell ([`runtime`]), which stop to be on ([`control`]), what to write on this
//!   machine ([`machine`], [`writes`], [`panel`]), `status` ([`status`]) and `measure`
//!   ([`measure`]).
//!
//! # Not here
//!
//! - The service's loop, the bus and the command line: `main.rs`.
//! - Writing as root: `system/files/power/kubaos-power-write`. The panel's monitor rule:
//!   `config/hypr/monitors.lua`.
//! - The slider: `config/quickshell/services/Power.qml` and the control center. The menu's Items:
//!   `crates/kotodama-kubaos/src/power.rs` and `config/kotodama/kubaos.toml`.

#![forbid(unsafe_code)]

pub mod control;
pub mod machine;
pub mod measure;
pub mod memory;
pub mod panel;
pub mod plan;
pub mod runtime;
pub mod settings;
pub mod status;
pub mod stop;
pub mod writes;

#[cfg(test)]
mod fake;
