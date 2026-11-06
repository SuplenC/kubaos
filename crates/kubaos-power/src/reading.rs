//! What one history sample reads from the machine, and nothing more, as `/sys`, `/proc` and the
//! runtime dir have it: no process spawned, no bus call.
//!
//! Every sample reads the battery, the DRM connectors' state, the backlight, Keep awake's state
//! file and `/proc/stat`. Each [`SCAN_EVERY`]th also reads `/proc/interrupts` and every thread's
//! context switches ([`crate::measure`]), by far the dearest part, and names the top wakers since
//! the last scan.
//!
//! # Owns
//!
//! - [`Reader`] and [`Reading`], [`Edge`] (the battery and the kernel's counters at either end of a
//!   suspend), and what two [`HwSleep`]s come to ([`hw_sleep_between`]).
//!
//! # Not here
//!
//! - The files' parsing: [`crate::machine`]. The counts: [`crate::measure`]. When to read:
//!   [`crate::recorder`].

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use serde::Deserialize;

use crate::history::Wakers;
use crate::history::round;
use crate::machine::BatteryNow;
use crate::machine::CpuTimes;
use crate::machine::HwSleep;
use crate::machine::Machine;
use crate::measure;

/// How many samples go between two scans of the wakers. Scanning every thread is the one dear part
/// of a sample (33 ms over 2,900 threads on the reference laptop, against 120 µs for all the rest),
/// and a count over 10 minutes names the same culprits as ten counts of a minute.
pub const SCAN_EVERY: u32 = 10;
/// How many interrupt lines and processes a scan keeps: a handful of names.
pub const TOP_WAKERS: usize = 5;

/// Keep awake's state file in the runtime dir.
pub const KEEP_AWAKE_FILE: &str = "kubaos-keep-awake.json";

/// Keep awake's state file, as far as the history needs it. `kubaos-idle` owns the file, and the
/// fields it has beyond these are ignored.
#[derive(Default, Deserialize)]
struct KeepAwake {
    #[serde(default)]
    active: bool,
    #[serde(default)]
    until: i64,
}

/// Whether the Keep awake state in `text` holds at `now`, in seconds since the epoch: on, and not
/// past its end. Text that doesn't read is off.
pub fn keep_awake_holds(text: &str, now: i64) -> bool {
    let state: KeepAwake = serde_json::from_str(text).unwrap_or_default();
    state.active && (state.until == 0 || state.until > now)
}

/// A sample's reading.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reading {
    pub battery: Option<BatteryNow>,
    pub panel_on: Option<bool>,
    pub brightness: Option<u8>,
    pub docked: bool,
    pub keep_awake: bool,
    pub cpu: Option<f64>,
    pub wakers: Option<Wakers>,
}

/// The battery and the kernel's counters at one end of a suspend.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Edge {
    /// In seconds since the epoch.
    pub at: i64,
    pub battery: Option<BatteryNow>,
    pub hw: HwSleep,
}

impl Edge {
    /// The machine as it stands at `at`.
    pub fn read(machine: &Machine, at: i64) -> Edge {
        Edge {
            at,
            battery: machine.battery(),
            hw: machine.hw_sleep(),
        }
    }
}

/// The time in hardware sleep between `before` and `after`, across a sleep of `seconds`, in
/// microseconds. `None` when it can't be told, which is never the same as none:
///
/// - The files are missing: the kernel shows them only on a machine with S0 low-power idle.
/// - `max_hw_sleep` is 0: no driver reports hardware sleep here, so `total_hw_sleep` stays 0
///   whatever the machine does. On the reference laptop it reads 2^64 - 1, no limit in practice,
///   which is why it is parsed as a `u64` and only ever compared.
/// - The sleep was longer than `max_hw_sleep`: the kernel says the counters may be wrong then.
/// - The total went down, which it never does within a boot.
///
/// The difference of the total is what counts: `last_hw_sleep` is the most recent suspend's, kept
/// beside it, and stays as it was if a driver reported nothing for this one.
///
/// <https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-power>
pub fn hw_sleep_between(before: HwSleep, after: HwSleep, seconds: i64) -> Option<u64> {
    let max = after.max_us?;
    let slept = u64::try_from(seconds).ok()?.saturating_mul(1_000_000);
    if max == 0 || slept > max {
        return None;
    }
    after.total_us?.checked_sub(before.total_us?)
}

/// A scan's counts, the start of the next one's difference.
#[derive(Debug)]
struct Scan {
    at: Instant,
    interrupts: BTreeMap<String, u64>,
    switches: BTreeMap<String, u64>,
}

impl Scan {
    fn take(machine: &Machine, at: Instant) -> Scan {
        Scan {
            at,
            interrupts: machine
                .interrupts()
                .map(|text| measure::interrupts(&text))
                .unwrap_or_default(),
            switches: measure::switches(machine),
        }
    }

    /// The top wakers from `self` to `after`.
    fn wakers_to(&self, after: &Scan) -> Option<Wakers> {
        let elapsed = after.at.saturating_duration_since(self.at);
        let seconds = elapsed.as_secs_f64();
        if seconds <= 0.0 {
            return None;
        }
        Some(Wakers {
            secs: u32::try_from(elapsed.as_secs()).unwrap_or(u32::MAX),
            irqs: rounded(measure::top(&self.interrupts, &after.interrupts, seconds, TOP_WAKERS)),
            procs: rounded(by_name(measure::top(
                &self.switches,
                &after.switches,
                seconds,
                usize::MAX,
            ))),
        })
    }
}

/// Rates by process name: a program's processes count as one, and a pid means nothing a day later.
fn by_name(rates: Vec<(String, f64)>) -> Vec<(String, f64)> {
    let mut summed: BTreeMap<String, f64> = BTreeMap::new();
    for (name, rate) in rates {
        let name = name.rsplit_once(" (").map_or(name.as_str(), |(name, _)| name);
        *summed.entry(name.to_owned()).or_default() += rate;
    }
    let mut rates: Vec<(String, f64)> = summed.into_iter().collect();
    rates.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rates.truncate(TOP_WAKERS);
    rates
}

fn rounded(rates: Vec<(String, f64)>) -> Vec<(String, f64)> {
    rates.into_iter().map(|(name, rate)| (name, round(rate, 1))).collect()
}

/// Reads samples, keeping what a difference needs from one to the next.
#[derive(Debug)]
pub struct Reader {
    machine: Machine,
    keep_awake: Option<PathBuf>,
    cpu: Option<CpuTimes>,
    scan: Option<Scan>,
    /// Samples read since the last scan.
    since_scan: u32,
}

impl Reader {
    /// A reader of `machine`, with Keep awake's state file at `keep_awake`.
    pub fn new(machine: Machine, keep_awake: Option<PathBuf>) -> Reader {
        Reader {
            machine,
            keep_awake,
            cpu: None,
            scan: None,
            since_scan: 0,
        }
    }

    pub fn machine(&self) -> &Machine {
        &self.machine
    }

    /// Whether Keep awake holds at `now`, in seconds since the epoch.
    pub fn keep_awake(&self, now: i64) -> bool {
        self.keep_awake
            .as_deref()
            .and_then(|path| fs::read_to_string(path).ok())
            .is_some_and(|text| keep_awake_holds(&text, now))
    }

    /// Lets the differences go: a charge or a sleep between two samples isn't the battery's time,
    /// and the monotonic clock doesn't count a suspend while the counters go on.
    pub fn forget(&mut self) {
        self.cpu = None;
        self.scan = None;
        self.since_scan = 0;
    }

    /// One sample, at `at` by the wall clock and `now` by the monotonic one.
    pub fn read(&mut self, at: i64, now: Instant) -> Reading {
        let displays = self.machine.displays();
        let cpu_now = self.machine.cpu_times();
        let busy = self
            .cpu
            .zip(cpu_now)
            .and_then(|(before, after)| after.busy_since(before));
        self.cpu = cpu_now;
        Reading {
            battery: self.machine.battery(),
            panel_on: displays.panel_on,
            brightness: self.machine.brightness(),
            docked: displays.external,
            keep_awake: self.keep_awake(at),
            cpu: busy.map(|busy| round(busy, 1)),
            wakers: self.scan(now),
        }
    }

    /// The wakers since the last scan, every [`SCAN_EVERY`] samples; the first scan only starts the
    /// count.
    fn scan(&mut self, now: Instant) -> Option<Wakers> {
        if self.scan.is_some() && self.since_scan + 1 < SCAN_EVERY {
            self.since_scan += 1;
            return None;
        }
        self.since_scan = 0;
        let after = Scan::take(&self.machine, now);
        let wakers = self.scan.as_ref().and_then(|before| before.wakers_to(&after));
        self.scan = Some(after);
        wakers
    }
}

/// The wakers measured live over `window`, for a question asked with no recent scan; `sleep` waits.
pub fn wakers_now(machine: &Machine, window: Duration, sleep: &mut dyn FnMut(Duration)) -> Option<Wakers> {
    let start = Instant::now();
    let before = Scan::take(machine, start);
    sleep(window);
    before.wakers_to(&Scan::take(machine, start + window))
}

#[cfg(test)]
mod tests;
