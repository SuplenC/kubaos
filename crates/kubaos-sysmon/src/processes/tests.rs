use std::fs;
use std::path::Path;

use super::Order;
use super::ProcStat;
use super::Signal;
use super::SignalError;
use super::Tracker;
use super::details;
use super::find;
use super::parse_stat;
use super::scan;
use super::signal;

const PAGE: u64 = 4096;

fn stat_line(pid: i32, name: &str, parent: i32, ticks: u64, rss_pages: u64) -> String {
    format!(
        "{pid} ({name}) S {parent} {pid} {pid} 0 -1 4194560 100 0 0 0 {ticks} 0 0 0 20 0 7 0 5000 123456789 {rss_pages} 18446744073709551615 0 0 0 0 0 0 0 0 0 0 0 0 17 3 0 0 0 0 0\n"
    )
}

fn process(root: &Path, pid: i32, name: &str, ticks: u64, rss_pages: u64) {
    let dir = root.join(format!("proc/{pid}"));
    fs::create_dir_all(&dir).expect("make the process");
    fs::write(dir.join("stat"), stat_line(pid, name, 1, ticks, rss_pages)).expect("write its stat");
}

#[test]
fn a_stat_line_with_spaces_and_parens_in_its_name_is_read() {
    let line = "4242 (Web Content (x)) R 1000 4242 4242 0 -1 0 0 0 0 0 150 50 0 0 20 0 31 0 9000 0 2560 0\n";

    let stat = parse_stat(line, PAGE).expect("a stat");

    assert_eq!(stat.pid, 4242);
    assert_eq!(stat.name, "Web Content (x)");
    assert_eq!(stat.state, 'R');
    assert_eq!(stat.ppid, 1000);
    assert_eq!(stat.ticks, 200);
    assert_eq!(stat.threads, 31);
    assert_eq!(stat.start, 9000);
    assert_eq!(stat.memory, 2560 * PAGE);
}

#[test]
fn the_scan_reads_every_process_and_leaves_kernel_threads_out() {
    let root = tempfile::tempdir().expect("a temp dir");
    process(root.path(), 1, "systemd", 10, 100);
    process(root.path(), 900, "firefox", 500, 50_000);
    let kthread = root.path().join("proc/2");
    fs::create_dir_all(&kthread).expect("make kthreadd");
    fs::write(kthread.join("stat"), stat_line(2, "kthreadd", 0, 1, 0)).expect("write its stat");
    fs::create_dir_all(root.path().join("proc/self")).expect("make self");

    let mut found = scan(root.path(), PAGE);
    found.sort_by_key(|stat| stat.pid);

    let names: Vec<&str> = found.iter().map(|stat| stat.name.as_str()).collect();
    assert_eq!(names, ["systemd", "firefox"]);
}

fn stat(pid: i32, name: &str, ticks: u64, memory: u64) -> ProcStat {
    ProcStat {
        pid,
        name: name.to_owned(),
        state: 'S',
        ppid: 1,
        ticks,
        threads: 1,
        start: 0,
        memory,
    }
}

#[test]
fn cpu_use_is_each_process_share_of_the_machines_ticks_between_scans() {
    let mut tracker = Tracker::default();
    tracker.update(vec![stat(10, "a", 100, 0), stat(11, "b", 100, 0)], 10_000);

    let now = tracker.update(
        vec![stat(10, "a", 300, 0), stat(11, "b", 150, 0), stat(12, "new", 40, 0)],
        11_000,
    );

    let cpu = |pid: i32| now.iter().find(|p| p.pid == pid).expect("the process").cpu;
    assert!((cpu(10) - 20.0).abs() < 0.01);
    assert!((cpu(11) - 5.0).abs() < 0.01);
    // A process born since the last scan counts no history yet.
    assert!(cpu(12).abs() < 0.01);
}

#[test]
fn the_top_five_by_cpu_and_by_memory_come_sorted() {
    let mut tracker = Tracker::default();
    let first: Vec<ProcStat> = (0..8).map(|i| stat(i, &format!("p{i}"), 0, 0)).collect();
    tracker.update(first, 0);
    let second: Vec<ProcStat> = (0..8)
        .map(|i| stat(i, &format!("p{i}"), i as u64 * 10, (8 - i) as u64 * 1000))
        .collect();
    let list = tracker.update(second, 1000);

    let by_cpu = Order::Cpu.top(&list, 5);
    let by_memory = Order::Memory.top(&list, 5);

    let pids = |top: &[super::Process]| top.iter().map(|p| p.pid).collect::<Vec<_>>();
    assert_eq!(pids(&by_cpu), [7, 6, 5, 4, 3]);
    assert_eq!(pids(&by_memory), [0, 1, 2, 3, 4]);
}

#[test]
fn processes_are_found_by_name_and_grouped() {
    let list = vec![
        stat(100, "chrome", 0, 3000),
        stat(101, "chrome", 0, 2000),
        stat(102, "chrome_crashpad", 0, 100),
        stat(200, "firefox", 0, 5000),
    ];

    let groups = find(&list, "CHROME");

    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].name, "chrome");
    assert_eq!(groups[0].pids, [100, 101]);
    assert_eq!(groups[0].memory, 5000);
    assert_eq!(groups[1].name, "chrome_crashpad");
    assert!(find(&list, "nothing").is_empty());
}

#[test]
fn details_name_the_command_line_owner_and_start() {
    let root = tempfile::tempdir().expect("a temp dir");
    process(root.path(), 900, "firefox", 500, 50_000);
    let dir = root.path().join("proc/900");
    fs::write(dir.join("cmdline"), "/usr/lib/firefox/firefox\0--new-window\0").expect("write the cmdline");
    fs::write(dir.join("status"), "Name:\tfirefox\nUid:\t1000\t1000\t1000\t1000\n").expect("write the status");
    fs::create_dir_all(root.path().join("etc")).expect("make etc");
    fs::write(
        root.path().join("etc/passwd"),
        "root:x:0:0:root:/root:/bin/bash\nkuba:x:1000:1000:Kuba:/home/kuba:/bin/zsh\n",
    )
    .expect("write passwd");

    let found = details(root.path(), 900, Some(1_000_000), PAGE).expect("details");

    assert_eq!(found.command, "/usr/lib/firefox/firefox --new-window");
    assert_eq!(found.user, "kuba");
    assert_eq!(found.threads, 7);
    assert_eq!(found.memory, 50_000 * PAGE);
    // starttime 5000 ticks at 100 a second: 50 s after boot.
    assert_eq!(found.started, Some(1_000_050));
}

#[test]
fn a_process_that_is_gone_has_no_details() {
    let root = tempfile::tempdir().expect("a temp dir");

    assert!(details(root.path(), 4242, None, PAGE).is_none());
}

#[test]
fn init_and_the_monitor_itself_are_never_signalled() {
    assert!(matches!(signal(1, Signal::End), Err(SignalError::Protected(1))));
    assert!(matches!(signal(0, Signal::Kill), Err(SignalError::Protected(0))));
    let me = std::process::id() as i32;
    assert!(matches!(signal(me, Signal::Kill), Err(SignalError::Protected(_))));
}

#[test]
fn signalling_a_process_ends_it() {
    let mut child = std::process::Command::new("sleep")
        .arg("30")
        .spawn()
        .expect("start sleep");

    signal(child.id() as i32, Signal::End).expect("end it");

    let status = child.wait().expect("wait for it");
    assert!(!status.success());
}
