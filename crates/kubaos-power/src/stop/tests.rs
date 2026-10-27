use super::*;

#[test]
fn the_stops_run_from_super_saver_to_max_performance_in_slider_order() {
    let titles: Vec<&str> = Stop::ALL.iter().map(|stop| stop.title()).collect();

    assert_eq!(
        titles,
        [
            "Super Saver",
            "Power Saver",
            "Balanced",
            "Performance",
            "Max Performance"
        ]
    );
    for (index, stop) in Stop::ALL.into_iter().enumerate() {
        assert_eq!(stop.index(), index);
        assert_eq!(Stop::from_index(index), Some(stop));
    }
    assert_eq!(Stop::from_index(5), None);
}

#[test]
fn the_extremes_sit_on_the_profile_next_to_them_so_other_tools_show_something_sensible() {
    assert_eq!(Stop::SuperSaver.profile(), Profile::PowerSaver);
    assert_eq!(Stop::PowerSaver.profile(), Profile::PowerSaver);
    assert_eq!(Stop::Balanced.profile(), Profile::Balanced);
    assert_eq!(Stop::Performance.profile(), Profile::Performance);
    assert_eq!(Stop::MaxPerformance.profile(), Profile::Performance);
}

#[test]
fn a_profile_set_from_outside_lands_on_the_stop_named_like_it() {
    for profile in [Profile::PowerSaver, Profile::Balanced, Profile::Performance] {
        let stop = Stop::from_profile(profile);

        assert_eq!(stop.profile(), profile);
        assert_eq!(stop.id(), profile.name());
    }
}

#[test]
fn a_stop_is_found_by_its_id_however_it_is_typed() {
    assert_eq!(Stop::from_id("super-saver"), Some(Stop::SuperSaver));
    assert_eq!(Stop::from_id("Max Performance"), Some(Stop::MaxPerformance));
    assert_eq!(Stop::from_id("power_saver"), Some(Stop::PowerSaver));
    assert_eq!(Stop::from_id("turbo"), None);
}

#[test]
fn battery_starts_on_power_saver_and_ac_on_balanced() {
    assert_eq!(Source::Battery.default_stop(), Stop::PowerSaver);
    assert_eq!(Source::Ac.default_stop(), Stop::Balanced);
    assert_eq!(Source::from_on_battery(true), Source::Battery);
    assert_eq!(Source::from_on_battery(false), Source::Ac);
}
