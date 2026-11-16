use std::fs;
use std::path::Path;

use super::Fake;
use super::FakeDpkg;
use super::World;
use super::phases;
use super::summary;
use super::warnings;
use super::yes;
use crate::apps::Added;
use crate::apps::AppState;
use crate::apps::Picks;
use crate::update::UpdateError;
use crate::update::event::AppChanges;
use crate::update::event::Phase;
use crate::update::record::Channel;

const AGENT: &str = r#"
name = "Agent"
summary = "A coding agent"

[apt]
repo = "vendor"
packages = ["agent"]
"#;

const DESK: &str = r#"
name = "Desk"
summary = "A desktop app"

[deb]
package = "desk"
version = "1.0"

[deb.amd64]
url = "https://example.com/desk-amd64.deb"
sha256 = "418c7825f73107b7b5ba3babae5e3b485801dc22d9dc0d518af632f7906cbdd1"

[deb.arm64]
url = "https://example.com/desk-arm64.deb"
sha256 = "418c7825f73107b7b5ba3babae5e3b485801dc22d9dc0d518af632f7906cbdd1"
"#;

const EDITOR: &str = r#"
name = "Editor"
summary = "An editor"

[nix]
packages = ["editor"]
"#;

/// A world whose checked-out release has the catalog agent, desk and editor, and whose home.nix
/// can be wired to kubaos.d/.
fn world() -> World {
    let w = World::new(Channel::Edge, true);
    for (id, text) in [("agent", AGENT), ("desk", DESK), ("editor", EDITOR)] {
        put(&w.sys.catalog().join(format!("{id}.toml")), text);
    }
    put(&w.user.home_nix(), "{\n}\n");
    w
}

/// A machine with `installed`, whose helper installs each app's package.
fn fake(installed: &[&str]) -> Fake {
    let mut m = Fake::new();
    let mut dpkg = FakeDpkg::with(installed);
    dpkg.apps.insert("agent".into(), vec!["agent".into()]);
    dpkg.apps.insert("desk".into(), vec!["desk".into()]);
    m.dpkg = Some(dpkg);
    m
}

fn put(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn picked(w: &World) -> Vec<String> {
    let picks = Picks::read(&w.user.app_picks()).unwrap();
    picks.iter().map(str::to_owned).collect()
}

/// The runs of the root helper, as the fake recorded them.
fn helper_runs(m: &Fake) -> Vec<&str> {
    m.ran
        .iter()
        .filter(|line| line.contains("/kubaos-apps "))
        .map(String::as_str)
        .collect()
}

fn names(names: &[&str]) -> Vec<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

#[test]
fn add_picks_the_apps_and_installs_them_in_one_run_of_the_helper_through_polkit() {
    let w = world();
    let mut m = fake(&[]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events)
        .add_apps(&names(&["desk", "agent"]))
        .unwrap();

    assert_eq!(picked(&w), ["agent", "desk"]);
    let runs = helper_runs(&m);
    assert_eq!(runs.len(), 1, "{runs:?}");
    assert!(runs[0].starts_with("pkexec "), "{runs:?}");
    assert!(
        runs[0].ends_with("/usr/local/libexec/kubaos-apps install agent desk"),
        "{runs:?}"
    );
    assert_eq!(
        summary(&events).apps,
        AppChanges {
            installed: names(&["Agent", "Desk"]),
            removed: Vec::new(),
        }
    );
    assert!(phases(&events).contains(&Phase::Apps));
}

#[test]
fn adding_an_app_that_is_there_only_picks_it() {
    let w = world();
    let mut m = fake(&["agent"]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).add_apps(&names(&["agent"])).unwrap();

    assert_eq!(picked(&w), ["agent"]);
    assert!(helper_runs(&m).is_empty(), "{:?}", m.ran);
    assert_eq!(summary(&events).apps.installed, ["Agent"]);
}

#[test]
fn removing_an_app_that_is_gone_lists_it_as_removed() {
    let w = world();
    put(&w.user.app_picks(), "apps = [\"agent\"]\n");
    let mut m = fake(&[]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events)
        .remove_apps(&names(&["agent"]))
        .unwrap();

    assert!(picked(&w).is_empty());
    assert!(helper_runs(&m).is_empty(), "{:?}", m.ran);
    assert_eq!(summary(&events).apps.removed, ["Agent"]);
}

#[test]
fn an_id_the_catalog_lacks_is_refused_before_anything_changes() {
    let w = world();
    let mut m = fake(&[]);

    let added = w.updater(&mut m).add_apps(&names(&["agent", "../etc"]));

    assert!(matches!(added, Err(UpdateError::NoApp { id }) if id == "../etc"));
    assert!(picked(&w).is_empty());
    assert!(helper_runs(&m).is_empty());
}

#[test]
fn a_failed_or_declined_install_puts_the_picks_back() {
    let w = world();
    put(&w.user.app_picks(), "apps = [\"desk\"]\n");
    let mut m = fake(&["desk"]);
    m.failing.push("pkexec");

    let added = w.updater(&mut m).add_apps(&names(&["agent"]));

    assert!(added.is_err());
    assert_eq!(picked(&w), ["desk"]);
}

#[test]
fn remove_drops_the_pick_and_takes_the_app_away_through_polkit() {
    let w = world();
    put(&w.user.app_picks(), "apps = [\"agent\", \"desk\"]\n");
    let mut m = fake(&["agent", "desk"]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events)
        .remove_apps(&names(&["agent"]))
        .unwrap();

    assert_eq!(picked(&w), ["desk"]);
    let runs = helper_runs(&m);
    assert_eq!(runs.len(), 1);
    assert!(
        runs[0].starts_with("pkexec ") && runs[0].ends_with("kubaos-apps remove agent"),
        "{runs:?}"
    );
    assert_eq!(summary(&events).apps.removed, ["Agent"]);
    assert!(!m.dpkg.as_ref().unwrap().installed.contains_key("agent"));
}

#[test]
fn an_app_from_nixpkgs_goes_into_kubaos_d_and_the_home_switches_without_root() {
    let w = world();
    let mut m = fake(&[]);

    w.updater(&mut m).add_apps(&names(&["editor"])).unwrap();

    let packages = fs::read_to_string(w.user.state_dir().packages_file()).unwrap();
    assert!(packages.contains("\"editor\""), "{packages}");
    assert_eq!(m.switches, 1);
    assert!(helper_runs(&m).is_empty());
    assert!(
        fs::read_to_string(w.user.home_nix())
            .unwrap()
            .contains("kubaos.stateDir")
    );

    w.updater(&mut m).remove_apps(&names(&["editor"])).unwrap();

    let packages = fs::read_to_string(w.user.state_dir().packages_file()).unwrap();
    assert!(!packages.contains("\"editor\""), "{packages}");
    assert_eq!(m.switches, 2);
}

#[test]
fn an_update_installs_the_picked_apps_the_machine_lacks_through_sudo_once() {
    let w = world();
    put(&w.user.app_picks(), "apps = [\"agent\", \"desk\", \"editor\"]\n");
    let mut m = fake(&[]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    let runs = helper_runs(&m);
    assert_eq!(runs.len(), 1, "{runs:?}");
    assert!(
        runs[0].starts_with("sudo ") && runs[0].ends_with("kubaos-apps install agent desk"),
        "{runs:?}"
    );
    let packages = fs::read_to_string(w.user.state_dir().packages_file()).unwrap();
    assert!(packages.contains("\"editor\""));
    assert_eq!(m.switches, 1, "the update's own switch installs editor");
    assert_eq!(summary(&events).apps.installed, ["Agent", "Desk", "Editor"]);
    let order = phases(&events);
    let at = |p| order.iter().position(|q| *q == p).unwrap();
    assert!(at(Phase::Apps) < at(Phase::Home), "{order:?}");
}

#[test]
fn a_second_update_with_the_same_picks_does_nothing_for_the_apps() {
    let w = world();
    put(&w.user.app_picks(), "apps = [\"agent\", \"editor\"]\n");
    let mut m = fake(&[]);
    w.updater(&mut m).update(yes()).unwrap();
    let mut again = Fake::new();
    again.dpkg = m.dpkg.take();
    let mut events = Vec::new();

    w.reporting(&mut again, &mut events).update(yes()).unwrap();

    assert!(helper_runs(&again).is_empty(), "{:?}", again.ran);
    assert!(!phases(&events).contains(&Phase::Apps));
    assert!(summary(&events).apps.is_empty());
}

#[test]
fn an_update_without_picks_runs_nothing_for_the_apps() {
    let w = world();
    let mut m = fake(&[]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(!phases(&events).contains(&Phase::Apps));
    assert!(helper_runs(&m).is_empty());
}

#[test]
fn an_app_that_fails_to_install_warns_and_the_update_goes_on() {
    let w = world();
    put(&w.user.app_picks(), "apps = [\"agent\", \"gone\"]\n");
    let mut m = fake(&[]);
    let helper = format!("sudo {}", w.sys.apps_helper().display());
    m.failing.push(Box::leak(helper.into_boxed_str()));
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    let warned = warnings(&events);
    assert!(warned.iter().any(|w| w.starts_with("gone is picked")), "{warned:?}");
    assert!(warned.iter().any(|w| w.starts_with("apps not installed")), "{warned:?}");
    assert!(summary(&events).apps.is_empty());
}

#[test]
fn the_status_reads_the_catalog_the_picks_and_dpkg() {
    let w = world();
    put(&w.user.app_picks(), "apps = [\"agent\", \"editor\"]\n");
    let mut m = fake(&["desk"]);

    let status = w.updater(&mut m).app_status().unwrap();

    let rows: Vec<(&str, bool, AppState)> = status.apps.iter().map(|a| (a.id.as_str(), a.picked, a.state)).collect();
    assert_eq!(
        rows,
        [
            ("agent", true, AppState::Missing),
            ("desk", false, AppState::Installed),
            ("editor", true, AppState::Missing),
        ]
    );
    assert_eq!(m.ran.len(), 1, "one dpkg query, nothing else: {:?}", m.ran);
}

#[test]
fn removing_an_app_from_nixpkgs_leaves_a_package_the_user_had_added_themselves() {
    let w = world();
    put(&w.user.state_dir().packages_file(), "packages = [\"editor\"]\n");
    let mut m = fake(&[]);

    w.updater(&mut m).add_apps(&names(&["editor"])).unwrap();
    w.updater(&mut m).remove_apps(&names(&["editor"])).unwrap();

    let packages = fs::read_to_string(w.user.state_dir().packages_file()).unwrap();
    assert!(packages.contains("\"editor\""), "{packages}");
    assert!(picked(&w).is_empty());
}

#[test]
fn a_package_two_apps_share_goes_only_with_the_last_of_them() {
    let w = world();
    put(
        &w.sys.catalog().join("studio.toml"),
        "name = \"Studio\"\nsummary = \"An editor's studio\"\n\n[nix]\npackages = [\"editor\", \"studio\"]\n",
    );
    let mut m = fake(&[]);
    let packages = || fs::read_to_string(w.user.state_dir().packages_file()).unwrap();

    w.updater(&mut m).add_apps(&names(&["editor"])).unwrap();
    w.updater(&mut m).add_apps(&names(&["studio"])).unwrap();
    w.updater(&mut m).remove_apps(&names(&["editor"])).unwrap();

    assert!(packages().contains("\"editor\""), "{}", packages());

    w.updater(&mut m).remove_apps(&names(&["studio"])).unwrap();

    assert!(!packages().contains("\"editor\""), "{}", packages());
    assert!(!packages().contains("\"studio\""), "{}", packages());
}

#[test]
fn an_add_that_fails_after_its_packages_are_written_keeps_the_record_of_them() {
    let w = world();
    let mut m = fake(&[]);
    m.failing.push("switch");

    let added = w.updater(&mut m).add_apps(&names(&["editor"]));

    assert!(added.is_err());
    assert!(picked(&w).is_empty());
    let added = Added::read(&w.user.apps_added()).unwrap();
    assert_eq!(added.of("editor"), ["editor"]);
}

#[test]
fn the_record_of_added_packages_is_state_and_the_picks_stay_a_list() {
    let w = world();
    let mut m = fake(&[]);

    w.updater(&mut m).add_apps(&names(&["editor"])).unwrap();

    let picks = fs::read_to_string(w.user.app_picks()).unwrap();
    assert!(!picks.contains("[nix_added]"), "{picks}");
    assert_eq!(Added::read(&w.user.apps_added()).unwrap().of("editor"), ["editor"]);
    assert!(!w.user.apps_added().starts_with(w.user.config_home()));
}
