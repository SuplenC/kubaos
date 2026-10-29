//! A sample: everything the pill, the panel and the menu show, read from `/proc` and `/sys` in one
//! go, with the speeds worked out against the sample before.
//!
//! A light sample (the pill's) reads a handful of files: `/proc/stat`, `/proc/meminfo`, the CPU
//! temperature, the GPUs, the routing table and `/proc/net/dev`, the battery, and the space on the
//! watched mounts. A detailed one (the panel's) adds the cores' clock, disk traffic, the address,
//! fans and snapshots, and a process scan when asked for one. Devices (sensors, GPUs, disks,
//! mounts) are looked for again every [`REDISCOVER_MS`], so a drive plugged in shows up.
//!
//! # Owns
//!
//! - The sample's shape, as the shell and the menu read it, and the counters kept between samples.
//!
//! # Not here
//!
//! - Parsing each file: the modules named in each field. Alerts: [`crate::alerts`]. When to sample:
//!   [`crate::serve`].

use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::cpu;
use crate::cpu::Stat;
use crate::disks;
use crate::disks::Disk;
use crate::disks::DiskSet;
use crate::disks::Mount;
use crate::disks::Snapshots;
use crate::disks::Watched;
use crate::gpu::Gpu;
use crate::gpu::Gpus;
use crate::hwmon::Fan;
use crate::hwmon::Sensors;
use crate::memory;
use crate::memory::Memory;
use crate::net;
use crate::power;
use crate::power::Battery;
use crate::processes;
use crate::processes::Order;
use crate::processes::Process;
use crate::processes::Tracker;
use crate::read;

/// How often devices and mounts are looked for again, in ms: a drive plugged in shows within it.
pub const REDISCOVER_MS: u64 = 15_000;

/// How many processes each top list holds in a light sample: the alerts read only the first.
pub const TOP: usize = 5;

/// How many processes each top list holds in a detailed sample. The panel shows as many as fit
/// in its room under the readings, and 40 rows fill it on a screen 2160 logical pixels tall.
pub const TOP_DETAILED: usize = 40;

/// The processor now.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cpu {
    /// Busy, in percent of every core; `None` on the first sample.
    pub usage: Option<f32>,
    /// Each core's, in a detailed sample.
    pub cores: Vec<f32>,
    pub count: usize,
    pub mhz: Option<u32>,
    pub temp: Option<f32>,
    pub load: Option<[f32; 3]>,
}

/// The active network interface now.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Net {
    /// The default route's interface; `None` when offline.
    pub interface: Option<String>,
    /// Its address, in a detailed sample.
    pub address: Option<String>,
    /// Bytes a second.
    pub down: Option<f64>,
    pub up: Option<f64>,
}

/// Disk traffic, bytes a second.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Io {
    pub read: f64,
    pub write: f64,
}

/// The processes using the most.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Top {
    pub cpu: Vec<Process>,
    pub memory: Vec<Process>,
    /// The name holding the most memory, its processes together, and the bytes.
    pub heaviest: Option<(String, u64)>,
}

/// Everything shown, at one moment.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sample {
    /// When, in ms since the epoch.
    pub t: u64,
    /// A detailed sample, the panel's.
    pub detail: bool,
    pub cpu: Cpu,
    pub memory: Option<Memory>,
    pub gpus: Vec<Gpu>,
    pub disks: Vec<Disk>,
    pub io: Option<Io>,
    pub snapshots: Option<Snapshots>,
    pub net: Net,
    pub battery: Option<Battery>,
    pub fans: Vec<Fan>,
    pub top: Option<Top>,
    /// Seconds since boot.
    pub uptime: u64,
    pub kernel: String,
}

/// The counters of the sample before, which speeds are worked out against.
#[derive(Clone, Debug, Default)]
struct Last {
    at: u64,
    stat: Stat,
    /// The interface, and its bytes received and sent.
    net: Option<(String, u64, u64)>,
    io: Option<(u64, u64)>,
}

/// What reads the samples, and what it keeps between them.
#[derive(Debug)]
pub struct Sampler {
    root: PathBuf,
    page: u64,
    kernel: String,
    sensors: Sensors,
    gpus: Gpus,
    disk_set: DiskSet,
    mounts: Vec<Mount>,
    watched: Vec<Watched>,
    /// When devices and mounts were last looked for.
    discovered: Option<u64>,
    last: Option<Last>,
    tracker: Tracker,
}

impl Sampler {
    /// A sampler reading the files under `root`: `/` on the machine, a fixture folder in tests.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Sampler {
            kernel: read::line(&root.join("proc/sys/kernel/osrelease")).unwrap_or_default(),
            page: rustix::param::page_size() as u64,
            root,
            sensors: Sensors::default(),
            gpus: Gpus::default(),
            disk_set: DiskSet::default(),
            mounts: Vec::new(),
            watched: Vec::new(),
            discovered: None,
            last: None,
            tracker: Tracker::default(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The page size, which a process's resident memory counts in.
    pub fn page(&self) -> u64 {
        self.page
    }

    /// When the machine booted, in seconds since the epoch.
    pub fn boot_time(&self) -> Option<u64> {
        self.last.as_ref().and_then(|last| last.stat.boot_time)
    }

    /// Forgets the counters, so the next sample counts from now: after a pause, a speed averaged
    /// over the whole pause would mislead. Reads the counters now, and scans the processes if
    /// `processes`, so the next sample has speeds.
    pub fn restart(&mut self, now: u64, processes: bool) {
        self.last = None;
        self.tracker = Tracker::default();
        self.baseline(now, processes);
    }

    /// Reads the counters (and the processes if asked) without making a sample.
    pub fn baseline(&mut self, now: u64, processes: bool) {
        let stat = self.stat();
        if processes {
            self.tracker
                .update(processes::scan(&self.root, self.page), stat.total.total);
        }
        let interface = self
            .routes()
            .and_then(|routes| net::default_interface(&routes).map(str::to_owned));
        self.last = Some(Last {
            at: now,
            net: interface.and_then(|name| self.net_counter(&name)),
            io: Some(self.io_counter()),
            stat,
        });
    }

    /// A sample at `now` (ms since the epoch): detailed for the panel, with a process scan if
    /// `processes`.
    pub fn sample(&mut self, now: u64, detail: bool, processes: bool) -> Sample {
        if self.discovered.is_none_or(|at| now.saturating_sub(at) >= REDISCOVER_MS) {
            self.discover(now);
        } else if detail {
            self.read_mounts();
        }
        let stat = self.stat();
        let last = self.last.take();
        let seconds = last
            .as_ref()
            .map(|last| now.saturating_sub(last.at) as f64 / 1000.0)
            .filter(|seconds| *seconds > 0.0);

        let cpu = self.cpu(&stat, last.as_ref(), detail);
        let routes = self.routes();
        let interface = routes.as_deref().and_then(net::default_interface).map(str::to_owned);
        let counter = interface.as_deref().and_then(|name| self.net_counter(name));
        let speed = |now: u64, before: u64| seconds.map(|seconds| now.saturating_sub(before) as f64 / seconds);
        let (down, up) = match (&counter, last.as_ref().and_then(|last| last.net.as_ref())) {
            (Some((name, rx, tx)), Some((before, rx0, tx0))) if name == before => (speed(*rx, *rx0), speed(*tx, *tx0)),
            _ => (None, None),
        };
        let address = if detail {
            interface
                .as_deref()
                .and_then(|name| self.address(routes.as_deref().unwrap_or(&[]), name))
        } else {
            None
        };
        let io_counter = self.io_counter();
        let io = match (detail, last.as_ref().and_then(|last| last.io)) {
            (true, Some((read0, written0))) => Some(Io {
                read: speed(io_counter.0, read0).unwrap_or(0.0),
                write: speed(io_counter.1, written0).unwrap_or(0.0),
            }),
            _ => None,
        };
        let top = processes.then(|| {
            let scanned = self
                .tracker
                .update(processes::scan(&self.root, self.page), stat.total.total);
            let count = if detail { TOP_DETAILED } else { TOP };
            Top {
                cpu: Order::Cpu.top(&scanned, count),
                memory: Order::Memory.top(&scanned, count),
                heaviest: processes::by_memory(&scanned).into_iter().next(),
            }
        });

        let sample = Sample {
            t: now,
            detail,
            cpu,
            memory: read::text(&self.root.join("proc/meminfo")).and_then(|text| memory::parse_meminfo(&text)),
            gpus: self.gpus.read(),
            disks: self.watched.iter().filter_map(disks::space).collect(),
            io,
            snapshots: if detail {
                disks::snapshots(&self.root, &self.mounts)
            } else {
                None
            },
            net: Net {
                interface: interface.clone(),
                address,
                down,
                up,
            },
            battery: power::read_battery(&self.root),
            fans: if detail { self.sensors.fans() } else { Vec::new() },
            top,
            uptime: read::text(&self.root.join("proc/uptime"))
                .and_then(|text| text.split('.').next()?.trim().parse().ok())
                .unwrap_or(0),
            kernel: self.kernel.clone(),
        };
        self.last = Some(Last {
            at: now,
            stat,
            net: counter,
            io: Some(io_counter),
        });
        sample
    }

    fn discover(&mut self, now: u64) {
        self.sensors = Sensors::discover(&self.root);
        self.gpus = Gpus::discover(&self.root);
        self.disk_set = DiskSet::discover(&self.root);
        self.read_mounts();
        self.discovered = Some(now);
    }

    fn read_mounts(&mut self) {
        self.mounts = read::text(&self.root.join("proc/self/mounts"))
            .map(|text| disks::parse_mounts(&text))
            .unwrap_or_default();
        self.watched = disks::watched(&self.mounts);
    }

    fn stat(&self) -> Stat {
        read::text(&self.root.join("proc/stat"))
            .map(|text| cpu::parse_stat(&text))
            .unwrap_or_default()
    }

    fn cpu(&self, stat: &Stat, last: Option<&Last>, detail: bool) -> Cpu {
        let before = last.map(|last| &last.stat);
        let cores = match before {
            Some(before) if detail => stat
                .cores
                .iter()
                .zip(&before.cores)
                .map(|(now, before)| now.usage_since(before).unwrap_or(0.0))
                .collect(),
            _ => Vec::new(),
        };
        Cpu {
            usage: before.and_then(|before| stat.total.usage_since(&before.total)),
            cores,
            count: stat.cores.len(),
            mhz: if detail {
                cpu::frequency_mhz(&self.root, stat.cores.len())
            } else {
                None
            },
            temp: self.sensors.cpu_temperature(),
            load: read::text(&self.root.join("proc/loadavg")).and_then(|text| cpu::parse_loadavg(&text)),
        }
    }

    fn routes(&self) -> Option<Vec<net::Route>> {
        read::text(&self.root.join("proc/net/route")).map(|text| net::parse_routes(&text))
    }

    fn net_counter(&self, interface: &str) -> Option<(String, u64, u64)> {
        let text = read::text(&self.root.join("proc/net/dev"))?;
        net::parse_net_dev(&text)
            .into_iter()
            .find(|counter| counter.name == interface)
            .map(|counter| (counter.name, counter.received, counter.sent))
    }

    fn address(&self, routes: &[net::Route], interface: &str) -> Option<String> {
        let ipv4 = read::text(&self.root.join("proc/net/fib_trie"))
            .and_then(|text| net::address(routes, &net::parse_fib_locals(&text), interface));
        match ipv4 {
            Some(address) => Some(address.to_string()),
            None => {
                read::text(&self.root.join("proc/net/if_inet6")).and_then(|text| net::parse_if_inet6(&text, interface))
            }
        }
    }

    fn io_counter(&self) -> (u64, u64) {
        read::text(&self.root.join("proc/diskstats"))
            .map(|text| disks::parse_diskstats(&text, &self.disk_set))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests;
