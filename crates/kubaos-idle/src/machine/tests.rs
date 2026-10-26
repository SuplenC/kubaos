use std::fs;
use std::path::Path;

use super::*;

fn supply(root: &Path, name: &str, kind: &str, online: Option<&str>) {
    let dir = root.join(name);
    fs::create_dir_all(&dir).expect("mkdir");
    fs::write(dir.join("type"), format!("{kind}\n")).expect("type");
    if let Some(online) = online {
        fs::write(dir.join("online"), format!("{online}\n")).expect("online");
    }
}

#[test]
fn an_online_mains_supply_is_plugged_in() {
    let root = tempfile::tempdir().expect("a temp dir");
    supply(root.path(), "BAT0", "Battery", None);
    supply(root.path(), "ADP1", "Mains", Some("1"));

    assert_eq!(power(root.path()), Power::Ac);
}

#[test]
fn a_battery_with_its_adapter_offline_is_on_battery() {
    let root = tempfile::tempdir().expect("a temp dir");
    supply(root.path(), "BAT0", "Battery", None);
    supply(root.path(), "ADP1", "Mains", Some("0"));
    supply(root.path(), "ucsi-source-psy-USBC000:001", "USB", Some("1"));

    assert_eq!(power(root.path()), Power::Battery);
}

#[test]
fn a_machine_with_no_battery_is_plugged_in() {
    let root = tempfile::tempdir().expect("a temp dir");

    assert_eq!(power(root.path()), Power::Ac);
    assert_eq!(power(&root.path().join("missing")), Power::Ac);
}

#[test]
fn busctl_answers_read_as_the_lid_and_the_dock() {
    assert_eq!(
        LidState::parse("b true\nb false\n"),
        Some(LidState {
            closed: true,
            docked: false
        })
    );
    assert_eq!(
        LidState::parse("b false\nb true"),
        Some(LidState {
            closed: false,
            docked: true
        })
    );
    assert_eq!(LidState::parse("b true\n"), None);
    assert_eq!(LidState::parse("Failed to get property\n"), None);
}

#[test]
fn hyprland_runs_while_its_lock_files_pid_does() {
    let runtime = tempfile::tempdir().expect("a temp dir");
    let proc = tempfile::tempdir().expect("a fake /proc");
    let lock = hyprland_lock(runtime.path(), "abc_123");
    fs::create_dir_all(lock.parent().expect("a parent")).expect("mkdir");
    fs::write(&lock, "4242\nwayland-1\n").expect("lock file");

    assert!(!hyprland_runs(&lock, proc.path()));
    fs::create_dir(proc.path().join("4242")).expect("the pid");
    assert!(hyprland_runs(&lock, proc.path()));
    assert!(!hyprland_runs(&runtime.path().join("nothing"), proc.path()));
    assert!(lock.ends_with("hypr/abc_123/hyprland.lock"));
}

#[test]
fn any_hyprland_counts_whichever_instance_runs() {
    let runtime = tempfile::tempdir().expect("a temp dir");
    let proc = tempfile::tempdir().expect("a fake /proc");
    assert!(!any_hyprland_runs(runtime.path(), proc.path()));
    for (signature, pid) in [("old", "100"), ("new", "200")] {
        let lock = hyprland_lock(runtime.path(), signature);
        fs::create_dir_all(lock.parent().expect("a parent")).expect("mkdir");
        fs::write(&lock, format!("{pid}\nwayland-1\n")).expect("lock file");
    }

    assert!(!any_hyprland_runs(runtime.path(), proc.path()));
    fs::create_dir(proc.path().join("200")).expect("the new pid");
    assert!(any_hyprland_runs(runtime.path(), proc.path()));
}
