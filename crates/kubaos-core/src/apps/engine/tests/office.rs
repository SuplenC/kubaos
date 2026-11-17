//! The notes and office apps of the shipped catalog, installed and removed on the fake machine.

use std::fs;
use std::path::Path;

use super::World;
use super::ai::run;
use super::ai::shipped;
use super::ai::world;
use crate::apps::Catalog;
use crate::apps::Platform;
use crate::apps::action::Kind;
use crate::apps::picks::Picks;
use crate::unfree::Unfree;

const APPS: [&str; 14] = [
    "anki",
    "apostrophe",
    "calibre",
    "joplin",
    "libreoffice",
    "linear",
    "logseq",
    "notion",
    "obsidian",
    "onlyoffice",
    "qalculate",
    "todoist",
    "xournalpp",
    "zotero",
];

const LOGSEQ: &str = "logseq";

/// The shipped app `id`'s world, each of its downloads served at the URL and sha256 its folder
/// pins for amd64, with `body` as what it downloads.
fn served(id: &str, body: &[u8]) -> World {
    let world = world(id);
    let catalog = Catalog::load(&world.catalog).unwrap();
    for action in catalog.get(id).unwrap().all_actions() {
        if let Kind::Download { amd64: Some(pin), .. } = &action.kind {
            world
                .host
                .state
                .borrow_mut()
                .downloads
                .insert(pin.url.clone(), (pin.sha256.clone(), body.to_vec()));
        }
    }
    world
}

#[test]
fn every_office_app_installs_on_the_fake_machine_and_its_removal_leaves_it_as_before() {
    for id in APPS {
        let world = served(id, b"#!/bin/sh\n");
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
                        if let Some(repo) = repo {
                            assert!(state.repos.contains(repo), "{id}: {repo} wasn't set up");
                        }
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
                    Kind::Download { .. } | Kind::File { .. } => {}
                    other => panic!("{id}: an office app is its package, download or window, not {other:?}"),
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
fn which_office_apps_a_machine_can_have_follows_its_architecture_and_the_unfree_setting() {
    let catalog = Catalog::load(&shipped("anki").join("..")).unwrap();
    let on = |arch: &str, unfree: Unfree| -> Vec<&str> {
        APPS.into_iter()
            .filter(|id| catalog.available(catalog.get(id).unwrap(), None, Platform::new(arch, unfree)))
            .collect()
    };

    assert_eq!(on("amd64", Unfree::Allowed), APPS);
    // The web apps' windows need Chrome, which Google builds for amd64 alone.
    assert_eq!(
        on("arm64", Unfree::Allowed),
        [
            "anki",
            "apostrophe",
            "calibre",
            "joplin",
            "libreoffice",
            "logseq",
            "obsidian",
            "onlyoffice",
            "qalculate",
            "xournalpp",
            "zotero",
        ]
    );
    // nixpkgs marks Obsidian unfree.
    let free = on("amd64", Unfree::Refused);
    assert_eq!(
        free,
        APPS.into_iter().filter(|id| *id != "obsidian").collect::<Vec<_>>()
    );
}

#[test]
fn onlyoffices_pin_is_there_before_apt_reads_its_repository() {
    // Ascensio's repository has a ttf-mscorefonts-installer newer than Ubuntu's, which apt would
    // take from it for anything that asks for the fonts, without the pin.
    let catalog = Catalog::load(&shipped("onlyoffice").join("..")).unwrap();
    let onlyoffice = catalog.get("onlyoffice").unwrap();

    let kinds: Vec<&Kind> = onlyoffice.all_actions().map(|a| &a.kind).collect();

    assert!(
        matches!(kinds[..], [Kind::File { to, .. }, Kind::Apt { repo: Some(repo), .. }]
            if to == "/etc/apt/preferences.d/kubaos-onlyoffice" && repo == "onlyoffice"),
        "{kinds:?}"
    );
    let pin = fs::read_to_string(onlyoffice.dir.join("kubaos-onlyoffice.pref")).unwrap();
    let stanza: Vec<&str> = pin.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(
        stanza,
        ["Package: *", "Pin: origin download.onlyoffice.com", "Pin-Priority: 100"]
    );
    let sources =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../system/files/apt/sources.list.d/onlyoffice.sources");
    assert!(
        fs::read_to_string(sources)
            .unwrap()
            .contains("URIs: https://download.onlyoffice.com/repo/debian\n"),
        "the pin's origin is the repository's host"
    );
}

#[test]
fn logseqs_menu_entry_runs_the_appimage_it_downloaded() {
    let world = served(LOGSEQ, b"#!/bin/sh\n");
    let mut picks = Picks::default();
    picks.add(LOGSEQ);

    run(&world, |e| e.add(LOGSEQ, None, &picks)).unwrap();

    let home = world.host_home();
    let appimage = home.join(".local/opt/logseq/Logseq.AppImage");
    assert!(appimage.is_file());
    let entry = fs::read_to_string(home.join(".local/share/applications/logseq.desktop")).unwrap();
    assert!(
        entry.contains(&format!("\nExec=\"{}\" %U\n", appimage.display())),
        "{entry}"
    );
    assert!(entry.contains("\nIcon=logseq\n"), "{entry}");
}

#[test]
fn a_release_bumping_logseqs_pin_never_downloads_over_the_logseq_that_updated_itself() {
    let world = served(LOGSEQ, b"#!/bin/sh\n# Logseq 2.0.2\n");
    let mut picks = Picks::default();
    picks.add(LOGSEQ);
    run(&world, |e| e.add(LOGSEQ, None, &picks)).unwrap();
    let appimage = world.host_home().join(".local/opt/logseq/Logseq.AppImage");
    // electron-updater writes the new release over the AppImage it runs from.
    fs::write(&appimage, "#!/bin/sh\n# Logseq 2.1.0, from its own updater\n").unwrap();
    let text = fs::read_to_string(world.catalog.join(LOGSEQ).join("app.toml")).unwrap();
    let catalog = Catalog::load(&world.catalog).unwrap();
    let pin = catalog
        .get(LOGSEQ)
        .unwrap()
        .all_actions()
        .find_map(|a| match &a.kind {
            Kind::Download { amd64, .. } => amd64.clone(),
            _ => None,
        })
        .unwrap();
    let bumped_url = "https://github.com/logseq/logseq/releases/download/2.0.3/Logseq-linux-x86_64-2.0.3.AppImage";
    let bumped_sha = "c".repeat(64);
    world.write_app(
        LOGSEQ,
        &text
            .replace("version = \"2.0.2\"", "version = \"2.0.3\"")
            .replace(&pin.url, bumped_url)
            .replace(&pin.sha256, &bumped_sha),
    );
    world.host.state.borrow_mut().downloads.insert(
        bumped_url.to_owned(),
        (bumped_sha, b"#!/bin/sh\n# Logseq 2.0.3\n".to_vec()),
    );

    run(&world, |e| e.add(LOGSEQ, None, &picks)).unwrap();

    assert_eq!(
        fs::read_to_string(&appimage).unwrap(),
        "#!/bin/sh\n# Logseq 2.1.0, from its own updater\n"
    );
    let fetched = world.host.state.borrow().log.iter().any(|l| l.ends_with(bumped_url));
    assert!(!fetched, "the bumped pin is never downloaded");
}
