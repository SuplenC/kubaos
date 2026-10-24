//! Temperatures and fans, from the kernel's hardware monitors (`/sys/class/hwmon`).
//!
//! The chips are looked for once, and the files the sample reads are kept, so a sample reads one
//! file a sensor. AMD's CPU temperature is `k10temp`'s Tctl, Intel's `coretemp`'s package; a
//! thermal zone stands in where neither is there. Fans are every `fan*_input`, which is where
//! TUXEDO's drivers put theirs too.
//!
//! # Owns
//!
//! - Finding the CPU temperature and the fans, and reading them.
//!
//! # Not here
//!
//! - The GPU's temperature, read from its own device: [`crate::gpu`].

use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::read;

/// CPU temperature chips, best first, each with the label of the sensor to read; `None` reads
/// `temp1`.
const CPU_CHIPS: [(&str, Option<&str>); 6] = [
    ("k10temp", Some("Tctl")),
    ("zenpower", Some("Tdie")),
    ("k10temp", None),
    ("coretemp", Some("Package id 0")),
    ("coretemp", None),
    ("cpu_thermal", None),
];

/// Thermal zone types that measure the processor, best first.
const CPU_ZONES: [&str; 3] = ["x86_pkg_temp", "cpu-thermal", "acpitz"];

/// A fan and its speed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fan {
    pub label: String,
    pub rpm: u32,
}

#[derive(Clone, Debug)]
struct FanSource {
    label: String,
    input: PathBuf,
}

/// The sensor files a sample reads.
#[derive(Clone, Debug, Default)]
pub struct Sensors {
    /// Millidegrees Celsius.
    cpu_temperature: Option<PathBuf>,
    fans: Vec<FanSource>,
}

/// A chip under `/sys/class/hwmon`: its name and folder.
struct Chip {
    name: String,
    dir: PathBuf,
}

impl Sensors {
    /// Looks for the chips under `root`.
    pub fn discover(root: &Path) -> Self {
        let class = root.join("sys/class/hwmon");
        let chips: Vec<Chip> = read::entries(&class)
            .into_iter()
            .filter_map(|entry| {
                let dir = class.join(entry);
                Some(Chip {
                    name: read::line(&dir.join("name"))?,
                    dir,
                })
            })
            .collect();
        Sensors {
            cpu_temperature: cpu_sensor(&chips).or_else(|| cpu_zone(root)),
            fans: chips.iter().flat_map(fan_sources).collect(),
        }
    }

    /// The CPU's temperature in °C.
    pub fn cpu_temperature(&self) -> Option<f32> {
        millidegrees(self.cpu_temperature.as_deref()?)
    }

    /// Each fan's speed now.
    pub fn fans(&self) -> Vec<Fan> {
        self.fans
            .iter()
            .filter_map(|fan| {
                Some(Fan {
                    label: fan.label.clone(),
                    rpm: read::number(&fan.input)?,
                })
            })
            .collect()
    }
}

/// A temperature file's millidegrees, in °C.
pub(crate) fn millidegrees(path: &Path) -> Option<f32> {
    Some(read::number::<i64>(path)? as f32 / 1000.0)
}

fn cpu_sensor(chips: &[Chip]) -> Option<PathBuf> {
    CPU_CHIPS.iter().find_map(|(name, label)| {
        chips
            .iter()
            .filter(|chip| chip.name == *name)
            .find_map(|chip| match label {
                Some(label) => labelled(&chip.dir, "temp", label),
                None => Some(chip.dir.join("temp1_input")).filter(|path| path.exists()),
            })
    })
}

/// The `<kind>N_input` of `dir` whose `<kind>N_label` is `label`.
fn labelled(dir: &Path, kind: &str, label: &str) -> Option<PathBuf> {
    read::entries(dir).into_iter().find_map(|entry| {
        let index = entry.strip_prefix(kind)?.strip_suffix("_label")?;
        let found = read::line(&dir.join(&entry))?;
        (found == label).then(|| dir.join(format!("{kind}{index}_input")))
    })
}

fn cpu_zone(root: &Path) -> Option<PathBuf> {
    let class = root.join("sys/class/thermal");
    let zones: Vec<(String, PathBuf)> = read::entries(&class)
        .into_iter()
        .filter(|entry| entry.starts_with("thermal_zone"))
        .filter_map(|entry| {
            let dir = class.join(entry);
            Some((read::line(&dir.join("type"))?, dir.join("temp")))
        })
        .collect();
    CPU_ZONES
        .iter()
        .find_map(|kind| zones.iter().find(|(found, _)| found == kind))
        .map(|(_, temp)| temp.clone())
}

fn fan_sources(chip: &Chip) -> Vec<FanSource> {
    read::entries(&chip.dir)
        .into_iter()
        .filter_map(|entry| {
            let index = entry.strip_prefix("fan")?.strip_suffix("_input")?;
            let label = read::line(&chip.dir.join(format!("fan{index}_label")))
                .unwrap_or_else(|| format!("{} fan {index}", chip.name));
            Some(FanSource {
                label,
                input: chip.dir.join(&entry),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
