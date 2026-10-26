use jiff::Zoned;
use kubaos_clock::settings::TimeFormat;

use super::*;
use crate::settings::OnPower;

fn mins(minutes: u32) -> Wait {
    Wait::from_secs(minutes * 60)
}

fn with(timer_: Timer, battery: Wait, ac: Wait) -> Settings {
    let mut settings = Settings::default();
    let set = |on: &mut OnPower, wait| match timer_ {
        Timer::Lock => on.lock = wait,
        Timer::ScreensOff => on.screens_off = wait,
        Timer::Suspend => on.suspend = wait,
    };
    set(&mut settings.battery, battery);
    set(&mut settings.ac, ac);
    settings
}

#[test]
fn a_wait_reads_as_a_person_says_it() {
    assert_eq!(wait(Wait::from_secs(30)), "30 s");
    assert_eq!(wait(mins(2)), "2 min");
    assert_eq!(wait(Wait::from_secs(90)), "1 min 30 s");
    assert_eq!(wait(mins(90)), "1 h 30 min");
    assert_eq!(wait(mins(300)), "5 h");
    assert_eq!(wait(Wait::NEVER), "never");
}

#[test]
fn the_same_wait_on_both_sources_is_said_once() {
    let settings = with(Timer::Lock, mins(2), mins(2));

    assert_eq!(
        timer(&settings, Timer::Lock),
        "Lock after 2 min idle, on battery and plugged in"
    );
}

#[test]
fn two_waits_are_said_battery_first() {
    let settings = with(Timer::Lock, mins(15), mins(30));

    assert_eq!(
        timer(&settings, Timer::Lock),
        "Lock after 15 min idle on battery, 30 min plugged in"
    );
}

#[test]
fn never_on_one_source_comes_second() {
    assert_eq!(
        timer(&Settings::default(), Timer::Suspend),
        "Suspend after 30 min idle on battery, never plugged in"
    );
    let settings = with(Timer::ScreensOff, Wait::NEVER, mins(5));
    assert_eq!(
        timer(&settings, Timer::ScreensOff),
        "Screens off after 5 min idle plugged in, never on battery"
    );
}

#[test]
fn never_on_both_says_so() {
    for (t, said) in [
        (Timer::Lock, "Never lock by itself"),
        (Timer::ScreensOff, "Screens never go off by themselves"),
        (Timer::Suspend, "Never suspend by itself"),
    ] {
        assert_eq!(timer(&with(t, Wait::NEVER, Wait::NEVER), t), said);
    }
}

fn lid_with(battery: LidAction, ac: LidAction) -> String {
    let mut settings = Settings::default();
    settings.battery.lid = battery;
    settings.ac.lid = ac;
    lid(&settings)
}

#[test]
fn the_lid_says_what_it_does_where_docked_always_nothing() {
    assert_eq!(
        lid(&Settings::default()),
        "Closing the lid suspends on battery, does nothing plugged in or docked"
    );
    assert_eq!(
        lid_with(LidAction::Suspend, LidAction::Lock),
        "Closing the lid suspends on battery, locks plugged in, does nothing docked"
    );
    assert_eq!(
        lid_with(LidAction::Lock, LidAction::Lock),
        "Closing the lid locks on battery and plugged in, does nothing docked"
    );
    assert_eq!(
        lid_with(LidAction::Nothing, LidAction::Suspend),
        "Closing the lid suspends plugged in, does nothing on battery or docked"
    );
    assert_eq!(
        lid_with(LidAction::Nothing, LidAction::Nothing),
        "Closing the lid does nothing"
    );
}

#[test]
fn a_time_of_day_follows_the_clocks_format_and_names_another_day() {
    let now: Zoned = "2026-10-26T10:00:00[Europe/Warsaw]".parse().expect("now");
    let at = |text: &str| -> Zoned { text.parse().expect("at") };

    assert_eq!(
        time_of_day(&at("2026-10-26T17:05:00[Europe/Warsaw]"), &now, None),
        "17:05"
    );
    assert_eq!(
        time_of_day(
            &at("2026-10-26T17:05:00[Europe/Warsaw]"),
            &now,
            Some(TimeFormat::TwelveHour)
        ),
        "5:05 PM"
    );
    assert_eq!(
        time_of_day(&at("2026-10-27T08:00:00[Europe/Warsaw]"), &now, None),
        "tomorrow at 08:00"
    );
    assert_eq!(
        time_of_day(&at("2026-10-30T17:00:00[Europe/Warsaw]"), &now, None),
        "Friday at 17:00"
    );
    assert_eq!(
        time_of_day(&at("2026-11-09T09:00:00[Europe/Warsaw]"), &now, None),
        "9 November at 09:00"
    );
}

#[test]
fn how_long_until_is_said_roughly() {
    assert_eq!(left(30), "30 s");
    assert_eq!(left(600), "10 min");
    assert_eq!(left(3599), "1 h");
    assert_eq!(left(2 * 3600 + 5 * 60 + 12), "2 h 5 min");
    assert_eq!(left(89 * 3600 + 42 * 60 + 18), "4 days");
    assert_eq!(left(-5), "0 s");
}

#[test]
fn when_says_at_for_today_and_the_day_otherwise() {
    let now: Zoned = "2026-10-26T10:00:00[Europe/Warsaw]".parse().expect("now");
    let today: Zoned = "2026-10-26T17:00:00[Europe/Warsaw]".parse().expect("today");
    let friday: Zoned = "2026-10-30T17:00:00[Europe/Warsaw]".parse().expect("friday");

    assert_eq!(when(&today, &now, None), "at 17:00");
    assert_eq!(when(&friday, &now, None), "Friday at 17:00");
}
