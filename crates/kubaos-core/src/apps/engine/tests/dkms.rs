//! A kernel module DKMS builds from a pinned source for every kernel, signed with the machine's
//! Machine Owner Key, and again for each kernel that comes later.

use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::EngineError;
use super::World;
use crate::apps::host::SecureBoot;
use crate::apps::journal::Journal;
use crate::apps::journal::State;
use crate::update::event::UpdateEvent;

/// DKMS and the headers first, then the module, as the xpadneo app is.
const PAD: &str = r#"
name = "Pad"
summary = "A gamepad's driver"
category = "games"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"

[[action]]
name = "Install DKMS and the kernel's headers"
kind = "apt"
packages = ["dkms", "linux-headers-generic"]

[[action]]
name = "Build the driver for each kernel"
kind = "dkms"
module = "hid-pad"
version = "1.0"
url = "https://example.com/pad-1.0.tar.gz"
sha256 = "2222222222222222222222222222222222222222222222222222222222222222"
strip = 1
folder = "hid-pad"
conf = "dkms.conf"
"#;

/// The same, with a group between, whose removal the tests can make fail after the module went.
const GROUPED: &str = r#"
name = "Grouped pad"
summary = "A gamepad's driver with a group"
category = "games"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"

[[action]]
name = "Install DKMS and the kernel's headers"
kind = "apt"
packages = ["dkms", "linux-headers-generic"]

[[action]]
name = "Let you use it"
kind = "group"
group = "players"

[[action]]
name = "Build the driver for each kernel"
kind = "dkms"
module = "hid-pad"
version = "1.0"
url = "https://example.com/pad-1.0.tar.gz"
sha256 = "2222222222222222222222222222222222222222222222222222222222222222"
strip = 1
folder = "hid-pad"
conf = "dkms.conf"
"#;

const CONF: &str = "PACKAGE_NAME=\"hid-pad\"\nPACKAGE_VERSION=\"@VERSION@\"\nAUTOINSTALL=\"yes\"\n";

const KERNEL: &str = "7.0.0-43-generic";

const MOK: &str = "1A:21:AB:C9";

/// A machine with one kernel and its headers, DKMS's MOK, and the module's source served: a
/// tarball whose one folder holds the module's folder and a file beside it, made by the tar of
/// the machine's sandbox.
fn pad(apps: &[(&str, &str)]) -> World {
    let world = World::new(apps);
    for (id, _) in apps {
        let conf = world.catalog.join(id).join("dkms.conf");
        fs::write(&conf, CONF).unwrap();
        fs::set_permissions(&conf, fs::Permissions::from_mode(0o644)).unwrap();
    }
    let src = tempfile::tempdir().unwrap();
    let top = src.path().join("pad-1.0");
    fs::create_dir_all(top.join("hid-pad/src")).unwrap();
    fs::write(top.join("hid-pad/src/pad.c"), "// the driver\n").unwrap();
    fs::write(top.join("README.md"), "not the module's\n").unwrap();
    let archive = src.path().join("pad.tar.gz");
    let made = world
        .host
        .sandbox()
        .command("tar")
        .arg("-czf")
        .arg(&archive)
        .arg("-C")
        .arg(src.path())
        .arg("pad-1.0")
        .status()
        .unwrap();
    assert!(made.success());
    let mut state = world.host.state.borrow_mut();
    state.downloads.insert(
        "https://example.com/pad-1.0.tar.gz".to_owned(),
        ("2".repeat(64), fs::read(archive).unwrap()),
    );
    state.kernels.insert(KERNEL.to_owned());
    state.mok = Some(MOK.to_owned());
    drop(state);
    world
}

/// Whether DKMS has the module installed for `kernel`, signed with the MOK.
fn signed(world: &World, kernel: &str) -> bool {
    world
        .host
        .state
        .borrow()
        .dkms
        .get("hid-pad/1.0")
        .and_then(|kernels| kernels.get(kernel))
        .is_some_and(|key| key.as_deref() == Some(MOK))
}

fn notes(events: &super::Events) -> Vec<String> {
    events
        .0
        .iter()
        .filter_map(|e| match e {
            UpdateEvent::Note { text } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn an_install_puts_the_source_in_place_with_its_conf_and_builds_it_signed_for_each_kernel() {
    let world = pad(&[("pad", PAD)]);
    world
        .host
        .state
        .borrow_mut()
        .kernels
        .insert("7.0.0-42-generic".to_owned());

    world.install("pad", true).0.unwrap();

    let source = world.system_path("/usr/src/hid-pad-1.0");
    assert_eq!(
        fs::read_to_string(source.join("dkms.conf")).unwrap(),
        "PACKAGE_NAME=\"hid-pad\"\nPACKAGE_VERSION=\"1.0\"\nAUTOINSTALL=\"yes\"\n"
    );
    assert!(source.join("src/pad.c").is_file());
    assert!(!source.join("README.md").exists(), "only the module's folder");
    assert!(signed(&world, KERNEL));
    assert!(signed(&world, "7.0.0-42-generic"));
    let root = Journal::read(&world.root_journals, "pad").unwrap().unwrap();
    let entry = root.entry(2).unwrap();
    assert_eq!(entry.state, State::Done);
    assert!(
        entry.claims.iter().any(|c| c.claim == "dkms:hid-pad/1.0" && c.owned),
        "{entry:?}"
    );
}

#[test]
fn removing_the_app_unloads_the_unused_module_and_takes_it_from_dkms_and_the_source() {
    let world = pad(&[("pad", PAD)]);
    let before = world.host.snapshot();
    world.install("pad", true).0.unwrap();
    world.host.state.borrow_mut().loaded.insert("hid_pad".to_owned(), 0);

    world.remove("pad").0.unwrap();

    assert!(!world.system_path("/usr/src/hid-pad-1.0").exists());
    assert_eq!(world.host.snapshot(), before);
    assert_eq!(world.journals("pad"), (None, None));
}

#[test]
fn a_module_in_use_stays_loaded_until_the_restart_and_the_removal_says_so() {
    let world = pad(&[("pad", PAD)]);
    world.install("pad", true).0.unwrap();
    world.host.state.borrow_mut().loaded.insert("hid_pad".to_owned(), 1);

    let (result, events) = world.remove("pad");

    result.unwrap();
    assert!(world.host.state.borrow().dkms.is_empty());
    assert_eq!(world.host.state.borrow().loaded.get("hid_pad"), Some(&1));
    assert!(
        notes(&events)
            .contains(&"hid-pad stays loaded until the machine restarts, as something uses it now.".to_owned()),
        "{:#?}",
        events.0
    );
}

#[test]
fn a_module_built_by_hand_is_found_and_left_by_the_removal() {
    let world = pad(&[("pad", PAD)]);
    let source = world.system_path("/usr/src/hid-pad-1.0");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("dkms.conf"), "PACKAGE_NAME=\"hid-pad\"\n").unwrap();
    world.host.state.borrow_mut().dkms.insert(
        "hid-pad/1.0".to_owned(),
        [(KERNEL.to_owned(), Some(MOK.to_owned()))].into(),
    );

    world.install("pad", true).0.unwrap();

    let root = Journal::read(&world.root_journals, "pad").unwrap().unwrap();
    assert_eq!(root.entry(2).unwrap().state, State::Found);

    world.remove("pad").0.unwrap();

    assert!(source.join("dkms.conf").is_file());
    assert!(world.host.state.borrow().dkms.contains_key("hid-pad/1.0"));
}

#[test]
fn a_new_kernel_dkms_left_without_the_module_or_unsigned_gets_it_signed_at_the_next_run() {
    let world = pad(&[("pad", PAD)]);
    world.install("pad", true).0.unwrap();
    // DKMS's own autoinstall at a kernel's install, with its signing set up wrong.
    {
        let mut state = world.host.state.borrow_mut();
        state.kernels.insert("7.0.0-44-generic".to_owned());
        state
            .dkms
            .get_mut("hid-pad/1.0")
            .unwrap()
            .insert("7.0.0-44-generic".to_owned(), None);
        state.kernels.insert("7.0.0-45-generic".to_owned());
    }

    world.install("pad", false).0.unwrap();

    for kernel in [KERNEL, "7.0.0-44-generic", "7.0.0-45-generic"] {
        assert!(signed(&world, kernel), "{kernel}");
    }
}

#[test]
fn a_module_dkms_installs_unsigned_fails_the_install_and_leaves_the_machine_as_it_was() {
    let world = pad(&[("pad", PAD)]);
    world.host.state.borrow_mut().fail.insert("dkms-sign".to_owned());
    let before = world.host.snapshot();

    let (result, _) = world.install("pad", true);

    let Err(EngineError::Step { name, source }) = result else {
        panic!("the install went through: {result:?}");
    };
    assert_eq!(name, "Build the driver for each kernel");
    assert!(source.to_string().contains("without signing it"), "{source}");
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn a_machine_without_kernel_headers_is_refused_the_module() {
    let world = pad(&[("pad", PAD)]);
    world.host.state.borrow_mut().kernels.clear();
    let before = world.host.snapshot();

    let (result, _) = world.install("pad", true);

    let Err(EngineError::Step { source, .. }) = result else {
        panic!("the install went through: {result:?}");
    };
    assert!(source.to_string().contains("no kernel has its headers"), "{source}");
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn with_secure_boot_on_and_the_mok_not_enrolled_the_install_says_what_to_do_at_the_next_boot() {
    let world = pad(&[("pad", PAD)]);
    world.host.state.borrow_mut().secure_boot = SecureBoot::NotEnrolled;

    let (result, events) = world.install("pad", true);

    result.unwrap();
    let said = notes(&events);
    assert!(
        said.iter()
            .any(|n| n.contains("sudo mokutil --import /var/lib/shim-signed/mok/MOK.der") && n.contains("Enroll MOK")),
        "{said:#?}"
    );

    world.host.state.borrow_mut().secure_boot = SecureBoot::Enrolled;
    let (result, events) = world.install("pad", true);

    result.unwrap();
    assert!(notes(&events).is_empty(), "{:#?}", events.0);
}

#[test]
fn a_removal_that_fails_after_the_module_went_builds_it_again() {
    let world = pad(&[("grouped", GROUPED)]);
    world.install("grouped", true).0.unwrap();
    let installed = world.host.snapshot();
    world.host.state.borrow_mut().fail.insert("groupdel:players".to_owned());

    let (result, _) = world.remove("grouped");

    assert!(
        matches!(result, Err(EngineError::RemovalRolledBack { ref name, .. }) if name == "Let you use it"),
        "{result:?}"
    );
    assert_eq!(world.host.snapshot(), installed);
    assert!(signed(&world, KERNEL));
}

#[test]
fn a_new_version_takes_the_old_ones_place_and_its_source() {
    let world = pad(&[("pad", PAD)]);
    world.install("pad", true).0.unwrap();
    let source = fs::read(world.system_path("/usr/src/hid-pad-1.0/src/pad.c")).unwrap();
    let downloads = world.host.state.borrow().downloads["https://example.com/pad-1.0.tar.gz"].clone();
    world.host.state.borrow_mut().downloads.insert(
        "https://example.com/pad-1.1.tar.gz".to_owned(),
        ("3".repeat(64), downloads.1),
    );
    world.write_app(
        "pad",
        &PAD.replace("version = \"1.0\"", "version = \"1.1\"")
            .replace("pad-1.0.tar.gz", "pad-1.1.tar.gz")
            .replace(&"2".repeat(64), &"3".repeat(64)),
    );

    world.install("pad", true).0.unwrap();

    assert_eq!(
        fs::read(world.system_path("/usr/src/hid-pad-1.1/src/pad.c")).unwrap(),
        source
    );
    assert!(!world.system_path("/usr/src/hid-pad-1.0").exists());
    let state = world.host.state.borrow();
    assert_eq!(state.dkms.keys().collect::<Vec<_>>(), ["hid-pad/1.1"]);
}

#[test]
fn a_new_source_at_the_same_version_is_built_again_for_every_kernel() {
    let world = pad(&[("pad", PAD)]);
    world
        .host
        .state
        .borrow_mut()
        .kernels
        .insert("7.0.0-42-generic".to_owned());
    world.install("pad", true).0.unwrap();
    let tarball = world.host.state.borrow().downloads["https://example.com/pad-1.0.tar.gz"]
        .1
        .clone();
    world.host.state.borrow_mut().downloads.insert(
        "https://example.com/pad-1.0-fixed.tar.gz".to_owned(),
        ("4".repeat(64), tarball),
    );
    world.write_app(
        "pad",
        &PAD.replace("pad-1.0.tar.gz", "pad-1.0-fixed.tar.gz")
            .replace(&"2".repeat(64), &"4".repeat(64)),
    );
    let from = world.host.state.borrow().log.len();

    world.install("pad", true).0.unwrap();

    let log = world.host.state.borrow().log[from..].to_vec();
    for kernel in [KERNEL, "7.0.0-42-generic"] {
        assert!(
            log.contains(&format!("dkms install hid-pad/1.0 {kernel}")),
            "{kernel}: {log:#?}"
        );
    }
}

#[test]
fn a_module_with_a_controller_bound_to_its_driver_stays_loaded() {
    let world = pad(&[("pad", PAD)]);
    world.install("pad", true).0.unwrap();
    {
        let mut state = world.host.state.borrow_mut();
        state.loaded.insert("hid_pad".to_owned(), 0);
        state.bound.insert("hid_pad".to_owned(), 1);
    }

    let (result, events) = world.remove("pad");

    result.unwrap();
    assert!(world.host.state.borrow().loaded.contains_key("hid_pad"));
    assert!(
        notes(&events)
            .contains(&"hid-pad stays loaded until the machine restarts, as something uses it now.".to_owned()),
        "{:#?}",
        events.0
    );
}

#[test]
fn a_version_bump_never_unloads_the_module_it_replaced() {
    let world = pad(&[("pad", PAD)]);
    world.install("pad", true).0.unwrap();
    world.host.state.borrow_mut().loaded.insert("hid_pad".to_owned(), 0);
    let tarball = world.host.state.borrow().downloads["https://example.com/pad-1.0.tar.gz"]
        .1
        .clone();
    world.host.state.borrow_mut().downloads.insert(
        "https://example.com/pad-1.1.tar.gz".to_owned(),
        ("3".repeat(64), tarball),
    );
    world.write_app(
        "pad",
        &PAD.replace("version = \"1.0\"", "version = \"1.1\"")
            .replace("pad-1.0.tar.gz", "pad-1.1.tar.gz")
            .replace(&"2".repeat(64), &"3".repeat(64)),
    );

    world.install("pad", true).0.unwrap();

    let state = world.host.state.borrow();
    assert!(state.loaded.contains_key("hid_pad"), "{:#?}", state.log);
    assert_eq!(state.dkms.keys().collect::<Vec<_>>(), ["hid-pad/1.1"]);
}

#[test]
fn a_kernel_the_conf_leaves_out_is_skipped_with_a_note() {
    let world = pad(&[("pad", PAD)]);
    {
        let mut state = world.host.state.borrow_mut();
        state.kernels.insert("5.10.0-old".to_owned());
        state.excluded.insert("5.10.0-old".to_owned());
    }

    let (result, events) = world.install("pad", true);

    result.unwrap();
    assert!(signed(&world, KERNEL));
    assert!(
        notes(&events)
            .contains(&"hid-pad isn't built for 5.10.0-old: its dkms.conf leaves that kernel out.".to_owned()),
        "{:#?}",
        events.0
    );
}
