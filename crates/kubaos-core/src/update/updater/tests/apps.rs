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
use crate::apps::host::Host;
use crate::apps::host::fake::FakeHost;
use crate::apps::journal::Adoption;
use crate::apps::journal::Journal;
use crate::update::UpdateError;
use crate::update::event::AppChanges;
use crate::update::event::Phase;
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

/// The checkout's catalog: agent, desk and lab, with lab's scripts, and the repository setup
/// root runs; all of it the test user's alone, as the root helper wants it.
fn world() -> World {
    let w = World::new(Channel::Edge, true);
    for (id, text) in [("agent", AGENT), ("desk", DESK), ("lab", LAB)] {
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
    for id in ["agent", "desk", "lab"] {
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
            fs::set_permissions(&path, fs::Permissions::from_mode(0o644 | mode | (mode >> 3) | (mode >> 6)))
                .unwrap();
        }
    }
}

fn script(path: &Path, body: &str) {
    put(path, &format!("#!/bin/sh\n{body}\n"));
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
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
            removed: Vec::new(),
        }
    );
}

#[test]
fn each_step_is_an_event_with_its_name() {
    let w = world();
    let mut m = fake(&w, &[]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).add_apps(&names(&["desk"]), None).unwrap();

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

    w.reporting(&mut m, &mut events).add_apps(&names(&["desk"]), None).unwrap();

    assert_eq!(host(&m).snapshot(), before);
    assert_eq!(root_starts(&m), 1, "only the first add asked for root");
    assert_eq!(summary(&events).apps.installed, names(&["Desk"]));
}

#[test]
fn an_id_the_catalog_lacks_is_refused_before_anything_changes() {
    let w = world();
    let mut m = fake(&w, &[]);

    let err = w.updater(&mut m).add_apps(&names(&["agent", "nope"]), None).unwrap_err();

    assert!(matches!(err, UpdateError::NoApp { ref id } if id == "nope"), "{err:?}");
    assert!(picked(&w).is_empty());
    assert!(root_helpers(&m).is_empty());
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
fn an_update_installs_the_picked_apps_through_sudo_and_a_second_starts_no_root() {
    let w = world();
    let mut picks = Picks::default();
    picks.add("agent");
    picks.write(&w.user.app_picks()).unwrap();
    let mut m = fake(&w, &[]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(has(&m, "agent"));
    assert!(root_helpers(&m).iter().all(|h| h.starts_with("sudo ")), "{:?}", root_helpers(&m));
    assert_eq!(summary(&events).apps.installed, names(&["Agent"]));
    assert!(phases(&events).contains(&Phase::Apps));

    let starts = root_starts(&m);
    let mut again = Vec::new();
    w.reporting(&mut m, &mut again).update(yes()).unwrap();

    assert_eq!(root_starts(&m), starts, "a second update asked for root again");
    assert!(summary(&again).apps.installed.is_empty());
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
    let without = w.updater(&mut m).add_apps(&names(&["agent"]), Some("desktop")).unwrap_err();

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
    assert!(notes(&events).iter().any(|n| n.contains("lab pair")), "{:?}", notes(&events));
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

    w.updater(&mut m).remove_apps(&names(&["desk"])).unwrap();

    assert_eq!(host(&m).snapshot(), before);
    assert_eq!(user_journal(&w, "desk"), None);
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
    assert!(root.entries.iter().all(|e| e.claims.iter().all(|c| c.owned)), "{root:?}");
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
