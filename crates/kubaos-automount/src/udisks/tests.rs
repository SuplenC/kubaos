use std::collections::HashMap;
use std::path::PathBuf;

use zbus::zvariant::ObjectPath;
use zbus::zvariant::OwnedValue;
use zbus::zvariant::Value;

use super::BLOCK;
use super::DRIVE;
use super::FILESYSTEM;
use super::Interfaces;
use super::Snapshot;

const STICK: &str = "/org/freedesktop/UDisks2/drives/SanDisk_Ultra_1234";

fn owned(value: Value<'_>) -> OwnedValue {
    OwnedValue::try_from(value).unwrap()
}

fn path(path: &str) -> OwnedValue {
    owned(Value::from(ObjectPath::try_from(path).unwrap()))
}

fn properties(pairs: Vec<(&str, OwnedValue)>) -> HashMap<String, OwnedValue> {
    pairs.into_iter().map(|(name, value)| (name.to_owned(), value)).collect()
}

fn stick_partition(mounts: Vec<&str>, configuration: Vec<(String, HashMap<String, Value<'static>>)>) -> Interfaces {
    let mounts: Vec<Vec<u8>> = mounts
        .into_iter()
        .map(|mount| {
            let mut bytes = mount.as_bytes().to_vec();
            bytes.push(0);
            bytes
        })
        .collect();
    let mut interfaces = Interfaces::new();
    interfaces.insert(
        BLOCK.to_owned(),
        properties(vec![
            ("Device", owned(Value::from(b"/dev/sdb1\0".to_vec()))),
            ("Drive", path(STICK)),
            ("IdUsage", owned(Value::from("filesystem"))),
            ("IdLabel", owned(Value::from("PHOTOS"))),
            ("HintIgnore", owned(Value::from(false))),
            ("HintSystem", owned(Value::from(false))),
            ("CryptoBackingDevice", path("/")),
            ("Configuration", owned(Value::from(configuration))),
        ]),
    );
    interfaces.insert(
        FILESYSTEM.to_owned(),
        properties(vec![("MountPoints", owned(Value::from(mounts)))]),
    );
    interfaces
}

fn stick() -> Interfaces {
    let mut interfaces = Interfaces::new();
    interfaces.insert(
        DRIVE.to_owned(),
        properties(vec![
            ("Vendor", owned(Value::from("SanDisk"))),
            ("Model", owned(Value::from("Ultra"))),
            ("Removable", owned(Value::from(true))),
            ("MediaRemovable", owned(Value::from(false))),
            ("Ejectable", owned(Value::from(true))),
            ("CanPowerOff", owned(Value::from(true))),
        ]),
    );
    interfaces
}

#[test]
fn a_mounted_partition_and_its_drive_are_read_from_udisks_properties() {
    let objects = vec![
        ("/org/freedesktop/UDisks2/block_devices/sdb1".to_owned(), stick_partition(vec!["/media/kuba/PHOTOS"], vec![])),
        (STICK.to_owned(), stick()),
    ];

    let snapshot = Snapshot::from_objects(objects);

    let block = snapshot.block("/org/freedesktop/UDisks2/block_devices/sdb1").unwrap();
    assert_eq!(block.device, "/dev/sdb1");
    assert_eq!(block.drive.as_deref(), Some(STICK));
    assert_eq!(block.usage, "filesystem");
    assert_eq!(block.label, "PHOTOS");
    assert!(block.filesystem);
    assert_eq!(block.mount_points, vec![PathBuf::from("/media/kuba/PHOTOS")]);
    assert_eq!(block.crypto_backing, None);
    assert!(!block.configured);
    let drive = snapshot.drive(STICK).unwrap();
    assert!(drive.removable && drive.ejectable && drive.can_power_off);
    assert_eq!(drive.model, "Ultra");
}

#[test]
fn an_fstab_entry_marks_the_block_configured() {
    let fstab = vec![("fstab".to_owned(), HashMap::new())];
    let objects = vec![("/org/freedesktop/UDisks2/block_devices/sdb1".to_owned(), stick_partition(vec![], fstab))];

    let snapshot = Snapshot::from_objects(objects);

    let block = &snapshot.blocks[0];
    assert!(block.configured);
    assert!(block.mount_points.is_empty());
}
