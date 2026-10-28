use std::fs;
use std::path::PathBuf;

use super::render;
use super::write;
use crate::block::MountedDrive;

fn drive(name: &str, mounts: &[&str]) -> MountedDrive {
    MountedDrive {
        drive: "/org/freedesktop/UDisks2/drives/SanDisk_Ultra_1234".to_owned(),
        name: name.to_owned(),
        mount_points: mounts.iter().map(PathBuf::from).collect(),
    }
}

#[test]
fn each_drive_is_a_line_of_its_path_name_and_mount_points() {
    let text = render(&[drive("PHOTOS", &["/media/kuba/PHOTOS", "/media/kuba/1234-ABCD"])]);

    assert_eq!(
        text,
        "/org/freedesktop/UDisks2/drives/SanDisk_Ultra_1234\tPHOTOS\t/media/kuba/PHOTOS\t/media/kuba/1234-ABCD\n"
    );
}

#[test]
fn a_tab_in_a_name_becomes_a_space_and_a_mount_point_with_one_is_left_out() {
    let text = render(&[drive("MY\tSTICK", &["/media/kuba/odd\tname", "/media/kuba/MY STICK"])]);

    assert_eq!(
        text,
        "/org/freedesktop/UDisks2/drives/SanDisk_Ultra_1234\tMY STICK\t/media/kuba/MY STICK\n"
    );
}

#[test]
fn a_drive_left_with_no_readable_mount_point_is_no_line() {
    assert_eq!(render(&[drive("ODD", &["/media/kuba/a\nb"])]), "");
}

#[test]
fn writing_replaces_the_file_whole() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("kubaos-drives");
    fs::write(&file, "old\n").unwrap();

    write(&file, &[drive("STICK", &["/media/kuba/STICK"])]).unwrap();
    let text = fs::read_to_string(&file).unwrap();

    assert!(text.ends_with("\tSTICK\t/media/kuba/STICK\n"), "{text}");
    assert!(!dir.path().join("kubaos-drives.new").exists());
}
