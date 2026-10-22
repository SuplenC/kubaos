use std::fs;
use std::path::Path;
use std::process::Command;

use super::GitSync;
use crate::state::StateDir;

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// A user flake under git, with `kubaos.d/packages.toml` committed.
fn repo() -> (tempfile::TempDir, StateDir) {
    let tmp = tempfile::tempdir().unwrap();
    let flake = tmp.path().join("home");
    fs::create_dir_all(flake.join("kubaos.d")).unwrap();
    fs::write(flake.join("flake.nix"), "{ }\n").unwrap();
    fs::write(flake.join("kubaos.d/packages.toml"), "packages = [\"hello\"]\n").unwrap();
    git(&flake, &["init", "-q"]);
    git(&flake, &["add", "-A"]);
    git(
        &flake,
        &[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-q",
            "-m",
            "init",
        ],
    );
    (tmp, StateDir::new(flake.join("kubaos.d")))
}

fn tracked(state: &StateDir) -> Vec<String> {
    git(state.flake(), &["ls-files"]).lines().map(str::to_owned).collect()
}

fn staged(state: &StateDir) -> String {
    git(state.flake(), &["diff", "--cached", "--name-only", "--diff-filter=AM"])
}

#[test]
fn a_flake_that_is_not_a_git_repo_is_left_alone() {
    let tmp = tempfile::tempdir().unwrap();
    let state = StateDir::new(tmp.path().join("home/kubaos.d"));
    fs::create_dir_all(state.root()).unwrap();

    let sync = state.sync_git().unwrap();

    assert_eq!(sync, GitSync::NotARepo);
    assert!(!tmp.path().join("home/.git").exists());
}

#[test]
fn a_new_file_becomes_visible_to_git_with_intent_to_add() {
    let (_tmp, state) = repo();
    fs::create_dir_all(state.root().join("suites")).unwrap();
    fs::write(state.root().join("suites/test.toml"), "version = \"1\"\n").unwrap();

    let sync = state.sync_git().unwrap();

    assert_eq!(sync, GitSync::Synced);
    assert!(tracked(&state).contains(&"kubaos.d/suites/test.toml".to_owned()));
    let content = git(state.flake(), &["show", ":kubaos.d/suites/test.toml"]);
    assert!(content.is_empty(), "only the intent is recorded, no content: {content}");
}

#[test]
fn a_deleted_file_leaves_the_index() {
    let (_tmp, state) = repo();
    fs::remove_file(state.root().join("packages.toml")).unwrap();

    state.sync_git().unwrap();

    assert!(!tracked(&state).contains(&"kubaos.d/packages.toml".to_owned()));
}

#[test]
fn a_changed_tracked_file_keeps_its_committed_content_in_the_index() {
    let (_tmp, state) = repo();
    fs::write(state.root().join("packages.toml"), "packages = [\"jq\"]\n").unwrap();

    state.sync_git().unwrap();

    let staged = staged(&state);
    assert!(staged.trim().is_empty(), "{staged}");
}

#[test]
fn a_missing_state_directory_in_a_repo_syncs_its_deletion() {
    let (_tmp, state) = repo();
    fs::remove_dir_all(state.root()).unwrap();

    let sync = state.sync_git().unwrap();

    assert_eq!(sync, GitSync::Synced);
    assert!(!tracked(&state).iter().any(|f| f.starts_with("kubaos.d/")));
}
