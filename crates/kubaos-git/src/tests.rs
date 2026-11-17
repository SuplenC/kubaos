use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use super::REPOSITORY;
use super::at;
use super::clear;

/// The variables `command` takes away from what it inherits.
fn removed(command: &Command) -> Vec<&OsStr> {
    command
        .get_envs()
        .filter(|(_, value)| value.is_none())
        .map(|(name, _)| name)
        .collect()
}

#[test]
fn git_runs_in_the_folder_it_is_given() {
    let command = at(Path::new("/srv/flake"));

    let args: Vec<&OsStr> = command.get_args().collect();
    assert_eq!(command.get_program(), "git");
    assert_eq!(args, ["-C", "/srv/flake"]);
}

#[test]
fn git_inherits_none_of_the_variables_that_point_it_elsewhere() {
    let command = at(Path::new("/srv/flake"));

    let removed = removed(&command);
    for name in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_COMMON_DIR",
        "GIT_CONFIG_PARAMETERS",
    ] {
        assert!(removed.contains(&OsStr::new(name)), "{name} is passed on");
    }
}

#[test]
fn the_variables_a_user_sets_for_every_git_are_passed_on() {
    let command = at(Path::new("/srv/flake"));

    let removed = removed(&command);
    for name in [
        "GIT_SSH_COMMAND",
        "GIT_ASKPASS",
        "GIT_SSL_CAINFO",
        "GIT_AUTHOR_NAME",
        "GIT_CONFIG_GLOBAL",
    ] {
        assert!(!removed.contains(&OsStr::new(name)), "{name} is taken away");
    }
}

#[test]
fn a_numbered_configuration_entry_set_on_the_command_is_taken_away() {
    // Outside the sandbox: never started, only its environment is read.
    let mut command = Command::new("nix");
    command
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "core.worktree")
        .env("GIT_CONFIG_VALUE_0", "/elsewhere");

    clear(&mut command);

    let removed = removed(&command);
    for name in ["GIT_CONFIG_COUNT", "GIT_CONFIG_KEY_0", "GIT_CONFIG_VALUE_0"] {
        assert!(removed.contains(&OsStr::new(name)), "{name} is passed on");
    }
}

#[test]
fn a_repository_variable_set_on_the_command_before_is_taken_away_too() {
    // Outside the sandbox: never started, only its environment is read.
    let mut command = Command::new("nix");
    command.env("GIT_DIR", "/elsewhere/.git").env("KEEP", "1");

    clear(&mut command);

    let removed = removed(&command);
    for name in REPOSITORY {
        assert!(removed.contains(&OsStr::new(name)), "{name} is passed on");
    }
    assert!(
        command
            .get_envs()
            .any(|(name, value)| name == "KEEP" && value.is_some())
    );
}
