use std::fmt::Write as _;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use super::Fake;
use super::FakeApps;
use super::World;
use super::notes;
use super::phases;
use super::summary;
use super::warnings;
use super::yes;
use crate::apps::AppState;
use crate::apps::Picks;
use crate::apps::action::Kind;
use crate::apps::host::Host;
use crate::apps::host::fake::FakeHost;
use crate::apps::journal::Adoption;
use crate::apps::journal::Journal;
use crate::apps::journal::State;
use crate::sandbox;
use crate::update::UpdateError;
use crate::update::event::AppChanges;
use crate::update::event::Phase;
use crate::update::event::chain;
use crate::update::record::Channel;

const ABOUT: &str = r#"category = "development"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/app"
"#;

/// An apt app from a vendor's repository.
const AGENT: &str = r#"
name = "Agent"
summary = "A coding agent"

[[action]]
name = "Install Agent"
kind = "apt"
repo = "vendor"
packages = ["agent"]
"#;

/// A pinned `.deb`, and a file of its own as root.
const DESK: &str = r#"
name = "Desk"
summary = "A desktop app"

[[action]]
name = "Install Desk"
kind = "deb"
package = "desk"
version = "1.0"
amd64 = { url = "https://example.com/desk-amd64.deb", sha256 = "418c7825f73107b7b5ba3babae5e3b485801dc22d9dc0d518af632f7906cbdd1" }

[[action]]
name = "Start it at login"
kind = "file"
to = "/etc/xdg/autostart/desk.desktop"
text = "[Desktop Entry]\nExec=desk\n"
"#;

/// An app with setups: its desktop app alone, or a service of the user's beside it, which pairs
/// in a window.
const LAB: &str = r#"
name = "Lab"
summary = "An app for agents"

[[action]]
name = "Install Lab"
kind = "deb"
package = "lab"
version = "2.0"
self_updates = true
amd64 = { url = "https://example.com/lab-amd64.deb", sha256 = "1111111111111111111111111111111111111111111111111111111111111111" }

[setup.desktop]
name = "Lab"
summary = "The desktop app"

[setup.service]
name = "Lab as a service"
summary = "Its agents in a service"

[[setup.service.ready]]
check = ["./closed"]
says = "Lab is open: quit it, then try again"

[[setup.service.action]]
name = "Write the service"
kind = "file"
to = "~/.config/systemd/user/lab.service"
text = "[Service]\nExecStart=lab serve\n"

[[setup.service.action]]
name = "Keep it running"
kind = "linger"

[[setup.service.action]]
name = "Pair Lab"
kind = "run"
do = ["./pair"]
check = ["./paired"]
undo = ["./unpair"]
window = "Open Lab and run `lab pair`"
"#;

/// An app for another architecture than the fake machine's.
const ARM: &str = r#"
name = "Arm"
summary = "Only for arm64"

[[action]]
name = "Install Arm"
kind = "apt"
arch = "arm64"
packages = ["arm-tool"]
"#;

/// The checkout's catalog: agent, desk, lab and arm, with lab's scripts, and the repository setup
/// root runs; all of it the test user's alone, as the root helper wants it.
fn world() -> World {
    let w = World::new(Channel::Edge, true);
    for (id, text) in [("agent", AGENT), ("desk", DESK), ("lab", LAB), ("arm", ARM)] {
        put(&w.sys.catalog().join(id).join("app.toml"), &format!("{ABOUT}{text}"));
    }
    let lab = w.sys.catalog().join("lab");
    script(&lab.join("closed"), r#"[ ! -e "$HOME/lab-open" ]"#);
    script(&lab.join("pair"), r#"touch "$HOME/paired""#);
    script(&lab.join("paired"), r#"[ -e "$HOME/paired" ]"#);
    script(&lab.join("unpair"), r#"rm -f "$HOME/paired""#);
    script(&w.sys.apt_repo_script(), "true");
    for dir in [w.sys.checkout(), w.sys.catalog(), w.sys.checkout().join("system")] {
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
    }
    for id in ["agent", "desk", "lab", "arm"] {
        lock_down(&w.sys.catalog().join(id));
    }
    w
}

/// `dir` and everything in it writable by its owner alone.
fn lock_down(dir: &Path) {
    fs::set_permissions(dir, fs::Permissions::from_mode(0o755)).unwrap();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            lock_down(&path);
        } else {
            let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o100;
            fs::set_permissions(
                &path,
                fs::Permissions::from_mode(0o644 | mode | (mode >> 3) | (mode >> 6)),
            )
            .unwrap();
        }
    }
}

fn script(path: &Path, body: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    sandbox::program(path, &format!("#!/bin/sh\n{body}\n"));
}

fn put(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// A machine with the apt packages `installed`, the downloads of desk and lab, and no picks.
fn fake(w: &World, installed: &[&str]) -> Fake {
    let host = FakeHost::new(&w.tmp.path().join("machine"));
    {
        let mut state = host.state.borrow_mut();
        for p in installed {
            state.dpkg.insert((*p).to_owned(), "1.0".to_owned());
        }
        state.downloads.insert(
            "https://example.com/desk-amd64.deb".to_owned(),
            (
                "418c7825f73107b7b5ba3babae5e3b485801dc22d9dc0d518af632f7906cbdd1".to_owned(),
                b"deb desk 1.0".to_vec(),
            ),
        );
        state.downloads.insert(
            "https://example.com/lab-amd64.deb".to_owned(),
            ("1".repeat(64), b"deb lab 2.0".to_vec()),
        );
    }
    let mut m = Fake::new();
    m.apps = Some(FakeApps {
        host,
        display: true,
        catalog: w.sys.catalog(),
        journals: w.sys.app_journals(),
        owner: fs::metadata(w.sys.catalog()).unwrap().uid(),
        declined: false,
        root_calls: std::rc::Rc::default(),
    });
    m
}

/// The machine `m` acts on.
fn host(m: &Fake) -> FakeHost {
    m.apps.as_ref().unwrap().host.clone()
}

fn has(m: &Fake, package: &str) -> bool {
    host(m).state.borrow().dpkg.contains_key(package)
}

/// How many times a root helper was started.
fn root_starts(m: &Fake) -> usize {
    m.apps
        .as_ref()
        .unwrap()
        .root_calls
        .borrow()
        .iter()
        .filter(|c| *c == "start")
        .count()
}

/// The root helpers made, as their command lines.
fn root_helpers(m: &Fake) -> Vec<&str> {
    m.ran
        .iter()
        .filter_map(|line| line.strip_prefix("root helper: "))
        .collect()
}

fn picked(w: &World) -> Vec<String> {
    let picks = Picks::read(&w.user.app_picks()).unwrap();
    picks.iter().map(str::to_owned).collect()
}

fn names(names: &[&str]) -> Vec<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

fn user_journal(w: &World, id: &str) -> Option<Journal> {
    Journal::read(&w.user.app_journals(), id).unwrap()
}

fn root_journal(w: &World, id: &str) -> Option<Journal> {
    Journal::read(&w.sys.app_journals(), id).unwrap()
}

#[test]
fn add_picks_the_apps_and_installs_them_through_polkit_once() {
    let w = world();
    let mut m = fake(&w, &[]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events)
        .add_apps(&names(&["agent", "desk"]), None)
        .unwrap();

    assert_eq!(picked(&w), ["agent", "desk"]);
    assert!(has(&m, "agent") && has(&m, "desk"));
    assert_eq!(root_starts(&m), 1);
    assert!(root_helpers(&m)[0].starts_with("pkexec ") && root_helpers(&m)[0].ends_with("kubaos-apps serve"));
    assert!(user_journal(&w, "agent").unwrap().complete);
    assert_eq!(
        summary(&events).apps,
        AppChanges {
            installed: names(&["Agent", "Desk"]),
            ..AppChanges::default()
        }
    );
}

#[test]
fn each_step_is_an_event_with_its_name() {
    let w = world();
    let mut m = fake(&w, &[]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events)
        .add_apps(&names(&["desk"]), None)
        .unwrap();

    let steps: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            crate::update::event::UpdateEvent::Step { text } => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert!(steps.contains(&"Install Desk".to_owned()), "{steps:?}");
    assert!(steps.contains(&"Start it at login".to_owned()), "{steps:?}");
}

#[test]
fn adding_an_app_that_is_installed_changes_nothing_and_asks_for_no_password() {
    let w = world();
    let mut m = fake(&w, &[]);
    w.updater(&mut m).add_apps(&names(&["desk"]), None).unwrap();
    let before = host(&m).snapshot();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events)
        .add_apps(&names(&["desk"]), None)
        .unwrap();

    assert_eq!(host(&m).snapshot(), before);
    assert_eq!(root_starts(&m), 1, "only the first add asked for root");
    assert_eq!(summary(&events).apps.installed, names(&["Desk"]));
}

#[test]
fn an_id_the_catalog_lacks_is_refused_before_anything_changes() {
    let w = world();
    let mut m = fake(&w, &[]);

    let err = w
        .updater(&mut m)
        .add_apps(&names(&["agent", "nope"]), None)
        .unwrap_err();

    assert!(matches!(err, UpdateError::NoApp { ref id } if id == "nope"), "{err:?}");
    assert!(picked(&w).is_empty());
    assert!(root_helpers(&m).is_empty());
}

#[test]
fn an_app_with_unfree_nixpkgs_is_refused_while_the_user_refuses_unfree_and_nothing_changes() {
    let w = world();
    let chat = w.sys.catalog().join("chat");
    put(
        &chat.join("app.toml"),
        &format!(
            "{ABOUT}name = \"Chat\"\nsummary = \"Talk\"\n\n[[action]]\nname = \"Install Chat\"\nkind = \"nix\"\n\
             packages = [\"discord\"]\nunfree = true\n"
        ),
    );
    lock_down(&chat);
    put(
        &w.user.state_dir().root().join("nixpkgs.toml"),
        "allow_unfree = false\n",
    );
    let mut m = fake(&w, &[]);
    let before = host(&m).snapshot();

    let err = w.updater(&mut m).add_apps(&names(&["chat"]), None).unwrap_err();
    let status = w.updater(&mut m).app_status().unwrap();

    assert!(matches!(err, UpdateError::Unfree { ref id } if id == "chat"), "{err:?}");
    assert!(err.to_string().contains("kubaos package unfree on"), "{err}");
    assert!(picked(&w).is_empty());
    assert_eq!(host(&m).snapshot(), before);
    assert_eq!(status.get("chat").unwrap().state, AppState::Unavailable);
    put(&w.user.state_dir().root().join("nixpkgs.toml"), "allow_unfree = true\n");
    let status = w.updater(&mut m).app_status().unwrap();
    assert_eq!(status.get("chat").unwrap().state, AppState::Available);
}

#[test]
fn a_declined_password_changes_nothing_and_puts_the_picks_back() {
    let w = world();
    let mut m = fake(&w, &[]);
    m.apps.as_mut().unwrap().declined = true;
    let before = host(&m).snapshot();
    let mut events = Vec::new();

    let err = w
        .reporting(&mut m, &mut events)
        .add_apps(&names(&["desk"]), None)
        .unwrap_err();

    assert!(matches!(err, UpdateError::Declined), "{err:?}");
    assert!(picked(&w).is_empty());
    assert_eq!(host(&m).snapshot(), before);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, crate::update::event::UpdateEvent::Declined))
            .count(),
        1
    );
}

#[test]
fn remove_drops_the_pick_and_undoes_the_journal_through_polkit() {
    let w = world();
    let mut m = fake(&w, &[]);
    let before = host(&m).snapshot();
    w.updater(&mut m).add_apps(&names(&["desk"]), None).unwrap();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).remove_apps(&names(&["desk"])).unwrap();

    assert!(picked(&w).is_empty());
    assert_eq!(host(&m).snapshot(), before);
    assert_eq!(user_journal(&w, "desk"), None);
    assert_eq!(root_journal(&w, "desk"), None);
    assert!(root_helpers(&m).iter().all(|h| h.starts_with("pkexec ")));
    assert_eq!(summary(&events).apps.removed, names(&["Desk"]));
}

#[test]
fn a_removal_rolled_back_keeps_its_app_picked_and_one_removed_before_it_stays_out() {
    let w = world();
    let mut m = fake(&w, &[]);
    w.updater(&mut m).add_apps(&names(&["agent", "desk"]), None).unwrap();
    // apt can't take Desk's package, as with dpkg's lock held.
    host(&m).state.borrow_mut().fail.insert("apt-remove:desk".to_owned());

    let err = w.updater(&mut m).remove_apps(&names(&["agent", "desk"])).unwrap_err();

    let why = crate::update::event::chain(&err);
    assert!(why.contains("the removal was rolled back"), "{why}");
    assert_eq!(picked(&w), names(&["desk"]));
    assert_eq!(user_journal(&w, "agent"), None);
    assert!(has(&m, "desk"));
    let status = w.updater(&mut m).app_status().unwrap();
    let desk = status.get("desk").unwrap();
    assert_eq!(desk.state, AppState::Installed);
    assert!(
        desk.removal_rolled_back
            .as_ref()
            .is_some_and(|m| m.whole && m.why.starts_with("Install Desk: ")),
        "{desk:?}"
    );
}

#[test]
fn an_update_installs_the_picked_apps_through_sudo_and_a_second_starts_no_root() {
    let w = world();
    let mut picks = Picks::default();
    picks.add("agent");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = fake(&w, &[]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(has(&m, "agent"));
    assert!(
        root_helpers(&m).iter().all(|h| h.starts_with("sudo ")),
        "{:?}",
        root_helpers(&m)
    );
    assert_eq!(summary(&events).apps.installed, names(&["Agent"]));
    assert!(phases(&events).contains(&Phase::Apps));

    let starts = root_starts(&m);
    let mut again = Vec::new();
    w.reporting(&mut m, &mut again).update(yes()).unwrap();

    assert_eq!(root_starts(&m), starts, "a second update asked for root again");
    assert!(summary(&again).apps.installed.is_empty());
}

#[test]
fn an_update_fetches_a_key_nearing_its_expiry_of_a_repository_an_app_set_up_again_and_only_then_asks_for_root() {
    let w = world();
    let mut picks = Picks::default();
    picks.add("agent");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = fake(&w, &[]);
    w.reporting(&mut m, &mut Vec::new()).update(yes()).unwrap();
    let installed = root_starts(&m);
    let mut quiet = Vec::new();
    w.reporting(&mut m, &mut quiet).update(yes()).unwrap();
    assert_eq!(root_starts(&m), installed, "an update with no key due asked for root");
    host(&m).state.borrow_mut().keys_due.insert("vendor".to_owned());

    let mut events = Vec::new();
    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert_eq!(root_starts(&m), installed + 1);
    assert!(host(&m).state.borrow().keys_due.is_empty());
    let keys: Vec<&str> = warnings(&events).into_iter().filter(|w| w.contains("key")).collect();
    assert!(keys.is_empty(), "{keys:?}");
    let detail = events.iter().find_map(|e| match e {
        crate::update::event::UpdateEvent::PhaseFinished {
            phase: Phase::Apps,
            detail,
            ..
        } => detail.clone(),
        _ => None,
    });
    assert_eq!(detail.as_deref(), Some("the signing key of vendor fetched again"));
}

#[test]
fn an_update_without_picks_runs_nothing_for_the_apps() {
    let w = world();
    let mut m = fake(&w, &[]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(root_helpers(&m).is_empty());
    assert!(!phases(&events).contains(&Phase::Apps));
}

#[test]
fn a_picked_app_unfree_while_the_user_refuses_unfree_warns_why_and_the_update_goes_on() {
    let w = world();
    let chat = w.sys.catalog().join("chat");
    put(
        &chat.join("app.toml"),
        &format!(
            "{ABOUT}name = \"Chat\"\nsummary = \"Talk\"\n\n[[action]]\nname = \"Install Chat\"\nkind = \"nix\"\n\
             packages = [\"discord\"]\nunfree = true\n"
        ),
    );
    lock_down(&chat);
    put(
        &w.user.state_dir().root().join("nixpkgs.toml"),
        "allow_unfree = false\n",
    );
    let mut picks = Picks::default();
    picks.add("chat");
    picks.add("desk");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = fake(&w, &[]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    let about_chat: Vec<_> = warnings(&events).into_iter().filter(|w| w.contains("chat")).collect();
    assert_eq!(about_chat.len(), 1, "{about_chat:?}");
    assert!(about_chat[0].contains("kubaos package unfree on"), "{about_chat:?}");
    assert!(has(&m, "desk"));
}

#[test]
fn an_app_that_fails_to_install_warns_and_the_update_goes_on() {
    let w = world();
    let mut picks = Picks::default();
    picks.add("agent");
    picks.add("desk");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = fake(&w, &[]);
    host(&m).state.borrow_mut().fail.insert("apt:agent".to_owned());
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(!has(&m, "agent") && has(&m, "desk"));
    assert!(
        warnings(&events).iter().any(|w| w.contains("installing Agent")),
        "{:?}",
        warnings(&events)
    );
    assert_eq!(summary(&events).apps.installed, names(&["Desk"]));
}

#[test]
fn a_picked_app_refused_over_a_file_of_the_users_own_names_the_remedy_at_every_update() {
    let w = world();
    let mut picks = Picks::default();
    picks.add("desk");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = fake(&w, &[]);
    let mine = host(&m).place(Path::new("/etc/xdg/autostart/desk.desktop"));
    put(&mine, "[Desktop Entry]\nExec=my-desk\n");

    for _ in 0..2 {
        let mut events = Vec::new();
        w.reporting(&mut m, &mut events).update(yes()).unwrap();

        let warned = warnings(&events);
        assert!(
            warned.iter().any(|w| w.contains("is yours")
                && w.contains("delete or rename it")
                && w.ends_with(", then run kubaos update")),
            "{warned:?}"
        );
        assert!(!has(&m, "desk"));
    }
    assert_eq!(fs::read_to_string(&mine).unwrap(), "[Desktop Entry]\nExec=my-desk\n");
    assert_eq!(picked(&w), names(&["desk"]));
}

#[test]
fn a_picked_app_refused_over_another_apps_path_says_the_next_update_tries_again() {
    let w = world();
    let rival = w.sys.catalog().join("rival");
    put(
        &rival.join("app.toml"),
        &format!(
            "{ABOUT}name = \"Rival\"\nsummary = \"Desk's rival\"\n\n[[action]]\nname = \"Start it at login\"\n\
             kind = \"file\"\nto = \"/etc/xdg/autostart/desk.desktop\"\ntext = \"[Desktop Entry]\\nExec=rival\\n\"\n"
        ),
    );
    lock_down(&rival);
    let mut picks = Picks::default();
    picks.add("desk");
    picks.add("rival");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = fake(&w, &[]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(has(&m, "desk"));
    let warned = warnings(&events);
    let refused: Vec<&&str> = warned.iter().filter(|w| w.contains("belongs to desk")).collect();
    assert_eq!(refused.len(), 1, "{warned:?}");
    assert!(refused[0].ends_with("; the next update tries again"), "{warned:?}");
    assert!(!refused[0].contains("then run kubaos update"), "{warned:?}");
}

#[test]
fn the_status_reads_the_journals_the_picks_and_dpkg() {
    let w = world();
    let mut m = fake(&w, &[]);
    w.updater(&mut m).add_apps(&names(&["agent"]), None).unwrap();
    let mut picks = Picks::read(&w.user.app_picks()).unwrap();
    picks.add("desk");
    picks.add("gone");
    picks.write(&w.user.app_picks()).unwrap();
    m.dpkg = Some(super::FakeDpkg::with(&["agent"]));

    let status = w.updater(&mut m).app_status().unwrap();

    let state = |id: &str| status.get(id).unwrap().state;
    assert_eq!(state("agent"), AppState::Installed);
    assert_eq!(state("desk"), AppState::Missing);
    assert_eq!(state("lab"), AppState::Available);
    assert_eq!(status.get("agent").unwrap().version.as_deref(), Some("1.0"));
    assert_eq!(status.unknown, ["gone"]);
}

#[test]
fn an_app_with_setups_needs_one_and_a_setup_it_lacks_is_refused() {
    let w = world();
    let mut m = fake(&w, &[]);

    let none = w.updater(&mut m).add_apps(&names(&["lab"]), None).unwrap_err();
    let unknown = w.updater(&mut m).add_apps(&names(&["lab"]), Some("cloud")).unwrap_err();
    let without = w
        .updater(&mut m)
        .add_apps(&names(&["agent"]), Some("desktop"))
        .unwrap_err();

    assert!(matches!(none, UpdateError::NoSetup { .. }), "{none:?}");
    assert!(matches!(unknown, UpdateError::UnknownSetup { .. }), "{unknown:?}");
    assert!(matches!(without, UpdateError::SetupWithout { .. }), "{without:?}");
    assert!(picked(&w).is_empty());
}

#[test]
fn the_service_setup_pairs_in_a_window_from_the_menu_and_leaves_it_to_a_note_in_an_update() {
    let w = world();
    let mut m = fake(&w, &[]);

    w.updater(&mut m).add_apps(&names(&["lab"]), Some("service")).unwrap();

    let home = host(&m).home().to_path_buf();
    assert!(home.join("paired").exists());
    assert_eq!(user_journal(&w, "lab").unwrap().setup.as_deref(), Some("service"));
    assert_eq!(Picks::read(&w.user.app_picks()).unwrap().setup("lab"), Some("service"));

    fs::remove_file(home.join("paired")).unwrap();
    let mut events = Vec::new();
    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(!home.join("paired").exists());
    assert!(
        notes(&events).iter().any(|n| n.contains("lab pair")),
        "{:?}",
        notes(&events)
    );
}

#[test]
fn adding_the_service_without_a_display_leaves_pairing_to_a_note() {
    // Regression: `kubaos app add` over SSH ran the pairing and marked it done, opening nothing.
    let w = world();
    let mut m = fake(&w, &[]);
    m.apps.as_mut().unwrap().display = false;
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events)
        .add_apps(&names(&["lab"]), Some("service"))
        .unwrap();

    assert!(!host(&m).home().join("paired").exists(), "the pairing ran");
    assert!(
        notes(&events).iter().any(|n| n.contains("lab pair")),
        "{:?}",
        notes(&events)
    );
    let journal = user_journal(&w, "lab").unwrap();
    let pairing = journal.entries.iter().find(|e| e.action.name == "Pair Lab").unwrap();
    assert_eq!(pairing.state, crate::apps::journal::State::Skipped);
}

#[test]
fn switching_to_the_desktop_app_takes_the_service_away_and_keeps_the_app() {
    let w = world();
    let mut m = fake(&w, &[]);
    w.updater(&mut m).add_apps(&names(&["lab"]), Some("service")).unwrap();
    let home = host(&m).home().to_path_buf();

    w.updater(&mut m).add_apps(&names(&["lab"]), Some("desktop")).unwrap();

    assert!(has(&m, "lab"));
    assert!(!home.join(".config/systemd/user/lab.service").exists());
    assert!(!host(&m).state.borrow().linger);
    assert_eq!(user_journal(&w, "lab").unwrap().setup.as_deref(), Some("desktop"));
}

#[test]
fn an_installed_service_whose_steps_stand_needs_no_ready_check_in_an_update() {
    let w = world();
    let mut picks = Picks::default();
    picks.add_setup("lab", "service");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = fake(&w, &[]);
    w.updater(&mut m).add_apps(&names(&["lab"]), Some("service")).unwrap();
    let home = host(&m).home().to_path_buf();
    fs::write(home.join("lab-open"), "").unwrap();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(
        !warnings(&events).iter().any(|w| w.contains("Lab")),
        "{:?}",
        warnings(&events)
    );
}

#[test]
fn removing_a_service_that_is_open_is_refused_before_root_is_asked() {
    let w = world();
    let mut m = fake(&w, &[]);
    w.updater(&mut m).add_apps(&names(&["lab"]), Some("service")).unwrap();
    let home = host(&m).home().to_path_buf();
    fs::write(home.join("lab-open"), "").unwrap();
    let starts = root_starts(&m);

    let err = w.updater(&mut m).remove_apps(&names(&["lab"])).unwrap_err();

    assert!(err.to_string().contains("removing Lab"), "{err}");
    assert_eq!(root_starts(&m), starts);
    assert!(has(&m, "lab"));
    assert_eq!(picked(&w), ["lab"]);
}

/// A machine with desk installed before KubaOS kept journals, as an older release left it:
/// its package and its autostart entry.
fn desk_from_before(w: &World) -> Fake {
    let m = fake(w, &["desk"]);
    let entry = host(&m).place(Path::new("/etc/xdg/autostart/desk.desktop"));
    put(&entry, "[Desktop Entry]\nExec=desk\n");
    fs::set_permissions(&entry, fs::Permissions::from_mode(0o644)).unwrap();
    m
}

#[test]
fn adopting_a_picked_app_takes_in_what_its_checks_find_as_owned_running_nothing() {
    let w = world();
    let mut picks = Picks::default();
    picks.add("desk");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = desk_from_before(&w);
    let before = host(&m).snapshot();

    let taken = w.updater(&mut m).adopt_apps().unwrap();

    assert_eq!(taken, ["Desk"]);
    assert_eq!(host(&m).snapshot(), before);
    assert_eq!(root_starts(&m), 0, "taking an app in asked for a password");
    let journal = user_journal(&w, "desk").unwrap();
    assert!(journal.complete);
    assert_eq!(journal.root_adoption, Some(Adoption::Owned));
    assert_eq!(
        w.updater(&mut m).app_status().unwrap().get("desk").unwrap().state,
        AppState::Installed
    );
}

#[test]
fn an_adopted_app_kubaos_put_there_goes_whole_on_its_removal() {
    let w = world();
    let mut picks = Picks::default();
    picks.add("desk");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = desk_from_before(&w);
    w.updater(&mut m).adopt_apps().unwrap();

    w.updater(&mut m).remove_apps(&names(&["desk"])).unwrap();

    assert!(!has(&m, "desk"));
    assert!(!host(&m).place(Path::new("/etc/xdg/autostart/desk.desktop")).exists());
    assert_eq!(root_journal(&w, "desk"), None);
}

#[test]
fn an_adopted_app_the_user_put_there_owns_nothing_and_its_removal_leaves_it() {
    let w = world();
    let mut m = desk_from_before(&w);
    let before = host(&m).snapshot();
    w.updater(&mut m).adopt_apps().unwrap();
    assert_eq!(user_journal(&w, "desk").unwrap().root_adoption, Some(Adoption::Found));
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).remove_apps(&names(&["desk"])).unwrap();

    assert_eq!(host(&m).snapshot(), before);
    assert_eq!(user_journal(&w, "desk"), None);
    assert_eq!(root_starts(&m), 0, "a removal that undoes nothing asked for a password");
    let apps = &summary(&events).apps;
    assert!(apps.removed.is_empty(), "{apps:?}");
    assert_eq!(apps.left, names(&["Desk"]));
}

#[test]
fn an_update_records_roots_half_of_an_adopted_app_and_changes_nothing() {
    let w = world();
    let mut picks = Picks::default();
    picks.add("desk");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = desk_from_before(&w);
    w.updater(&mut m).adopt_apps().unwrap();
    let before = host(&m).snapshot();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert_eq!(host(&m).snapshot(), before);
    assert!(summary(&events).apps.installed.is_empty());
    assert_eq!(user_journal(&w, "desk").unwrap().root_adoption, None);
    let root = root_journal(&w, "desk").unwrap();
    assert!(
        root.entries.iter().all(|e| e.claims.iter().all(|c| c.owned)),
        "{root:?}"
    );
}

#[test]
fn a_service_set_up_by_hand_is_taken_in_as_that_setup_and_nothing_runs() {
    let w = world();
    let mut m = fake(&w, &["lab"]);
    let home = host(&m).home().to_path_buf();
    let unit = home.join(".config/systemd/user/lab.service");
    put(&unit, "[Service]\nExecStart=lab serve\n");
    fs::set_permissions(&unit, fs::Permissions::from_mode(0o644)).unwrap();
    host(&m).state.borrow_mut().linger = true;
    let before = host(&m).snapshot();

    w.updater(&mut m).adopt_apps().unwrap();

    assert_eq!(host(&m).snapshot(), before);
    let journal = user_journal(&w, "lab").unwrap();
    assert_eq!(journal.setup.as_deref(), Some("service"));
    assert!(journal.complete);
    assert!(journal.entries.iter().all(|e| e.claims.iter().all(|c| !c.owned)));
    assert!(!host(&m).state.borrow().log.iter().any(|l| l == "run ./pair"));
}

#[test]
fn an_app_not_on_the_machine_is_not_taken_in() {
    let w = world();
    let mut m = fake(&w, &[]);

    let taken = w.updater(&mut m).adopt_apps().unwrap();

    assert!(taken.is_empty());
    assert_eq!(user_journal(&w, "desk"), None);
}

#[test]
fn a_mark_left_beside_the_journals_is_no_apps_journal() {
    let w = world();
    let mut m = fake(&w, &[]);
    put(&w.user.app_journals().join("lab-linger-was-off"), "");

    w.updater(&mut m).add_apps(&names(&["agent"]), None).unwrap();

    assert!(has(&m, "agent"));
}

#[test]
fn a_service_kubaos_set_up_takes_lingering_only_when_its_mark_says_it_was_off() {
    let w = world();
    let mut picks = Picks::default();
    picks.add_setup("lab", "service");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = fake(&w, &["lab"]);
    let home = host(&m).home().to_path_buf();
    let unit = home.join(".config/systemd/user/lab.service");
    put(&unit, "[Service]\nExecStart=lab serve\n");
    fs::set_permissions(&unit, fs::Permissions::from_mode(0o644)).unwrap();
    host(&m).state.borrow_mut().linger = true;
    let mark: PathBuf = w.user.app_journals().join("lab-linger-was-off");
    put(&mark, "");

    w.updater(&mut m).adopt_apps().unwrap();

    assert!(!mark.exists(), "the mark stays beside the journal that replaces it");
    w.updater(&mut m).remove_apps(&names(&["lab"])).unwrap();
    assert!(!host(&m).state.borrow().linger, "lingering was off before KubaOS");
    assert!(!home.join(".config/systemd/user/lab.service").exists());
}

#[test]
fn an_app_for_another_architecture_is_not_taken_in_and_stops_nothing() {
    let w = world();
    let mut m = desk_from_before(&w);

    let taken = w.updater(&mut m).adopt_apps().unwrap();

    assert_eq!(taken, ["Desk"]);
    assert_eq!(user_journal(&w, "arm"), None);
}

#[test]
fn a_file_kubaos_put_there_with_an_older_releases_text_is_owned_and_brought_to_this_ones() {
    let w = world();
    let mut picks = Picks::default();
    picks.add("desk");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = desk_from_before(&w);
    let entry = host(&m).place(Path::new("/etc/xdg/autostart/desk.desktop"));
    fs::write(&entry, "[Desktop Entry]\nExec=desk --old\n").unwrap();
    w.updater(&mut m).adopt_apps().unwrap();

    w.updater(&mut m).update(yes()).unwrap();

    assert_eq!(fs::read_to_string(&entry).unwrap(), "[Desktop Entry]\nExec=desk\n");
    w.updater(&mut m).remove_apps(&names(&["desk"])).unwrap();
    assert!(!entry.exists(), "the file an older release wrote was KubaOS's");
}

/// The release's folder of the app `id` in the checkout's catalog, where it is.
fn with_app(w: &World, id: &str) -> PathBuf {
    let from = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps").join(id);
    let to = w.sys.catalog().join(id);
    fs::create_dir_all(&to).unwrap();
    for entry in fs::read_dir(&from).unwrap() {
        let entry = entry.unwrap();
        fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
    }
    lock_down(&to);
    to
}

/// T3 Code's service as KubaOS set it up, after `t3 update` moved the runtime from the pinned
/// 0.1.9 on to 0.2.0 and removed 0.1.9; lingering on, which the older mark says was off.
/// Its `t3` answers that nothing paired and uninstalls itself, and the sandbox's `systemctl` says
/// the service is enabled while its unit is there.
fn t3_service_after_t3_update(w: &World) -> Fake {
    let m = fake(w, &["t3code"]);
    let home = host(&m).home().to_path_buf();
    let runtime = home.join(".t3/runtime/versions/0.2.0");
    script(
        &runtime.join("t3"),
        r#"case "$*" in
  "auth session list"*) echo "[]" ;;
  "uninstall --yes") rm -rf "$HOME/.local/bin/t3" "$HOME/.t3/runtime" ;;
  *) exit 1 ;;
esac"#,
    );
    host(&m).sandbox().stub(
        "systemctl",
        r#"[ "$*" != "--user is-enabled --quiet t3code.service" ] || [ -e "$HOME/.config/systemd/user/t3code.service" ]"#,
    );
    put(&runtime.join(".install-complete"), "0.2.0\n");
    fs::create_dir_all(home.join(".local/bin")).unwrap();
    std::os::unix::fs::symlink(runtime.join("t3"), home.join(".local/bin/t3")).unwrap();
    put(&home.join(".config/systemd/user/t3code.service"), "[Service]\n");
    // The icon KubaOS's install put in the theme, as it stays through T3's own updates.
    let icon = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/t3-code/t3-code.svg");
    put(
        &home.join(".local/share/icons/hicolor/scalable/apps/t3-code.svg"),
        &fs::read_to_string(icon).unwrap(),
    );
    put(
        &home.join(".t3/userdata/desktop-settings.json"),
        "{\n  \"localEnvironmentEnabled\": false\n}\n",
    );
    host(&m).state.borrow_mut().linger = true;
    put(&w.user.app_journals().join("t3-code-linger-was-off"), "");
    m
}

#[test]
fn a_picked_t3_service_whose_pinned_runtime_t3_removed_is_taken_in_as_the_service_and_removed() {
    // Regression: the service was taken in as the desktop app, and the linger mark lost.
    let w = world();
    with_app(&w, "t3-code");
    let mut picks = Picks::default();
    picks.add_setup("t3-code", "service");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = t3_service_after_t3_update(&w);

    w.updater(&mut m).adopt_apps().unwrap();

    let journal = user_journal(&w, "t3-code").unwrap();
    assert_eq!(journal.setup.as_deref(), Some("service"));
    assert!(journal.complete);
    let linger = journal
        .entries
        .iter()
        .find(|e| matches!(e.action.kind, crate::apps::action::Kind::Linger {}))
        .expect("the linger step is recorded");
    assert!(linger.claims.iter().all(|c| c.owned), "{linger:?}");
    assert!(!w.user.app_journals().join("t3-code-linger-was-off").exists());
    let before = host(&m).snapshot();
    let from = host(&m).state.borrow().log.len();

    w.updater(&mut m).update(yes()).unwrap();

    let log = host(&m).state.borrow().log[from..].to_vec();
    assert!(!log.iter().any(|l| l.starts_with("fetch")), "{log:?}");
    assert_eq!(host(&m).snapshot(), before);

    w.updater(&mut m).remove_apps(&names(&["t3-code"])).unwrap();

    let home = host(&m).home().to_path_buf();
    assert!(!home.join(".config/systemd/user/t3code.service").exists());
    assert!(!home.join(".local/bin/t3").exists());
    assert!(!has(&m, "t3code"));
    assert!(!host(&m).state.borrow().linger, "lingering is off again, as it was");
    let calls = host(&m).sandbox().calls();
    for call in [
        "systemctl --user disable --now t3code.service",
        "systemctl --user daemon-reload",
    ] {
        assert!(calls.iter().any(|c| c == call), "{calls:?}");
    }
}

#[test]
fn a_linger_mark_stays_when_the_app_is_taken_in_without_a_linger_step() {
    let w = world();
    let mut picks = Picks::default();
    picks.add_setup("lab", "desktop");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = fake(&w, &["lab"]);
    let mark = w.user.app_journals().join("lab-linger-was-off");
    put(&mark, "");

    w.updater(&mut m).adopt_apps().unwrap();

    assert_eq!(user_journal(&w, "lab").unwrap().setup.as_deref(), Some("desktop"));
    assert!(
        mark.exists(),
        "the mark is the only word on lingering until a linger step holds it"
    );
}

#[test]
fn a_picked_app_is_taken_in_its_picked_setup_and_the_next_update_does_what_it_lacks() {
    let w = world();
    let mut picks = Picks::default();
    picks.add_setup("lab", "service");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = fake(&w, &["lab"]);

    w.updater(&mut m).adopt_apps().unwrap();

    assert_eq!(user_journal(&w, "lab").unwrap().setup.as_deref(), Some("service"));
    w.updater(&mut m).update(yes()).unwrap();
    let unit = host(&m).home().join(".config/systemd/user/lab.service");
    assert!(unit.exists(), "the service step an older setup lacked is done");
}

/// The comments the files of Docker's and Chrome's folders had in the release before, which
/// named the script that installed them then: by file, this release's lines and those.
const OLDER_COMMENTS: [(&str, &str, &str); 4] = [
    (
        "/etc/systemd/system/docker.service.d/kubaos-stop.conf",
        "# KubaOS: Docker's app (apps/docker) installs it beside the Docker it\n\
         # installs, and docker_on_demand (system/lib.sh) beside a Docker of your own.\n\
         # On SIGTERM dockerd gives each running container 10 s to stop and waits up\n\
         # to 15 s in all (its shutdown-timeout). The 10 s default from\n\
         # system.conf.d/kubaos-timeouts.conf would kill it first, and a database\n\
         # container with it, mid-write.\n",
        "# KubaOS: installed by system/40-apps.sh. On SIGTERM dockerd gives each\n\
         # running container 10 s to stop and waits up to 15 s in all (its\n\
         # shutdown-timeout). The 10 s default from system.conf.d/kubaos-timeouts.conf\n\
         # would kill it first, and a database container with it, mid-write.\n",
    ),
    (
        "/usr/local/libexec/dot-chrome-theme",
        "# Root-owned helper, installed to /usr/local/libexec by Chrome's app\n\
         # (apps/google-chrome) and started by dot-chrome-theme.path. It takes nothing\n\
         # from the user's file but a value that matches #rrggbb, so the policy\n\
         # directory stays root's: the user can't use it to set any other policy.",
        "# Root-owned helper, installed to /usr/local/libexec by system/40-apps.sh and\n\
         # started by dot-chrome-theme.path. It takes nothing from the user's file but\n\
         # a value that matches #rrggbb, so the policy directory stays root's: the user\n\
         # can't use it to set any other policy.",
    ),
    (
        "/etc/systemd/system/dot-chrome-theme.path",
        "Rendered and installed by Chrome's app (apps/google-chrome).",
        "Rendered and installed by system/40-apps.sh.",
    ),
    (
        "/etc/systemd/system/dot-chrome-theme.service",
        "(apps/google-chrome)",
        "(system/files/chrome/)",
    ),
];

#[test]
fn files_an_older_release_wrote_with_older_comments_are_taken_in_and_rewritten() {
    let w = world();
    let catalog = w.sys.catalog();
    for id in ["docker", "google-chrome"] {
        with_app(&w, id);
    }
    let mut picks = Picks::default();
    picks.add("docker");
    picks.add("google-chrome");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = fake(
        &w,
        &[
            "docker-ce",
            "docker-ce-cli",
            "containerd.io",
            "docker-buildx-plugin",
            "docker-compose-plugin",
            "google-chrome-stable",
        ],
    );
    let home = host(&m).home().to_string_lossy().into_owned();
    let files = [
        (
            "/etc/systemd/system/docker.service.d/kubaos-stop.conf",
            "docker/kubaos-stop.conf",
        ),
        ("/usr/local/libexec/dot-chrome-theme", "google-chrome/dot-chrome-theme"),
        (
            "/etc/systemd/system/dot-chrome-theme.path",
            "google-chrome/dot-chrome-theme.path",
        ),
        (
            "/etc/systemd/system/dot-chrome-theme.service",
            "google-chrome/dot-chrome-theme.service",
        ),
    ];
    let mut now = Vec::new();
    for ((path, from), (named, new, older)) in files.iter().zip(OLDER_COMMENTS) {
        assert_eq!(*path, named);
        let text = fs::read_to_string(catalog.join(from))
            .unwrap()
            .replace("@USER@", "kuba")
            .replace("@HOME@", &home);
        assert!(text.contains(new), "{from} no longer has the comment the test replaces");
        let place = host(&m).place(Path::new(path));
        put(&place, &text.replace(new, older));
        let mode = if path.contains("libexec") { 0o755 } else { 0o644 };
        fs::set_permissions(&place, fs::Permissions::from_mode(mode)).unwrap();
        now.push((place, text));
    }
    let machine = host(&m);
    {
        let mut state = machine.state.borrow_mut();
        state
            .groups
            .entry("docker".to_owned())
            .or_default()
            .insert("kuba".to_owned());
        for unit in [
            "docker.service",
            "containerd.service",
            "docker.socket",
            "dot-chrome-theme.path",
        ] {
            let on = matches!(unit, "docker.socket" | "dot-chrome-theme.path");
            state.units.insert(
                format!("system:{unit}"),
                crate::apps::host::fake::Unit {
                    exists: true,
                    enabled: on,
                    active: on,
                    ..crate::apps::host::fake::Unit::default()
                },
            );
        }
    }

    w.updater(&mut m).adopt_apps().unwrap();
    w.updater(&mut m).update(yes()).unwrap();

    for (place, text) in &now {
        assert_eq!(&fs::read_to_string(place).unwrap(), text, "{}", place.display());
    }
    for id in ["docker", "google-chrome"] {
        let root = root_journal(&w, id).unwrap();
        let file_paths: Vec<_> = root
            .entries
            .iter()
            .filter(|e| matches!(e.action.kind, crate::apps::action::Kind::File { .. }))
            .flat_map(|e| e.claims.iter())
            .filter(|c| c.claim.starts_with("path:"))
            .collect();
        assert!(
            !file_paths.is_empty() && file_paths.iter().all(|c| c.owned),
            "{id}: {file_paths:?}"
        );
    }
}

/// The fake machine restarted: what is enabled runs, a oneshot once, and nothing else.
fn reboot(m: &Fake) {
    for unit in host(m).state.borrow_mut().units.values_mut() {
        let oneshot = unit.oneshot && !unit.remain_after_exit;
        unit.active = unit.enabled && !oneshot;
        unit.exited = unit.enabled && oneshot;
    }
}

#[test]
fn an_update_after_a_reboot_leaves_chromes_colouring_service_alone() {
    // Regression: every update after a reboot ran the colouring service again, as root.
    let w = world();
    with_app(&w, "google-chrome");
    let mut picks = Picks::default();
    picks.add("google-chrome");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = fake(&w, &["google-chrome-stable"]);
    for unit in ["dot-chrome-theme.path", "dot-chrome-theme.service"] {
        host(&m).state.borrow_mut().units.insert(
            format!("system:{unit}"),
            crate::apps::host::fake::Unit {
                exists: true,
                oneshot: unit == "dot-chrome-theme.service",
                ..crate::apps::host::fake::Unit::default()
            },
        );
    }
    w.updater(&mut m).update(yes()).unwrap();
    assert!(root_journal(&w, "google-chrome").is_some());
    // What the service writes from the wallpaper's colours, which the fake doesn't run.
    let policy = host(&m).place(Path::new("/etc/opt/chrome/policies/managed/dot-theme.json"));
    put(&policy, "{\n  \"BrowserThemeColor\": \"#202020\"\n}\n");
    reboot(&m);
    let starts = root_starts(&m);

    w.updater(&mut m).update(yes()).unwrap();

    assert_eq!(root_starts(&m), starts, "root was asked to colour Chrome again");
}

#[test]
fn a_browser_from_its_vendors_repository_sets_the_repository_up_and_goes_whole() {
    let w = world();
    let browsers = [
        ("firefox", "mozilla", "firefox"),
        ("microsoft-edge", "microsoft-edge", "microsoft-edge-stable"),
        ("vivaldi", "vivaldi", "vivaldi-stable"),
        ("librewolf", "librewolf", "librewolf"),
    ];
    for (id, _, _) in browsers {
        with_app(&w, id);
    }
    let mut m = fake(&w, &[]);
    host(&m)
        .state
        .borrow_mut()
        .candidates
        .insert("firefox".to_owned(), "160.0~build1".to_owned());
    let pin = host(&m).place(Path::new("/etc/apt/preferences.d/kubaos-firefox"));

    for (id, repo, package) in browsers {
        w.updater(&mut m).add_apps(&names(&[id]), None).unwrap();

        assert!(host(&m).state.borrow().repos.contains(repo), "{id}");
        assert!(has(&m, package), "{id}");
    }
    assert_eq!(
        fs::read_to_string(&pin).unwrap(),
        fs::read_to_string(w.sys.catalog().join("firefox/kubaos-firefox.pref")).unwrap()
    );

    for (id, _, package) in browsers {
        w.updater(&mut m).remove_apps(&names(&[id])).unwrap();

        assert!(!has(&m, package), "{id}");
        assert!(user_journal(&w, id).is_none(), "{id}");
    }
    assert!(!pin.exists(), "the pin goes with Firefox");
}

#[test]
fn firefox_takes_the_place_of_ubuntus_snap_firefox_and_goes_whole() {
    let w = world();
    with_app(&w, "firefox");
    let mut m = fake(&w, &[]);
    {
        let host = host(&m);
        let mut state = host.state.borrow_mut();
        state
            .dpkg
            .insert("firefox".to_owned(), "1:1snap1-0ubuntu9.1".to_owned());
        state.candidates.insert("firefox".to_owned(), "160.0~build1".to_owned());
    }

    w.updater(&mut m).add_apps(&names(&["firefox"]), None).unwrap();

    let version = host(&m).state.borrow().dpkg.get("firefox").cloned();
    assert_eq!(version.as_deref(), Some("160.0~build1"));
    let journal = root_journal(&w, "firefox").unwrap();
    assert!(
        journal
            .entries
            .iter()
            .flat_map(|e| &e.claims)
            .any(|c| c.claim == "package:firefox" && c.owned),
        "{journal:?}"
    );

    w.updater(&mut m).remove_apps(&names(&["firefox"])).unwrap();

    assert!(!has(&m, "firefox"));
}

#[test]
fn tor_browser_brave_and_chromium_install_and_go_whole() {
    let w = world();
    let browsers = [
        ("tor-browser", "torbrowser-launcher"),
        ("brave", "brave"),
        ("chromium", "chromium"),
    ];
    for (id, _) in browsers {
        with_app(&w, id);
    }
    let mut m = fake(&w, &[]);
    let there = |m: &Fake, package: &str| has(m, package) || host(m).state.borrow().nix.contains(package);

    for (id, package) in browsers {
        w.updater(&mut m).add_apps(&names(&[id]), None).unwrap();

        assert!(there(&m, package), "{id}");
        assert!(user_journal(&w, id).is_some_and(|j| j.complete), "{id}");
    }
    assert!(has(&m, "torbrowser-launcher"), "the launcher is apt's");
    assert!(
        host(&m)
            .state
            .borrow()
            .nix
            .is_superset(&names(&["brave", "chromium"]).into_iter().collect())
    );

    for (id, package) in browsers {
        w.updater(&mut m).remove_apps(&names(&[id])).unwrap();

        assert!(!there(&m, package), "{id}");
        assert!(user_journal(&w, id).is_none() && root_journal(&w, id).is_none(), "{id}");
    }
}

/// A sandbox where `curl` saves the URL it is given as the key it downloads, failing for a URL
/// with `fails` in it, and `gpg` answers for such a key with the fingerprint the package manifest
/// pins for its URL, and for a keyring holding one of a repo's `global_fingerprints` as its text
/// with that one: a key fetch that works, without the network or the machine's keyrings.
fn key_sandbox(dir: &Path, fails: &str) -> crate::sandbox::Sandbox {
    let manifest =
        crate::apt::Manifest::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../system/packages.toml"))
            .unwrap()
            .unwrap();
    let mut answers = String::new();
    for repo in manifest.repos().values() {
        let keys = [
            (Some(&repo.key_url), Some(&repo.fingerprint)),
            (repo.next_key_url.as_ref(), repo.next_fingerprint.as_ref()),
        ];
        for (url, fingerprint) in keys {
            if let (Some(url), Some(fingerprint)) = (url, fingerprint) {
                let _ = writeln!(answers, "[ \"$url\" != '{url}' ] || f={fingerprint}");
            }
        }
        // A vendor's older key, in a keyring that holds its fingerprint as its text.
        for fingerprint in &repo.global_fingerprints {
            let _ = writeln!(answers, "[ \"$url\" != '{fingerprint}' ] || f={fingerprint}");
        }
    }
    let home = dir.join("home");
    fs::create_dir_all(&home).unwrap();
    let sandbox = crate::sandbox::Sandbox::new(&dir.join("sandbox"), &home, "kuba");
    sandbox.stub("dpkg", "echo amd64");
    sandbox.stub(
        "curl",
        &format!(
            "out=; url=\nwhile [ $# -gt 0 ]; do case \"$1\" in -o) out=$2; shift ;; -*) ;; *) url=$1 ;; esac; shift; done\n\
             case \"$url\" in *{fails}*) exit 22 ;; esac\nprintf '%s' \"$url\" >\"$out\""
        ),
    );
    sandbox.stub(
        "gpg",
        &format!(
            "case \"$*\" in *--dearmor*) cat; exit 0 ;; esac\nfor last; do :; done\nurl=$(cat \"$last\") || exit 2\nf=\n\
             {answers}[ -n \"$f\" ] || exit 2\necho 'pub:-:4096:1:0000000000000000:::::::'\necho \"fpr:::::::::$f:\""
        ),
    );
    sandbox
}

#[test]
fn the_browser_repos_migration_refreshes_only_the_repositories_kubaos_set_up_and_never_fails() {
    let w = world();
    for id in ["firefox", "vivaldi", "librewolf"] {
        with_app(&w, id);
    }
    let mut m = fake(&w, &[]);
    w.updater(&mut m)
        .add_apps(&names(&["firefox", "vivaldi", "librewolf"]), None)
        .unwrap();
    let dir = w.tmp.path().join("migration");
    let sources = dir.join("sources.list.d");
    let edge = "Types: deb\nURIs: https://packages.microsoft.com/repos/edge-stable\nSigned-By: /usr/share/keyrings/microsoft-edge.gpg\n";
    put(&sources.join("microsoft-edge.sources"), edge);
    let sandbox = key_sandbox(&dir, "librewolf");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let migration = fs::read_dir(root.join("migrations/system"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.to_string_lossy().ends_with("-browser-repos.sh"))
        .unwrap();

    let out = sandbox
        .command("bash")
        .args(["-euo", "pipefail"])
        .arg(&migration)
        .env("KUBAOS_DIR", &root)
        .env("APT_SOURCES_D", &sources)
        .env("KUBAOS_APP_JOURNALS", w.sys.app_journals())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    let calls = sandbox.calls();
    let wrote = |file: &str| {
        calls
            .iter()
            .any(|c| c.starts_with("sudo install -D -m 0644 ") && c.ends_with(&format!(" {file}")))
    };
    let at = |name: &str| sources.join(name).display().to_string();
    assert!(
        wrote(&at("mozilla.sources")) && wrote(&at("vivaldi.sources")),
        "{calls:#?}"
    );
    assert!(
        wrote("/usr/share/keyrings/vivaldi-16BD9233.gpg") && wrote("/usr/share/keyrings/vivaldi-5220E3E9.gpg"),
        "both of Vivaldi's keys: {calls:#?}"
    );
    assert!(
        !calls.iter().any(|c| c.contains("microsoft")),
        "Edge was installed by hand, so its source is Microsoft's: {calls:#?}"
    );
    assert_eq!(
        fs::read_to_string(sources.join("microsoft-edge.sources")).unwrap(),
        edge
    );
    assert!(!wrote(&at("librewolf.sources")), "{calls:#?}");
    assert!(stderr.contains("apt repository librewolf not refreshed"), "{stderr}");
    let updates = calls.iter().filter(|c| *c == "sudo apt-get update").count();
    assert_eq!(updates, 1, "one apt update for every repository: {calls:#?}");
}

#[test]
fn the_communication_repos_migration_refreshes_only_what_kubaos_set_up_and_untrusts_slacks_keys() {
    let w = world();
    for id in ["signal", "slack", "element", "zulip"] {
        with_app(&w, id);
    }
    let mut m = fake(&w, &[]);
    w.updater(&mut m)
        .add_apps(&names(&["signal", "slack", "element"]), None)
        .unwrap();
    let dir = w.tmp.path().join("migration");
    let sources = dir.join("sources.list.d");
    let zulips_own =
        "Types: deb\nURIs: https://download.zulip.com/desktop/apt\nSigned-By: /usr/share/keyrings/zulip-desktop.asc\n";
    put(&sources.join("zulip-desktop.sources"), zulips_own);
    let trusted = dir.join("trusted.gpg.d");
    let manifest =
        crate::apt::Manifest::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../system/packages.toml"))
            .unwrap()
            .unwrap();
    let slack = &manifest.repos()["slack"];
    put(&trusted.join("slack-desktop.gpg"), &slack.key_url);
    put(&trusted.join("packagecloud.gpg"), &slack.global_fingerprints[0]);
    put(&trusted.join("someone.gpg"), "https://example.com/someone.asc");
    let sandbox = key_sandbox(&dir, "element");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let migration = fs::read_dir(root.join("migrations/system"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.to_string_lossy().ends_with("-communication-repos.sh"))
        .unwrap();

    let out = sandbox
        .command("bash")
        .args(["-euo", "pipefail"])
        .arg(&migration)
        .env("KUBAOS_DIR", &root)
        .env("APT_SOURCES_D", &sources)
        .env("APT_TRUSTED_D", &trusted)
        .env("KUBAOS_APP_JOURNALS", w.sys.app_journals())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    let calls = sandbox.calls();
    let wrote = |file: &str| {
        calls
            .iter()
            .any(|c| c.starts_with("sudo install -D -m 0644 ") && c.ends_with(&format!(" {file}")))
    };
    let at = |name: &str| sources.join(name).display().to_string();
    assert!(
        wrote(&at("signal.sources")) && wrote(&at("slack.sources")),
        "{calls:#?}"
    );
    assert!(
        !wrote(&at("zulip-desktop.sources")),
        "Zulip was installed by hand: {calls:#?}"
    );
    assert_eq!(
        fs::read_to_string(sources.join("zulip-desktop.sources")).unwrap(),
        zulips_own
    );
    assert!(!wrote(&at("element.sources")), "{calls:#?}");
    assert!(stderr.contains("apt repository element not refreshed"), "{stderr}");
    let removed: Vec<&str> = calls.iter().filter_map(|c| c.strip_prefix("sudo rm -f ")).collect();
    let gone = |name: &str| trusted.join(name).display().to_string();
    assert_eq!(
        removed,
        [gone("slack-desktop.gpg"), gone("packagecloud.gpg")],
        "Slack's keys, trusted for every repository, go; another's stays"
    );
    let updates = calls.iter().filter(|c| *c == "sudo apt-get update").count();
    assert_eq!(updates, 1, "one apt update for every repository: {calls:#?}");
}

/// The root journal's entry of the app `id`'s `apt` step.
fn apt_entry(w: &World, id: &str) -> crate::apps::journal::Entry {
    root_journal(w, id)
        .unwrap()
        .entries
        .into_iter()
        .find(|e| matches!(e.action.kind, Kind::Apt { .. }))
        .unwrap()
}

/// Agent, picked, on a machine an older release put it on, its repository set up or not.
fn agent_from_before(w: &World, repo_set_up: bool) -> Fake {
    let mut picks = Picks::default();
    picks.add("agent");
    picks.write(&w.user.app_picks()).unwrap();
    let m = fake(w, &["agent"]);
    if repo_set_up {
        host(&m).state.borrow_mut().repos.insert("vendor".to_owned());
    }
    m
}

#[test]
fn an_app_kubaos_put_there_from_its_set_up_repository_is_taken_in_owned_and_nothing_runs() {
    let w = world();
    let mut m = agent_from_before(&w, true);
    w.updater(&mut m).adopt_apps().unwrap();
    let before = host(&m).snapshot();

    w.updater(&mut m).update(yes()).unwrap();

    assert_eq!(host(&m).snapshot(), before);
    assert_eq!(host(&m).state.borrow().repos, ["vendor".to_owned()].into());
    let entry = apt_entry(&w, "agent");
    assert_eq!(entry.state, State::Done);
    assert!(entry.claims.iter().all(|c| c.owned), "{entry:?}");
}

#[test]
fn an_app_kubaos_put_there_without_its_repository_set_up_has_it_set_up_and_its_packages_stay_owned() {
    let w = world();
    let mut m = agent_from_before(&w, false);
    let taken = w.updater(&mut m).adopt_apps().unwrap();
    assert_eq!(taken, ["Agent"]);

    w.updater(&mut m).update(yes()).unwrap();

    assert!(host(&m).state.borrow().repos.contains("vendor"));
    let entry = apt_entry(&w, "agent");
    assert_eq!(entry.state, State::Done);
    assert!(entry.claims.iter().all(|c| c.owned), "{entry:?}");
    w.updater(&mut m).remove_apps(&names(&["agent"])).unwrap();
    assert!(!has(&m, "agent"), "KubaOS put it there, so its removal takes it");
}

/// Slack's daily job's `slack.list`, as a Slack installed by hand has it.
const SLACKS_OWN_LIST: &str = "### THIS FILE IS AUTOMATICALLY CONFIGURED ###\n\
     deb [arch=amd64] https://packagecloud.io/slacktechnologies/slack/debian/ jessie main\n";

#[test]
fn a_slack_installed_by_hand_is_set_up_from_kubaos_repository_and_its_global_keys_go() {
    let w = world();
    with_app(&w, "slack");
    let mut m = fake(&w, &["slack-desktop"]);
    put(
        &host(&m).place(Path::new("/etc/default/slack")),
        "repo_add_once=\"false\"\nrepo_reenable_on_distupgrade=\"true\"\n",
    );

    w.updater(&mut m).add_apps(&names(&["slack"]), None).unwrap();

    assert!(
        host(&m).state.borrow().repos.contains("slack"),
        "its repository wasn't set up"
    );
    assert!(has(&m, "slack-desktop"));
    let entry = apt_entry(&w, "slack");
    assert_eq!(entry.state, State::Done, "KubaOS set its repository up");
    assert!(
        entry.claims.iter().all(|c| !c.owned),
        "the package was the user's: {entry:?}"
    );
    assert_eq!(
        w.updater(&mut m).app_status().unwrap().get("slack").unwrap().state,
        AppState::Found
    );
    w.updater(&mut m).update(yes()).unwrap();
    assert_eq!(
        apt_entry(&w, "slack").state,
        State::Done,
        "an update that checks it keeps it KubaOS's"
    );

    // What the real machine runs to set the repository up, system/apt-repo.sh slack, on a
    // machine where Slack's job ran.
    let dir = w.tmp.path().join("setup");
    let sources = dir.join("sources.list.d");
    put(&sources.join("slack.list"), SLACKS_OWN_LIST);
    let trusted = dir.join("trusted.gpg.d");
    let manifest =
        crate::apt::Manifest::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../system/packages.toml"))
            .unwrap()
            .unwrap();
    let slack = &manifest.repos()["slack"];
    put(&trusted.join("slack-desktop.gpg"), &slack.key_url);
    put(&trusted.join("packagecloud.gpg"), &slack.global_fingerprints[0]);
    let sandbox = key_sandbox(&dir, "nothing");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

    let out = sandbox
        .command(root.join("system/apt-repo.sh"))
        .arg("slack")
        .env("APT_SOURCES_D", &sources)
        .env("APT_TRUSTED_D", &trusted)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    let calls = sandbox.calls();
    let wrote = |path: &Path| {
        calls
            .iter()
            .any(|c| c.starts_with("sudo install -D -m 0644 ") && c.ends_with(&format!(" {}", path.display())))
    };
    assert!(wrote(&sources.join("slack.sources")), "{calls:#?}");
    assert!(wrote(&sources.join("slack.list")), "slack.list turned off: {calls:#?}");
    let removed: Vec<&str> = calls.iter().filter_map(|c| c.strip_prefix("sudo rm -f ")).collect();
    let gone = |name: &str| trusted.join(name).display().to_string();
    assert_eq!(removed, [gone("slack-desktop.gpg"), gone("packagecloud.gpg")]);

    // Its journal names the repository KubaOS set up, so a release's migration refreshes it.
    let set_up = sandbox
        .command("bash")
        .args([
            "-c",
            "source \"$1/migrations/lib.sh\"; kubaos_app_set_up_repo slack slack",
            "-",
        ])
        .arg(&root)
        .env("KUBAOS_APP_JOURNALS", w.sys.app_journals())
        .status()
        .unwrap();
    assert!(set_up.success());
}

#[test]
fn zen_is_its_pinned_appimage_with_an_entry_that_runs_it_and_its_profile_stays() {
    let w = world();
    let folder = with_app(&w, "zen-browser");
    let app: toml::Table = toml::from_str(&fs::read_to_string(folder.join("app.toml")).unwrap()).unwrap();
    let download = app["action"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["kind"].as_str() == Some("download"))
        .unwrap();
    let pin = |key: &str| download["amd64"][key].as_str().unwrap().to_owned();
    let mut m = fake(&w, &[]);
    host(&m)
        .state
        .borrow_mut()
        .downloads
        .insert(pin("url"), (pin("sha256"), b"an AppImage".to_vec()));
    let home = host(&m).home().to_path_buf();
    let appimage = home.join(".local/opt/zen-browser/zen.AppImage");
    let entry = home.join(".local/share/applications/zen.desktop");

    w.updater(&mut m).add_apps(&names(&["zen-browser"]), None).unwrap();

    assert!(has(&m, "fuse3"));
    assert_eq!(fs::read(&appimage).unwrap(), b"an AppImage");
    assert_eq!(fs::metadata(&appimage).unwrap().permissions().mode() & 0o111, 0o111);
    let desktop = fs::read_to_string(&entry).unwrap();
    assert!(
        desktop.contains(&format!("\nExec=\"{}\" %u\n", appimage.display())),
        "{desktop}"
    );
    assert!(!desktop.contains('@'), "{desktop}");
    let profile = home.join(".config/zen/profiles.ini");
    put(&profile, "[General]\n");

    w.updater(&mut m).remove_apps(&names(&["zen-browser"])).unwrap();

    assert!(!appimage.exists() && !entry.exists());
    assert!(!home.join(".local/opt/zen-browser").exists());
    assert!(!has(&m, "fuse3"), "FUSE goes with the app that brought it");
    assert!(profile.exists(), "the profile is the user's");
}

/// Agent's kit: it requires Agent, and writes KubaOS's defaults for it in a configure step.
const KIT: &str = r#"
name = "Kit"
summary = "Agent's kit"
requires = ["agent"]

[[action]]
name = "Write the kit"
kind = "file"
to = "~/.config/kit/kit.toml"
text = "kit = true\n"

[[action]]
name = "Write KubaOS's defaults"
kind = "file"
stage = "configure"
to = "~/.config/kubaos/kit/defaults.toml"
text = "theme = \"night\"\n"
"#;

/// [`world`] with Kit beside the others.
fn world_with_kit() -> World {
    let w = world();
    put(&w.sys.catalog().join("kit/app.toml"), &format!("{ABOUT}{KIT}"));
    lock_down(&w.sys.catalog().join("kit"));
    w
}

#[test]
fn adding_an_app_installs_what_it_requires_first_without_picking_it() {
    let w = world_with_kit();
    let mut m = fake(&w, &[]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events)
        .add_apps(&names(&["kit"]), None)
        .unwrap();

    assert_eq!(picked(&w), ["kit"]);
    assert!(has(&m, "agent"));
    assert_eq!(root_starts(&m), 1);
    assert_eq!(user_journal(&w, "agent").unwrap().brought_by, ["kit"]);
    assert_eq!(summary(&events).apps.installed, names(&["Agent", "Kit"]));
}

#[test]
fn removing_an_app_another_needs_is_refused_and_the_picks_stay() {
    let w = world_with_kit();
    let mut m = fake(&w, &[]);
    w.updater(&mut m).add_apps(&names(&["agent", "kit"]), None).unwrap();

    let err = w.updater(&mut m).remove_apps(&names(&["agent"])).unwrap_err();

    assert_eq!(chain(&err), "removing Agent: Agent is needed by Kit: remove that first");
    assert_eq!(picked(&w), ["agent", "kit"]);
    assert!(has(&m, "agent"));
}

#[test]
fn removing_an_app_takes_what_it_brought_and_lists_it() {
    let w = world_with_kit();
    let mut m = fake(&w, &[]);
    w.updater(&mut m).add_apps(&names(&["kit"]), None).unwrap();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).remove_apps(&names(&["kit"])).unwrap();

    assert!(picked(&w).is_empty());
    assert!(!has(&m, "agent"));
    assert_eq!(summary(&events).apps.removed, names(&["Kit", "Agent"]));
}

#[test]
fn configure_runs_the_configure_stage_alone_and_changes_no_pick() {
    let w = world_with_kit();
    let mut m = fake(&w, &[]);
    w.updater(&mut m).add_apps(&names(&["kit"]), None).unwrap();
    let defaults = host(&m).home().join(".config/kubaos/kit/defaults.toml");
    fs::write(&defaults, "theme = \"day\"\n").unwrap();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events)
        .configure_apps(&names(&["kit"]))
        .unwrap();

    assert_eq!(fs::read_to_string(&defaults).unwrap(), "theme = \"night\"\n");
    assert_eq!(picked(&w), ["kit"]);
    assert_eq!(root_starts(&m), 1, "only the add asked for root");
    assert_eq!(summary(&events).apps.configured, names(&["Kit"]));
}

#[test]
fn an_update_installs_what_a_picked_app_requires() {
    let w = world_with_kit();
    let mut m = fake(&w, &[]);
    let mut picks = Picks::default();
    picks.add("kit");
    picks.write(&w.user.app_picks()).unwrap();

    w.updater(&mut m).reconcile_apps();

    assert!(has(&m, "agent"));
    assert!(user_journal(&w, "kit").unwrap().complete);
    assert_eq!(user_journal(&w, "agent").unwrap().brought_by, ["kit"]);
}

/// An app in two setups: with Agent, or on its own.
const RIG: &str = r#"
name = "Rig"
summary = "A rig"

[setup.full]
name = "Rig with Agent"
summary = "The rig and its agent"
requires = ["agent"]

[[setup.full.action]]
name = "Write the full rig"
kind = "file"
to = "~/.config/rig/full.toml"
text = "full = true\n"

[setup.bare]
name = "Rig"
summary = "The rig alone"

[[setup.bare.action]]
name = "Write the bare rig"
kind = "file"
to = "~/.config/rig/bare.toml"
text = "bare = true\n"
"#;

#[test]
fn an_update_lists_the_app_a_picked_setup_change_left_behind_as_removed() {
    let w = world();
    put(&w.sys.catalog().join("rig/app.toml"), &format!("{ABOUT}{RIG}"));
    lock_down(&w.sys.catalog().join("rig"));
    let mut m = fake(&w, &[]);
    w.updater(&mut m).add_apps(&names(&["rig"]), Some("full")).unwrap();
    assert!(has(&m, "agent"));
    let mut picks = Picks::read(&w.user.app_picks()).unwrap();
    picks.add_setup("rig", "bare");
    picks.write(&w.user.app_picks()).unwrap();

    let changes = w.updater(&mut m).reconcile_apps();

    assert!(!has(&m, "agent"));
    assert_eq!(user_journal(&w, "rig").unwrap().setup.as_deref(), Some("bare"));
    assert_eq!(changes.removed, names(&["Agent"]));
}

mod ai;
mod devtools;
mod editors;
mod games;
mod graphics;
mod media;
mod office;
mod security;
mod system;

#[test]
fn vivaldi_rewriting_its_sources_file_with_its_one_key_leaves_the_next_update_nothing_to_do() {
    let w = world();
    with_app(&w, "vivaldi");
    let mut m = fake(&w, &[]);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest = crate::apt::Manifest::read(&root.join("system/packages.toml"))
        .unwrap()
        .unwrap();
    let ours = fs::read_to_string(root.join("system/files/apt/sources.list.d/vivaldi.sources")).unwrap();
    host(&m).state.borrow_mut().repo_files.insert(
        "vivaldi".to_owned(),
        (manifest.repos()["vivaldi"].clone(), ours.clone()),
    );
    w.updater(&mut m).add_apps(&names(&["vivaldi"]), None).unwrap();
    // Vivaldi's postinst and daily cron job write the file again, naming the one key that build
    // signs with.
    let sources = host(&m).place(Path::new("/etc/apt/sources.list.d/vivaldi.sources"));
    let vivaldis_own = ours.replace(" /usr/share/keyrings/vivaldi-5220E3E9.gpg", "");
    assert_ne!(vivaldis_own, ours);
    put(&sources, &vivaldis_own);
    let starts = root_starts(&m);
    let before = host(&m).snapshot();
    let log = host(&m).state.borrow().log.len();

    w.updater(&mut m).update(yes()).unwrap();

    assert_eq!(root_starts(&m), starts, "the update asked for root");
    assert_eq!(host(&m).state.borrow().log[log..], [] as [String; 0]);
    assert_eq!(host(&m).snapshot(), before);
}
