use std::fs;
use std::path::Path;

use super::Fake;
use super::FakeDpkg;
use super::World;
use super::notes;
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

/// An app with setups: its desktop app alone, or a service of the user's beside it.
const LAB: &str = r#"
name = "Lab"
summary = "An app for agents"

[deb]
package = "lab"
version = "2.0"
self_updates = true

[deb.amd64]
url = "https://example.com/lab-amd64.deb"
sha256 = "418c7825f73107b7b5ba3babae5e3b485801dc22d9dc0d518af632f7906cbdd1"

[setup.desktop]
name = "Lab"
summary = "The desktop app"

[setup.service]
name = "Lab as a service"
summary = "Its agents in a service"
user_hooks = "lab_service"
marker = ".config/systemd/user/lab.service"
"#;

/// Lab's service's marker, under the home.
const LAB_MARKER: &str = ".config/systemd/user/lab.service";

/// A stand-in for the user helper: it writes down how it was called, sets lab's service's marker
/// and leaves pairing to the user or takes the marker away, unless `$HOME/user-helper-fails` is
/// there, and says the setup can't go now while `$HOME/lab-open` is.
const USER_HELPER: &str = r#"
echo "$*" >>"$HOME/user-helper.log"
[[ ! -e $HOME/user-helper-fails ]] || exit 1
case "$1" in
  ready) [[ ! -e $HOME/lab-open ]] || { echo "Lab is open: quit it, then try again" >&2; exit 1; } ;;
  install) mkdir -p "$HOME/.config/systemd/user" && touch "$HOME/.config/systemd/user/lab.service"
    echo "user-helper: next: Run \`lab pair\`" ;;
  remove) rm -f "$HOME/.config/systemd/user/lab.service" ;;
esac
"#;

/// A world whose checked-out release has the catalog agent, desk, editor and lab, with a stand-in
/// user helper, and whose home.nix can be wired to kubaos.d/.
fn world() -> World {
    let w = World::new(Channel::Edge, true);
    for (id, text) in [("agent", AGENT), ("desk", DESK), ("editor", EDITOR), ("lab", LAB)] {
        put(&w.sys.catalog().join(format!("{id}.toml")), text);
    }
    put(&w.sys.apps_user_helper(), USER_HELPER);
    put(&w.user.home_nix(), "{\n}\n");
    w
}

/// The user helper's runs, as its stand-in wrote them down.
fn user_helper_runs(w: &World) -> Vec<String> {
    fs::read_to_string(w.user.home().join("user-helper.log"))
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect()
}

fn lab_service_there(w: &World) -> bool {
    w.user.home().join(LAB_MARKER).exists()
}

/// A machine with `installed`, whose helper installs each app's package.
fn fake(installed: &[&str]) -> Fake {
    let mut m = Fake::new();
    let mut dpkg = FakeDpkg::with(installed);
    dpkg.apps.insert("agent".into(), vec!["agent".into()]);
    dpkg.apps.insert("desk".into(), vec!["desk".into()]);
    dpkg.apps.insert("lab".into(), vec!["lab".into()]);
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
        .add_apps(&names(&["desk", "agent"]), None)
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

    w.reporting(&mut m, &mut events)
        .add_apps(&names(&["agent"]), None)
        .unwrap();

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

    let added = w.updater(&mut m).add_apps(&names(&["agent", "../etc"]), None);

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

    let added = w.updater(&mut m).add_apps(&names(&["agent"]), None);

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

    w.updater(&mut m).add_apps(&names(&["editor"]), None).unwrap();

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
            ("lab", false, AppState::Available),
        ]
    );
    assert_eq!(m.ran.len(), 1, "one dpkg query, nothing else: {:?}", m.ran);
}

#[test]
fn removing_an_app_from_nixpkgs_leaves_a_package_the_user_had_added_themselves() {
    let w = world();
    put(&w.user.state_dir().packages_file(), "packages = [\"editor\"]\n");
    let mut m = fake(&[]);

    w.updater(&mut m).add_apps(&names(&["editor"]), None).unwrap();
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

    w.updater(&mut m).add_apps(&names(&["editor"]), None).unwrap();
    w.updater(&mut m).add_apps(&names(&["studio"]), None).unwrap();
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

    let added = w.updater(&mut m).add_apps(&names(&["editor"]), None);

    assert!(added.is_err());
    assert!(picked(&w).is_empty());
    let added = Added::read(&w.user.apps_added()).unwrap();
    assert_eq!(added.of("editor"), ["editor"]);
}

#[test]
fn the_record_of_added_packages_is_state_and_the_picks_stay_a_list() {
    let w = world();
    let mut m = fake(&[]);

    w.updater(&mut m).add_apps(&names(&["editor"]), None).unwrap();

    let picks = fs::read_to_string(w.user.app_picks()).unwrap();
    assert!(!picks.contains("[nix_added]"), "{picks}");
    assert_eq!(Added::read(&w.user.apps_added()).unwrap().of("editor"), ["editor"]);
    assert!(!w.user.apps_added().starts_with(w.user.config_home()));
}

#[test]
fn adding_an_app_with_setups_without_one_refuses_and_names_them() {
    let w = world();
    let mut m = fake(&[]);

    let added = w.updater(&mut m).add_apps(&names(&["agent", "lab"]), None);

    let Err(err) = added else {
        panic!("added without a setup")
    };
    assert!(matches!(&err, UpdateError::NoSetup { id, setups } if id == "lab" && setups == &["desktop", "service"]));
    assert_eq!(
        err.to_string(),
        "lab can be set up more than one way, and none is the default: pick one with \
         kubaos app add lab --setup desktop, or kubaos app add lab --setup service"
    );
    assert!(picked(&w).is_empty());
    assert!(helper_runs(&m).is_empty() && user_helper_runs(&w).is_empty());
}

#[test]
fn a_setup_the_app_lacks_or_one_for_apps_without_setups_is_refused() {
    let w = world();
    let mut m = fake(&[]);

    let unknown = w.updater(&mut m).add_apps(&names(&["lab"]), Some("server"));
    let without = w.updater(&mut m).add_apps(&names(&["agent"]), Some("service"));

    assert!(matches!(unknown, Err(UpdateError::UnknownSetup { setup, .. }) if setup == "server"));
    assert!(matches!(without, Err(UpdateError::SetupWithout { setup }) if setup == "service"));
    assert!(picked(&w).is_empty());
    assert!(helper_runs(&m).is_empty());
}

#[test]
fn the_desktop_setup_is_the_app_alone_and_the_picks_record_it() {
    let w = world();
    let mut m = fake(&[]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events)
        .add_apps(&names(&["lab"]), Some("desktop"))
        .unwrap();

    assert!(helper_runs(&m)[0].ends_with("kubaos-apps install lab"), "{:?}", m.ran);
    assert!(user_helper_runs(&w).is_empty());
    let picks = Picks::read(&w.user.app_picks()).unwrap();
    assert_eq!(picks.setup("lab"), Some("desktop"));
    assert_eq!(summary(&events).apps.installed, ["Lab"]);
}

#[test]
fn the_service_setup_installs_the_app_then_its_own_parts_as_the_user() {
    let w = world();
    let mut m = fake(&[]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events)
        .add_apps(&names(&["lab"]), Some("service"))
        .unwrap();

    let runs = helper_runs(&m);
    assert!(runs.len() == 1 && runs[0].starts_with("pkexec "), "{runs:?}");
    let helper = format!(
        "bash {} install --window lab service",
        w.sys.apps_user_helper().display()
    );
    let at = |line: &str| m.ran.iter().position(|ran| ran.starts_with(line));
    assert!(
        at("pkexec").unwrap() < at(&helper).expect("the user helper ran, not as root"),
        "{:?}",
        m.ran
    );
    assert_eq!(user_helper_runs(&w), ["install --window lab service"]);
    assert!(lab_service_there(&w));
    assert_eq!(Picks::read(&w.user.app_picks()).unwrap().setup("lab"), Some("service"));
    assert_eq!(summary(&events).apps.installed, ["Lab as a service"]);
    assert_eq!(notes(&events), ["Run `lab pair`"]);
}

#[test]
fn switching_to_the_desktop_app_takes_the_service_away_and_keeps_the_app() {
    let w = world();
    put(&w.user.app_picks(), "apps = [\"lab\"]\n\n[setups]\nlab = \"service\"\n");
    put(&w.user.home().join(LAB_MARKER), "");
    let mut m = fake(&["lab"]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events)
        .add_apps(&names(&["lab"]), Some("desktop"))
        .unwrap();

    assert_eq!(user_helper_runs(&w), ["remove lab service"]);
    assert!(!lab_service_there(&w));
    assert!(helper_runs(&m).is_empty(), "the app stays: {:?}", m.ran);
    assert_eq!(Picks::read(&w.user.app_picks()).unwrap().setup("lab"), Some("desktop"));
    assert_eq!(summary(&events).apps.installed, ["Lab"]);
}

#[test]
fn switching_to_the_service_adds_only_its_parts() {
    let w = world();
    put(&w.user.app_picks(), "apps = [\"lab\"]\n\n[setups]\nlab = \"desktop\"\n");
    let mut m = fake(&["lab"]);

    w.updater(&mut m).add_apps(&names(&["lab"]), Some("service")).unwrap();

    assert_eq!(user_helper_runs(&w), ["install --window lab service"]);
    assert!(helper_runs(&m).is_empty(), "{:?}", m.ran);
    assert!(lab_service_there(&w));
}

#[test]
fn a_service_set_up_by_hand_is_that_setup_and_nothing_installs_over_it() {
    let w = world();
    put(&w.user.home().join(LAB_MARKER), "");
    let mut m = fake(&["lab"]);
    let mut events = Vec::new();

    let status = w.updater(&mut m).app_status().unwrap();
    w.reporting(&mut m, &mut events)
        .add_apps(&names(&["lab"]), Some("service"))
        .unwrap();

    let lab = status.apps.iter().find(|a| a.id == "lab").unwrap();
    assert_eq!(
        (lab.state, lab.setup.as_deref()),
        (AppState::Installed, Some("service"))
    );
    assert!(
        helper_runs(&m).is_empty() && user_helper_runs(&w).is_empty(),
        "{:?}",
        m.ran
    );
    assert_eq!(summary(&events).apps.installed, ["Lab as a service"]);
}

#[test]
fn removing_takes_the_app_then_the_setups_parts_and_drops_the_setup() {
    let w = world();
    put(&w.user.app_picks(), "apps = [\"lab\"]\n\n[setups]\nlab = \"service\"\n");
    put(&w.user.home().join(LAB_MARKER), "");
    let mut m = fake(&["lab"]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).remove_apps(&names(&["lab"])).unwrap();

    assert_eq!(user_helper_runs(&w), ["ready remove lab service", "remove lab service"]);
    let runs = helper_runs(&m);
    assert!(
        runs.len() == 1 && runs[0].ends_with("kubaos-apps remove lab"),
        "{runs:?}"
    );
    assert!(!lab_service_there(&w));
    let picks = fs::read_to_string(w.user.app_picks()).unwrap();
    assert!(!picks.contains("lab"), "{picks}");
    assert_eq!(summary(&events).apps.removed, ["Lab"]);
}

#[test]
fn a_setup_that_fails_to_set_up_puts_the_picks_back() {
    let w = world();
    put(&w.user.home().join("user-helper-fails"), "");
    let mut m = fake(&["lab"]);

    let added = w.updater(&mut m).add_apps(&names(&["lab"]), Some("service"));

    assert!(matches!(added, Err(UpdateError::Step { .. })), "{added:?}");
    assert!(picked(&w).is_empty());
}

#[test]
fn an_update_installs_a_picked_setups_missing_parts_and_warns_of_a_pick_without_one() {
    let w = world();
    put(&w.user.app_picks(), "apps = [\"lab\"]\n\n[setups]\nlab = \"service\"\n");
    let mut m = fake(&["lab"]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert_eq!(user_helper_runs(&w), ["install lab service"]);
    assert!(helper_runs(&m).is_empty(), "{:?}", m.ran);
    assert_eq!(summary(&events).apps.installed, ["Lab as a service"]);
    assert!(notes(&events).contains(&"Run `lab pair`"), "{:?}", notes(&events));

    let unsettled = world();
    put(&unsettled.user.app_picks(), "apps = [\"lab\"]\n");
    let mut m = fake(&[]);
    let mut events = Vec::new();

    unsettled.reporting(&mut m, &mut events).update(yes()).unwrap();

    let warned = warnings(&events);
    assert!(
        warned.contains(
            &"lab is picked with no setup, so nothing installs it: kubaos app add lab --setup <desktop|service>"
        ),
        "{warned:?}"
    );
    assert!(helper_runs(&m).is_empty() && user_helper_runs(&unsettled).is_empty());
}

#[test]
fn removing_the_root_half_first_leaves_the_setup_whole_when_polkit_is_declined() {
    let w = world();
    put(&w.user.app_picks(), "apps = [\"lab\"]\n\n[setups]\nlab = \"service\"\n");
    put(&w.user.home().join(LAB_MARKER), "");
    let mut m = fake(&["lab"]);
    m.failing.push("pkexec");

    let removed = w.updater(&mut m).remove_apps(&names(&["lab"]));

    assert!(matches!(removed, Err(UpdateError::Step { .. })), "{removed:?}");
    assert!(lab_service_there(&w));
    assert_eq!(user_helper_runs(&w), ["ready remove lab service"]);
    assert_eq!(picked(&w), ["lab"]);
}

#[test]
fn a_setup_that_cant_go_now_refuses_the_removal_before_root_is_asked() {
    let w = world();
    put(&w.user.app_picks(), "apps = [\"lab\"]\n\n[setups]\nlab = \"service\"\n");
    put(&w.user.home().join(LAB_MARKER), "");
    put(&w.user.home().join("lab-open"), "");
    let mut m = fake(&["lab"]);

    let removed = w.updater(&mut m).remove_apps(&names(&["lab"]));

    assert!(matches!(removed, Err(UpdateError::Step { .. })), "{removed:?}");
    assert!(helper_runs(&m).is_empty(), "{:?}", m.ran);
    assert!(lab_service_there(&w));
    assert_eq!(picked(&w), ["lab"]);
}
