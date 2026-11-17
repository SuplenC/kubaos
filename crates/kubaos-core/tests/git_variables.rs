//! What reaches a program KubaOS starts of the git variables KubaOS itself was started with: the
//! ones a user sets for every git do, the ones that name a repository don't. The test starts
//! this binary again in a sandbox, with the variables set, so they are inherited as they would
//! be from a shell or a git hook.

use std::env;
use std::process::Command;

use kubaos_core::sandbox::Sandbox;

/// Set in the started copy, which then runs the check instead of starting another.
const CHILD: &str = "KUBAOS_TEST_GIT_VARIABLES_CHILD";

#[test]
fn a_started_program_gets_the_users_git_settings_and_none_of_its_repository_variables() {
    let dir = tempfile::tempdir().unwrap();
    let sandbox = Sandbox::new(&dir.path().join("sandbox"), &dir.path().join("home"), "tester");

    let out = sandbox
        .command(env::current_exe().unwrap())
        .args(["--exact", "inherited", "--nocapture"])
        .env(CHILD, "1")
        .env("GIT_SSH_COMMAND", "ssh -i /keys/deploy")
        .env("GIT_DIR", "/elsewhere/.git")
        .output()
        .unwrap();

    let said = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{said}{}", String::from_utf8_lossy(&out.stderr));
    assert!(said.contains("1 passed"), "the check didn't run: {said}");
}

/// The check, in the started copy: a program started with `kubaos_git::clear` sees what this
/// process inherited, less the repository's variables.
#[test]
fn inherited() {
    if env::var_os(CHILD).is_none() {
        return;
    }
    // Outside the sandbox: this process is the sandbox's already, started above, and its
    // inherited environment is what the check is about.
    let mut command = Command::new("sh");
    kubaos_git::clear(&mut command).args(["-c", r#"printf '%s|%s' "$GIT_SSH_COMMAND" "${GIT_DIR-unset}""#]);

    let out = command.output().unwrap();

    assert_eq!(String::from_utf8_lossy(&out.stdout), "ssh -i /keys/deploy|unset");
}
