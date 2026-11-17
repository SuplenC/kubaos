//! The games of the shipped catalog, installed and removed on the fake machine.

use std::fs;

use super::super::Engine;
use super::super::EngineError;
use super::super::Keeper;
use super::World;
use super::ai::run;
use super::ai::shipped;
use super::ai::world;
use crate::apps::Catalog;
use crate::apps::Folder;
use crate::apps::Platform;
use crate::apps::action::Kind;
use crate::apps::action::Side;
use crate::apps::picks::Picks;
use crate::unfree::Unfree;
use crate::update::event::UpdateEvent;

const APPS: [&str; 13] = [
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
    "steam",
    "sunshine",
    "xbox-cloud-gaming",
];

const MINECRAFT: &str = "minecraft-launcher";

const STEAM_LIST: &str = "/etc/apt/sources.list.d/steam-stable.list";

/// `sunshine --creds <user> <password>` as Sunshine's own does it: the sign-in written into the
/// state file of its folder, the password hashed with a new salt, the file's other keys kept.
const SUNSHINE_CREDS: &str = r#"[ "$1" = --creds ] || exit 7
dir="${XDG_CONFIG_HOME:-$HOME/.config}/sunshine"
mkdir -p "$dir"
state="$dir/sunshine_state.json"
[ -f "$state" ] || echo '{}' >"$state"
salt=$(head -c 12 /dev/urandom | base64 | tr -dc A-Za-z0-9)
hash=$(printf '%s%s' "$3" "$salt" | sha256sum | cut -d ' ' -f 1 | tr a-f A-F)
jq --arg u "$2" --arg s "$salt" --arg p "$hash" '.username=$u | .salt=$s | .password=$p' "$state" >"$state.new"
mv "$state.new" "$state""#;

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
    world.host.sandbox().stub("sunshine", SUNSHINE_CREDS);
    // Steam's steps name its steam-stable.list as an argument, which its scripts write as they
    // are, so its folder names the fake machine's instead, in a sources folder that is there.
    if id == "steam" {
        let list = world.system_path(STEAM_LIST);
        fs::create_dir_all(list.parent().unwrap()).unwrap();
        let text = fs::read_to_string(world.catalog.join(id).join("app.toml")).unwrap();
        world.write_app(id, &text.replace(STEAM_LIST, &list.display().to_string()));
    }
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
            Kind::File { to, .. } | Kind::Output { path: to } => {
                assert!(home_or_system(to).is_file(), "{id}: {to}");
            }
            // Each app's own test checks what its runs did.
            Kind::Run { .. } => {}
            Kind::Architecture { architecture } => assert!(state.foreign.contains(architecture), "{id}"),
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
    // Mojang builds its launcher for amd64 alone, nixpkgs ProtonUp-Qt, Valve its Steam client, and
    // the two cloud services' windows need Chrome, which Google builds for amd64 alone.
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

/// Sunshine's state file and the password its install saved, in `world`'s home.
fn sunshine_sign_in(world: &World) -> (std::path::PathBuf, std::path::PathBuf) {
    let home = world.host_home();
    (
        home.join(".config/sunshine/sunshine_state.json"),
        home.join(".config/kubaos/sunshine/password"),
    )
}

#[test]
fn sunshines_web_page_has_your_sign_in_before_it_first_starts_and_its_removal_takes_it() {
    use std::os::unix::fs::PermissionsExt;

    let world = served("sunshine");
    let before = world.host.snapshot();
    let (state, saved) = sunshine_sign_in(&world);

    run(&world, |e| e.add("sunshine", None, &picks("sunshine"))).unwrap();

    let sign_in: serde_json::Value = serde_json::from_str(&fs::read_to_string(&state).unwrap()).unwrap();
    let user = world.host.sandbox().command("id").arg("-un").output().unwrap().stdout;
    assert_eq!(
        sign_in["username"],
        String::from_utf8(user).unwrap().trim(),
        "the login name"
    );
    let password = fs::read_to_string(&saved).unwrap();
    let password = password.trim_end();
    assert!(
        password.len() >= 32 && password.chars().all(|c| c.is_ascii_alphanumeric()),
        "{password:?}"
    );
    assert_eq!(fs::metadata(&saved).unwrap().permissions().mode() & 0o777, 0o600);
    let calls = fs::read_to_string(world.host.sandbox().log()).unwrap();
    assert!(calls.lines().any(|l| l.starts_with("sunshine --creds ")), "{calls}");
    let salted = world
        .host
        .sandbox()
        .command("sh")
        .arg("-c")
        .arg("printf '%s%s' \"$1\" \"$2\" | sha256sum")
        .arg("sh")
        .arg(password)
        .arg(sign_in["salt"].as_str().unwrap())
        .output()
        .unwrap()
        .stdout;
    assert_eq!(
        sign_in["password"].as_str().unwrap().to_lowercase(),
        String::from_utf8(salted).unwrap()[..64],
        "the saved password is the page's"
    );

    run(&world, |e| e.remove_with("sunshine", &Picks::default())).unwrap();

    assert!(!saved.exists() && !state.exists());
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn a_sunshine_sign_in_already_there_is_adopted_and_left_alone() {
    let world = served("sunshine");
    let (state, saved) = sunshine_sign_in(&world);
    fs::create_dir_all(state.parent().unwrap()).unwrap();
    let mine =
        "{\"username\": \"mine\", \"salt\": \"pepper\", \"password\": \"ABC\", \"root\": {\"named_devices\": []}}\n";
    fs::write(&state, mine).unwrap();

    run(&world, |e| e.add("sunshine", None, &picks("sunshine"))).unwrap();

    assert_eq!(fs::read_to_string(&state).unwrap(), mine);
    assert!(!saved.exists(), "no password of KubaOS's for a sign-in it didn't set");
    let calls = fs::read_to_string(world.host.sandbox().log()).unwrap_or_default();
    let creds = calls.lines().any(|l| l.starts_with("sunshine "));
    assert!(!creds, "Sunshine's command never ran");

    run(&world, |e| e.remove_with("sunshine", &Picks::default())).unwrap();

    assert_eq!(fs::read_to_string(&state).unwrap(), mine);
}

#[test]
fn a_sunshine_sign_in_you_changed_and_your_paired_clients_stay_after_its_removal() {
    let world = served("sunshine");
    let (state, saved) = sunshine_sign_in(&world);

    run(&world, |e| e.add("sunshine", None, &picks("sunshine"))).unwrap();
    // The user pairs a client, then changes the password in Sunshine's page.
    let mut sign_in: serde_json::Value = serde_json::from_str(&fs::read_to_string(&state).unwrap()).unwrap();
    sign_in["root"] = serde_json::json!({ "named_devices": [{ "name": "Steam Deck" }] });
    let paired = sign_in.clone();
    sign_in["password"] = serde_json::json!("0".repeat(64));
    let changed = serde_json::to_string_pretty(&sign_in).unwrap();
    fs::write(&state, &changed).unwrap();

    run(&world, |e| e.remove_with("sunshine", &Picks::default())).unwrap();

    assert!(!saved.exists(), "the saved password goes with the app");
    assert_eq!(fs::read_to_string(&state).unwrap(), changed);

    // Its own sign-in, still the one it set, goes; the paired client stays.
    fs::remove_file(&state).unwrap();
    run(&world, |e| e.add("sunshine", None, &picks("sunshine"))).unwrap();
    let mut set: serde_json::Value = serde_json::from_str(&fs::read_to_string(&state).unwrap()).unwrap();
    set["root"] = paired["root"].clone();
    fs::write(&state, serde_json::to_string(&set).unwrap()).unwrap();

    run(&world, |e| e.remove_with("sunshine", &Picks::default())).unwrap();

    let left: serde_json::Value = serde_json::from_str(&fs::read_to_string(&state).unwrap()).unwrap();
    assert_eq!(left, serde_json::json!({ "root": paired["root"] }));
}

#[test]
fn a_sunshine_removal_apt_cant_finish_puts_its_sign_in_and_its_local_page_back() {
    let world = served("sunshine");
    run(&world, |e| e.add("sunshine", None, &picks("sunshine"))).unwrap();
    let (state, saved) = sunshine_sign_in(&world);
    let conf = world.host_home().join(".config/sunshine/sunshine.conf");
    // apt can't take the package, as with dpkg's lock held, after the sign-in and the setting went.
    world
        .host
        .state
        .borrow_mut()
        .fail
        .insert("apt-remove:sunshine".to_owned());

    let mut events = super::Events::default();
    let removed = {
        let mut root = world.root();
        let mut user_host = world.host.clone();
        let keeper = Keeper::new(&mut user_host, Side::User, &world.catalog, &world.user_journals);
        let mut engine = Engine::new(keeper, &mut root, &world.root_journals, &world.lock, &mut events, true);
        engine.remove_with("sunshine", &Picks::default())
    };

    assert!(
        matches!(removed, Err(EngineError::RemovalRolledBack { .. })),
        "{removed:?}"
    );
    let renewed = "Sign Sunshine in as you before a stranger does, with a new password in \
                   ~/.config/kubaos/sunshine/password: I ran it again to put it back, so what it makes is new.";
    assert!(
        events.0.contains(&UpdateEvent::Note {
            text: renewed.to_owned()
        }),
        "{:?}",
        events.0
    );
    assert!(world.host.state.borrow().dpkg.contains_key("sunshine"));
    assert_eq!(fs::read_to_string(&conf).unwrap(), "origin_web_ui_allowed = pc\n");
    let sign_in: serde_json::Value = serde_json::from_str(&fs::read_to_string(&state).unwrap()).unwrap();
    assert!(sign_in.get("username").is_some(), "{sign_in}");
    assert!(saved.is_file(), "a new sign-in, with its password saved");
    let user = world.journals("sunshine").0.unwrap();
    assert!(user.complete, "{user:?}");
}

#[test]
fn a_sunshine_conf_line_already_set_keeps_your_comment_and_stays_after_the_removal() {
    let world = served("sunshine");
    let conf = world.host_home().join(".config/sunshine/sunshine.conf");
    fs::create_dir_all(conf.parent().unwrap()).unwrap();
    let mine = "origin_web_ui_allowed = pc # this machine only\n";
    fs::write(&conf, mine).unwrap();

    run(&world, |e| e.add("sunshine", None, &picks("sunshine"))).unwrap();
    assert_eq!(fs::read_to_string(&conf).unwrap(), mine);
    run(&world, |e| e.remove_with("sunshine", &Picks::default())).unwrap();

    assert_eq!(fs::read_to_string(&conf).unwrap(), mine);
}

#[test]
fn an_adopted_sunshine_sign_in_is_found_and_no_step_claims_a_password_file() {
    let world = served("sunshine");
    let (state, _) = sunshine_sign_in(&world);
    fs::create_dir_all(state.parent().unwrap()).unwrap();
    fs::write(
        &state,
        "{\"username\": \"mine\", \"salt\": \"pepper\", \"password\": \"ABC\"}\n",
    )
    .unwrap();

    run(&world, |e| e.add("sunshine", None, &picks("sunshine"))).unwrap();

    let user = world.journals("sunshine").0.unwrap();
    let sign_in = user
        .entries
        .iter()
        .find(|e| matches!(e.action.kind, Kind::Run { .. }))
        .unwrap();
    assert_eq!(sign_in.state, crate::apps::journal::State::Found);
    let claims: Vec<&str> = user
        .entries
        .iter()
        .flat_map(|e| &e.claims)
        .map(|c| c.claim.as_str())
        .collect();
    assert!(claims.iter().all(|c| !c.contains("password")), "{claims:?}");
}

#[test]
fn steam_takes_i386_and_opts_out_of_its_packages_source_before_the_package_lands() {
    let world = served("steam");

    run(&world, |e| e.add("steam", None, &picks("steam"))).unwrap();

    let state = world.host.state.borrow();
    assert!(state.foreign.contains("i386"));
    assert!(state.repos.contains("steam"));
    for package in ["steam-launcher", "steam-libs-i386:i386", "mesa-vulkan-drivers:i386"] {
        assert!(state.dpkg.contains_key(package), "{package}");
    }
    let at = |what: &str| state.log.iter().position(|l| l.starts_with(what));
    assert!(at("arch i386 true") < at("run ./opt-out"), "{:#?}", state.log);
    assert!(
        at("run ./opt-out") < at("apt install steam-launcher"),
        "{:#?}",
        state.log
    );
    let list = fs::read_to_string(world.system_path(STEAM_LIST)).unwrap();
    assert!(list.lines().all(|l| l.starts_with('#')), "{list}");
}

#[test]
fn steams_removal_takes_valves_copy_of_its_source_and_keeps_the_library() {
    let world = served("steam");
    let before = world.host.snapshot();
    run(&world, |e| e.add("steam", None, &picks("steam"))).unwrap();
    // What dpkg leaves beside the conffile it found, and what Steam makes as you play.
    let list = world.system_path(STEAM_LIST);
    let valves = list.with_extension("list.dpkg-dist");
    fs::write(
        &valves,
        "deb [arch=amd64,i386 signed-by=/usr/share/keyrings/steam.gpg] https://repo.steampowered.com/steam/ stable steam\n",
    )
    .unwrap();
    let library = world
        .host_home()
        .join(".local/share/Steam/steamapps/libraryfolders.vdf");
    fs::create_dir_all(library.parent().unwrap()).unwrap();
    fs::write(&library, "").unwrap();

    run(&world, |e| e.remove_with("steam", &Picks::default())).unwrap();

    assert!(!list.exists() && !valves.exists());
    assert!(library.is_file(), "the library stays");
    assert!(!world.host.state.borrow().foreign.contains("i386"));
    fs::remove_dir_all(world.host_home().join(".local")).unwrap();
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn the_source_of_a_steam_installed_by_hand_stays_and_so_does_i386() {
    let world = served("steam");
    let list = world.system_path(STEAM_LIST);
    let valves = "deb [arch=amd64,i386 signed-by=/usr/share/keyrings/steam.gpg] https://repo.steampowered.com/steam/ stable steam\n";
    fs::write(&list, valves).unwrap();
    world.host.state.borrow_mut().foreign.insert("i386".to_owned());

    run(&world, |e| e.add("steam", None, &picks("steam"))).unwrap();
    run(&world, |e| e.remove_with("steam", &Picks::default())).unwrap();

    assert_eq!(fs::read_to_string(&list).unwrap(), valves);
    assert!(world.host.state.borrow().foreign.contains("i386"));
}
