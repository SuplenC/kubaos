use super::*;
use crate::fake::FakeRoot;

fn hw(total: u64, last: u64, max: u64) -> HwSleep {
    HwSleep {
        total_us: Some(total),
        last_us: Some(last),
        max_us: Some(max),
    }
}

#[test]
fn hardware_sleep_is_the_totals_difference_in_microseconds() {
    let before = hw(5_000_000, 5_000_000, u64::MAX);
    let after = hw(3_605_000_000, 3_600_000_000, u64::MAX);

    assert_eq!(hw_sleep_between(before, after, 4000), Some(3_600_000_000));
}

#[test]
fn a_sleep_that_reached_nothing_is_zero_not_unknown() {
    let before = hw(5_000_000, 5_000_000, u64::MAX);
    let after = hw(5_000_000, 0, u64::MAX);

    assert_eq!(hw_sleep_between(before, after, 5100), Some(0));
}

#[test]
fn a_machine_without_the_files_is_unknown_not_zero() {
    assert_eq!(hw_sleep_between(HwSleep::default(), HwSleep::default(), 5100), None);
}

#[test]
fn a_kernel_with_no_driver_reporting_is_unknown_not_zero() {
    assert_eq!(hw_sleep_between(hw(0, 0, 0), hw(0, 0, 0), 5100), None);
}

#[test]
fn a_sleep_longer_than_the_hardware_can_count_is_unknown() {
    let max = 3600 * 1_000_000;

    assert_eq!(hw_sleep_between(hw(0, 0, max), hw(max, max, max), 3601), None);
    assert_eq!(hw_sleep_between(hw(0, 0, max), hw(max, max, max), 3600), Some(max));
}

#[test]
fn a_total_that_went_down_is_unknown() {
    assert_eq!(hw_sleep_between(hw(10, 0, u64::MAX), hw(5, 0, u64::MAX), 60), None);
}

#[test]
fn the_real_kernels_max_value_parses_and_means_no_limit() {
    let root = FakeRoot::empty();
    root.write("sys/power/suspend_stats/total_hw_sleep", "7");
    root.write("sys/power/suspend_stats/last_hw_sleep", "7");
    root.write("sys/power/suspend_stats/max_hw_sleep", "18446744073709551615");

    let read = root.machine().hw_sleep();

    assert_eq!(read, hw(7, 7, u64::MAX));
    assert_eq!(hw_sleep_between(hw(0, 0, u64::MAX), read, 3600 * 24 * 365), Some(7));
}

#[test]
fn keep_awake_holds_while_on_and_not_past_its_end_and_ignores_fields_it_does_not_know() {
    assert!(keep_awake_holds(r#"{"active":true,"until":0,"battery":0}"#, 100));
    assert!(keep_awake_holds(
        r#"{"active":true,"until":200,"battery":0,"lid":true}"#,
        100
    ));
    assert!(!keep_awake_holds(r#"{"active":true,"until":50}"#, 100));
    assert!(!keep_awake_holds(r#"{"active":false,"until":0,"battery":0}"#, 100));
    assert!(!keep_awake_holds("not json", 100));
}

#[test]
fn a_program_with_many_processes_counts_as_one_and_loses_its_pids() {
    let rates = vec![
        ("kworker/0:1 (12)".to_owned(), 3.0),
        ("kworker/0:1 (13)".to_owned(), 2.0),
        ("qs (40)".to_owned(), 9.0),
        ("Web Content (77)".to_owned(), 1.0),
    ];

    assert_eq!(
        by_name(rates),
        [
            ("qs".to_owned(), 9.0),
            ("kworker/0:1".to_owned(), 5.0),
            ("Web Content".to_owned(), 1.0)
        ]
    );
}

fn counting_root() -> FakeRoot {
    let root = FakeRoot::laptop();
    root.write("proc/stat", "cpu  100 0 100 800 0 0 0 0 0 0\ncpu0 1 1 1 1\n");
    root.write(
        "proc/interrupts",
        "           CPU0       CPU1\n  24:        10         10   PCI-MSI 1  edge      amdgpu\n",
    );
    root.write("proc/40/comm", "qs");
    root.write(
        "proc/40/task/40/status",
        "voluntary_ctxt_switches:\t100\nnonvoluntary_ctxt_switches:\t0",
    );
    root
}

#[test]
fn the_first_sample_starts_the_counts_and_every_tenth_names_the_wakers() {
    let root = counting_root();
    let mut reader = Reader::new(root.machine(), None);
    let start = Instant::now();

    let first = reader.read(0, start);
    root.write("proc/stat", "cpu  150 0 100 850 0 0 0 0 0 0\n");
    let second = reader.read(60, start + Duration::from_mins(1));
    for minute in 2..SCAN_EVERY {
        assert_eq!(
            reader.read(0, start + Duration::from_mins(u64::from(minute))).wakers,
            None
        );
    }
    root.write(
        "proc/interrupts",
        "           CPU0       CPU1\n  24:      6010       6010   PCI-MSI 1  edge      amdgpu\n",
    );
    root.write(
        "proc/40/task/40/status",
        "voluntary_ctxt_switches:\t1300\nnonvoluntary_ctxt_switches:\t0",
    );
    let tenth = reader.read(600, start + Duration::from_mins(u64::from(SCAN_EVERY)));

    assert_eq!(first.wakers, None);
    assert_eq!(first.cpu, None);
    assert_eq!(second.wakers, None);
    assert_eq!(second.cpu, Some(50.0));
    assert_eq!(
        tenth.wakers,
        Some(Wakers {
            secs: 600,
            irqs: vec![("24 amdgpu".to_owned(), 20.0)],
            procs: vec![("qs".to_owned(), 2.0)],
        })
    );
}

#[test]
fn forgetting_starts_the_counts_again() {
    let root = counting_root();
    let mut reader = Reader::new(root.machine(), None);
    let start = Instant::now();
    reader.read(0, start);

    reader.forget();
    root.write("proc/stat", "cpu  150 0 100 850 0 0 0 0 0 0\n");
    let next = reader.read(60, start + Duration::from_mins(1));

    assert_eq!(next.cpu, None);
    assert_eq!(next.wakers, None);
}

#[test]
fn a_sample_reads_the_battery_the_panel_the_backlight_and_keep_awake() {
    let root = counting_root();
    root.write("sys/class/drm/card1-eDP-1/dpms", "Off");
    root.write("sys/class/backlight/amdgpu_bl1/brightness", "128");
    root.write("sys/class/backlight/amdgpu_bl1/max_brightness", "255");
    root.write(
        "runtime/kubaos-keep-awake.json",
        r#"{"active":true,"until":0,"battery":0}"#,
    );
    let mut reader = Reader::new(root.machine(), Some(root.path("runtime/kubaos-keep-awake.json")));

    let reading = reader.read(100, Instant::now());

    assert_eq!(reading.battery.and_then(|battery| battery.percent), Some(64.0));
    assert_eq!(reading.panel_on, Some(false));
    assert_eq!(reading.brightness, Some(50));
    assert!(reading.keep_awake);
    assert!(reading.docked);
}

#[test]
fn a_live_count_measures_over_its_window_and_waits_through_the_given_sleep() {
    let root = counting_root();
    let machine = root.machine();
    let window = Duration::from_secs(1);

    let wakers = wakers_now(&machine, window, &mut |waited| {
        assert_eq!(waited, window);
        root.write(
            "proc/40/task/40/status",
            "voluntary_ctxt_switches:\t150\nnonvoluntary_ctxt_switches:\t0",
        );
    });

    assert_eq!(wakers.expect("a count").procs, [("qs".to_owned(), 50.0)]);
}
