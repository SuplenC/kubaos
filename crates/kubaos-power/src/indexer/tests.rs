use super::*;

#[test]
fn a_quiet_stop_holds_a_running_indexer() {
    assert_eq!(hold(true, true, false), Some(Hold::Start));
    assert_eq!(hold(true, true, true), None);
}

#[test]
fn a_quiet_stop_waits_for_an_indexer_that_isnt_running() {
    assert_eq!(hold(true, false, false), None);
}

#[test]
fn a_hold_on_an_indexer_that_went_stops_so_the_next_one_gets_its_own() {
    assert_eq!(hold(true, false, true), Some(Hold::Stop));
}

#[test]
fn a_normal_stop_lets_the_indexer_go() {
    assert_eq!(hold(false, true, true), Some(Hold::Stop));
    assert_eq!(hold(false, false, true), Some(Hold::Stop));
    assert_eq!(hold(false, true, false), None);
    assert_eq!(hold(false, false, false), None);
}

#[test]
fn only_the_indexer_doing_the_wrong_thing_disagrees_with_a_stop() {
    assert!(State::Paused.agrees(true));
    assert!(!State::Running.agrees(true));
    assert!(State::NotRunning.agrees(true));

    assert!(State::Running.agrees(false));
    assert!(!State::Paused.agrees(false));
    assert!(State::NotRunning.agrees(false));
}
