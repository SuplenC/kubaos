//! `kubaos power measure`: the battery's average draw over a while, and what woke the machine
//! most in it, as a report to compare the stops by.
//!
//! The draw is the battery's own, sampled every second from `/sys/class/power_supply`, the files
//! UPower reads. The wakeups are two counts taken at the start and the end: each interrupt line of
//! `/proc/interrupts` (devices and timers), and each process's context switches from
//! `/proc/<pid>/task/*/status` (how often it woke and was put to sleep), the way powertop ranks
//! them without perf.
//!
//! # Owns
//!
//! - Sampling, the counts and their differences, and the report's text.
//!
//! # Not here
//!
//! - Where the report is saved, and the waiting: `main.rs`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::time::Duration;

use crate::machine::Machine;

/// How many interrupt lines and processes the report names.
pub const TOP: usize = 10;
/// How long a measurement runs by default: long enough to average out a burst, short enough to
/// run once per stop.
pub const DEFAULT_SECONDS: u64 = 120;

/// Each interrupt line's count, summed over the CPUs, by its name: `"24 amdgpu"`.
pub fn interrupts(text: &str) -> BTreeMap<String, u64> {
    let mut counts = BTreeMap::new();
    let mut lines = text.lines();
    let cpus = lines.next().map_or(0, |header| header.split_whitespace().count());
    for line in lines {
        let Some((irq, rest)) = line.split_once(':') else {
            continue;
        };
        let words: Vec<&str> = rest.split_whitespace().collect();
        let numbers = words
            .iter()
            .take(cpus)
            .take_while(|word| word.chars().all(|c| c.is_ascii_digit()))
            .count();
        let total: u64 = words[..numbers]
            .iter()
            .filter_map(|word| word.parse::<u64>().ok())
            .sum();
        let what = words[numbers..].last().copied().unwrap_or("");
        let what = if irq.trim().chars().all(|c| c.is_ascii_digit()) {
            // A numbered line ends with the device's name, after the chip and the trigger.
            format!("{} {what}", irq.trim())
        } else {
            format!("{} ({})", irq.trim(), words[numbers..].join(" "))
        };
        counts.insert(what, total);
    }
    counts
}

/// What grew most from `before` to `after`, as a rate per second over `seconds`, highest first.
pub fn top(
    before: &BTreeMap<String, u64>,
    after: &BTreeMap<String, u64>,
    seconds: f64,
    count: usize,
) -> Vec<(String, f64)> {
    let mut grown: Vec<(String, f64)> = after
        .iter()
        .filter_map(|(name, &end)| {
            let start = before.get(name).copied().unwrap_or(0);
            let delta = end.checked_sub(start)?;
            (delta > 0).then(|| (name.clone(), delta as f64 / seconds))
        })
        .collect();
    grown.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    grown.truncate(count);
    grown
}

/// Every process's context switches, by `name (pid)`.
pub fn switches(machine: &Machine) -> BTreeMap<String, u64> {
    machine
        .pids()
        .into_iter()
        .filter_map(|pid| {
            let (name, total) = machine.switches(pid)?;
            Some((format!("{name} ({pid})"), total))
        })
        .collect()
}

/// A measurement's result.
#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    pub seconds: f64,
    /// The stop it ran in, by name.
    pub stop: String,
    /// The draws sampled, in watts.
    pub watts: Vec<f64>,
    /// The battery at the start and the end, in percent.
    pub percent: (Option<f64>, Option<f64>),
    pub plugged_in: bool,
    pub interrupts: Vec<(String, f64)>,
    pub processes: Vec<(String, f64)>,
}

impl Report {
    pub fn average(&self) -> Option<f64> {
        (!self.watts.is_empty()).then(|| self.watts.iter().sum::<f64>() / self.watts.len() as f64)
    }

    /// The report's text.
    pub fn text(&self) -> String {
        let mut out = format!("Power measurement: {} for {:.0} s\n\n", self.stop, self.seconds);
        match self.average() {
            Some(average) => {
                let low = self.watts.iter().copied().fold(f64::INFINITY, f64::min);
                let high = self.watts.iter().copied().fold(0.0, f64::max);
                let _ = writeln!(
                    out,
                    "Battery draw: {average:.2} W average, {low:.2} to {high:.2} W over {} samples",
                    self.watts.len()
                );
            }
            None if self.plugged_in => {
                out.push_str("Battery draw: none, it's plugged in. Unplug to measure the draw.\n");
            }
            None => out.push_str("Battery draw: the battery doesn't report one.\n"),
        }
        if let (Some(start), Some(end)) = self.percent {
            let _ = writeln!(out, "Battery: {start:.0} % to {end:.0} %");
        }
        out.push_str("\nInterrupts (per second):\n");
        for (name, rate) in &self.interrupts {
            let _ = writeln!(out, "  {rate:>9.1}  {name}");
        }
        out.push_str("\nProcesses waking (context switches per second):\n");
        for (name, rate) in &self.processes {
            let _ = writeln!(out, "  {rate:>9.1}  {name}");
        }
        out
    }
}

/// Measures for `duration` on `machine`, a sample a `tick`; `sleep` waits.
pub fn run(
    machine: &Machine,
    stop: &str,
    duration: Duration,
    tick: Duration,
    sleep: &mut dyn FnMut(Duration),
) -> Report {
    let interrupts_before = machine.interrupts().map(|text| interrupts(&text)).unwrap_or_default();
    let switches_before = switches(machine);
    let start = machine.battery();
    let samples = (duration.as_secs_f64() / tick.as_secs_f64()).ceil().max(1.0) as u64;
    let mut watts = Vec::new();
    let mut plugged_in = start.is_some_and(|battery| !battery.discharging);
    for _ in 0..samples {
        sleep(tick);
        if let Some(battery) = machine.battery() {
            plugged_in |= !battery.discharging;
            watts.extend(battery.watts);
        }
    }
    let seconds = tick.as_secs_f64() * samples as f64;
    let interrupts_after = machine.interrupts().map(|text| interrupts(&text)).unwrap_or_default();
    let switches_after = switches(machine);
    Report {
        seconds,
        stop: stop.to_owned(),
        watts,
        percent: (
            start.and_then(|battery| battery.percent),
            machine.battery().and_then(|battery| battery.percent),
        ),
        plugged_in,
        interrupts: top(&interrupts_before, &interrupts_after, seconds, TOP),
        processes: top(&switches_before, &switches_after, seconds, TOP),
    }
}

#[cfg(test)]
mod tests;
