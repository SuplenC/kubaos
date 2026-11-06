use jiff::civil::date;

use super::*;
use crate::fake::FakeRoot;

const UTC: TimeZone = TimeZone::UTC;

fn at(month: i8, day: i8, hour: i8, minute: i8) -> i64 {
    date(2026, month, day)
        .at(hour, minute, 0, 0)
        .to_zoned(UTC)
        .expect("a time")
        .timestamp()
        .as_second()
}

/// A stretch of samples a minute apart from `start` to `end`, the charge falling evenly, the energy
/// beside it when `wh` says from where to where.
#[expect(
    clippy::too_many_arguments,
    reason = "Each fixture line reads as the stretch it stands for, every field in view."
)]
fn samples(
    start: i64,
    end: i64,
    percent: (f64, f64),
    wh: Option<(f64, f64)>,
    watts: f64,
    stop: &str,
    lid_closed: bool,
    keep_awake: bool,
) -> Vec<Entry> {
    let minutes = (end - start) / 60;
    (0..=minutes)
        .map(|minute| {
            let done = minute as f64 / minutes as f64;
            let between = |(from, to): (f64, f64)| from + (to - from) * done;
            Entry::Sample(Sample {
                at: start + minute * 60,
                percent: Some(between(percent).round()),
                wh: wh.map(between),
                watts: Some(watts),
                stop: stop.to_owned(),
                lid_closed: Some(lid_closed),
                keep_awake,
                wakers: (minute > 0 && minute % 10 == 0).then(|| Wakers {
                    secs: 600,
                    irqs: vec![("24 amdgpu".to_owned(), 120.0)],
                    procs: vec![("qs".to_owned(), 30.0), ("kubaos-sysmon".to_owned(), 4.0)],
                }),
                ..Sample::default()
            })
        })
        .collect()
}

/// Tuesday morning's failed sleep, Friday night's lid-closed drain, a good sleep and an afternoon.
fn week() -> Vec<Entry> {
    let mut entries = Vec::new();
    entries.push(Entry::Sleep(Sleep {
        at: at(11, 3, 7, 51),
        end: at(11, 3, 9, 16),
        trigger: Some("lid".to_owned()),
        percent_before: Some(100.0),
        percent_after: Some(96.0),
        percent_per_hour: Some(2.82),
        hw_sleep_us: Some(0),
        last_hw_sleep_us: Some(0),
        hw_share: Some(0.0),
        ..Sleep::default()
    }));
    entries.extend(samples(
        at(11, 4, 14, 0),
        at(11, 4, 15, 30),
        (90.0, 70.0),
        Some((72.0, 56.0)),
        10.7,
        "balanced",
        false,
        false,
    ));
    entries.push(Entry::Sleep(Sleep {
        at: at(11, 4, 23, 0),
        end: at(11, 5, 7, 0),
        percent_before: Some(70.0),
        percent_after: Some(68.0),
        wh_before: Some(56.0),
        wh_after: Some(54.4),
        watts: Some(0.2),
        percent_per_hour: Some(0.25),
        hw_sleep_us: Some(27_360_000_000),
        last_hw_sleep_us: Some(27_360_000_000),
        hw_share: Some(0.95),
        ..Sleep::default()
    }));
    entries.extend(samples(
        at(11, 6, 0, 50),
        at(11, 6, 10, 16),
        (99.0, 15.0),
        Some((79.2, 12.0)),
        7.1,
        "power-saver",
        true,
        true,
    ));
    entries
}

fn report() -> Report {
    Report::of(&week(), 7, at(11, 6, 12, 0))
}

#[test]
fn what_went_wrong_comes_first_then_the_rest_newest_first() {
    let kinds: Vec<(Kind, bool)> = report()
        .stretches
        .iter()
        .map(|stretch| (stretch.kind, stretch.went_wrong))
        .collect();

    assert_eq!(
        kinds,
        [
            (Kind::AwakeLidClosed, true),
            (Kind::AsleepShallow, true),
            (Kind::AsleepDeep, false),
            (Kind::AwakeLidOpen, false),
        ]
    );
}

#[test]
fn the_night_of_november_6_reads_as_it_happened() {
    let report = report();
    let night = &report.stretches[0];

    assert_eq!(night.start, at(11, 6, 0, 50));
    assert_eq!(night.end, at(11, 6, 10, 16));
    assert_eq!(night.percent_before, Some(99.0));
    assert_eq!(night.percent_after, Some(15.0));
    assert_eq!(night.lost_percent, Some(84.0));
    assert_eq!(night.lost_wh, Some(67.2));
    assert_eq!(night.watts, Some(7.1));
    assert!(night.keep_awake);
}

#[test]
fn the_failed_sleep_of_tuesday_reads_as_it_happened() {
    let report = report();
    let sleep = &report.stretches[1];

    assert_eq!(sleep.start, at(11, 3, 7, 51));
    assert_eq!(sleep.end, at(11, 3, 9, 16));
    assert_eq!(sleep.deep_percent, Some(0.0));
    assert_eq!(sleep.lost_percent, Some(4.0));
    assert_eq!(sleep.percent_per_hour, Some(2.82));
}

#[test]
fn the_text_is_the_one_a_person_reads() {
    assert_eq!(
        report().text(&UTC),
        "\
Battery report: the last 7 days, from Fri Oct 30 12:00

Went wrong
  Fri Nov 6 00:50 to 10:16 (9 h 26 min): awake, lid closed, Keep awake on
    99 → 15 %, lost 84 % (67.2 Wh), 7.1 W
  Tue Nov 3 07:51 to 09:16 (1 h 25 min): asleep, 0 % deep sleep
    100 → 96 %, lost 4 %, 2.8 %/h

Long stretches
  Wed Nov 4 23:00 to Thu Nov 5 07:00 (8 h): asleep, 95 % deep sleep
    70 → 68 %, lost 2 % (1.6 Wh), 0.2 W
  Wed Nov 4 14:00 to 15:30 (1 h 30 min): awake, lid open
    90 → 70 %, lost 20 % (16.0 Wh), 10.7 W

Average draw on battery
  Balanced, lid open: 10.7 W over 1 h 31 min
  Power Saver, lid closed: 7.1 W over 9 h 27 min

Top interrupts (per second)
      120.0  24 amdgpu

Top processes waking (per second)
       30.0  qs
        4.0  kubaos-sysmon
"
    );
}

#[test]
fn a_gauge_that_jumps_shows_beside_the_energy() {
    let entries = samples(
        at(11, 6, 9, 0),
        at(11, 6, 9, 40),
        (15.0, 6.0),
        Some((12.0, 10.5)),
        2.2,
        "super-saver",
        false,
        false,
    );

    let report = Report::of(&entries, 7, at(11, 6, 12, 0));

    assert!(report.text(&UTC).contains("15 → 6 %, lost 9 % (1.5 Wh), 2.2 W"));
}

#[test]
fn a_sleep_with_a_charger_shows_no_loss_and_never_went_wrong() {
    let entries = [Entry::Sleep(Sleep {
        at: at(11, 5, 0, 0),
        end: at(11, 5, 8, 0),
        percent_before: Some(50.0),
        percent_after: Some(80.0),
        charger: true,
        hw_share: Some(0.0),
        hw_sleep_us: Some(0),
        ..Sleep::default()
    })];

    let report = Report::of(&entries, 7, at(11, 6, 12, 0));

    assert!(!report.stretches[0].went_wrong);
    assert_eq!(report.stretches[0].lost_percent, None);
    assert!(report.text(&UTC).contains("50 → 80 %, with a charger in"));
}

#[test]
fn a_sleep_on_a_machine_that_does_not_report_is_unknown_not_wrong() {
    let entries = [Entry::Sleep(Sleep {
        at: at(11, 5, 0, 0),
        end: at(11, 5, 8, 0),
        percent_before: Some(50.0),
        percent_after: Some(40.0),
        percent_per_hour: Some(1.25),
        ..Sleep::default()
    })];

    let report = Report::of(&entries, 7, at(11, 6, 12, 0));

    assert_eq!(report.stretches[0].kind, Kind::AsleepUnknown);
    assert!(!report.stretches[0].went_wrong);
    assert!(report.text(&UTC).contains("asleep, deep sleep unknown"));
}

#[test]
fn a_short_stretch_gets_no_line_but_still_counts_in_the_averages() {
    let entries = samples(
        at(11, 6, 9, 0),
        at(11, 6, 9, 10),
        (50.0, 49.0),
        None,
        9.0,
        "balanced",
        false,
        false,
    );

    let report = Report::of(&entries, 7, at(11, 6, 12, 0));

    assert_eq!(report.stretches, []);
    assert_eq!(report.averages[0].minutes, 11);
}

#[test]
fn a_gap_or_a_lid_that_moved_splits_a_stretch() {
    let mut entries = samples(
        at(11, 6, 1, 0),
        at(11, 6, 2, 0),
        (90.0, 85.0),
        None,
        5.0,
        "balanced",
        false,
        false,
    );
    entries.extend(samples(
        at(11, 6, 2, 1),
        at(11, 6, 3, 0),
        (85.0, 80.0),
        None,
        5.0,
        "balanced",
        true,
        false,
    ));
    entries.extend(samples(
        at(11, 6, 5, 0),
        at(11, 6, 6, 0),
        (80.0, 75.0),
        None,
        5.0,
        "balanced",
        true,
        false,
    ));

    let report = Report::of(&entries, 7, at(11, 6, 12, 0));

    assert_eq!(report.stretches.len(), 3);
}

#[test]
fn an_empty_history_says_so() {
    let report = Report::of(&[], 7, at(11, 6, 12, 0));

    assert!(report.text(&UTC).contains("Nothing written down yet"));
}

#[test]
fn the_days_stay_within_what_the_file_keeps() {
    let now = at(11, 6, 12, 0);

    assert_eq!(since(1000, now), now - RETENTION_DAYS * DAY);
    assert_eq!(Report::of(&[], 1000, now).days, RETENTION_DAYS);
    assert_eq!(since(0, now), now - DAY);
}

#[test]
fn the_json_says_the_same() {
    let text = serde_json::to_string(&report()).expect("JSON");

    assert!(text.contains(r#""kind":"awake-lid-closed""#));
    assert!(text.contains(r#""went_wrong":true"#));
    assert!(text.contains(r#""irqs":[{"name":"24 amdgpu","rate":120.0}]"#));
}

#[test]
fn the_wakers_weigh_each_count_by_how_long_it_ran() {
    let sample = |secs, rate| Sample {
        wakers: Some(Wakers {
            secs,
            irqs: Vec::new(),
            procs: vec![("qs".to_owned(), rate)],
        }),
        ..Sample::default()
    };
    let (long, short) = (sample(900, 10.0), sample(100, 110.0));

    let (_, procs) = top_wakers(&[&long, &short]);

    assert_eq!(
        procs,
        [Waker {
            name: "qs".to_owned(),
            rate: 20.0
        }]
    );
}

// The menu's short answer.

fn drain_root() -> FakeRoot {
    let root = FakeRoot::laptop();
    root.write("proc/40/comm", "qs");
    root.write(
        "proc/40/task/40/status",
        "voluntary_ctxt_switches:\t100\nnonvoluntary_ctxt_switches:\t0",
    );
    root
}

#[test]
fn on_battery_a_recent_count_answers_at_once() {
    let root = drain_root();
    let path = root.path("history.jsonl");
    let now = at(11, 6, 12, 0);
    for entry in samples(now - 1200, now - 60, (70.0, 69.0), None, 7.0, "balanced", false, false) {
        history::append(&path, &entry).expect("append");
    }

    let drain = Drain::now(&root.machine(), "Balanced", Some(&path), now, &mut |_| {
        panic!("no waiting")
    });

    assert_eq!(
        drain.rows(),
        [
            ("Drawing 16.8 W now".to_owned(), "Balanced, 64 % left".to_owned()),
            ("qs: 30 a second".to_owned(), "A process waking".to_owned()),
            ("kubaos-sysmon: 4 a second".to_owned(), "A process waking".to_owned()),
            ("24 amdgpu: 120 a second".to_owned(), "An interrupt line".to_owned()),
        ]
    );
}

#[test]
fn on_battery_with_no_recent_count_it_measures_for_a_moment() {
    let root = drain_root();
    let mut waited = None;

    let drain = Drain::now(&root.machine(), "Balanced", None, 1000, &mut |window| {
        waited = Some(window);
        root.write(
            "proc/40/task/40/status",
            "voluntary_ctxt_switches:\t160\nnonvoluntary_ctxt_switches:\t0",
        );
    });

    assert_eq!(waited, Some(DRAIN_WINDOW));
    assert_eq!(drain.wakers.expect("a count").procs, [("qs".to_owned(), 60.0)]);
}

#[test]
fn an_old_count_is_no_answer() {
    let entries = samples(0, 1200, (70.0, 69.0), None, 7.0, "balanced", false, false);

    assert_eq!(last_wakers(&entries, 1200 + DRAIN_FRESH_SECONDS + 1), None);
    assert!(last_wakers(&entries, 1200).is_some());
}

#[test]
fn plugged_in_nothing_is_draining_and_nothing_is_measured() {
    let root = drain_root();
    root.write("sys/class/power_supply/BAT0/status", "Charging");

    let drain = Drain::now(&root.machine(), "Balanced", None, 1000, &mut |_| panic!("no waiting"));

    assert_eq!(
        drain.rows(),
        [(
            "Plugged in: nothing is draining the battery".to_owned(),
            "Balanced, 64 % left".to_owned()
        )]
    );
}
