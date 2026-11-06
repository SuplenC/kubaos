use super::*;
use crate::fake::FakeRoot;

#[test]
fn the_cpu_policies_are_read_in_natural_order() {
    let root = FakeRoot::laptop();

    let policies = root.machine().policies();

    let names: Vec<&str> = policies.iter().map(|policy| policy.name.as_str()).collect();
    assert_eq!(names, ["policy0", "policy1", "policy10"]);
    assert_eq!(policies[0].epp.as_deref(), Some("power"));
    assert_eq!(policies[0].boost, Some(false));
    assert_eq!(policies[0].max_khz, Some(2_000_000));
}

#[test]
fn the_gpu_and_the_laptop_panel_are_found_and_the_externals_left_out() {
    let root = FakeRoot::laptop();
    let machine = root.machine();

    assert_eq!(machine.gpus(), [("card1".to_owned(), "auto".to_owned())]);
    assert_eq!(machine.panels(), [("card1-eDP-1".to_owned(), 0)]);
}

#[test]
fn a_disconnected_panel_is_not_one() {
    let root = FakeRoot::laptop();
    root.write("sys/class/drm/card1-eDP-1/status", "disconnected");

    assert_eq!(root.machine().panels(), []);
}

#[test]
fn the_aspm_policy_in_use_is_the_bracketed_one() {
    let root = FakeRoot::laptop();

    assert_eq!(root.machine().aspm().as_deref(), Some("default"));
}

#[test]
fn hubs_are_left_out_and_input_devices_and_the_dock_lan_are_known() {
    let root = FakeRoot::laptop();

    let devices = root.machine().usb_devices();

    let names: Vec<(&str, bool, bool)> = devices
        .iter()
        .map(|device| (device.name.as_str(), device.is_input(), device.kept_awake()))
        .collect();
    assert_eq!(
        names,
        [
            ("1-1", false, false),
            ("1-3", false, false),
            ("3-2", true, true),
            ("7-1.1.3", true, true),
            ("8-1.1.2", false, true)
        ]
    );
}

#[test]
fn wifi_is_the_interfaces_with_a_wireless_folder() {
    let root = FakeRoot::laptop();

    assert_eq!(root.machine().wifi(), ["wlp99s0"]);
}

#[test]
fn the_charge_limit_is_tuxedos_profile_else_the_kernels_threshold() {
    let root = FakeRoot::laptop();

    assert_eq!(
        root.machine().charge(),
        Some(ChargeKnob::Tuxedo {
            profile: "high_capacity".to_owned()
        })
    );

    root.remove("sys/devices/platform/tuxedo_keyboard");
    root.write("sys/class/power_supply/BAT0/charge_control_end_threshold", "80");
    let knob = root.machine().charge().expect("a threshold");
    assert_eq!(knob.limit(), Some(ChargeLimit::Eighty));
}

#[test]
fn the_battery_draw_comes_from_current_and_voltage_and_a_mouse_battery_is_not_the_laptops() {
    let root = FakeRoot::laptop();

    let battery = root.machine().battery().expect("a battery");

    assert!((battery.watts.expect("a draw") - 16.8).abs() < 1e-9);
    assert_eq!(battery.percent, Some(64.0));
    assert!(battery.discharging);
}

#[test]
fn the_energy_is_energy_now_else_the_charge_at_the_design_voltage() {
    let root = FakeRoot::laptop();
    assert_eq!(root.machine().battery().expect("a battery").wh, None);

    root.write("sys/class/power_supply/BAT0/charge_now", "1144000");
    root.write("sys/class/power_supply/BAT0/voltage_now", "16000000");
    let by_voltage = root.machine().battery().expect("a battery").wh.expect("an energy");
    root.write("sys/class/power_supply/BAT0/voltage_min_design", "15400000");
    let by_design = root.machine().battery().expect("a battery").wh.expect("an energy");
    root.write("sys/class/power_supply/BAT0/energy_now", "51200000");
    let reported = root.machine().battery().expect("a battery").wh.expect("an energy");

    assert!((by_voltage - 18.304).abs() < 1e-9);
    assert!((by_design - 17.6176).abs() < 1e-9);
    assert!((reported - 51.2).abs() < 1e-9);
}

#[test]
fn the_panel_and_a_second_screen_are_told_apart() {
    let root = FakeRoot::laptop();
    root.write("sys/class/drm/card1-eDP-1/dpms", "On");

    assert_eq!(
        root.machine().displays(),
        Displays {
            panel_on: Some(true),
            external: true
        }
    );

    root.write("sys/class/drm/card1-eDP-1/dpms", "Off");
    root.write("sys/class/drm/card1-HDMI-A-1/status", "disconnected");
    assert_eq!(
        root.machine().displays(),
        Displays {
            panel_on: Some(false),
            external: false
        }
    );
}

#[test]
fn the_backlight_is_a_percent_of_its_range_and_may_be_missing() {
    let root = FakeRoot::laptop();
    assert_eq!(root.machine().brightness(), None);

    root.write("sys/class/backlight/amdgpu_bl1/brightness", "102");
    root.write("sys/class/backlight/amdgpu_bl1/max_brightness", "255");
    assert_eq!(root.machine().brightness(), Some(40));

    root.write("sys/class/backlight/amdgpu_bl1/max_brightness", "0");
    assert_eq!(root.machine().brightness(), None);
}

#[test]
fn the_cpu_is_as_busy_as_the_ticks_that_were_not_idle() {
    let before = CpuTimes::parse("cpu  100 0 100 800 0 0 0 0 0 0\ncpu0 1 1 1 1\n").expect("reads");
    let after = CpuTimes::parse("cpu  150 0 100 830 20 0 0 0 7 7\n").expect("reads");

    assert_eq!(before.total, 1000);
    assert_eq!(before.idle, 800);
    assert_eq!(after.busy_since(before), Some(50.0));
    assert_eq!(before.busy_since(before), None);
    assert_eq!(CpuTimes::parse("intr 1 2 3"), None);
}

#[test]
fn a_charging_battery_has_no_draw() {
    let root = FakeRoot::laptop();
    root.write("sys/class/power_supply/BAT0/status", "Charging");

    assert_eq!(root.machine().battery().expect("a battery").watts, None);
}

#[test]
fn seamless_refresh_needs_amdgpus_freesync_video() {
    let root = FakeRoot::laptop();
    assert!(root.machine().seamless_refresh());

    root.write("sys/module/amdgpu/parameters/freesync_video", "0");
    assert!(!root.machine().seamless_refresh());
}

#[test]
fn an_empty_machine_has_no_knob_at_all() {
    let root = FakeRoot::empty();
    let machine = root.machine();

    assert_eq!(machine.policies(), Vec::<Policy>::new());
    assert_eq!(machine.gpus(), Vec::<(String, String)>::new());
    assert_eq!(machine.panels(), Vec::<(String, u8)>::new());
    assert_eq!(machine.aspm(), None);
    assert_eq!(machine.usb_devices(), Vec::<UsbDevice>::new());
    assert_eq!(machine.wifi(), Vec::<String>::new());
    assert_eq!(machine.charge(), None);
    assert_eq!(machine.battery(), None);
    assert!(!machine.seamless_refresh());
}

#[test]
fn a_process_counts_the_switches_of_all_its_threads() {
    let root = FakeRoot::empty();
    root.write("proc/42/comm", "localsearch-3");
    root.write(
        "proc/42/task/42/status",
        "Name:\tx\nvoluntary_ctxt_switches:\t10\nnonvoluntary_ctxt_switches:\t2",
    );
    root.write(
        "proc/42/task/43/status",
        "voluntary_ctxt_switches:\t5\nnonvoluntary_ctxt_switches:\t0",
    );

    assert_eq!(root.machine().pids(), [42]);
    assert_eq!(root.machine().switches(42), Some(("localsearch-3".to_owned(), 17)));
}
