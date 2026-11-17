//! The system and sync apps of the shipped catalog, installed and removed on the fake machine.

use std::fs;
use std::path::PathBuf;

use super::Events;
use super::World;
use super::ai::run;
use super::ai::shipped;
use super::ai::world;
use crate::apps::Catalog;
use crate::apps::Folder;
use crate::apps::Platform;
use crate::apps::action::Kind;
use crate::apps::host::fake::Unit;
use crate::apps::picks::Picks;
use crate::unfree::Unfree;
use crate::update::event::UpdateEvent;

const APPS: [&str; 14] = [
    "cascadia-mono-nerd-font",
    "dropbox",
    "fira-code-nerd-font",
    "gparted",
    "inter-font",
    "iosevka-nerd-font",
    "localsend",
    "meslo-lg-nerd-font",
    "mission-center",
    "nextcloud",
    "openssh-server",
    "syncthing",
    "tailscale",
    "victor-mono-nerd-font",
];

const DROPBOX_REPO: &str = "/etc/default/dropbox-repo";

/// The units the apps' packages bring, there on the fake machine as dpkg leaves them: neither
/// enabled nor running.
const PACKAGED_UNITS: [&str; 3] = [
    "system:tailscaled.service",
    "user:syncthing.service",
    "system:ssh.socket",
];

fn picks(id: &str) -> Picks {
    let mut picks = Picks::default();
    picks.add(id);
    picks
}

/// The shipped app `id`'s world, with its package's units. Dropbox's steps name
/// `/etc/default/dropbox-repo` as an argument, which its scripts write as they are, so its folder
/// names the fake machine's instead, in a folder that is there, as on any machine.
fn system_world(id: &str) -> World {
    let world = world(id);
    {
        let mut state = world.host.state.borrow_mut();
        for unit in PACKAGED_UNITS {
            state.units.insert(
                unit.to_owned(),
                Unit {
                    exists: true,
                    ..Unit::default()
                },
            );
        }
    }
    if id == "dropbox" {
        let file = world.system_path(DROPBOX_REPO);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        let text = fs::read_to_string(world.catalog.join(id).join("app.toml")).unwrap();
        world.write_app(id, &text.replace(DROPBOX_REPO, &file.display().to_string()));
    }
    world
}

/// The kinds of the app `id`'s steps, in order.
fn kinds(world: &World, id: &str) -> Vec<&'static str> {
    let catalog = Catalog::load(&world.catalog).unwrap();
    catalog
        .get(id)
        .unwrap()
        .actions
        .iter()
        .map(|a| match a.kind {
            Kind::Apt { .. } => "apt",
            Kind::File { .. } => "file",
            Kind::Unit { .. } => "unit",
            Kind::Run { .. } => "run",
            _ => "another",
        })
        .collect()
}

fn home_path(world: &World, path: &str) -> PathBuf {
    world.host_home().join(path.trim_start_matches("~/"))
}

/// Each of the installed `app`'s steps there on the fake machine: its packages, files and units.
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
            Kind::File { to, .. } => {
                let path = match to.strip_prefix("~/") {
                    Some(_) => home_path(world, to),
                    None => world.system_path(to),
                };
                assert!(path.is_file(), "{id}: {}", path.display());
            }
            Kind::Unit { unit, scope, .. } => {
                let unit = &state.units[&format!("{}:{unit}", scope.word())];
                assert!(unit.enabled && unit.active, "{id}: {unit:?}");
            }
            // Room made for what the app writes as it first runs, and the steps of scripts.
            Kind::Output { .. } | Kind::Run { .. } => {}
            other => panic!("{id}: a system app is its package, files and units, not {other:?}"),
        }
    }
}

#[test]
fn every_system_app_installs_on_the_fake_machine_and_its_removal_leaves_it_as_before() {
    for id in APPS {
        let world = system_world(id);
        let before = world.host.snapshot();

        let ran = run(&world, |e| e.add(id, None, &picks(id))).unwrap_or_else(|e| panic!("{id}: {e}"));

        let catalog = Catalog::load(&world.catalog).unwrap();
        let app = catalog.get(id).unwrap();
        let names: Vec<&str> = ran.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, [app.name.as_str()], "{id}");
        in_place(&world, app);

        let gone = run(&world, |e| e.remove_with(id, &Picks::default())).unwrap_or_else(|e| panic!("{id}: {e}"));

        assert_eq!(gone.len(), 1, "{id}");
        assert_eq!(world.host.snapshot(), before, "{id}");
        assert_eq!(world.journals(id), (None, None), "{id}");
    }
}

#[test]
fn which_system_apps_a_machine_can_have_follows_its_architecture() {
    let catalog = Catalog::load(&shipped("tailscale").join("..")).unwrap();
    let on = |arch: &str, unfree: Unfree| -> Vec<&str> {
        APPS.into_iter()
            .filter(|id| catalog.available(catalog.get(id).unwrap(), None, Platform::new(arch, unfree)))
            .collect()
    };

    assert_eq!(on("amd64", Unfree::Allowed), APPS);
    // None of them is unfree in nixpkgs: Dropbox comes from its own repository.
    assert_eq!(on("amd64", Unfree::Refused), APPS);
    // Dropbox builds for amd64 alone, and Nextcloud's PPA builds its client for amd64 alone.
    let arm64 = on("arm64", Unfree::Allowed);
    let missing: Vec<&str> = APPS.into_iter().filter(|id| !arm64.contains(id)).collect();
    assert_eq!(missing, ["dropbox", "nextcloud"]);
}

#[test]
fn tailscales_daemon_starts_at_boot_while_the_app_is_installed_and_no_longer_once_it_goes() {
    let world = system_world("tailscale");

    run(&world, |e| e.add("tailscale", None, &picks("tailscale"))).unwrap();

    let unit = world.host.state.borrow().units["system:tailscaled.service"];
    assert!(unit.enabled && unit.active, "{unit:?}");

    run(&world, |e| e.remove_with("tailscale", &Picks::default())).unwrap();

    let unit = world.host.state.borrow().units["system:tailscaled.service"];
    assert!(!unit.enabled && !unit.active, "{unit:?}");
    let log = world.host.state.borrow().log.clone();
    let at = |what: &str| log.iter().position(|l| l == what).unwrap();
    assert!(
        at("enable tailscaled.service false") < at("apt remove tailscale"),
        "{log:#?}"
    );
}

#[test]
fn a_tailscale_daemon_the_user_had_enabled_stays_enabled_when_the_app_goes() {
    let world = system_world("tailscale");
    world.host.state.borrow_mut().units.insert(
        "system:tailscaled.service".to_owned(),
        Unit {
            exists: true,
            enabled: true,
            active: true,
            ..Unit::default()
        },
    );

    run(&world, |e| e.add("tailscale", None, &picks("tailscale"))).unwrap();
    run(&world, |e| e.remove_with("tailscale", &Picks::default())).unwrap();

    let unit = world.host.state.borrow().units["system:tailscaled.service"];
    assert!(unit.enabled && unit.active, "{unit:?}");
}

#[test]
fn syncthings_web_page_answers_on_this_machine_alone_before_its_service_first_starts() {
    let world = system_world("syncthing");

    run(&world, |e| e.add("syncthing", None, &picks("syncthing"))).unwrap();

    let drop_in = home_path(&world, "~/.config/systemd/user/syncthing.service.d/kubaos-gui.conf");
    let text = fs::read_to_string(&drop_in).unwrap();
    let lines: Vec<&str> = text.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(lines, ["[Service]", "Environment=STGUIADDRESS=127.0.0.1:8384"]);
    let unit = world.host.state.borrow().units["user:syncthing.service"];
    assert!(unit.enabled && unit.active, "{unit:?}");
    assert_eq!(
        kinds(&world, "syncthing"),
        ["apt", "file", "unit"],
        "the drop-in is there before the unit starts"
    );
}

#[test]
fn dropbox_opts_out_of_its_packages_source_before_the_package_lands() {
    let world = system_world("dropbox");
    let file = world.system_path(DROPBOX_REPO);

    run(&world, |e| e.add("dropbox", None, &picks("dropbox"))).unwrap();

    let text = fs::read_to_string(&file).unwrap();
    assert!(text.lines().all(|l| l.starts_with('#')), "nothing but comments: {text}");
    let state = world.host.state.borrow();
    let at = |what: &str| state.log.iter().position(|l| l.starts_with(what)).unwrap();
    assert!(at("run ./opt-out") < at("apt install dropbox"), "{:#?}", state.log);
    assert!(
        state.dpkg.contains_key("python3-gpg"),
        "the daemon's download is checked"
    );
    assert!(state.repos.contains("dropbox"));
}

#[test]
fn a_dropbox_repo_file_the_user_wrote_stays_when_kubaos_removes_dropbox() {
    let world = system_world("dropbox");
    let file = world.system_path(DROPBOX_REPO);
    fs::write(&file, "# mine\n").unwrap();

    run(&world, |e| e.add("dropbox", None, &picks("dropbox"))).unwrap();
    run(&world, |e| e.remove_with("dropbox", &Picks::default())).unwrap();

    assert_eq!(fs::read_to_string(&file).unwrap(), "# mine\n");
    let calls = world.host.state.borrow().log.clone();
    assert!(!calls.iter().any(|c| c.starts_with("run ./opt-out")), "{calls:#?}");
}

#[test]
fn the_daemon_and_entry_dropbox_writes_as_it_first_runs_go_with_it_and_ones_there_before_stay() {
    let world = system_world("dropbox");
    let daemon = home_path(&world, "~/.dropbox-dist/dropbox-lnx.x86_64-240.4.8493/dropbox");
    let entry = home_path(&world, "~/.config/autostart/dropbox.desktop");

    run(&world, |e| e.add("dropbox", None, &picks("dropbox"))).unwrap();
    // Dropbox's first start, from its menu entry.
    fs::create_dir_all(daemon.parent().unwrap()).unwrap();
    fs::write(&daemon, "daemon").unwrap();
    fs::write(&entry, "[Desktop Entry]\n").unwrap();
    run(&world, |e| e.remove_with("dropbox", &Picks::default())).unwrap();

    assert!(!home_path(&world, "~/.dropbox-dist").exists());
    assert!(!entry.exists());

    // A Dropbox the user had started before: its daemon and entry are theirs.
    fs::create_dir_all(daemon.parent().unwrap()).unwrap();
    fs::create_dir_all(entry.parent().unwrap()).unwrap();
    fs::write(&daemon, "daemon").unwrap();
    fs::write(&entry, "[Desktop Entry]\n").unwrap();
    run(&world, |e| e.add("dropbox", None, &picks("dropbox"))).unwrap();
    run(&world, |e| e.remove_with("dropbox", &Picks::default())).unwrap();

    assert!(daemon.is_file());
    assert!(entry.is_file());
}

#[test]
fn nextclouds_start_at_login_entry_goes_with_it() {
    let world = system_world("nextcloud");
    let entry = home_path(&world, "~/.config/autostart/Nextcloud.desktop");

    run(&world, |e| e.add("nextcloud", None, &picks("nextcloud"))).unwrap();
    // What Nextcloud writes as it first starts.
    fs::write(&entry, "[Desktop Entry]\n").unwrap();
    run(&world, |e| e.remove_with("nextcloud", &Picks::default())).unwrap();

    assert!(!entry.exists());
    assert!(
        !home_path(&world, "~/.config/autostart").exists(),
        "the folder the step made goes"
    );
}

#[test]
fn the_ssh_servers_drop_in_turns_passwords_and_root_off_before_sshd_can_run() {
    let world = system_world("openssh-server");

    run(&world, |e| e.add("openssh-server", None, &picks("openssh-server"))).unwrap();

    let text = fs::read_to_string(world.system_path("/etc/ssh/sshd_config.d/00-kubaos-key-only.conf")).unwrap();
    let settings: Vec<&str> = text.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(
        settings,
        [
            "PasswordAuthentication no",
            "KbdInteractiveAuthentication no",
            "PermitRootLogin no"
        ]
    );
    assert_eq!(
        kinds(&world, "openssh-server"),
        ["file", "apt", "unit", "run"],
        "the drop-in is there before sshd"
    );
    let socket = world.host.state.borrow().units["system:ssh.socket"];
    assert!(socket.enabled && socket.active, "{socket:?}");
}

/// The notes of `events`.
fn notes(events: &Events) -> Vec<String> {
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
fn an_ssh_server_no_key_can_log_in_to_says_so_and_installs_all_the_same() {
    // With a window: a notification on the desktop.
    let world = system_world("openssh-server");

    let (done, _) = world.install("openssh-server", true);

    done.unwrap();
    let calls = world.host.sandbox().calls();
    assert!(
        calls
            .iter()
            .any(|c| c.starts_with("notify-send ") && c.contains("authorized_keys")),
        "{calls:#?}"
    );
    assert!(world.journals("openssh-server").0.unwrap().complete);

    // Without one, as an update runs it: a note, every time until a key is there.
    let world = system_world("openssh-server");

    let (done, events) = world.install("openssh-server", false);

    done.unwrap();
    let says = notes(&events);
    assert!(says.iter().any(|n| n.contains("authorized_keys")), "{says:#?}");
    assert!(world.journals("openssh-server").0.unwrap().complete);
    let (_, events) = world.install("openssh-server", false);
    assert!(notes(&events).iter().any(|n| n.contains("authorized_keys")));
}

#[test]
fn an_ssh_server_whose_user_has_a_key_says_nothing_of_it() {
    let world = system_world("openssh-server");
    let keys = home_path(&world, "~/.ssh/authorized_keys");
    fs::create_dir_all(keys.parent().unwrap()).unwrap();
    fs::write(
        &keys,
        "# my laptop\nssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIExample kuba@laptop\n",
    )
    .unwrap();

    let (done, events) = world.install("openssh-server", false);

    done.unwrap();
    assert!(notes(&events).is_empty(), "{:#?}", notes(&events));
    assert!(
        !world
            .host
            .sandbox()
            .calls()
            .iter()
            .any(|c| c.starts_with("notify-send"))
    );
}

#[test]
fn an_authorized_keys_file_of_comments_alone_lets_nobody_in() {
    let world = system_world("openssh-server");
    let keys = home_path(&world, "~/.ssh/authorized_keys");
    fs::create_dir_all(keys.parent().unwrap()).unwrap();
    fs::write(&keys, "# nothing yet\n\n").unwrap();

    let (done, events) = world.install("openssh-server", false);

    done.unwrap();
    assert!(notes(&events).iter().any(|n| n.contains("authorized_keys")));
}
