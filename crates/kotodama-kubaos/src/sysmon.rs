//! The system monitor in the search: `cpu`, `ram` and `battery` answered in the top row, and
//! `kill chrome` or `end slack` finding the processes to stop, each stop confirmed first.
//!
//! The numbers come from the monitor's stream file while the shell's monitor writes it, so the
//! menu, the bar's pill and the panel show one reading. Without a fresh one (the shell down, or
//! the screen locked) they are read from `/proc` and `/sys` here.
//!
//! # Owns
//!
//! - Which Live queries get a monitor answer or process results, and what their Actions do.
//!
//! # Not here
//!
//! - Reading the machine and signalling processes: `kubaos_sysmon`.
//! - The monitor's Items (Open system monitor, Show/hide the bar pill, Pick bar metrics): registry
//!   actions in `config/kotodama/kubaos.toml`.
//! - The protocol loop: `main.rs`.

use std::path::PathBuf;
use std::time::Duration;

use kotodama_extension::Chain;
use kotodama_protocol::Action;
use kotodama_protocol::Fraction;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::LiveResult;
use kotodama_protocol::NamespacedKind;
use kubaos_sysmon::Sample;
use kubaos_sysmon::Sampler;
use kubaos_sysmon::format;
use kubaos_sysmon::processes;
use kubaos_sysmon::processes::Signal;
use kubaos_sysmon::processes::SignalError;
use kubaos_sysmon::serve;

use crate::run::Runner;

/// The Item kind of a monitor answer.
pub const KIND: &str = "kubaos.sysmon";
/// The Item kind of a process result, one a name.
pub const PROCESS_KIND: &str = "kubaos.process";
/// Copies the answer, as shown.
pub const COPY: &str = "copy";
/// Opens the monitor's panel.
pub const OPEN: &str = "open";
/// Asks every process of the name to exit.
pub const END: &str = "end";
/// Stops every process of the name at once.
pub const KILL: &str = "kill";
/// Opens the panel on the process's details.
pub const SHOW: &str = "show";

/// How old a sample in the stream file may be and still answer: the pill's pace with room to
/// spare. An older one means nothing is writing it.
const FRESH_MS: u64 = 10_000;
/// The most process results one query lists.
const MAX_PROCESSES: usize = 8;

/// What a monitor answer is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Topic {
    Cpu,
    Ram,
    Battery,
}

impl Topic {
    fn parse(query: &str) -> Option<Self> {
        match query {
            "cpu" => Some(Topic::Cpu),
            "ram" | "memory" => Some(Topic::Ram),
            "battery" => Some(Topic::Battery),
            _ => None,
        }
    }

    fn id(self) -> &'static str {
        match self {
            Topic::Cpu => "cpu",
            Topic::Ram => "ram",
            Topic::Battery => "battery",
        }
    }
}

/// The machine's numbers and processes, for the search.
#[derive(Debug)]
pub struct Monitor {
    root: PathBuf,
    stream: Option<PathBuf>,
    /// Reads the machine when the stream is stale; made on first need, and kept so a second
    /// keystroke has a CPU reading to measure from.
    sampler: Option<Sampler>,
}

impl Monitor {
    /// A monitor reading the machine under `root` (`/`), and the shell monitor's samples from
    /// `stream` (`$XDG_RUNTIME_DIR/kubaos-sysmon.json`).
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, stream: Option<PathBuf>) -> Self {
        Monitor {
            root: root.into(),
            stream,
            sampler: None,
        }
    }

    /// The results for the Live query `query` at `now` (ms since the epoch): one answer for
    /// `cpu`, `ram` or `battery`, the processes for `kill NAME` or `end NAME`, else none.
    pub fn live(&mut self, query: &str, now: u64) -> Vec<LiveResult> {
        let query = query.trim().to_lowercase();
        if let Some(topic) = Topic::parse(&query) {
            let sample = self.sample(now);
            return vec![answer(topic, &sample)];
        }
        let Some((verb, name)) = query.split_once(char::is_whitespace) else {
            return Vec::new();
        };
        let first = match verb {
            "kill" => KILL,
            "end" => END,
            _ => return Vec::new(),
        };
        let name = name.trim();
        let scan = self.scan();
        processes::find(&scan, name)
            .into_iter()
            .take(MAX_PROCESSES)
            .map(|group| process_result(&group, name, first))
            .collect()
    }

    /// Runs `action` on the Item `id` of `kind`: what to copy, or `None` once it is done.
    /// `send` signals one process; `processes::signal` outside the tests.
    ///
    /// # Errors
    ///
    /// When the Action is unknown, the panel can't be reached, or a process couldn't be stopped;
    /// the message is what the user reads.
    pub fn run(
        &mut self,
        kind: &str,
        action: &str,
        id: &str,
        runner: &impl Runner,
        send: impl FnMut(i32, Signal) -> Result<(), SignalError>,
    ) -> Result<Option<String>, String> {
        match (kind, action) {
            (KIND, COPY) => {
                let topic = Topic::parse(id).ok_or_else(|| format!("the monitor has no answer `{id}`"))?;
                let sample = self.sample(serve::now_ms());
                Ok(Some(answer(topic, &sample).item.title))
            }
            (KIND, OPEN) => launch(runner, &["open"]).map(|()| None),
            (PROCESS_KIND, SHOW) => {
                let scan = self.scan();
                let pid = scan
                    .iter()
                    .find(|stat| stat.name == id)
                    .ok_or_else(|| format!("{id} is no longer running"))?
                    .pid;
                launch(runner, &["process", &pid.to_string()]).map(|()| None)
            }
            (PROCESS_KIND, END) => self.stop(id, Signal::End, send).map(|()| None),
            (PROCESS_KIND, KILL) => self.stop(id, Signal::Kill, send).map(|()| None),
            _ => Err(format!("a `{kind}` Item has no Action `{action}`")),
        }
    }

    /// Signals every process named exactly `name`, scanned afresh: the list the user confirmed
    /// may be a few seconds old. One that exited in the meantime counts as stopped.
    fn stop(
        &mut self,
        name: &str,
        how: Signal,
        mut send: impl FnMut(i32, Signal) -> Result<(), SignalError>,
    ) -> Result<(), String> {
        let pids: Vec<i32> = self
            .scan()
            .into_iter()
            .filter(|stat| stat.name == name)
            .map(|stat| stat.pid)
            .collect();
        if pids.is_empty() {
            return Err(format!("{name} is no longer running"));
        }
        let mut failed = Vec::new();
        for &pid in &pids {
            match send(pid, how) {
                Ok(()) | Err(SignalError::Gone(_)) => {}
                Err(err) => failed.push(Chain(&err).to_string()),
            }
        }
        let Some(first) = failed.first() else {
            return Ok(());
        };
        let verb = match how {
            Signal::End => "Ended",
            Signal::Kill => "Killed",
        };
        Err(format!(
            "{verb} {} of {} {name} processes; {first}",
            pids.len() - failed.len(),
            pids.len()
        ))
    }

    fn scan(&mut self) -> Vec<processes::ProcStat> {
        let page = self.sampler().page();
        processes::scan(&self.root, page)
    }

    fn sampler(&mut self) -> &mut Sampler {
        let root = &self.root;
        self.sampler.get_or_insert_with(|| Sampler::new(root.clone()))
    }

    /// The shell monitor's last sample if it is fresh, else one read here.
    fn sample(&mut self, now: u64) -> Sample {
        if let Some(sample) = self
            .stream
            .as_deref()
            .and_then(|stream| serve::read_stream(stream, now, FRESH_MS))
        {
            return sample;
        }
        self.sampler().sample(now, false, false)
    }
}

fn launch(runner: &impl Runner, words: &[&str]) -> Result<(), String> {
    let argv: Vec<String> = ["qs", "ipc", "call", "sysmon"]
        .iter()
        .chain(words)
        .map(|arg| (*arg).to_owned())
        .collect();
    runner
        .launch(&argv, Duration::ZERO)
        .map_err(|err| Chain(&err).to_string())
}

fn answer(topic: Topic, sample: &Sample) -> LiveResult {
    let (title, subtitle, icon) = match topic {
        Topic::Cpu => cpu(sample),
        Topic::Ram => ram(sample),
        Topic::Battery => battery(sample),
    };
    LiveResult {
        item: Item {
            id: topic.id().to_owned(),
            kind: ItemKind::Namespaced(kind(KIND)),
            identity: None,
            title,
            subtitle: Some(subtitle),
            icon: Some(icon.into()),
            keyword: None,
            keywords: vec![],
            hidden: false,
            app_only: false,
            weak_keywords: Vec::new(),
            yields: false,
            actions: vec![
                Action::new(OPEN, "Open System Monitor"),
                Action::new(COPY, "Copy Answer").shortcut("ctrl+c"),
            ],
        },
        score: certain(),
        answer: true,
    }
}

fn cpu(sample: &Sample) -> (String, String, &'static str) {
    let cpu = &sample.cpu;
    let mut parts = Vec::new();
    if cpu.count > 0 {
        parts.push(format!("{} cores", cpu.count));
    }
    if let Some(mhz) = cpu.mhz {
        parts.push(format!("{:.1} GHz", f64::from(mhz) / 1000.0));
    }
    if let Some(temp) = cpu.temp {
        parts.push(format!("{temp:.0} °C"));
    }
    if let Some([one, five, fifteen]) = cpu.load {
        parts.push(format!("load {one:.2} {five:.2} {fifteen:.2}"));
    }
    // The first read here has nothing to measure from; the load says how busy it has been.
    let title = match (cpu.usage, cpu.load) {
        (Some(usage), _) => format!("CPU {usage:.0} %"),
        (None, Some([one, ..])) => format!("CPU load {one:.2}"),
        (None, None) => "CPU".to_owned(),
    };
    (title, parts.join(" · "), "cpu")
}

fn ram(sample: &Sample) -> (String, String, &'static str) {
    let Some(memory) = &sample.memory else {
        return ("RAM".to_owned(), "Memory can't be read".to_owned(), "memory");
    };
    let mut subtitle = format!(
        "{} of {} used · {} cache · {} available",
        format::bytes(memory.used),
        format::bytes(memory.total),
        format::bytes(memory.cache),
        format::bytes(memory.available)
    );
    if memory.swap_total > 0 {
        subtitle.push_str(" · swap ");
        subtitle.push_str(&format::bytes(memory.swap_used));
        subtitle.push_str(" of ");
        subtitle.push_str(&format::bytes(memory.swap_total));
    }
    (format!("RAM {:.0} %", memory.percent), subtitle, "memory")
}

fn battery(sample: &Sample) -> (String, String, &'static str) {
    let Some(battery) = &sample.battery else {
        return (
            "No battery".to_owned(),
            "This machine runs on mains power".to_owned(),
            "battery-missing",
        );
    };
    let mut parts = Vec::new();
    if let Some(watts) = battery.watts {
        parts.push(format!("{watts:.1} W"));
    }
    if let Some(seconds) = battery.seconds_left {
        let to = if battery.status == "Charging" { "full" } else { "empty" };
        parts.push(format!("{} to {to}", format::duration(seconds)));
    }
    if let Some(health) = battery.health {
        parts.push(format!("health {health:.0} %"));
    }
    let status = battery.status.to_lowercase();
    (
        format!("Battery {:.0} %, {status}", battery.percent),
        parts.join(" · "),
        "battery",
    )
}

/// A process result: `name`, its count and memory, with Actions led by the query's verb.
fn process_result(group: &processes::Group, query: &str, first: &str) -> LiveResult {
    let count = group.pids.len();
    let subtitle = if count == 1 {
        format!("Process {} · {}", group.pids[0], format::bytes(group.memory))
    } else {
        format!("{count} processes · {}", format::bytes(group.memory))
    };
    let end = Action::new(END, "End (ask to quit)").confirm();
    let kill = Action::new(KILL, "Kill (stop at once)").confirm();
    let mut actions = if first == KILL {
        vec![kill, end]
    } else {
        vec![end, kill]
    };
    actions.push(Action::new(SHOW, "Show in System Monitor").shortcut("ctrl+o"));
    // The exact name first, then the shorter names, which hold more of what was typed.
    let exact = group.name.eq_ignore_ascii_case(query);
    let score = if exact {
        0.95
    } else {
        0.6 + 0.3 * query.len() as f32 / group.name.len().max(1) as f32
    };
    LiveResult {
        item: Item {
            id: group.name.clone(),
            kind: ItemKind::Namespaced(kind(PROCESS_KIND)),
            identity: None,
            title: group.name.clone(),
            subtitle: Some(subtitle),
            icon: Some("utilities-system-monitor".into()),
            keyword: None,
            keywords: vec![],
            hidden: false,
            app_only: false,
            weak_keywords: Vec::new(),
            yields: false,
            actions,
        },
        score: Fraction::new(score).unwrap_or_else(|_| certain()),
        answer: false,
    }
}

fn kind(name: &str) -> NamespacedKind {
    NamespacedKind::new(name).expect("the monitor's kinds are valid")
}

/// An answer's score: the highest there is.
fn certain() -> Fraction {
    Fraction::new(1.0).expect("1 is a fraction")
}

#[cfg(test)]
mod tests;
