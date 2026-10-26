use jiff::Zoned;

use super::*;

fn zoned(text: &str) -> Zoned {
    text.parse().expect("a zoned time")
}

fn hm(at: &Zoned) -> String {
    at.strftime("%Y-%m-%d %H:%M:%S").to_string()
}

#[test]
fn lock_in_reads_durations_as_the_clocks_timer() {
    let now = zoned("2026-10-26T10:00:00[Europe/Warsaw]");

    assert_eq!(hm(&lock_in("10m", &now).expect("10m")), "2026-10-26 10:10:00");
    assert_eq!(hm(&lock_in("1h30m", &now).expect("1h30m")), "2026-10-26 11:30:00");
    assert_eq!(hm(&lock_in("90s", &now).expect("90s")), "2026-10-26 10:01:30");
    assert_eq!(hm(&lock_in("25", &now).expect("25")), "2026-10-26 10:25:00");
    assert_eq!(lock_in("0", &now), None);
    assert_eq!(lock_in("soon", &now), None);
}

#[test]
fn lock_at_is_the_next_time_it_comes_round_across_midnight() {
    let evening = zoned("2026-10-26T21:00:00[Europe/Warsaw]");

    assert_eq!(hm(&lock_at("20:00", &evening).expect("20:00")), "2026-10-27 20:00:00");
    assert_eq!(hm(&lock_at("22:30", &evening).expect("22:30")), "2026-10-26 22:30:00");
    assert_eq!(hm(&lock_at("1am", &evening).expect("1am")), "2026-10-27 01:00:00");
}

#[test]
fn lock_at_a_weekday_is_its_next_one() {
    let monday = zoned("2026-10-26T10:00:00[Europe/Warsaw]");
    let friday_evening = zoned("2026-10-30T18:00:00[Europe/Warsaw]");

    assert_eq!(
        hm(&lock_at("friday 17:00", &monday).expect("friday")),
        "2026-10-30 17:00:00"
    );
    assert_eq!(
        hm(&lock_at("tomorrow 8am", &monday).expect("tomorrow")),
        "2026-10-27 08:00:00"
    );
    assert_eq!(
        hm(&lock_at("next friday 17:00", &friday_evening).expect("next friday")),
        "2026-11-06 17:00:00"
    );
}

#[test]
fn lock_at_refuses_the_past_and_nonsense() {
    let now = zoned("2026-10-26T10:00:00[Europe/Warsaw]");

    assert_eq!(lock_at("yesterday 9am", &now), None);
    assert_eq!(lock_at("whenever", &now), None);
    assert_eq!(lock_at("", &now), None);
}

#[test]
fn a_lock_far_enough_away_gets_both_heads_ups_then_the_lock() {
    let now = zoned("2026-10-26T10:00:00[Europe/Warsaw]");
    let at = zoned("2026-10-26T10:10:00[Europe/Warsaw]");

    let plan = plan(&at, &now, "/bin/kubaos-idle");

    let units: Vec<&str> = plan.iter().map(|timer| timer.unit).collect();
    assert_eq!(units, [WARN_MINUTE_UNIT, WARN_SECONDS_UNIT, LOCK_UNIT]);
    assert_eq!(hm(&plan[0].at), "2026-10-26 10:09:00");
    assert_eq!(hm(&plan[1].at), "2026-10-26 10:09:50");
    assert_eq!(plan[0].argv, ["/bin/kubaos-idle", "lock", "warn", "60"]);
    assert_eq!(plan[1].argv, ["/bin/kubaos-idle", "lock", "warn", "10"]);
    assert_eq!(plan[2].argv, ["/bin/kubaos-idle", "lock", "now"]);
}

#[test]
fn a_lock_in_under_a_minute_skips_the_minutes_heads_up() {
    let now = zoned("2026-10-26T10:00:00[Europe/Warsaw]");

    let soon = plan(&zoned("2026-10-26T10:00:40[Europe/Warsaw]"), &now, "k");
    let sooner = plan(&zoned("2026-10-26T10:00:12[Europe/Warsaw]"), &now, "k");
    let one_minute = plan(&zoned("2026-10-26T10:01:00[Europe/Warsaw]"), &now, "k");

    assert_eq!(
        soon.iter().map(|t| t.unit).collect::<Vec<_>>(),
        [WARN_SECONDS_UNIT, LOCK_UNIT]
    );
    assert_eq!(sooner.iter().map(|t| t.unit).collect::<Vec<_>>(), [LOCK_UNIT]);
    assert_eq!(
        one_minute.iter().map(|t| t.unit).collect::<Vec<_>>(),
        [WARN_SECONDS_UNIT, LOCK_UNIT]
    );
}

#[test]
fn each_timer_is_a_fixed_unit_on_the_calendar_to_the_second() {
    let at = zoned("2026-10-30T17:00:00[Europe/Warsaw]").with_time_zone(jiff::tz::TimeZone::system());
    let timer = Transient {
        unit: LOCK_UNIT,
        at: at.clone(),
        argv: vec!["k".into(), "lock".into(), "now".into()],
    };

    let argv = timer.systemd_run();

    assert_eq!(
        argv[..6],
        [
            "systemd-run",
            "--user",
            "--quiet",
            "--collect",
            "--timer-property=AccuracySec=1s",
            "--timer-property=RemainAfterElapse=no"
        ]
    );
    assert!(argv.contains(&"--unit=kubaos-scheduled-lock".to_owned()), "{argv:?}");
    assert!(
        argv.contains(&format!("--on-calendar={}", at.strftime("%Y-%m-%d %H:%M:%S"))),
        "{argv:?}"
    );
    assert_eq!(argv[argv.len() - 4..], ["--", "k", "lock", "now"]);
}

#[test]
fn cancelling_stops_every_timer_a_lock_may_have() {
    assert_eq!(
        timers(),
        [
            "kubaos-scheduled-lock.timer",
            "kubaos-scheduled-lock-warn-1m.timer",
            "kubaos-scheduled-lock-warn-10s.timer"
        ]
    );
}

#[test]
fn the_state_is_pending_until_its_time_and_reads_back() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let file = path(dir.path());

    assert!(!State::read(&file).pending(0));
    State { at: 2_000 }.write(&file).expect("write");

    assert!(State::read(&file).pending(1_999));
    assert!(!State::read(&file).pending(2_000));
}
