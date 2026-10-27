//! What a plan comes to on this machine: the writes the root helper makes, each only where the
//! machine has the knob and it isn't already at its value. An empty list asks nothing of polkit.
//!
//! The helper (`system/files/power/kubaos-power-write`) takes them as `knob=value` arguments,
//! checks each against its own list of knobs and values, and writes only what differs, so it
//! stays the one thing that decides what may be written as root.
//!
//! # Owns
//!
//! - [`Write`] and its argument, and [`for_plan`]: which devices a row reaches, the USB skip list
//!   applied.
//!
//! # Not here
//!
//! - The rows' values: [`crate::plan`]. The panel's refresh rate and VRR, which Hyprland sets:
//!   [`crate::panel`].

use crate::machine::ChargeKnob;
use crate::machine::Machine;
use crate::machine::UsbDevice;
use crate::plan::Plan;
use crate::settings::ChargeLimit;
use crate::settings::Settings;

/// One write the helper makes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Write {
    /// Every policy's `scaling_max_freq`, in kHz; `None` for the CPU's top.
    CpuMax(Option<u32>),
    GpuDpm {
        card: String,
        level: &'static str,
    },
    PanelPowerSaving {
        connector: String,
        level: u8,
    },
    Aspm(&'static str),
    Usb {
        device: String,
        autosuspend: bool,
    },
    Wifi {
        interface: String,
        power_save: bool,
    },
    TuxedoCharge(&'static str),
    ChargeEnd {
        battery: String,
        percent: u8,
    },
}

impl Write {
    /// Its argument to the helper: `gpu-dpm=card1:low`.
    pub fn arg(&self) -> String {
        match self {
            Write::CpuMax(Some(khz)) => format!("cpu-max={khz}"),
            Write::CpuMax(None) => "cpu-max=max".to_owned(),
            Write::GpuDpm { card, level } => format!("gpu-dpm={card}:{level}"),
            Write::PanelPowerSaving { connector, level } => {
                format!("panel-power={connector}:{level}")
            }
            Write::Aspm(policy) => format!("aspm={policy}"),
            Write::Usb { device, autosuspend } => {
                format!("usb={device}:{}", if *autosuspend { "auto" } else { "on" })
            }
            Write::Wifi { interface, power_save } => {
                format!("wifi={interface}:{}", if *power_save { "on" } else { "off" })
            }
            Write::TuxedoCharge(profile) => format!("charge-profile={profile}"),
            Write::ChargeEnd { battery, percent } => format!("charge-end={battery}:{percent}"),
        }
    }
}

/// The knobs the kernel refused for the stop in use, which the service leaves alone until the stop
/// changes: firmware that keeps ASPM for itself refuses every policy, and asking again at each
/// recheck would run pkexec for nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Refused {
    args: Vec<String>,
}

impl Refused {
    /// Takes the helper's stdout, whose `failed KNOB=VALUE` lines name the knobs it couldn't set.
    pub fn record(&mut self, helper_stdout: &str) {
        for arg in helper_stdout
            .lines()
            .filter_map(|line| line.trim().strip_prefix("failed "))
        {
            if !self.args.iter().any(|known| known == arg) {
                self.args.push(arg.to_owned());
            }
        }
    }

    /// Forgets them, for a new stop or new settings.
    pub fn clear(&mut self) {
        self.args.clear();
    }

    /// `writes` without the refused ones.
    pub fn filter(&self, writes: Vec<Write>) -> Vec<Write> {
        writes
            .into_iter()
            .filter(|write| !self.args.contains(&write.arg()))
            .collect()
    }

    /// The refused knobs, as the helper's arguments.
    pub fn args(&self) -> &[String] {
        &self.args
    }
}

/// Whether `device` autosuspends under `plan`: never an input device or one on the user's skip
/// list, whatever the stop.
pub fn autosuspends(device: &UsbDevice, plan: &Plan, settings: &Settings) -> bool {
    plan.usb_autosuspend
        && !device.kept_awake()
        && !settings
            .usb_skip
            .iter()
            .any(|id| id.matches(&device.vendor, &device.product))
}

/// The writes `plan` needs on `machine`, the charge limit of `settings` among them.
/// `wifi_power_save` tells whether an interface saves power now, `None` when it can't tell.
pub fn for_plan(
    plan: &Plan,
    settings: &Settings,
    machine: &Machine,
    wifi_power_save: &dyn Fn(&str) -> Option<bool>,
) -> Vec<Write> {
    let mut writes = Vec::new();
    let policies = machine.policies();
    // The kernel keeps a cap above a policy's top at the top, so that counts as in place.
    let capped = |khz: u32| {
        policies.iter().any(|policy| {
            policy
                .max_khz
                .is_some_and(|max| max != policy.top_khz.map_or(khz, |top| khz.min(top)))
        })
    };
    let cpu_max = match plan.cpu_max_mhz {
        Some(mhz) => capped(mhz * 1000).then_some(Write::CpuMax(Some(mhz * 1000))),
        None => policies
            .iter()
            .any(|policy| policy.max_khz.is_some() && policy.max_khz != policy.top_khz)
            .then_some(Write::CpuMax(None)),
    };
    writes.extend(cpu_max);
    for (card, level) in machine.gpus() {
        if level != plan.gpu_dpm.name() {
            writes.push(Write::GpuDpm {
                card,
                level: plan.gpu_dpm.name(),
            });
        }
    }
    for (connector, level) in machine.panels() {
        if level != plan.panel_power_saving {
            writes.push(Write::PanelPowerSaving {
                connector,
                level: plan.panel_power_saving,
            });
        }
    }
    if machine.aspm().is_some_and(|policy| policy != plan.aspm.name()) {
        writes.push(Write::Aspm(plan.aspm.name()));
    }
    for device in machine.usb_devices() {
        let autosuspend = autosuspends(&device, plan, settings);
        let now = device.control == "auto";
        if now != autosuspend {
            writes.push(Write::Usb {
                device: device.name,
                autosuspend,
            });
        }
    }
    for interface in machine.wifi() {
        if wifi_power_save(&interface) != Some(plan.wifi_power_save) {
            writes.push(Write::Wifi {
                interface,
                power_save: plan.wifi_power_save,
            });
        }
    }
    writes.extend(charge(machine, settings.charge_limit));
    writes
}

/// The write that limits the charge to `limit`, when the machine can and isn't there.
pub fn charge(machine: &Machine, limit: ChargeLimit) -> Option<Write> {
    let knob = machine.charge()?;
    if knob.limit() == Some(limit) {
        return None;
    }
    Some(match knob {
        ChargeKnob::Tuxedo { .. } => Write::TuxedoCharge(limit.tuxedo_profile()),
        ChargeKnob::EndThreshold { battery, .. } => Write::ChargeEnd {
            battery,
            percent: limit.percent(),
        },
    })
}

#[cfg(test)]
mod tests;
