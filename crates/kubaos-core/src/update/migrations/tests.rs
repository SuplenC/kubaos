use std::fs;

use super::file_name;
use super::list;
use super::pending;
use super::stamp_of;

#[test]
fn only_timestamped_scripts_with_a_slug_are_migrations() {
    assert_eq!(stamp_of("1761130000-move-the-bar.sh"), Some(1_761_130_000));
    for not in ["move.sh", "1761130000.sh", "1761130000-.sh", "abc-move.sh", "1761130000-Move.sh", "1-x.bash"] {
        assert_eq!(stamp_of(not), None, "{not}");
    }
}

#[test]
fn a_new_migration_is_named_from_its_stamp_and_slug() {
    assert_eq!(file_name(42, "pin-hyprland").unwrap(), "42-pin-hyprland.sh");
    assert!(file_name(42, "Pin Hyprland").is_err());
    assert!(file_name(42, "-x").is_err());
}

#[test]
fn migrations_run_in_timestamp_order_and_a_missing_folder_has_none() {
    let tmp = tempfile::tempdir().unwrap();
    for name in ["20-b.sh", "3-a.sh", "100-c.sh", "README.md", "lib.sh"] {
        fs::write(tmp.path().join(name), "").unwrap();
    }

    let names: Vec<String> = list(tmp.path()).unwrap().into_iter().map(|m| m.name).collect();

    assert_eq!(names, ["3-a.sh", "20-b.sh", "100-c.sh"]);
    assert!(list(&tmp.path().join("missing")).unwrap().is_empty());
}

#[test]
fn a_migration_with_a_marker_is_not_pending() {
    let tmp = tempfile::tempdir().unwrap();
    let (dir, markers) = (tmp.path().join("system"), tmp.path().join("markers"));
    fs::create_dir_all(&dir).unwrap();
    fs::create_dir_all(&markers).unwrap();
    for name in ["1-a.sh", "2-b.sh"] {
        fs::write(dir.join(name), "").unwrap();
    }
    fs::write(markers.join("1-a.sh"), "").unwrap();

    let names: Vec<String> = pending(&dir, &markers).unwrap().into_iter().map(|m| m.name).collect();

    assert_eq!(names, ["2-b.sh"]);
}
