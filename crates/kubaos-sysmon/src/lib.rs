//! KubaOS's system monitor: CPU, memory, GPU, disks, network, battery, fans and processes, read
//! straight from `/proc` and `/sys`, with no process started for a sample. One stream feeds the
//! bar's pill, its popup and the Command menu.
//!
//! ```text
//! /proc, /sys ─▶ Sampler ─▶ kubaos-sysmon serve ─{"type":"sample",...}─▶ Sysmon.qml ─▶ pill, popup
//!                                  │      ▲                                   │
//!                                  │      └──{"op":"mode","mode":"popup"}─────┘
//!                                  └─▶ $XDG_RUNTIME_DIR/kubaos-sysmon.json ─▶ kotodama-kubaos (`cpu`)
//! ```
//!
//! # Owns
//!
//! - Reading each source ([`cpu`], [`memory`], [`gpu`], [`disks`], [`net`], [`power`], [`hwmon`],
//!   [`processes`]), the sample made of them ([`sampler`]), the alerts ([`alerts`]), the settings
//!   in the user layer ([`settings`]) and the shell's stream ([`serve`]).
//!
//! # Not here
//!
//! - The pill and the popup: `config/quickshell/modules/bar/SysmonPill.qml` and
//!   `config/quickshell/modules/sysmon/`. The menu's answers and process search:
//!   `crates/kotodama-kubaos/src/sysmon.rs`; its Items: `config/kotodama/kubaos.toml`.

pub mod alerts;
pub mod cpu;
pub mod disks;
pub mod format;
pub mod gpu;
pub mod hwmon;
pub mod memory;
pub mod net;
pub mod power;
pub mod processes;
mod read;
pub mod sampler;
pub mod serve;
pub mod settings;

#[cfg(test)]
mod fixture;

pub use self::sampler::Sample;
pub use self::sampler::Sampler;
