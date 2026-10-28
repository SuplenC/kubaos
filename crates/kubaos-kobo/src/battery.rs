//! The laptop's battery, as the kernel reports it under `/sys/class/power_supply`. The live image
//! runs no UPower, so Kōbō reads sysfs itself.
//!
//! # Owns
//!
//! - Reading every system battery and the AC adapter, and combining them into one `Battery`.
//! - When a battery counts as low.
//!
//! # Not here
//!
//! - How often it's read and on which thread: `App::subscription` in `app.rs`.
//! - Drawing it: `widgets::battery` and `view.rs`.

use std::fs;
use std::path::Path;
use std::time::Duration;

/// Where the kernel lists power supplies.
pub const SYSFS: &str = "/sys/class/power_supply";

/// How often the battery is read. A percent moves in minutes, and plugging the charger in shows
/// within this long.
pub const POLL: Duration = Duration::from_secs(15);

/// At or below this percent, discharging, the battery is low: an install takes a quarter of an
/// hour or more, and a machine that dies halfway leaves a disk that won't boot.
pub const LOW_PERCENT: u8 = 20;

/// Where the machine's power comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Supply {
    Charging,
    /// On the charger, not charging: full, or held below full by the firmware.
    PluggedIn,
    Discharging,
}

/// Every system battery of the machine, combined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Battery {
    /// 0 to 100.
    pub percent: u8,
    pub supply: Supply,
}

impl Battery {
    #[must_use]
    pub fn on_charger(self) -> bool {
        self.supply != Supply::Discharging
    }

    #[must_use]
    pub fn is_low(self) -> bool {
        self.supply == Supply::Discharging && self.percent <= LOW_PERCENT
    }
}

/// The machine's batteries under `root` (`SYSFS` on a real machine), or `None` when it has none:
/// a desktop or a VM. A file that is missing or unreadable counts as not reported.
#[must_use]
pub fn read(root: &Path) -> Option<Battery> {
    let mut cells = Vec::new();
    let mut mains_online = false;
    for entry in fs::read_dir(root).ok()?.flatten() {
        let dir = entry.path();
        match attr(&dir, "type").as_deref() {
            Some("Battery") => cells.extend(Cell::read(&dir)),
            Some("Mains") => mains_online |= attr(&dir, "online").as_deref() == Some("1"),
            _ => {}
        }
    }
    combine(&cells, mains_online)
}

/// One battery's own report.
#[derive(Debug)]
struct Cell {
    /// `capacity`, in percent.
    capacity: Option<u8>,
    /// `energy_now` and `energy_full`, in µWh.
    energy: Option<(u64, u64)>,
    status: String,
}

impl Cell {
    /// `None` for a battery that isn't the machine's (a wireless mouse's reports `scope` `Device`),
    /// an empty bay (`present` `0`), or one that reports no charge at all.
    fn read(dir: &Path) -> Option<Self> {
        if attr(dir, "scope").as_deref() == Some("Device") || attr(dir, "present").as_deref() == Some("0") {
            return None;
        }
        let capacity = attr(dir, "capacity")
            .and_then(|c| c.parse::<u8>().ok())
            .map(|c| c.min(100));
        let energy = number(dir, "energy_now")
            .zip(number(dir, "energy_full"))
            .filter(|&(_, full)| full > 0);
        if capacity.is_none() && energy.is_none() {
            return None;
        }
        Some(Self {
            capacity,
            energy,
            status: attr(dir, "status").unwrap_or_default(),
        })
    }

    fn percent(&self) -> u8 {
        match (self.capacity, self.energy) {
            (Some(c), _) => c,
            (None, Some((now, full))) => ratio(now, full),
            (None, None) => 0,
        }
    }
}

/// Batteries that all report energy combine by it, so a full small one and an empty big one
/// aren't shown half full. Otherwise their percents are averaged.
fn combine(cells: &[Cell], mains_online: bool) -> Option<Battery> {
    if cells.is_empty() {
        return None;
    }
    let energies: Option<Vec<(u64, u64)>> = cells.iter().map(|c| c.energy).collect();
    let percent = match energies {
        Some(e) => ratio(e.iter().map(|x| x.0).sum(), e.iter().map(|x| x.1).sum()),
        None => {
            let sum: usize = cells.iter().map(|c| usize::from(c.percent())).sum();
            let avg = (sum + cells.len() / 2) / cells.len();
            u8::try_from(avg).unwrap_or(100)
        }
    };
    Some(Battery {
        percent,
        supply: supply(cells, mains_online),
    })
}

/// A machine without a `Mains` supply still reports `Full` or `Not charging` on the charger, and
/// one with two batteries may charge one while the other idles.
fn supply(cells: &[Cell], mains_online: bool) -> Supply {
    let any = |s: &str| cells.iter().any(|c| c.status == s);
    if any("Charging") {
        Supply::Charging
    } else if mains_online {
        Supply::PluggedIn
    } else if any("Discharging") {
        Supply::Discharging
    } else if any("Full") || any("Not charging") {
        Supply::PluggedIn
    } else {
        Supply::Discharging
    }
}

/// `now / full` in whole percent, rounded and capped at 100. A worn battery's `energy_now` can
/// pass its `energy_full`.
fn ratio(now: u64, full: u64) -> u8 {
    if full == 0 {
        return 0;
    }
    let pct = (u128::from(now) * 100 + u128::from(full) / 2) / u128::from(full);
    u8::try_from(pct.min(100)).unwrap_or(100)
}

fn attr(dir: &Path, name: &str) -> Option<String> {
    fs::read_to_string(dir.join(name)).ok().map(|s| s.trim().to_string())
}

fn number(dir: &Path, name: &str) -> Option<u64> {
    attr(dir, name)?.parse().ok()
}

#[cfg(test)]
mod tests;
