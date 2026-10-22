//! The KubaOS Extension of the Command menu. KubaOS's actions come from the
//! registry in its config table, which the Defaults fill and the user layer
//! adds to; the system toggles (Wi-Fi, Bluetooth, audio, brightness, power
//! profiles) talk to the services and programs directly, so they work with
//! the shell down.
//!
//! # Owns
//!
//! - The registry, the toggles, and what each of their Actions runs.
//!
//! # Not here
//!
//! - The protocol loop: `main.rs`, on `kotodama_extension`.
//! - Power and session: Kotodama's Bundled `system` Extension.
//! - Hyprland's binds, which reach the search through the Bundled `hyprland` Extension.

#![forbid(unsafe_code)]

pub mod audio;
pub mod bus;
pub mod extension;
pub mod registry;
pub mod run;
pub mod toggles;

#[cfg(test)]
mod fake;
