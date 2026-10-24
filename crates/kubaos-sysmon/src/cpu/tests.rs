use std::fs;

use super::Times;
use super::frequency_mhz;
use super::parse_loadavg;
use super::parse_stat;

const STAT: &str = "\
cpu  4705 356 584 3699 23 23 0 0 0 0
cpu0 1393 280 307 1817 12 13 0 0 0 0
cpu1 3312 76 277 1882 11 10 0 0 0 0
intr 114930548 113199788 3 0 5 263 0 4 [... lots more numbers ...]
ctxt 1990473
btime 1062191376
processes 2915
procs_running 1
";

#[test]
fn the_total_and_each_core_are_read_from_proc_stat() {
    let stat = parse_stat(STAT);

    assert_eq!(stat.cores.len(), 2);
    assert_eq!(stat.total.total, 4705 + 356 + 584 + 3699 + 23 + 23);
    assert_eq!(stat.total.idle, 3699 + 23);
    assert_eq!(stat.boot_time, Some(1_062_191_376));
}

#[test]
fn usage_is_the_busy_share_of_the_ticks_between_two_reads() {
    let before = Times { total: 1000, idle: 800 };
    let after = Times { total: 1200, idle: 850 };

    let usage = after.usage_since(&before).expect("ticks went by");

    assert!((usage - 75.0).abs() < 0.01, "{usage}");
}

#[test]
fn no_ticks_between_two_reads_is_no_usage() {
    let times = Times { total: 1000, idle: 800 };

    assert_eq!(times.usage_since(&times), None);
}

#[test]
fn guest_time_is_not_counted_twice() {
    // The guest columns are already inside `user` and `nice`.
    let stat = parse_stat("cpu  100 0 0 100 0 0 0 0 50 50\n");

    assert_eq!(stat.total.total, 200);
}

#[test]
fn the_load_average_is_its_three_numbers() {
    assert_eq!(parse_loadavg("0.52 1.04 0.98 2/811 12345\n"), Some([0.52, 1.04, 0.98]));
    assert_eq!(parse_loadavg("garbage"), None);
}

#[test]
fn the_clock_speed_is_the_cores_average_in_mhz() {
    let root = tempfile::tempdir().expect("a temp dir");
    for (core, khz) in [(0, "3000000"), (1, "1000000")] {
        let dir = root.path().join(format!("sys/devices/system/cpu/cpu{core}/cpufreq"));
        fs::create_dir_all(&dir).expect("make cpufreq");
        fs::write(dir.join("scaling_cur_freq"), format!("{khz}\n")).expect("write the speed");
    }

    assert_eq!(frequency_mhz(root.path(), 2), Some(2000));
    assert_eq!(frequency_mhz(&root.path().join("nothing"), 2), None);
}
