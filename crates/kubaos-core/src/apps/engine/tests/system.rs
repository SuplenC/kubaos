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
use crate::apps::journal::State;
use crate::apps::picks::Picks;
use crate::unfree::Unfree;
use crate::update::event::UpdateEvent;

const APPS: [&str; 8] = [
    "dropbox",
    "gparted",
    "localsend",
    "mission-center",
    "nextcloud",
    "openssh-server",
    "syncthing",
    "tailscale",
];

const DROPBOX_REPO: &str = "/etc/default/dropbox-repo";

const DROP_IN: &str = "/etc/ssh/sshd_config.d/00-kubaos-key-only.conf";

fn picks(id: &str) -> Picks {
    let mut picks = Picks::default();
    picks.add(id);
    picks
}

fn unit(enabled: bool, active: bool) -> Unit {
    Unit {
        exists: true,
        enabled,
        active,
        ..Unit::default()
    }
}

/// The units the apps' packages bring, as each package's postinst leaves them on a first install:
/// openssh-server's enables and starts `ssh.socket`, tailscale's starts tailscaled without
/// enabling it, syncthing's ships its user unit and does nothing with it.
fn postinsts(world: &World) {
    let mut state = world.host.state.borrow_mut();
    for (package, units) in [
        (
            "openssh-server",
            vec![
                ("system:ssh.socket", unit(true, true)),
                ("system:ssh.service", unit(false, false)),
            ],
        ),
        ("tailscale", vec![("system:tailscaled.service", unit(false, true))]),
        ("syncthing", vec![("user:syncthing.service", unit(false, false))]),
    ] {
        state.package_units.insert(
            package.to_owned(),
            units.into_iter().map(|(k, u)| (k.to_owned(), u)).collect(),
        );
    }
}

/// Where the fake machine keeps the SSH server's mark that a reload went through, outside the
/// machine's own folders, as `/run` empties at every boot.
fn mark(world: &World) -> PathBuf {
    world
        .host
        .sandbox()
        .log()
        .with_file_name("run/kubaos/openssh-server-reloaded")
}

/// A file of the sandbox's beside its log, through which a test tells `systemctl` what to say.
fn told(world: &World, name: &str) -> PathBuf {
    world.host.sandbox().log().with_file_name(name)
}

/// A `systemctl` that answers `is-enabled`, `is-active` and `show -p ActiveState` from the fake
/// machine's units (the `units` file it writes before each script); `ExecMainStartTimestamp` from
/// `ssh-started`, when a test wrote it; and, reloading sshd, fails while `reload-fails` is there,
/// and logs whether the drop-in was there.
fn systemctl_reads_the_units(world: &World) {
    let sandbox = world.host.sandbox();
    let units = sandbox.log().with_file_name("units");
    sandbox.stub(
        "systemctl",
        &format!(
            r#"units='{units}'
scope=system
if [ "$1" = --user ]; then scope=user; shift; fi
field() {{ grep "^$scope:$1 " "$units" 2>/dev/null | cut -d' ' -f"$2"; }}
case "$1" in
is-enabled) s=$(field "$2" 2); echo "${{s:-disabled}}"; [ "$s" = enabled ] ;;
is-active) s=$(field "$2" 3); echo "${{s:-inactive}}"; [ "$s" = active ] ;;
show)
	case "$*" in
	*ActiveState*) s=$(field "$2" 3); echo "${{s:-inactive}}" ;;
	*ExecMainStartTimestamp*) cat '{started}' 2>/dev/null || echo ;;
	esac ;;
try-reload-or-restart)
	if [ -e '{fails}' ]; then echo 'Job for ssh.service failed.' >&2; exit 1; fi
	if [ -e '{drop_in}' ]; then w=with; else w=without; fi
	echo "sshd reloaded $w the drop-in" >>'{log}' ;;
esac"#,
            units = units.display(),
            started = told(world, "ssh-started").display(),
            fails = told(world, "reload-fails").display(),
            drop_in = world.system_path(DROP_IN).display(),
            log = sandbox.log().display(),
        ),
    );
}

/// The shipped app `id`'s world, its packages' postinsts and a `systemctl` that sees the units.
/// Dropbox's steps name `/etc/default/dropbox-repo`, and the SSH server's check names its drop-in,
/// as arguments, which their scripts read as they are, so their folders name the fake machine's
/// paths instead, in folders that are there, as on any machine.
fn system_world(id: &str) -> World {
    let world = world(id);
    postinsts(&world);
    systemctl_reads_the_units(&world);
    let toml = world.catalog.join(id).join("app.toml");
    let text = fs::read_to_string(&toml).unwrap();
    if id == "dropbox" {
        let file = world.system_path(DROPBOX_REPO);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        world.write_app(id, &text.replace(DROPBOX_REPO, &file.display().to_string()));
    }
    if id == "openssh-server" {
        let check = format!("  \"{DROP_IN}\",\n");
        assert!(text.contains(&check), "{text}");
        let fake = format!("  \"{}\",\n", world.system_path(DROP_IN).display());
        let text = text.replace(&check, &fake).replace(
            "/run/kubaos/openssh-server-reloaded",
            &mark(&world).display().to_string(),
        );
        world.write_app(id, &text);
    }
    world
}

/// The state of step `step` of the app `id` in its root journal.
fn root_step(world: &World, id: &str, step: usize) -> Option<State> {
    let (_, root) = world.journals(id);
    root?.entries.into_iter().find(|e| e.step == step).map(|e| e.state)
}

/// The sandbox's calls of `systemctl` that change something, and its notes of sshd's reloads.
fn systemctl_calls(world: &World) -> Vec<String> {
    world
        .host
        .sandbox()
        .calls()
        .into_iter()
        .filter(|c| {
            c.starts_with("systemctl enable")
                || c.starts_with("systemctl disable")
                || c.starts_with("systemctl try-")
                || c.starts_with("systemctl stop")
                || c.starts_with("sshd reloaded")
        })
        .collect()
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

    // Its postinst started it; the step enabled it.
    let unit = world.host.state.borrow().units["system:tailscaled.service"];
    assert!(unit.enabled && unit.active, "{unit:?}");
    assert_eq!(root_step(&world, "tailscale", 2), Some(State::Done));

    run(&world, |e| e.remove_with("tailscale", &Picks::default())).unwrap();

    let log = world.host.state.borrow().log.clone();
    let at = |what: &str| log.iter().position(|l| l == what).unwrap();
    assert!(
        at("enable tailscaled.service false") < at("apt remove tailscale"),
        "{log:#?}"
    );
    assert!(
        !world
            .host
            .state
            .borrow()
            .units
            .contains_key("system:tailscaled.service")
    );
}

#[test]
fn syncthings_web_page_answers_on_this_machine_alone_before_its_service_first_starts() {
    let world = system_world("syncthing");

    run(&world, |e| e.add("syncthing", None, &picks("syncthing"))).unwrap();

    let drop_in = home_path(&world, "~/.config/systemd/user/syncthing.service.d/kubaos-gui.conf");
    let text = fs::read_to_string(&drop_in).unwrap();
    let lines: Vec<&str> = text.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(lines, ["[Service]", "Environment=STGUIADDRESS=http://127.0.0.1:8384"]);
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
fn a_fresh_ssh_server_takes_keys_alone_and_listens_through_the_socket_its_package_starts() {
    let world = system_world("openssh-server");

    run(&world, |e| e.add("openssh-server", None, &picks("openssh-server"))).unwrap();

    let text = fs::read_to_string(world.system_path(DROP_IN)).unwrap();
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
        ["run", "file", "run", "apt", "run", "run", "file"],
        "the drop-in is there before sshd, the removal's reload comes after it goes, and Puffy last"
    );
    // The package's postinst enabled and started ssh.socket, so the step found it.
    let socket = world.host.state.borrow().units["system:ssh.socket"];
    assert!(socket.enabled && socket.active, "{socket:?}");
    assert_eq!(root_step(&world, "openssh-server", 5), Some(State::Found));
    // No sshd ran before the package, so none was reloaded.
    assert_eq!(root_step(&world, "openssh-server", 3), Some(State::Found));
    assert!(systemctl_calls(&world).is_empty(), "{:#?}", systemctl_calls(&world));

    run(&world, |e| e.remove_with("openssh-server", &Picks::default())).unwrap();

    let state = world.host.state.borrow();
    assert!(!state.dpkg.contains_key("openssh-server"));
    assert!(
        !state.units.contains_key("system:ssh.socket"),
        "the package took its socket"
    );
    drop(state);
    assert_eq!(
        systemctl_calls(&world),
        [
            "systemctl try-reload-or-restart ssh.service",
            "sshd reloaded without the drop-in"
        ]
    );
}

/// The SSH server's package there already, with its units `socket` and `service`.
fn ssh_server_there(world: &World, socket: Unit, service: Unit) {
    let mut state = world.host.state.borrow_mut();
    state
        .dpkg
        .insert("openssh-server".to_owned(), "1:10.2p1-2ubuntu3.8".to_owned());
    state.units.insert("system:ssh.socket".to_owned(), socket);
    state.units.insert("system:ssh.service".to_owned(), service);
}

#[test]
fn an_ssh_server_there_with_its_socket_off_listens_while_the_app_is_installed() {
    let world = system_world("openssh-server");
    ssh_server_there(&world, unit(false, false), unit(false, false));

    run(&world, |e| e.add("openssh-server", None, &picks("openssh-server"))).unwrap();

    assert_eq!(root_step(&world, "openssh-server", 5), Some(State::Done));
    assert_eq!(systemctl_calls(&world), ["systemctl enable --now ssh.socket"]);

    run(&world, |e| e.remove_with("openssh-server", &Picks::default())).unwrap();

    let calls = systemctl_calls(&world);
    assert_eq!(
        calls[1..3],
        ["systemctl disable --now ssh.socket", "systemctl stop ssh.service"],
        "the socket's sshd stops with it: {calls:#?}"
    );
}

#[test]
fn a_running_sshd_takes_keys_alone_at_once_and_the_users_configuration_again_once_the_app_goes() {
    let world = system_world("openssh-server");
    ssh_server_there(&world, unit(true, true), unit(false, true));

    run(&world, |e| e.add("openssh-server", None, &picks("openssh-server"))).unwrap();

    assert_eq!(
        systemctl_calls(&world),
        [
            "systemctl try-reload-or-restart ssh.service",
            "sshd reloaded with the drop-in"
        ]
    );
    assert_eq!(root_step(&world, "openssh-server", 3), Some(State::Done));

    run(&world, |e| e.remove_with("openssh-server", &Picks::default())).unwrap();

    let calls = systemctl_calls(&world);
    assert_eq!(
        calls.last().map(String::as_str),
        Some("sshd reloaded without the drop-in"),
        "{calls:#?}"
    );
    assert!(!world.system_path(DROP_IN).exists());
}

#[test]
fn an_sshd_its_configuration_check_refuses_fails_the_install_and_takes_the_drop_in_out() {
    let world = system_world("openssh-server");
    ssh_server_there(&world, unit(true, true), unit(false, true));
    // ssh.service's reload runs sshd -t first, which refuses the user's configuration.
    fs::write(told(&world, "reload-fails"), "").unwrap();
    let before = world.host.snapshot();

    let added = run(&world, |e| e.add("openssh-server", None, &picks("openssh-server")));

    let err = added.unwrap_err().to_string();
    assert!(err.contains("Have a running sshd take keys alone at once"), "{err}");
    assert!(!world.system_path(DROP_IN).exists());
    assert!(!mark(&world).exists(), "a reload that failed counts for nothing");
    assert_eq!(world.host.snapshot(), before);
}

/// The calls that reloaded sshd.
fn reloads(world: &World) -> usize {
    systemctl_calls(world)
        .iter()
        .filter(|c| *c == "systemctl try-reload-or-restart ssh.service")
        .count()
}

#[test]
fn a_running_sshd_that_read_the_drop_in_isnt_reloaded_again_by_the_next_update() {
    let world = system_world("openssh-server");
    ssh_server_there(&world, unit(true, true), unit(false, true));
    // sshd started long before the drop-in.
    fs::write(told(&world, "ssh-started"), "Mon 2026-01-05 10:00:00.000000 UTC\n").unwrap();

    run(&world, |e| e.add("openssh-server", None, &picks("openssh-server"))).unwrap();
    // A daemon-reload since, which forgets systemd's own record of the reload, then the update.
    run(&world, |e| e.add("openssh-server", None, &picks("openssh-server"))).unwrap();

    assert_eq!(reloads(&world), 1, "{:#?}", systemctl_calls(&world));
    assert_eq!(root_step(&world, "openssh-server", 3), Some(State::Done));
}

#[test]
fn an_sshd_that_started_after_the_drop_in_was_written_needs_no_reload() {
    let world = system_world("openssh-server");
    ssh_server_there(&world, unit(true, true), unit(false, true));
    fs::write(told(&world, "ssh-started"), "Mon 2100-01-04 10:00:00.000000 UTC\n").unwrap();

    run(&world, |e| e.add("openssh-server", None, &picks("openssh-server"))).unwrap();

    assert_eq!(reloads(&world), 0, "{:#?}", systemctl_calls(&world));
}

#[test]
fn a_reload_that_failed_on_an_update_is_tried_again_on_the_next() {
    let world = system_world("openssh-server");
    ssh_server_there(&world, unit(true, true), unit(false, true));
    fs::write(told(&world, "ssh-started"), "Mon 2026-01-05 10:00:00.000000 UTC\n").unwrap();
    run(&world, |e| e.add("openssh-server", None, &picks("openssh-server"))).unwrap();
    // A release rewrites the drop-in after sshd read it, and sshd -t refuses the result.
    let drop_in = fs::File::options()
        .append(true)
        .open(world.system_path(DROP_IN))
        .unwrap();
    let later = fs::metadata(mark(&world)).unwrap().modified().unwrap() + std::time::Duration::from_millis(1);
    drop_in.set_modified(later).unwrap();
    fs::write(told(&world, "reload-fails"), "").unwrap();

    run(&world, |e| e.add("openssh-server", None, &picks("openssh-server"))).unwrap_err();

    assert_eq!(reloads(&world), 2);
    fs::remove_file(told(&world, "reload-fails")).unwrap();

    run(&world, |e| e.add("openssh-server", None, &picks("openssh-server"))).unwrap();

    assert_eq!(reloads(&world), 3, "{:#?}", systemctl_calls(&world));
    assert!(fs::metadata(mark(&world)).unwrap().modified().unwrap() > later);
}

#[test]
fn sshd_counts_as_started_by_something_by_the_states_of_systemctl_is_enabled_that_say_so() {
    let tmp = crate::sandbox::tempdir().unwrap();
    let home = tmp.path().join("home");
    fs::create_dir_all(&home).unwrap();
    let sandbox = crate::sandbox::Sandbox::new(&tmp.path().join("sandbox"), &home, "tester");
    let listening = shipped("openssh-server").join("listening");
    for (state, started) in [
        ("enabled", true),
        ("enabled-runtime", true),
        ("alias", true),
        ("indirect", true),
        ("generated", true),
        ("static", true),
        ("disabled", false),
        ("masked", false),
        ("masked-runtime", false),
        ("linked", false),
        ("transient", false),
        ("not-found", false),
    ] {
        for unit in ["ssh.socket", "ssh.service"] {
            sandbox.stub(
                "systemctl",
                &format!(r#"if [ "$2" = {unit} ]; then echo {state}; else echo disabled; fi; [ {state} = enabled ]"#),
            );

            let ran = sandbox.command(&listening).output().unwrap();

            assert_eq!(ran.status.success(), started, "{unit} {state}: {ran:?}");
        }
    }
}

#[test]
fn an_ssh_service_enabled_the_classic_way_is_found_and_no_socket_starts_against_it() {
    let world = system_world("openssh-server");
    ssh_server_there(&world, unit(false, false), unit(true, true));

    run(&world, |e| e.add("openssh-server", None, &picks("openssh-server"))).unwrap();

    assert_eq!(root_step(&world, "openssh-server", 5), Some(State::Found));
    let calls = systemctl_calls(&world);
    assert!(!calls.iter().any(|c| c.contains("ssh.socket")), "{calls:#?}");

    run(&world, |e| e.remove_with("openssh-server", &Picks::default())).unwrap();

    let calls = systemctl_calls(&world);
    assert!(!calls.iter().any(|c| c.contains("ssh.socket")), "{calls:#?}");
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
