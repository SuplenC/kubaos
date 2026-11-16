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
