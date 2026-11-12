//! KubaOS's launch watcher. An app takes a moment to show its window, and Hyprland opens it on
//! whichever workspace has focus by then, not the one it was launched from. Hyprland can follow a
//! program it starts itself, but launchers have systemd start apps as services of their own, and an
//! app already running hands a launch to the process it has. This service follows the systemd user
//! manager for each app's new service and tells Hyprland of the launch at once, so Hyprland's
//! config (`config/hypr/launches.lua`) notes the workspace and places the app's window there.
//!
//! ```text
//! launcher ──StartTransientUnit──▶ systemd ──UnitNew──▶ service ──eval──▶ Hyprland
//! ```
//!
//! # Owns
//!
//! - Which units are an app's launch and the app's id ([`unit`]), the setting ([`settings`]),
//!   telling Hyprland ([`hyprland`]) and the loop ([`service`]).
//!
//! # Not here
//!
//! - Where the window goes: `config/hypr/launches.lua`.
//! - The user unit: `home/launches.nix`.
//! - The Command menu's Items for the setting: `config/kotodama/kubaos.toml`.

pub mod hyprland;
pub mod service;
pub mod settings;
pub mod unit;
