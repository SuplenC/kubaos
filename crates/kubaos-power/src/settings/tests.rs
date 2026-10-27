use std::fs;

use super::*;

const SEED: &str = include_str!("../../../../user-layer/desktop/power/power.toml");

fn file(text: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("a temp dir");
    let path = dir.path().join("power").join(FILE);
    if !text.is_empty() {
        fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
        fs::write(&path, text).expect("write the settings");
    }
    (dir, path)
}

#[test]
fn a_missing_file_and_the_seed_are_the_defaults() {
    let (_dir, path) = file("");

    assert_eq!(Settings::load(&path).expect("loads"), Settings::default());
    assert_eq!(Settings::parse(SEED).expect("the seed parses"), Settings::default());
}

#[test]
fn the_defaults_drop_to_power_saver_at_20_and_charge_full() {
    let settings = Settings::default();

    assert_eq!(settings.low_battery, 20);
    assert_eq!(settings.low_battery_stop, Some(Stop::PowerSaver));
    assert_eq!(settings.charge_limit, ChargeLimit::Full);
    assert!(settings.panel_power_saving);
    assert_eq!(settings.usb_skip, Vec::<UsbId>::new());
}

#[test]
fn every_key_reads() {
    let settings = Settings::parse(
        "panel_power_saving = false\ncharge_limit = 80\nlow_battery = 15\nlow_battery_stop = \"super-saver\"\n\
         [usb]\nskip = [\"046D:C547\", \"1050:*\"]\n[super-saver]\npanel = \"vrr\"\ncpu_max_mhz = 1400\n",
    )
    .expect("parses");

    assert!(!settings.panel_power_saving);
    assert_eq!(settings.charge_limit, ChargeLimit::Eighty);
    assert_eq!(settings.low_battery, 15);
    assert_eq!(settings.low_battery_stop, Some(Stop::SuperSaver));
    assert!(settings.usb_skip[0].matches("046d", "c547"));
    assert!(settings.usb_skip[1].matches("1050", "0407"));
    assert!(!settings.usb_skip[0].matches("046d", "c548"));
    assert_eq!(
        settings.stop(Stop::SuperSaver),
        StopSettings {
            panel: PanelMode::Vrr,
            cpu_max_mhz: Some(1400)
        }
    );
}

#[test]
fn values_out_of_range_are_refused_and_say_why() {
    let refused = |text: &str| Settings::parse(text).expect_err("refused").to_string();

    assert_eq!(refused("charge_limit = 85"), "`85` isn't a charge limit: 80, 90 or 100");
    assert_eq!(
        refused("low_battery = 70"),
        "the low-battery drop comes between 5 and 50 %, not at 70 %"
    );
    assert_eq!(
        refused("low_battery_stop = \"balanced\""),
        "`balanced` isn't where the low-battery drop can go: power-saver, super-saver or off"
    );
    assert!(refused("[usb]\nskip = [\"keyboard\"]").starts_with("`keyboard` isn't a USB device"));
    assert_eq!(refused("[balanced]\npanel = \"30hz\""), "in [balanced]");
    assert!(matches!(Settings::parse("turbo = true"), Err(SettingsError::Parse(_))));
}

#[test]
fn the_low_battery_drop_can_be_turned_off() {
    let settings = Settings::parse("low_battery_stop = \"off\"").expect("parses");

    assert_eq!(settings.low_battery_stop, None);
}

#[test]
fn a_change_keeps_the_users_comments() {
    let (_dir, path) = file("# My laptop lives on a desk.\ncharge_limit = 100\n");

    let settings = set_charge_limit(&path, ChargeLimit::Eighty).expect("changes");

    assert_eq!(settings.charge_limit, ChargeLimit::Eighty);
    assert_eq!(
        fs::read_to_string(&path).expect("read"),
        "# My laptop lives on a desk.\ncharge_limit = 80\n"
    );
}

#[test]
fn every_change_lands_in_the_file() {
    let (_dir, path) = file("");

    set_panel_power_saving(&path, false).expect("panel power saving");
    set_low_battery_stop(&path, Some(Stop::SuperSaver)).expect("drop stop");
    set_low_battery(&path, 25).expect("drop level");
    let settings = set_panel(&path, Stop::Balanced, PanelMode::Native).expect("panel");

    assert!(!settings.panel_power_saving);
    assert_eq!(settings.low_battery_stop, Some(Stop::SuperSaver));
    assert_eq!(settings.low_battery, 25);
    assert_eq!(settings.stop(Stop::Balanced).panel, PanelMode::Native);
    assert_eq!(Settings::load(&path).expect("reads back"), settings);
    assert!(set_low_battery(&path, 3).is_err());
}

#[test]
fn a_broken_file_is_left_as_it_is() {
    let (_dir, path) = file("charge_limit = 85\n");

    assert!(set_charge_limit(&path, ChargeLimit::Ninety).is_err());
    assert_eq!(fs::read_to_string(&path).expect("read"), "charge_limit = 85\n");
}

#[test]
fn the_tuxedo_profiles_stand_for_80_90_and_100() {
    assert_eq!(
        ChargeLimit::from_tuxedo_profile("stationary"),
        Some(ChargeLimit::Eighty)
    );
    assert_eq!(ChargeLimit::from_tuxedo_profile("balanced"), Some(ChargeLimit::Ninety));
    assert_eq!(
        ChargeLimit::from_tuxedo_profile("high_capacity"),
        Some(ChargeLimit::Full)
    );
}
