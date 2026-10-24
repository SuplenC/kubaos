//! The KubaOS Extension of the Command menu. KubaOS's actions come from the
//! registry in its config table, which the Defaults fill and the user layer
//! adds to; the system toggles (Wi-Fi, Bluetooth, audio, brightness, power
//! profiles) talk to the services and programs directly, so they work with
//! the shell down.
//!
//! # Owns
//!
//! - The registry, the toggles, the keyboard layouts, the wallpaper sets, the user folders, the
//!   mounted drives, and what each of their Actions runs.
//! - The date answers, from the clock popup's engine.
//!
//! # Not here
//!
//! - The protocol loop: `main.rs`, on `kotodama_extension`.
//! - Power and session: Kotodama's Bundled `system` Extension.
//! - Hyprland's binds, which reach the search through the Bundled `hyprland` Extension.

#![forbid(unsafe_code)]

pub mod audio;
pub mod bus;
pub mod clock;
pub mod drives;
pub mod extension;
pub mod files;
pub mod keyboard;
pub mod registry;
pub mod run;
pub mod toggles;
pub mod updates;
pub mod wallpapers;

#[cfg(test)]
mod fake;
