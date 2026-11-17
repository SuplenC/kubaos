use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

use super::*;

fn sandbox() -> (tempfile::TempDir, Sandbox) {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    fs::create_dir_all(&home).unwrap();
    let sandbox = Sandbox::new(&dir.path().join("sandbox"), &home, "kuba");
    (dir, sandbox)
}

/// `script` run by `sh` in `sandbox`: its exit code, stdout and stderr.
fn sh(sandbox: &Sandbox, script: &str) -> (Option<i32>, String, String) {
    let output = sandbox.command("sh").args(["-c", script]).output().unwrap();
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// The real program `name` on the test's own `PATH`.
fn real(name: &str) -> Option<PathBuf> {
    env::split_paths(&env::var_os("PATH")?)
        .map(|dir| dir.join(name))
        .find(|p| p.is_file())
}

#[test]
fn the_environment_holds_no_route_to_the_session() {
    let (_dir, sandbox) = sandbox();

    let (_, shown, _) = sh(&sandbox, "env");

    let seen: BTreeMap<&str, &str> = shown.lines().filter_map(|l| l.split_once('=')).collect();
    assert_eq!(seen.get("DBUS_SESSION_BUS_ADDRESS"), Some(&NOWHERE));
    assert_eq!(seen.get("DBUS_SYSTEM_BUS_ADDRESS"), Some(&NOWHERE));
    assert_eq!(seen.get("PATH").map(PathBuf::from), Some(sandbox.bin()));
    assert_eq!(seen.get("XDG_RUNTIME_DIR").map(PathBuf::from), Some(sandbox.runtime()));
    for gone in ["DISPLAY", "WAYLAND_DISPLAY", "SSH_AUTH_SOCK", "SUDO_ASKPASS"] {
        assert!(!seen.contains_key(gone), "{gone} reached the sandbox");
    }
    if let Ok(live) = env::var("DBUS_SESSION_BUS_ADDRESS") {
        assert!(!shown.contains(&live), "the live session's bus reached the sandbox");
    }
}

#[test]
fn a_command_with_effects_is_a_stand_in_that_logs_its_call() {
    let (_dir, sandbox) = sandbox();

    let (code, out, _) = sh(
        &sandbox,
        "command -v systemctl; systemctl --user disable --now t3code.service; pkexec apt-get install -y sl",
    );

    assert_eq!(code, Some(0));
    assert_eq!(out.trim(), sandbox.bin().join("systemctl").display().to_string());
    assert_eq!(
        sandbox.calls(),
        [
            "systemctl --user disable --now t3code.service",
            "pkexec apt-get install -y sl"
        ]
    );
}

#[test]
fn a_stand_in_for_a_process_search_finds_nothing() {
    let (_dir, sandbox) = sandbox();

    let (code, _, _) = sh(&sandbox, "pgrep -x t3code");

    assert_eq!(code, Some(1));
}

#[test]
fn a_test_gives_a_command_its_own_answer() {
    let (_dir, sandbox) = sandbox();
    sandbox.stub("systemctl", "echo degraded; exit 3");

    let (code, out, _) = sh(&sandbox, "systemctl --user is-system-running");

    assert_eq!((code, out.as_str()), (Some(3), "degraded\n"));
    assert_eq!(sandbox.calls(), ["systemctl --user is-system-running"]);
}

#[test]
fn a_command_taken_out_is_not_found() {
    let (_dir, sandbox) = sandbox();
    sandbox.without("nix");

    let (found, shown, _) = sh(&sandbox, "command -v nix");
    let (ran, _, err) = sh(&sandbox, "nix --version");

    assert_ne!(found, Some(0));
    assert_eq!(shown, "");
    assert_eq!(ran, Some(127), "{err}");
}

#[test]
fn the_sandboxs_shell_is_the_one_a_scripts_shebang_runs() {
    let (_dir, sandbox) = sandbox();

    let linked = fs::read_link(sandbox.bin().join("sh")).unwrap();

    assert_eq!(linked, Path::new(SYSTEM_SHELL));
}

#[test]
fn a_command_neither_stood_in_for_nor_a_tool_is_not_found() {
    let (_dir, sandbox) = sandbox();

    let (code, _, err) = sh(&sandbox, "ip link show");

    assert_eq!(code, Some(127), "{err}");
    assert!(err.contains("not found"), "{err}");
}

#[test]
fn the_tools_a_script_needs_are_there() {
    let (_dir, sandbox) = sandbox();

    let (code, out, err) = sh(&sandbox, "printf 'b\\na\\n' | sort | tr a-z A-Z | sed 's/^/-/'");

    assert_eq!(code, Some(0), "{err}");
    assert_eq!(out, "-A\n-B\n");
}

#[test]
fn a_program_called_by_its_full_path_finds_no_user_manager_and_no_logind() {
    // Read-only questions, which only show that nothing answers.
    for (name, args) in [
        ("systemctl", &["--user", "is-system-running"][..]),
        ("loginctl", &["list-users", "--no-legend"][..]),
    ] {
        let Some(program) = real(name) else {
            continue;
        };
        let (_dir, sandbox) = sandbox();

        let output = sandbox.command(&program).args(args).output().unwrap();

        let out = String::from_utf8_lossy(&output.stdout);
        assert!(!output.status.success(), "{name} reached its manager: {out}");
        assert!(out.trim().is_empty(), "{name} answered: {out}");
    }
}

#[test]
fn none_of_gits_variables_gets_into_the_sandbox() {
    let (_dir, sandbox) = sandbox();

    let (_, shown, _) = sh(&sandbox, "env");

    let git: Vec<&str> = shown.lines().filter(|l| l.starts_with("GIT_")).collect();
    assert!(git.is_empty(), "{git:?}");
}

#[test]
fn git_in_the_sandbox_works_on_the_repository_it_is_given() {
    let (dir, sandbox) = sandbox();
    let repo = dir.path().join("repo");
    fs::create_dir_all(&repo).unwrap();

    let init = sandbox.git(&repo).args(["init", "-q"]).status().unwrap();
    let top = git(&repo, &["rev-parse", "--show-toplevel"]);

    assert!(init.success());
    assert_eq!(PathBuf::from(top), repo.canonicalize().unwrap());
}

#[test]
fn a_program_beyond_the_tools_is_linked_in_for_a_one_off_command() {
    let out = output("git", ["--version"]);

    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).starts_with("git version"));
}
