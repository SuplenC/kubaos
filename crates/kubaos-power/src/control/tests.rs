use super::*;

const DROP: Drop = Drop {
    percent: 20,
    to: Some(Stop::PowerSaver),
};

fn applied(effects: &[Effect]) -> Vec<Stop> {
    effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::Apply(stop) => Some(*stop),
            _ => None,
        })
        .collect()
}

fn asked(effects: &[Effect]) -> Vec<Profile> {
    effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::SetProfile(profile) => Some(*profile),
            _ => None,
        })
        .collect()
}

fn toasts(effects: &[Effect]) -> Vec<Toast> {
    effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::Toast(toast) => Some(*toast),
            _ => None,
        })
        .collect()
}

fn remembered(effects: &[Effect]) -> Option<Memory> {
    effects.iter().rev().find_map(|effect| match effect {
        Effect::Remember(memory) => Some(*memory),
        _ => None,
    })
}

/// A control on `source` that has been through `start`, the profile already the stop's.
fn on(source: Source, memory: Memory) -> Control {
    let profile = memory.stop_for(source).profile();
    Control::start(memory, source, Some(profile), DROP).0
}

#[test]
fn a_first_start_takes_power_saver_on_battery_and_balanced_plugged_in() {
    let (battery, effects) = Control::start(Memory::default(), Source::Battery, Some(Profile::Balanced), DROP);

    assert_eq!(battery.stop(), Stop::PowerSaver);
    assert_eq!(asked(&effects), [Profile::PowerSaver]);
    assert_eq!(applied(&effects), [Stop::PowerSaver]);

    let (ac, effects) = Control::start(Memory::default(), Source::Ac, Some(Profile::Balanced), DROP);

    assert_eq!(ac.stop(), Stop::Balanced);
    assert_eq!(asked(&effects), []);
    assert_eq!(applied(&effects), [Stop::Balanced]);
}

#[test]
fn unplugging_moves_to_the_stop_last_used_on_battery_with_a_quiet_toast() {
    let memory = Memory {
        battery: Stop::SuperSaver,
        ..Memory::default()
    };
    let mut control = on(Source::Ac, memory);

    let effects = control.handle(Input::Source(Source::Battery));

    assert_eq!(control.stop(), Stop::SuperSaver);
    assert_eq!(asked(&effects), [Profile::PowerSaver]);
    assert_eq!(applied(&effects), [Stop::SuperSaver]);
    assert_eq!(
        toasts(&effects),
        [Toast::Followed {
            source: Source::Battery,
            stop: Stop::SuperSaver
        }]
    );
}

#[test]
fn plugging_in_moves_to_the_stop_last_used_plugged_in() {
    let memory = Memory {
        ac: Stop::MaxPerformance,
        ..Memory::default()
    };
    let mut control = on(Source::Battery, memory);

    let effects = control.handle(Input::Source(Source::Ac));

    assert_eq!(control.stop(), Stop::MaxPerformance);
    assert_eq!(asked(&effects), [Profile::Performance]);
    assert_eq!(applied(&effects), [Stop::MaxPerformance]);
}

#[test]
fn a_source_whose_stop_is_the_same_moves_nothing_and_says_nothing() {
    let memory = Memory {
        battery: Stop::Balanced,
        ac: Stop::Balanced,
        ..Memory::default()
    };
    let mut control = on(Source::Ac, memory);

    let effects = control.handle(Input::Source(Source::Battery));

    assert_eq!(applied(&effects), []);
    assert_eq!(toasts(&effects), []);
}

#[test]
fn a_choice_on_each_source_is_remembered_for_that_source_alone() {
    let mut control = on(Source::Ac, Memory::default());

    control.handle(Input::Set(Stop::Performance));
    control.handle(Input::Source(Source::Battery));
    let effects = control.handle(Input::Set(Stop::SuperSaver));
    let memory = remembered(&effects).expect("the choice is written down");

    assert_eq!((memory.battery, memory.ac), (Stop::SuperSaver, Stop::Performance));
    control.handle(Input::Source(Source::Ac));
    assert_eq!(control.stop(), Stop::Performance);
    control.handle(Input::Source(Source::Battery));
    assert_eq!(control.stop(), Stop::SuperSaver);
}

#[test]
fn moving_between_stops_on_one_profile_asks_nothing_of_power_profiles_daemon() {
    let mut control = on(Source::Battery, Memory::default());

    let effects = control.handle(Input::Set(Stop::SuperSaver));

    assert_eq!(asked(&effects), []);
    assert_eq!(applied(&effects), [Stop::SuperSaver]);
}

#[test]
fn our_own_profile_change_coming_back_is_not_taken_for_an_outside_one() {
    let mut control = on(Source::Ac, Memory::default());

    control.handle(Input::Set(Stop::MaxPerformance));
    let effects = control.handle(Input::Profile {
        profile: Profile::Performance,
        held: false,
    });

    assert_eq!(control.stop(), Stop::MaxPerformance);
    assert_eq!(effects, []);
}

#[test]
fn quick_steps_through_profiles_come_back_in_order_and_change_nothing() {
    let mut control = on(Source::Ac, Memory::default());

    control.handle(Input::Set(Stop::Performance));
    control.handle(Input::Set(Stop::PowerSaver));
    let first = control.handle(Input::Profile {
        profile: Profile::Performance,
        held: false,
    });
    let second = control.handle(Input::Profile {
        profile: Profile::PowerSaver,
        held: false,
    });

    assert_eq!((first, second), (vec![], vec![]));
    assert_eq!(control.stop(), Stop::PowerSaver);
}

#[test]
fn a_profile_changed_from_outside_moves_the_slider_to_its_stop_as_the_sources_choice() {
    let memory = Memory {
        ac: Stop::MaxPerformance,
        ..Memory::default()
    };
    let mut control = on(Source::Ac, memory);

    let effects = control.handle(Input::Profile {
        profile: Profile::PowerSaver,
        held: false,
    });

    assert_eq!(control.stop(), Stop::PowerSaver);
    assert_eq!(applied(&effects), [Stop::PowerSaver]);
    assert_eq!(remembered(&effects).map(|memory| memory.ac), Some(Stop::PowerSaver));
}

#[test]
fn an_outside_profile_that_fits_the_stop_leaves_super_saver_alone() {
    let memory = Memory {
        battery: Stop::SuperSaver,
        ..Memory::default()
    };
    let mut control = on(Source::Battery, memory);

    let effects = control.handle(Input::Profile {
        profile: Profile::PowerSaver,
        held: false,
    });

    assert_eq!(control.stop(), Stop::SuperSaver);
    assert_eq!(effects, []);
}

#[test]
fn at_the_level_on_battery_it_drops_to_power_saver_with_undo_and_remembers_nothing_of_it() {
    let memory = Memory {
        battery: Stop::Performance,
        ..Memory::default()
    };
    let mut control = on(Source::Battery, memory);

    assert_eq!(control.handle(Input::Percent(21.0)), []);
    let effects = control.handle(Input::Percent(20.0));

    assert_eq!(control.stop(), Stop::PowerSaver);
    assert_eq!(
        toasts(&effects),
        [Toast::Dropped {
            to: Stop::PowerSaver,
            from: Stop::Performance,
            percent: 20
        }]
    );
    let memory = remembered(&effects).expect("the drop is written down");
    assert_eq!(memory.battery, Stop::Performance);
    assert_eq!(memory.dropped_from, Some(Stop::Performance));
}

#[test]
fn undo_goes_back_to_the_remembered_stop_and_no_drop_comes_again_until_plugged_in() {
    let memory = Memory {
        battery: Stop::Balanced,
        ..Memory::default()
    };
    let mut control = on(Source::Battery, memory);
    control.handle(Input::Percent(19.0));

    let effects = control.handle(Input::Undo);

    assert_eq!(control.stop(), Stop::Balanced);
    assert_eq!(applied(&effects), [Stop::Balanced]);
    assert_eq!(control.handle(Input::Percent(15.0)), []);
    assert_eq!(control.stop(), Stop::Balanced);

    control.handle(Input::Source(Source::Ac));
    control.handle(Input::Source(Source::Battery));
    control.handle(Input::Percent(14.0));
    assert_eq!(control.stop(), Stop::PowerSaver);
}

#[test]
fn undo_without_a_drop_does_nothing() {
    let mut control = on(Source::Battery, Memory::default());

    assert_eq!(control.handle(Input::Undo), []);
}

#[test]
fn the_drop_can_go_to_super_saver_or_be_off() {
    let memory = Memory {
        battery: Stop::Balanced,
        ..Memory::default()
    };
    let mut control = on(Source::Battery, memory);
    control.set_drop(Drop {
        percent: 20,
        to: Some(Stop::SuperSaver),
    });

    control.handle(Input::Percent(18.0));
    assert_eq!(control.stop(), Stop::SuperSaver);

    let mut off = on(Source::Battery, memory);
    off.set_drop(Drop { percent: 20, to: None });
    assert_eq!(off.handle(Input::Percent(5.0)), []);
}

#[test]
fn a_stop_as_low_as_the_drop_is_left_where_it_is() {
    let memory = Memory {
        battery: Stop::SuperSaver,
        ..Memory::default()
    };
    let mut control = on(Source::Battery, memory);

    assert_eq!(control.handle(Input::Percent(10.0)), []);
    assert_eq!(control.stop(), Stop::SuperSaver);
}

#[test]
fn a_choice_made_below_the_level_holds_against_the_drop() {
    let mut control = on(Source::Battery, Memory::default());
    control.handle(Input::Percent(12.0));

    control.handle(Input::Set(Stop::Performance));
    let effects = control.handle(Input::Percent(11.0));

    assert_eq!(effects, []);
    assert_eq!(control.stop(), Stop::Performance);
}

#[test]
fn plugging_in_during_a_drop_ends_it_and_unplugging_again_drops_again_while_low() {
    let memory = Memory {
        battery: Stop::Balanced,
        ac: Stop::Performance,
        ..Memory::default()
    };
    let mut control = on(Source::Battery, memory);
    control.handle(Input::Percent(18.0));

    control.handle(Input::Source(Source::Ac));
    assert_eq!(control.stop(), Stop::Performance);
    assert_eq!(control.dropped_from(), None);

    let effects = control.handle(Input::Source(Source::Battery));
    assert_eq!(control.stop(), Stop::PowerSaver);
    assert!(matches!(
        toasts(&effects).last(),
        Some(Toast::Dropped {
            from: Stop::Balanced,
            ..
        })
    ));
}

#[test]
fn a_restart_keeps_the_stop_it_was_left_on_super_saver_and_a_drop_included() {
    let memory = Memory {
        battery: Stop::Balanced,
        last: Some((Source::Battery, Stop::SuperSaver)),
        ..Memory::default()
    };

    let (control, effects) = Control::start(memory, Source::Battery, Some(Profile::PowerSaver), DROP);

    assert_eq!(control.stop(), Stop::SuperSaver);
    assert_eq!(asked(&effects), []);

    let dropped = Memory {
        battery: Stop::Performance,
        last: Some((Source::Battery, Stop::PowerSaver)),
        dropped_from: Some(Stop::Performance),
        ..Memory::default()
    };
    let (control, _) = Control::start(dropped, Source::Battery, Some(Profile::PowerSaver), DROP);
    assert_eq!(control.stop(), Stop::PowerSaver);
    assert_eq!(control.dropped_from(), Some(Stop::Performance));
}

#[test]
fn a_restart_on_the_other_source_takes_that_sources_stop() {
    let memory = Memory {
        battery: Stop::SuperSaver,
        ac: Stop::Balanced,
        last: Some((Source::Ac, Stop::Balanced)),
        ..Memory::default()
    };

    let (control, effects) = Control::start(memory, Source::Battery, Some(Profile::Balanced), DROP);

    assert_eq!(control.stop(), Stop::SuperSaver);
    assert_eq!(asked(&effects), [Profile::PowerSaver]);
}

#[test]
fn a_profile_changed_while_nothing_watched_is_followed_at_the_restart() {
    let memory = Memory {
        ac: Stop::MaxPerformance,
        last: Some((Source::Ac, Stop::MaxPerformance)),
        ..Memory::default()
    };

    let (control, effects) = Control::start(memory, Source::Ac, Some(Profile::Balanced), DROP);

    assert_eq!(control.stop(), Stop::Balanced);
    assert_eq!(asked(&effects), []);
    assert_eq!(remembered(&effects).map(|memory| memory.ac), Some(Stop::Balanced));
}

#[test]
fn a_bouncing_dock_switches_nothing_and_a_steady_change_switches_once() {
    let start = Instant::now();
    let mut settle = Settle::default();

    settle.observe(Source::Battery, Source::Ac, start);
    settle.observe(Source::Ac, Source::Ac, start + Duration::from_millis(400));
    settle.observe(Source::Battery, Source::Ac, start + Duration::from_millis(900));
    settle.observe(Source::Ac, Source::Ac, start + Duration::from_millis(1500));
    assert_eq!(settle.due(start + Duration::from_secs(10)), None);
    assert_eq!(settle.wait(start), None);

    settle.observe(Source::Battery, Source::Ac, start);
    settle.observe(Source::Battery, Source::Ac, start + Duration::from_secs(1));
    assert_eq!(settle.due(start + Duration::from_secs(2)), None);
    assert_eq!(
        settle.wait(start + Duration::from_secs(2)),
        Some(Duration::from_secs(1))
    );
    assert_eq!(settle.due(start + SETTLE), Some(Source::Battery));
    assert_eq!(settle.due(start + SETTLE * 2), None);
}

#[test]
fn a_programs_hold_is_followed_but_never_remembered_and_its_end_returns_to_the_sources_stop() {
    let memory = Memory {
        battery: Stop::SuperSaver,
        ..Memory::default()
    };
    let mut control = on(Source::Battery, memory);

    let held = control.handle(Input::Profile {
        profile: Profile::Performance,
        held: true,
    });

    assert_eq!(control.stop(), Stop::Performance);
    assert_eq!(applied(&held), [Stop::Performance]);
    assert_eq!(remembered(&held).map(|memory| memory.battery), Some(Stop::SuperSaver));

    let released = control.handle(Input::Profile {
        profile: Profile::PowerSaver,
        held: false,
    });

    assert_eq!(control.stop(), Stop::SuperSaver);
    assert_eq!(applied(&released), [Stop::SuperSaver]);
    assert_eq!(
        remembered(&released).map(|memory| memory.battery),
        Some(Stop::SuperSaver)
    );
}

#[test]
fn a_burst_of_device_events_is_checked_once_after_it_goes_quiet() {
    let start = Instant::now();
    let mut recheck = Recheck::default();

    recheck.ask(start);
    recheck.ask(start + Duration::from_millis(600));
    assert!(!recheck.due(start + Duration::from_millis(1200)));
    assert_eq!(
        recheck.wait(start + Duration::from_millis(1200)),
        Some(Duration::from_millis(400))
    );

    assert!(recheck.due(start + Duration::from_millis(1600)));
    assert!(!recheck.due(start + Duration::from_secs(5)));
    assert_eq!(recheck.wait(start), None);
}

#[test]
fn a_device_that_never_goes_quiet_is_checked_after_5_seconds_at_most() {
    let start = Instant::now();
    let mut recheck = Recheck::default();
    let mut checked = Vec::new();

    for tenth in 0..120u32 {
        let now = start + Duration::from_millis(u64::from(tenth) * 100);
        if recheck.due(now) {
            checked.push(tenth);
        }
        recheck.ask(now);
    }

    assert_eq!(checked, [50, 100]);
}
