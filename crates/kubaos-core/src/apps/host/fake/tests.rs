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

#[test]
fn apt_shows_a_protected_packages_removal_and_then_refuses_it_as_apt_does() {
    use crate::apps::host::AptPlan;
    use crate::apps::host::Removal;

    let dir = tempfile::tempdir().unwrap();
    let mut host = FakeHost::new(dir.path());
    {
        let mut state = host.state.borrow_mut();
        state.dpkg.insert("libgcc-s1:i386".to_owned(), "1.0".to_owned());
        state.protected.insert("libgcc-s1:i386".to_owned());
    }
    let gone = ["libgcc-s1:i386".to_owned()];

    let simulated = host.apt_simulate(AptPlan::Remove(&gone)).unwrap();
    let removed = host.apt_remove(&gone, Removal::KeepConfig);

    assert!(simulated.removes.contains("libgcc-s1:i386"));
    let err = removed.unwrap_err().to_string();
    assert!(err.contains("Essential packages were removed"), "{err}");
    assert!(host.state.borrow().dpkg.contains_key("libgcc-s1:i386"));
    host.apt_remove(&gone, Removal::PurgeEssential).unwrap();
    assert!(host.state.borrow().dpkg.is_empty());
}

#[test]
fn a_removal_leaves_a_package_with_conffiles_configured_and_a_purge_takes_them() {
    use crate::apps::host::Known;
    use crate::apps::host::Removal;

    let dir = tempfile::tempdir().unwrap();
    let mut host = FakeHost::new(dir.path());
    let conffile = host.place(std::path::Path::new("/etc/ld.so.conf.d/i386.conf"));
    fs::create_dir_all(conffile.parent().unwrap()).unwrap();
    fs::write(&conffile, "/lib/i386\n").unwrap();
    {
        let mut state = host.state.borrow_mut();
        state.dpkg.insert("libc6:i386".to_owned(), "1.0".to_owned());
        state
            .conffiles
            .insert("libc6:i386".to_owned(), vec!["/etc/ld.so.conf.d/i386.conf".to_owned()]);
    }
    let libc = ["libc6:i386".to_owned()];

    host.apt_remove(&libc, Removal::KeepConfig).unwrap();

    assert_eq!(
        host.arch_packages("i386").unwrap(),
        [("libc6:i386".to_owned(), Known::ConfigFiles)].into()
    );
    assert!(host.set_foreign_arch("i386", false).is_err(), "dpkg still knows it");
    host.apt_remove(&libc, Removal::Purge).unwrap();
    assert!(host.arch_packages("i386").unwrap().is_empty());
    assert!(!conffile.exists());
}
