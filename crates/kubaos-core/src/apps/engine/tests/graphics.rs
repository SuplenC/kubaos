//! The graphics apps of the shipped catalog, installed and removed on the fake machine.

use std::fs;

use super::World;
use super::ai::run;
use super::ai::shipped;
use super::ai::world;
use crate::apps::Catalog;
use crate::apps::Platform;
use crate::apps::action::Kind;
use crate::apps::picks::Picks;
use crate::unfree::Unfree;

/// Every graphics app of this piece; Penpot, with its setups, has tests of its own.
const APPS: [&str; 9] = [
    "darktable",
    "excalidraw",
    "figma",
    "gimp",
    "inkscape",
    "krita",
    "pinta",
    "rawtherapee",
    "upscayl",
];

fn picks(id: &str) -> Picks {
    let mut picks = Picks::default();
    picks.add(id);
    picks
}

/// Each download of the app served at its pin, as the fake machine downloads it.
fn serve_the_downloads(world: &World, id: &str) {
    let catalog = Catalog::load(&world.catalog).unwrap();
    for action in &catalog.get(id).unwrap().actions {
        if let Some(pin) = action.download("amd64") {
            world.host.state.borrow_mut().downloads.insert(
                pin.url.clone(),
                (pin.sha256.clone(), format!("{id} release").into_bytes()),
            );
        }
    }
}

#[test]
fn every_graphics_app_installs_on_the_fake_machine_and_its_removal_leaves_it_as_before() {
    for id in APPS {
        let world = world(id);
        serve_the_downloads(&world, id);
        let before = world.host.snapshot();

        let ran = run(&world, |e| e.add(id, None, &picks(id))).unwrap_or_else(|e| panic!("{id}: {e}"));

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
                    Kind::Download { to, .. } | Kind::File { to, .. } => {
                        let path = world.host_home().join(to.trim_start_matches("~/"));
                        assert!(path.is_file(), "{id}: {}", path.display());
                    }
                    Kind::WebApp { id: web, icon, .. } => {
                        let entry = world
                            .host_home()
                            .join(format!(".local/share/applications/kubaos-web-{web}.desktop"));
                        let text = fs::read_to_string(&entry).unwrap();
                        assert!(text.contains(&format!("\nIcon={icon}\n")), "{id}: {text}");
                        assert!(state.dpkg.contains_key("google-chrome-stable"), "{id}");
                    }
                    other => panic!("{id}: a graphics app has no {other:?}"),
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
fn which_graphics_apps_a_machine_can_have_follows_its_architecture() {
    let catalog = Catalog::load(&shipped("gimp").join("..")).unwrap();
    let on = |arch: &str, unfree: Unfree| -> Vec<&str> {
        APPS.into_iter()
            .filter(|id| catalog.available(catalog.get(id).unwrap(), None, Platform::new(arch, unfree)))
            .collect()
    };

    assert_eq!(on("amd64", Unfree::Allowed), APPS);
    // darktable's repository builds amd64 alone, nixpkgs builds Upscayl from its amd64 AppImage,
    // and the web apps need Chrome, which Google builds for amd64 alone.
    assert_eq!(
        on("arm64", Unfree::Allowed),
        ["gimp", "inkscape", "krita", "pinta", "rawtherapee"]
    );
    assert_eq!(on("amd64", Unfree::Refused), APPS, "none of them is unfree");
}

#[test]
fn rawtherapees_entry_runs_its_appimage_once_fuse_is_there_and_its_removal_keeps_your_settings() {
    let id = "rawtherapee";
    let world = world(id);
    serve_the_downloads(&world, id);
    let home = world.host_home();
    let before = world.host.snapshot();

    run(&world, |e| e.add(id, None, &picks(id))).unwrap();

    let appimage = home.join(".local/opt/rawtherapee/RawTherapee.AppImage");
    assert_eq!(fs::read(&appimage).unwrap(), b"rawtherapee release");
    let entry = fs::read_to_string(home.join(".local/share/applications/rawtherapee.desktop")).unwrap();
    assert!(
        entry.contains(&format!("\nExec=\"{}\" %f\n", appimage.display())),
        "{entry}"
    );
    assert!(entry.contains("\nIcon=com.rawtherapee.RawTherapee\n"), "{entry}");
    assert!(
        !entry.contains("inode/directory") && !entry.contains("image/jpeg"),
        "it opens raw files, not every folder and picture: {entry}"
    );
    {
        let state = world.host.state.borrow();
        for package in ["fuse3", "libfuse2t64"] {
            assert!(state.dpkg.contains_key(package), "{package}");
        }
    }
    // What RawTherapee makes as you use it.
    let settings = home.join(".config/RawTherapee/options");
    fs::create_dir_all(settings.parent().unwrap()).unwrap();
    fs::write(&settings, "").unwrap();

    run(&world, |e| e.remove_with(id, &Picks::default())).unwrap();

    assert!(!appimage.exists());
    assert!(settings.is_file(), "your settings stay");
    fs::remove_dir_all(home.join(".config")).unwrap();
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn excalidraws_window_names_the_one_copy_of_its_icon_the_install_puts_in_the_theme() {
    let id = "excalidraw";
    let world = world(id);
    let home = world.host_home();

    run(&world, |e| e.add(id, None, &picks(id))).unwrap();

    let icons = home.join(".local/share/icons/hicolor/scalable/apps");
    let installed: Vec<String> = fs::read_dir(&icons)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(installed, ["excalidraw.svg"]);
    assert_eq!(
        fs::read(icons.join("excalidraw.svg")).unwrap(),
        fs::read(shipped(id).join("excalidraw.svg")).unwrap()
    );
    let entry = fs::read_to_string(home.join(".local/share/applications/kubaos-web-excalidraw.desktop")).unwrap();
    assert!(entry.contains("\nIcon=excalidraw\n"), "{entry}");
}
