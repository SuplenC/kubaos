use std::path::Path;
use std::path::PathBuf;

use super::Battery;
use super::Supply;
use super::read;

/// A fake `/sys/class/power_supply`, removed when dropped.
struct Sysfs(PathBuf);

impl Sysfs {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("kobo-battery-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create the fake sysfs");
        Self(dir)
    }

    /// A supply named `name` with these attribute files.
    fn supply(self, name: &str, attrs: &[(&str, &str)]) -> Self {
        let dir = self.0.join(name);
        std::fs::create_dir_all(&dir).expect("create the supply dir");
        for (file, value) in attrs {
            std::fs::write(dir.join(file), format!("{value}\n")).expect("write a sysfs attribute");
        }
        self
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Sysfs {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn battery(percent: u8, supply: Supply) -> Option<Battery> {
    Some(Battery { percent, supply })
}

#[test]
fn one_discharging_battery_reads_its_capacity() {
    let sys = Sysfs::new("one").supply(
        "BAT0",
        &[
            ("type", "Battery"),
            ("capacity", "64"),
            ("status", "Discharging"),
        ],
    );

    assert_eq!(read(sys.path()), battery(64, Supply::Discharging));
}

#[test]
fn a_charging_battery_is_charging() {
    let sys = Sysfs::new("charging")
        .supply(
            "BAT0",
            &[
                ("type", "Battery"),
                ("capacity", "41"),
                ("status", "Charging"),
            ],
        )
        .supply("AC", &[("type", "Mains"), ("online", "1")]);

    assert_eq!(read(sys.path()), battery(41, Supply::Charging));
}

#[test]
fn a_full_battery_on_the_charger_is_plugged_in() {
    let sys = Sysfs::new("full")
        .supply(
            "BAT0",
            &[("type", "Battery"), ("capacity", "100"), ("status", "Full")],
        )
        .supply("ACAD", &[("type", "Mains"), ("online", "1")]);

    assert_eq!(read(sys.path()), battery(100, Supply::PluggedIn));
}

#[test]
fn a_battery_held_below_full_on_the_charger_is_plugged_in() {
    let sys = Sysfs::new("held")
        .supply(
            "BAT0",
            &[
                ("type", "Battery"),
                ("capacity", "80"),
                ("status", "Not charging"),
            ],
        )
        .supply("AC", &[("type", "Mains"), ("online", "1")]);

    assert_eq!(read(sys.path()), battery(80, Supply::PluggedIn));
}

#[test]
fn a_full_battery_with_no_mains_supply_listed_is_plugged_in() {
    let sys = Sysfs::new("full-no-mains").supply(
        "BAT0",
        &[("type", "Battery"), ("capacity", "100"), ("status", "Full")],
    );

    assert_eq!(read(sys.path()), battery(100, Supply::PluggedIn));
}

#[test]
fn an_unplugged_charger_leaves_the_battery_discharging() {
    let sys = Sysfs::new("unplugged")
        .supply(
            "BAT0",
            &[
                ("type", "Battery"),
                ("capacity", "77"),
                ("status", "Discharging"),
            ],
        )
        .supply("AC", &[("type", "Mains"), ("online", "0")]);

    assert_eq!(read(sys.path()), battery(77, Supply::Discharging));
}

#[test]
fn two_batteries_combine_by_energy() {
    let sys = Sysfs::new("two-energy")
        .supply(
            "BAT0",
            &[
                ("type", "Battery"),
                ("capacity", "100"),
                ("energy_now", "20000000"),
                ("energy_full", "20000000"),
                ("status", "Full"),
            ],
        )
        .supply(
            "BAT1",
            &[
                ("type", "Battery"),
                ("capacity", "10"),
                ("energy_now", "6000000"),
                ("energy_full", "60000000"),
                ("status", "Discharging"),
            ],
        );

    assert_eq!(read(sys.path()), battery(33, Supply::Discharging));
}

#[test]
fn two_batteries_without_energy_average_their_capacity() {
    let sys = Sysfs::new("two-capacity")
        .supply(
            "BAT0",
            &[
                ("type", "Battery"),
                ("capacity", "90"),
                ("status", "Discharging"),
            ],
        )
        .supply(
            "BAT1",
            &[
                ("type", "Battery"),
                ("capacity", "31"),
                ("energy_now", "6000000"),
                ("energy_full", "60000000"),
                ("status", "Discharging"),
            ],
        );

    assert_eq!(read(sys.path()), battery(61, Supply::Discharging));
}

#[test]
fn one_of_two_batteries_charging_makes_the_machine_charging() {
    let sys = Sysfs::new("two-charging")
        .supply(
            "BAT0",
            &[
                ("type", "Battery"),
                ("capacity", "50"),
                ("status", "Charging"),
            ],
        )
        .supply(
            "BAT1",
            &[
                ("type", "Battery"),
                ("capacity", "100"),
                ("status", "Not charging"),
            ],
        );

    assert_eq!(read(sys.path()), battery(75, Supply::Charging));
}

#[test]
fn a_battery_without_capacity_reads_its_energy() {
    let sys = Sysfs::new("energy-only").supply(
        "BAT0",
        &[
            ("type", "Battery"),
            ("energy_now", "45000000"),
            ("energy_full", "50000000"),
            ("status", "Discharging"),
        ],
    );

    assert_eq!(read(sys.path()), battery(90, Supply::Discharging));
}

#[test]
fn a_worn_battery_never_reads_past_full() {
    let sys = Sysfs::new("worn").supply(
        "BAT0",
        &[
            ("type", "Battery"),
            ("energy_now", "51000000"),
            ("energy_full", "50000000"),
            ("status", "Full"),
        ],
    );

    assert_eq!(read(sys.path()), battery(100, Supply::PluggedIn));
}

#[test]
fn a_battery_with_no_status_counts_as_discharging() {
    let sys = Sysfs::new("no-status").supply("BAT0", &[("type", "Battery"), ("capacity", "12")]);

    assert_eq!(read(sys.path()), battery(12, Supply::Discharging));
}

#[test]
fn a_machine_with_only_a_charger_has_no_battery() {
    let sys = Sysfs::new("desktop").supply("AC", &[("type", "Mains"), ("online", "1")]);

    assert_eq!(read(sys.path()), None);
}

#[test]
fn a_machine_with_no_power_supplies_has_no_battery() {
    let sys = Sysfs::new("vm");

    assert_eq!(read(sys.path()), None);
}

#[test]
fn a_missing_power_supply_dir_means_no_battery() {
    assert_eq!(read(Path::new("/nonexistent/power_supply")), None);
}

#[test]
fn a_battery_reporting_no_charge_is_skipped() {
    let sys = Sysfs::new("no-charge")
        .supply("BAT0", &[("type", "Battery"), ("status", "Unknown")])
        .supply(
            "BAT1",
            &[
                ("type", "Battery"),
                ("capacity", "58"),
                ("status", "Discharging"),
            ],
        );

    assert_eq!(read(sys.path()), battery(58, Supply::Discharging));
}

#[test]
fn a_mouse_battery_is_not_the_machine_s() {
    let sys = Sysfs::new("mouse").supply(
        "hidpp_battery_0",
        &[
            ("type", "Battery"),
            ("scope", "Device"),
            ("capacity", "5"),
            ("status", "Discharging"),
        ],
    );

    assert_eq!(read(sys.path()), None);
}

#[test]
fn an_empty_battery_bay_is_skipped() {
    let sys = Sysfs::new("empty-bay")
        .supply(
            "BAT0",
            &[
                ("type", "Battery"),
                ("capacity", "70"),
                ("status", "Discharging"),
            ],
        )
        .supply(
            "BAT1",
            &[("type", "Battery"), ("present", "0"), ("capacity", "0")],
        );

    assert_eq!(read(sys.path()), battery(70, Supply::Discharging));
}

#[test]
fn only_a_discharging_battery_at_twenty_percent_or_less_is_low() {
    let at = |percent, supply| Battery { percent, supply }.is_low();

    assert!(at(20, Supply::Discharging));
    assert!(at(3, Supply::Discharging));
    assert!(!at(21, Supply::Discharging));
    assert!(!at(5, Supply::Charging));
    assert!(!at(5, Supply::PluggedIn));
}
