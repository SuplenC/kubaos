//! The developer tools of the shipped catalog, installed and removed on the fake machine.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use super::super::EngineError;
use super::World;
use super::ai::run;
use super::ai::shipped;
use super::ai::world;
use crate::apps::Catalog;
use crate::apps::Platform;
use crate::apps::action::Kind;
use crate::apps::host::fake::Unit;
use crate::apps::picks::Picks;
use crate::unfree::Unfree;

/// Each its packages, from a vendor's repository, a vendor's `.deb`, Ubuntu's archive or nixpkgs.
const PACKAGED: [&str; 11] = [
    "bruno",
    "dbeaver",
    "distrobox",
    "github-desktop",
    "insomnia",
    "k9s",
    "kubectl",
    "lens",
    "podman-desktop",
    "postman",
    "wine",
];

/// Every developer tool: those, the virtual machines with their groups and units, Android Studio's
/// download and Wireshark with its answer to its question.
const ALL: [&str; 15] = [
    "android-studio",
    "bruno",
    "dbeaver",
    "distrobox",
    "github-desktop",
    "gnome-boxes",
    "insomnia",
    "k9s",
    "kubectl",
    "lens",
    "podman-desktop",
    "postman",
    "virt-manager",
    "wine",
    "wireshark",
];

fn picks(id: &str) -> Picks {
    let mut picks = Picks::default();
    picks.add(id);
    picks
}

/// Each `.deb` of the app served at its pin, as the fake machine downloads it.
fn serve_the_debs(world: &World, id: &str) {
    let catalog = Catalog::load(&world.catalog).unwrap();
    for action in &catalog.get(id).unwrap().actions {
        if let Kind::Deb { package, version, .. } = &action.kind {
            let pin = action.download("amd64").unwrap();
            world.host.state.borrow_mut().downloads.insert(
                pin.url.clone(),
                (pin.sha256.clone(), format!("deb {package} {version}").into_bytes()),
            );
        }
    }
}

#[test]
fn every_packaged_developer_tool_installs_on_the_fake_machine_and_its_removal_leaves_it_as_before() {
    for id in PACKAGED {
        let world = world(id);
        serve_the_debs(&world, id);
        let before = world.host.snapshot();

        let ran = run(&world, |e| e.add(id, None, &picks(id))).unwrap_or_else(|e| panic!("{id}: {e}"));

        let catalog = Catalog::load(&world.catalog).unwrap();
        let app = catalog.get(id).unwrap();
        assert_eq!(ran.len(), 1, "{id}");
        {
            let state = world.host.state.borrow();
            for action in &app.actions {
                match &action.kind {
                    Kind::Apt { packages, repo, .. } => {
                        assert!(packages.iter().all(|p| state.dpkg.contains_key(p)), "{id}");
                        if let Some(repo) = repo {
                            assert!(state.repos.contains(repo), "{id}: {repo} wasn't set up");
                        }
                    }
                    Kind::Deb { package, version, .. } => assert_eq!(&state.dpkg[package], version, "{id}"),
                    Kind::Nix { packages, .. } => assert!(packages.iter().all(|p| state.nix.contains(p)), "{id}"),
                    // The icon of the app's folder, in the theme.
                    Kind::File { to, .. } if to.starts_with("~/.local/share/icons/") => {
                        let icon = world.host_home().join(to.trim_start_matches("~/"));
                        assert!(icon.is_file(), "{id}: {}", icon.display());
                    }
                    other => panic!("{id}: a packaged developer tool has no {other:?}"),
                }
            }
        }

        let gone = run(&world, |e| e.remove_with(id, &Picks::default())).unwrap_or_else(|e| panic!("{id}: {e}"));

        assert_eq!(gone.len(), 1, "{id}");
        assert_eq!(world.host.snapshot(), before, "{id}");
        assert_eq!(world.journals(id), (None, None), "{id}");
    }
}

#[test]
fn which_developer_tools_a_machine_can_have_follows_its_architecture_and_the_unfree_setting() {
    let catalog = Catalog::load(&shipped("wine").join("..")).unwrap();
    let on = |arch: &str, unfree: Unfree| -> Vec<&str> {
        ALL.into_iter()
            .filter(|id| catalog.available(catalog.get(id).unwrap(), None, Platform::new(arch, unfree)))
            .collect()
    };

    assert_eq!(on("amd64", Unfree::Allowed), ALL);
    // DBeaver's, Bruno's, Lens' and WineHQ's repositories, Insomnia's .deb and Android Studio's
    // tarball are amd64 alone, and the virtual machines' firmware is amd64's.
    assert_eq!(
        on("arm64", Unfree::Allowed),
        [
            "distrobox",
            "github-desktop",
            "k9s",
            "kubectl",
            "podman-desktop",
            "postman",
            "wireshark"
        ]
    );
    let free = on("amd64", Unfree::Refused);
    assert!(!free.contains(&"postman"), "nixpkgs marks Postman unfree");
    assert_eq!(free.len(), ALL.len() - 1);
}

/// libvirt as its package leaves it: libvirtd enabled at boot and running, its socket there.
fn libvirt_as_packaged(world: &World) {
    let mut state = world.host.state.borrow_mut();
    state.units.insert(
        "system:libvirtd.service".to_owned(),
        Unit {
            exists: true,
            enabled: true,
            active: true,
            ..Unit::default()
        },
    );
    state.units.insert(
        "system:libvirtd.socket".to_owned(),
        Unit {
            exists: true,
            ..Unit::default()
        },
    );
    // systemd makes kvm on every machine.
    state.groups.insert("kvm".to_owned(), [].into());
}

#[test]
fn virt_manager_leaves_libvirt_starting_at_boot_and_puts_its_groups_back_as_it_goes() {
    let world = world("virt-manager");
    libvirt_as_packaged(&world);
    let before = world.host.snapshot();
    let units = world.host.state.borrow().units.clone();

    run(&world, |e| e.add("virt-manager", None, &picks("virt-manager"))).unwrap();

    {
        let state = world.host.state.borrow();
        assert_eq!(state.units, units, "libvirtd starts at boot, as Ubuntu sets it");
        for group in ["libvirt", "kvm"] {
            assert!(state.groups[group].contains("kuba"), "{group}");
        }
    }

    run(&world, |e| e.remove_with("virt-manager", &Picks::default())).unwrap();

    let state = world.host.state.borrow();
    assert!(state.groups.contains_key("kvm"), "kvm is systemd's, and stays");
    assert!(!state.groups.contains_key("libvirt"));
    drop(state);
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn gnome_boxes_needs_kvm_and_leaves_the_libvirtd_its_package_brings_as_it_is() {
    let world = world("gnome-boxes");
    libvirt_as_packaged(&world);
    let before = world.host.snapshot();
    let units = world.host.state.borrow().units.clone();

    run(&world, |e| e.add("gnome-boxes", None, &picks("gnome-boxes"))).unwrap();

    {
        let state = world.host.state.borrow();
        assert_eq!(state.units, units, "libvirtd starts at boot, as Ubuntu sets it");
        assert!(state.groups["kvm"].contains("kuba"));
        assert!(!state.groups.contains_key("libvirt"), "Boxes runs its machines as you");
    }

    run(&world, |e| e.remove_with("gnome-boxes", &Picks::default())).unwrap();

    assert_eq!(world.host.snapshot(), before);
}

/// Where the fake debconf keeps its answers: outside the machine's snapshot, as debconf's
/// database is no file of an app's.
fn debconf(world: &World) -> PathBuf {
    world.host_home().join("../../debconf")
}

/// debconf as `debconf-set-selections`, `debconf-communicate` and `dpkg-reconfigure` treat one
/// package's questions: an answer set is marked seen, as `debconf-set-selections` marks it, and
/// configuring the package again marks its questions unseen, as `dpkg-reconfigure` does; and dpkg
/// saying whether wireshark-common is configured.
fn wireshark_world(installed: bool) -> World {
    let world = world("wireshark");
    let answers = debconf(&world);
    fs::create_dir_all(&answers).unwrap();
    let dir = answers.display();
    let sandbox = world.host.sandbox();
    sandbox.stub(
        "debconf-set-selections",
        &format!(
            "read -r _ question _ value\nf='{dir}'/\"${{question##*/}}\"\n\
             printf '%s\\n' \"$value\" >\"$f\"; echo true >\"$f.seen\""
        ),
    );
    sandbox.stub(
        "debconf-communicate",
        &format!(
            "read -r command question flag value\nf='{dir}'/\"${{question##*/}}\"\n\
             if [ ! -f \"$f\" ]; then printf \"10 %s doesn't exist\\n\" \"$question\"; exit 10; fi\n\
             case $command in\n\
             GET) printf '0 %s\\n' \"$(cat \"$f\")\" ;;\n\
             FGET) printf '0 %s\\n' \"$(cat \"$f.seen\" 2>/dev/null || echo false)\" ;;\n\
             FSET) echo \"$value\" >\"$f.seen\"; echo 0 ;;\n\
             esac"
        ),
    );
    sandbox.stub("dpkg-reconfigure", &format!("rm -f '{dir}'/*.seen"));
    if installed {
        sandbox.stub("dpkg-query", "printf 'ii '");
    }
    world
}

/// The user's own answer to wireshark-common's question, given before KubaOS installs it.
fn answered(world: &World, value: &str) {
    let question = debconf(world).join("install-setuid");
    fs::write(&question, format!("{value}\n")).unwrap();
    fs::write(question.with_extension("seen"), "true\n").unwrap();
}

fn answer(world: &World) -> Option<String> {
    fs::read_to_string(debconf(world).join("install-setuid"))
        .ok()
        .map(|a| a.trim().to_owned())
}

#[test]
fn wireshark_answers_its_question_yes_before_its_package_lands_and_no_once_it_is_gone() {
    let world = wireshark_world(false);
    // The package makes the group as it is configured to let it capture.
    world
        .host
        .state
        .borrow_mut()
        .groups
        .insert("wireshark".to_owned(), [].into());
    let before = world.host.snapshot();

    run(&world, |e| e.add("wireshark", None, &picks("wireshark"))).unwrap();

    assert_eq!(answer(&world).as_deref(), Some("true"));
    {
        let state = world.host.state.borrow();
        assert!(state.dpkg.contains_key("wireshark"));
        assert!(state.groups["wireshark"].contains("kuba"));
        let at = |what: &str| state.log.iter().position(|l| l.contains(what)).unwrap();
        assert!(at("run ./capture") < at("wireshark"), "{:#?}", state.log);
    }
    let calls = world.host.sandbox().calls();
    assert!(
        !calls.iter().any(|c| c.starts_with("dpkg-reconfigure")),
        "nothing to configure again before the package: {calls:#?}"
    );

    run(&world, |e| e.remove_with("wireshark", &Picks::default())).unwrap();

    assert_eq!(answer(&world).as_deref(), Some("false"), "back to its default");
    let seen = fs::read_to_string(debconf(&world).join("install-setuid.seen")).unwrap();
    assert_eq!(seen.trim(), "false", "unseen, so a later install answers it again");
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn a_wireshark_the_user_answered_yes_for_keeps_the_answer_when_it_goes() {
    let world = wireshark_world(false);
    answered(&world, "true");

    run(&world, |e| e.add("wireshark", None, &picks("wireshark"))).unwrap();
    run(&world, |e| e.remove_with("wireshark", &Picks::default())).unwrap();

    assert_eq!(answer(&world).as_deref(), Some("true"));
    let calls = world.host.sandbox().calls();
    assert!(
        !calls.iter().any(|c| c.starts_with("debconf-set-selections")),
        "{calls:#?}"
    );
}

#[test]
fn a_wireshark_the_user_answered_no_for_keeps_the_no() {
    let world = wireshark_world(true);
    answered(&world, "false");

    run(&world, |e| e.add("wireshark", None, &picks("wireshark"))).unwrap();

    assert_eq!(answer(&world).as_deref(), Some("false"), "the user's no stays");
    let calls = world.host.sandbox().calls();
    assert!(
        !calls
            .iter()
            .any(|c| c.starts_with("debconf-set-selections") || c.starts_with("dpkg-reconfigure")),
        "{calls:#?}"
    );

    run(&world, |e| e.remove_with("wireshark", &Picks::default())).unwrap();

    assert_eq!(answer(&world).as_deref(), Some("false"));
    let calls = world.host.sandbox().calls();
    assert!(
        !calls.iter().any(|c| c.starts_with("debconf-set-selections")),
        "{calls:#?}"
    );
}

#[test]
fn a_wireshark_already_configured_is_configured_again_with_each_answer() {
    let world = wireshark_world(true);
    fs::write(debconf(&world).join("install-setuid"), "false\n").unwrap();

    run(&world, |e| e.add("wireshark", None, &picks("wireshark"))).unwrap();

    assert_eq!(answer(&world).as_deref(), Some("true"));
    let reconfigured = || {
        world
            .host
            .sandbox()
            .calls()
            .iter()
            .filter(|c| *c == "dpkg-reconfigure --frontend noninteractive wireshark-common")
            .count()
    };
    assert_eq!(reconfigured(), 1);

    run(&world, |e| e.remove_with("wireshark", &Picks::default())).unwrap();

    assert_eq!(answer(&world).as_deref(), Some("false"));
    assert_eq!(reconfigured(), 2, "dumpcap is root's alone again");
}

const STUDIO: &str = "android-studio";

/// A Studio release, packed as Google packs it (`android-studio/bin/`).
fn studio_tarball() -> Vec<u8> {
    let src = tempfile::tempdir().unwrap();
    let bin = src.path().join("android-studio/bin");
    fs::create_dir_all(&bin).unwrap();
    fs::write(bin.join("studio"), "#!/bin/sh\n").unwrap();
    fs::set_permissions(bin.join("studio"), fs::Permissions::from_mode(0o755)).unwrap();
    let archive = src.path().join("studio.tar.gz");
    let made = std::process::Command::new("tar")
        .arg("-czf")
        .arg(&archive)
        .arg("-C")
        .arg(src.path())
        .arg("android-studio")
        .status()
        .unwrap();
    assert!(made.success());
    fs::read(archive).unwrap()
}

/// The shipped Studio's world, its release served at the URL and sha256 its folder pins.
fn studio_world() -> World {
    let world = world(STUDIO);
    let catalog = Catalog::load(&world.catalog).unwrap();
    let Kind::Download { amd64, .. } = &catalog.get(STUDIO).unwrap().actions[0].kind else {
        panic!("Studio's first step is its download");
    };
    let pin = amd64.as_ref().unwrap();
    world
        .host
        .state
        .borrow_mut()
        .downloads
        .insert(pin.url.clone(), (pin.sha256.clone(), studio_tarball()));
    world
}

#[test]
fn android_studio_unpacks_where_it_updates_itself_and_its_removal_keeps_the_sdk_and_projects() {
    let world = studio_world();
    let home = world.host_home();
    fs::create_dir_all(home.join(".local/share/applications")).unwrap();
    let before = world.host.snapshot();

    run(&world, |e| e.add(STUDIO, None, &picks(STUDIO))).unwrap();

    let studio = home.join(".local/share/android-studio/bin/studio");
    assert!(studio.is_file(), "the release unpacks into Studio's own folder");
    let entry = fs::read_to_string(home.join(".local/share/applications/jetbrains-studio.desktop")).unwrap();
    assert!(
        entry.contains(&format!("Exec=\"{}\" %f\n", studio.display())),
        "{entry}"
    );
    // What Studio and its SDK manager make as you use them.
    let sdk = home.join("Android/Sdk/platform-tools/adb");
    fs::create_dir_all(sdk.parent().unwrap()).unwrap();
    fs::write(&sdk, "").unwrap();

    run(&world, |e| e.remove_with(STUDIO, &Picks::default())).unwrap();

    assert!(!studio.exists());
    assert!(sdk.is_file(), "the SDK stays");
    assert_eq!(world.journals(STUDIO), (None, None));
    fs::remove_dir_all(home.join("Android")).unwrap();
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn android_studio_running_is_left_alone_until_it_is_quit() {
    let world = studio_world();
    let before = world.host.snapshot();
    // pgrep as the machine answers it while Studio's Java runs from its folder, its pattern matched
    // against the command line as `pgrep -f` matches.
    let java = world.host_home().join(".local/share/android-studio/jbr/bin/java");
    world.host.sandbox().stub(
        "pgrep",
        &format!(
            "full=; for a; do [ \"$a\" = -f ] && full=1; pattern=$a; done\n\
             [ -n \"$full\" ] && printf '%s -classpath x\\n' '{}' | grep -Eq -- \"$pattern\"",
            java.display()
        ),
    );

    let refused = run(&world, |e| e.add(STUDIO, None, &Picks::default())).unwrap_err();

    assert!(
        matches!(&refused, EngineError::NotReady(says) if says.contains("quit it")),
        "{refused}"
    );
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn a_release_bumping_studios_pin_never_downloads_over_the_studio_that_updated_itself() {
    let world = studio_world();
    run(&world, |e| e.add(STUDIO, None, &picks(STUDIO))).unwrap();
    let program = world.host_home().join(".local/share/android-studio/bin/studio");
    let updated = "#!/bin/sh\n# Studio, patched by its own updater\n";
    fs::write(&program, updated).unwrap();
    // A later release of KubaOS pins an older Studio than the one it updated itself to.
    let bumped_url = "https://dl.google.com/dl/android/studio/ide-zips/2026.1.1.1/android-studio-older-linux.tar.gz";
    let bumped_sha = "c".repeat(64);
    let text = fs::read_to_string(world.catalog.join(STUDIO).join("app.toml")).unwrap();
    let catalog = Catalog::load(&world.catalog).unwrap();
    let Kind::Download { version, amd64, .. } = &catalog.get(STUDIO).unwrap().actions[0].kind else {
        panic!("Studio's first step is its download");
    };
    let pin = amd64.as_ref().unwrap();
    world.write_app(
        STUDIO,
        &text
            .replace(&format!("version = \"{version}\""), "version = \"2026.1.1.1\"")
            .replace(&pin.url, bumped_url)
            .replace(&pin.sha256, &bumped_sha),
    );
    world
        .host
        .state
        .borrow_mut()
        .downloads
        .insert(bumped_url.to_owned(), (bumped_sha, studio_tarball()));

    run(&world, |e| e.add(STUDIO, None, &picks(STUDIO))).unwrap();

    assert_eq!(fs::read_to_string(&program).unwrap(), updated);
    let fetched = world.host.state.borrow().log.iter().any(|l| l.ends_with(bumped_url));
    assert!(!fetched, "the bumped pin is never downloaded");
}

#[test]
fn the_flat_repositories_name_their_root_as_their_suite() {
    // DBeaver's and Kubernetes' repositories have no dists/: apt reads a flat repository's index
    // from the URI itself, which a suite of `/` and no components says.
    let sources = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../system/files/apt/sources.list.d");
    for repo in ["dbeaver", "kubernetes"] {
        let text = fs::read_to_string(sources.join(format!("{repo}.sources"))).unwrap();
        let suites: Vec<&str> = text.lines().filter_map(|l| l.strip_prefix("Suites: ")).collect();
        assert_eq!(suites, ["/"], "{repo}");
        assert!(!text.lines().any(|l| l.starts_with("Components:")), "{repo}");
    }
}
