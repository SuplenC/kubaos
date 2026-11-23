//! The communication apps of the shipped catalog, installed and removed on the fake machine.

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
use crate::apps::host::Host;
use crate::apps::journal::Journal;
use crate::apps::journal::State;
use crate::apps::picks::Picks;
use crate::unfree::Unfree;

const APPS: [&str; 12] = [
    "discord",
    "element",
    "mattermost",
    "microsoft-teams",
    "proton-mail-bridge",
    "signal",
    "slack",
    "telegram",
    "thunderbird",
    "whatsapp",
    "zoom",
    "zulip",
];

/// Chrome as the web apps require it, without the theme helper the shipped one brings.
const CHROME: &str = r#"name = "Google Chrome"
category = "browsers"
tags = ["browser"]
summary = "A browser for the tests"
description = "A browser the tests install."
description_from = "https://example.com/chrome"
icon = "fixture"

[[action]]
name = "Install Google Chrome"
kind = "apt"
arch = "amd64"
packages = ["google-chrome-stable"]
"#;

fn shipped(id: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(CATALOG)
        .join(id)
}

/// A machine whose catalog is the shipped app `id`, every file of its folder copied with its mode
/// as git keeps it, and the Chrome stand-in.
fn world(id: &str) -> World {
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

fn picks(id: &str) -> Picks {
    let mut picks = Picks::default();
    picks.add(id);
    picks
}

/// `kubaos app add <id>`, then `kubaos app remove <id>`, as the engine runs them.
fn add(world: &World, id: &str) -> Result<Vec<Ran>, EngineError> {
    run(world, |e| e.add(id, None, &picks(id)))
}

fn remove(world: &World, id: &str) -> Result<Vec<Ran>, EngineError> {
    run(world, |e| e.remove_with(id, &Picks::default()))
}

fn run(
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

/// Proton's .deb as the fake machine downloads it, at the pin the folder has.
fn serve_the_deb(world: &World, id: &str) {
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

#[test]
fn every_communication_app_installs_on_the_fake_machine_and_its_removal_leaves_it_as_before() {
    for id in APPS {
        let world = world(id);
        serve_the_deb(&world, id);
        let before = world.host.snapshot();

        let ran = add(&world, id).unwrap_or_else(|e| panic!("{id}: {e}"));

        let catalog = Catalog::load(&world.catalog).unwrap();
        let app = catalog.get(id).unwrap();
        let names: Vec<&str> = ran.iter().map(|r| r.name.as_str()).collect();
        let own = app.name.as_str();
        if app.requires(None).next().is_some() {
            assert_eq!(names, ["Google Chrome", own], "{id}");
        } else {
            assert_eq!(names, [own], "{id}");
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
                    Kind::Deb { package, version, .. } => assert_eq!(&state.dpkg[package], version, "{id}"),
                    Kind::Nix { packages, .. } => assert!(packages.iter().all(|p| state.nix.contains(p)), "{id}"),
                    Kind::WebApp { id: web, .. } => {
                        let entry = world
                            .host_home()
                            .join(format!(".local/share/applications/kubaos-web-{web}.desktop"));
                        assert!(entry.is_file(), "{id}");
                        assert!(state.dpkg.contains_key("google-chrome-stable"), "{id}");
                    }
                    _ => {}
                }
            }
        }

        let gone = remove(&world, id).unwrap_or_else(|e| panic!("{id}: {e}"));

        assert_eq!(gone.len(), ran.len(), "{id}: what it brought goes with it");
        assert_eq!(world.host.snapshot(), before, "{id}");
        assert_eq!(world.journals(id), (None, None), "{id}");
    }
}

#[test]
fn slacks_daily_job_is_told_before_its_package_lands_to_leave_the_apt_sources_alone() {
    let world = world("slack");
    let catalog = Catalog::load(&world.catalog).unwrap();
    let kinds: Vec<&str> = catalog
        .get("slack")
        .unwrap()
        .actions
        .iter()
        .map(|a| match &a.kind {
            Kind::File { to, .. } => to.as_str(),
            Kind::Apt { .. } => "apt",
            _ => "other",
        })
        .collect();
    assert_eq!(kinds, ["/etc/default/slack", "apt"]);

    add(&world, "slack").unwrap();

    let defaults = fs::read_to_string(world.host.place(Path::new("/etc/default/slack"))).unwrap();
    let values: Vec<&str> = defaults.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(
        values,
        ["repo_add_once=\"false\"", "repo_reenable_on_distupgrade=\"false\""]
    );
}

#[test]
fn zulips_repository_is_the_sources_file_its_package_ships_so_apt_sees_it_once() {
    let catalog = Catalog::load(&shipped("zulip").join("..")).unwrap();
    let repo = catalog
        .get("zulip")
        .unwrap()
        .actions
        .iter()
        .find_map(|a| match &a.kind {
            Kind::Apt { repo, .. } => repo.clone(),
            _ => None,
        });

    assert_eq!(repo.as_deref(), Some("zulip-desktop"));
}

#[test]
fn the_apps_built_for_amd64_alone_are_unavailable_on_arm64() {
    let catalog = Catalog::load(&shipped("signal").join("..")).unwrap();
    let on_arm: Vec<&str> = APPS
        .into_iter()
        .filter(|id| {
            let app = catalog.get(id).unwrap();
            assert!(
                catalog.available(app, None, Platform::new("amd64", Unfree::Allowed)),
                "{id}"
            );
            catalog.available(app, None, Platform::new("arm64", Unfree::Allowed))
        })
        .collect();

    // Signal, Slack and Zulip build for amd64 alone, Proton builds no arm64 .deb, nixpkgs builds
    // Discord and Zoom for amd64 alone, and the web apps need Chrome, which Google builds for
    // amd64 alone.
    assert_eq!(on_arm, ["element", "mattermost", "telegram", "thunderbird"]);
}

#[test]
fn discord_and_zoom_are_unavailable_while_the_users_nixpkgs_refuses_unfree() {
    let catalog = Catalog::load(&shipped("discord").join("..")).unwrap();
    let unfree: Vec<&str> = APPS
        .into_iter()
        .filter(|id| !catalog.available(catalog.get(id).unwrap(), None, Platform::new("amd64", Unfree::Refused)))
        .collect();

    assert_eq!(unfree, ["discord", "zoom"]);
    for id in unfree {
        let world = world(id);
        world.host.state.borrow_mut().unfree = Unfree::Refused;
        let before = world.host.snapshot();

        assert!(add(&world, id).is_err(), "{id}");

        assert!(world.host.state.borrow().nix.is_empty(), "{id}");
        assert_eq!(world.host.snapshot(), before, "{id}");
    }
}

/// `/etc/default/slack` as the daily job of a Slack installed by hand leaves it: it writes both
/// keys `"true"` with dash's `echo`, then turns `repo_add_once` off once it has added the
/// repository.
const SLACKS_OWN_DEFAULTS: &str = "repo_add_once=\"false\"\nrepo_reenable_on_distupgrade=\"true\"\n\n";

#[test]
fn a_slack_installed_by_hand_has_its_repository_set_up_its_jobs_defaults_taken_over_and_left_on_removal() {
    let world = world("slack");
    world
        .host
        .state
        .borrow_mut()
        .dpkg
        .insert("slack-desktop".to_owned(), "4.53.287".to_owned());
    let defaults = world.host.place(Path::new("/etc/default/slack"));
    fs::create_dir_all(defaults.parent().unwrap()).unwrap();
    fs::write(&defaults, SLACKS_OWN_DEFAULTS).unwrap();

    add(&world, "slack").unwrap();

    let ours = fs::read_to_string(shipped("slack").join("slack.default")).unwrap();
    assert_eq!(fs::read_to_string(&defaults).unwrap(), ours);
    assert!(
        world.host.state.borrow().repos.contains("slack"),
        "taking it in sets its repository up"
    );
    // A run root takes part in, as one after Slack's job wrote its defaults again, checks every
    // step.
    fs::write(&defaults, SLACKS_OWN_DEFAULTS).unwrap();
    add(&world, "slack").unwrap();
    let apt = Journal::read(&world.root_journals, "slack")
        .unwrap()
        .unwrap()
        .entries
        .into_iter()
        .find(|e| matches!(e.action.kind, Kind::Apt { .. }))
        .unwrap();
    assert_eq!(apt.state, State::Done, "KubaOS set its repository up, on every run");
    assert!(
        apt.claims.iter().all(|c| !c.owned),
        "the package was the user's: {apt:?}"
    );

    remove(&world, "slack").unwrap();

    assert_eq!(
        fs::read_to_string(&defaults).unwrap(),
        ours,
        "the job's file stays, turned off"
    );
    assert!(
        world.host.state.borrow().dpkg.contains_key("slack-desktop"),
        "Slack was the user's"
    );
}

#[test]
fn a_slack_defaults_file_that_sets_anything_else_or_runs_anything_is_the_users_and_refused() {
    let theirs = [
        format!("{SLACKS_OWN_DEFAULTS}http_proxy=\"http://proxy:3128\"\n"),
        "repo_add_once=\"$(touch /tmp/x)\"\n".to_owned(),
        "repo_add_once=`touch /tmp/x`\n".to_owned(),
        "repo_add_once=false; touch /tmp/x\n".to_owned(),
        "repo_add_once=$PROXY\n".to_owned(),
        "repo_add_once=false touch\n".to_owned(),
        "repo_add_once=\"false\"x\"\n".to_owned(),
    ];
    for theirs in theirs {
        let world = world("slack");
        let defaults = world.host.place(Path::new("/etc/default/slack"));
        fs::create_dir_all(defaults.parent().unwrap()).unwrap();
        fs::write(&defaults, &theirs).unwrap();

        let err = add(&world, "slack").unwrap_err();

        assert!(err.to_string().contains("Slack"), "{theirs}: {err}");
        assert_eq!(fs::read_to_string(&defaults).unwrap(), theirs);
        assert!(
            !world.host.state.borrow().dpkg.contains_key("slack-desktop"),
            "{theirs}"
        );
    }
}

#[test]
fn a_slack_defaults_file_of_plain_words_quoted_or_bare_is_the_vendors() {
    for theirs in [
        "repo_add_once=false\nrepo_reenable_on_distupgrade='true'\n",
        "# Slack's\nrepo_add_once=\"false\"\n\n",
    ] {
        let world = world("slack");
        let defaults = world.host.place(Path::new("/etc/default/slack"));
        fs::create_dir_all(defaults.parent().unwrap()).unwrap();
        fs::write(&defaults, theirs).unwrap();

        add(&world, "slack").unwrap();

        let ours = fs::read_to_string(shipped("slack").join("slack.default")).unwrap();
        assert_eq!(fs::read_to_string(&defaults).unwrap(), ours, "{theirs}");
    }
}

/// A `zoomus.conf` as Zoom writes it through Qt's `QSettings`: `[General]` first, its keys
/// sorted, `key=value` with no spaces, Qt's quoting and percent-escaped keys, and its other
/// sections after a blank line.
const ZOOMS_OWN_CONF: &str = "[General]\nGeoLocale=system\nQFmw49%2By6zXS%23=1616522911\nautoScale=true\n\
                              conf.webserver=https://us04web.zoom.us\nfake.version=\nxwayland=true\n\n\
                              [zoom_new_im]\nmain_frame_pixel_pos_wide=\"1860,990\"\n";

fn zoom_conf(world: &World) -> PathBuf {
    world.host_home().join(".config/zoomus.conf")
}

#[test]
fn zoom_starts_on_wayland_from_a_file_it_makes_and_its_removal_takes_the_file_away() {
    let world = world("zoom");
    let before = world.host.snapshot();

    add(&world, "zoom").unwrap();

    assert_eq!(
        fs::read_to_string(zoom_conf(&world)).unwrap(),
        "[General]\nxwayland=false\n"
    );
    remove(&world, "zoom").unwrap();

    assert!(!zoom_conf(&world).exists());
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn zooms_own_settings_keep_every_other_line_and_get_their_text_back_whole() {
    let world = world("zoom");
    fs::create_dir_all(zoom_conf(&world).parent().unwrap()).unwrap();
    fs::write(zoom_conf(&world), ZOOMS_OWN_CONF).unwrap();
    let before = world.host.snapshot();

    add(&world, "zoom").unwrap();

    assert_eq!(
        fs::read_to_string(zoom_conf(&world)).unwrap(),
        ZOOMS_OWN_CONF.replace("xwayland=true", "xwayland=false")
    );
    remove(&world, "zoom").unwrap();

    assert_eq!(fs::read_to_string(zoom_conf(&world)).unwrap(), ZOOMS_OWN_CONF);
    assert_eq!(world.host.snapshot(), before);
}

#[test]
fn zoom_rewriting_its_file_keeps_the_step_done_and_its_removal_puts_back_only_the_key() {
    let world = world("zoom");
    fs::create_dir_all(zoom_conf(&world).parent().unwrap()).unwrap();
    fs::write(zoom_conf(&world), ZOOMS_OWN_CONF).unwrap();
    add(&world, "zoom").unwrap();
    // Zoom, run once, writes the whole file again: its keys sorted, one more of its own.
    let rewritten = "[General]\nGeoLocale=system\nQFmw49%2By6zXS%23=1616522911\nautoScale=true\n\
                     conf.webserver=https://us04web.zoom.us\nenableMiniWindow=true\nfake.version=\n\
                     xwayland=false\n\n[zoom_new_im]\nmain_frame_pixel_pos_wide=\"1860,990\"\n";
    fs::write(zoom_conf(&world), rewritten).unwrap();

    add(&world, "zoom").unwrap();
    assert_eq!(
        fs::read_to_string(zoom_conf(&world)).unwrap(),
        rewritten,
        "nothing to do again"
    );
    remove(&world, "zoom").unwrap();

    assert_eq!(
        fs::read_to_string(zoom_conf(&world)).unwrap(),
        rewritten.replace("xwayland=false", "xwayland=true")
    );
}

#[test]
fn zooms_removal_after_a_rewrite_puts_back_an_old_value_qt_quoted() {
    let world = world("zoom");
    fs::create_dir_all(zoom_conf(&world).parent().unwrap()).unwrap();
    // A value Qt quoted, as the step would never write one.
    let mine = "[General]\nautoScale=true\nxwayland=\"0, off\"\n";
    fs::write(zoom_conf(&world), mine).unwrap();
    add(&world, "zoom").unwrap();
    let rewritten = "[General]\nautoScale=true\nenableMiniWindow=true\nxwayland=false\n";
    fs::write(zoom_conf(&world), rewritten).unwrap();

    remove(&world, "zoom").unwrap();

    assert_eq!(
        fs::read_to_string(zoom_conf(&world)).unwrap(),
        "[General]\nautoScale=true\nenableMiniWindow=true\nxwayland=\"0, off\"\n"
    );
    assert_eq!(world.journals("zoom"), (None, None));
}

#[test]
fn a_zoom_setting_you_changed_after_the_install_stays_as_you_set_it() {
    let world = world("zoom");
    add(&world, "zoom").unwrap();
    // Back on XWayland, by hand.
    fs::write(zoom_conf(&world), "[General]\nautoScale=true\nxwayland=true\n").unwrap();

    remove(&world, "zoom").unwrap();

    assert_eq!(
        fs::read_to_string(zoom_conf(&world)).unwrap(),
        "[General]\nautoScale=true\nxwayland=true\n"
    );
}
