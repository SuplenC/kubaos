use jiff::SignedDuration;

use super::*;

fn level(percent: u8) -> Level {
    Level::new(percent).expect("a level")
}

#[test]
fn it_holds_until_its_end_or_forever_while_on() {
    assert!(State::on(0).holds(1_000));
    assert!(State::on(2_000).holds(1_999));
    assert!(!State::on(2_000).holds(2_000));
    assert!(!State::default().holds(0));
    assert!(State::until_battery(level(20)).holds(1_000));
}

#[test]
fn the_state_file_reads_back_and_a_missing_or_broken_one_is_off() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let file = path(dir.path());

    assert_eq!(State::read(&file), State::default());
    State::on(1_793_000_000).write(&file).expect("write");
    assert_eq!(State::read(&file), State::on(1_793_000_000));
    assert_eq!(
        std::fs::read_to_string(&file).expect("read"),
        "{\"active\":true,\"until\":1793000000,\"battery\":0,\"lid\":false}\n"
    );
    std::fs::write(&file, "{half").expect("break it");
    assert_eq!(State::read(&file), State::default());
}

#[test]
fn a_state_file_without_a_battery_level_reads_as_none() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let file = path(dir.path());
    std::fs::write(&file, "{\"active\":true,\"until\":0}\n").expect("an older file");

    let state = State::read(&file);

    assert_eq!(state, State::on(0));
    assert_eq!(state.level(), None);
}

#[test]
fn a_state_file_without_the_lid_flag_doesnt_hold_the_lid() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let file = path(dir.path());
    std::fs::write(&file, "{\"active\":true,\"until\":0,\"battery\":20}\n").expect("an older file");

    assert!(!State::read(&file).holds_lid(1_000));
}

#[test]
fn the_lid_flag_reads_back_from_the_file() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let file = path(dir.path());

    State::on(0).holding_lid().write(&file).expect("write");

    assert!(State::read(&file).holds_lid(1_000));
}

#[test]
fn only_a_keep_awake_asked_to_holds_the_lid_and_only_while_it_holds() {
    assert!(!State::on(0).holds_lid(1_000));
    assert!(!State::until_battery(level(20)).holds_lid(1_000));
    assert!(State::on(0).holding_lid().holds_lid(1_000));
    assert!(State::until_battery(level(20)).holding_lid().holds_lid(1_000));

    assert!(State::on(2_000).holding_lid().holds_lid(1_999));
    assert!(!State::on(2_000).holding_lid().holds_lid(2_000));
    assert!(!State::default().holding_lid().holds_lid(1_000));
}

#[test]
fn holding_the_lid_changes_nothing_about_when_the_battery_ends_it() {
    let forever = State::on(0).holding_lid();
    let to_twenty = State::until_battery(level(20)).holding_lid();

    assert!(!forever.battery_ends(Power::Battery, Some(1)));
    assert!(to_twenty.battery_ends(Power::Battery, Some(20)));
    assert!(!to_twenty.battery_ends(Power::Ac, Some(5)));
}

#[test]
fn a_battery_level_reads_back_from_the_file() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let file = path(dir.path());

    State::until_battery(level(20)).write(&file).expect("write");

    assert_eq!(State::read(&file).level(), Some(level(20)));
}

#[test]
fn plugged_in_the_battery_never_ends_it() {
    let state = State::until_battery(level(20));

    assert!(!state.battery_ends(Power::Ac, Some(5)));
    assert!(!state.battery_ends(Power::Ac, Some(20)));
    assert!(!state.battery_ends(Power::Ac, None));
}

#[test]
fn on_battery_it_ends_at_its_level_or_below_and_stays_above() {
    let state = State::until_battery(level(20));

    assert!(!state.battery_ends(Power::Battery, Some(21)));
    assert!(state.battery_ends(Power::Battery, Some(20)));
    assert!(state.battery_ends(Power::Battery, Some(7)));
    assert!(!state.battery_ends(Power::Battery, None));
}

#[test]
fn a_keep_awake_for_a_while_or_until_turned_off_never_ends_for_the_battery() {
    for state in [State::on(0), State::on(1_793_000_000)] {
        assert!(!state.battery_ends(Power::Battery, Some(1)));
        assert!(!state.battery_ends(Power::Battery, Some(0)));
    }
}

#[test]
fn a_level_is_one_to_ninety_nine_percent() {
    assert_eq!(Level::new(0), None);
    assert_eq!(Level::new(1).map(Level::percent), Some(1));
    assert_eq!(Level::new(99).map(Level::percent), Some(99));
    assert_eq!(Level::new(100), None);
}

#[test]
fn a_typed_keep_awake_is_a_while_or_a_level() {
    assert_eq!(Length::parse("30m"), Ok(Length::For(SignedDuration::from_mins(30))));
    assert_eq!(Length::parse("1h30m"), Ok(Length::For(SignedDuration::from_mins(90))));
    assert_eq!(Length::parse("20%"), Ok(Length::UntilBattery(level(20))));
    assert_eq!(Length::parse(" 25 % "), Ok(Length::UntilBattery(level(25))));
}

#[test]
fn a_level_outside_one_to_ninety_nine_or_nonsense_is_refused() {
    assert_eq!(Length::parse("0%"), Err(LengthError::LevelOutOfRange(0)));
    assert_eq!(Length::parse("100%"), Err(LengthError::LevelOutOfRange(100)));
    assert_eq!(Length::parse("1000%"), Err(LengthError::LevelOutOfRange(1000)));
    assert_eq!(Length::parse("much%"), Err(LengthError::Unreadable("much%".to_owned())));
    assert_eq!(Length::parse("-5%"), Err(LengthError::Unreadable("-5%".to_owned())));
    assert_eq!(Length::parse("soon"), Err(LengthError::Unreadable("soon".to_owned())));
}

fn asked(length: Option<Length>, lid: bool) -> Asked {
    Asked { length, lid }
}

#[test]
fn the_plain_forms_dont_hold_the_lid() {
    let hour = Some(Length::For(SignedDuration::from_hours(1)));
    let twenty = Some(Length::UntilBattery(level(20)));

    assert_eq!(Asked::parse(&["on"]), Ok(asked(None, false)));
    assert_eq!(Asked::parse(&["1h"]), Ok(asked(hour, false)));
    assert_eq!(Asked::parse(&["for", "1h"]), Ok(asked(hour, false)));
    assert_eq!(Asked::parse(&["20%"]), Ok(asked(twenty, false)));
    assert_eq!(Asked::parse(&["until", "20%"]), Ok(asked(twenty, false)));
}

#[test]
fn the_lid_forms_hold_the_lid() {
    let hours = Some(Length::For(SignedDuration::from_hours(2)));
    let twenty = Some(Length::UntilBattery(level(20)));

    assert_eq!(Asked::parse(&["lid", "on"]), Ok(asked(None, true)));
    assert_eq!(Asked::parse(&["lid", "2h"]), Ok(asked(hours, true)));
    assert_eq!(Asked::parse(&["lid", "for", "2h"]), Ok(asked(hours, true)));
    assert_eq!(Asked::parse(&["lid", "20%"]), Ok(asked(twenty, true)));
    assert_eq!(Asked::parse(&["lid", "until", "20%"]), Ok(asked(twenty, true)));
}

#[test]
fn a_form_with_no_length_or_a_bad_one_is_refused() {
    let unreadable = |typed: &str| Err(LengthError::Unreadable(typed.to_owned()));

    assert_eq!(Asked::parse(&["lid"]), unreadable("lid"));
    assert_eq!(Asked::parse(&["for"]), unreadable("for"));
    assert_eq!(Asked::parse(&["lid", "until"]), unreadable("lid until"));
    assert_eq!(Asked::parse(&["lid", "soon"]), unreadable("soon"));
    assert_eq!(Asked::parse(&["for", "on"]), unreadable("on"));
    assert_eq!(Asked::parse(&["lid", "100%"]), Err(LengthError::LevelOutOfRange(100)));
}
