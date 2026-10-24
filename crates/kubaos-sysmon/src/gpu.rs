//! Graphics cards, from DRM's sysfs (`/sys/class/drm/card*/device`).
//!
//! amdgpu exposes its load (`gpu_busy_percent`), VRAM and, through its hwmon, temperature and
//! shader clock. Another card (a discrete NVIDIA one) still gets a row, with what its driver
//! exposes, which may be no more than whether it sleeps.
//!
//! # Owns
//!
//! - Finding the cards and reading them.
//!
//! # Not here
//!
//! - The CPU's sensors: [`crate::hwmon`].

use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::hwmon;
use crate::read;

/// A graphics card now.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gpu {
    /// `amd`, `nvidia`, `intel` or `other`.
    pub vendor: String,
    /// Not the card the machine booted its display on.
    pub discrete: bool,
    /// Awake: a discrete card asleep in runtime power management shows nothing else.
    pub active: bool,
    /// Busy, in percent.
    pub usage: Option<f32>,
    pub vram_used: Option<u64>,
    pub vram_total: Option<u64>,
    pub temp: Option<f32>,
    /// The shader clock.
    pub mhz: Option<u32>,
}

#[derive(Clone, Debug)]
struct Card {
    vendor: &'static str,
    discrete: bool,
    device: PathBuf,
    hwmon: Option<PathBuf>,
}

/// The cards found at the start.
#[derive(Clone, Debug, Default)]
pub struct Gpus {
    cards: Vec<Card>,
}

impl Gpus {
    /// Looks for the cards under `root`, the boot display's card first.
    pub fn discover(root: &Path) -> Self {
        let class = root.join("sys/class/drm");
        let mut cards: Vec<Card> = read::entries(&class)
            .into_iter()
            .filter(|entry| {
                entry
                    .strip_prefix("card")
                    .is_some_and(|n| n.bytes().all(|b| b.is_ascii_digit()))
            })
            .filter_map(|entry| {
                let device = class.join(entry).join("device");
                let vendor = match read::line(&device.join("vendor"))?.as_str() {
                    "0x1002" => "amd",
                    "0x10de" => "nvidia",
                    "0x8086" => "intel",
                    _ => "other",
                };
                let discrete = vendor == "nvidia" || read::line(&device.join("boot_vga")).as_deref() == Some("0");
                let hwmon_dir = device.join("hwmon");
                let hwmon = read::entries(&hwmon_dir).first().map(|entry| hwmon_dir.join(entry));
                Some(Card {
                    vendor,
                    discrete,
                    device,
                    hwmon,
                })
            })
            .collect();
        cards.sort_by_key(|card| card.discrete);
        Gpus { cards }
    }

    /// Each card now.
    pub fn read(&self) -> Vec<Gpu> {
        self.cards.iter().map(Card::read).collect()
    }
}

impl Card {
    fn read(&self) -> Gpu {
        let mut gpu = Gpu {
            vendor: self.vendor.to_owned(),
            discrete: self.discrete,
            active: read::line(&self.device.join("power/runtime_status")).as_deref() != Some("suspended"),
            ..Gpu::default()
        };
        // HAZARD: a card asleep in runtime power management wakes up when any of its counters is
        // read, and a woken discrete GPU costs watts; a sleeping card is left alone.
        if !gpu.active {
            return gpu;
        }
        gpu.usage = read::number(&self.device.join("gpu_busy_percent"));
        gpu.vram_used = read::number(&self.device.join("mem_info_vram_used"));
        gpu.vram_total = read::number(&self.device.join("mem_info_vram_total"));
        gpu.temp = self
            .hwmon
            .as_deref()
            .and_then(|dir| hwmon::millidegrees(&dir.join("temp1_input")));
        gpu.mhz = self
            .hwmon
            .as_deref()
            .and_then(|dir| read::number::<u64>(&dir.join("freq1_input")))
            .map(|hz| (hz / 1_000_000) as u32)
            .or_else(|| starred_level(&self.device.join("pp_dpm_sclk")));
        gpu
    }
}

/// The level marked `*` in a `pp_dpm_sclk` table (`1: 1200Mhz *`), in MHz.
fn starred_level(path: &Path) -> Option<u32> {
    let text = read::text(path)?;
    let line = text.lines().find(|line| line.trim_end().ends_with('*'))?;
    let speed = line.split_ascii_whitespace().nth(1)?;
    speed.trim_end_matches(|c: char| c.is_ascii_alphabetic()).parse().ok()
}

#[cfg(test)]
mod tests;
