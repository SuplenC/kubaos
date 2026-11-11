use std::time::Duration;

use super::*;
use crate::awake;

fn secs(secs: u64) -> Duration {
    Duration::from_secs(secs)
}

#[test]
fn docked_the_lid_does_nothing_whatever_the_setting() {
    let mut settings = Settings::default();
    settings.ac.lid = LidAction::Suspend;

    assert_eq!(action(&settings, true, Power::Battery, false), LidAction::Nothing);
    assert_eq!(action(&settings, true, Power::Ac, false), LidAction::Nothing);
    assert_eq!(action(&settings, false, Power::Ac, false), LidAction::Suspend);
}

#[test]
fn undocked_the_lid_does_the_power_sources_setting() {
    let settings = Settings::default();

    assert_eq!(action(&settings, false, Power::Battery, false), LidAction::Suspend);
    assert_eq!(action(&settings, false, Power::Ac, false), LidAction::Nothing);
}

#[test]
fn logind_keeps_the_lid_while_the_settings_are_its_own() {
    let mut settings = Settings::default();
    assert!(!needs_holder(&settings, false));

    settings.battery.lid = LidAction::Lock;
    assert!(needs_holder(&settings, false));

    settings.battery.lid = LidAction::Suspend;
    settings.ac.lid = LidAction::Suspend;
    assert!(needs_holder(&settings, false));
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
        assert!(!docked.tick(secs(now), true, action(&settings, true, Power::Battery, false)));
    }
}

#[test]
fn a_held_lid_does_nothing_on_battery_plugged_in_and_docked() {
    let mut settings = Settings::default();
    settings.ac.lid = LidAction::Lock;

    for power in [Power::Battery, Power::Ac] {
        for docked in [false, true] {
            assert_eq!(action(&settings, docked, power, true), LidAction::Nothing);
        }
    }
    assert_eq!(action(&settings, false, Power::Battery, false), LidAction::Suspend);
    assert_eq!(action(&settings, false, Power::Ac, false), LidAction::Lock);
}

#[test]
fn a_lid_holding_keep_awake_needs_the_holder_even_with_loginds_own_settings() {
    let settings = Settings::default();

    assert!(needs_holder(&settings, true));
    assert!(!needs_holder(&settings, false));
}

#[test]
fn the_holder_ends_only_with_the_lid_open_and_nothing_needing_it() {
    let mut settings = Settings::default();

    assert!(holder_ends(&settings, false, false));
    assert!(!holder_ends(&settings, false, true));
    assert!(!holder_ends(&settings, true, false));
    assert!(!holder_ends(&settings, true, true));
    settings.ac.lid = LidAction::Lock;
    assert!(!holder_ends(&settings, false, false));
}

#[test]
fn a_lid_closed_while_held_suspends_once_the_keep_awake_ends() {
    let settings = Settings::default();
    let mut recheck = Recheck::default();

    for now in [0, 30, 600] {
        let kept = action(&settings, false, Power::Battery, true);
        assert!(!recheck.tick(secs(now), true, kept));
    }
    assert!(!holder_ends(&settings, false, true));
    assert!(recheck.tick(secs(605), true, action(&settings, false, Power::Battery, false)));
}

#[test]
fn a_lid_closed_just_before_the_keep_awake_ends_waits_the_holdoff() {
    let settings = Settings::default();
    let mut recheck = Recheck::default();
    let over = action(&settings, false, Power::Battery, false);

    assert!(!recheck.tick(secs(100), true, action(&settings, false, Power::Battery, true)));
    assert!(!recheck.tick(secs(105), true, over));
    assert!(!recheck.tick(secs(125), true, over));
    assert!(recheck.tick(secs(130), true, over));
}

/// Whether the lid is held, as the state file of each form of Keep awake says.
fn lid_held(state: awake::State) -> bool {
    state.holds_lid(1_000)
}

#[test]
fn a_plain_keep_awake_leaves_the_lid_to_the_settings_undocked() {
    let settings = Settings::default();
    let plain = lid_held(awake::State::on(0));

    assert_eq!(action(&settings, false, Power::Battery, plain), LidAction::Suspend);
    assert_eq!(action(&settings, false, Power::Ac, plain), LidAction::Nothing);
    assert_eq!(action(&settings, true, Power::Battery, plain), LidAction::Nothing);
    assert_eq!(action(&settings, true, Power::Ac, plain), LidAction::Nothing);
}

#[test]
fn a_lid_holding_keep_awake_does_nothing_whatever_the_power_or_the_dock() {
    let settings = Settings::default();
    let held = lid_held(awake::State::on(0).holding_lid());

    for power in [Power::Battery, Power::Ac] {
        for docked in [false, true] {
            assert_eq!(action(&settings, docked, power, held), LidAction::Nothing);
        }
    }
}

#[test]
fn a_plain_keep_awake_starts_no_holder_and_lets_a_running_one_end() {
    let settings = Settings::default();
    let plain = lid_held(awake::State::on(0));

    assert!(!needs_holder(&settings, plain));
    assert!(holder_ends(&settings, plain, false));
}

#[test]
fn a_lid_holding_keep_awake_needs_the_holder_and_keeps_it() {
    let settings = Settings::default();
    let held = lid_held(awake::State::on(0).holding_lid());

    assert!(needs_holder(&settings, held));
    assert!(!holder_ends(&settings, held, false));
    assert!(!holder_ends(&settings, held, true));
}

#[test]
fn a_holder_the_settings_need_keeps_working_under_a_plain_keep_awake() {
    let mut settings = Settings::default();
    settings.ac.lid = LidAction::Suspend;
    let plain = lid_held(awake::State::on(0));
    let mut recheck = Recheck::default();

    assert!(needs_holder(&settings, plain));
    assert!(!holder_ends(&settings, plain, false));
    assert_eq!(action(&settings, false, Power::Ac, plain), LidAction::Suspend);
    assert!(!recheck.tick(secs(0), true, action(&settings, false, Power::Ac, plain)));
    assert!(recheck.tick(secs(30), true, action(&settings, false, Power::Ac, plain)));
}

#[test]
fn a_lid_closed_under_a_keep_awake_turned_plain_suspends_after_the_holdoff() {
    let settings = Settings::default();
    let held = lid_held(awake::State::on(0).holding_lid());
    let plain = lid_held(awake::State::on(0));
    let mut recheck = Recheck::default();

    assert!(!recheck.tick(secs(0), true, action(&settings, false, Power::Battery, held)));
    assert!(!holder_ends(&settings, plain, true));
    assert!(recheck.tick(secs(30), true, action(&settings, false, Power::Battery, plain)));
}
