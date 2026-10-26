use jiff::Zoned;

use super::*;

fn zoned(text: &str) -> Zoned {
    text.parse().expect("a zoned time")
}

fn hm(at: &Zoned) -> String {
    at.strftime("%Y-%m-%d %H:%M:%S").to_string()
}

fn units(plan: &[Transient]) -> Vec<&str> {
    plan.iter().map(|timer| timer.unit).collect()
}

#[test]
fn in_reads_durations_as_the_clocks_timer() {
    let now = zoned("2026-10-26T10:00:00[Europe/Warsaw]");

    assert_eq!(hm(&after("10m", &now).expect("10m")), "2026-10-26 10:10:00");
    assert_eq!(hm(&after("1h30m", &now).expect("1h30m")), "2026-10-26 11:30:00");
    assert_eq!(hm(&after("90s", &now).expect("90s")), "2026-10-26 10:01:30");
    assert_eq!(hm(&after("25", &now).expect("25")), "2026-10-26 10:25:00");
    assert_eq!(after("0", &now), None);
    assert_eq!(after("soon", &now), None);
}

#[test]
fn at_is_the_next_time_it_comes_round_across_midnight() {
    let evening = zoned("2026-10-26T21:00:00[Europe/Warsaw]");

    assert_eq!(hm(&at_time("20:00", &evening).expect("20:00")), "2026-10-27 20:00:00");
    assert_eq!(hm(&at_time("22:30", &evening).expect("22:30")), "2026-10-26 22:30:00");
    assert_eq!(hm(&at_time("1am", &evening).expect("1am")), "2026-10-27 01:00:00");
}

#[test]
fn at_a_weekday_is_its_next_one() {
    let monday = zoned("2026-10-26T10:00:00[Europe/Warsaw]");
    let friday_evening = zoned("2026-10-30T18:00:00[Europe/Warsaw]");

    assert_eq!(
        hm(&at_time("friday 17:00", &monday).expect("friday")),
        "2026-10-30 17:00:00"
    );
    assert_eq!(
        hm(&at_time("tomorrow 8am", &monday).expect("tomorrow")),
        "2026-10-27 08:00:00"
    );
    assert_eq!(
        hm(&at_time("next friday 17:00", &friday_evening).expect("next friday")),
        "2026-11-06 17:00:00"
    );
}

#[test]
fn at_refuses_the_past_and_nonsense() {
    let now = zoned("2026-10-26T10:00:00[Europe/Warsaw]");

    assert_eq!(at_time("yesterday 9am", &now), None);
    assert_eq!(at_time("whenever", &now), None);
    assert_eq!(at_time("", &now), None);
}

#[test]
fn each_action_runs_through_the_sessions_own_path() {
    assert_eq!(Action::Lock.command(), ["loginctl", "lock-session"]);
    assert_eq!(Action::Suspend.command(), ["systemctl", "suspend"]);
    assert_eq!(Action::Shutdown.command(), ["dot-session", "leave", "poweroff"]);
    assert_eq!(Action::Reboot.command(), ["dot-session", "leave", "reboot"]);
}

#[test]
fn actions_are_named_and_found_by_name() {
    for action in Action::ALL {
        assert_eq!(Action::from_name(action.name()), Some(action));
    }
    assert_eq!(Action::from_name("poweroff"), Some(Action::Shutdown));
    assert_eq!(Action::from_name("restart"), Some(Action::Reboot));
    assert_eq!(Action::from_name("hibernate"), None);
    assert_eq!(
        (
            Action::Shutdown.does(),
            Action::Shutdown.doing(),
            Action::Shutdown.noun()
        ),
        ("Shuts down", "Shutting down", "shut down")
    );
}

#[test]
fn an_action_far_enough_away_gets_both_heads_ups_then_itself() {
    let now = zoned("2026-10-26T10:00:00[Europe/Warsaw]");
    let at = zoned("2026-10-26T10:10:00[Europe/Warsaw]");
    let secs = at.timestamp().as_second().to_string();

    let plan = plan(Action::Shutdown, &at, &now, "/bin/kubaos-idle");

    assert_eq!(units(&plan), [WARN_MINUTE_UNIT, WARN_SECONDS_UNIT, ACTION_UNIT]);
    assert_eq!(hm(&plan[0].at), "2026-10-26 10:09:00");
    assert_eq!(hm(&plan[1].at), "2026-10-26 10:09:50");
    assert_eq!(
        plan[0].argv,
        ["/bin/kubaos-idle", "schedule", "warn", "60", "shutdown", &secs]
    );
    assert_eq!(
        plan[1].argv,
        ["/bin/kubaos-idle", "schedule", "warn", "10", "shutdown", &secs]
    );
    assert_eq!(
        plan[2].argv,
        ["/bin/kubaos-idle", "schedule", "fire", "shutdown", &secs]
    );
}

#[test]
fn every_action_uses_the_same_fixed_units_so_one_replaces_another() {
    let now = zoned("2026-10-26T10:00:00[Europe/Warsaw]");
    let at = zoned("2026-10-26T12:00:00[Europe/Warsaw]");

    let plans: Vec<Vec<&str>> = Action::ALL
        .iter()
        .map(|&action| plan(action, &at, &now, "k").iter().map(|timer| timer.unit).collect())
        .collect();

    assert!(plans.iter().all(|units| *units == plans[0]), "{plans:?}");
    for units in &plans {
        for unit in units {
            assert!(timers().contains(&format!("{unit}.timer")), "{unit} isn't cancelled");
        }
    }
}

#[test]
fn an_action_under_a_minute_away_skips_the_minutes_heads_up() {
    let now = zoned("2026-10-26T10:00:00[Europe/Warsaw]");

    let soon = plan(Action::Lock, &zoned("2026-10-26T10:00:40[Europe/Warsaw]"), &now, "k");
    let sooner = plan(Action::Reboot, &zoned("2026-10-26T10:00:12[Europe/Warsaw]"), &now, "k");
    let one_minute = plan(Action::Suspend, &zoned("2026-10-26T10:01:00[Europe/Warsaw]"), &now, "k");

    assert_eq!(units(&soon), [WARN_SECONDS_UNIT, ACTION_UNIT]);
    assert_eq!(units(&sooner), [ACTION_UNIT]);
    assert_eq!(units(&one_minute), [WARN_SECONDS_UNIT, ACTION_UNIT]);
}

#[test]
fn each_timer_is_a_fixed_unit_on_the_calendar_to_the_second() {
    let at = zoned("2026-10-30T17:00:00[Europe/Warsaw]").with_time_zone(jiff::tz::TimeZone::system());
    let timer = Transient {
        unit: ACTION_UNIT,
        at: at.clone(),
        argv: vec!["k".into(), "schedule".into(), "fire".into()],
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
    assert!(argv.contains(&"--unit=kubaos-scheduled-action".to_owned()), "{argv:?}");
    assert!(
        argv.contains(&format!("--on-calendar={}", at.strftime("%Y-%m-%d %H:%M:%S"))),
        "{argv:?}"
    );
    assert_eq!(argv[argv.len() - 4..], ["--", "k", "schedule", "fire"]);
}

#[test]
fn cancelling_stops_every_timer_an_action_may_have() {
    assert_eq!(
        timers(),
        [
            "kubaos-scheduled-action.timer",
            "kubaos-scheduled-warn-1m.timer",
            "kubaos-scheduled-warn-10s.timer"
        ]
    );
}

#[test]
fn a_heads_up_or_an_action_left_from_a_replaced_schedule_isnt_the_current_one() {
    let current = State::new(Action::Reboot, 2_000);

    assert!(current.is(Action::Reboot, 2_000));
    assert!(!current.is(Action::Lock, 2_000));
    assert!(!current.is(Action::Reboot, 1_500));
    assert!(!State::default().is(Action::Lock, 0));
}

#[test]
fn the_state_is_pending_until_its_time_and_reads_back_with_its_action() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let file = path(dir.path());

    assert!(!State::read(&file).pending(0));
    State::new(Action::Shutdown, 2_000).write(&file).expect("write");

    assert_eq!(State::read(&file), State::new(Action::Shutdown, 2_000));
    assert!(State::read(&file).pending(1_999));
    assert!(!State::read(&file).pending(2_000));
    assert_eq!(
        std::fs::read_to_string(&file).expect("read"),
        "{\"at\":2000,\"action\":\"shutdown\"}\n"
    );
}

#[test]
fn a_state_without_an_action_is_a_lock() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let file = path(dir.path());
    std::fs::write(&file, "{\"at\":2000}").expect("write");

    assert_eq!(State::read(&file), State::new(Action::Lock, 2_000));
}
