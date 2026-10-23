//! KubaOS's Bluetooth pairing agent. bluetoothd asks an agent for every code a pairing needs, and
//! with none registered a keyboard waiting for its passkey never pairs. This one registers as the
//! default agent and asks the user in the shell; it also pairs and scans for the shell, so every
//! pairing ends and a scan can always start again.
//!
//! ```text
//! shell ──kubaos-bluetooth pair/answer/scan──▶ session bus ──▶ agent ──Pair, StartDiscovery──▶ bluetoothd
//!   ▲                                                            │  ◀──RequestPasskey, Cancel──
//!   └──────────── $XDG_RUNTIME_DIR/kubaos-bluetooth.json ◀───────┘
//! ```
//!
//! # Owns
//!
//! - The state the shell reads ([`state`]) and the one place it changes ([`hub`]).
//! - The agent's answers ([`agent`]), pairing ([`pairing`]) and scanning ([`scan`]).
//! - bluetoothd's side ([`bluez`]) and the shell's ([`service`]).
//!
//! # Not here
//!
//! - The prompt's look: `config/quickshell/modules/controlcenter/BluetoothList.qml`.
//! - The user unit: `home/bluetooth.nix`.

#![forbid(unsafe_code)]

pub mod agent;
pub mod bluez;
pub mod hub;
pub mod pairing;
pub mod scan;
pub mod service;
pub mod state;

#[cfg(test)]
mod fake;
