//! The laptop's battery, from `/sys/class/power_supply`: its charge, the power it draws or takes,
//! the time to empty or full, and its health (what it holds now against what it was made to).
//!
//! A battery reports energy (µWh, with `power_now` in µW) or charge (µAh, with `current_now` in
//! µA and `voltage_now` in µV); both come out in watts and seconds.
//!
//! # Owns
//!
//! - Finding the system battery and reading it.
//!
//! # Not here
//!
//! - The power profile, which the shell switches through power-profiles-daemon:
//!   `config/quickshell/services/Profiles.qml`.

use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use crate::read;

/// The battery now.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Battery {
    pub percent: f32,
    /// `Charging`, `Discharging`, `Full` or `Not charging`, as the kernel says.
    pub status: String,
    /// What it draws or takes, in watts.
    pub watts: Option<f32>,
    /// To empty while discharging, to full while charging.
    pub seconds_left: Option<u64>,
    /// What it holds when full, as a percentage of what it was made to hold.
    pub health: Option<f32>,
}

/// The system battery under `root`, the first if there are two; `None` on a desktop.
pub fn read_battery(root: &Path) -> Option<Battery> {
    let class = root.join("sys/class/power_supply");
    read::entries(&class).into_iter().find_map(|entry| {
        let dir = class.join(entry);
        let system = read::line(&dir.join("type")).as_deref() == Some("Battery")
            && read::line(&dir.join("scope")).as_deref() != Some("Device");
        system.then(|| battery(&dir))
    })
}

fn battery(dir: &Path) -> Battery {
    let micro = |name: &str| read::number::<u64>(&dir.join(name));
    let status = read::line(&dir.join("status")).unwrap_or_else(|| "Unknown".to_owned());
    // Energy in µWh and power in µW, or charge in µAh and current in µA: the sums are alike.
    let (now, full, design, rate, volts) = if micro("energy_full").is_some() {
        (
            micro("energy_now"),
            micro("energy_full"),
            micro("energy_full_design"),
            micro("power_now"),
            None,
        )
    } else {
        let volts = micro("voltage_now").map(|uv| uv as f32 / 1e6);
        (
            micro("charge_now"),
            micro("charge_full"),
            micro("charge_full_design"),
            micro("current_now"),
            volts,
        )
    };
    let watts = rate.map(|rate| match volts {
        Some(volts) => rate as f32 / 1e6 * volts,
        None => rate as f32 / 1e6,
    });
    let percent = read::number::<f32>(&dir.join("capacity"))
        .or_else(|| Some(now? as f32 * 100.0 / full.filter(|full| *full > 0)? as f32))
        .unwrap_or(0.0);
    let seconds_left = match (status.as_str(), now, full, rate) {
        (_, _, _, None | Some(0)) => None,
        ("Discharging", Some(now), _, Some(rate)) => Some(now * 3600 / rate),
        ("Charging", Some(now), Some(full), Some(rate)) => Some(full.saturating_sub(now) * 3600 / rate),
        _ => None,
    };
    let health = match (full, design) {
        (Some(full), Some(design)) if design > 0 => Some(full as f32 * 100.0 / design as f32),
        _ => None,
    };
    Battery {
        percent,
        status,
        watts: watts.filter(|watts| *watts > 0.0),
        seconds_left,
        health,
    }
}

#[cfg(test)]
mod tests;
