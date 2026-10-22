use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::sync::Barrier;
use std::thread;

use super::EditError;
use super::Packages;
use super::try_command;
use super::validate_name;
use crate::state::StateDir;

fn packages(dir: &Path) -> Packages {
    Packages::new(&StateDir::new(dir.join("kubaos.d")), dir.join("state/kubaos.d.lock"))
}

#[test]
fn adding_to_a_missing_state_directory_creates_packages_toml() {
    let tmp = tempfile::tempdir().unwrap();
    let p = packages(tmp.path());

    let change = p.add(&["ripgrep"]).unwrap();

    assert_eq!(change.changed, ["ripgrep"]);
    assert_eq!(p.list().unwrap(), ["ripgrep"]);
    assert!(tmp.path().join("kubaos.d/packages.toml").is_file());
}

#[test]
fn the_list_stays_sorted_and_without_duplicates() {
    let tmp = tempfile::tempdir().unwrap();
    let p = packages(tmp.path());
    p.add(&["ripgrep", "hello"]).unwrap();

    let change = p.add(&["jq", "hello", "jq"]).unwrap();

    assert_eq!(change.changed, ["jq"]);
    assert_eq!(change.unchanged, ["hello"]);
    assert_eq!(p.list().unwrap(), ["hello", "jq", "ripgrep"]);
}

#[test]
fn the_written_file_is_what_the_state_directory_reads() {
    let tmp = tempfile::tempdir().unwrap();
    let p = packages(tmp.path());

    p.add(&["python3Packages.black", "hello"]).unwrap();

    let declared = StateDir::new(tmp.path().join("kubaos.d")).declared().unwrap();
    assert_eq!(declared.names(), ["hello", "python3Packages.black"]);
    let text = fs::read_to_string(tmp.path().join("kubaos.d/packages.toml")).unwrap();
    assert!(text.starts_with('#'), "the file says who writes it: {text}");
}

#[test]
fn removing_takes_a_package_out_and_reports_one_that_was_not_there() {
    let tmp = tempfile::tempdir().unwrap();
    let p = packages(tmp.path());
    p.add(&["ripgrep", "hello"]).unwrap();

    let change = p.remove(&["ripgrep", "cowsay"]).unwrap();

    assert_eq!(change.changed, ["ripgrep"]);
    assert_eq!(change.unchanged, ["cowsay"]);
    assert_eq!(p.list().unwrap(), ["hello"]);
}

#[test]
fn removing_from_a_missing_state_directory_creates_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let p = packages(tmp.path());

    let change = p.remove(&["ripgrep"]).unwrap();

    assert!(change.changed.is_empty());
    assert!(!tmp.path().join("kubaos.d").exists());
}

#[test]
fn an_invalid_name_changes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let p = packages(tmp.path());
    p.add(&["hello"]).unwrap();

    let err = p.add(&["jq", "rm -rf"]).unwrap_err();

    assert!(matches!(err, EditError::InvalidName(_)), "{err:?}");
    assert_eq!(p.list().unwrap(), ["hello"]);
}

#[test]
fn package_names_are_nix_attribute_paths() {
    for ok in [
        "ripgrep",
        "python3Packages.black",
        "nerd-fonts.jetbrains-mono",
        "_7zz",
        "go_1_25",
    ] {
        assert!(validate_name(ok).is_ok(), "{ok}");
    }
    for bad in [
        "",
        ".hello",
        "hello.",
        "a..b",
        "rm -rf",
        "a/b",
        "nixpkgs#hello",
        "$(id)",
        "-v",
        "\"x\"",
        "gtk+3",
        "1password",
    ] {
        assert!(validate_name(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn concurrent_adds_lose_none_of_the_packages() {
    let tmp = tempfile::tempdir().unwrap();
    let names: Vec<String> = (0..12).map(|i| format!("pkg{i:02}")).collect();
    let start = Arc::new(Barrier::new(names.len()));

    let threads: Vec<_> = names
        .iter()
        .map(|name| {
            let p = packages(tmp.path());
            let name = name.clone();
            let start = Arc::clone(&start);
            thread::spawn(move || {
                start.wait();
                p.add(&[name]).unwrap();
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }

    assert_eq!(packages(tmp.path()).list().unwrap(), names);
}

#[test]
fn trying_a_package_runs_nix_shell_from_the_user_flakes_nixpkgs() {
    let cmd = try_command(Path::new("/home/kuba/.config/kubaos/home"), "cowsay").unwrap();

    let want: Vec<OsString> = [
        "nix",
        "shell",
        "--inputs-from",
        "/home/kuba/.config/kubaos/home",
        "nixpkgs#cowsay",
    ]
    .into_iter()
    .map(OsString::from)
    .collect();
    assert_eq!(cmd, want);
}

#[test]
fn trying_an_invalid_name_runs_nothing() {
    assert!(try_command(Path::new("/f"), "hello; rm -rf ~").is_err());
}

#[test]
fn an_added_package_file_is_visible_to_git_when_the_flake_is_a_repo() {
    let tmp = tempfile::tempdir().unwrap();
    let status = Command::new("git")
        .args(["init", "-q"])
        .arg(tmp.path())
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .status()
        .unwrap();
    assert!(status.success());
    let p = packages(tmp.path());

    p.add(&["ripgrep"]).unwrap();

    let out = Command::new("git")
        .arg("-C")
        .arg(tmp.path())
        .arg("ls-files")
        .output()
        .unwrap();
    assert_eq!(String::from_utf8(out.stdout).unwrap(), "kubaos.d/packages.toml\n");
}
