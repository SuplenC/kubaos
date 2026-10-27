use std::fs;
use std::path::Path;

use super::*;
use crate::fake::FakeRunner;

const STICK: &str = "/org/freedesktop/UDisks2/drives/SanDisk_Ultra_1234";

fn one_stick() -> Drives {
    Drives::parse(&format!("{STICK}\tPHOTOS\t/media/kuba/PHOTOS\t/media/kuba/1234-ABCD\n"))
}

#[test]
fn each_mounted_drive_is_an_eject_item() {
    let items = one_stick().items();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, "drive:SanDisk_Ultra_1234");
    assert_eq!(items[0].kind.name(), KIND);
    assert_eq!(items[0].title, "Eject PHOTOS");
    assert_eq!(
        items[0].subtitle.as_deref(),
        Some("Unmount /media/kuba/PHOTOS, /media/kuba/1234-ABCD and power it off")
    );
    assert!(items[0].keywords.contains(&"eject drive".to_owned()));
}

#[test]
fn a_line_missing_its_name_or_mount_points_is_skipped() {
    let drives = Drives::parse(&format!("{STICK}\tPHOTOS\n{STICK}\t\t/media/x\nnot a path\tX\t/media/y\n\n"));

    assert_eq!(drives.items(), [] as [kotodama_protocol::Item; 0]);
}

#[test]
fn no_drives_file_means_no_items() {
    assert_eq!(Drives::read(Path::new("/nonexistent/kubaos-drives")).items(), [] as [kotodama_protocol::Item; 0]);
}

#[test]
fn reading_the_file_gives_its_drives() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join(FILE_NAME);
    fs::write(&file, format!("{STICK}\tSTICK\t/media/kuba/STICK\n")).unwrap();

    assert_eq!(Drives::read(&file).items()[0].title, "Eject STICK");
}

#[test]
fn eject_runs_kubaos_automount_on_the_drive_and_closes_the_menu() {
    let runner = FakeRunner::default();

    let ran = one_stick().run("drive:SanDisk_Ultra_1234", EJECT, &runner);

    assert!(matches!(ran, Ok(Ran::Close)));
    assert_eq!(runner.launched.borrow()[0].0, ["kubaos-automount", "eject", STICK]);
}

#[test]
fn an_unknown_drive_or_action_is_refused() {
    let runner = FakeRunner::default();

    assert!(one_stick().run("drive:Other", EJECT, &runner).is_err());
    assert!(one_stick().run("drive:SanDisk_Ultra_1234", "format", &runner).is_err());
    assert!(runner.launched.borrow().is_empty());
}
