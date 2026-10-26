use std::time::Duration;

use super::*;

fn secs(secs: u64) -> Duration {
    Duration::from_secs(secs)
}

#[test]
fn docked_the_lid_does_nothing_whatever_the_setting() {
    let mut settings = Settings::default();
    settings.ac.lid = LidAction::Suspend;

    assert_eq!(action(&settings, true, Power::Battery), LidAction::Nothing);
    assert_eq!(action(&settings, true, Power::Ac), LidAction::Nothing);
    assert_eq!(action(&settings, false, Power::Ac), LidAction::Suspend);
}

#[test]
fn undocked_the_lid_does_the_power_sources_setting() {
    let settings = Settings::default();

    assert_eq!(action(&settings, false, Power::Battery), LidAction::Suspend);
    assert_eq!(action(&settings, false, Power::Ac), LidAction::Nothing);
}

#[test]
fn logind_keeps_the_lid_while_the_settings_are_its_own() {
    let mut settings = Settings::default();
    assert!(!needs_holder(&settings));

    settings.battery.lid = LidAction::Lock;
    assert!(needs_holder(&settings));

    settings.battery.lid = LidAction::Suspend;
    settings.ac.lid = LidAction::Suspend;
    assert!(needs_holder(&settings));
}

#[test]
fn a_lid_closed_plugged_in_then_unplugged_suspends_after_the_holdoff() {
    let mut recheck = Recheck::default();

    assert!(!recheck.tick(secs(0), true, LidAction::Nothing));
    assert!(!recheck.tick(secs(600), true, LidAction::Nothing));
    assert!(recheck.tick(secs(605), true, LidAction::Suspend));
}

#[test]
fn a_lid_just_closed_waits_the_holdoff_before_a_recheck_suspends() {
    let mut recheck = Recheck::default();

    assert!(!recheck.tick(secs(100), true, LidAction::Suspend));
    assert!(!recheck.tick(secs(125), true, LidAction::Suspend));
    assert!(recheck.tick(secs(130), true, LidAction::Suspend));
}

#[test]
fn after_a_suspend_it_waits_the_holdoff_again_and_an_open_lid_starts_over() {
    let mut recheck = Recheck::default();
    recheck.tick(secs(0), true, LidAction::Suspend);
    assert!(recheck.tick(secs(30), true, LidAction::Suspend));

    assert!(!recheck.tick(secs(35), true, LidAction::Suspend));
    assert!(recheck.tick(secs(60), true, LidAction::Suspend));

    assert!(!recheck.tick(secs(65), false, LidAction::Suspend));
    assert!(!recheck.tick(secs(70), true, LidAction::Suspend));
}

#[test]
fn a_recheck_never_locks_and_docked_never_suspends() {
    let mut recheck = Recheck::default();

    for now in [0, 30, 60, 90] {
        assert!(!recheck.tick(secs(now), true, LidAction::Lock));
    }
    let settings = Settings::default();
    let mut docked = Recheck::default();
    for now in [0, 30, 60, 90] {
        assert!(!docked.tick(secs(now), true, action(&settings, true, Power::Battery)));
    }
}
