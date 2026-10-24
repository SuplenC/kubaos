use std::path::PathBuf;

use super::Block;
use super::Drive;
use super::MountedDrive;
use super::Skip;
use super::Step;
use super::Verdict;
use super::eject_plan;
use super::find_drive;
use super::mounted_drives;
use super::verdict;

const STICK: &str = "/org/freedesktop/UDisks2/drives/SanDisk_Ultra_1234";
const NVME: &str = "/org/freedesktop/UDisks2/drives/Samsung_990_PRO_5678";

fn stick() -> Drive {
    Drive {
        path: STICK.to_owned(),
        vendor: "SanDisk".to_owned(),
        model: "Ultra".to_owned(),
        removable: true,
        ejectable: true,
        can_power_off: true,
    }
}

fn nvme() -> Drive {
    Drive {
        path: NVME.to_owned(),
        vendor: String::new(),
        model: "Samsung SSD 990 PRO".to_owned(),
        removable: false,
        ejectable: false,
        can_power_off: false,
    }
}

fn partition(name: &str, drive: &str) -> Block {
    Block {
        path: format!("/org/freedesktop/UDisks2/block_devices/{name}"),
        device: format!("/dev/{name}"),
        drive: Some(drive.to_owned()),
        usage: "filesystem".to_owned(),
        filesystem: true,
        ..Block::default()
    }
}

fn mounted(mut block: Block, at: &str) -> Block {
    block.mount_points.push(PathBuf::from(at));
    block
}

#[test]
fn a_new_filesystem_on_a_usb_stick_is_mounted() {
    let block = partition("sdb1", STICK);

    assert_eq!(verdict(&block, Some(&stick())), Verdict::Mount);
}

#[test]
fn an_internal_disk_is_never_mounted() {
    let mut block = partition("nvme0n1p3", NVME);
    block.hint_system = true;

    assert_eq!(verdict(&block, Some(&nvme())), Verdict::Skip(Skip::Internal));
}

#[test]
fn a_block_udisks_calls_system_is_internal_even_on_a_removable_drive() {
    let mut block = partition("sdb1", STICK);
    block.hint_system = true;

    assert_eq!(verdict(&block, Some(&stick())), Verdict::Skip(Skip::Internal));
}

#[test]
fn a_loop_device_with_no_drive_is_left_alone() {
    let mut block = partition("loop0", STICK);
    block.drive = None;

    assert_eq!(verdict(&block, None), Verdict::Skip(Skip::Internal));
}

#[test]
fn a_filesystem_already_mounted_is_left_as_it_is() {
    let block = mounted(partition("sdb1", STICK), "/media/kuba/STICK");

    assert_eq!(verdict(&block, Some(&stick())), Verdict::Skip(Skip::AlreadyMounted));
}

#[test]
fn a_partition_table_or_swap_has_nothing_to_mount() {
    let mut table = partition("sdb", STICK);
    table.usage = String::new();
    table.filesystem = false;
    let mut swap = partition("sdb2", STICK);
    swap.usage = "other".to_owned();
    swap.filesystem = false;

    assert_eq!(verdict(&table, Some(&stick())), Verdict::Skip(Skip::NoFilesystem));
    assert_eq!(verdict(&swap, Some(&stick())), Verdict::Skip(Skip::NoFilesystem));
}

#[test]
fn a_locked_luks_volume_on_a_stick_is_left_to_the_user() {
    let mut block = partition("sdb1", STICK);
    block.usage = "crypto".to_owned();
    block.filesystem = false;

    assert_eq!(verdict(&block, Some(&stick())), Verdict::Encrypted);
}

#[test]
fn a_luks_volume_on_an_internal_disk_is_not_even_mentioned() {
    let mut block = partition("nvme0n1p3", NVME);
    block.usage = "crypto".to_owned();
    block.hint_system = true;

    assert_eq!(verdict(&block, Some(&nvme())), Verdict::Skip(Skip::Internal));
}

#[test]
fn an_unlocked_volume_is_mounted_by_whoever_unlocked_it() {
    let mut cleartext = partition("dm-0", STICK);
    cleartext.drive = None;
    cleartext.crypto_backing = Some("/org/freedesktop/UDisks2/block_devices/sdb1".to_owned());

    assert_eq!(verdict(&cleartext, Some(&stick())), Verdict::Skip(Skip::Unlocked));
}

#[test]
fn hints_and_fstab_entries_are_respected() {
    let mut ignored = partition("sdb1", STICK);
    ignored.hint_ignore = true;
    let mut configured = partition("sdb2", STICK);
    configured.configured = true;

    assert_eq!(verdict(&ignored, Some(&stick())), Verdict::Skip(Skip::Ignored));
    assert_eq!(verdict(&configured, Some(&stick())), Verdict::Skip(Skip::Configured));
}

#[test]
fn only_removable_drives_with_a_mount_are_offered_for_eject() {
    let mut label = mounted(partition("sdb1", STICK), "/media/kuba/PHOTOS");
    label.label = "PHOTOS".to_owned();
    let blocks = [
        label,
        partition("sdb2", STICK),
        mounted(partition("sdb3", STICK), "/media/kuba/1234-ABCD"),
        mounted(partition("nvme0n1p2", NVME), "/"),
    ];

    let drives = mounted_drives(&blocks, &[stick(), nvme()]);

    assert_eq!(
        drives,
        vec![MountedDrive {
            drive: STICK.to_owned(),
            name: "PHOTOS".to_owned(),
            mount_points: vec![PathBuf::from("/media/kuba/PHOTOS"), PathBuf::from("/media/kuba/1234-ABCD")],
        }]
    );
}

#[test]
fn a_drive_with_no_label_is_named_by_its_vendor_and_model() {
    let blocks = [mounted(partition("sdb1", STICK), "/media/kuba/1234-ABCD")];

    let drives = mounted_drives(&blocks, &[stick()]);

    assert_eq!(drives[0].name, "SanDisk Ultra");
}

#[test]
fn a_removable_drive_with_nothing_mounted_is_not_offered() {
    let blocks = [partition("sdb1", STICK)];

    assert!(mounted_drives(&blocks, &[stick()]).is_empty());
}

#[test]
fn an_unlocked_volume_counts_for_the_drive_its_locked_side_is_on() {
    let mut locked = partition("sdb1", STICK);
    locked.usage = "crypto".to_owned();
    let mut cleartext = mounted(partition("dm-0", STICK), "/media/kuba/SECRET");
    cleartext.drive = None;
    cleartext.crypto_backing = Some(locked.path.clone());
    let blocks = [locked, cleartext];

    let drives = mounted_drives(&blocks, &[stick()]);

    assert_eq!(drives.len(), 1);
    assert_eq!(drives[0].mount_points, vec![PathBuf::from("/media/kuba/SECRET")]);
}

#[test]
fn eject_finds_the_drive_by_any_name_the_user_has_for_it() {
    let blocks = [mounted(partition("sdb1", STICK), "/media/kuba/STICK")];
    let drives = [stick(), nvme()];

    for target in [STICK, "/org/freedesktop/UDisks2/block_devices/sdb1", "/dev/sdb1", "/media/kuba/STICK"] {
        assert_eq!(find_drive(target, &blocks, &drives).map(|drive| drive.path.as_str()), Some(STICK), "{target}");
    }
    assert_eq!(find_drive("/media/kuba/OTHER", &blocks, &drives), None);
}

#[test]
fn ejecting_a_stick_unmounts_each_filesystem_then_powers_it_off() {
    let blocks = [
        mounted(partition("sdb1", STICK), "/media/kuba/A"),
        partition("sdb2", STICK),
        mounted(partition("sdb3", STICK), "/media/kuba/B"),
        mounted(partition("nvme0n1p2", NVME), "/"),
    ];

    let plan = eject_plan(&stick(), &blocks);

    assert_eq!(
        plan,
        vec![
            Step::Unmount("/org/freedesktop/UDisks2/block_devices/sdb1".to_owned()),
            Step::Unmount("/org/freedesktop/UDisks2/block_devices/sdb3".to_owned()),
            Step::PowerOff(STICK.to_owned()),
        ]
    );
}

#[test]
fn ejecting_a_card_reader_ejects_the_card_when_it_cannot_power_off() {
    let mut reader = stick();
    reader.can_power_off = false;
    let blocks = [mounted(partition("mmcblk0p1", STICK), "/media/kuba/CARD")];

    let plan = eject_plan(&reader, &blocks);

    assert_eq!(plan.last(), Some(&Step::Eject(STICK.to_owned())));
}

#[test]
fn ejecting_an_unlocked_volume_unmounts_it_then_locks_it() {
    let mut locked = partition("sdb1", STICK);
    locked.usage = "crypto".to_owned();
    let mut cleartext = mounted(partition("dm-0", STICK), "/media/kuba/SECRET");
    cleartext.drive = None;
    cleartext.crypto_backing = Some(locked.path.clone());
    let blocks = [locked, cleartext];

    let plan = eject_plan(&stick(), &blocks);

    assert_eq!(
        plan,
        vec![
            Step::Unmount("/org/freedesktop/UDisks2/block_devices/dm-0".to_owned()),
            Step::Lock("/org/freedesktop/UDisks2/block_devices/sdb1".to_owned()),
            Step::PowerOff(STICK.to_owned()),
        ]
    );
}
