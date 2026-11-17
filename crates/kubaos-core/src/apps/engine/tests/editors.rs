//! The editors and terminals of the shipped catalog, installed and removed on the fake machine.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::super::EngineError;
use super::ai::run;
use super::ai::shipped;
use super::ai::world;
use crate::apps::Catalog;
use crate::apps::Platform;
use crate::apps::action::Kind;
use crate::apps::picks::Picks;
use crate::unfree::Unfree;

/// Each a package, from a vendor's repository, Ubuntu's archive or nixpkgs.
const PACKAGED: [&str; 9] = [
    "alacritty",
    "emacs",
    "helix",
    "kitty",
    "neovide",
    "sublime-text",
    "vscode",
    "wezterm",
    "zed",
];

const TERMINALS: [&str; 3] = ["alacritty", "kitty", "wezterm"];

#[test]
fn every_packaged_editor_and_terminal_installs_on_the_fake_machine_and_its_removal_leaves_it_as_before() {
    for id in PACKAGED {
        let world = world(id);
        let before = world.host.snapshot();
        let mut picks = Picks::default();
        picks.add(id);

        let ran = run(&world, |e| e.add(id, None, &picks)).unwrap_or_else(|e| panic!("{id}: {e}"));

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
                    Kind::Nix { packages, .. } => assert!(packages.iter().all(|p| state.nix.contains(p)), "{id}"),
                    other => panic!("{id}: a packaged editor is its package, not {other:?}"),
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
fn a_catalog_terminal_is_its_nix_package_alone_so_ghostty_stays_the_default() {
    // An apt terminal registers itself as x-terminal-emulator, and any other step could reach the
    // session's binds; nixpkgs' package changes nothing outside the user's profile.
    let catalog = Catalog::load(&shipped("kitty").join("..")).unwrap();

    for id in TERMINALS {
        let app = catalog.get(id).unwrap();
        let kinds: Vec<&Kind> = app.all_actions().map(|a| &a.kind).collect();

        assert!(matches!(kinds[..], [Kind::Nix { .. }]), "{id}: {kinds:?}");
    }
}

#[test]
fn every_editor_and_terminal_installs_on_either_architecture_with_unfree_packages_refused() {
    let catalog = Catalog::load(&shipped("vscode").join("..")).unwrap();
    let all: Vec<&str> = PACKAGED.into_iter().chain(["jetbrains-toolbox"]).collect();

    for arch in ["amd64", "arm64"] {
        let on: Vec<&str> = all
            .iter()
            .copied()
            .filter(|id| catalog.available(catalog.get(id).unwrap(), None, Platform::new(arch, Unfree::Refused)))
            .collect();

        assert_eq!(on, all, "{arch}");
    }
}

#[test]
fn vscodes_sources_file_is_the_one_its_package_writes_word_for_word() {
    // What VS Code 1.146's postinst writes, its architecture the package's; a different file
    // would be the user's source to its postrm, and one more source of the same repository.
    let postinst = "### THIS FILE IS AUTOMATICALLY CONFIGURED ###\n\
                    # You may comment out this entry, but any other modifications may be lost.\n\
                    Types: deb\n\
                    URIs: https://packages.microsoft.com/repos/code\n\
                    Suites: stable\n\
                    Components: main\n\
                    Architectures: @ARCH@\n\
                    Signed-By: /usr/share/keyrings/microsoft.gpg\n";
    let ours = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../system/files/apt/sources.list.d/vscode.sources");

    assert_eq!(fs::read_to_string(ours).unwrap(), postinst);
}

const TOOLBOX: &str = "jetbrains-toolbox";

/// A Toolbox release, packed as its vendor packs it (`jetbrains-toolbox-<build>/bin/`), whose
/// program makes the entries the real one makes as it first starts.
fn toolbox_tarball() -> Vec<u8> {
    let src = tempfile::tempdir().unwrap();
    let bin = src.path().join("jetbrains-toolbox-3.9.1.89198/bin");
    fs::create_dir_all(&bin).unwrap();
    let program = "#!/bin/sh\n\
                   mkdir -p \"$HOME/.local/share/applications\" \"$HOME/.config/autostart\"\n\
                   printf '[Desktop Entry]\\nName=JetBrains Toolbox\\n' >\"$HOME/.local/share/applications/jetbrains-toolbox.desktop\"\n\
                   printf '[Desktop Entry]\\nName=JetBrains Toolbox\\n' >\"$HOME/.config/autostart/jetbrains-toolbox.desktop\"\n";
    fs::write(bin.join("jetbrains-toolbox"), program).unwrap();
    fs::set_permissions(bin.join("jetbrains-toolbox"), fs::Permissions::from_mode(0o755)).unwrap();
    let archive = src.path().join("toolbox.tar.gz");
    let made = std::process::Command::new("tar")
        .arg("-czf")
        .arg(&archive)
        .arg("-C")
        .arg(src.path())
        .arg("jetbrains-toolbox-3.9.1.89198")
        .status()
        .unwrap();
    assert!(made.success());
    fs::read(archive).unwrap()
}

/// The shipped Toolbox's world, its release served at the URL and sha256 its folder pins, and
/// `setsid` starting what it is given, as the real one does in the background.
fn toolbox_world() -> super::World {
    let world = world(TOOLBOX);
    let catalog = Catalog::load(&world.catalog).unwrap();
    let Kind::Download { amd64, .. } = &catalog.get(TOOLBOX).unwrap().actions[0].kind else {
        panic!("Toolbox's first step is its download");
    };
    let pin = amd64.as_ref().unwrap();
    world
        .host
        .state
        .borrow_mut()
        .downloads
        .insert(pin.url.clone(), (pin.sha256.clone(), toolbox_tarball()));
    world.host.sandbox().stub("setsid", "[ \"$1\" = -f ] && shift\n\"$@\"");
    world
}

#[test]
fn toolbox_starts_once_from_where_it_updates_itself_and_its_removal_keeps_the_ides() {
    let world = toolbox_world();
    let home = world.host_home();
    // The folders of the entries, which a session has before any app is installed.
    for dir in [".local/share/applications", ".config/autostart"] {
        fs::create_dir_all(home.join(dir)).unwrap();
    }
    let before = world.host.snapshot();
    let mut picks = Picks::default();
    picks.add(TOOLBOX);

    run(&world, |e| e.add(TOOLBOX, None, &picks)).unwrap();

    let program = home.join(".local/share/JetBrains/Toolbox/bin/jetbrains-toolbox");
    assert!(program.is_file(), "the release unpacks into Toolbox's own folder");
    let entry = home.join(".local/share/applications/jetbrains-toolbox.desktop");
    let autostart = home.join(".config/autostart/jetbrains-toolbox.desktop");
    assert!(
        entry.is_file() && autostart.is_file(),
        "Toolbox made its entries as it started"
    );
    // An IDE Toolbox installed, and its settings.
    let ide = home.join(".local/share/JetBrains/Toolbox/apps/intellij-idea/bin/idea");
    fs::create_dir_all(ide.parent().unwrap()).unwrap();
    fs::write(&ide, "#!/bin/sh\n").unwrap();

    run(&world, |e| e.remove_with(TOOLBOX, &Picks::default())).unwrap();

    assert!(!program.exists() && !entry.exists() && !autostart.exists());
    assert!(ide.is_file(), "the IDEs stay");
    assert_eq!(world.journals(TOOLBOX), (None, None));
    fs::remove_dir_all(home.join(".local/share/JetBrains")).unwrap();
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn toolbox_running_is_left_alone_until_it_is_quit() {
    let world = toolbox_world();
    let before = world.host.snapshot();
    // pgrep as the machine answers it while Toolbox runs from its folder: the scripts' pattern is
    // matched against Toolbox's command line, as `pgrep -f` matches, and a process name, which the
    // kernel cuts at 15 characters, matches nothing.
    let running = world
        .host_home()
        .join(".local/share/JetBrains/Toolbox/bin/jetbrains-toolbox");
    world.host.sandbox().stub(
        "pgrep",
        &format!(
            "full=; for a; do [ \"$a\" = -f ] && full=1; pattern=$a; done\n\
             [ -n \"$full\" ] && printf '%s --minimize\\n' '{}' | grep -Eq -- \"$pattern\"",
            running.display()
        ),
    );

    let refused = run(&world, |e| e.add(TOOLBOX, None, &Picks::default())).unwrap_err();

    assert!(
        matches!(&refused, EngineError::NotReady(says) if says.contains("quit it from its tray icon")),
        "{refused}"
    );
    assert_eq!(world.host.snapshot(), before);
}
