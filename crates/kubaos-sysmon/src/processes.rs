//! Processes: a scan of `/proc/<pid>/stat`, CPU use between two scans, the top five, a process's
//! details, finding processes by name, and ending or killing one.
//!
//! A scan reads one file a process. CPU use is a process's ticks as a share of the whole
//! machine's (every core) between two scans, so 100 % is every core busy. Kernel threads are left
//! out: they can't be ended and only crowd the list.
//!
//! # Owns
//!
//! - Parsing a process's `stat`, `status` and `cmdline`, the owner's name from `/etc/passwd`, and
//!   the signals.
//!
//! # Not here
//!
//! - Asking before a process is ended: the popup (`config/quickshell/modules/sysmon/`) and the
//!   Command menu (`crates/kotodama-kubaos/src/sysmon.rs`) both confirm first.

use std::collections::HashMap;
use std::io;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use crate::read;

/// Ticks a second in `/proc`'s times: `USER_HZ`, which Linux fixes at 100 for every architecture
/// KubaOS runs on, whatever the kernel's own tick rate.
pub const TICKS_PER_SECOND: u64 = 100;

/// kthreadd, the parent of every kernel thread.
const KTHREADD: i32 = 2;

/// What a process's `stat` says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcStat {
    pub pid: i32,
    /// The kernel's name for it (`comm`), at most 15 bytes.
    pub name: String,
    pub state: char,
    pub ppid: i32,
    /// User and system ticks since it started.
    pub ticks: u64,
    pub threads: u32,
    /// When it started, in ticks after boot.
    pub start: u64,
    /// Resident memory, in bytes.
    pub memory: u64,
}

/// Parses a `/proc/<pid>/stat` line; `page` is the page size, which its resident size counts in.
pub fn parse_stat(text: &str, page: u64) -> Option<ProcStat> {
    let open = text.find('(')?;
    let close = text.rfind(')')?;
    let pid = text[..open].trim().parse().ok()?;
    let name = text.get(open + 1..close)?.to_owned();
    // The fields after the name, from the third (state) on: field N is at N - 3.
    let fields: Vec<&str> = text.get(close + 1..)?.split_ascii_whitespace().collect();
    let number = |field: usize| fields.get(field - 3).and_then(|value| value.parse::<u64>().ok());
    Some(ProcStat {
        pid,
        name,
        state: fields.first()?.chars().next()?,
        ppid: fields.get(1)?.parse().ok()?,
        ticks: number(14)? + number(15)?,
        threads: number(20)? as u32,
        start: number(22)?,
        memory: number(24)? * page,
    })
}

/// Every process under `root`, kernel threads left out.
pub fn scan(root: &Path, page: u64) -> Vec<ProcStat> {
    let proc = root.join("proc");
    let Ok(entries) = std::fs::read_dir(&proc) else {
        return Vec::new();
    };
    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name();
            let name = name.to_str()?;
            if !name.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            let text = read::text(&entry.path().join("stat"))?;
            parse_stat(&text, page)
        })
        .filter(|stat| stat.pid != KTHREADD && stat.ppid != KTHREADD)
        .collect()
}

/// A process as the popup lists it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Process {
    pub pid: i32,
    pub name: String,
    /// Percent of the whole machine.
    pub cpu: f32,
    /// Resident memory, in bytes.
    pub memory: u64,
}

/// The ticks each process had at the last scan, for CPU use between scans.
#[derive(Clone, Debug, Default)]
pub struct Tracker {
    last: HashMap<i32, u64>,
    /// The machine's ticks (every core) at the last scan.
    last_total: u64,
}

impl Tracker {
    /// Each process in `scan` with its CPU use since the last call; `total` is the machine's
    /// ticks now, `/proc/stat`'s `cpu` line.
    pub fn update(&mut self, scan: Vec<ProcStat>, total: u64) -> Vec<Process> {
        let elapsed = total.saturating_sub(self.last_total);
        let mut ticks = HashMap::with_capacity(scan.len());
        let list = scan
            .into_iter()
            .map(|stat| {
                let before = self.last.get(&stat.pid).copied();
                ticks.insert(stat.pid, stat.ticks);
                let cpu = match before {
                    Some(before) if elapsed > 0 => stat.ticks.saturating_sub(before) as f32 * 100.0 / elapsed as f32,
                    _ => 0.0,
                };
                Process {
                    pid: stat.pid,
                    name: stat.name,
                    cpu,
                    memory: stat.memory,
                }
            })
            .collect();
        self.last = ticks;
        self.last_total = total;
        list
    }
}

/// What the top list sorts by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    Cpu,
    Memory,
}

impl Order {
    /// The first `count` of `list` by this order.
    pub fn top(self, list: &[Process], count: usize) -> Vec<Process> {
        let mut sorted: Vec<&Process> = list.iter().collect();
        match self {
            Order::Cpu => sorted.sort_by(|a, b| b.cpu.total_cmp(&a.cpu).then(b.memory.cmp(&a.memory))),
            Order::Memory => sorted.sort_by_key(|process| std::cmp::Reverse(process.memory)),
        }
        sorted.into_iter().take(count).cloned().collect()
    }
}

/// The processes of one name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub name: String,
    pub pids: Vec<i32>,
    /// Their resident memory together, in bytes. Shared pages count once a process, so it reads
    /// high for a browser.
    pub memory: u64,
}

/// The processes whose name holds `query`, any case, one group a name, most memory first.
pub fn find(list: &[ProcStat], query: &str) -> Vec<Group> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    let mut groups: Vec<Group> = Vec::new();
    for stat in list.iter().filter(|stat| stat.name.to_lowercase().contains(&query)) {
        if let Some(group) = groups.iter_mut().find(|group| group.name == stat.name) {
            group.pids.push(stat.pid);
            group.memory += stat.memory;
        } else {
            groups.push(Group {
                name: stat.name.clone(),
                pids: vec![stat.pid],
                memory: stat.memory,
            });
        }
    }
    for group in &mut groups {
        group.pids.sort_unstable();
    }
    groups.sort_by(|a, b| b.memory.cmp(&a.memory).then_with(|| a.name.cmp(&b.name)));
    groups
}

/// The processes grouped by name, most memory first: who holds the memory.
pub fn by_memory(list: &[Process]) -> Vec<(String, u64)> {
    let mut totals: HashMap<&str, u64> = HashMap::new();
    for process in list {
        *totals.entry(process.name.as_str()).or_default() += process.memory;
    }
    let mut sorted: Vec<(String, u64)> = totals
        .into_iter()
        .map(|(name, bytes)| (name.to_owned(), bytes))
        .collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    sorted
}

/// What the popup shows for one process.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Details {
    pub pid: i32,
    pub name: String,
    /// The command line, its arguments joined by spaces; the name for a process without one.
    pub command: String,
    pub user: String,
    pub state: String,
    pub parent: i32,
    pub threads: u32,
    pub memory: u64,
    /// When it started, in seconds since the epoch.
    pub started: Option<u64>,
}

/// The details of `pid` under `root`; `boot` is when the machine booted (`/proc/stat`'s `btime`).
pub fn details(root: &Path, pid: i32, boot: Option<u64>, page: u64) -> Option<Details> {
    let dir = root.join(format!("proc/{pid}"));
    let stat = parse_stat(&read::text(&dir.join("stat"))?, page)?;
    let command = read::text(&dir.join("cmdline"))
        .map(|line| {
            line.split('\0')
                .filter(|arg| !arg.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|line| !line.is_empty())
        .unwrap_or_else(|| stat.name.clone());
    let uid = read::text(&dir.join("status")).and_then(|status| {
        status
            .lines()
            .find_map(|line| line.strip_prefix("Uid:"))
            .and_then(|ids| ids.split_ascii_whitespace().next())
            .map(str::to_owned)
    });
    let user = uid.map(|uid| user_name(root, &uid).unwrap_or(uid)).unwrap_or_default();
    Some(Details {
        pid,
        command,
        user,
        state: state_name(stat.state).to_owned(),
        parent: stat.ppid,
        threads: stat.threads,
        memory: stat.memory,
        started: boot.map(|boot| boot + stat.start / TICKS_PER_SECOND),
        name: stat.name,
    })
}

/// The login name of `uid` in `/etc/passwd`.
fn user_name(root: &Path, uid: &str) -> Option<String> {
    let passwd = read::text(&root.join("etc/passwd"))?;
    passwd.lines().find_map(|line| {
        let mut fields = line.split(':');
        let name = fields.next()?;
        (fields.nth(1)? == uid).then(|| name.to_owned())
    })
}

fn state_name(state: char) -> &'static str {
    match state {
        'R' => "running",
        'S' => "sleeping",
        'D' => "waiting on I/O",
        'Z' => "zombie",
        'T' | 't' => "stopped",
        'I' => "idle",
        _ => "other",
    }
}

/// How a process is asked to go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    /// SIGTERM: it may save its work and exit.
    End,
    /// SIGKILL: it stops at once.
    Kill,
}

impl Signal {
    /// `term`/`end` or `kill`.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "term" | "end" => Some(Signal::End),
            "kill" => Some(Signal::Kill),
            _ => None,
        }
    }
}

/// Why a process couldn't be signalled.
#[derive(Debug, thiserror::Error)]
pub enum SignalError {
    #[error("process {0} can't be ended from here")]
    Protected(i32),
    #[error("process {0} is gone")]
    Gone(i32),
    #[error("process {0} belongs to another user")]
    NotAllowed(i32),
    #[error("signalling process {pid}")]
    Failed {
        pid: i32,
        #[source]
        source: io::Error,
    },
}

/// Ends or kills `pid`. Init and the monitor itself are refused.
///
/// # Errors
///
/// [`SignalError::Protected`] for those two, [`SignalError::Gone`] when it has exited,
/// [`SignalError::NotAllowed`] when it is another user's, else [`SignalError::Failed`].
pub fn signal(pid: i32, how: Signal) -> Result<(), SignalError> {
    if pid <= 1 || pid == std::process::id() as i32 {
        return Err(SignalError::Protected(pid));
    }
    let target = rustix::process::Pid::from_raw(pid).ok_or(SignalError::Protected(pid))?;
    let sig = match how {
        Signal::End => rustix::process::Signal::TERM,
        Signal::Kill => rustix::process::Signal::KILL,
    };
    rustix::process::kill_process(target, sig).map_err(|errno| match errno {
        rustix::io::Errno::SRCH => SignalError::Gone(pid),
        rustix::io::Errno::PERM => SignalError::NotAllowed(pid),
        errno => SignalError::Failed {
            pid,
            source: io::Error::from(errno),
        },
    })
}

#[cfg(test)]
mod tests;
