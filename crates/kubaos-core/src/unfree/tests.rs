use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use super::CheckError;
use super::License;
use super::LicenseCheck;
use super::Licenses;
use super::Setting;
use super::Unfree;
use crate::state::StateDir;

fn setting(dir: &Path) -> Setting {
    Setting::new(&StateDir::new(dir.join("kubaos.d")), dir.join("state/kubaos.d.lock"))
}

/// A fake `nix` in `dir` that saves its arguments, one per line, to `dir/args`, prints `stdout`
/// and `stderr` (each `\n` in it a new line), and exits with `code`.
fn fake_nix(dir: &Path, stdout: &str, stderr: &str, code: i32) -> PathBuf {
    let path = dir.join("nix");
    let args = dir.join("args");
    fs::write(
        &path,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >'{}'\nprintf '%s' '{stdout}'\nprintf '%b\\n' '{stderr}' >&2\nexit {code}\n",
            args.display()
        ),
    )
    .expect("write the fake nix");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("make the fake nix executable");
    path
}

fn licenses(pairs: &[(&str, License)]) -> Licenses {
    pairs
        .iter()
        .map(|(name, license)| ((*name).to_owned(), *license))
        .collect()
}

#[test]
fn unfree_packages_are_allowed_without_the_file() {
    let tmp = tempfile::tempdir().unwrap();

    assert_eq!(setting(tmp.path()).get().unwrap(), Unfree::Allowed);
}

#[test]
fn turning_unfree_packages_off_writes_a_file_the_home_reads() {
    let tmp = tempfile::tempdir().unwrap();
    let s = setting(tmp.path());

    let changed = s.set(Unfree::Refused).unwrap();

    assert!(changed);
    assert_eq!(s.get().unwrap(), Unfree::Refused);
    let text = fs::read_to_string(tmp.path().join("kubaos.d/nixpkgs.toml")).unwrap();
    assert!(text.starts_with('#'), "the file says who writes it: {text}");
    assert!(text.lines().any(|l| l == "allow_unfree = false"), "{text}");
}

#[test]
fn setting_what_is_already_set_changes_and_writes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let s = setting(tmp.path());

    let changed = s.set(Unfree::Allowed).unwrap();

    assert!(!changed);
    assert!(!tmp.path().join("kubaos.d/nixpkgs.toml").exists());
}

#[test]
fn turning_unfree_packages_back_on_says_so_in_the_file() {
    let tmp = tempfile::tempdir().unwrap();
    let s = setting(tmp.path());
    s.set(Unfree::Refused).unwrap();

    let changed = s.set(Unfree::Allowed).unwrap();

    assert!(changed);
    assert_eq!(s.get().unwrap(), Unfree::Allowed);
    let text = fs::read_to_string(tmp.path().join("kubaos.d/nixpkgs.toml")).unwrap();
    assert!(text.lines().any(|l| l == "allow_unfree = true"), "{text}");
}

#[test]
fn a_file_without_the_key_allows_unfree_packages() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("kubaos.d")).unwrap();
    fs::write(tmp.path().join("kubaos.d/nixpkgs.toml"), "# nothing set\n").unwrap();

    assert_eq!(setting(tmp.path()).get().unwrap(), Unfree::Allowed);
}

#[test]
fn a_file_that_does_not_parse_is_an_error_and_setting_replaces_it() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("kubaos.d")).unwrap();
    fs::write(tmp.path().join("kubaos.d/nixpkgs.toml"), "allow_unfree = \"no\"\n").unwrap();
    let s = setting(tmp.path());

    assert!(s.get().is_err());
    assert!(s.set(Unfree::Refused).unwrap());
    assert_eq!(s.get().unwrap(), Unfree::Refused);
}

#[test]
fn the_check_asks_the_user_flakes_nixpkgs_once_for_every_name() {
    let check = LicenseCheck::new("/home/u/.config/kubaos/home");

    let argv = check.command(&["discord", "python3Packages.black"]).unwrap();

    let argv: Vec<String> = argv.iter().map(|a| a.to_string_lossy().into_owned()).collect();
    assert_eq!(
        argv[..6],
        [
            "nix",
            "eval",
            "--json",
            "--inputs-from",
            "/home/u/.config/kubaos/home",
            &format!("nixpkgs#legacyPackages.{}-linux", std::env::consts::ARCH),
        ]
    );
    assert_eq!(argv[6], "--apply");
    assert!(argv[7].starts_with("pkgs: "), "{}", argv[7]);
    assert!(
        argv[7].ends_with(r#"[ "discord" "python3Packages.black" ]"#),
        "{}",
        argv[7]
    );
}

#[test]
fn the_check_refuses_a_name_that_could_break_out_of_its_nix_string() {
    let check = LicenseCheck::new("/flake");

    assert!(check.command(&[r#"a" + builtins.abort "x"#]).is_err());
}

#[test]
fn the_check_reads_each_names_license_from_nix_eval() {
    let tmp = tempfile::tempdir().unwrap();
    let nix = fake_nix(tmp.path(), "[true,false,null]", "", 0);
    let check = LicenseCheck::new("/flake").with_program(&nix);

    let found = check.run(&["discord", "hello", "nope"]).unwrap();

    assert_eq!(found.get("discord"), License::Unfree);
    assert_eq!(found.get("hello"), License::Free);
    assert_eq!(found.get("nope"), License::Unknown);
    assert_eq!(found.unfree(), ["discord"]);
    let args = fs::read_to_string(tmp.path().join("args")).unwrap();
    assert!(args.starts_with("eval\n--json\n--inputs-from\n/flake\n"), "{args}");
}

#[test]
fn the_check_runs_nothing_without_names() {
    let tmp = tempfile::tempdir().unwrap();
    let nix = fake_nix(tmp.path(), "[]", "", 0);
    let check = LicenseCheck::new("/flake").with_program(&nix);

    let found = check.run(&[] as &[&str]).unwrap();

    assert_eq!(found, Licenses::default());
    assert!(!tmp.path().join("args").exists());
}

#[test]
fn a_failed_nix_eval_is_an_error_with_its_error_line() {
    let tmp = tempfile::tempdir().unwrap();
    let nix = fake_nix(
        tmp.path(),
        "",
        "warning: Git tree is dirty\\nerror: cannot find flake flake:nixpkgs\\n       hint: check the registry",
        1,
    );
    let check = LicenseCheck::new("/flake").with_program(&nix);

    let err = check.run(&["discord"]).unwrap_err();

    assert!(matches!(err, CheckError::Failed { .. }), "{err:?}");
    assert!(
        err.to_string().ends_with("error: cannot find flake flake:nixpkgs"),
        "{err}"
    );
}

#[test]
fn a_failed_nix_eval_without_an_error_line_gives_its_last_line() {
    let tmp = tempfile::tempdir().unwrap();
    let nix = fake_nix(tmp.path(), "", "warning: something\\nKilled", 137);
    let check = LicenseCheck::new("/flake").with_program(&nix);

    let err = check.run(&["discord"]).unwrap_err();

    assert!(err.to_string().ends_with(": Killed"), "{err}");
}

#[test]
fn an_answer_with_a_value_missing_is_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    let nix = fake_nix(tmp.path(), "[true]", "", 0);
    let check = LicenseCheck::new("/flake").with_program(&nix);

    let err = check.run(&["discord", "hello"]).unwrap_err();

    assert!(matches!(err, CheckError::Mismatch { asked: 2, got: 1 }), "{err:?}");
}

#[test]
fn an_answer_that_is_not_a_list_of_licenses_is_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    let nix = fake_nix(tmp.path(), "{}", "", 0);
    let check = LicenseCheck::new("/flake").with_program(&nix);

    let err = check.run(&["discord"]).unwrap_err();

    assert!(matches!(err, CheckError::Answer(_)), "{err:?}");
}

#[test]
fn a_name_that_is_no_package_name_is_unknown_and_never_reaches_nix() {
    let tmp = tempfile::tempdir().unwrap();
    let nix = fake_nix(tmp.path(), "[true]", "", 0);
    let check = LicenseCheck::new("/flake").with_program(&nix);

    let found = check.run(&["discord", "not a name"]).unwrap();

    assert_eq!(found.get("discord"), License::Unfree);
    assert_eq!(found.get("not a name"), License::Unknown);
    let args = fs::read_to_string(tmp.path().join("args")).unwrap();
    assert!(!args.contains("not a name"), "{args}");
}

#[test]
fn only_names_that_are_no_package_names_run_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let nix = fake_nix(tmp.path(), "[]", "", 0);
    let check = LicenseCheck::new("/flake").with_program(&nix);

    let found = check.run(&["not a name"]).unwrap();

    assert_eq!(found.get("not a name"), License::Unknown);
    assert!(!tmp.path().join("args").exists());
}

#[test]
fn a_missing_nix_is_an_error_naming_it() {
    let check = LicenseCheck::new("/flake").with_program("/nonexistent/nix");

    let err = check.run(&["discord"]).unwrap_err();

    assert!(matches!(err, CheckError::Spawn { .. }), "{err:?}");
}

#[test]
fn allowed_unfree_packages_admit_everything() {
    let found = licenses(&[("discord", License::Unfree), ("hello", License::Free)]);

    assert!(Unfree::Allowed.admit(&found).is_ok());
}

#[test]
fn refused_unfree_packages_admit_free_and_unknown_ones() {
    let found = licenses(&[("hello", License::Free), ("nope", License::Unknown)]);

    assert!(Unfree::Refused.admit(&found).is_ok());
}

#[test]
fn refusing_names_the_unfree_packages_the_setting_and_how_to_turn_it_back_on() {
    let found = licenses(&[
        ("discord", License::Unfree),
        ("hello", License::Free),
        ("spotify", License::Unfree),
    ]);

    let err = Unfree::Refused.admit(&found).unwrap_err();

    assert_eq!(err.names(), ["discord", "spotify"]);
    let message = err.to_string();
    assert!(message.starts_with("discord and spotify are unfree"), "{message}");
    assert!(message.contains("allow_unfree in kubaos.d/nixpkgs.toml"), "{message}");
    assert!(message.contains("kubaos package unfree on"), "{message}");
}

#[test]
fn refusing_one_package_says_is() {
    let found = licenses(&[("discord", License::Unfree)]);

    let message = Unfree::Refused.admit(&found).unwrap_err().to_string();

    assert!(message.starts_with("discord is unfree"), "{message}");
}
