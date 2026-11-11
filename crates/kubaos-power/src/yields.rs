//! Background work yields: build tools and Nix's builds run at nice 19, so input, audio, the
//! compositor and the app in front keep the CPU while a build takes every core.
//!
//! ```text
//! background_yields ─┬▶ yields=on|off ─▶ pkexec kubaos-power-write
//!                    │     ─▶ nix-daemon.service.d/kubaos-yields.conf, its threads reniced
//!                    └▶ the service's watch, every 2 s ─▶ the user's build tools at nice 19
//! ```
//!
//! Nice is the lever because scx_lavd ignores cgroup `cpu.weight` and EEVDF weighs both. Under
//! EEVDF it is what keeps a frame on time beside a build in the same cgroup; under LAVD, whose
//! latency criticality decides first, a reniced build measured no different. Nothing goes to
//! `SCHED_IDLE` or the idle I/O class: a build starved while it holds a lock (the Nix store's)
//! would stall whatever waits on that lock, and nice 19 still yields without starving.
//!
//! A process inherits its parent's nice, so reaching `cargo` or `make` within one scan reaches
//! every compiler it starts after, and a program `cargo run` starts runs at nice 19 too. Raising
//! a nice needs no privilege; lowering it again does (`RLIMIT_NICE` is 0), so turning yielding
//! off puts the user's processes back through the root helper.
//!
//! # Owns
//!
//! - The rules ([`RULES`], [`NICE`]), what the watch demotes ([`Watch`], [`demote`]), what yields
//!   now ([`demoted`]), and the report's rows on it ([`details`]).
//!
//! # Not here
//!
//! - nix-daemon's drop-in and putting a process back to nice 0, which take root:
//!   `system/files/power/kubaos-power-write`. The watch's thread: `main.rs`. When the drop-in is
//!   written: [`crate::writes::for_plan`].

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::MetadataExt as _;
use std::path::Path;
use std::time::Duration;

use rustix::process::Pid;

use crate::machine::Machine;

/// The nice background work runs at: the lowest, a 70th of a nice-0 task's weight, which still
/// never starves. Under EEVDF, with two 24-job builds in the same cgroup as a 180 Hz task doing
/// 1 ms a frame, 0.09–0.28 % of its frames ended late at nice 19, 0.43–2.35 % at nice 10 and
/// 3.1–7.5 % at nice 0. It is not `nice`'s default of 10 either, so turning yielding off never
/// undoes a `nice` someone ran. nix-daemon's drop-in sets the same (`kubaos-power-write`).
pub const NICE: i32 = 19;

/// How often the watch looks for new build tools. A tool's children inherit its nice, so one scan
/// reaches a whole build; a scan that finds no new process reads one file.
pub const SCAN: Duration = Duration::from_secs(2);

/// The build tools and compilers that yield, by the kernel's name for the process (`comm`, the
/// program's file name cut to 15 bytes).
pub const RULES: &[&str] = &[
    "cargo", "rustc", "rust-lld", "make", "gmake", "ninja", "cc", "c++", "gcc", "g++", "cc1", "cc1plus", "as", "clang",
    "clang++", "ld", "ld.bfd", "ld.gold", "ld.lld", "ld.mold", "lld", "mold", "collect2", "go",
];

/// Nix's daemon: its drop-in sets its nice, and its builds inherit it.
pub const NIX_DAEMON: &str = "nix-daemon";

/// How many parents [`demoted`] climbs to find a rule, past any a build runs under.
const MAX_DEPTH: usize = 64;

/// A process as `/proc/PID/stat` and the owner of `/proc/PID` tell it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Process {
    pub pid: u32,
    pub ppid: u32,
    /// The kernel's name for it, `comm`.
    pub name: String,
    /// Its main thread's nice.
    pub nice: i32,
    pub uid: u32,
}

impl Process {
    /// Reads `/proc/PID/stat`'s line: `PID (NAME) STATE PPID …`, the nice 19th. The name may hold
    /// spaces and parentheses, so it ends at the last `)`.
    pub fn parse(stat: &str, uid: u32) -> Option<Process> {
        let (pid, rest) = stat.split_once(" (")?;
        let (name, rest) = rest.rsplit_once(") ")?;
        let fields: Vec<&str> = rest.split_whitespace().collect();
        Some(Process {
            pid: pid.trim().parse().ok()?,
            ppid: fields.get(1)?.parse().ok()?,
            name: name.to_owned(),
            nice: fields.get(16)?.parse().ok()?,
            uid,
        })
    }

    /// Whether a rule names it.
    pub fn is_build_tool(&self) -> bool {
        RULES.contains(&self.name.as_str())
    }
}

/// The process `pid` in `proc`; `None` once it is gone.
fn read_process(proc: &Path, pid: u32) -> Option<Process> {
    let dir = proc.join(pid.to_string());
    let uid = fs::metadata(&dir).ok()?.uid();
    Process::parse(&fs::read_to_string(dir.join("stat")).ok()?, uid)
}

/// The numbered entries of `dir`: processes in `/proc`, threads in `/proc/PID/task`.
fn ids(dir: &Path) -> Vec<u32> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|entry| entry.ok()?.file_name().to_str()?.parse().ok())
        .collect()
}

/// Every process in `proc`.
pub fn processes(proc: &Path) -> Vec<Process> {
    ids(proc)
        .into_iter()
        .filter_map(|pid| read_process(proc, pid))
        .collect()
}

/// Finds the user's build tools as they start.
#[derive(Debug, Default)]
pub struct Watch {
    /// The last process the kernel started at the last scan (`/proc/loadavg`'s last field).
    last_pid: Option<String>,
    /// Every process the last scan looked at, so a scan reads only the new ones.
    seen: HashSet<u32>,
}

impl Watch {
    /// The `uid`'s build tools in `proc` that started since the last scan and run below [`NICE`].
    /// It reads nothing more when no process started since.
    pub fn scan(&mut self, proc: &Path, uid: u32) -> Vec<Process> {
        let last_pid = fs::read_to_string(proc.join("loadavg"))
            .ok()
            .and_then(|line| line.split_whitespace().nth(4).map(str::to_owned));
        if last_pid.is_some() && last_pid == self.last_pid {
            return Vec::new();
        }
        self.last_pid = last_pid;
        let mut seen = HashSet::new();
        let mut found = Vec::new();
        for pid in ids(proc) {
            seen.insert(pid);
            if self.seen.contains(&pid) {
                continue;
            }
            if let Some(process) = read_process(proc, pid)
                && process.uid == uid
                && process.nice < NICE
                && process.is_build_tool()
            {
                found.push(process);
            }
        }
        self.seen = seen;
        found
    }

    /// Forgets what it has seen, so the next scan looks at every process again.
    pub fn forget(&mut self) {
        *self = Watch::default();
    }
}

/// Puts every thread of `pid` below [`NICE`] at it. Nice is a thread's own, and a compiler may
/// have started its threads before the scan found it. A process or thread gone meanwhile is
/// skipped.
pub fn demote(proc: &Path, pid: u32) {
    let tasks = proc.join(pid.to_string()).join("task");
    for tid in ids(&tasks) {
        let nice = fs::read_to_string(tasks.join(tid.to_string()).join("stat"))
            .ok()
            .and_then(|stat| Process::parse(&stat, 0))
            .map(|thread| thread.nice);
        if nice.is_some_and(|nice| nice < NICE)
            && let Some(tid) = i32::try_from(tid).ok().and_then(Pid::from_raw)
            && let Err(err) = rustix::process::setpriority_process(Some(tid), NICE)
        {
            log::debug!("couldn't put thread {tid:?} of {pid} at nice {NICE}: {err}");
        }
    }
}

/// The processes yielding now: at [`NICE`], and a build tool or nix-daemon, or started by one.
pub fn demoted(processes: &[Process]) -> Vec<&Process> {
    let by_pid: HashMap<u32, &Process> = processes.iter().map(|process| (process.pid, process)).collect();
    processes
        .iter()
        .filter(|process| process.nice == NICE && under_a_rule(process, &by_pid))
        .collect()
}

fn under_a_rule(process: &Process, by_pid: &HashMap<u32, &Process>) -> bool {
    let mut at = Some(process);
    for _ in 0..MAX_DEPTH {
        let Some(process) = at else {
            return false;
        };
        if process.is_build_tool() || process.name == NIX_DAEMON {
            return true;
        }
        at = by_pid.get(&process.ppid).copied();
    }
    false
}

/// The processes by name, the most first: `rustc ×12, cargo, nix-daemon ×2`.
pub fn summary(processes: &[&Process]) -> String {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for process in processes {
        *counts.entry(process.name.as_str()).or_default() += 1;
    }
    let mut counts: Vec<(&str, usize)> = counts.into_iter().collect();
    counts.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    let names: Vec<String> = counts
        .into_iter()
        .map(|(name, count)| {
            if count == 1 {
                name.to_owned()
            } else {
                format!("{name} ×{count}")
            }
        })
        .collect();
    names.join(", ")
}

/// The report's rows on background work, with `on` the setting and `processes` the machine's now.
pub fn details(on: bool, machine: &Machine, processes: &[Process]) -> Vec<(String, String)> {
    let nix = machine.nix_yields();
    if !on {
        let mut rows = vec![(
            "Background work yields".to_owned(),
            "off (background_yields = false in power.toml)".to_owned(),
        )];
        if nix == Some(true) {
            rows.push((
                "Nix's builds".to_owned(),
                "still yield: the next stop removes nix-daemon's drop-in".to_owned(),
            ));
        }
        return rows;
    }
    let mut rows = vec![
        (
            "Background work yields".to_owned(),
            format!("on, at nice {NICE}: the build tools you start, and Nix's builds"),
        ),
        ("Build tools".to_owned(), RULES.join(", ")),
    ];
    match nix {
        Some(true) => rows.push(("Nix's builds".to_owned(), "yield (nix-daemon's drop-in)".to_owned())),
        Some(false) => rows.push((
            "Nix's builds".to_owned(),
            "don't yield yet: the next stop writes nix-daemon's drop-in".to_owned(),
        )),
        None => {}
    }
    let now = demoted(processes);
    let now = if now.is_empty() {
        "nothing".to_owned()
    } else {
        summary(&now)
    };
    rows.push(("Yielding now".to_owned(), now));
    rows
}

#[cfg(test)]
mod tests;
