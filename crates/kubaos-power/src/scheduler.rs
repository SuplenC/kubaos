//! The CPU scheduler: Linux's own (EEVDF), or scx_lavd through sched_ext, in the stop's mode.
//!
//! ```text
//! power.toml `scheduler` ─▶ Plan::scheduler ─▶ scheduler=MODE[:CPUS] | off | reset
//!   ─▶ pkexec kubaos-power-write ─▶ /var/lib/kubaos/scx-lavd.env, kubaos-scx.service ─▶ scx_lavd
//! ```
//!
//! scx_lavd takes its mode on its command line only (`--autopower`, the one way it changes mode
//! as it runs, follows power-profiles-daemon's three profiles, which have no autopilot and no
//! fewest cores), so a new mode restarts it, and the kernel runs EEVDF for the moment between.
//! The kernel also falls back to EEVDF by itself when the BPF scheduler errors or stalls; the
//! unit restarts it twice, then stays failed until LAVD is turned on again.
//!
//! # Owns
//!
//! - The helper's `scheduler=` value for a mode on this machine ([`asked`], with the order of
//!   [`fewest_cores`]), when to send it ([`writes`]), the unit's state ([`UnitState`]), when its
//!   end is a fallback worth a toast ([`fallback`]), and what `status` says ([`report`]).
//!
//! # Not here
//!
//! - Running it as root: `system/files/power/kubaos-power-write` and
//!   `system/files/systemd/kubaos-scx.service`. Asking systemd and the journal: `main.rs` and
//!   [`crate::live`].

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::machine::Cpu;
use crate::machine::Machine;
use crate::plan::LavdMode;
use crate::plan::Plan;
use crate::plan::Scheduler;
use crate::status::Row;
use crate::writes::Write;

/// The system unit that runs scx_lavd.
pub const UNIT: &str = "kubaos-scx.service";
/// Its object on systemd's bus: the unit's name with `-` and `.` escaped.
pub const UNIT_PATH: &str = "/org/freedesktop/systemd1/unit/kubaos_2dscx_2eservice";
/// How long the unit waits before it starts LAVD again after a crash (`RestartSec`).
pub const RESTART_SECONDS: u32 = 2;
/// When the unit gives up instead (`StartLimitBurst` in `StartLimitIntervalSec`). The crash that
/// reaches it still enters `auto-restart` first, so a restart's toast can't promise the restart.
pub const START_LIMIT: &str = "3 times in 5 minutes";

/// What the kernel's own scheduler is called everywhere it shows.
const KERNEL: &str = "Linux's own (EEVDF)";

impl LavdMode {
    /// The mode for people: `powersave on the fewest cores`.
    pub fn words(self) -> &'static str {
        match self {
            LavdMode::PowersaveFewestCores => "powersave on the fewest cores",
            LavdMode::Powersave => "powersave",
            LavdMode::Autopilot => "autopilot",
            LavdMode::Performance => "performance",
        }
    }

    /// The flag that picks it, without its dashes.
    fn flag(self) -> &'static str {
        match self {
            LavdMode::PowersaveFewestCores | LavdMode::Powersave => "powersave",
            LavdMode::Autopilot => "autopilot",
            LavdMode::Performance => "performance",
        }
    }
}

/// The root helper's `scheduler=` value for `mode` on `machine`: the flag, and for the fewest
/// cores the CPUs in the order LAVD fills them, `powersave:4,16,5,17,…`. Where the cores are all
/// alike the order adds nothing to LAVD's own, and the fewest cores is plain powersave.
pub fn asked(mode: LavdMode, machine: &Machine) -> String {
    let order = match mode {
        LavdMode::PowersaveFewestCores => fewest_cores(&machine.cpus()),
        _ => None,
    };
    match order {
        Some(cpus) => {
            let cpus: Vec<String> = cpus.iter().map(u32::to_string).collect();
            format!("{}:{}", mode.flag(), cpus.join(","))
        }
        None => mode.flag().to_owned(),
    }
}

/// The CPUs in the order that keeps the most cores asleep under core compaction: the slowest
/// cores first, which are the efficient ones (Zen 5c on a Ryzen AI 9 HX 370), each core's threads
/// side by side, so a second thread fills a core before the next one wakes. `None` when every
/// core reaches the same performance or none says.
///
/// LAVD's own powersave order, without an energy model, sorts by last-level cache before speed
/// (scx's `scheds/rust/scx_lavd/src/cpu_order.rs`), and on the HX 370 the four Zen 5 cores
/// share the first cache.
pub fn fewest_cores(cpus: &[Cpu]) -> Option<Vec<u32>> {
    let mut cores: BTreeMap<Vec<u32>, u32> = BTreeMap::new();
    for cpu in cpus {
        let mut siblings = cpu.siblings.clone();
        siblings.sort_unstable();
        let perf = cpu.highest_perf?;
        let core = cores.entry(siblings).or_insert(perf);
        *core = (*core).max(perf);
    }
    let slowest = cores.values().min()?;
    if cores.values().all(|perf| perf == slowest) {
        return None;
    }
    let online: Vec<u32> = cpus.iter().map(|cpu| cpu.id).collect();
    let mut order: Vec<(&u32, &Vec<u32>)> = cores.iter().map(|(siblings, perf)| (perf, siblings)).collect();
    order.sort_by_key(|(perf, siblings)| (**perf, siblings.first().copied()));
    Some(
        order
            .into_iter()
            .flat_map(|(_, siblings)| siblings.iter().copied())
            .filter(|cpu| online.contains(cpu))
            .collect(),
    )
}

/// The command line an `asked` value comes to, as the helper writes it for the unit:
/// `--powersave --cpu-pref-order 4,16`.
pub fn command_line(asked: &str) -> String {
    match asked.split_once(':') {
        Some((flag, cpus)) => format!("--{flag} --cpu-pref-order {cpus}"),
        None => format!("--{asked}"),
    }
}

/// What to ask the helper for `plan`'s scheduler at a stop or a settings change: nothing when the
/// machine already runs it. `retry` lets a LAVD that crashed until systemd gave up start again.
///
/// It is never asked at a recheck: LAVD stopped by SysRq-S, or given up on, stays stopped until
/// the stop or the settings change.
pub fn writes(plan: &Plan, machine: &Machine, retry: bool) -> Vec<Write> {
    let sched_ext = machine.sched_ext();
    let asked_now = machine.lavd_asked();
    let runs_lavd = sched_ext.as_ref().is_some_and(crate::machine::SchedExt::runs_lavd);
    match plan.scheduler {
        Scheduler::Kernel if asked_now.is_some() || runs_lavd => {
            vec![Write::Scheduler("off".to_owned())]
        }
        Scheduler::Kernel => Vec::new(),
        Scheduler::Lavd(_) if sched_ext.is_none() || !machine.has_lavd() => Vec::new(),
        Scheduler::Lavd(mode) => {
            let want = asked(mode, machine);
            let mut writes = Vec::new();
            if retry {
                writes.push(Write::Scheduler("reset".to_owned()));
            }
            if retry || !runs_lavd || asked_now.as_deref() != Some(want.as_str()) {
                writes.push(Write::Scheduler(want));
            }
            writes
        }
    }
}

/// The unit as systemd tells it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UnitState {
    /// `LoadState`: `loaded`, or `not-found` before `kubaos update` installs it.
    pub load: String,
    /// `ActiveState`: `active`, `activating`, `deactivating`, `inactive` or `failed`.
    pub active: String,
    /// `SubState`: `running`, `auto-restart` between a crash and its restart, `dead`, `failed`.
    pub sub: String,
    /// `Result` of its last run: `success`, `exit-code`, `signal`, `core-dump`,
    /// `start-limit-hit`.
    pub result: String,
    /// `NRestarts`: the restarts since it was last started by hand.
    pub restarts: u32,
}

impl UnitState {
    /// Reads `systemctl show`'s `KEY=VALUE` lines.
    pub fn parse(show: &str) -> UnitState {
        let mut state = UnitState::default();
        for (key, value) in show.lines().filter_map(|line| line.split_once('=')) {
            let value = value.trim().to_owned();
            match key.trim() {
                "LoadState" => state.load = value,
                "ActiveState" => state.active = value,
                "SubState" => state.sub = value,
                "Result" => state.result = value,
                "NRestarts" => state.restarts = value.parse().unwrap_or(0),
                _ => {}
            }
        }
        state
    }

    fn restarting(&self) -> bool {
        self.sub == "auto-restart"
    }

    fn failed(&self) -> bool {
        self.active == "failed"
    }

    /// The unit's state in a few words: `active (running), 1 restart`.
    pub fn words(&self) -> String {
        if self.load == "not-found" {
            return "not installed".to_owned();
        }
        let restarts = match self.restarts {
            0 => String::new(),
            1 => ", 1 restart".to_owned(),
            n => format!(", {n} restarts"),
        };
        let result = if self.result.is_empty() || self.result == "success" {
            String::new()
        } else {
            format!(", last {}", self.result)
        };
        format!("{} ({}){restarts}{result}", self.active, self.sub)
    }
}

/// LAVD's end, as the kernel fell back to EEVDF.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fallback {
    /// It crashed, or the kernel ejected it, and the unit starts it again shortly.
    Restarting { result: String },
    /// It crashed again and again, and the unit gave up on it.
    GaveUp,
}

/// Whether the unit's change from `before` to `now` is a fallback to say: the moment it enters a
/// restart after a crash, or gives up. `before` is `None` for the first look, where a unit
/// already failed or restarting is said too.
pub fn fallback(before: Option<&UnitState>, now: &UnitState) -> Option<Fallback> {
    if now.failed() && !before.is_some_and(UnitState::failed) {
        return Some(Fallback::GaveUp);
    }
    if now.restarting() && !before.is_some_and(UnitState::restarting) {
        return Some(Fallback::Restarting {
            result: now.result.clone(),
        });
    }
    None
}

impl Fallback {
    /// The toast's title, body and urgency.
    pub fn toast(&self) -> (&'static str, String, &'static str) {
        match self {
            Fallback::Restarting { result } => (
                "CPU scheduler",
                format!(
                    "LAVD stopped ({}) and the kernel took over with its own scheduler. systemd \
                     starts LAVD again in {RESTART_SECONDS} s, unless it crashed {START_LIMIT}.",
                    result_words(result)
                ),
                "normal",
            ),
            Fallback::GaveUp => (
                "CPU scheduler",
                "LAVD kept crashing, so it stays off and Linux's own scheduler runs. \
                 \"CPU scheduler: LAVD on\" tries it again; `kubaos power scheduler` says why."
                    .to_owned(),
                "critical",
            ),
        }
    }
}

/// systemd's `Result` for people.
fn result_words(result: &str) -> &str {
    match result {
        "exit-code" => "it exited with an error",
        "signal" => "it was killed",
        "core-dump" => "it crashed",
        "watchdog" => "its watchdog fired",
        "" => "no reason given",
        other => other,
    }
}

/// Which scheduler runs, why, and the details behind it: what `kubaos power scheduler` prints
/// and the Command menu's "CPU scheduler: status" lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// The scheduler in charge now.
    pub running: String,
    /// Why it is that one.
    pub why: String,
    /// Whether it is the one the stop and the settings want.
    pub agrees: bool,
    /// The rest, a label and its value a line.
    pub details: Vec<(String, String)>,
}

/// What the stop's plan asks of the scheduler, in words.
pub fn wanted(plan: &Plan) -> String {
    match plan.scheduler {
        Scheduler::Kernel => KERNEL.to_owned(),
        Scheduler::Lavd(mode) => format!("LAVD, {}", mode.words()),
    }
}

/// The report for `plan` in `stop` on `machine`. `unit` is the unit's state, `None` when systemd
/// couldn't be asked; `exit` is LAVD's last exit message from the journal, when it can be read.
pub fn report(plan: &Plan, machine: &Machine, unit: Option<&UnitState>, exit: Option<&str>) -> Report {
    let stop = plan.stop;
    let sched_ext = machine.sched_ext();
    let asked_now = machine.lavd_asked();
    let want = match plan.scheduler {
        Scheduler::Lavd(mode) => Some(asked(mode, machine)),
        Scheduler::Kernel => None,
    };
    let running = match &sched_ext {
        Some(state) if state.runs_lavd() => match &asked_now {
            Some(asked) => format!("LAVD ({})", command_line(asked)),
            None => "LAVD".to_owned(),
        },
        Some(state) if state.state == "enabled" => {
            format!(
                "{} (a sched_ext scheduler KubaOS didn't start)",
                state.ops.as_deref().unwrap_or("unnamed")
            )
        }
        _ => KERNEL.to_owned(),
    };
    let runs_lavd = sched_ext.as_ref().is_some_and(crate::machine::SchedExt::runs_lavd);
    let agrees = match &want {
        Some(want) => runs_lavd && asked_now.as_ref() == Some(want),
        None => sched_ext.as_ref().is_none_or(|state| state.state == "disabled"),
    };
    let why = why(plan, machine, unit, exit, runs_lavd, agrees);
    let mut details = vec![(
        "Setting".to_owned(),
        match plan.scheduler {
            Scheduler::Kernel => "LAVD off (scheduler = \"kernel\" in power.toml)".to_owned(),
            Scheduler::Lavd(_) => "LAVD on (scheduler = \"lavd\" in power.toml)".to_owned(),
        },
    )];
    if let (Scheduler::Lavd(mode), Some(want)) = (plan.scheduler, &want) {
        details.push((
            format!("Mode in {stop}"),
            format!("{} ({})", mode.words(), command_line(want)),
        ));
    }
    if sched_ext.is_none() {
        details.push(("sched_ext".to_owned(), "not in this kernel".to_owned()));
    }
    if let Some(unit) = unit {
        details.push(("Service".to_owned(), format!("{UNIT}: {}", unit.words())));
    }
    if let Some(exit) = exit {
        details.push(("LAVD's last exit".to_owned(), exit.to_owned()));
    }
    Report {
        running,
        why,
        agrees,
        details,
    }
}

fn why(
    plan: &Plan,
    machine: &Machine,
    unit: Option<&UnitState>,
    exit: Option<&str>,
    runs_lavd: bool,
    agrees: bool,
) -> String {
    let Scheduler::Lavd(mode) = plan.scheduler else {
        return if runs_lavd {
            "LAVD is off in power.toml but still runs; the next stop stops it.".to_owned()
        } else {
            "LAVD is off: scheduler = \"kernel\" in power.toml.".to_owned()
        };
    };
    let stop = plan.stop;
    if agrees {
        return format!("LAVD is on, in {stop}'s mode: {}.", mode.words());
    }
    if runs_lavd {
        return format!("LAVD runs, but not yet in {stop}'s mode ({}).", mode.words());
    }
    if machine.sched_ext().is_none() {
        return "This kernel has no sched_ext, so LAVD can't run.".to_owned();
    }
    if !machine.has_lavd() || unit.is_some_and(|unit| unit.load == "not-found") {
        return "LAVD isn't installed yet: `kubaos update` installs it.".to_owned();
    }
    let because = exit.map(|exit| format!(" ({exit})")).unwrap_or_default();
    match unit {
        Some(unit) if unit.failed() => format!(
            "LAVD crashed until systemd stopped restarting it{because}, so the kernel fell back to its own \
             scheduler. \"CPU scheduler: LAVD on\" tries it again."
        ),
        Some(unit) if unit.restarting() => format!(
            "LAVD crashed{because} and the kernel fell back to its own scheduler; systemd starts it again \
             in {RESTART_SECONDS} s, unless it crashed {START_LIMIT}."
        ),
        Some(unit) if unit.active == "activating" || unit.active == "active" => "LAVD is starting.".to_owned(),
        _ => "LAVD isn't running: it was stopped, or unloaded with SysRq-S. The next stop starts it.".to_owned(),
    }
}

/// The last error scx_lavd ended on, from its output: `EXIT: runtime error (…)` when the kernel
/// ejected it, or why it couldn't load. A clean stop prints its `EXIT:` without `Error: `, and
/// isn't one.
pub fn last_exit(log: &str) -> Option<String> {
    log.lines()
        .rev()
        .find_map(|line| line.trim().strip_prefix("Error: "))
        .map(|error| error.trim().to_owned())
        .filter(|error| !error.is_empty())
}

/// The scheduler's row in `status`: what the stop wants, what runs, and why when they differ.
pub fn row(plan: &Plan, report: &Report) -> Row {
    let now = if report.agrees {
        report.running.clone()
    } else {
        format!("{}: {}", report.running, report.why)
    };
    Row {
        name: "CPU scheduler",
        wanted: wanted(plan),
        now: Some(now),
        agrees: Some(report.agrees),
    }
}

/// The report as `kubaos power scheduler` prints it.
pub fn text(report: &Report) -> String {
    let mut out = format!("CPU scheduler: {}\n{}\n", report.running, report.why);
    let width = report
        .details
        .iter()
        .map(|(label, _)| label.chars().count())
        .max()
        .unwrap_or(0);
    if !report.details.is_empty() {
        out.push('\n');
    }
    for (label, value) in &report.details {
        let _ = writeln!(out, "{label:width$}  {value}");
    }
    out
}

#[cfg(test)]
mod tests;
