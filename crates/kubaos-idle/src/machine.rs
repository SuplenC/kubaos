//! What the idle settings read from the machine: the power source and the battery's level,
//! logind's lid and dock, and whether the Hyprland session the holders belong to still runs. The
//! readers take their folders and the programs' output as arguments, so they are tested on files
//! of the tests' own.
//!
//! # Owns
//!
//! - Reading `/sys/class/power_supply`, `busctl`'s answer about the lid, and Hyprland's lock file.
//!
//! # Not here
//!
//! - Running `busctl`: `main.rs`. What the lid does with them: [`crate::lid`]. When the battery
//!   ends a Keep awake: [`crate::awake`].

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

/// The machine's battery level in percent, its batteries combined; `None` without one that
/// reports a charge. A battery of a device (a mouse's, `scope` `Device`) or an empty bay (`present`
/// `0`) isn't the machine's. Batteries that all report their energy combine by it, so a full small
/// one and an empty big one don't read half full, as UPower combines them; otherwise their
/// `capacity` is averaged.
pub fn battery(supplies: &Path) -> Option<u8> {
    let entries = fs::read_dir(supplies).ok()?;
    let (mut capacities, mut count) = (0_u64, 0_u64);
    let mut energies = Some((0_u64, 0_u64));
    for entry in entries.flatten() {
        let read = |name: &str| fs::read_to_string(entry.path().join(name)).ok();
        let word = |name: &str| read(name).map(|text| text.trim().to_owned());
        let number = |name: &str| read(name).and_then(|text| text.trim().parse::<u64>().ok());
        if word("type").as_deref() != Some("Battery")
            || word("scope").as_deref() == Some("Device")
            || word("present").as_deref() == Some("0")
        {
            continue;
        }
        let Some(capacity) = number("capacity") else {
            continue;
        };
        capacities += capacity.min(100);
        count += 1;
        energies = energies
            .zip(number("energy_now").zip(number("energy_full").filter(|&full| full > 0)))
            .map(|((now, full), (more_now, more_full))| (now + more_now, full + more_full));
    }
    if count == 0 {
        return None;
    }
    let percent = match energies {
        Some((now, full)) => (now.min(full) * 100 + full / 2) / full,
        None => (capacities + count / 2) / count,
    };
    u8::try_from(percent).ok()
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
    running_hyprland(runtime, proc).is_some()
}

/// The instance signature of a Hyprland of this user that runs, to send it a dispatch with
/// `hyprctl -i`. A user has one session, so one runs; a crashed one left behind doesn't count.
pub fn running_hyprland(runtime: &Path, proc: &Path) -> Option<String> {
    let instances = fs::read_dir(runtime.join("hypr")).ok()?;
    instances
        .flatten()
        .find(|instance| hyprland_runs(&instance.path().join("hyprland.lock"), proc))
        .map(|instance| instance.file_name().to_string_lossy().into_owned())
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
