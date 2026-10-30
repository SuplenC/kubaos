//! Alerts: a hot CPU, memory almost full, a full disk, a battery draining fast. Each names the
//! culprit and carries an action the shell offers on the notification.
//!
//! An alert fires once its condition has held for its time (30 s for heat and draw, 10 s for
//! memory, at once for a disk), and not again until the condition has cleared and
//! [`REPEAT_AFTER_MS`] has gone by since it last fired.
//!
//! ```text
//! clear ──over──▶ pending ──held long enough──▶ fired ──clear──▶ clear
//!   ▲               │                                (fires again only after REPEAT_AFTER_MS)
//!   └─────clear─────┘
//! ```
//!
//! # Owns
//!
//! - The conditions, their wording and actions, and when each fires.
//!
//! # Not here
//!
//! - The notification and running its action: `config/quickshell/services/Sysmon.qml`.

use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

use crate::format;
use crate::sampler::Sample;
use crate::settings::Thresholds;

/// How long the CPU stays hot before it is an alert, in ms.
pub const HOT_FOR_MS: u64 = 30_000;
/// How long memory stays nearly full before it is an alert, in ms: a build's peak passes in less.
pub const FULL_FOR_MS: u64 = 10_000;
/// How long the battery drains fast before it is an alert, in ms.
pub const DRAIN_FOR_MS: u64 = 30_000;
/// The least time between two of the same alert, in ms.
pub const REPEAT_AFTER_MS: u64 = 10 * 60_000;
/// The least share of the CPU's load the busiest process has to carry to be named: below a third,
/// stopping it leaves most of the load running, so naming it would blame the wrong thing.
const NAMED_SHARE: f32 = 1.0 / 3.0;

/// An alert, as the shell notifies it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    /// `cpu-temp`, `memory`, `disk:<mount>` or `battery`.
    pub id: String,
    pub title: String,
    pub body: String,
    /// What the notification's button does: `process:<pid>` (the panel on that process),
    /// `files:<mount>` (the mount in Files), `power-saver`, or `open` (the panel).
    pub action: String,
    pub action_label: String,
}

/// Where each alert stands.
#[derive(Clone, Debug, Default)]
pub struct Alerts {
    /// Since when each condition has held, by id.
    since: HashMap<String, u64>,
    /// When each alert last fired, by id.
    fired: HashMap<String, u64>,
    /// The alerts whose condition still holds since they fired.
    holding: Vec<String>,
}

impl Alerts {
    /// The alerts that fire on `sample`, at `now` in ms; `saving` while the power stop already is
    /// a saver one, which a drain can't be sent to again.
    pub fn check(&mut self, sample: &Sample, thresholds: &Thresholds, saving: bool, now: u64) -> Vec<Alert> {
        let over = conditions(sample, thresholds, saving);
        self.since.retain(|id, _| over.iter().any(|(over, _, _)| over == id));
        self.holding.retain(|id| over.iter().any(|(over, _, _)| over == id));
        let mut alerts = Vec::new();
        for (id, held_for, alert) in over {
            let since = *self.since.entry(id.clone()).or_insert(now);
            let rested = self
                .fired
                .get(&id)
                .is_none_or(|at| now.saturating_sub(*at) >= REPEAT_AFTER_MS);
            if now.saturating_sub(since) >= held_for && rested && !self.holding.contains(&id) {
                self.fired.insert(id.clone(), now);
                self.holding.push(id);
                alerts.push(alert);
            }
        }
        alerts
    }

    /// Whether a condition holds, so the next sample should scan the processes to name the
    /// culprit.
    pub fn wants_processes(sample: &Sample, thresholds: &Thresholds) -> bool {
        conditions(sample, thresholds, false)
            .iter()
            .any(|(id, _, _)| !id.starts_with("disk:"))
    }
}

/// Each condition that holds in `sample`: its id, how long it must hold, and its alert.
fn conditions(sample: &Sample, thresholds: &Thresholds, saving: bool) -> Vec<(String, u64, Alert)> {
    let mut over = Vec::new();
    if let Some(temp) = sample.cpu.temp.filter(|temp| *temp >= thresholds.cpu_temp) {
        let (body, action, action_label) = busiest(sample);
        over.push((
            "cpu-temp".to_owned(),
            HOT_FOR_MS,
            Alert {
                id: "cpu-temp".to_owned(),
                title: format!("CPU at {temp:.0} °C for {} s", HOT_FOR_MS / 1000),
                body,
                action,
                action_label,
            },
        ));
    }
    if let Some(memory) = sample.memory.as_ref().filter(|memory| memory.percent >= thresholds.ram) {
        let used = format!("{} of {} used", format::bytes(memory.used), format::bytes(memory.total));
        let top = sample.top.as_ref();
        let heaviest = top.and_then(|top| top.heaviest.as_ref());
        let (title, action, action_label) = match (heaviest, top.and_then(|top| top.memory.first())) {
            (Some((name, bytes)), Some(process)) => {
                let pid = top
                    .and_then(|top| top.memory.iter().find(|process| &process.name == name))
                    .map_or(process.pid, |process| process.pid);
                (
                    format!("Memory almost full, {name} uses {}", format::bytes(*bytes)),
                    format!("process:{pid}"),
                    format!("Show {name}"),
                )
            }
            _ => (
                "Memory almost full".to_owned(),
                "open".to_owned(),
                "Open system monitor".to_owned(),
            ),
        };
        over.push((
            "memory".to_owned(),
            FULL_FOR_MS,
            Alert {
                id: "memory".to_owned(),
                title,
                body: used,
                action,
                action_label,
            },
        ));
    }
    for disk in sample.disks.iter().filter(|disk| disk.percent >= thresholds.disk) {
        let mut body = format!("{} free of {}", format::bytes(disk.free), format::bytes(disk.total));
        if disk.point == "/"
            && let Some(bytes) = sample.snapshots.as_ref().and_then(|snapshots| snapshots.bytes)
        {
            body.push_str("; snapshots hold ");
            body.push_str(&format::bytes(bytes));
        }
        let id = format!("disk:{}", disk.point);
        over.push((
            id.clone(),
            0,
            Alert {
                id,
                title: format!("Disk {} is {:.0} % full", disk.label, disk.percent),
                body,
                action: format!("files:{}", disk.point),
                action_label: "Open in Files".to_owned(),
            },
        ));
    }
    if let Some(battery) = sample
        .battery
        .as_ref()
        .filter(|battery| battery.status == "Discharging")
        && let Some(watts) = battery.watts.filter(|watts| *watts >= thresholds.battery_draw)
    {
        let (mut body, _, _) = busiest(sample);
        if let Some(seconds) = battery.seconds_left {
            body.push_str("; ");
            body.push_str(&format::duration(seconds));
            body.push_str(" left");
        }
        over.push((
            "battery".to_owned(),
            DRAIN_FOR_MS,
            Alert {
                id: "battery".to_owned(),
                title: format!("Battery draining fast ({watts:.0} W)"),
                body,
                action: if saving { "open" } else { "power-saver" }.to_owned(),
                action_label: if saving { "Open system monitor" } else { "Power saver" }.to_owned(),
            },
        ));
    }
    over
}

/// What keeps the CPU busy, as an alert's body and action: the busiest process when it carries a
/// real share of the load, or else the load itself.
fn busiest(sample: &Sample) -> (String, String, String) {
    let usage = sample.cpu.usage.unwrap_or(0.0);
    let top = sample.top.as_ref().and_then(|top| top.cpu.first());
    match top.filter(|process| process.cpu >= 1.0 && process.cpu >= usage * NAMED_SHARE) {
        Some(process) => (
            format!("{} uses {:.0} % of the CPU", process.name, process.cpu),
            format!("process:{}", process.pid),
            format!("Show {}", process.name),
        ),
        None if usage >= 1.0 => (
            format!("The CPU is {usage:.0} % busy across many processes"),
            "open".to_owned(),
            "Open system monitor".to_owned(),
        ),
        None => (
            "No single process stands out".to_owned(),
            "open".to_owned(),
            "Open system monitor".to_owned(),
        ),
    }
}

#[cfg(test)]
mod tests;
