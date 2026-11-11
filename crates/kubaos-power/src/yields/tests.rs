use std::fs;

use super::*;
use crate::fake::FakeRoot;
use crate::machine::NIX_DROP_IN;
use crate::machine::NIX_UNIT;

/// A fake `/proc`: `loadavg`, and a `stat` a process.
struct FakeProc {
    dir: tempfile::TempDir,
}

impl FakeProc {
    fn new() -> FakeProc {
        let proc = FakeProc {
            dir: tempfile::tempdir().expect("a temp dir"),
        };
        proc.started(100);
        proc
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    /// The test's own uid, which owns every fake process.
    fn uid(&self) -> u32 {
        fs::metadata(self.path()).expect("the fake /proc").uid()
    }

    /// The kernel's last PID, as `loadavg` says it.
    fn started(&self, last_pid: u32) {
        fs::write(
            self.path().join("loadavg"),
            format!("1.00 0.50 0.25 2/600 {last_pid}\n"),
        )
        .expect("loadavg");
    }

    fn process(&self, pid: u32, name: &str, parent: u32, nice: i32) {
        let dir = self.path().join(pid.to_string());
        fs::create_dir_all(&dir).expect("the process's dir");
        fs::write(dir.join("stat"), stat(pid, name, parent, nice)).expect("stat");
    }
}

/// `/proc/PID/stat`'s line, with the nice as its 19th field.
fn stat(pid: u32, name: &str, parent: u32, nice: i32) -> String {
    format!(
        "{pid} ({name}) S {parent} {pid} {pid} 0 -1 4194304 120 0 0 0 5 2 0 0 {} {nice} 1 0 4242 1000 100",
        20 + nice
    )
}

fn process(pid: u32, name: &str, parent: u32, nice: i32) -> Process {
    Process::parse(&stat(pid, name, parent, nice), 1000).expect("parses")
}

#[test]
fn a_stat_line_reads_whatever_the_name_holds() {
    let parsed = Process::parse(&stat(4242, "Web Content (x)", 7, 10), 1000).expect("parses");

    assert_eq!(parsed.pid, 4242);
    assert_eq!(parsed.ppid, 7);
    assert_eq!(parsed.name, "Web Content (x)");
    assert_eq!(parsed.nice, 10);
    assert_eq!(Process::parse("4242 (cut", 1000), None);
}

#[test]
fn the_watch_finds_the_users_build_tools_below_nice_10_once() {
    let proc = FakeProc::new();
    proc.process(10, "zsh", 1, 0);
    proc.process(11, "cargo", 10, 0);
    proc.process(12, "rustc", 11, 0);
    proc.process(13, "make", 10, NICE);
    let mut watch = Watch::default();

    let first: Vec<u32> = watch
        .scan(proc.path(), proc.uid())
        .iter()
        .map(|found| found.pid)
        .collect();
    let again = watch.scan(proc.path(), proc.uid());

    assert_eq!(sorted(first), [11, 12]);
    assert_eq!(again, Vec::<Process>::new());
}

#[test]
fn the_watch_reads_only_what_started_since_its_last_scan() {
    let proc = FakeProc::new();
    proc.process(11, "cargo", 1, 0);
    let mut watch = Watch::default();
    watch.scan(proc.path(), proc.uid());

    proc.process(14, "cc1plus", 11, 0);
    let unchanged = watch.scan(proc.path(), proc.uid());
    proc.started(14);
    let found: Vec<u32> = watch
        .scan(proc.path(), proc.uid())
        .iter()
        .map(|found| found.pid)
        .collect();

    assert_eq!(unchanged, Vec::<Process>::new());
    assert_eq!(found, [14]);
}

#[test]
fn the_watch_leaves_other_users_processes_alone() {
    let proc = FakeProc::new();
    proc.process(11, "cargo", 1, 0);

    let found = Watch::default().scan(proc.path(), proc.uid() + 1);

    assert_eq!(found, Vec::<Process>::new());
}

#[test]
fn a_forgotten_watch_looks_at_every_process_again() {
    let proc = FakeProc::new();
    proc.process(11, "cargo", 1, 0);
    let mut watch = Watch::default();
    watch.scan(proc.path(), proc.uid());

    watch.forget();

    assert_eq!(watch.scan(proc.path(), proc.uid()).len(), 1);
}

#[test]
fn what_a_build_tool_or_nix_daemon_started_yields_with_it() {
    let processes = [
        process(1, "systemd", 0, 0),
        process(10, "foot", 1, 0),
        process(11, "cargo", 10, NICE),
        process(12, "rustc", 11, NICE),
        process(13, "build-script-bu", 11, NICE),
        process(20, "nix-daemon", 1, NICE),
        process(21, "bash", 20, NICE),
        process(30, "nice", 10, NICE),
        process(31, "rustc", 10, 0),
    ];

    let demoted: Vec<u32> = demoted(&processes).iter().map(|process| process.pid).collect();

    assert_eq!(demoted, [11, 12, 13, 20, 21]);
}

#[test]
fn the_summary_counts_by_name_the_most_first() {
    let processes = [
        process(11, "cargo", 1, NICE),
        process(12, "rustc", 11, NICE),
        process(13, "rustc", 11, NICE),
        process(14, "rustc", 11, NICE),
    ];
    let all: Vec<&Process> = processes.iter().collect();

    assert_eq!(summary(&all), "rustc ×3, cargo");
}

#[test]
fn the_report_names_the_rules_and_what_yields_now() {
    let root = FakeRoot::laptop();
    root.write(NIX_UNIT, "[Service]\n");
    let processes = [process(11, "cargo", 1, NICE), process(12, "rustc", 11, NICE)];

    let rows = details(true, &root.machine(), &processes);

    let labels: Vec<&str> = rows.iter().map(|(label, _)| label.as_str()).collect();
    assert_eq!(
        labels,
        ["Background work yields", "Build tools", "Nix's builds", "Yielding now"]
    );
    assert!(rows[1].1.starts_with("cargo, rustc"));
    assert_eq!(rows[2].1, "don't yield yet: the next stop writes nix-daemon's drop-in");
    assert_eq!(rows[3].1, "cargo, rustc");
}

#[test]
fn turned_off_the_report_says_so_and_what_still_yields() {
    let root = FakeRoot::laptop();
    root.write(NIX_UNIT, "[Service]\n");
    root.write(NIX_DROP_IN, "[Service]\nNice=19\n");

    let rows = details(false, &root.machine(), &[]);

    assert_eq!(
        rows,
        [
            (
                "Background work yields".to_owned(),
                "off (background_yields = false in power.toml)".to_owned()
            ),
            (
                "Nix's builds".to_owned(),
                "still yield: the next stop removes nix-daemon's drop-in".to_owned()
            ),
        ]
    );
}

fn sorted(mut pids: Vec<u32>) -> Vec<u32> {
    pids.sort_unstable();
    pids
}

#[test]
fn demoting_a_process_puts_it_at_nice_19() {
    let proc = Path::new("/proc");
    let mut child = std::process::Command::new("sleep")
        .arg("5")
        .spawn()
        .expect("start sleep");

    demote(proc, child.id());
    let after = fs::read_to_string(proc.join(child.id().to_string()).join("stat")).expect("its stat");
    child.kill().expect("stop sleep");
    child.wait().expect("reap sleep");

    assert_eq!(Process::parse(&after, 0).expect("parses").nice, NICE);
}
