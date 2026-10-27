use std::fs;
use std::path::Path;
use std::process::Command;

use kubaos_design::baseline;

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.name=test", "-c", "user.email=test@example.com"])
        .args(args)
        .status()
        .expect("run git");
    assert!(status.success(), "git {args:?}");
}

#[test]
fn a_baseline_git_has_had_counts_as_in_history_even_once_deleted() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    fs::create_dir_all(dir.path().join("design")).unwrap();
    fs::write(dir.path().join("design/lint-baseline.toml"), "").unwrap();

    let before = baseline::in_history(dir.path()).unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "baseline"]);
    git(dir.path(), &["rm", "-q", "design/lint-baseline.toml"]);
    git(dir.path(), &["commit", "-q", "-m", "gone"]);
    let after = baseline::in_history(dir.path()).unwrap();

    assert!(!before);
    assert!(after);
}

#[test]
fn outside_a_git_checkout_the_history_is_unknown() {
    let dir = tempfile::tempdir().unwrap();

    assert!(baseline::in_history(dir.path()).is_err());
}
