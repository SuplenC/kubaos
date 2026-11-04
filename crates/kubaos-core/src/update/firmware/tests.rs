use super::Firmware;
use super::FirmwareOutcome;
use super::FirmwareUpdate;
use super::NO_FIRMWARE;
use super::Setting;
use super::parse_updates;
use crate::apt::Switch;

/// `fwupdmgr get-updates --json` on a laptop whose Secure Boot dbx waits, trimmed.
const REBOOT: &str = include_str!("reboot.json");
/// A dock and a receiver that update at once, and system firmware that needs a reboot.
const MIXED: &str = include_str!("mixed.json");

fn update(device: &str, id: &str, from: &str, to: &str, reboot: bool) -> FirmwareUpdate {
    FirmwareUpdate {
        device: device.to_owned(),
        id: id.to_owned(),
        from: from.to_owned(),
        to: to.to_owned(),
        reboot,
    }
}

#[test]
fn nothing_waiting_is_an_empty_list_or_no_output() {
    assert_eq!(parse_updates("{\"Devices\": []}").unwrap(), []);
    assert_eq!(parse_updates("{}").unwrap(), []);
    assert_eq!(parse_updates("\n").unwrap(), []);
}

#[test]
fn a_device_that_needs_a_reboot_says_so_and_takes_its_newest_release() {
    assert_eq!(
        parse_updates(REBOOT).unwrap(),
        [update(
            "UEFI dbx",
            "362301da643102b9f38477387e2193e57abaa590",
            "20250507",
            "20260707",
            true
        )]
    );
}

#[test]
fn devices_without_a_reboot_flag_update_at_once() {
    let updates = parse_updates(MIXED).unwrap();

    assert_eq!(
        updates,
        [
            update(
                "USB-C Dock",
                "4d3b0f2c9a1e7b6a5c8d9e0f1a2b3c4d5e6f7a8b",
                "1.2.3",
                "1.3.0",
                false
            ),
            update(
                "Unifying Receiver",
                "9f8e7d6c5b4a39281706f5e4d3c2b1a098765432",
                "",
                "RQR12.11_B0032",
                false
            ),
            update(
                "System Firmware",
                "a45df35ac0e948ee180fe216a5f703f32dda163f",
                "0.1.2",
                "0.1.4",
                true
            ),
        ]
    );
    assert_eq!(updates[1].describe(), "Unifying Receiver → RQR12.11_B0032");
    assert_eq!(updates[2].describe(), "System Firmware 0.1.2 → 0.1.4");
}

#[test]
fn a_device_without_releases_waits_for_nothing() {
    let json = r#"{"Devices": [{"Name": "Touchpad", "DeviceId": "ab", "Flags": ["updatable"]}]}"#;

    assert_eq!(parse_updates(json).unwrap(), []);
}

#[test]
fn output_that_isnt_fwupds_list_is_an_error() {
    assert!(parse_updates("No updates available").is_err());
    assert!(parse_updates(r#"{"Devices": [{"Name": "no id"}]}"#).is_err());
}

#[test]
fn the_result_says_what_happened_in_a_few_words() {
    let dock = update("Dock", "a", "1", "2", false);
    let bios = update("BIOS", "b", "1", "2", true);

    assert_eq!(Firmware::default().detail(), "nothing waiting");
    assert!(Firmware::default().is_empty());
    let mut done = Firmware::default();
    done.push(dock.clone(), FirmwareOutcome::Failed);
    done.push(bios.clone(), FirmwareOutcome::Staged);
    done.push(dock.clone(), FirmwareOutcome::Applied);
    assert_eq!(done.detail(), "1 applied, 1 staged for the reboot, 1 failed");
    assert_eq!(done.with(FirmwareOutcome::Staged).collect::<Vec<_>>(), [&bios]);
    let mut waiting = Firmware::default();
    waiting.push(bios, FirmwareOutcome::Waiting);
    assert_eq!(waiting.detail(), "1 waiting for a reboot");
}

#[test]
fn the_result_is_one_object_a_device_with_its_outcome() {
    let mut waiting = Firmware::default();
    waiting.push(update("BIOS", "b", "1", "2", true), FirmwareOutcome::Waiting);

    let json = serde_json::to_string(&waiting).unwrap();

    assert_eq!(
        json,
        r#"[{"device":"BIOS","id":"b","from":"1","to":"2","reboot":true,"outcome":"waiting"}]"#
    );
    assert_eq!(serde_json::from_str::<Firmware>(&json).unwrap(), waiting);
}

#[test]
fn firmware_updates_are_on_until_the_flag_file_turns_them_off() {
    let layer = tempfile::tempdir().unwrap();
    let setting = Setting::in_layer(layer.path());
    assert_eq!(setting.path(), layer.path().join("desktop/update").join(NO_FIRMWARE));
    assert!(setting.is_on());

    setting.set(Switch::Off).unwrap();
    assert!(!setting.is_on());
    setting.set(Switch::Off).unwrap();
    assert!(!setting.is_on());

    setting.set(Switch::On).unwrap();
    assert!(setting.is_on());
    setting.set(Switch::On).unwrap();
    assert!(setting.is_on());
}
