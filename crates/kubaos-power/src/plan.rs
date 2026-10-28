//! What each stop sets, layer by layer: the one place a stop becomes its settings.
//!
//! ```text
//!                  Super Saver   Power Saver   Balanced      Performance   Max Performance
//! profile (ppd)    power-saver   power-saver   balanced      performance   performance
//! EPP, boost       ppd's own: power / off, power / off, balance_* / on, performance / on
//! clock cap        1.6 GHz       none          none          none          none
//! platform         ppd's own: low-power, low-power, balanced, performance, performance
//! LAVD (when on)   powersave+    powersave     autopilot     performance   performance
//! GPU DPM          low           low           auto          auto          high
//! panel            60 Hz         VRR           VRR           native        native
//! panel saving     3             2             0             0             0
//! ASPM             powersupersave powersave   default       performance   performance
//! USB autosuspend  on            on            on            on            off
//! Wi-Fi saving     on            on            on            off           off
//! background       paused        paused        normal        normal        normal
//! ```
//!
//! power-profiles-daemon owns the rows it already sets on its profile (EPP, boost, the minimum
//! clock and the firmware's platform profile), and sets them again when the power source
//! changes; [`Plan`] says what to expect of them, for `status`, and the service writes the rest
//! after ppd, so its values are the last word. The user's settings change the panel and the clock
//! cap per stop, and panel power saving and the CPU scheduler everywhere: LAVD runs only when the
//! settings turn it on, and then in the stop's mode (`powersave+` packs the work on the fewest,
//! most efficient cores).
//!
//! # Owns
//!
//! - [`Plan::for_stop`] and the values of every row.
//!
//! # Not here
//!
//! - Which files and devices a row writes on this machine: [`crate::writes`]. Reading them back:
//!   [`crate::status`].

use crate::settings::CpuScheduler;
use crate::settings::PanelMode;
use crate::settings::Settings;
use crate::stop::Profile;
use crate::stop::Stop;

/// The CPU's energy-performance preference, as `amd-pstate` and `intel_pstate` name it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Epp {
    Power,
    /// `balance_performance` plugged in, `balance_power` on battery: power-profiles-daemon's
    /// Balanced, which the stop keeps.
    Balance,
    Performance,
}

/// The CPU scheduler a stop runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scheduler {
    /// Linux's own, EEVDF: the settings leave LAVD off.
    Kernel,
    /// scx_lavd through sched_ext, in a mode of its own.
    Lavd(LavdMode),
}

/// scx_lavd's modes, as its command line names them; [`crate::scheduler`] turns one into the
/// command line on this machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LavdMode {
    /// `--powersave` with the efficient cores first, each core's threads together, so the work
    /// packs onto the fewest cores.
    PowersaveFewestCores,
    /// `--powersave`: core compaction, in LAVD's own order.
    Powersave,
    /// `--autopilot`: LAVD picks powersave, balanced or performance by the load.
    Autopilot,
    /// `--performance`: every core, no compaction.
    Performance,
}

/// amdgpu's `power_dpm_force_performance_level`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuDpm {
    Low,
    Auto,
    High,
}

impl GpuDpm {
    pub fn name(self) -> &'static str {
        match self {
            GpuDpm::Low => "low",
            GpuDpm::Auto => "auto",
            GpuDpm::High => "high",
        }
    }
}

/// The PCIe ASPM policy, as `/sys/module/pcie_aspm/parameters/policy` names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aspm {
    Default,
    Performance,
    /// L0s and L1.
    PowerSave,
    /// L1 substates too, the deepest: the NVMe and the Wi-Fi card sleep hardest, and the ones known
    /// to drop off the bus do it here.
    PowerSuperSave,
}

impl Aspm {
    pub fn name(self) -> &'static str {
        match self {
            Aspm::Default => "default",
            Aspm::Performance => "performance",
            Aspm::PowerSave => "powersave",
            Aspm::PowerSuperSave => "powersupersave",
        }
    }

    pub const ALL: [Aspm; 4] = [Aspm::Default, Aspm::Performance, Aspm::PowerSave, Aspm::PowerSuperSave];

    /// The policy named `name`.
    pub fn from_name(name: &str) -> Option<Aspm> {
        Aspm::ALL.into_iter().find(|aspm| aspm.name() == name)
    }
}

/// The laptop panel's refresh rate and VRR.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Panel {
    /// The refresh rate in Hz, `None` for the panel's native one.
    pub refresh: Option<u32>,
    pub vrr: bool,
}

impl Panel {
    /// What `mode` asks of the panel.
    pub fn of(mode: PanelMode) -> Panel {
        match mode {
            PanelMode::Fixed60 => Panel {
                refresh: Some(60),
                vrr: false,
            },
            PanelMode::Vrr => Panel {
                refresh: None,
                vrr: true,
            },
            PanelMode::Native => Panel {
                refresh: None,
                vrr: false,
            },
        }
    }
}

/// Everything one stop sets.
#[expect(
    clippy::struct_excessive_bools,
    reason = "Each bool is one row of the stops' table, on or off, as the table reads."
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plan {
    pub stop: Stop,
    pub profile: Profile,
    /// What power-profiles-daemon sets the energy preference to.
    pub epp: Epp,
    /// Whether power-profiles-daemon leaves CPU boost on.
    pub boost: bool,
    /// The CPU's clock cap in MHz; `None` lifts it to the CPU's top.
    pub cpu_max_mhz: Option<u32>,
    pub scheduler: Scheduler,
    pub gpu_dpm: GpuDpm,
    pub panel: Panel,
    /// AMD's panel power saving (the display's adaptive backlight), 0 off to 4 the most.
    pub panel_power_saving: u8,
    pub aspm: Aspm,
    /// USB autosuspend for every device but the skip list and input devices.
    pub usb_autosuspend: bool,
    pub wifi_power_save: bool,
    /// The file indexer paused and the system monitor sampling less often.
    pub quiet_background: bool,
}

impl Plan {
    /// What `stop` sets, under the user's `settings`.
    pub fn for_stop(stop: Stop, settings: &Settings) -> Plan {
        let own = settings.stop(stop);
        let saving = |level| {
            if settings.panel_power_saving { level } else { 0 }
        };
        let lavd = |mode| match settings.scheduler {
            CpuScheduler::Kernel => Scheduler::Kernel,
            CpuScheduler::Lavd => Scheduler::Lavd(mode),
        };
        let base = Plan {
            stop,
            profile: stop.profile(),
            epp: Epp::Balance,
            boost: true,
            cpu_max_mhz: own.cpu_max_mhz,
            scheduler: lavd(LavdMode::Autopilot),
            gpu_dpm: GpuDpm::Auto,
            panel: Panel::of(own.panel),
            panel_power_saving: 0,
            aspm: own.aspm,
            usb_autosuspend: true,
            wifi_power_save: true,
            quiet_background: false,
        };
        match stop {
            Stop::SuperSaver => Plan {
                epp: Epp::Power,
                boost: false,
                scheduler: lavd(LavdMode::PowersaveFewestCores),
                gpu_dpm: GpuDpm::Low,
                panel_power_saving: saving(3),
                quiet_background: true,
                ..base
            },
            Stop::PowerSaver => Plan {
                epp: Epp::Power,
                boost: false,
                scheduler: lavd(LavdMode::Powersave),
                gpu_dpm: GpuDpm::Low,
                panel_power_saving: saving(2),
                quiet_background: true,
                ..base
            },
            Stop::Balanced => base,
            Stop::Performance => Plan {
                epp: Epp::Performance,
                scheduler: lavd(LavdMode::Performance),
                wifi_power_save: false,
                ..base
            },
            Stop::MaxPerformance => Plan {
                epp: Epp::Performance,
                scheduler: lavd(LavdMode::Performance),
                gpu_dpm: GpuDpm::High,
                usb_autosuspend: false,
                wifi_power_save: false,
                ..base
            },
        }
    }
}

#[cfg(test)]
mod tests;
