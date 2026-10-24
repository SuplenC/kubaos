use std::fs;
use std::path::Path;

use super::Sensors;

fn chip(root: &Path, index: u32, name: &str, files: &[(&str, &str)]) {
    let dir = root.join(format!("sys/class/hwmon/hwmon{index}"));
    fs::create_dir_all(&dir).expect("make the chip");
    fs::write(dir.join("name"), format!("{name}\n")).expect("write its name");
    for (file, text) in files {
        fs::write(dir.join(file), format!("{text}\n")).expect("write a sensor");
    }
}

#[test]
fn the_cpu_temperature_is_amds_tctl() {
    let root = tempfile::tempdir().expect("a temp dir");
    chip(root.path(), 0, "acpitz", &[("temp1_input", "40000")]);
    chip(
        root.path(),
        1,
        "k10temp",
        &[
            ("temp1_input", "61250"),
            ("temp1_label", "Tctl"),
            ("temp3_input", "50000"),
            ("temp3_label", "Tccd1"),
        ],
    );

    let sensors = Sensors::discover(root.path());

    assert_eq!(sensors.cpu_temperature(), Some(61.25));
}

#[test]
fn intels_package_temperature_stands_in() {
    let root = tempfile::tempdir().expect("a temp dir");
    chip(
        root.path(),
        2,
        "coretemp",
        &[
            ("temp2_input", "45000"),
            ("temp2_label", "Core 0"),
            ("temp1_input", "52000"),
            ("temp1_label", "Package id 0"),
        ],
    );

    assert_eq!(Sensors::discover(root.path()).cpu_temperature(), Some(52.0));
}

#[test]
fn a_thermal_zone_is_the_last_resort() {
    let root = tempfile::tempdir().expect("a temp dir");
    let zone = root.path().join("sys/class/thermal/thermal_zone3");
    fs::create_dir_all(&zone).expect("make the zone");
    fs::write(zone.join("type"), "x86_pkg_temp\n").expect("write its type");
    fs::write(zone.join("temp"), "48000\n").expect("write its temperature");

    assert_eq!(Sensors::discover(root.path()).cpu_temperature(), Some(48.0));
}

#[test]
fn no_sensor_is_no_temperature() {
    let root = tempfile::tempdir().expect("a temp dir");

    let sensors = Sensors::discover(root.path());

    assert_eq!(sensors.cpu_temperature(), None);
    assert!(sensors.fans().is_empty());
}

#[test]
fn every_fan_is_listed_with_its_label_or_its_chip() {
    let root = tempfile::tempdir().expect("a temp dir");
    chip(
        root.path(),
        4,
        "tuxedo",
        &[("fan1_input", "2400"), ("fan1_label", "cpu0"), ("fan2_input", "0")],
    );

    let fans = Sensors::discover(root.path()).fans();

    assert_eq!(fans.len(), 2);
    assert_eq!((fans[0].label.as_str(), fans[0].rpm), ("cpu0", 2400));
    assert_eq!((fans[1].label.as_str(), fans[1].rpm), ("tuxedo fan 2", 0));
}
