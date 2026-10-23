use std::ffi::OsStr;

use super::Cmd;
use super::Lines;
use super::MachineError;

fn argv(cmd: &std::process::Command) -> Vec<String> {
    std::iter::once(cmd.get_program())
        .chain(cmd.get_args())
        .map(|a| a.to_string_lossy().into_owned())
        .collect()
}

#[test]
fn a_root_command_runs_through_sudo_with_its_variables_on_the_command_line() {
    let cmd = Cmd::root("bash")
        .args(["-euo", "pipefail", "m.sh"])
        .env("KUBAOS_DIR", "/usr/share/kubaos");

    let as_user = cmd.command(false, None);
    let as_root = cmd.command(true, None);

    assert_eq!(
        argv(&as_user),
        [
            "sudo",
            "env",
            "KUBAOS_DIR=/usr/share/kubaos",
            "bash",
            "-euo",
            "pipefail",
            "m.sh"
        ]
    );
    assert_eq!(
        argv(&as_root),
        [
            "env",
            "KUBAOS_DIR=/usr/share/kubaos",
            "bash",
            "-euo",
            "pipefail",
            "m.sh"
        ]
    );
    assert_eq!(cmd.display(), "sudo bash -euo pipefail m.sh");
}

#[test]
fn a_user_command_gets_its_variables_and_the_path_in_its_environment() {
    let cmd = Cmd::user("nix").args(["flake", "update"]).env("A", "1");

    let command = cmd.command(false, Some(OsStr::new("/nix/bin:/usr/bin")));

    assert_eq!(argv(&command), ["nix", "flake", "update"]);
    let envs: Vec<_> = command
        .get_envs()
        .map(|(k, v)| (k.to_owned(), v.map(OsStr::to_owned)))
        .collect();
    assert!(envs.contains(&("A".into(), Some("1".into()))));
    assert!(envs.contains(&("PATH".into(), Some("/nix/bin:/usr/bin".into()))));
}

#[test]
fn a_failure_keeps_the_last_lines_of_stderr() {
    let cmd = Cmd::root("apt-get").arg("upgrade");
    let stderr = (1..=8).map(|n| format!("line {n}")).collect::<Vec<_>>().join("\n");

    let err = MachineError::exited(&cmd, Some(100), &stderr);

    assert_eq!(
        err.to_string(),
        "`sudo apt-get upgrade` exited with code 100: line 4\nline 5\nline 6\nline 7\nline 8"
    );
}

#[test]
fn output_is_cut_into_lines_at_newlines_and_progress_redraws() {
    let mut lines = Lines::default();
    let mut got = Vec::new();

    lines.push(b"Unpacking vim\nProgress: 10%\rProgress: 2", |l| got.push(l.to_owned()));
    lines.push(b"0%\r\n\nSetting up vim", |l| got.push(l.to_owned()));
    lines.finish(|l| got.push(l.to_owned()));

    assert_eq!(got, ["Unpacking vim", "Progress: 10%", "Progress: 20%", "Setting up vim"]);
}

#[test]
fn a_user_command_that_runs_sudo_itself_uses_sudo() {
    assert!(Cmd::user("system/relink.sh").runs_sudo().uses_sudo());
    assert!(Cmd::root("apt-get").uses_sudo());
    assert!(!Cmd::user("nix").uses_sudo());
}
