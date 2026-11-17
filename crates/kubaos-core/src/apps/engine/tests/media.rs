//! The media apps of the shipped catalog, installed and removed on the fake machine.

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
    "ardour",
    "audacity",
    "handbrake",
    "jellyfin-desktop",
    "kdenlive",
    "losslesscut",
    "mpv",
    "obs-studio",
    "shotcut",
    "spotify",
    "vlc",
    "youtube-music",
];

const SPOTIFY_LIST: &str = "/etc/apt/sources.list.d/spotify.list";

fn picks(id: &str) -> Picks {
    let mut picks = Picks::default();
    picks.add(id);
    picks
}

/// The shipped app `id`'s world. Spotify's steps name `/etc/apt/sources.list.d/spotify.list` as an
/// argument, which its scripts write as they are, so its folder names the fake machine's instead,
/// in a sources folder that is there, as on any machine.
fn media_world(id: &str) -> World {
    let world = world(id);
    if id == "spotify" {
        let list = world.system_path(SPOTIFY_LIST);
        fs::create_dir_all(list.parent().unwrap()).unwrap();
        let text = fs::read_to_string(world.catalog.join(id).join("app.toml")).unwrap();
        world.write_app(id, &text.replace(SPOTIFY_LIST, &list.display().to_string()));
    }
    world
}

/// Each of the installed `app`'s steps there on the fake machine: its packages, files and window.
fn in_place(world: &World, app: &Folder) {
    let id = app.id.as_str();
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
            Kind::File { to, .. } => {
                let path = match to.strip_prefix("~/") {
                    Some(rel) => world.host_home().join(rel),
                    None => world.system_path(to),
                };
                assert!(path.is_file(), "{id}: {}", path.display());
            }
            Kind::Run { .. } if id == "spotify" => {}
            other => panic!("{id}: a media app is its package, file or window, not {other:?}"),
        }
    }
}

#[test]
fn every_media_app_installs_on_the_fake_machine_and_its_removal_leaves_it_as_before() {
    for id in APPS {
        let world = media_world(id);
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
fn which_media_apps_a_machine_can_have_follows_its_architecture() {
    let catalog = Catalog::load(&shipped("vlc").join("..")).unwrap();
    let on = |arch: &str, unfree: Unfree| -> Vec<&str> {
        APPS.into_iter()
            .filter(|id| catalog.available(catalog.get(id).unwrap(), None, Platform::new(arch, unfree)))
            .collect()
    };

    assert_eq!(on("amd64", Unfree::Allowed), APPS);
    // None of them is unfree in nixpkgs: Spotify comes from its own repository.
    assert_eq!(on("amd64", Unfree::Refused), APPS);
    // Spotify, the PPA's OBS and nixpkgs' LosslessCut are amd64's alone, and YouTube Music's
    // window needs Chrome, which Google builds for amd64 alone.
    assert_eq!(
        on("arm64", Unfree::Allowed),
        [
            "ardour",
            "audacity",
            "handbrake",
            "jellyfin-desktop",
            "kdenlive",
            "mpv",
            "shotcut",
            "vlc",
        ]
    );
}

#[test]
fn spotify_opts_out_of_its_packages_source_and_key_before_the_package_lands() {
    let world = media_world("spotify");
    let list = world.system_path(SPOTIFY_LIST);

    run(&world, |e| e.add("spotify", None, &picks("spotify"))).unwrap();

    let text = fs::read_to_string(&list).unwrap();
    assert!(text.lines().all(|l| l.starts_with('#')), "no repository in it: {text}");
    let cfg = fs::read_to_string(world.system_path("/etc/dpkg/dpkg.cfg.d/kubaos-spotify")).unwrap();
    let rules: Vec<&str> = cfg.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(rules, ["path-exclude=/usr/share/spotify/apt-keys/*"]);
    let state = world.host.state.borrow();
    let at = |what: &str| state.log.iter().position(|l| l.starts_with(what)).unwrap();
    assert!(
        at("run ./opt-out") < at("apt install spotify-client"),
        "{:#?}",
        state.log
    );
    assert!(state.repos.contains("spotify"));
}

#[test]
fn the_spotify_list_of_a_spotify_installed_by_hand_stays_when_kubaos_removes_spotify() {
    let world = media_world("spotify");
    let list = world.system_path(SPOTIFY_LIST);
    // What Spotify's postinst writes, which repo setup turns off on a real machine.
    fs::write(&list, "deb https://repository.spotify.com stable non-free\n").unwrap();

    run(&world, |e| e.add("spotify", None, &picks("spotify"))).unwrap();
    run(&world, |e| e.remove_with("spotify", &Picks::default())).unwrap();

    assert_eq!(
        fs::read_to_string(&list).unwrap(),
        "deb https://repository.spotify.com stable non-free\n"
    );
    let calls = world.host.state.borrow().log.clone();
    assert!(!calls.iter().any(|c| c.starts_with("run ./opt-out")), "{calls:#?}");
}

#[test]
fn ardours_menu_entry_takes_the_packages_place_and_starts_it_through_pipewires_jack() {
    let world = media_world("ardour");

    run(&world, |e| e.add("ardour", None, &picks("ardour"))).unwrap();

    let home = world.host_home();
    let entry = fs::read_to_string(home.join(".local/share/applications/ardour.desktop")).unwrap();
    assert!(
        entry.contains("\nExec=env FLTK_BACKEND=x11 pw-jack /usr/bin/ardour %f\n"),
        "{entry}"
    );
    assert!(entry.contains("\nIcon=ardour\n"), "{entry}");
    assert!(
        home.join(".local/share/icons/hicolor/scalable/apps/ardour.svg")
            .is_file()
    );
    let state = world.host.state.borrow();
    assert!(state.dpkg.contains_key("pipewire-jack"), "pw-jack is pipewire-jack's");
}

/// The menu entry KubaOS writes for Spotify, which takes the package's place.
const SPOTIFY_ENTRY: &str = ".local/share/applications/spotify.desktop";

#[test]
fn spotify_starts_on_wayland_from_the_menu_and_its_removal_takes_the_entry_back() {
    let world = media_world("spotify");
    let entry = world.host_home().join(SPOTIFY_ENTRY);

    run(&world, |e| e.add("spotify", None, &picks("spotify"))).unwrap();

    let text = fs::read_to_string(&entry).unwrap();
    assert!(text.contains("\nExec=env -u DISPLAY spotify %U\n"), "{text}");
    run(&world, |e| e.remove_with("spotify", &Picks::default())).unwrap();
    assert!(!entry.exists(), "the package's own entry is the menu's again");
}

#[test]
fn a_spotify_menu_entry_of_the_users_own_is_never_written_over_and_the_refusal_says_what_to_do() {
    let world = media_world("spotify");
    let entry = world.host_home().join(SPOTIFY_ENTRY);
    fs::create_dir_all(entry.parent().unwrap()).unwrap();
    fs::write(&entry, "[Desktop Entry]\nName=Mine\n").unwrap();
    let before = world.host.snapshot();

    let result = run(&world, |e| e.add("spotify", None, &picks("spotify")));

    let Err(EngineError::Refused { why, yours, .. }) = result else {
        panic!("Spotify was installed over the user's entry: {result:?}");
    };
    assert!(yours, "the refusal is over a file of the user's");
    assert!(
        why.iter()
            .any(|w| w.contains("is yours") && w.contains("delete or rename it")),
        "{why:?}"
    );
    assert_eq!(world.host.snapshot(), before);
    assert_eq!(fs::read_to_string(&entry).unwrap(), "[Desktop Entry]\nName=Mine\n");
}
