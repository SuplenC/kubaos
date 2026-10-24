//! The processor: the ticks in `/proc/stat`, the load average, and the cores' clock speed.
//!
//! Usage is a share of the ticks that went by between two reads of `/proc/stat`, so the first
//! sample has none.
//!
//! # Owns
//!
//! - Parsing `/proc/stat` and `/proc/loadavg`, and reading `scaling_cur_freq`.
//!
//! # Not here
//!
//! - The temperature: [`crate::hwmon`]. Keeping the last read: [`crate::sampler`].

use std::path::Path;

use crate::read;

/// The ticks a processor (or all of them) has counted since boot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Times {
    /// Every tick, busy or idle.
    pub total: u64,
    /// Ticks idle or waiting for I/O.
    pub idle: u64,
}

impl Times {
    /// The busy share of the ticks since `earlier`, in percent; `None` when no tick went by.
    pub fn usage_since(&self, earlier: &Times) -> Option<f32> {
        let total = self.total.checked_sub(earlier.total)?;
        if total == 0 {
            return None;
        }
        let idle = self.idle.saturating_sub(earlier.idle).min(total);
        Some((total - idle) as f32 * 100.0 / total as f32)
    }
}

/// What `/proc/stat` says.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stat {
    pub total: Times,
    pub cores: Vec<Times>,
    /// When the machine booted, in seconds since the epoch.
    pub boot_time: Option<u64>,
}

/// Parses `/proc/stat`.
pub fn parse_stat(text: &str) -> Stat {
    let mut stat = Stat::default();
    for line in text.lines() {
        let mut fields = line.split_ascii_whitespace();
        let Some(name) = fields.next() else {
            continue;
        };
        if name == "cpu" {
            stat.total = times(fields);
        } else if name.starts_with("cpu") {
            stat.cores.push(times(fields));
        } else if name == "btime" {
            stat.boot_time = fields.next().and_then(|value| value.parse().ok());
        }
    }
    stat
}

/// A `cpu` line's columns: user, nice, system, idle, iowait, irq, softirq, steal, guest,
/// `guest_nice`. The guest columns are left out, since `user` and `nice` count them already.
fn times<'a>(fields: impl Iterator<Item = &'a str>) -> Times {
    let columns: Vec<u64> = fields.take(8).map(|value| value.parse().unwrap_or(0)).collect();
    let column = |index: usize| columns.get(index).copied().unwrap_or(0);
    Times {
        total: columns.iter().sum(),
        idle: column(3) + column(4),
    }
}

/// The load average over 1, 5 and 15 minutes, from `/proc/loadavg`.
pub fn parse_loadavg(text: &str) -> Option<[f32; 3]> {
    let mut fields = text.split_ascii_whitespace().map(str::parse::<f32>);
    Some([fields.next()?.ok()?, fields.next()?.ok()?, fields.next()?.ok()?])
}

/// The average clock speed of the first `cores` cores in MHz, from cpufreq; `None` without it (a
/// virtual machine, a container on another kernel).
pub fn frequency_mhz(root: &Path, cores: usize) -> Option<u32> {
    let dir = root.join("sys/devices/system/cpu");
    let speeds: Vec<u64> = (0..cores)
        .filter_map(|core| read::number::<u64>(&dir.join(format!("cpu{core}/cpufreq/scaling_cur_freq"))))
        .collect();
    if speeds.is_empty() {
        return None;
    }
    Some((speeds.iter().sum::<u64>() / speeds.len() as u64 / 1000) as u32)
}

#[cfg(test)]
mod tests;
