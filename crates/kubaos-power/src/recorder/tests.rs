use super::*;
use crate::machine::HwSleep;

const HOUR: i64 = 3600;

fn battery(percent: f64, wh: Option<f64>, discharging: bool) -> BatteryNow {
    BatteryNow {
        watts: discharging.then_some(7.1),
        percent: Some(percent),
        wh,
        discharging,
    }
}

fn hw(total: u64, last: u64) -> HwSleep {
    HwSleep {
        total_us: Some(total),
        last_us: Some(last),
        max_us: Some(u64::MAX),
    }
}

fn edge(at: i64, percent: f64, total_hw: u64) -> Edge {
    Edge {
        at,
        battery: Some(battery(percent, None, true)),
        hw: hw(total_hw, total_hw),
    }
}

fn on_battery() -> Facts<'static> {
    Facts {
        on_battery: true,
        stop: "power-saver",
        dropped_from: None,
    }
}

fn plugged_in() -> Facts<'static> {
    Facts {
        on_battery: false,
        stop: "balanced",
        dropped_from: None,
    }
}

fn whats(out: &Out) -> Vec<What> {
    out.entries
        .iter()
        .filter_map(|entry| match entry {
            Entry::Event(event) => Some(event.what),
            _ => None,
        })
        .collect()
}

/// Sleeps from `at` for `seconds`, with `total_hw` microseconds of hardware sleep, and comes back
/// with what the wake's reading said.
fn sleep_through(recorder: &mut Recorder, at: i64, seconds: i64, from: f64, to: f64, total_hw: u64) -> Out {
    let now = Instant::now();
    recorder.going_to_sleep(edge(at, from, 0));
    recorder.woke(at + seconds, now);
    recorder.after_wake(&edge(at + seconds + 5, to, total_hw), now + WAKE_READING)
}

#[test]
fn the_first_word_marks_the_service_start_and_says_no_change() {
    let mut recorder = Recorder::default();

    let out = recorder.follow(&on_battery(), 10, Some(64.0), true, Instant::now());

    assert_eq!(whats(&out), [What::ServiceStart]);
    assert!(!recorder.changes(&on_battery()));
}

#[test]
fn a_plug_a_stop_and_a_drop_are_each_written_once() {
    let now = Instant::now();
    let mut recorder = Recorder::default();
    recorder.follow(&plugged_in(), 10, Some(64.0), false, now);

    let unplug = recorder.follow(&on_battery(), 20, Some(64.0), false, now);
    let same = recorder.changes(&on_battery());
    let drop = recorder.follow(
        &Facts {
            on_battery: true,
            stop: "super-saver",
            dropped_from: Some("power-saver"),
        },
        30,
        Some(20.0),
        false,
        now,
    );

    assert_eq!(whats(&unplug), [What::Unplug, What::Stop]);
    assert!(unplug.forget);
    assert!(!same);
    assert_eq!(whats(&drop), [What::Stop, What::LowBattery]);
    assert!(!drop.forget);
}

#[test]
fn no_timer_runs_while_plugged_in_or_asleep_and_one_a_minute_on_battery() {
    let now = Instant::now();
    let mut recorder = Recorder::default();

    recorder.follow(&plugged_in(), 10, Some(64.0), false, now);
    assert_eq!(recorder.wait(now), None);

    recorder.follow(&on_battery(), 20, Some(64.0), false, now);
    assert_eq!(recorder.wait(now), Some(SAMPLE_EVERY));
    assert_eq!(recorder.due(now), None);
    assert_eq!(recorder.due(now + SAMPLE_EVERY), Some(Due::Sample));

    recorder.going_to_sleep(edge(30, 64.0, 0));
    assert_eq!(recorder.wait(now), None);
    assert_eq!(recorder.due(now + SAMPLE_EVERY), None);

    recorder.woke(40, now);
    assert_eq!(recorder.wait(now), Some(WAKE_READING));
    assert_eq!(recorder.due(now + WAKE_READING), Some(Due::Wake));

    recorder.after_wake(&edge(45, 63.0, 0), now + WAKE_READING);
    assert_eq!(recorder.wait(now + WAKE_READING), Some(SAMPLE_EVERY));
}

#[test]
fn a_sample_is_written_on_battery_and_the_next_is_a_minute_on() {
    let now = Instant::now();
    let mut recorder = Recorder::default();
    recorder.follow(&on_battery(), 10, Some(64.0), false, now);
    recorder.lid(true, 11, Some(64.0), false);

    let out = recorder.sample(
        Reading {
            battery: Some(battery(64.0, Some(51.206), true)),
            panel_on: Some(false),
            brightness: Some(40),
            docked: true,
            keep_awake: false,
            cpu: Some(3.2),
            wakers: None,
        },
        70,
        now + SAMPLE_EVERY,
    );

    assert_eq!(
        out.entries,
        [Entry::Sample(Sample {
            at: 70,
            percent: Some(64.0),
            wh: Some(51.21),
            watts: Some(7.1),
            stop: "power-saver".to_owned(),
            lid_closed: Some(true),
            panel_on: Some(false),
            brightness: Some(40),
            keep_awake: false,
            docked: true,
            cpu: Some(3.2),
            wakers: None,
        })]
    );
    assert_eq!(recorder.wait(now + SAMPLE_EVERY), Some(SAMPLE_EVERY));
}

#[test]
fn a_battery_that_stopped_discharging_writes_no_sample() {
    let now = Instant::now();
    let mut recorder = Recorder::default();
    recorder.follow(&on_battery(), 10, Some(64.0), false, now);

    let out = recorder.sample(
        Reading {
            battery: Some(battery(64.0, None, false)),
            ..Reading::default()
        },
        70,
        now,
    );

    assert_eq!(out.entries, []);
}

#[test]
fn keep_awake_turning_on_and_off_is_an_event_seen_by_the_samples() {
    let now = Instant::now();
    let mut recorder = Recorder::default();
    recorder.follow(&on_battery(), 10, Some(64.0), false, now);
    let reading = |keep_awake| Reading {
        battery: Some(battery(64.0, None, true)),
        keep_awake,
        ..Reading::default()
    };

    let on = recorder.sample(reading(true), 70, now);
    let still = recorder.sample(reading(true), 130, now);
    let off = recorder.sample(reading(false), 190, now);

    assert_eq!(whats(&on), [What::KeepAwakeOn]);
    assert_eq!(whats(&still), []);
    assert_eq!(whats(&off), [What::KeepAwakeOff]);
}

#[test]
fn the_lids_first_word_is_no_event_and_its_moves_are() {
    let now = Instant::now();
    let mut recorder = Recorder::default();
    recorder.follow(&on_battery(), 10, Some(64.0), false, now);

    let first = recorder.lid(false, 11, Some(64.0), false);
    let closed = recorder.lid(true, 12, Some(64.0), false);
    let opened = recorder.lid(false, 13, Some(64.0), false);

    assert_eq!(whats(&first), []);
    assert_eq!(whats(&closed), [What::LidClosed]);
    assert_eq!(whats(&opened), [What::LidOpen]);
}

// The sleep's calculation.

#[test]
fn a_sleep_entry_comes_from_the_battery_and_the_counters_at_either_end() {
    let before = Edge {
        at: 1000,
        battery: Some(battery(100.0, Some(80.0), true)),
        hw: hw(5_000_000, 5_000_000),
    };
    let after = Edge {
        at: 1000 + 2 * HOUR + 5,
        battery: Some(battery(98.0, Some(78.4), true)),
        hw: hw(5_000_000 + 7_000_000_000, 7_000_000_000),
    };

    let sleep = sleep_entry(&before, &after, 1000 + 2 * HOUR, Some("lid"));

    assert_eq!(sleep.at, 1000);
    assert_eq!(sleep.end, 1000 + 2 * HOUR);
    assert_eq!(sleep.trigger.as_deref(), Some("lid"));
    assert_eq!(sleep.hw_sleep_us, Some(7_000_000_000));
    assert_eq!(sleep.last_hw_sleep_us, Some(7_000_000_000));
    assert_eq!(sleep.hw_share, Some(0.972));
    assert_eq!(sleep.watts, Some(0.8));
    assert_eq!(sleep.percent_per_hour, Some(1.0));
    assert!(!sleep.charger);
}

#[test]
fn a_machine_that_reports_no_hardware_sleep_has_no_share_and_no_toast() {
    let before = Edge {
        at: 0,
        battery: Some(battery(100.0, None, true)),
        hw: HwSleep::default(),
    };
    let after = Edge {
        at: 2 * HOUR,
        battery: Some(battery(90.0, None, true)),
        hw: HwSleep::default(),
    };

    let sleep = sleep_entry(&before, &after, 2 * HOUR, None);

    assert_eq!(sleep.hw_sleep_us, None);
    assert_eq!(sleep.hw_share, None);
    assert_eq!(sleep_note(&sleep), None);
}

#[test]
fn hardware_sleep_longer_than_the_clock_says_is_all_of_it() {
    let before = edge(0, 100.0, 0);
    let after = edge(HOUR, 100.0, 2 * 3_600_000_000);

    assert_eq!(sleep_entry(&before, &after, HOUR, None).hw_share, Some(1.0));
}

#[test]
fn a_charger_at_either_end_or_a_charge_that_rose_leaves_the_rates_out() {
    let draining = |percent| Edge {
        at: 0,
        battery: Some(battery(percent, None, true)),
        hw: hw(0, 0),
    };
    let charging = |percent| Edge {
        at: 0,
        battery: Some(battery(percent, None, false)),
        hw: hw(0, 0),
    };
    for (before, after) in [
        (charging(50.0), draining(40.0)),
        (draining(50.0), charging(60.0)),
        (draining(50.0), draining(60.0)),
    ] {
        let sleep = sleep_entry(&before, &after, 2 * HOUR, None);

        assert!(sleep.charger);
        assert_eq!(sleep.percent_per_hour, None);
        assert_eq!(sleep.watts, None);
        assert_eq!(sleep_note(&sleep), None);
    }
}

#[test]
fn the_energys_noise_is_not_a_charger_but_a_real_rise_is() {
    let at = |wh| Edge {
        at: 0,
        battery: Some(battery(50.0, Some(wh), true)),
        hw: hw(0, 0),
    };

    assert!(!sleep_entry(&at(40.00), &at(40.01), HOUR, None).charger);
    assert!(sleep_entry(&at(40.0), &at(41.0), HOUR, None).charger);
}

#[test]
fn the_sleep_says_it_came_from_the_lid_when_the_lid_was_closed() {
    let now = Instant::now();
    let mut recorder = Recorder::default();
    recorder.follow(&on_battery(), 10, Some(64.0), false, now);
    recorder.lid(true, 11, Some(64.0), false);

    let out = sleep_through(&mut recorder, 20, 100, 64.0, 64.0, 0);

    let [Entry::Sleep(sleep)] = out.entries.as_slice() else {
        panic!("a sleep");
    };
    assert_eq!(sleep.trigger.as_deref(), Some("lid"));
}

#[test]
fn a_wake_with_no_sleep_heard_writes_nothing() {
    let now = Instant::now();
    let mut recorder = Recorder::default();
    recorder.follow(&on_battery(), 10, Some(64.0), false, now);

    recorder.woke(50, now);

    assert_eq!(recorder.due(now + WAKE_READING), None);
    assert_eq!(recorder.after_wake(&edge(55, 64.0, 0), now).entries, []);
}

#[test]
fn a_sleep_soon_after_a_wake_finishes_the_first_with_what_it_read() {
    let now = Instant::now();
    let mut recorder = Recorder::default();
    recorder.follow(&on_battery(), 10, Some(64.0), false, now);
    recorder.going_to_sleep(edge(100, 64.0, 0));
    recorder.woke(200, now);

    let out = recorder.going_to_sleep(edge(202, 63.0, 0));

    let [Entry::Sleep(sleep)] = out.entries.as_slice() else {
        panic!("the first sleep");
    };
    assert_eq!((sleep.at, sleep.end), (100, 200));
    assert_eq!(sleep.percent_after, Some(63.0));
    assert_eq!(recorder.wait(now), None);
}

// The toasts.

#[test]
fn a_long_sleep_with_no_hardware_sleep_gets_the_toast_of_tuesday_morning() {
    let now = Instant::now();
    let mut recorder = Recorder::default();
    recorder.follow(&on_battery(), 10, Some(100.0), false, now);

    let out = sleep_through(&mut recorder, 100, HOUR + 25 * 60, 100.0, 96.0, 0);

    assert_eq!(
        out.notes,
        [Note::Slept {
            seconds: HOUR + 25 * 60,
            lost: 4.0,
            share: 0.0
        }]
    );
    assert_eq!(
        out.notes[0].text(),
        "Slept 1 h 25 min and lost 4 %: it didn't reach deep sleep."
    );
}

#[test]
fn a_sleep_that_reached_some_hardware_sleep_says_how_much() {
    let note = Note::Slept {
        seconds: 2 * HOUR,
        lost: 3.0,
        share: 0.2,
    };

    assert_eq!(note.text(), "Slept 2 h and lost 3 %: only 20 % of it in deep sleep.");
}

#[test]
fn a_short_sleep_or_a_deep_one_is_silent() {
    let mut recorder = Recorder::default();
    recorder.follow(&on_battery(), 10, Some(100.0), false, Instant::now());

    let short = sleep_through(&mut recorder, 100, LONG_SLEEP_SECONDS, 100.0, 99.0, 0);
    let half = sleep_through(&mut recorder, 10_000, 2 * HOUR, 99.0, 98.0, 3_600_000_000);

    assert_eq!(short.notes, []);
    assert_eq!(half.notes, []);
}

#[test]
fn just_under_half_in_hardware_sleep_gets_the_toast() {
    let mut recorder = Recorder::default();
    recorder.follow(&on_battery(), 10, Some(100.0), false, Instant::now());

    let out = sleep_through(&mut recorder, 100, 2 * HOUR, 99.0, 98.0, 3_500_000_000);

    assert_eq!(out.notes.len(), 1);
}

fn closed_night() -> (Recorder, Instant) {
    let now = Instant::now();
    let mut recorder = Recorder::default();
    recorder.follow(&plugged_in(), 0, Some(100.0), true, now);
    recorder.lid(false, 1, Some(100.0), true);
    recorder.lid(true, 2, Some(100.0), true);
    recorder.follow(&on_battery(), 10, Some(99.0), true, now);
    (recorder, now)
}

#[test]
fn opening_the_lid_after_the_night_of_november_6_gets_its_toast_naming_keep_awake() {
    let (mut recorder, _) = closed_night();

    let out = recorder.lid(false, 10 + 9 * HOUR + 26 * 60, Some(15.0), true);

    assert_eq!(
        out.notes,
        [Note::AwakeClosed {
            seconds: 9 * HOUR + 26 * 60,
            lost: 84.0,
            keep_awake: true
        }]
    );
    assert_eq!(
        out.notes[0].text(),
        "Awake 9 h with the lid closed: Keep awake was on. Lost 84 %."
    );
}

#[test]
fn without_keep_awake_the_toast_does_not_name_it() {
    let now = Instant::now();
    let mut recorder = Recorder::default();
    recorder.follow(&on_battery(), 0, Some(90.0), false, now);
    recorder.lid(true, 10, Some(90.0), false);

    let out = recorder.lid(false, 10 + 2 * HOUR, Some(70.0), false);

    assert_eq!(out.notes[0].text(), "Awake 2 h with the lid closed. Lost 20 %.");
}

#[test]
fn keep_awake_on_for_part_of_the_stretch_is_named_even_if_it_ended() {
    let now = Instant::now();
    let mut recorder = Recorder::default();
    recorder.follow(&on_battery(), 0, Some(90.0), true, now);
    recorder.lid(true, 10, Some(90.0), true);
    recorder.sample(
        Reading {
            battery: Some(battery(80.0, None, true)),
            keep_awake: false,
            ..Reading::default()
        },
        HOUR,
        now,
    );

    let out = recorder.lid(false, 2 * HOUR, Some(70.0), false);

    assert!(matches!(out.notes[0], Note::AwakeClosed { keep_awake: true, .. }));
}

#[test]
fn an_hour_or_less_with_the_lid_closed_is_silent() {
    let now = Instant::now();
    let mut recorder = Recorder::default();
    recorder.follow(&on_battery(), 0, Some(90.0), false, now);
    recorder.lid(true, 10, Some(90.0), false);

    let out = recorder.lid(false, 10 + LONG_CLOSED_SECONDS, Some(80.0), false);

    assert_eq!(out.notes, []);
}

#[test]
fn plugged_in_with_the_lid_closed_nothing_counts() {
    let now = Instant::now();
    let mut recorder = Recorder::default();
    recorder.follow(&plugged_in(), 0, Some(90.0), false, now);
    recorder.lid(true, 10, Some(90.0), false);

    let out = recorder.lid(false, 10 + 9 * HOUR, Some(90.0), false);

    assert_eq!(out.notes, []);
}

#[test]
fn a_plug_ends_the_stretch_and_the_toast_waits_for_the_lid_to_open() {
    let (mut recorder, now) = closed_night();

    let plugged = recorder.follow(&plugged_in(), 10 + 2 * HOUR, Some(80.0), true, now);
    let opened = recorder.lid(false, 10 + 5 * HOUR, Some(100.0), true);
    let again = recorder.lid(true, 10 + 6 * HOUR, Some(100.0), true);

    assert_eq!(plugged.notes, []);
    assert_eq!(
        opened.notes,
        [Note::AwakeClosed {
            seconds: 2 * HOUR,
            lost: 19.0,
            keep_awake: true
        }]
    );
    assert_eq!(again.notes, []);
}

#[test]
fn a_suspend_ends_the_stretch_and_the_toast_comes_on_resume_before_the_sleeps_own() {
    let (mut recorder, _) = closed_night();

    let out = sleep_through(&mut recorder, 10 + 2 * HOUR, 2 * HOUR, 80.0, 70.0, 0);

    assert_eq!(
        out.notes,
        [
            Note::AwakeClosed {
                seconds: 2 * HOUR,
                lost: 19.0,
                keep_awake: true
            },
            Note::Slept {
                seconds: 2 * HOUR,
                lost: 10.0,
                share: 0.0
            }
        ]
    );
}

#[test]
fn waking_with_the_lid_still_closed_starts_a_new_stretch() {
    let (mut recorder, now) = closed_night();
    recorder.going_to_sleep(edge(100, 99.0, 0));
    recorder.woke(200, now);
    recorder.after_wake(&edge(205, 99.0, 0), now);

    let out = recorder.lid(false, 205 + 2 * HOUR, Some(79.0), true);

    assert_eq!(
        out.notes,
        [Note::AwakeClosed {
            seconds: 2 * HOUR,
            lost: 20.0,
            keep_awake: true
        }]
    );
}

#[test]
fn lengths_read_as_a_person_says_them() {
    assert_eq!(exact(45 * 60), "45 min");
    assert_eq!(exact(2 * HOUR), "2 h");
    assert_eq!(exact(HOUR + 25 * 60), "1 h 25 min");
    assert_eq!(rough(2 * HOUR + 59 * 60), "2 h 59 min");
    assert_eq!(rough(9 * HOUR + 26 * 60), "9 h");
    assert_eq!(rough(9 * HOUR + 31 * 60), "10 h");
}

// The log around the recorder, on a machine of its own.

#[test]
fn a_write_that_fails_disturbs_nothing() {
    let root = crate::fake::FakeRoot::laptop();
    root.write("blocked", "a file where the folder should be");
    let mut log = Log::new(root.machine(), Some(root.path("blocked/history.jsonl")), None);
    let now = Instant::now();

    log.follow(&on_battery(), now);
    let notes = log.tick(now + SAMPLE_EVERY);

    assert_eq!(notes, []);
    assert!(log.write_failed);
}

#[test]
fn the_log_writes_the_start_a_sample_and_a_sleep_to_its_file() {
    let root = crate::fake::FakeRoot::laptop();
    root.write("sys/power/suspend_stats/total_hw_sleep", "0");
    root.write("sys/power/suspend_stats/last_hw_sleep", "0");
    root.write("sys/power/suspend_stats/max_hw_sleep", "18446744073709551615");
    let path = root.path("state/history.jsonl");
    let mut log = Log::new(root.machine(), Some(path.clone()), None);
    let now = Instant::now();

    log.follow(&on_battery(), now);
    log.follow(&on_battery(), now);
    log.tick(now + SAMPLE_EVERY);
    log.going_to_sleep(Edge::read(&root.machine(), history::now()));
    log.woke(now + SAMPLE_EVERY);
    assert_eq!(log.wait(now + SAMPLE_EVERY), Some(WAKE_READING));
    log.tick(now + SAMPLE_EVERY + WAKE_READING);

    let entries = history::read(&path, 0);
    assert!(matches!(
        entries.as_slice(),
        [
            Entry::Event(Event {
                what: What::ServiceStart,
                ..
            }),
            Entry::Sample(_),
            Entry::Sleep(_)
        ]
    ));
}
