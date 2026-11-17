//! The games of the shipped catalog, installed and removed on the fake machine.

use std::fs;

use super::super::EngineError;
use super::World;
use super::ai::run;
use super::ai::shipped;
use super::ai::world;
use crate::apps::Catalog;
use crate::apps::Folder;
use crate::apps::Platform;
use crate::apps::action::Kind;
use crate::apps::picks::Picks;
use crate::unfree::Unfree;

const APPS: [&str; 12] = [
    "gamescope",
    "geforce-now",
    "heroic",
    "lutris",
    "mangohud",
    "minecraft-launcher",
    "moonlight",
    "prismlauncher",
    "protonup-qt",
    "retroarch",
    "sunshine",
    "xbox-cloud-gaming",
];

const MINECRAFT: &str = "minecraft-launcher";

fn picks(id: &str) -> Picks {
    let mut picks = Picks::default();
    picks.add(id);
    picks
}

/// A gzip tarball of Mojang's: its one folder, `minecraft-launcher`, holding the program, made by
/// the tar of `world`'s sandbox.
fn minecraft_tarball(world: &World) -> Vec<u8> {
    let src = tempfile::tempdir().unwrap();
    let folder = src.path().join("minecraft-launcher");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("minecraft-launcher"), "#!/bin/sh\n").unwrap();
    let archive = src.path().join("Minecraft.tar.gz");
    let made = world
        .host
        .sandbox()
        .command("tar")
        .arg("-czf")
        .arg(&archive)
        .arg("-C")
        .arg(src.path())
        .arg("minecraft-launcher")
        .status()
        .unwrap();
    assert!(made.success());
    fs::read(archive).unwrap()
}

/// The shipped game `id`'s world, its `.deb`s and downloads served at the pins its folder names
/// for amd64.
fn served(id: &str) -> World {
    let world = world(id);
    let catalog = Catalog::load(&world.catalog).unwrap();
    for action in catalog.get(id).unwrap().all_actions() {
        let body = match &action.kind {
            Kind::Deb { package, version, .. } => format!("deb {package} {version}").into_bytes(),
            Kind::Download { .. } => minecraft_tarball(&world),
            _ => continue,
        };
        let pin = action.download("amd64").unwrap();
        world
            .host
            .state
            .borrow_mut()
            .downloads
            .insert(pin.url.clone(), (pin.sha256.clone(), body));
    }
    world
}

/// Each of the installed `app`'s steps there on the fake machine: its packages, files and window.
fn in_place(world: &World, app: &Folder) {
    let id = app.id.as_str();
    let state = world.host.state.borrow();
    let home_or_system = |to: &str| match to.strip_prefix("~/") {
        Some(rel) => world.host_home().join(rel),
        None => world.system_path(to),
    };
    for action in &app.actions {
        match &action.kind {
            Kind::Apt { packages, .. } => assert!(packages.iter().all(|p| state.dpkg.contains_key(p)), "{id}"),
            Kind::Deb { package, version, .. } => assert_eq!(&state.dpkg[package], version, "{id}"),
            Kind::Nix { packages, .. } => assert!(packages.iter().all(|p| state.nix.contains(p)), "{id}"),
            Kind::Download { version, .. } => {
                let folder = home_or_system(&format!("~/.local/opt/minecraft-launcher/{version}"));
                assert!(
                    folder.join("minecraft-launcher").is_file(),
                    "{id}: {}",
                    folder.display()
                );
            }
            Kind::File { to, .. } => assert!(home_or_system(to).is_file(), "{id}: {to}"),
            Kind::Setting { file, key, .. } => {
                let text = fs::read_to_string(home_or_system(file)).unwrap();
                assert!(
                    text.lines().any(|l| l.starts_with(&format!("{key} = "))),
                    "{id}: {text}"
                );
            }
            Kind::WebApp { id: web, icon, .. } => {
                let entry = world
                    .host_home()
                    .join(format!(".local/share/applications/kubaos-web-{web}.desktop"));
                let text = fs::read_to_string(&entry).unwrap();
                assert!(text.contains(&format!("\nIcon={icon}\n")), "{id}: {text}");
                assert!(state.dpkg.contains_key("google-chrome-stable"), "{id}");
            }
            other => panic!("{id}: a game is its packages, download, files or window, not {other:?}"),
        }
    }
}

#[test]
fn every_game_installs_on_the_fake_machine_and_its_removal_leaves_it_as_before() {
    for id in APPS {
        let world = served(id);
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
        in_place(&world, app);

        let gone = run(&world, |e| e.remove_with(id, &Picks::default())).unwrap_or_else(|e| panic!("{id}: {e}"));

        assert_eq!(gone.len(), ran.len(), "{id}: what it brought goes with it");
        assert_eq!(world.host.snapshot(), before, "{id}");
        assert_eq!(world.journals(id), (None, None), "{id}");
    }
}

#[test]
fn which_games_a_machine_can_have_follows_its_architecture() {
    let catalog = Catalog::load(&shipped("heroic").join("..")).unwrap();
    let on = |arch: &str, unfree: Unfree| -> Vec<&str> {
        APPS.into_iter()
            .filter(|id| catalog.available(catalog.get(id).unwrap(), None, Platform::new(arch, unfree)))
            .collect()
    };

    assert_eq!(on("amd64", Unfree::Allowed), APPS);
    // None of them is unfree in nixpkgs.
    assert_eq!(on("amd64", Unfree::Refused), APPS);
    // Mojang builds its launcher for amd64 alone, nixpkgs ProtonUp-Qt, and the two cloud services'
    // windows need Chrome, which Google builds for amd64 alone.
    assert_eq!(
        on("arm64", Unfree::Allowed),
        [
            "gamescope",
            "heroic",
            "lutris",
            "mangohud",
            "moonlight",
            "prismlauncher",
            "retroarch",
            "sunshine",
        ]
    );
}

#[test]
fn the_minecraft_launcher_runs_from_its_versions_folder_and_its_removal_keeps_the_worlds() {
    let world = served(MINECRAFT);
    let home = world.host_home();
    let before = world.host.snapshot();

    run(&world, |e| e.add(MINECRAFT, None, &picks(MINECRAFT))).unwrap();

    let program = home.join(".local/opt/minecraft-launcher/2.1.3/minecraft-launcher");
    assert!(program.is_file(), "the tarball's folder is the version's");
    let entry = fs::read_to_string(home.join(".local/share/applications/minecraft-launcher.desktop")).unwrap();
    assert!(
        entry.contains(&format!("\nExec=\"{}\"\n", program.display())),
        "{entry}"
    );
    assert!(entry.contains("\nIcon=minecraft-launcher\n"), "{entry}");
    // What the launcher makes as you play: itself, as Mojang's bootstrap downloads it, and a world.
    let world_save = home.join(".minecraft/saves/New World/level.dat");
    fs::create_dir_all(world_save.parent().unwrap()).unwrap();
    fs::write(&world_save, "").unwrap();

    run(&world, |e| e.remove_with(MINECRAFT, &Picks::default())).unwrap();

    assert!(!program.exists());
    assert!(world_save.is_file(), "the worlds stay");
    assert_eq!(world.journals(MINECRAFT), (None, None));
    fs::remove_dir_all(home.join(".minecraft")).unwrap();
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn sunshine_starts_nothing_at_login_and_its_icon_takes_the_packages_name() {
    let world = served("sunshine");
    let (units, groups) = {
        let state = world.host.state.borrow();
        (state.units.clone(), state.groups.clone())
    };

    run(&world, |e| e.add("sunshine", None, &picks("sunshine"))).unwrap();

    let state = world.host.state.borrow();
    assert_eq!(state.units, units, "its menu entry starts it, nothing else does");
    assert_eq!(state.groups, groups, "its udev rules hand the seat's user the devices");
    assert!(
        world
            .host_home()
            .join(".local/share/icons/hicolor/scalable/apps/dev.lizardbyte.app.Sunshine.svg")
            .is_file()
    );
}

#[test]
fn sunshines_web_page_answers_this_machine_alone_and_its_removal_leaves_your_settings() {
    let world = served("sunshine");
    let conf = world.host_home().join(".config/sunshine/sunshine.conf");
    fs::create_dir_all(conf.parent().unwrap()).unwrap();
    // A sunshine.conf as Sunshine writes it, from an install of the user's own before.
    let mine = "sunshine_name = den\nencoder = vaapi\n";
    fs::write(&conf, mine).unwrap();

    run(&world, |e| e.add("sunshine", None, &picks("sunshine"))).unwrap();

    assert_eq!(
        fs::read_to_string(&conf).unwrap(),
        format!("{mine}origin_web_ui_allowed = pc\n"),
        "the page answers this machine alone, and the user's lines stay"
    );
    // Sunshine rewrites its file as the user changes a setting in its page.
    fs::write(
        &conf,
        "sunshine_name = den\nencoder = nvenc\norigin_web_ui_allowed = pc\n",
    )
    .unwrap();

    run(&world, |e| e.remove_with("sunshine", &Picks::default())).unwrap();

    assert_eq!(
        fs::read_to_string(&conf).unwrap(),
        "sunshine_name = den\nencoder = nvenc\n"
    );
}

#[test]
fn sunshine_running_is_left_alone_until_it_is_quit() {
    let world = served("sunshine");
    run(&world, |e| e.add("sunshine", None, &picks("sunshine"))).unwrap();
    let installed = world.host.snapshot();
    // pgrep as the machine answers it while Sunshine's service runs it: the script's pattern is
    // matched against its command line, as `pgrep -f` matches.
    world.host.sandbox().stub(
        "pgrep",
        "full=; for a; do [ \"$a\" = -f ] && full=1; pattern=$a; done\n\
         [ -n \"$full\" ] && printf '/usr/bin/sunshine\\n' | grep -Eq -- \"$pattern\"",
    );

    let refused = run(&world, |e| e.remove_with("sunshine", &Picks::default())).unwrap_err();

    assert!(
        matches!(&refused, EngineError::NotReady(says) if says.contains("Sunshine is running")),
        "{refused}"
    );
    assert_eq!(world.host.snapshot(), installed);
}
