//! What the idle settings read from the machine: the power source, logind's lid and dock, and
//! whether the Hyprland session the holders belong to still runs. The readers take their folders
//! and the programs' output as arguments, so they are tested on files of the tests' own.
//!
//! # Owns
//!
//! - Reading `/sys/class/power_supply`, `busctl`'s answer about the lid, and Hyprland's lock file.
//!
//! # Not here
//!
//! - Running `busctl`: `main.rs`. What the lid does with them: [`crate::lid`].

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::settings::Power;

/// Where the kernel lists the power supplies.
pub const POWER_SUPPLY: &str = "/sys/class/power_supply";

/// The power source in use: plugged in when any supply of type `Mains` is online, as
/// `scripts/dot-hypr-on-ac` and logind tell. USB-C charging shows as `Mains` on the ACPI adapter
/// of most laptops. A machine with no supply listed counts as plugged in.
pub fn power(supplies: &Path) -> Power {
    let Ok(entries) = fs::read_dir(supplies) else {
        return Power::Ac;
    };
    let mut any_battery = false;
    for entry in entries.flatten() {
        let read = |name: &str| fs::read_to_string(entry.path().join(name)).unwrap_or_default();
        match read("type").trim() {
            "Mains" if read("online").trim() == "1" => return Power::Ac,
            "Battery" => any_battery = true,
            _ => {}
        }
    }
    if any_battery { Power::Battery } else { Power::Ac }
}

/// What logind says about the lid, from `busctl get-property org.freedesktop.login1
/// /org/freedesktop/login1 org.freedesktop.login1.Manager LidClosed Docked`: a `b true` or
/// `b false` line each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LidState {
    pub closed: bool,
    /// logind's own idea of docked: a dock, or more than one display connected. Closing the lid of
    /// a docked laptop never does anything.
    pub docked: bool,
}

impl LidState {
    /// `busctl`'s answer, `None` when it isn't two booleans.
    pub fn parse(busctl: &str) -> Option<LidState> {
        let mut values = busctl.lines().map(|line| match line.trim() {
            "b true" => Some(true),
            "b false" => Some(false),
            _ => None,
        });
        let closed = values.next()??;
        let docked = values.next()??;
        Some(LidState { closed, docked })
    }
}

/// The lock file of the Hyprland whose instance is `signature`, under the runtime dir.
pub fn hyprland_lock(runtime: &Path, signature: &str) -> PathBuf {
    runtime.join("hypr").join(signature).join("hyprland.lock")
}

/// Whether any Hyprland of this user runs, as their lock files under the runtime dir `runtime`
/// tell. The holders follow any, not the one they were started in: a program started from the
/// menu may carry the signature of a Hyprland that has since restarted.
pub fn any_hyprland_runs(runtime: &Path, proc: &Path) -> bool {
    let Ok(instances) = fs::read_dir(runtime.join("hypr")) else {
        return false;
    };
    instances
        .flatten()
        .any(|instance| hyprland_runs(&instance.path().join("hyprland.lock"), proc))
}

/// Whether the Hyprland with the lock file `lock` runs: its first line is Hyprland's pid, which
/// `proc` (`/proc`) still lists. A crashed Hyprland leaves the file with a dead pid.
pub fn hyprland_runs(lock: &Path, proc: &Path) -> bool {
    let Ok(text) = fs::read_to_string(lock) else {
        return false;
    };
    let Some(pid) = text.lines().next().and_then(|line| line.trim().parse::<u32>().ok()) else {
        return false;
    };
    proc.join(pid.to_string()).is_dir()
}

#[cfg(test)]
mod tests;
