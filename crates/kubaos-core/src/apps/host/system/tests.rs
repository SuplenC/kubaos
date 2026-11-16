use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::SystemHost;
use super::candidate;
use super::simulation;
use crate::apps::host::Downgrade;
use crate::apps::host::Host;
use crate::sandbox::Sandbox;

/// What a script run by `host` sees of its environment: its `HOME`, and whether it inherited
/// a variable of the process that ran it (cargo sets `CARGO_PKG_NAME` for every test).
fn environment(host: &mut SystemHost) -> String {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("env");
    fs::write(
        &script,
        "#!/bin/sh\nprintf '%s %s' \"$HOME\" \"${CARGO_PKG_NAME:-none}\" > seen\n",
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();

    host.run(&["./env".to_owned()], dir.path()).unwrap();

    fs::read_to_string(dir.path().join("seen")).unwrap()
}

#[test]
fn a_run_as_root_gets_roots_environment_and_none_of_the_users() {
    let mut host = SystemHost::new("kuba", "/home/kuba", "/usr/share/kubaos").as_root();

    assert_eq!(environment(&mut host), "/root none");
}

#[test]
fn a_run_as_the_user_gets_their_home_and_environment() {
    let mut host = SystemHost::new("kuba", "/home/kuba", "/usr/share/kubaos");

    assert_eq!(environment(&mut host), "/home/kuba kubaos-core");
}

#[test]
fn a_simulation_names_what_apt_would_newly_install_and_remove_but_not_upgrades() {
    let shown = "NOTE: This is only a simulation!\n\
                 Remv docker.io [24.0.7-0ubuntu4]\n\
                 Inst libfoo1:amd64 (1.2-3 Ubuntu:24.04/noble [amd64])\n\
                 Inst tool (2.0 Vendor:stable [amd64])\n\
                 Inst libc6 [2.39-0ubuntu8] (2.39-0ubuntu8.4 Ubuntu:24.04/noble-updates [amd64])\n\
                 Conf tool (2.0 Vendor:stable [amd64])\n";

    let simulation = simulation(shown, "amd64");

    assert_eq!(simulation.removes, ["docker.io".to_owned()].into());
    assert_eq!(simulation.installs, ["libfoo1".to_owned(), "tool".to_owned()].into());
}

#[test]
fn a_package_of_another_architecture_keeps_its_architecture_in_its_name() {
    let shown = "Inst libfoo1:i386 (1.2-3 Ubuntu:24.04/noble [i386])\n\
                 Inst libfoo1 (1.2-3 Ubuntu:24.04/noble [amd64])\n\
                 Remv libbar1:i386 [1.0]\n";

    let simulation = simulation(shown, "amd64");

    assert_eq!(
        simulation.installs,
        ["libfoo1".to_owned(), "libfoo1:i386".to_owned()].into()
    );
    assert_eq!(simulation.removes, ["libbar1:i386".to_owned()].into());
}

#[test]
fn the_candidate_is_what_apt_cache_policy_would_install() {
    let policy =
        "neovim:\n  Installed: (none)\n  Candidate: 0.9.5-6ubuntu2\n  Version table:\n     0.9.5-6ubuntu2 500\n";

    assert_eq!(candidate(policy).as_deref(), Some("0.9.5-6ubuntu2"));
    assert_eq!(candidate("neovim:\n  Installed: (none)\n  Candidate: (none)\n"), None);
    assert_eq!(candidate(""), None);
}

#[test]
fn apt_moves_a_package_down_only_when_the_install_allows_it() {
    let dir = tempfile::tempdir().unwrap();
    let sandbox = Sandbox::new(&dir.path().join("sandbox"), dir.path(), "kuba");
    let mut host = SystemHost::new("kuba", dir.path(), "/usr/share/kubaos")
        .as_root()
        .with_path(sandbox.bin());

    host.apt_install(&["firefox".to_owned()], Downgrade::Refuse).unwrap();
    host.apt_install(&["firefox".to_owned()], Downgrade::Allow).unwrap();

    let calls = sandbox.calls();
    assert_eq!(calls.len(), 2, "{calls:?}");
    assert!(!calls[0].contains("--allow-downgrades"), "{}", calls[0]);
    assert!(calls[1].contains("--allow-downgrades"), "{}", calls[1]);
}
