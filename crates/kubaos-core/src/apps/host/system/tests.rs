use std::fs;
use std::path::Path;

use super::Named;
use super::SystemHost;
use super::bound_devices;
use super::candidate;
use super::certificate_serial;
use super::dkms_certificate;
use super::dkms_status;
use super::enrolment;
use super::important;
use super::known_of_arch;
use super::mok_source_args;
use super::signing_named;
use super::simulation;
use crate::apps::host::Downgrade;
use crate::apps::host::Host;
use crate::apps::host::Known;
use crate::sandbox;
use crate::sandbox::Sandbox;

/// What a script run by `host`, finding programs in a sandbox, sees of its environment: its
/// `HOME`, and whether it inherited a variable of the process that ran it (cargo sets
/// `CARGO_PKG_NAME` for every test).
fn environment(host: SystemHost) -> String {
    let dir = tempfile::tempdir().unwrap();
    let sandbox = Sandbox::new(&dir.path().join("sandbox"), &dir.path().join("home"), "kuba");
    let mut host = host.with_path(sandbox.bin());
    sandbox::program(
        &dir.path().join("env"),
        "#!/bin/sh\nprintf '%s %s' \"$HOME\" \"${CARGO_PKG_NAME:-none}\" > seen\n",
    );

    host.run(&["./env".to_owned()], dir.path()).unwrap();

    fs::read_to_string(dir.path().join("seen")).unwrap()
}

#[test]
fn a_run_as_root_gets_roots_environment_and_none_of_the_users() {
    let host = SystemHost::new("kuba", "/home/kuba", "/usr/share/kubaos").as_root();

    assert_eq!(environment(host), "/root none");
}

#[test]
fn a_run_as_the_user_gets_their_home_and_environment() {
    let host = SystemHost::new("kuba", "/home/kuba", "/usr/share/kubaos");

    assert_eq!(environment(host), "/home/kuba kubaos-core");
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
fn a_simulation_names_what_apt_would_move_down() {
    let shown = "Inst firefox [1:1snap1-0ubuntu9.1] (160.0~build1 mozilla:all [amd64])\n\
                 Inst libnss3 [2:3.110-1] (2:3.98-1build1 Ubuntu:24.04/noble [amd64])\n\
                 Inst libc6 [2.39-0ubuntu8] (2.39-0ubuntu8.4 Ubuntu:24.04/noble-updates [amd64])\n";

    let simulation = simulation(shown, "amd64");

    assert_eq!(
        simulation.downgrades,
        ["firefox".to_owned(), "libnss3".to_owned()].into()
    );
    assert!(simulation.installs.is_empty());
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

#[test]
fn an_architectures_packages_are_those_dpkg_has_installed_or_configured() {
    let shown = "ii \tlibc6\ti386\nrc \tlibgl1\ti386\nun \tlibold\ti386\nii \tlibc6\tamd64\nii \ttzdata\tall\n";

    let known = known_of_arch(shown, "i386", "amd64");

    assert_eq!(
        known,
        [
            ("libc6:i386".to_owned(), Known::Installed),
            ("libgl1:i386".to_owned(), Known::ConfigFiles)
        ]
        .into()
    );
}

#[test]
fn dkms_status_says_whether_the_module_is_added_and_for_which_kernels_it_is_installed() {
    let shown = "hid-xpadneo/0.10.5, 7.0.0-42-generic, x86_64: installed\n\
                 hid-xpadneo/0.10.5, 7.0.0-43-generic, x86_64: built\n\
                 hid-xpadneo/0.10.4, 7.0.0-43-generic, x86_64: installed\n\
                 tuxedo-drivers/4.25.0, 7.0.0-43-generic, x86_64: installed (Original modules exist)\n";

    let status = dkms_status(shown, "hid-xpadneo", "0.10.5");
    let added = dkms_status("hid-xpadneo/0.10.5: added\n", "hid-xpadneo", "0.10.5");
    let none = dkms_status("", "hid-xpadneo", "0.10.5");

    assert!(status.added);
    assert_eq!(status.installed, ["7.0.0-42-generic".to_owned()].into());
    assert!(added.added && added.installed.is_empty());
    assert!(!none.added);
}

#[test]
fn the_signing_key_and_certificate_are_what_dkms_configuration_sets_last() {
    let framework = "# mok_certificate=/var/lib/dkms/mok.pub\nmok_signing_key=/keys/old.key\n";
    let ours = "mok_signing_key=\"/keys/mok.key\"\nmok_certificate='/keys/mok.der'\n";
    let variable = "mok_certificate=/keys/$kernelver.der\n";
    let values = |key: &str, certificate: &str| Named::Values {
        key: key.to_owned(),
        certificate: certificate.to_owned(),
    };

    assert_eq!(signing_named([""].into_iter()), values("", ""));
    assert_eq!(signing_named([framework].into_iter()), values("/keys/old.key", ""));
    assert_eq!(
        signing_named([framework, ours].into_iter()),
        values("/keys/mok.key", "/keys/mok.der")
    );
    assert_eq!(signing_named([ours, variable].into_iter()), Named::Variable);
}

#[test]
fn dkms_signs_with_shim_signeds_mok_without_a_key_and_its_own_without_a_certificate() {
    assert_eq!(
        dkms_certificate("", "/keys/mok.der"),
        Path::new("/var/lib/shim-signed/mok/MOK.der")
    );
    assert_eq!(
        dkms_certificate("/keys/mok.key", ""),
        Path::new("/var/lib/dkms/mok.pub")
    );
    assert_eq!(
        dkms_certificate("/keys/mok.key", "/keys/mok.der"),
        Path::new("/keys/mok.der")
    );
}

#[test]
fn a_certificates_serial_reads_as_modinfo_names_the_key() {
    assert_eq!(certificate_serial("serial=1A2b3\n").as_deref(), Some("01:A2:B3"));
    assert_eq!(certificate_serial("no serial"), None);
}

#[test]
fn mokutil_says_whether_the_key_is_enrolled_queued_or_neither() {
    use crate::apps::host::SecureBoot;

    assert_eq!(enrolment("MOK.der is already enrolled\n"), SecureBoot::Enrolled);
    assert_eq!(
        enrolment("MOK.der is already in the enrollment request\n"),
        SecureBoot::Queued
    );
    assert_eq!(enrolment("MOK.der is not enrolled\n"), SecureBoot::NotEnrolled);
    assert_eq!(enrolment("Failed to read MokListRT\n"), SecureBoot::Unknown);
}

#[test]
fn a_certificate_named_through_kernelver_is_the_one_dkms_works_out_for_the_kernel() {
    let dir = tempfile::tempdir().unwrap();
    let sandbox = Sandbox::new(&dir.path().join("sandbox"), &dir.path().join("home"), "kuba");
    let framework = dir.path().join("framework.conf");
    let more = dir.path().join("signing.conf");
    fs::write(
        &framework,
        "mok_certificate=/var/lib/dkms/mok.pub\nmok_signing_key=/keys/mok.key\n",
    )
    .unwrap();
    fs::write(&more, "mok_certificate=\"/keys/$kernelver/$arch.der\"\n").unwrap();

    let out = sandbox
        .command("bash")
        .args(mok_source_args(&[framework, more], "7.0.0-43-generic", "x86_64"))
        .output()
        .unwrap();

    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "/keys/mok.key\0/keys/7.0.0-43-generic/x86_64.der"
    );
}

#[test]
fn an_essential_or_protected_package_is_important_on_any_architecture() {
    let shown = "libgcc-s1\ti386\t\tyes\nlibc6\ti386\t\t\nbash\tamd64\tyes\t\nzenity\tamd64\t\t\n";

    assert_eq!(
        important(shown, "amd64"),
        ["bash".to_owned(), "libgcc-s1:i386".to_owned()].into()
    );
}

#[test]
fn a_modules_users_count_the_devices_bound_to_its_drivers() {
    let dir = tempfile::tempdir().unwrap();
    let module = dir.path().join("module/hid_xpadneo");
    let driver = dir.path().join("bus/hid/drivers/xpadneo");
    fs::create_dir_all(module.join("drivers")).unwrap();
    fs::create_dir_all(&driver).unwrap();
    std::os::unix::fs::symlink(&driver, module.join("drivers/hid:xpadneo")).unwrap();
    std::os::unix::fs::symlink(&module, driver.join("module")).unwrap();
    fs::write(driver.join("bind"), "").unwrap();
    assert_eq!(bound_devices(&module), 0);

    std::os::unix::fs::symlink(dir.path(), driver.join("0005:045E:0B13.0001")).unwrap();

    assert_eq!(bound_devices(&module), 1);
}

#[test]
fn a_variable_dkms_configuration_sets_empty_reads_as_unset() {
    let dir = tempfile::tempdir().unwrap();
    let sandbox = Sandbox::new(&dir.path().join("sandbox"), &dir.path().join("home"), "kuba");
    let framework = dir.path().join("framework.conf");
    fs::write(&framework, "mok_certificate=\"$unset_by_anyone\"\n").unwrap();

    let out = sandbox
        .command("bash")
        .args(mok_source_args(&[framework], "7.0.0-43-generic", "x86_64"))
        .output()
        .unwrap();

    assert_eq!(String::from_utf8_lossy(&out.stdout), "\0");
    assert_eq!(dkms_certificate("", ""), Path::new("/var/lib/shim-signed/mok/MOK.der"));
}
