use std::fs;
use std::path::Path;
use std::path::PathBuf;

use super::MigrateError;
use super::Migration;
use super::STATE_DIR_LINE;
use super::is_wired;
use super::wire;
use super::wire_home_nix;

/// The `home.nix` the installer wrote before the state directory existed.
const OLD_TEMPLATE: &str = "\
# Your own home-manager settings, on top of KubaOS's. pkgs is your nixpkgs
# (flake.nix). Apply with
#   home-manager switch --flake ~/.config/kubaos/home
{ pkgs, ... }:
{
  home.packages = [
    # pkgs.hello
  ];
}
";

#[test]
fn wiring_adds_the_line_before_the_closing_brace_and_changes_nothing_else() {
    let wired = wire(OLD_TEMPLATE).unwrap();

    let body = wired.strip_suffix("}\n").unwrap();
    assert!(body.starts_with(OLD_TEMPLATE.strip_suffix("}\n").unwrap()), "{wired}");
    assert!(body.trim_end().ends_with(STATE_DIR_LINE.trim()), "{wired}");
    assert_eq!(wired.lines().count(), OLD_TEMPLATE.lines().count() + 3);
}

#[test]
fn wiring_is_seen_as_wired() {
    assert!(!is_wired(OLD_TEMPLATE));
    assert!(is_wired(&wire(OLD_TEMPLATE).unwrap()));
}

#[test]
fn a_commented_out_line_does_not_count_as_wired() {
    let text = OLD_TEMPLATE.replace("  home.packages", "  # kubaos.stateDir = ./kubaos.d;\n  home.packages");

    assert!(!is_wired(&text));
}

#[test]
fn trailing_comments_and_blank_lines_after_the_brace_stay_after_it() {
    let text = format!("{OLD_TEMPLATE}\n# The end.\n\n");

    let wired = wire(&text).unwrap();

    assert!(wired.ends_with("}\n\n# The end.\n\n"), "{wired}");
    assert!(is_wired(&wired));
}

#[test]
fn a_file_without_a_final_newline_is_wired_too() {
    let text = OLD_TEMPLATE.trim_end();

    let wired = wire(text).unwrap();

    assert!(wired.ends_with('}'), "{wired}");
    assert!(is_wired(&wired));
}

#[test]
fn a_file_that_does_not_end_in_a_lone_brace_is_left_alone() {
    assert_eq!(wire("{ pkgs, ... }: { home.packages = [ ]; }\n"), None);
    assert_eq!(wire("{ lib, ... }:\nlib.mkMerge [\n  { }\n]\n"), None);
}

/// A user flake holding `home.nix`, and the backups folder outside it.
fn flake_with(text: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let flake = tmp.path().join("flake");
    fs::create_dir(&flake).unwrap();
    let home_nix = flake.join("home.nix");
    fs::write(&home_nix, text).unwrap();
    let backups = tmp.path().join("state/backups");
    (tmp, home_nix, backups)
}

fn files_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .map(|entries| {
            entries
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

#[test]
fn wiring_the_file_backs_it_up_first() {
    let (_tmp, home_nix, backups) = flake_with(OLD_TEMPLATE);

    let migration = wire_home_nix(&home_nix, &backups).unwrap();

    let Migration::Wired { backup } = migration else {
        panic!("{migration:?}")
    };
    assert_eq!(fs::read_to_string(&backup).unwrap(), OLD_TEMPLATE);
    assert!(is_wired(&fs::read_to_string(&home_nix).unwrap()));
}

#[test]
fn the_backup_lives_outside_the_user_flake() {
    let (_tmp, home_nix, backups) = flake_with(OLD_TEMPLATE);

    let Migration::Wired { backup } = wire_home_nix(&home_nix, &backups).unwrap() else {
        panic!("not wired")
    };

    assert!(backup.starts_with(&backups), "{backup:?}");
    assert_eq!(files_in(home_nix.parent().unwrap()), ["home.nix"]);
}

#[test]
fn wiring_twice_changes_the_file_once_and_backs_it_up_once() {
    let (_tmp, home_nix, backups) = flake_with(OLD_TEMPLATE);
    wire_home_nix(&home_nix, &backups).unwrap();
    let once = fs::read_to_string(&home_nix).unwrap();

    let second = wire_home_nix(&home_nix, &backups).unwrap();

    assert!(matches!(second, Migration::AlreadyWired), "{second:?}");
    assert_eq!(fs::read_to_string(&home_nix).unwrap(), once);
    assert_eq!(files_in(&backups).len(), 1);
}

#[test]
fn an_earlier_backup_is_never_overwritten() {
    let (_tmp, home_nix, backups) = flake_with(OLD_TEMPLATE);
    fs::create_dir_all(&backups).unwrap();
    fs::write(backups.join("home.nix.bak"), "older").unwrap();

    let Migration::Wired { backup } = wire_home_nix(&home_nix, &backups).unwrap() else {
        panic!("not wired")
    };

    assert_eq!(fs::read_to_string(backups.join("home.nix.bak")).unwrap(), "older");
    assert_eq!(fs::read_to_string(backup).unwrap(), OLD_TEMPLATE);
}

#[test]
fn a_file_it_cannot_wire_is_an_error_and_stays_as_it_was() {
    let text = "{ pkgs, ... }: { home.packages = [ ]; }\n";
    let (_tmp, home_nix, backups) = flake_with(text);

    let err = wire_home_nix(&home_nix, &backups).unwrap_err();

    assert!(matches!(err, MigrateError::NoClosingBrace { .. }), "{err:?}");
    assert_eq!(fs::read_to_string(&home_nix).unwrap(), text);
    assert_eq!(files_in(home_nix.parent().unwrap()), ["home.nix"]);
    assert!(files_in(&backups).is_empty());
}

#[test]
fn the_shipped_template_is_already_wired() {
    let template = include_str!("../../../../../user-flake/home.nix");

    assert!(is_wired(template));
}
