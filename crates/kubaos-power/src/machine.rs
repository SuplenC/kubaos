//! What the machine has to turn, read from `/sys` and `/proc` under a root that the tests point at
//! a folder of their own. Each reader returns nothing where the machine has no such knob, so a
//! stop skips that row and `status` says so.
//!
//! # Owns
//!
//! - [`Machine`]: finding the CPU's policies, the GPUs, the laptop panels, the PCIe ASPM policy,
//!   the USB devices, the Wi-Fi interfaces, the charge limit and the battery, and reading each;
//!   the CPUs' cores and how fast each can go, sched_ext's state, and the scheduler's files.
//!
//! # Not here
//!
//! - What to write: [`crate::writes`]. Writing it, as root: `system/files/power/kubaos-power-write`.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::settings::ChargeLimit;

/// The machine under `root`, `/` for the real one.
#[derive(Clone, Debug)]
pub struct Machine {
    root: PathBuf,
}

/// A CPU frequency policy: `policy0`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub name: String,
    pub epp: Option<String>,
    pub boost: Option<bool>,
    /// `scaling_max_freq`, in kHz.
    pub max_khz: Option<u32>,
    /// The highest clock the CPU reaches as it stands, boost included when on, in kHz.
    pub top_khz: Option<u32>,
}

/// A USB device: `7-1.1.3`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsbDevice {
    pub name: String,
    pub vendor: String,
    pub product: String,
    pub title: String,
    /// The interface classes, as hex bytes: `03` is HID.
    pub classes: Vec<String>,
    /// `power/control`: `auto` autosuspends, `on` never.
    pub control: String,
}

/// The USB interface classes of input and security devices: HID (keyboards, mice, the
/// YubiKey's FIDO and OTP), smart cards (the YubiKey's PIV), and audio, whose headsets drop out
/// and pop when suspended.
pub const NEVER_SUSPENDED_CLASSES: [&str; 3] = ["03", "0b", "01"];
/// Yubico's USB vendor id, never suspended whatever interfaces it shows.
pub const YUBICO: &str = "1050";

impl UsbDevice {
    /// Whether it is an input or security device, which is never autosuspended.
    pub fn is_input(&self) -> bool {
        self.vendor.eq_ignore_ascii_case(YUBICO)
            || self
                .classes
                .iter()
                .any(|class| NEVER_SUSPENDED_CLASSES.contains(&class.as_str()))
    }

    /// Whether it stays awake in every stop: an input device, a printer, or a network adapter, whose
    /// link drops and renegotiates when it suspends, as a dock's Ethernet does.
    pub fn kept_awake(&self) -> bool {
        self.is_input()
            || self
                .classes
                .iter()
                .any(|class| KEPT_AWAKE_CLASSES.contains(&class.as_str()))
            || (self.vendor.eq_ignore_ascii_case(REALTEK)
                && REALTEK_ETHERNET
                    .iter()
                    .any(|product| self.product.eq_ignore_ascii_case(product)))
    }
}

/// The interface classes of printers (07) and network adapters (02, CDC), kept awake.
pub const KEPT_AWAKE_CLASSES: [&str; 2] = ["07", "02"];
/// Realtek's USB vendor id. Its Ethernet adapters (the r8152 driver's, in most docks) show a
/// vendor class, not CDC's.
pub const REALTEK: &str = "0bda";
/// The products of Realtek's USB Ethernet adapters.
pub const REALTEK_ETHERNET: [&str; 6] = ["8050", "8152", "8153", "8155", "8156", "8157"];

/// How the charge is limited on this machine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChargeKnob {
    /// TUXEDO's `charging_profile`, with its current value.
    Tuxedo { profile: String },
    /// The kernel's `charge_control_end_threshold` of a battery, in percent.
    EndThreshold { battery: String, percent: u8 },
}

impl ChargeKnob {
    /// The limit it is at now, when it is one of KubaOS's three.
    pub fn limit(&self) -> Option<ChargeLimit> {
        match self {
            ChargeKnob::Tuxedo { profile } => ChargeLimit::from_tuxedo_profile(profile),
            ChargeKnob::EndThreshold { percent, .. } => ChargeLimit::from_percent(*percent),
        }
    }
}

/// The battery as it stands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BatteryNow {
    /// The draw in watts, while discharging.
    pub watts: Option<f64>,
    pub percent: Option<f64>,
    pub discharging: bool,
}

/// The TUXEDO platform device's charging profile.
const TUXEDO_CHARGING: &str = "sys/devices/platform/tuxedo_keyboard/charging_profile/charging_profile";

/// An online CPU, a hardware thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cpu {
    pub id: u32,
    /// The highest performance it reaches, in CPPC's units (`amd_pstate_highest_perf`, else ACPI
    /// CPPC's `highest_perf`), which stay put whether boost is on or off: on a Ryzen AI 9 HX 370
    /// about 200 for a Zen 5 core and 125 for a Zen 5c one.
    pub highest_perf: Option<u32>,
    /// The threads of its core, itself among them: `thread_siblings_list`.
    pub siblings: Vec<u32>,
}

/// sched_ext, the kernel's scheduler class run by a BPF program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchedExt {
    /// `disabled`, `enabling`, `enabled` or `disabling`.
    pub state: String,
    /// The BPF scheduler's name while one runs: `lavd_1.1.2_…`.
    pub ops: Option<String>,
}

impl SchedExt {
    /// Whether scx_lavd is the scheduler in charge.
    pub fn runs_lavd(&self) -> bool {
        self.state == "enabled" && self.ops.as_deref().is_some_and(|ops| ops.starts_with("lavd"))
    }
}

/// Where the root helper keeps the scheduler's command line, for `kubaos-scx.service`'s
/// `EnvironmentFile`: there while LAVD is on, so a boot starts it in the last stop's mode.
pub const LAVD_ENV: &str = "var/lib/kubaos/scx-lavd.env";
/// scx_lavd, linked to KubaOS's build by `install_scx` (`system/lib.sh`).
pub const LAVD_BIN: &str = "usr/local/libexec/kubaos-scx-lavd";

impl Machine {
    /// The real machine.
    pub fn real() -> Machine {
        Machine {
            root: PathBuf::from("/"),
        }
    }

    /// The machine whose `/` is `root`.
    pub fn at(root: impl Into<PathBuf>) -> Machine {
        Machine { root: root.into() }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn read_at(&self, relative: &str) -> Option<String> {
        read(&self.path(relative))
    }

    /// The names in `relative` that `keep` accepts, sorted naturally (`policy2` before
    /// `policy10`).
    fn list(&self, relative: &str, keep: impl Fn(&str) -> bool) -> Vec<String> {
        let Ok(entries) = fs::read_dir(self.path(relative)) else {
            return Vec::new();
        };
        let mut names: Vec<String> = entries
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| keep(name))
            .collect();
        names.sort_by_key(|name| natural(name));
        names
    }

    /// The CPU's frequency policies.
    pub fn policies(&self) -> Vec<Policy> {
        let dir = "sys/devices/system/cpu/cpufreq";
        self.list(dir, |name| name.starts_with("policy"))
            .into_iter()
            .map(|name| {
                let at = |file: &str| self.read_at(&format!("{dir}/{name}/{file}"));
                let khz = |file: &str| at(file).and_then(|text| text.parse().ok());
                Policy {
                    epp: at("energy_performance_preference"),
                    boost: at("boost").map(|boost| boost == "1"),
                    max_khz: khz("scaling_max_freq"),
                    top_khz: khz("cpuinfo_max_freq"),
                    name,
                }
            })
            .collect()
    }

    /// The firmware's platform profile: `low-power`.
    pub fn platform_profile(&self) -> Option<String> {
        self.read_at("sys/firmware/acpi/platform_profile")
    }

    /// The DRM cards with amdgpu's DPM level, with the level: `("card1", "auto")`.
    pub fn gpus(&self) -> Vec<(String, String)> {
        self.list("sys/class/drm", is_card)
            .into_iter()
            .filter_map(|card| {
                let level = self.read_at(&format!(
                    "sys/class/drm/{card}/device/power_dpm_force_performance_level"
                ))?;
                Some((card, level))
            })
            .collect()
    }

    /// The connected laptop panels with AMD's panel power saving, with its level:
    /// `("card1-eDP-1", 0)`.
    pub fn panels(&self) -> Vec<(String, u8)> {
        self.list("sys/class/drm", |name| {
            name.split_once('-')
                .is_some_and(|(card, connector)| is_card(card) && connector.starts_with("eDP-"))
        })
        .into_iter()
        .filter(|connector| self.read_at(&format!("sys/class/drm/{connector}/status")).as_deref() == Some("connected"))
        .filter_map(|connector| {
            let level = self.read_at(&format!("sys/class/drm/{connector}/amdgpu/panel_power_savings"))?;
            Some((connector, level.parse().ok()?))
        })
        .collect()
    }

    /// The PCIe ASPM policy in use, from the list the kernel shows with it in brackets.
    pub fn aspm(&self) -> Option<String> {
        let text = self.read_at("sys/module/pcie_aspm/parameters/policy")?;
        text.split_whitespace()
            .find_map(|word| word.strip_prefix('[')?.strip_suffix(']'))
            .map(str::to_owned)
    }

    /// The USB devices, hubs and root hubs left out: they autosuspend by themselves once what
    /// hangs from them does.
    pub fn usb_devices(&self) -> Vec<UsbDevice> {
        let dir = "sys/bus/usb/devices";
        self.list(dir, |name| !name.contains(':'))
            .into_iter()
            .filter_map(|name| {
                let at = |file: &str| self.read_at(&format!("{dir}/{name}/{file}"));
                let vendor = at("idVendor")?;
                let product = at("idProduct")?;
                let control = at("power/control")?;
                let classes = self.interface_classes(&format!("{dir}/{name}"));
                if classes.iter().all(|class| class == "09") {
                    return None;
                }
                Some(UsbDevice {
                    title: at("product").unwrap_or_default(),
                    name,
                    vendor,
                    product,
                    classes,
                    control,
                })
            })
            .collect()
    }

    fn interface_classes(&self, device: &str) -> Vec<String> {
        let mut classes: Vec<String> = self
            .list(device, |name| name.contains(':'))
            .into_iter()
            .filter_map(|interface| self.read_at(&format!("{device}/{interface}/bInterfaceClass")))
            .map(|class| class.to_ascii_lowercase())
            .collect();
        classes.sort();
        classes.dedup();
        classes
    }

    /// The Wi-Fi interfaces: `wlp99s0`.
    pub fn wifi(&self) -> Vec<String> {
        self.list("sys/class/net", |_| true)
            .into_iter()
            .filter(|name| self.path(&format!("sys/class/net/{name}/wireless")).is_dir())
            .collect()
    }

    /// How the charge is limited here, TUXEDO's profile first.
    pub fn charge(&self) -> Option<ChargeKnob> {
        if let Some(profile) = self.read_at(TUXEDO_CHARGING) {
            return Some(ChargeKnob::Tuxedo { profile });
        }
        self.batteries().into_iter().find_map(|battery| {
            let percent = self
                .read_at(&format!(
                    "sys/class/power_supply/{battery}/charge_control_end_threshold"
                ))?
                .parse()
                .ok()?;
            Some(ChargeKnob::EndThreshold { battery, percent })
        })
    }

    /// Whether amdgpu switches between refresh rates of the same mode without a modeset
    /// (`amdgpu.freesync_video=1`), so the panel can drop to 60 Hz without going dark for a
    /// moment.
    pub fn seamless_refresh(&self) -> bool {
        matches!(
            self.read_at("sys/module/amdgpu/parameters/freesync_video").as_deref(),
            Some("1" | "Y")
        )
    }

    /// The online CPUs, by id.
    pub fn cpus(&self) -> Vec<Cpu> {
        let dir = "sys/devices/system/cpu";
        self.list(dir, |name| {
            name.strip_prefix("cpu")
                .is_some_and(|id| !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()))
        })
        .into_iter()
        // cpu0 has no `online` file on most machines: it can't go offline.
        .filter(|name| {
            self.read_at(&format!("{dir}/{name}/online"))
                .is_none_or(|online| online == "1")
        })
        .filter_map(|name| {
            let at = |file: &str| self.read_at(&format!("{dir}/{name}/{file}"));
            let id = name.strip_prefix("cpu")?.parse().ok()?;
            let highest_perf = at("cpufreq/amd_pstate_highest_perf")
                .or_else(|| at("acpi_cppc/highest_perf"))
                .and_then(|perf| perf.parse().ok());
            let siblings = at("topology/thread_siblings_list")
                .map(|list| cpu_list(&list))
                .filter(|list| !list.is_empty())
                .unwrap_or_else(|| vec![id]);
            Some(Cpu {
                id,
                highest_perf,
                siblings,
            })
        })
        .collect()
    }

    /// sched_ext's state; `None` on a kernel without it.
    pub fn sched_ext(&self) -> Option<SchedExt> {
        let state = self.read_at("sys/kernel/sched_ext/state")?;
        Some(SchedExt {
            state,
            ops: self
                .read_at("sys/kernel/sched_ext/root/ops")
                .filter(|ops| !ops.is_empty()),
        })
    }

    /// The mode the root helper last asked of scx_lavd, as its `scheduler=` value
    /// (`powersave:4,16`); `None` while LAVD is off.
    pub fn lavd_asked(&self) -> Option<String> {
        self.read_at(LAVD_ENV)?
            .lines()
            .find_map(|line| line.strip_prefix("KUBAOS_SCHEDULER="))
            .map(str::to_owned)
    }

    /// Whether scx_lavd is installed.
    pub fn has_lavd(&self) -> bool {
        self.path(LAVD_BIN).exists()
    }

    fn batteries(&self) -> Vec<String> {
        self.list("sys/class/power_supply", |_| true)
            .into_iter()
            .filter(|name| self.read_at(&format!("sys/class/power_supply/{name}/type")).as_deref() == Some("Battery"))
            .filter(|name| self.read_at(&format!("sys/class/power_supply/{name}/scope")).as_deref() != Some("Device"))
            .collect()
    }

    /// The laptop battery now: the draw from `power_now`, or `current_now` times `voltage_now`.
    pub fn battery(&self) -> Option<BatteryNow> {
        let battery = self.batteries().into_iter().next()?;
        let at = |file: &str| self.read_at(&format!("sys/class/power_supply/{battery}/{file}"));
        let micro = |file: &str| at(file).and_then(|text| text.parse::<f64>().ok());
        let discharging = at("status").as_deref() == Some("Discharging");
        let watts = micro("power_now")
            .map(|uw| uw / 1e6)
            .or_else(|| Some(micro("current_now")? * micro("voltage_now")? / 1e12))
            .filter(|_| discharging)
            .map(f64::abs);
        Some(BatteryNow {
            watts,
            percent: micro("capacity"),
            discharging,
        })
    }

    /// `/proc/interrupts`'s text.
    pub fn interrupts(&self) -> Option<String> {
        fs::read_to_string(self.path("proc/interrupts")).ok()
    }

    /// The processes' ids, from `/proc`.
    pub fn pids(&self) -> Vec<u32> {
        let Ok(entries) = fs::read_dir(self.path("proc")) else {
            return Vec::new();
        };
        entries
            .flatten()
            .filter_map(|entry| entry.file_name().to_str()?.parse().ok())
            .collect()
    }

    /// A process's name and its context switches, voluntary and not, summed over its threads: how
    /// often it woke up and was put to sleep.
    pub fn switches(&self, pid: u32) -> Option<(String, u64)> {
        let dir = self.path(&format!("proc/{pid}"));
        let name = read(&dir.join("comm"))?;
        let tasks = fs::read_dir(dir.join("task")).ok()?;
        let mut total = 0;
        for task in tasks.flatten() {
            let Some(status) = read(&task.path().join("status")) else {
                continue;
            };
            total += status
                .lines()
                .filter(|line| {
                    line.starts_with("voluntary_ctxt_switches") || line.starts_with("nonvoluntary_ctxt_switches")
                })
                .filter_map(|line| line.split_whitespace().last()?.parse::<u64>().ok())
                .sum::<u64>();
        }
        Some((name, total))
    }
}

/// The file at `path`, trimmed; `None` when it can't be read.
fn read(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|text| text.trim().to_owned())
}

/// The CPUs of a kernel CPU list, `0-3,12` or `4,16`; a part that doesn't read is left out.
fn cpu_list(list: &str) -> Vec<u32> {
    let mut cpus = Vec::new();
    for part in list.split(',').map(str::trim).filter(|part| !part.is_empty()) {
        match part.split_once('-') {
            Some((from, to)) => {
                if let (Ok(from), Ok(to)) = (from.parse::<u32>(), to.parse::<u32>()) {
                    cpus.extend(from..=to);
                }
            }
            None => cpus.extend(part.parse::<u32>().ok()),
        }
    }
    cpus
}

/// `card1`, not `card1-eDP-1` or `renderD128`.
fn is_card(name: &str) -> bool {
    name.strip_prefix("card")
        .is_some_and(|number| !number.is_empty() && number.chars().all(|c| c.is_ascii_digit()))
}

/// A name split into text and numbers, so `policy2` sorts before `policy10`.
fn natural(name: &str) -> Vec<(String, u64)> {
    let mut parts = Vec::new();
    let mut text = String::new();
    let mut digits = String::new();
    for c in name.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
        } else {
            if !digits.is_empty() {
                parts.push((std::mem::take(&mut text), digits.parse().unwrap_or(0)));
                digits.clear();
            }
            text.push(c);
        }
    }
    parts.push((text, digits.parse().unwrap_or(0)));
    parts
}

#[cfg(test)]
mod tests;
