use super::*;
use crate::fake::FakeRoot;
use crate::settings::UsbId;
use crate::stop::Stop;

fn args(stop: Stop, settings: &Settings, root: &FakeRoot, wifi: Option<bool>) -> Vec<String> {
    let plan = Plan::for_stop(stop, settings);
    for_plan(&plan, settings, &root.machine(), &|_| wifi)
        .iter()
        .map(Write::arg)
        .collect()
}

#[test]
fn super_saver_on_the_laptop_today_writes_every_row_it_lacks() {
    let root = FakeRoot::laptop();

    let args = args(Stop::SuperSaver, &Settings::default(), &root, Some(true));

    assert_eq!(
        args,
        [
            "cpu-max=1600000",
            "gpu-dpm=card1:low",
            "panel-power=card1-eDP-1:3",
            "aspm=powersupersave",
            "usb=1-3:auto",
        ]
    );
}

#[test]
fn input_devices_never_autosuspend_and_the_skip_list_keeps_others_awake() {
    let root = FakeRoot::laptop();
    root.write("sys/bus/usb/devices/7-1.1.3/power/control", "auto");
    root.write("sys/bus/usb/devices/3-2/power/control", "auto");
    let mut settings = Settings::default();
    settings.usb_skip = vec![UsbId::parse("0bda:0177").expect("an id")];

    let args = args(Stop::PowerSaver, &settings, &root, Some(true));

    assert!(args.contains(&"usb=7-1.1.3:on".to_owned()));
    assert!(args.contains(&"usb=3-2:on".to_owned()));
    assert!(!args.iter().any(|arg| arg.starts_with("usb=1-3")));
    assert!(!args.iter().any(|arg| arg.ends_with(":auto") && arg.starts_with("usb=")));
}

#[test]
fn max_performance_keeps_every_device_awake_and_runs_the_gpu_high() {
    let root = FakeRoot::laptop();

    let args = args(Stop::MaxPerformance, &Settings::default(), &root, Some(true));

    assert!(args.contains(&"gpu-dpm=card1:high".to_owned()));
    assert!(args.contains(&"usb=1-1:on".to_owned()));
    assert!(args.contains(&"aspm=performance".to_owned()));
    assert!(args.contains(&"wifi=wlp99s0:off".to_owned()));
}

#[test]
fn leaving_super_saver_lifts_the_clock_cap_to_the_cpus_top() {
    let root = FakeRoot::laptop();
    for policy in ["policy0", "policy1", "policy10"] {
        root.write(
            &format!("sys/devices/system/cpu/cpufreq/{policy}/scaling_max_freq"),
            "1600000",
        );
        root.write(
            &format!("sys/devices/system/cpu/cpufreq/{policy}/cpuinfo_max_freq"),
            "5157895",
        );
    }

    let args = args(Stop::Balanced, &Settings::default(), &root, Some(true));

    assert_eq!(args.first().map(String::as_str), Some("cpu-max=max"));
}

#[test]
fn rows_already_in_place_are_not_written_again() {
    let root = FakeRoot::laptop();

    let args = args(Stop::Balanced, &Settings::default(), &root, Some(true));

    assert_eq!(args, ["usb=1-3:auto"]);
}

#[test]
fn a_machine_without_the_knobs_gets_no_writes() {
    let root = FakeRoot::empty();

    assert_eq!(
        args(Stop::SuperSaver, &Settings::default(), &root, None),
        Vec::<String>::new()
    );
}

#[test]
fn the_charge_limit_goes_to_tuxedos_profile_or_the_kernels_threshold() {
    let root = FakeRoot::laptop();
    let mut settings = Settings::default();
    settings.charge_limit = ChargeLimit::Eighty;

    assert!(args(Stop::Balanced, &settings, &root, Some(true)).contains(&"charge-profile=stationary".to_owned()));

    root.remove("sys/devices/platform/tuxedo_keyboard");
    root.write("sys/class/power_supply/BAT0/charge_control_end_threshold", "100");
    assert!(args(Stop::Balanced, &settings, &root, Some(true)).contains(&"charge-end=BAT0:80".to_owned()));
}

#[test]
fn panel_power_saving_off_writes_zero_in_the_savers() {
    let root = FakeRoot::laptop();
    root.write("sys/class/drm/card1-eDP-1/amdgpu/panel_power_savings", "3");
    let mut settings = Settings::default();
    settings.panel_power_saving = false;

    assert!(args(Stop::SuperSaver, &settings, &root, Some(true)).contains(&"panel-power=card1-eDP-1:0".to_owned()));
}

/// Runs the root helper on `root`'s fake `/sys`, as the service runs it under pkexec; its exit
/// status and what it said on stderr.
fn helper(root: &FakeRoot, args: &[String]) -> (bool, String) {
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../system/files/power/kubaos-power-write"
    );
    let output = std::process::Command::new("bash")
        .arg(script)
        .args(args)
        .env("KUBAOS_POWER_ROOT", root.dir.path())
        .output()
        .expect("run the helper");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn the_helper_puts_super_saver_in_place_and_then_nothing_is_left_to_write() {
    let root = FakeRoot::laptop();
    let writes = args(Stop::SuperSaver, &Settings::default(), &root, Some(true));

    let (ok, said) = helper(&root, &writes);

    assert!(ok, "{said}");
    assert_eq!(
        root.read("sys/devices/system/cpu/cpufreq/policy10/scaling_max_freq"),
        "1600000"
    );
    assert_eq!(
        root.read("sys/class/drm/card1/device/power_dpm_force_performance_level"),
        "low"
    );
    assert_eq!(root.read("sys/class/drm/card1-eDP-1/amdgpu/panel_power_savings"), "3");
    assert_eq!(root.read("sys/module/pcie_aspm/parameters/policy"), "powersupersave");
    assert_eq!(root.read("sys/bus/usb/devices/1-3/power/control"), "auto");
    assert_eq!(root.read("sys/bus/usb/devices/7-1.1.3/power/control"), "on");
    // The kernel shows the policies again with the new one in brackets; a plain file can't.
    root.write(
        "sys/module/pcie_aspm/parameters/policy",
        "default performance powersave [powersupersave]",
    );
    assert_eq!(
        args(Stop::SuperSaver, &Settings::default(), &root, Some(true)),
        Vec::<String>::new()
    );
}

#[test]
fn the_helper_lifts_the_cap_to_each_policys_top() {
    let root = FakeRoot::laptop();
    root.write("sys/devices/system/cpu/cpufreq/policy0/scaling_max_freq", "1600000");
    root.write("sys/devices/system/cpu/cpufreq/policy0/cpuinfo_max_freq", "5157895");

    let (ok, said) = helper(&root, &["cpu-max=max".to_owned()]);

    assert!(ok, "{said}");
    assert_eq!(
        root.read("sys/devices/system/cpu/cpufreq/policy0/scaling_max_freq"),
        "5157895"
    );
    assert_eq!(
        root.read("sys/devices/system/cpu/cpufreq/policy1/scaling_max_freq"),
        "2000000"
    );
}

#[test]
fn the_helper_never_lets_an_input_device_autosuspend_whatever_it_is_asked() {
    let root = FakeRoot::laptop();

    let (ok, said) = helper(&root, &["usb=7-1.1.3:auto".to_owned(), "usb=3-2:auto".to_owned()]);

    assert!(!ok);
    assert!(
        said.contains("USB device 7-1.1.3 is an input, security or audio device"),
        "{said}"
    );
    assert!(said.contains("USB device 3-2 is an input"), "{said}");
    assert_eq!(root.read("sys/bus/usb/devices/7-1.1.3/power/control"), "on");
    assert_eq!(root.read("sys/bus/usb/devices/3-2/power/control"), "on");
}

#[test]
fn the_helper_refuses_what_isnt_a_knob_or_a_value_and_still_writes_the_rest() {
    let root = FakeRoot::laptop();
    let asked = [
        "gpu-dpm=card1/../../x:low",
        "panel-power=card1-HDMI-A-1:3",
        "aspm=turbo",
        "cpu-max=99",
        "charge-profile=stationary",
        "fans=max",
    ]
    .map(str::to_owned);

    let (ok, said) = helper(&root, &asked);

    assert!(!ok);
    for refused in [
        "gpu-dpm=card1/../../x:low",
        "panel-power=card1-HDMI-A-1:3",
        "aspm=turbo",
        "cpu-max=99",
        "fans",
    ] {
        assert!(said.contains(refused), "{refused}: {said}");
    }
    assert_eq!(
        root.read("sys/devices/platform/tuxedo_keyboard/charging_profile/charging_profile"),
        "stationary"
    );
    assert_eq!(
        root.read("sys/module/pcie_aspm/parameters/policy"),
        "[default] performance powersave powersupersave"
    );
}

#[test]
fn the_helper_without_arguments_is_a_usage_error() {
    let root = FakeRoot::laptop();
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../system/files/power/kubaos-power-write"
    );

    let status = std::process::Command::new("bash")
        .arg(script)
        .env("KUBAOS_POWER_ROOT", root.dir.path())
        .output()
        .expect("run the helper")
        .status;

    assert_eq!(status.code(), Some(2));
}

#[test]
fn a_cap_above_the_cpus_top_counts_as_in_place() {
    let root = FakeRoot::laptop();
    let settings = Settings::parse("[super-saver]\ncpu_max_mhz = 3000\n").expect("parses");

    let args = args(Stop::SuperSaver, &settings, &root, Some(true));

    assert!(!args.iter().any(|arg| arg.starts_with("cpu-max")), "{args:?}");
}

#[test]
fn network_adapters_and_printers_stay_awake_in_every_stop() {
    let root = FakeRoot::laptop();
    root.usb("4-2", "04f9", "0042", &["07"], "on", "Printer");
    root.usb("4-3", "0b95", "1790", &["02", "0a"], "on", "AX88179 Ethernet");

    let args = args(Stop::SuperSaver, &Settings::default(), &root, Some(true));

    for device in ["8-1.1.2", "4-2", "4-3"] {
        assert!(
            !args.iter().any(|arg| arg.starts_with(&format!("usb={device}:"))),
            "{device}: {args:?}"
        );
    }
}

#[test]
fn a_device_plugged_in_after_the_stop_gets_the_stops_autosuspend() {
    let root = FakeRoot::laptop();
    let settled = args(Stop::PowerSaver, &Settings::default(), &root, Some(true));
    assert!(helper(&root, &settled).0);

    root.usb("2-1", "0781", "5583", &["08"], "on", "Ultra Fit");

    assert!(args(Stop::PowerSaver, &Settings::default(), &root, Some(true)).contains(&"usb=2-1:auto".to_owned()));
}

#[test]
fn max_performance_puts_the_gpu_back_on_high_after_ppd_sets_it_to_auto() {
    let root = FakeRoot::laptop();
    root.write("sys/class/drm/card1/device/power_dpm_force_performance_level", "high");
    assert!(
        !args(Stop::MaxPerformance, &Settings::default(), &root, Some(false))
            .iter()
            .any(|arg| arg.starts_with("gpu-dpm"))
    );

    root.write("sys/class/drm/card1/device/power_dpm_force_performance_level", "auto");

    assert!(
        args(Stop::MaxPerformance, &Settings::default(), &root, Some(false)).contains(&"gpu-dpm=card1:high".to_owned())
    );
}

#[test]
fn a_knob_the_helper_cant_read_fails_alone_and_the_rest_are_written() {
    let root = FakeRoot::laptop();
    root.remove("sys/class/drm/card1/device/power_dpm_force_performance_level");
    root.dir_at("sys/class/drm/card1/device/power_dpm_force_performance_level");

    let (ok, said) = helper(&root, &["gpu-dpm=card1:low".to_owned(), "aspm=powersave".to_owned()]);

    assert!(!ok);
    assert!(said.contains("couldn't read"), "{said}");
    assert_eq!(root.read("sys/module/pcie_aspm/parameters/policy"), "powersave");
}
