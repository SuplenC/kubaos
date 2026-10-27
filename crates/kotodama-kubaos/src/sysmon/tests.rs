use std::cell::RefCell;
use std::fs;
use std::path::Path;

use kubaos_sysmon::Sampler;
use kubaos_sysmon::processes::Signal;
use kubaos_sysmon::processes::SignalError;
use kubaos_sysmon::serve;

use super::COPY;
use super::END;
use super::KILL;
use super::KIND;
use super::Monitor;
use super::OPEN;
use super::PROCESS_KIND;
use super::SHOW;
use crate::fake::FakeRunner;

const NOW: u64 = 1_800_000_000_000;

/// A machine's files under a temporary folder: 16 GiB with 4 GiB available, two cores, and a
/// battery at 80 % drawing 12 W.
fn machine() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("a temp dir");
    write(
        dir.path(),
        "proc/meminfo",
        "MemTotal: 16777216 kB\nMemAvailable: 4194304 kB\n",
    );
    write(dir.path(), "proc/loadavg", "0.50 0.40 0.30 1/500 4242\n");
    stat(dir.path(), 1000);
    write(dir.path(), "sys/class/power_supply/BAT0/type", "Battery\n");
    write(dir.path(), "sys/class/power_supply/BAT0/status", "Discharging\n");
    write(dir.path(), "sys/class/power_supply/BAT0/capacity", "80\n");
    write(dir.path(), "sys/class/power_supply/BAT0/energy_now", "48000000\n");
    write(dir.path(), "sys/class/power_supply/BAT0/energy_full", "60000000\n");
    write(dir.path(), "sys/class/power_supply/BAT0/power_now", "12000000\n");
    dir
}

fn write(root: &Path, file: &str, text: &str) {
    let path = root.join(file);
    fs::create_dir_all(path.parent().expect("a parent")).expect("make the folder");
    fs::write(path, text).expect("write the file");
}

/// `/proc/stat` at `total` ticks, a fifth of them busy.
fn stat(root: &Path, total: u64) {
    let busy = total / 5;
    let idle = total - busy;
    write(
        root,
        "proc/stat",
        &format!(
            "cpu  {busy} 0 0 {idle} 0 0 0 0 0 0\ncpu0 1 0 0 1 0 0 0 0 0 0\ncpu1 1 0 0 1 0 0 0 0 0 0\nbtime 1700000000\n"
        ),
    );
}

fn process(root: &Path, pid: i32, name: &str, pages: u64) {
    write(
        root,
        &format!("proc/{pid}/stat"),
        &format!("{pid} ({name}) S 1 {pid} {pid} 0 -1 0 0 0 0 0 10 0 0 0 20 0 1 0 100 0 {pages} 0\n"),
    );
}

fn never(_: i32, _: Signal) -> Result<(), SignalError> {
    panic!("nothing should be signalled");
}

#[test]
fn ram_is_answered_in_the_top_row() {
    let dir = machine();
    let mut monitor = Monitor::new(dir.path(), None);

    let results = monitor.live(" RAM ", NOW);

    assert_eq!(results.len(), 1);
    let result = &results[0];
    assert!(result.answer);
    assert_eq!(result.item.id, "ram");
    assert_eq!(result.item.kind.name(), KIND);
    assert_eq!(result.item.title, "RAM 75 %");
    assert!(
        result
            .item
            .subtitle
            .as_deref()
            .is_some_and(|line| line.starts_with("12 GB of 16 GB used")),
        "{:?}",
        result.item.subtitle
    );
    assert_eq!(monitor.live("memory", NOW)[0].item.id, "ram");
}

#[test]
fn the_battery_says_its_draw_and_time_left() {
    let dir = machine();
    let mut monitor = Monitor::new(dir.path(), None);

    let result = &monitor.live("battery", NOW)[0];

    assert_eq!(result.item.title, "Battery 80 %, discharging");
    assert_eq!(result.item.subtitle.as_deref(), Some("12.0 W · 4 h 0 min to empty"));
}

#[test]
fn a_desktop_has_no_battery() {
    let dir = machine();
    fs::remove_dir_all(dir.path().join("sys/class/power_supply")).expect("unplug the battery");
    let mut monitor = Monitor::new(dir.path(), None);

    assert_eq!(monitor.live("battery", NOW)[0].item.title, "No battery");
}

#[test]
fn the_first_cpu_read_shows_the_load_and_the_next_the_usage() {
    let dir = machine();
    let mut monitor = Monitor::new(dir.path(), None);

    let first = monitor.live("cpu", NOW).remove(0);
    stat(dir.path(), 2000);
    let second = monitor.live("cpu", NOW + 1000).remove(0);

    assert_eq!(first.item.title, "CPU load 0.50");
    assert_eq!(second.item.title, "CPU 20 %");
    assert!(
        second
            .item
            .subtitle
            .as_deref()
            .is_some_and(|line| line.contains("2 cores") && line.contains("load 0.50 0.40 0.30"))
    );
}

#[test]
fn a_fresh_stream_sample_is_the_one_shown() {
    let shell = machine();
    write(
        shell.path(),
        "proc/meminfo",
        "MemTotal: 16777216 kB\nMemAvailable: 8388608 kB\n",
    );
    let streamed = Sampler::new(shell.path()).sample(NOW - 2000, false, false);
    let stream = shell.path().join("kubaos-sysmon.json");
    fs::write(&stream, serve::sample_line(&streamed)).expect("write the stream");
    let dir = machine();
    let mut monitor = Monitor::new(dir.path(), Some(stream));

    let fresh = monitor.live("ram", NOW).remove(0);
    let stale = monitor.live("ram", NOW + 60_000).remove(0);

    assert_eq!(fresh.item.title, "RAM 50 %");
    assert_eq!(stale.item.title, "RAM 75 %");
}

#[test]
fn kill_lists_the_processes_by_name_with_confirmed_actions() {
    let dir = machine();
    process(dir.path(), 300, "chrome", 1000);
    process(dir.path(), 301, "chrome", 3000);
    process(dir.path(), 400, "chromedriver", 100);
    process(dir.path(), 500, "firefox", 9000);
    let mut monitor = Monitor::new(dir.path(), None);

    let kill = monitor.live("kill chrome", NOW);
    let end = monitor.live("end chrome", NOW);

    let names: Vec<&str> = kill.iter().map(|result| result.item.title.as_str()).collect();
    assert_eq!(names, ["chrome", "chromedriver"]);
    let chrome = &kill[0].item;
    assert_eq!(chrome.kind.name(), PROCESS_KIND);
    assert!(
        chrome
            .subtitle
            .as_deref()
            .is_some_and(|line| line.starts_with("2 processes"))
    );
    assert!(!kill[0].answer);
    assert!(kill[0].score.get() > kill[1].score.get());
    assert_eq!(chrome.actions[0].id, KILL);
    assert_eq!(end[0].item.actions[0].id, END);
    assert!(
        chrome
            .actions
            .iter()
            .filter(|action| action.id != SHOW)
            .all(|action| action.confirm)
    );
}

#[test]
fn other_queries_are_left_to_the_search() {
    let dir = machine();
    process(dir.path(), 300, "chrome", 1000);
    let mut monitor = Monitor::new(dir.path(), None);

    assert_eq!(monitor.live("firefox", NOW), [] as [kotodama_protocol::LiveResult; 0]);
    assert_eq!(monitor.live("kill", NOW), [] as [kotodama_protocol::LiveResult; 0]);
    assert_eq!(monitor.live("kill zzz", NOW), [] as [kotodama_protocol::LiveResult; 0]);
    assert_eq!(monitor.live("cpus are fast", NOW), [] as [kotodama_protocol::LiveResult; 0]);
}

#[test]
fn ending_signals_every_process_of_that_exact_name() {
    let dir = machine();
    process(dir.path(), 300, "chrome", 1000);
    process(dir.path(), 301, "chrome", 3000);
    process(dir.path(), 400, "chromedriver", 100);
    let mut monitor = Monitor::new(dir.path(), None);
    let sent = RefCell::new(Vec::new());

    let done = monitor.run(PROCESS_KIND, END, "chrome", &FakeRunner::default(), |pid, how| {
        sent.borrow_mut().push((pid, how));
        // One exited between the list and the confirm.
        if pid == 301 {
            Err(SignalError::Gone(pid))
        } else {
            Ok(())
        }
    });

    assert_eq!(done, Ok(None));
    // The processes come in `/proc`'s listing order, which the filesystem decides.
    let mut sent = sent.into_inner();
    sent.sort_by_key(|(pid, _)| *pid);
    assert_eq!(sent, [(300, Signal::End), (301, Signal::End)]);
}

#[test]
fn a_process_that_wont_go_is_named_with_how_many_did() {
    let dir = machine();
    process(dir.path(), 300, "chrome", 1000);
    process(dir.path(), 301, "chrome", 3000);
    let mut monitor = Monitor::new(dir.path(), None);

    let partly = monitor.run(PROCESS_KIND, KILL, "chrome", &FakeRunner::default(), |pid, _| {
        if pid == 301 {
            Err(SignalError::NotAllowed(pid))
        } else {
            Ok(())
        }
    });
    let gone = monitor.run(PROCESS_KIND, KILL, "slack", &FakeRunner::default(), never);

    assert_eq!(
        partly,
        Err("Killed 1 of 2 chrome processes; process 301 belongs to another user".to_owned())
    );
    assert_eq!(gone, Err("slack is no longer running".to_owned()));
}

#[test]
fn open_and_show_reach_the_popup() {
    let dir = machine();
    process(dir.path(), 300, "chrome", 1000);
    let mut monitor = Monitor::new(dir.path(), None);
    let runner = FakeRunner::default();

    let opened = monitor.run(KIND, OPEN, "ram", &runner, never);
    let shown = monitor.run(PROCESS_KIND, SHOW, "chrome", &runner, never);

    assert_eq!((opened, shown), (Ok(None), Ok(None)));
    let launched: Vec<String> = runner
        .launched
        .borrow()
        .iter()
        .map(|(argv, _)| argv.join(" "))
        .collect();
    assert_eq!(launched, ["qs ipc call sysmon open", "qs ipc call sysmon process 300"]);
}

#[test]
fn copy_copies_the_answer() {
    let dir = machine();
    let mut monitor = Monitor::new(dir.path(), None);

    let copied = monitor.run(KIND, COPY, "ram", &FakeRunner::default(), never);
    let unknown = monitor.run(KIND, COPY, "disk", &FakeRunner::default(), never);

    assert_eq!(copied, Ok(Some("RAM 75 %".to_owned())));
    assert!(unknown.is_err());
}
