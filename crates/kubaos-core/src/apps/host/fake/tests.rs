use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::FakeHost;
use crate::apps::host::Host;
use crate::sandbox::NOWHERE;

#[test]
fn a_script_the_fake_machine_runs_reaches_the_stand_ins_and_never_the_session() {
    let dir = tempfile::tempdir().unwrap();
    let mut host = FakeHost::new(dir.path());
    let scripts = dir.path().join("app");
    fs::create_dir_all(&scripts).unwrap();
    fs::write(
        scripts.join("ask"),
        "#!/bin/sh\n\
         {\n\
           command -v systemctl; command -v loginctl\n\
           systemctl --user is-system-running; echo \"systemctl $?\"\n\
           loginctl; echo \"loginctl $?\"\n\
           echo \"bus $DBUS_SESSION_BUS_ADDRESS\"\n\
         } >\"$HOME/seen\"\n",
    )
    .unwrap();
    fs::set_permissions(scripts.join("ask"), fs::Permissions::from_mode(0o755)).unwrap();

    host.run(&["./ask".to_owned()], &scripts).unwrap();

    let bin = host.sandbox().bin();
    let seen = fs::read_to_string(host.home().join("seen")).unwrap();
    assert_eq!(
        seen,
        format!(
            "{}\n{}\nsystemctl 0\nloginctl 0\nbus {NOWHERE}\n",
            bin.join("systemctl").display(),
            bin.join("loginctl").display()
        )
    );
    assert_eq!(
        host.sandbox().calls(),
        ["systemctl --user is-system-running", "loginctl"]
    );
    if let Ok(live) = env::var("DBUS_SESSION_BUS_ADDRESS") {
        assert!(!seen.contains(&live), "the live session's bus reached the script");
    }
}

#[test]
fn a_run_naming_a_bare_command_runs_its_stand_in() {
    let dir = tempfile::tempdir().unwrap();
    let mut host = FakeHost::new(dir.path());

    host.run(
        &["systemctl", "--user", "is-enabled", "--quiet", "t3code.service"].map(str::to_owned),
        dir.path(),
    )
    .unwrap();

    assert_eq!(
        host.sandbox().calls(),
        ["systemctl --user is-enabled --quiet t3code.service"]
    );
}
