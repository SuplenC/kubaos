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
        "{\"active\":true,\"until\":1793000000,\"battery\":0}\n"
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
