use std::fs;
use std::path::Path;

use super::DiskSet;
use super::parse_diskstats;
use super::parse_mounts;
use super::snapshots;
use super::watched;

const MOUNTS: &str = "\
/dev/mapper/root / btrfs rw,noatime,compress=zstd:1,subvolid=256,subvol=/@ 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
tmpfs /run tmpfs rw,nosuid,nodev,size=3266512k,mode=755 0 0
/dev/mapper/root /home btrfs rw,noatime,compress=zstd:1,subvolid=257,subvol=/@home 0 0
/dev/mapper/root /.snapshots btrfs rw,noatime,subvolid=258,subvol=/@snapshots 0 0
/dev/nvme0n1p1 /boot/efi vfat rw,relatime 0 0
/dev/sda1 /media/kuba/MY\\040STICK exfat rw,nosuid,nodev,relatime 0 0
/dev/sdb1 /mnt/backup ext4 rw,relatime 0 0
";

#[test]
fn mounts_are_read_with_their_escapes_undone() {
    let mounts = parse_mounts(MOUNTS);

    assert_eq!(mounts.len(), 8);
    assert_eq!(mounts[6].point, "/media/kuba/MY STICK");
    assert_eq!(mounts[0].subvolume_id(), Some(256));
}

#[test]
fn root_and_removable_drives_are_watched_and_home_on_the_same_disk_is_not() {
    let mounts = parse_mounts(MOUNTS);

    let watched = watched(&mounts);

    let points: Vec<&str> = watched.iter().map(|disk| disk.point.as_str()).collect();
    assert_eq!(points, ["/", "/media/kuba/MY STICK"]);
    assert!(!watched[0].removable);
    assert!(watched[1].removable);
    assert_eq!(watched[1].label, "MY STICK");
}

#[test]
fn home_on_its_own_disk_is_watched() {
    let mounts = parse_mounts("/dev/nvme0n1p2 / ext4 rw 0 0\n/dev/nvme1n1p1 /home ext4 rw 0 0\n");

    let points: Vec<String> = watched(&mounts).into_iter().map(|disk| disk.point).collect();

    assert_eq!(points, ["/", "/home"]);
}

#[test]
fn disk_traffic_counts_whole_disks_only() {
    let text = "\
 259       0 nvme0n1 1000 0 2048 0 500 0 4096 0 0 0 0
 259       1 nvme0n1p1 10 0 1024 0 5 0 2048 0 0 0 0
   7       0 loop0 10 0 999999 0 0 0 0 0 0 0 0
 253       0 dm-0 900 0 2000 0 400 0 4000 0 0 0 0
";
    let set = DiskSet {
        names: vec!["nvme0n1".to_owned()],
    };

    assert_eq!(parse_diskstats(text, &set), (2048 * 512, 4096 * 512));
}

#[test]
fn the_disk_set_leaves_out_loops_and_mapped_devices() {
    let root = tempfile::tempdir().expect("a temp dir");
    for name in ["nvme0n1", "loop0", "dm-0", "zram0", "sda"] {
        fs::create_dir_all(root.path().join("sys/block").join(name)).expect("make a block device");
    }

    assert_eq!(DiskSet::discover(root.path()).names, ["nvme0n1", "sda"]);
}

fn qgroup(root: &Path, uuid: &str, id: &str, exclusive: u64) {
    let dir = root.join(format!("sys/fs/btrfs/{uuid}/qgroups/{id}"));
    fs::create_dir_all(&dir).expect("make the qgroup");
    fs::write(dir.join("exclusive"), format!("{exclusive}\n")).expect("write its size");
}

#[test]
fn snapshot_space_is_what_the_unmounted_subvolumes_hold_alone() {
    let root = tempfile::tempdir().expect("a temp dir");
    let uuid = "0f1e2d3c-aaaa-bbbb-cccc-000000000000";
    fs::create_dir_all(root.path().join("sys/fs/btrfs/features")).expect("make the features folder");
    qgroup(root.path(), uuid, "0_5", 16_384);
    qgroup(root.path(), uuid, "0_256", 9_000_000_000);
    qgroup(root.path(), uuid, "0_257", 50_000_000_000);
    qgroup(root.path(), uuid, "0_300", 1_000_000);
    qgroup(root.path(), uuid, "0_301", 2_000_000);
    for number in ["1", "2", "info.xml"] {
        fs::create_dir_all(root.path().join(".snapshots").join(number)).expect("make a snapshot");
    }
    let mounts = parse_mounts(MOUNTS);

    let found = snapshots(root.path(), &mounts).expect("snapshots");

    assert_eq!(found.bytes, Some(3_000_000));
    assert_eq!(found.count, Some(2));
}

#[test]
fn snappers_own_qgroup_is_read_when_it_has_one() {
    let root = tempfile::tempdir().expect("a temp dir");
    let uuid = "0f1e2d3c-aaaa-bbbb-cccc-000000000000";
    qgroup(root.path(), uuid, "0_300", 1_000_000);
    qgroup(root.path(), uuid, "1_0", 7_000_000);

    let found = snapshots(root.path(), &parse_mounts(MOUNTS)).expect("snapshots");

    assert_eq!(found.bytes, Some(7_000_000));
}

#[test]
fn without_quotas_the_space_is_unknown() {
    let root = tempfile::tempdir().expect("a temp dir");
    fs::create_dir_all(
        root.path()
            .join("sys/fs/btrfs/0f1e2d3c-aaaa-bbbb-cccc-000000000000/devices"),
    )
    .expect("make the filesystem");

    let found = snapshots(root.path(), &parse_mounts(MOUNTS)).expect("snapshots");

    assert_eq!(found.bytes, None);
}

#[test]
fn a_root_that_is_not_btrfs_has_no_snapshots() {
    let mounts = parse_mounts("/dev/nvme0n1p2 / ext4 rw 0 0\n");

    assert!(snapshots(Path::new("/nonexistent"), &mounts).is_none());
}
