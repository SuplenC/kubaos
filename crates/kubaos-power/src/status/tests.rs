use super::*;
use crate::fake::FakeRoot;
use crate::stop::Stop;

fn live() -> Live {
    Live {
        profile: Some("power-saver".to_owned()),
        source: Some(Source::Battery),
        panel: Some((120.0, false)),
        wifi: vec![("wlp99s0".to_owned(), Some(true))],
        indexer_paused: Some(false),
        scheduler_unit: None,
        scheduler_exit: None,
    }
}

fn find<'a>(rows: &'a [Row], name: &str) -> &'a Row {
    rows.iter().find(|row| row.name == name).expect("a row")
}

#[test]
fn power_saver_on_the_laptop_today_shows_what_ppd_did_and_what_is_missing() {
    let root = FakeRoot::laptop();
    let settings = Settings::default();
    let plan = Plan::for_stop(Stop::PowerSaver, &settings);

    let rows = rows(&plan, &settings, &root.machine(), &live());

    assert_eq!(find(&rows, "Power profile (power-profiles-daemon)").agrees, Some(true));
    assert_eq!(find(&rows, "CPU boost").agrees, Some(true));
    assert_eq!(find(&rows, "Firmware profile").agrees, Some(true));
    assert_eq!(find(&rows, "GPU (amdgpu DPM)").agrees, Some(false));
    assert_eq!(find(&rows, "Panel power saving").agrees, Some(false));
    assert_eq!(find(&rows, "PCIe ASPM").agrees, Some(false));
    let panel = find(&rows, "Laptop panel");
    assert_eq!(panel.now.as_deref(), Some("120 Hz"));
    assert_eq!(panel.agrees, Some(false));
    assert_eq!(find(&rows, "Background work").agrees, Some(false));
    assert_eq!(find(&rows, "Charge limit").agrees, Some(true));
    let scheduler = find(&rows, "CPU scheduler");
    assert_eq!(scheduler.now.as_deref(), Some("Linux's own (EEVDF)"));
    assert_eq!(scheduler.agrees, Some(true));
}

#[test]
fn the_scheduler_row_says_lavd_fell_back_and_why() {
    let root = FakeRoot::laptop();
    root.install_lavd();
    let settings = Settings::parse("scheduler = \"lavd\"\n").expect("parses");
    let plan = Plan::for_stop(Stop::PowerSaver, &settings);
    let mut live = live();
    live.scheduler_unit = Some(crate::scheduler::UnitState::parse(
        "LoadState=loaded\nActiveState=failed\nSubState=failed\nResult=start-limit-hit\nNRestarts=2\n",
    ));
    live.scheduler_exit = Some("EXIT: runtime error (watchdog)".to_owned());

    let rows = rows(&plan, &settings, &root.machine(), &live);
    let text = text("Power: Power Saver", &rows);

    let scheduler = find(&rows, "CPU scheduler");
    assert_eq!(scheduler.wanted, "LAVD, powersave");
    assert_eq!(scheduler.agrees, Some(false));
    assert!(text.contains("LAVD crashed until systemd stopped restarting it (EXIT: runtime error (watchdog))"));
}

#[test]
fn rows_without_a_knob_say_so() {
    let root = FakeRoot::empty();
    let settings = Settings::default();
    let plan = Plan::for_stop(Stop::Balanced, &settings);

    let rows = rows(&plan, &settings, &root.machine(), &Live::default());
    let text = text("Power: Balanced", &rows);

    for name in [
        "CPU boost",
        "GPU (amdgpu DPM)",
        "PCIe ASPM",
        "Charge limit",
        "USB autosuspend",
    ] {
        assert_eq!(find(&rows, name).now, None, "{name}");
    }
    assert!(text.contains("not on this machine"));
}

#[test]
fn balanced_on_battery_expects_ppds_balance_power() {
    let root = FakeRoot::laptop();
    let settings = Settings::default();
    let plan = Plan::for_stop(Stop::Balanced, &settings);

    let rows = rows(&plan, &settings, &root.machine(), &live());

    assert_eq!(find(&rows, "CPU energy preference").wanted, "balance_power");
}

#[test]
fn the_panel_row_says_when_60_hz_waits_for_a_reboot() {
    let root = FakeRoot::laptop();
    root.write("sys/module/amdgpu/parameters/freesync_video", "0");
    let settings = Settings::default();
    let plan = Plan::for_stop(Stop::SuperSaver, &settings);

    let rows = rows(&plan, &settings, &root.machine(), &live());

    assert!(
        find(&rows, "Laptop panel")
            .wanted
            .starts_with("native rate with VRR (60 Hz waits")
    );
}

#[test]
fn the_usb_row_names_the_devices_kept_awake() {
    let root = FakeRoot::laptop();
    let settings = Settings::default();
    let plan = Plan::for_stop(Stop::SuperSaver, &settings);

    let rows = rows(&plan, &settings, &root.machine(), &live());

    let usb = find(&rows, "USB autosuspend");
    assert_eq!(
        usb.now.as_deref(),
        Some("1 of 5 devices; never: YubiKey FIDO+CCID, USB Receiver, USB 10/100/1000 LAN")
    );
    assert_eq!(usb.agrees, Some(false));
}

#[test]
fn a_disagreement_is_marked_in_the_text() {
    let rows = vec![Row {
        name: "PCIe ASPM",
        wanted: "powersupersave".to_owned(),
        now: Some("default".to_owned()),
        agrees: Some(false),
    }];

    assert_eq!(
        text("Power: Super Saver", &rows),
        "Power: Super Saver\n\nPCIe ASPM  powersupersave  (now default  ✗)\n"
    );
}
