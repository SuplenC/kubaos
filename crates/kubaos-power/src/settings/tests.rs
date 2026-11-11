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
    assert!(settings.battery_notes);
    assert!(settings.background_yields);
    assert_eq!(settings.usb_skip, Vec::<UsbId>::new());
}

#[test]
fn every_key_reads() {
    let settings = Settings::parse(
        "panel_power_saving = false\ncharge_limit = 80\nlow_battery = 15\nlow_battery_stop = \"super-saver\"\n\
         [usb]\nskip = [\"046D:C547\", \"1050:*\"]\n[super-saver]\npanel = \"vrr\"\ncpu_max_mhz = 1400\naspm = \"powersave\"\n",
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
            aspm: Aspm::PowerSave,
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
    assert_eq!(refused("[power-saver]\naspm = \"l3\""), "in [power-saver]");
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
fn lavd_is_on_until_the_user_turns_it_off() {
    let (_dir, path) = file("# Measured first.\ncharge_limit = 100\n");

    let off = set_scheduler(&path, CpuScheduler::Kernel).expect("off");
    let text = fs::read_to_string(&path).expect("read");
    let on = set_scheduler(&path, CpuScheduler::Lavd).expect("on");
    let after = fs::read_to_string(&path).expect("read");

    assert_eq!(Settings::default().scheduler, CpuScheduler::Lavd);
    assert_eq!(off.scheduler, CpuScheduler::Kernel);
    assert_eq!(text, "# Measured first.\ncharge_limit = 100\nscheduler = \"kernel\"\n");
    assert_eq!(on.scheduler, CpuScheduler::Lavd);
    assert_eq!(after, "# Measured first.\ncharge_limit = 100\n");
}

#[test]
fn a_file_that_names_lavd_keeps_it() {
    let settings = Settings::parse("scheduler = \"lavd\"\n").expect("parses");

    assert_eq!(settings.scheduler, CpuScheduler::Lavd);
}

#[test]
fn a_scheduler_that_isnt_one_is_refused() {
    let refused = Settings::parse("scheduler = \"bpfland\"").expect_err("refused");

    assert_eq!(refused.to_string(), "`bpfland` isn't a CPU scheduler: kernel or lavd");
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

#[test]
fn the_critical_warning_defaults_to_five_percent() {
    assert_eq!(Settings::default().critical_warning, Some(5));
}

#[test]
fn the_critical_warning_reads_a_percent_or_off() {
    let level = Settings::parse("critical_warning = 8\n").expect("parses");
    let off = Settings::parse("critical_warning = \"off\"\n").expect("parses");

    assert_eq!(level.critical_warning, Some(8));
    assert_eq!(off.critical_warning, None);
}

#[test]
fn the_critical_warning_refuses_anything_else_in_the_file() {
    for text in [
        "critical_warning = 2\n",
        "critical_warning = 21\n",
        "critical_warning = -5\n",
        "critical_warning = \"soon\"\n",
        "critical_warning = true\n",
    ] {
        assert!(
            matches!(Settings::parse(text), Err(SettingsError::NotACriticalWarning(_))),
            "{text}"
        );
    }
}

#[test]
fn the_critical_warning_parses_what_the_command_is_given() {
    assert_eq!(parse_critical_warning("off").expect("off"), None);
    assert_eq!(parse_critical_warning("7").expect("7"), Some(7));
    assert_eq!(parse_critical_warning("7%").expect("7%"), Some(7));

    for text in ["2", "21", "on", ""] {
        assert!(parse_critical_warning(text).is_err(), "{text:?}");
    }
}

#[test]
fn setting_the_critical_warning_keeps_the_comments_and_the_default_removes_the_key() {
    let (_dir, path) = file("# mine\nlow_battery = 15\n");

    let off = set_critical_warning(&path, None).expect("off");
    assert_eq!(off.critical_warning, None);
    assert!(
        fs::read_to_string(&path)
            .expect("read")
            .contains("critical_warning = \"off\"")
    );

    let level = set_critical_warning(&path, Some(8)).expect("8");
    assert_eq!(level.critical_warning, Some(8));

    let default = set_critical_warning(&path, Some(DEFAULT_CRITICAL_WARNING)).expect("the default");
    let text = fs::read_to_string(&path).expect("read");
    assert_eq!(default.critical_warning, Some(DEFAULT_CRITICAL_WARNING));
    assert!(!text.contains("critical_warning"), "{text}");
    assert!(text.contains("# mine"), "{text}");
}

#[test]
fn setting_the_critical_warning_out_of_range_changes_nothing() {
    let (_dir, path) = file("low_battery = 15\n");

    assert!(set_critical_warning(&path, Some(2)).is_err());

    assert_eq!(fs::read_to_string(&path).expect("read"), "low_battery = 15\n");
}

#[test]
fn battery_notes_off_is_a_line_and_on_removes_it() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let path = dir.path().join(FILE);
    fs::write(&path, "# my comment\ncharge_limit = 80\n").expect("write");

    let off = set_battery_notes(&path, false).expect("off");
    let written = fs::read_to_string(&path).expect("read");
    let on = set_battery_notes(&path, true).expect("on");

    assert!(!off.battery_notes);
    assert!(written.contains("battery_notes = false"));
    assert!(written.contains("# my comment"));
    assert!(on.battery_notes);
    assert!(!fs::read_to_string(&path).expect("read").contains("battery_notes"));
}

#[test]
fn background_yields_off_is_a_line_and_on_removes_it() {
    let (_dir, path) = file("# my comment\ncharge_limit = 80\n");

    let off = set_background_yields(&path, false).expect("off");
    let written = fs::read_to_string(&path).expect("read");
    let on = set_background_yields(&path, true).expect("on");

    assert!(!off.background_yields);
    assert_eq!(written, "# my comment\ncharge_limit = 80\nbackground_yields = false\n");
    assert!(on.background_yields);
    assert_eq!(
        fs::read_to_string(&path).expect("read"),
        "# my comment\ncharge_limit = 80\n"
    );
}
