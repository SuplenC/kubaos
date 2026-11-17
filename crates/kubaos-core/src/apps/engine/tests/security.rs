//! The security and VPN apps of the shipped catalog, installed and removed on the fake machine.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use super::World;
use super::ai::run;
use super::ai::shipped;
use super::ai::world;
use crate::apps::Catalog;
use crate::apps::Platform;
use crate::apps::action::Kind;
use crate::apps::host::fake::Unit;
use crate::apps::picks::Picks;
use crate::unfree::Unfree;

const APPS: [&str; 9] = [
    "1password",
    "cryptomator",
    "ente-auth",
    "keepassxc",
    "mullvad-vpn",
    "nordvpn",
    "proton-pass",
    "proton-vpn",
    "yubico-authenticator",
];

/// The system units of each VPN's package that its steps keep as its postinst sets them.
const UNITS: [(&str, &[&str]); 3] = [
    ("mullvad-vpn", &["mullvad-daemon.service"]),
    ("proton-vpn", &["me.proton.vpn.split_tunneling.service"]),
    (
        "nordvpn",
        &["nordvpnd.socket", "nordvpnd.service", "nordvpnd-killswitch.service"],
    ),
];

/// The groups of 1Password's helpers, which its first step makes.
const ONEPASSWORD_GROUPS: [&str; 3] = ["onepassword", "onepassword-mcp", "onepassword-cli"];

fn picks(id: &str) -> Picks {
    let mut picks = Picks::default();
    picks.add(id);
    picks
}

/// Where the fake group database keeps a group a line: outside the machine's snapshot, as
/// `/etc/group` is no file of an app's.
fn group_file(world: &World) -> PathBuf {
    world.host_home().join("../../group")
}

/// The groups the fake group database has, sorted.
fn groups(world: &World) -> Vec<String> {
    let mut groups: Vec<String> = fs::read_to_string(group_file(world))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    groups.sort();
    groups
}

/// The shipped app `id`'s world: its .debs served; `getent`, `groupadd` and `groupdel` keep their
/// groups in a file of the test's, so a script never reads the machine's `/etc/group`; each VPN's
/// units are there as its package ships them, neither enabled nor running, as the fake's apt
/// installs no files; and 1Password's and Proton VPN's cleanups work in the fake machine's `/`
/// rather than the real one.
fn security_world(id: &str) -> World {
    let world = world(id);
    let file = group_file(&world);
    fs::write(&file, "").unwrap();
    let f = file.display();
    let sandbox = world.host.sandbox();
    sandbox.stub("getent", &format!("grep -qx \"$2\" '{f}' && echo \"$2:x:999:\""));
    sandbox.stub("groupadd", &format!("for g; do :; done; echo \"$g\" >>'{f}'"));
    sandbox.stub(
        "groupdel",
        &format!("grep -vx \"$1\" '{f}' >'{f}.new' || true; mv '{f}.new' '{f}'"),
    );
    for (app, units) in UNITS {
        if app != id {
            continue;
        }
        let mut state = world.host.state.borrow_mut();
        for unit in units {
            state.units.insert(
                format!("system:{unit}"),
                Unit {
                    exists: true,
                    ..Unit::default()
                },
            );
        }
    }
    serve_the_debs(&world, id);
    let script = match id {
        "1password" => Some("./ungroup"),
        "proton-vpn" => Some("./kill-switch-off"),
        _ => None,
    };
    if let Some(script) = script {
        let text = fs::read_to_string(world.catalog.join(id).join("app.toml")).unwrap();
        let root = world.system_path("/").display().to_string();
        let ours = text.replace(&format!(r#"["{script}", "/"]"#), &format!(r#"["{script}", "{root}"]"#));
        assert_ne!(ours, text, "{id}'s cleanup names the machine's /");
        world.write_app(id, &ours);
    }
    world
}

/// The app `id`'s .debs as the fake machine downloads them, at the pins the folder has.
fn serve_the_debs(world: &World, id: &str) {
    let catalog = Catalog::load(&world.catalog).unwrap();
    for action in &catalog.get(id).unwrap().actions {
        if let Kind::Deb { package, version, .. } = &action.kind {
            let pin = action.download("amd64").unwrap();
            world.host.state.borrow_mut().downloads.insert(
                pin.url.clone(),
                (pin.sha256.clone(), format!("deb {package} {version}").into_bytes()),
            );
        }
    }
}

/// Each of the installed `id`'s steps there on the fake machine: its packages, units, group and
/// icon.
fn in_place(world: &World, id: &str) {
    let catalog = Catalog::load(&world.catalog).unwrap();
    let app = catalog.get(id).unwrap();
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
            Kind::Deb { package, version, .. } => assert_eq!(&state.dpkg[package], version, "{id}"),
            Kind::Unit {
                unit, enable, start, ..
            } => {
                let at = &state.units[&format!("system:{unit}")];
                assert_eq!(at.enabled, enable.unwrap_or(false), "{id}: {unit}");
                assert_eq!(at.active, start.unwrap_or(false), "{id}: {unit}");
            }
            Kind::Group { group } => assert!(state.groups[group].contains("kuba"), "{id}: {group}"),
            // The icon of the folder, put in the theme.
            Kind::File { to, .. } => {
                let path = world.host_home().join(to.strip_prefix("~/").unwrap());
                assert!(path.is_file(), "{id}: {}", path.display());
            }
            Kind::Run { .. } if id == "1password" || id == "proton-vpn" => {}
            other => panic!("{id}: a security app is its package, .deb, units, group or icon, not {other:?}"),
        }
    }
}

#[test]
fn every_security_app_installs_on_the_fake_machine_and_its_removal_leaves_it_as_before() {
    for id in APPS {
        let world = security_world(id);
        let before = world.host.snapshot();

        let ran = run(&world, |e| e.add(id, None, &picks(id))).unwrap_or_else(|e| panic!("{id}: {e}"));

        assert_eq!(ran.len(), 1, "{id}: it requires nothing");
        in_place(&world, id);

        run(&world, |e| e.remove_with(id, &Picks::default())).unwrap_or_else(|e| panic!("{id}: {e}"));

        assert_eq!(world.host.snapshot(), before, "{id}");
        assert_eq!(world.journals(id), (None, None), "{id}");
        assert!(groups(&world).is_empty(), "{id}: {:?}", groups(&world));
    }
}

#[test]
fn which_security_apps_a_machine_can_have_follows_its_architecture() {
    let catalog = Catalog::load(&shipped("keepassxc").join("..")).unwrap();
    let on = |arch: &str, unfree: Unfree| -> Vec<&str> {
        APPS.into_iter()
            .filter(|id| catalog.available(catalog.get(id).unwrap(), None, Platform::new(arch, unfree)))
            .collect()
    };

    assert_eq!(on("amd64", Unfree::Allowed), APPS);
    // None of them is unfree in nixpkgs: 1Password comes from its own repository.
    assert_eq!(on("amd64", Unfree::Refused), APPS);
    // 1Password builds its app for amd64 alone, and Proton its Pass's .deb.
    assert_eq!(
        on("arm64", Unfree::Allowed),
        [
            "cryptomator",
            "ente-auth",
            "keepassxc",
            "mullvad-vpn",
            "nordvpn",
            "proton-vpn",
            "yubico-authenticator",
        ]
    );
}

#[test]
fn onepassword_makes_its_helpers_groups_before_its_packages_and_puts_nobody_in_them() {
    let world = security_world("1password");

    run(&world, |e| e.add("1password", None, &picks("1password"))).unwrap();

    assert_eq!(groups(&world), ["onepassword", "onepassword-cli", "onepassword-mcp"]);
    let calls = world.host.sandbox().calls();
    for group in ONEPASSWORD_GROUPS {
        assert!(calls.contains(&format!("groupadd --system {group}")), "{calls:#?}");
    }
    assert!(!calls.iter().any(|c| c.starts_with("usermod")), "{calls:#?}");
    let state = world.host.state.borrow();
    let at = |what: &str| state.log.iter().position(|l| l.starts_with(what)).unwrap();
    assert!(at("run ./groups") < at("apt install"), "{:#?}", state.log);
}

#[test]
fn onepasswords_removal_takes_what_its_setup_leaves_and_keeps_an_allowlist_you_edited() {
    let world = security_world("1password");
    run(&world, |e| e.add("1password", None, &picks("1password"))).unwrap();
    // What 1Password's postinst writes and its own removal leaves.
    let left = [
        "/etc/debsig/policies/AC2D62742012EA22/1password.pol",
        "/usr/share/debsig/keyrings/AC2D62742012EA22/debsig.gpg",
        "/opt/1Password/com.1password.1Password.policy",
    ];
    for path in left {
        let path = world.system_path(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "1Password's\n").unwrap();
    }
    let allowlist = world.system_path("/etc/1password/custom_allowed_browsers");
    fs::create_dir_all(allowlist.parent().unwrap()).unwrap();
    fs::write(&allowlist, "librewolf\n").unwrap();

    run(&world, |e| e.remove_with("1password", &Picks::default())).unwrap();

    for path in ["/etc/debsig", "/usr/share/debsig", "/opt/1Password"] {
        assert!(!world.system_path(path).exists(), "{path}");
    }
    assert_eq!(
        fs::read_to_string(&allowlist).unwrap(),
        "librewolf\n",
        "yours, so it stays"
    );
    assert!(groups(&world).is_empty(), "{:?}", groups(&world));
}

#[test]
fn a_1password_cli_still_installed_keeps_its_group_when_1password_goes() {
    let world = security_world("1password");
    run(&world, |e| e.add("1password", None, &picks("1password"))).unwrap();
    // The CLI stayed, as one the user installed by name would.
    world
        .host
        .sandbox()
        .stub("dpkg-query", "[ \"$4\" = 1password-cli ] && printf 'ii '; exit 0");

    run(&world, |e| e.remove_with("1password", &Picks::default())).unwrap();

    assert_eq!(groups(&world), ["onepassword-cli"]);
}

#[test]
fn nordvpn_puts_you_in_its_group_after_its_preinst_and_leaves_the_group_to_its_package() {
    let world = security_world("nordvpn");
    // NordVPN's preinst on an install, which puts in the group the user it guesses, not you.
    let guessed = || ("nordvpn".to_owned(), "someone".to_owned());
    world
        .host
        .state
        .borrow_mut()
        .preinst_groups
        .insert("nordvpn".to_owned(), guessed());
    let members = |world: &World| world.host.state.borrow().groups.get("nordvpn").cloned();

    run(&world, |e| e.add("nordvpn", None, &picks("nordvpn"))).unwrap();

    assert_eq!(
        members(&world),
        Some(BTreeSet::from(["kuba".to_owned(), "someone".to_owned()])),
        "{:#?}",
        world.host.state.borrow().log
    );

    run(&world, |e| e.remove_with("nordvpn", &Picks::default())).unwrap();

    assert_eq!(members(&world), Some(BTreeSet::from([guessed().1])));
}

#[test]
fn proton_vpns_removal_turns_its_kill_switch_off_before_its_packages_go() {
    let world = security_world("proton-vpn");
    // The machine's nftables, a table a line: Proton's firewall kill switch's two and another
    // program's.
    let tables = world.host_home().join("../../nftables");
    fs::write(&tables, "inet proton_vpn_ks\nip6 proton_vpn_ks_ipv6\ninet filter\n").unwrap();
    let f = tables.display();
    let sandbox = world.host.sandbox();
    sandbox.stub(
        "nft",
        &format!(
            "case $1 in\n\
             list) grep -qx \"$3 $4\" '{f}' ;;\n\
             delete) grep -vx \"$3 $4\" '{f}' >'{f}.new'; mv '{f}.new' '{f}' ;;\n\
             esac"
        ),
    );
    // Proton's own command, which fails as when its service couldn't be reached.
    sandbox.stub("python3", "exit 1");
    run(&world, |e| e.add("proton-vpn", None, &picks("proton-vpn"))).unwrap();
    let from = world.host.state.borrow().log.len();

    run(&world, |e| e.remove_with("proton-vpn", &Picks::default())).unwrap();

    assert_eq!(fs::read_to_string(&tables).unwrap(), "inet filter\n");
    assert!(
        sandbox
            .calls()
            .contains(&"python3 -I -m proton.vpn.killswitch.disable".to_owned()),
        "{:#?}",
        sandbox.calls()
    );
    let log = world.log_since(from);
    let at = |what: &str| log.iter().position(|l| l.starts_with(what));
    assert!(
        at("run ./kill-switch-off").is_some_and(|off| at("apt remove").is_some_and(|gone| off < gone)),
        "{log:#?}"
    );
}

/// Proton VPN installed, with Proton's own command failing as when what it reaches is down, and
/// no nftables table up.
fn proton_vpn() -> World {
    let world = security_world("proton-vpn");
    let sandbox = world.host.sandbox();
    sandbox.stub("python3", "exit 1");
    sandbox.stub("nft", "exit 1");
    run(&world, |e| e.add("proton-vpn", None, &picks("proton-vpn"))).unwrap();
    world
}

/// A running NetworkManager whose connections are `names`, one a line, as the stand-in `nmcli`:
/// it lists each as `<uuid>:<name>`, `u<n>` its UUID, from a file of the test's; with `deletes`,
/// deleting one by its UUID takes its line away. The file.
fn network_manager(world: &World, names: &str, deletes: bool) -> PathBuf {
    let file = world.host_home().join("../../connections");
    let mut listed = String::new();
    for (n, name) in names.lines().enumerate() {
        writeln!(listed, "u{n}:{name}").unwrap();
    }
    fs::write(&file, listed).unwrap();
    let f = file.display();
    let delete = if deletes {
        format!("[ \"$3\" = uuid ] && grep -v \"^$4:\" '{f}' >'{f}.new'; mv '{f}.new' '{f}'")
    } else {
        ":".to_owned()
    };
    world.host.sandbox().stub(
        "nmcli",
        &format!("case $1 in\n-t) cat '{f}' ;;\nconnection) {delete} ;;\nesac"),
    );
    file
}

#[test]
fn proton_vpns_removal_deletes_its_kill_switchs_connections_networkmanager_still_has() {
    let world = proton_vpn();
    let connections = network_manager(
        &world,
        "pvpn-killswitch-perm\npvpn-routed-killswitch\nHome Wi-Fi\n",
        true,
    );

    run(&world, |e| e.remove_with("proton-vpn", &Picks::default())).unwrap();

    assert_eq!(fs::read_to_string(&connections).unwrap(), "u2:Home Wi-Fi\n");
}

#[test]
fn proton_vpns_removal_deletes_its_saved_kill_switch_with_networkmanager_down() {
    let world = proton_vpn();
    // NetworkManager isn't running, as nmcli says.
    world.host.sandbox().stub("nmcli", "exit 8");
    let saved = world.system_path("/etc/NetworkManager/system-connections");
    fs::create_dir_all(&saved).unwrap();
    let ours = saved.join("pvpn-killswitch-perm.nmconnection");
    fs::write(&ours, "[connection]\nid=pvpn-killswitch-perm\ntype=dummy\n").unwrap();
    let theirs = saved.join("home.nmconnection");
    fs::write(&theirs, "[connection]\nid=Home Wi-Fi\ntype=wifi\n").unwrap();

    run(&world, |e| e.remove_with("proton-vpn", &Picks::default())).unwrap();

    assert!(!ours.exists());
    assert!(theirs.is_file());
}

#[test]
fn a_proton_kill_switch_connection_that_stays_stops_the_removal_with_the_packages_kept() {
    let world = proton_vpn();
    // NetworkManager won't let it go.
    network_manager(&world, "pvpn-killswitch-perm\n", false);

    let removed = run(&world, |e| e.remove_with("proton-vpn", &Picks::default()));

    assert!(removed.is_err(), "the removal went through");
    assert!(world.host.state.borrow().dpkg.contains_key("proton-vpn-gnome-desktop"));
}

#[test]
fn a_failing_nmcli_stops_proton_vpns_removal_once_it_answered() {
    let world = proton_vpn();
    // It lists its connections once, then fails.
    let asked = world.host_home().join("../../asked");
    let a = asked.display();
    world.host.sandbox().stub(
        "nmcli",
        &format!("case $1 in\n-t) [ -e '{a}' ] && exit 10; touch '{a}'; echo 'u0:Home Wi-Fi' ;;\nesac"),
    );

    let removed = run(&world, |e| e.remove_with("proton-vpn", &Picks::default()));

    assert!(removed.is_err(), "the removal went through");
    assert!(world.host.state.borrow().dpkg.contains_key("proton-vpn-gnome-desktop"));
}

#[test]
fn mullvads_early_boot_blocker_you_turned_off_stays_off_through_an_update() {
    let world = security_world("mullvad-vpn");
    run(&world, |e| e.add("mullvad-vpn", None, &picks("mullvad-vpn"))).unwrap();
    // Mullvad's blocker, turned off by hand after the install.
    let blocker = "system:mullvad-early-boot-blocking.service".to_owned();
    world.host.state.borrow_mut().units.insert(
        blocker.clone(),
        Unit {
            exists: true,
            ..Unit::default()
        },
    );

    run(&world, |e| e.add("mullvad-vpn", None, &picks("mullvad-vpn"))).unwrap();

    assert!(
        !world.host.state.borrow().units[&blocker].enabled,
        "{:#?}",
        world.host.state.borrow().log
    );
}

#[test]
fn a_vpns_units_are_stopped_and_turned_off_before_its_package_goes() {
    for (id, units) in UNITS {
        let world = security_world(id);
        run(&world, |e| e.add(id, None, &picks(id))).unwrap();
        let from = world.host.state.borrow().log.len();

        run(&world, |e| e.remove_with(id, &Picks::default())).unwrap();

        let log = world.log_since(from);
        let removed = log.iter().position(|l| l.starts_with("apt remove")).unwrap();
        for unit in units {
            let off = log.iter().position(|l| l == &format!("enable {unit} false"));
            assert!(off.is_some_and(|at| at < removed), "{id}: {unit}: {log:#?}");
            let state = world.host.state.borrow();
            assert!(!state.units[&format!("system:{unit}")].active, "{id}: {unit}");
        }
    }
}
