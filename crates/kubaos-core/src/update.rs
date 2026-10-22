//! Updating an installed KubaOS: `kubaos update`, `upgrade`, `rollback`, `channel` and the daily
//! check behind the update notification.
//!
//! The machine's KubaOS is the root-owned checkout `/usr/share/kubaos`; its `release` branch is
//! what the user flake builds. An update fetches `origin`, lets the channel's release policy pick
//! the commit `release` moves to, upgrades apt, runs the system migrations not run yet, updates
//! and switches the user flake, then runs the user migrations. `/var/lib/kubaos/release` records
//! what is installed and `update-history` what each update changed, which `rollback` undoes.
//!
//! # Owns
//!
//! - [`Updater`], the steps; the release-policy seam (`policy.rs`) and the git reading it uses.
//! - The files under `/etc/kubaos` and `/var/lib/kubaos`, and the user's update state.
//!
//! # Not here
//!
//! - The home-manager switch: `switch.rs`. Writing the records at install: `engine/install.rs`.

pub mod git;
pub mod hyprland;
pub mod machine;
pub mod migrations;
pub mod notice;
pub mod policy;
pub mod record;
pub mod version;

mod updater;

pub use updater::MIN_FREE_BYTES;
pub use updater::UpdateError;
pub use updater::UpdateOptions;
pub use updater::Updater;

#[cfg(test)]
mod fixture;
