//! The AI apps of the shipped catalog, installed and removed on the fake machine.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use super::super::Engine;
use super::super::EngineError;
use super::super::Keeper;
use super::super::Ran;
use super::Events;
use super::World;
use crate::apps::Catalog;
use crate::apps::Platform;
use crate::apps::action::Kind;
use crate::apps::action::Side;
use crate::apps::catalog::CATALOG;
use crate::apps::picks::Picks;
use crate::unfree::Unfree;

const APPS: [&str; 10] = [
    "aider",
    "chatgpt",
    "claude-desktop",
    "codex",
    "cursor",
    "devin-desktop",
    "gemini-cli",
    "github-copilot-cli",
    "lm-studio",
    "opencode",
];

/// Chrome as the web apps require it, without the theme helper the shipped one brings.
const CHROME: &str = r#"name = "Google Chrome"
category = "browsers"
tags = ["browser"]
summary = "A browser for the tests"
description = "A browser the tests install."
description_from = "https://example.com/chrome"

[[action]]
name = "Install Google Chrome"
kind = "apt"
arch = "amd64"
packages = ["google-chrome-stable"]
"#;

pub(super) fn shipped(id: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(CATALOG)
        .join(id)
}

/// A machine whose catalog is the shipped app `id`, every file of its folder copied with its mode
/// as git keeps it, and the Chrome stand-in.
pub(super) fn world(id: &str) -> World {
    let text = fs::read_to_string(shipped(id).join("app.toml")).unwrap();
    let world = World::new(&[(id, &text), ("google-chrome", CHROME)]);
    for entry in fs::read_dir(shipped(id)).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap();
        if name == "app.toml" {
            continue;
        }
        let to = world.catalog.join(id).join(name);
        fs::copy(&path, &to).unwrap();
        let executable = fs::metadata(&path).unwrap().permissions().mode() & 0o100 != 0;
        let mode = if executable { 0o755 } else { 0o644 };
        fs::set_permissions(&to, fs::Permissions::from_mode(mode)).unwrap();
    }
    world
}

pub(super) fn run(
    world: &World,
    go: impl FnOnce(&mut Engine<'_>) -> Result<Vec<Ran>, EngineError>,
) -> Result<Vec<Ran>, EngineError> {
    let mut events = Events::default();
    let mut root = world.root();
    let mut user_host = world.host.clone();
    let keeper = Keeper::new(&mut user_host, Side::User, &world.catalog, &world.user_journals);
    let mut engine = Engine::new(keeper, &mut root, &world.root_journals, &world.lock, &mut events, true);
    go(&mut engine)
}

#[test]
fn every_ai_app_installs_on_the_fake_machine_and_its_removal_leaves_it_as_before() {
    for id in APPS {
        let world = world(id);
        let before = world.host.snapshot();
        let mut picks = Picks::default();
        picks.add(id);

        let ran = run(&world, |e| e.add(id, None, &picks)).unwrap_or_else(|e| panic!("{id}: {e}"));

        let catalog = Catalog::load(&world.catalog).unwrap();
        let app = catalog.get(id).unwrap();
        let names: Vec<&str> = ran.iter().map(|r| r.name.as_str()).collect();
        if app.requires(None).next().is_some() {
            assert_eq!(names, ["Google Chrome", app.name.as_str()], "{id}");
        } else {
            assert_eq!(names, [app.name.as_str()], "{id}");
        }
        {
            let state = world.host.state.borrow();
            for action in &app.actions {
                match &action.kind {
                    Kind::Apt { packages, repo, .. } => {
                        assert!(packages.iter().all(|p| state.dpkg.contains_key(p)), "{id}");
                        let repo = repo.as_ref().unwrap();
                        assert!(state.repos.contains(repo), "{id}: {repo} wasn't set up");
                    }
                    Kind::Nix { packages, .. } => assert!(packages.iter().all(|p| state.nix.contains(p)), "{id}"),
                    Kind::WebApp { id: web, icon, .. } => {
                        let entry = world
                            .host_home()
                            .join(format!(".local/share/applications/kubaos-web-{web}.desktop"));
                        let text = fs::read_to_string(&entry).unwrap();
                        assert!(text.contains(&format!("\nIcon={icon}\n")), "{id}: {text}");
                        assert!(state.dpkg.contains_key("google-chrome-stable"), "{id}");
                    }
                    other => panic!("{id}: an AI app is its package or its window, not {other:?}"),
                }
            }
        }

        let gone = run(&world, |e| e.remove_with(id, &Picks::default())).unwrap_or_else(|e| panic!("{id}: {e}"));

        assert_eq!(gone.len(), ran.len(), "{id}: what it brought goes with it");
        assert_eq!(world.host.snapshot(), before, "{id}");
        assert_eq!(world.journals(id), (None, None), "{id}");
    }
}

#[test]
fn which_ai_apps_a_machine_can_have_follows_its_architecture_and_the_unfree_setting() {
    let catalog = Catalog::load(&shipped("codex").join("..")).unwrap();
    let on = |arch: &str, unfree: Unfree| -> Vec<&str> {
        APPS.into_iter()
            .filter(|id| catalog.available(catalog.get(id).unwrap(), None, Platform::new(arch, unfree)))
            .collect()
    };

    assert_eq!(on("amd64", Unfree::Allowed), APPS);
    // Devin Desktop's repository has amd64 alone, and ChatGPT's window needs Chrome, which Google
    // builds for amd64 alone.
    assert_eq!(
        on("arm64", Unfree::Allowed),
        [
            "aider",
            "claude-desktop",
            "codex",
            "cursor",
            "gemini-cli",
            "github-copilot-cli",
            "lm-studio",
            "opencode"
        ]
    );
    // nixpkgs marks Copilot CLI and LM Studio unfree.
    let free = on("amd64", Unfree::Refused);
    assert!(
        !free.contains(&"github-copilot-cli") && !free.contains(&"lm-studio"),
        "{free:?}"
    );
    assert_eq!(free.len(), APPS.len() - 2, "{free:?}");
}

#[test]
fn cursors_sources_file_is_the_one_its_package_writes_word_for_word() {
    // What Cursor 3.30's postinst writes on every install and upgrade, on an apt with
    // `apt modernize-sources` (Ubuntu 26.04's); a different file would be overwritten by the
    // vendor's at its next upgrade.
    let postinst = "### THIS FILE IS AUTOMATICALLY CONFIGURED ###\n\
                    # You may comment out this entry, but any other modifications may be lost.\n\
                    Types: deb\n\
                    URIs: https://downloads.cursor.com/aptrepo\n\
                    Suites: stable\n\
                    Components: main\n\
                    Architectures: amd64,arm64\n\
                    Signed-By: /usr/share/keyrings/anysphere.gpg\n";
    let ours = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../system/files/apt/sources.list.d/cursor.sources");

    assert_eq!(fs::read_to_string(ours).unwrap(), postinst);
}

/// Ollama's shipped folder on a fake machine whose release serves stand-ins for its archives at
/// their pins: the program and a CPU library, and AMD's library for its graphics cards under
/// `lib/ollama/rocm_v7_2`.
fn ollama() -> World {
    let world = world("ollama");
    let catalog = Catalog::load(&world.catalog).unwrap();
    let app = catalog.get("ollama").unwrap();
    let archives = [
        (
            "ollama-linux-amd64.tar.zst",
            ["bin/ollama", "lib/ollama/libggml-base.so"],
        ),
        (
            "ollama-linux-amd64-rocm.tar.zst",
            [
                "lib/ollama/rocm_v7_2/libggml-hip.so",
                "lib/ollama/rocm_v7_2/libamdhip64.so.7",
            ],
        ),
    ];
    for (name, files) in archives {
        let download = app
            .all_actions()
            .filter_map(|a| a.download("amd64"))
            .find(|d| d.url.ends_with(&format!("/{name}")))
            .unwrap();
        let bytes = zstd_tarball(&world, &files);
        world
            .host
            .state
            .borrow_mut()
            .downloads
            .insert(download.url.clone(), (download.sha256.clone(), bytes));
    }
    world
}

/// A zstd tarball of `files`, each holding its own path, made by the tar of `world`'s sandbox.
fn zstd_tarball(world: &World, files: &[&str]) -> Vec<u8> {
    let src = tempfile::tempdir().unwrap();
    for file in files {
        let path = src.path().join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, file).unwrap();
    }
    let archive = src.path().join("archive.tar.zst");
    let made = world
        .host
        .sandbox()
        .command("tar")
        .args(["-c", "--zstd", "-f"])
        .arg(&archive)
        .arg("-C")
        .arg(src.path())
        .args(
            ["bin", "lib"]
                .iter()
                .filter(|top| files.iter().any(|f| f.starts_with(*top))),
        )
        .status()
        .unwrap();
    assert!(made.success());
    fs::read(archive).unwrap()
}

/// Ollama picked in `setup`.
fn picked(setup: &str) -> Picks {
    let mut picks = Picks::default();
    picks.add("ollama");
    picks.add_setup("ollama", setup);
    picks
}

const ROCM: &str = "/opt/ollama/0.40.16/lib/ollama/rocm_v7_2/libggml-hip.so";

#[test]
fn ollama_installs_in_each_setup_as_a_service_on_this_machine_and_goes_whole() {
    for setup in ["standard", "amd"] {
        let world = ollama();
        let before = world.host.snapshot();

        run(&world, |e| e.add("ollama", Some(setup), &picked(setup))).unwrap_or_else(|e| panic!("{setup}: {e}"));

        let program = world.system_path("/opt/ollama/0.40.16/bin/ollama");
        assert_eq!(fs::read_to_string(&program).unwrap(), "bin/ollama", "{setup}");
        assert_eq!(
            fs::read_link(world.system_path("/usr/local/bin/ollama")).unwrap(),
            Path::new("/opt/ollama/0.40.16/bin/ollama"),
            "{setup}"
        );
        let unit = fs::read_to_string(world.system_path("/etc/systemd/system/ollama.service")).unwrap();
        for line in [
            "ExecStart=/opt/ollama/0.40.16/bin/ollama serve",
            "DynamicUser=yes",
            "ExecStartPre=-+/usr/bin/nvidia-modprobe -u -c=0",
            "StateDirectory=ollama",
            "SupplementaryGroups=render video",
            "Environment=OLLAMA_HOST=127.0.0.1:11434",
        ] {
            assert!(unit.lines().any(|l| l == line), "{setup}: no {line} in\n{unit}");
        }
        {
            let state = world.host.state.borrow();
            let service = &state.units["system:ollama.service"];
            assert!(service.enabled && service.active, "{setup}: {service:?}");
            assert!(state.dpkg.contains_key("zstd"), "{setup}");
        }
        assert_eq!(world.system_path(ROCM).is_file(), setup == "amd", "{setup}");

        run(&world, |e| e.remove_with("ollama", &Picks::default())).unwrap_or_else(|e| panic!("{setup}: {e}"));

        assert_eq!(world.host.snapshot(), before, "{setup}");
        assert_eq!(world.journals("ollama"), (None, None), "{setup}");
    }
}

#[test]
fn switching_ollama_to_amd_and_back_downloads_and_removes_rocm_alone() {
    let world = ollama();
    run(&world, |e| e.add("ollama", Some("standard"), &picked("standard"))).unwrap();
    let fetched = || {
        let state = world.host.state.borrow();
        state.log.iter().filter(|l| l.starts_with("fetch ")).count()
    };
    let main = fetched();

    run(&world, |e| e.add("ollama", Some("amd"), &picked("amd"))).unwrap();
    assert!(world.system_path(ROCM).is_file());
    assert_eq!(fetched(), main + 1, "the main archive stays as it is");
    run(&world, |e| e.add("ollama", Some("standard"), &picked("standard"))).unwrap();

    assert!(!world.system_path(ROCM).exists());
    assert!(
        world
            .system_path("/opt/ollama/0.40.16/lib/ollama/libggml-base.so")
            .is_file()
    );
    assert!(world.host.state.borrow().units["system:ollama.service"].active);
}

#[test]
fn ollama_for_amd_graphics_cards_is_amd64_only() {
    let catalog = Catalog::load(&shipped("ollama").join("..")).unwrap();
    let ollama = catalog.get("ollama").unwrap();
    let on =
        |setup: &str, arch: &str| catalog.available(ollama, ollama.setup(setup), Platform::new(arch, Unfree::Refused));

    assert!(on("standard", "amd64") && on("standard", "arm64") && on("amd", "amd64"));
    assert!(!on("amd", "arm64"));
}
