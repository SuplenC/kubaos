use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::SystemHost;
use crate::apps::host::Host;

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
