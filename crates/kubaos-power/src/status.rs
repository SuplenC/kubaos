//! What a stop does, row by row: what the stop wants and what the machine says now, read live
//! from `/sys`, power-profiles-daemon and Hyprland. A row the machine has no knob for says so and
//! is skipped by the stop. `kubaos power status` prints it, and the Command menu's "Power: what
//! this stop does" lists it.
//!
//! # Owns
//!
//! - [`Row`], [`rows`] and the text `status` prints.
//!
//! # Not here
//!
//! - Asking power-profiles-daemon, Hyprland and `iw`: `main.rs`, which hands the answers in as
//!   [`Live`].

use std::fmt::Write as _;

use crate::machine::Machine;
use crate::panel::Target;
use crate::plan::Epp;
use crate::plan::Plan;
use crate::plan::Scheduler;
use crate::settings::Settings;
use crate::stop::Source;
use crate::writes;

/// One layer of a stop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub name: &'static str,
    /// What the stop sets.
    pub wanted: String,
    /// What the machine says now; `None` where it has no such knob.
    pub now: Option<String>,
    /// Whether the two agree; `None` where there is nothing to compare.
    pub agrees: Option<bool>,
}

/// What the rows read from outside `/sys`.
#[derive(Clone, Debug, Default)]
pub struct Live {
    /// power-profiles-daemon's profile.
    pub profile: Option<String>,
    pub source: Option<Source>,
    /// The laptop panel's refresh rate in Hz and whether VRR is on, from Hyprland.
    pub panel: Option<(f64, bool)>,
    /// Whether each Wi-Fi interface saves power, as `iw` says.
    pub wifi: Vec<(String, Option<bool>)>,
    /// Whether the file indexer is held paused, `None` without one.
    pub indexer_paused: Option<bool>,
}

fn row(name: &'static str, wanted: impl Into<String>, now: Option<String>, agrees: Option<bool>) -> Row {
    Row {
        name,
        wanted: wanted.into(),
        now,
        agrees,
    }
}

fn compare(name: &'static str, wanted: &str, now: Option<String>) -> Row {
    let agrees = now.as_deref().map(|now| now == wanted);
    row(name, wanted, now, agrees)
}

fn on_off(on: bool) -> &'static str {
    if on { "on" } else { "off" }
}

/// Every row of `plan` on `machine`.
pub fn rows(plan: &Plan, settings: &Settings, machine: &Machine, live: &Live) -> Vec<Row> {
    let policies = machine.policies();
    let first = policies.first();
    let epp = match (plan.epp, live.source) {
        (Epp::Power, _) => "power",
        (Epp::Balance, Some(Source::Battery)) => "balance_power",
        (Epp::Balance, _) => "balance_performance",
        (Epp::Performance, _) => "performance",
    };
    let target = Target::of(plan.panel, machine.seamless_refresh());
    let mut rows = vec![
        compare(
            "Power profile (power-profiles-daemon)",
            plan.profile.name(),
            live.profile.clone(),
        ),
        compare(
            "CPU energy preference",
            epp,
            first.and_then(|policy| policy.epp.clone()),
        ),
        compare(
            "CPU boost",
            on_off(plan.boost),
            first
                .and_then(|policy| policy.boost)
                .map(|boost| on_off(boost).to_owned()),
        ),
        cpu_cap(plan, &policies),
        compare("Firmware profile", firmware(plan), machine.platform_profile()),
        row("Fans", "the firmware's, by its profile", None, None),
        row("Scheduler", scheduler(plan.scheduler), None, None),
    ];
    let gpus = machine.gpus();
    rows.push(compare(
        "GPU (amdgpu DPM)",
        plan.gpu_dpm.name(),
        (!gpus.is_empty()).then(|| join(gpus.iter().map(|(_, level)| level.as_str()))),
    ));
    rows.push(panel(target, live.panel));
    let panels = machine.panels();
    rows.push(compare(
        "Panel power saving",
        &plan.panel_power_saving.to_string(),
        (!panels.is_empty()).then(|| join(panels.iter().map(|(_, level)| level.to_string()))),
    ));
    rows.push(compare("PCIe ASPM", plan.aspm.name(), machine.aspm()));
    rows.push(usb(plan, settings, machine));
    rows.push(wifi(plan, live));
    rows.push(row(
        "Background work",
        if plan.quiet_background {
            "indexer paused, monitor every 10 s"
        } else {
            "normal"
        },
        live.indexer_paused
            .map(|paused| if paused { "indexer paused" } else { "indexer running" }.to_owned()),
        live.indexer_paused.map(|paused| paused == plan.quiet_background),
    ));
    rows.push(charge(settings, machine));
    rows
}

fn join<S: AsRef<str>>(values: impl Iterator<Item = S>) -> String {
    let mut all: Vec<String> = values.map(|value| value.as_ref().to_owned()).collect();
    all.dedup();
    all.join(", ")
}

fn cpu_cap(plan: &Plan, policies: &[crate::machine::Policy]) -> Row {
    let maxes: Vec<u32> = policies.iter().filter_map(|policy| policy.max_khz).collect();
    let tops: Vec<u32> = policies.iter().filter_map(|policy| policy.top_khz).collect();
    let now = maxes.iter().max().map(|&max| {
        let top = tops.iter().max().copied().unwrap_or(max);
        if max >= top {
            format!("none ({})", ghz(top))
        } else {
            ghz(max)
        }
    });
    let wanted = plan
        .cpu_max_mhz
        .map_or_else(|| "none".to_owned(), |mhz| ghz(mhz * 1000));
    let agrees = (!maxes.is_empty()).then(|| match plan.cpu_max_mhz {
        Some(mhz) => maxes.iter().all(|&max| max == mhz * 1000),
        None => policies.iter().all(|policy| policy.max_khz == policy.top_khz),
    });
    row("CPU clock cap", wanted, now, agrees)
}

fn ghz(khz: u32) -> String {
    format!("{:.1} GHz", f64::from(khz) / 1e6)
}

fn firmware(plan: &Plan) -> &'static str {
    match plan.profile {
        crate::stop::Profile::PowerSaver => "low-power",
        crate::stop::Profile::Balanced => "balanced",
        crate::stop::Profile::Performance => "performance",
    }
}

fn scheduler(scheduler: Scheduler) -> &'static str {
    match scheduler {
        Scheduler::PowersaveFewestCores => "powersave on the fewest cores, once a scheduler runs",
        Scheduler::Powersave => "powersave, once a scheduler runs",
        Scheduler::Autopilot => "autopilot, once a scheduler runs",
        Scheduler::Performance => "performance, once a scheduler runs",
    }
}

fn panel(target: Target, now: Option<(f64, bool)>) -> Row {
    let mut wanted = target.words();
    if target.held_back {
        wanted.push_str(" (60 Hz waits for seamless refresh switching: reboot after the power migration)");
    }
    let agrees =
        now.map(|(hz, vrr)| vrr == target.vrr && target.refresh.is_none_or(|want| (hz - f64::from(want)).abs() < 1.0));
    let now = now.map(|(hz, vrr)| format!("{hz:.0} Hz{}", if vrr { " with VRR" } else { "" }));
    row("Laptop panel", wanted, now, agrees)
}

fn usb(plan: &Plan, settings: &Settings, machine: &Machine) -> Row {
    let devices = machine.usb_devices();
    if devices.is_empty() {
        return row("USB autosuspend", on_off(plan.usb_autosuspend), None, None);
    }
    let kept: Vec<&str> = devices
        .iter()
        .filter(|device| !writes::autosuspends(device, plan, settings))
        .map(|device| {
            if device.title.is_empty() {
                device.name.as_str()
            } else {
                device.title.as_str()
            }
        })
        .collect();
    let wanted = if plan.usb_autosuspend {
        "on, input devices and the skip list never".to_owned()
    } else {
        "off".to_owned()
    };
    let suspending = devices.iter().filter(|device| device.control == "auto").count();
    let now = if kept.is_empty() {
        format!("{suspending} of {} devices", devices.len())
    } else {
        format!("{suspending} of {} devices; never: {}", devices.len(), kept.join(", "))
    };
    let agrees = devices
        .iter()
        .all(|device| (device.control == "auto") == writes::autosuspends(device, plan, settings));
    row("USB autosuspend", wanted, Some(now), Some(agrees))
}

fn wifi(plan: &Plan, live: &Live) -> Row {
    let wanted = on_off(plan.wifi_power_save);
    if live.wifi.is_empty() {
        return row("Wi-Fi power saving", wanted, None, None);
    }
    let now = join(
        live.wifi
            .iter()
            .map(|(name, on)| format!("{name} {}", on.map_or("unknown", on_off))),
    );
    let agrees = live.wifi.iter().all(|(_, on)| *on == Some(plan.wifi_power_save));
    row("Wi-Fi power saving", wanted, Some(now), Some(agrees))
}

fn charge(settings: &Settings, machine: &Machine) -> Row {
    let wanted = format!("{} %", settings.charge_limit.percent());
    let Some(knob) = machine.charge() else {
        return row("Charge limit", wanted, None, None);
    };
    let now = knob
        .limit()
        .map_or_else(|| "another limit".to_owned(), |limit| format!("{} %", limit.percent()));
    row(
        "Charge limit",
        wanted,
        Some(now),
        Some(knob.limit() == Some(settings.charge_limit)),
    )
}

/// The rows as `status` prints them: a header, then a line a row, a mark where the machine
/// disagrees.
pub fn text(header: &str, rows: &[Row]) -> String {
    let width = rows.iter().map(|row| row.name.chars().count()).max().unwrap_or(0);
    let mut out = format!("{header}\n\n");
    for row in rows {
        let now = match (&row.now, row.agrees) {
            (None, _) => "not on this machine".to_owned(),
            (Some(now), Some(false)) => format!("now {now}  ✗"),
            (Some(now), _) => format!("now {now}"),
        };
        let _ = writeln!(out, "{:width$}  {}  ({now})", row.name, row.wanted);
    }
    out
}

#[cfg(test)]
mod tests;
