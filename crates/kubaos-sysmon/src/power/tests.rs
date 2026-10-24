use std::fs;
use std::path::Path;

use super::read_battery;

fn supply(root: &Path, name: &str, files: &[(&str, &str)]) {
    let dir = root.join(format!("sys/class/power_supply/{name}"));
    fs::create_dir_all(&dir).expect("make the supply");
    for (file, text) in files {
        fs::write(dir.join(file), format!("{text}\n")).expect("write a supply file");
    }
}

#[test]
fn a_discharging_battery_shows_its_draw_time_left_and_health() {
    let root = tempfile::tempdir().expect("a temp dir");
    supply(root.path(), "AC", &[("type", "Mains"), ("online", "0")]);
    supply(
        root.path(),
        "BAT0",
        &[
            ("type", "Battery"),
            ("status", "Discharging"),
            ("capacity", "76"),
            ("energy_now", "45600000"),
            ("energy_full", "60000000"),
            ("energy_full_design", "80000000"),
            ("power_now", "12000000"),
        ],
    );

    let battery = read_battery(root.path()).expect("a battery");

    assert!((battery.percent - 76.0).abs() < 0.01);
    assert_eq!(battery.status, "Discharging");
    assert_eq!(battery.watts, Some(12.0));
    assert_eq!(battery.seconds_left, Some(3 * 3600 + 48 * 60));
    assert_eq!(battery.health, Some(75.0));
}

#[test]
fn a_charging_battery_counts_the_time_to_full() {
    let root = tempfile::tempdir().expect("a temp dir");
    supply(
        root.path(),
        "BAT1",
        &[
            ("type", "Battery"),
            ("status", "Charging"),
            ("capacity", "50"),
            ("energy_now", "30000000"),
            ("energy_full", "60000000"),
            ("power_now", "30000000"),
        ],
    );

    let battery = read_battery(root.path()).expect("a battery");

    assert_eq!(battery.seconds_left, Some(3600));
    assert_eq!(battery.health, None);
}

#[test]
fn a_battery_reporting_charge_works_out_watts_from_current_and_voltage() {
    let root = tempfile::tempdir().expect("a temp dir");
    supply(
        root.path(),
        "BAT0",
        &[
            ("type", "Battery"),
            ("status", "Discharging"),
            ("charge_now", "2000000"),
            ("charge_full", "4000000"),
            ("charge_full_design", "5000000"),
            ("current_now", "1000000"),
            ("voltage_now", "15000000"),
        ],
    );

    let battery = read_battery(root.path()).expect("a battery");

    assert_eq!(battery.watts, Some(15.0));
    assert!((battery.percent - 50.0).abs() < 0.01);
    assert_eq!(battery.seconds_left, Some(7200));
    assert_eq!(battery.health, Some(80.0));
}

#[test]
fn a_mouse_battery_is_not_the_laptops() {
    let root = tempfile::tempdir().expect("a temp dir");
    supply(
        root.path(),
        "hidpp_battery_0",
        &[("type", "Battery"), ("scope", "Device"), ("capacity", "40")],
    );

    assert!(read_battery(root.path()).is_none());
}

#[test]
fn a_desktop_has_no_battery() {
    let root = tempfile::tempdir().expect("a temp dir");

    assert!(read_battery(root.path()).is_none());
}
