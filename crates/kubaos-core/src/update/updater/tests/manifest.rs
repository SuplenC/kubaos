use std::fs;
use std::path::Path;

use super::Fake;
use super::FakeDpkg;
use super::World;
use super::phases;
use super::summary;
use super::warnings;
use super::yes;
use crate::apt::Choices;
use crate::apt::Record;
use crate::apt::Switch;
use crate::apt::plan::PackageState;
use crate::update::UpdateError;
use crate::update::event::GroupChange;
use crate::update::event::Phase;
use crate::update::event::Preview;
use crate::update::event::UpdateEvent;
use crate::update::record::Channel;

const MANIFEST: &str = "system/packages.toml";

/// A manifest with `tools` listing `tools`, and the optional groups docker (a vendor repo) and
/// u2f (scripts to turn it on and off).
fn manifest(tools: &[&str]) -> String {
    let tools: Vec<String> = tools.iter().map(|p| format!("{p:?}")).collect();
    format!(
        r#"
[repo.docker]
key_url = "https://example.com/gpg"
fingerprint = "AB"
keyring = "/etc/apt/keyrings/docker.gpg"

[group.tools]
summary = "Tools"
packages = [{}]

[group.docker]
summary = "Docker"
repo = "docker"
optional = "on"
packages = ["docker-ce"]

[group.u2f]
summary = "U2F"
optional = "on"
when_on = ["system/u2f-on.sh", "--now"]
when_off = ["system/u2f-off.sh"]
packages = ["libpam-u2f"]
"#,
        tools.join(", ")
    )
}

/// A fake whose packages are `installed`.
fn fake(installed: &[&str]) -> Fake {
    let mut m = Fake::new();
    m.dpkg = Some(FakeDpkg::with(installed));
    m
}

fn recorded(w: &World) -> Record {
    Record::parse(&fs::read_to_string(w.sys.apt_record()).unwrap_or_default())
}

/// Writes `text` to `path`, making its folder.
fn put(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// The lines `m` ran that start with `prefix`.
fn ran<'a>(m: &'a Fake, prefix: &str) -> Vec<&'a str> {
    m.ran
        .iter()
        .filter(|line| line.starts_with(prefix))
        .map(String::as_str)
        .collect()
}

const INSTALL: &str = "sudo apt-get install -y --no-install-recommends \
    -o Dpkg::Options::=--force-confdef -o Dpkg::Options::=--force-confold";
const REMOVE: &str = "sudo apt-get remove -y -o Dpkg::Options::=--force-confdef -o Dpkg::Options::=--force-confold";

fn change(group: &str, installed: &[&str], removed: &[&str]) -> GroupChange {
    GroupChange {
        group: group.into(),
        installed: installed.iter().map(|p| (*p).to_owned()).collect(),
        removed: removed.iter().map(|p| (*p).to_owned()).collect(),
    }
}

#[test]
fn a_package_added_to_the_manifest_is_installed_by_the_next_update_and_recorded() {
    let w = World::new(Channel::Edge, true);
    w.push(&[(MANIFEST, &manifest(&["curl", "git"]))], true);
    let mut m = fake(&["curl", "libpam-u2f"]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert_eq!(
        ran(&m, "sudo /"),
        [format!("sudo {} docker", w.sys.apt_repo_script().display())]
    );
    assert_eq!(ran(&m, "sudo apt-get install"), [format!("{INSTALL} git docker-ce")]);
    let record = recorded(&w);
    assert_eq!(
        (record.group_of("git"), record.group_of("docker-ce")),
        (Some("tools"), Some("docker"))
    );
    assert!(
        !record.has("curl") && !record.has("libpam-u2f"),
        "they were there before"
    );
    assert!(phases(&events).contains(&Phase::Manifest));
    assert_eq!(
        summary(&events).apt,
        [change("docker", &["docker-ce"], &[]), change("tools", &["git"], &[])]
    );

    w.push(&[(MANIFEST, &manifest(&["curl", "git", "jq"]))], true);
    let mut again = Fake::new();
    again.dpkg = m.dpkg.take();
    w.updater(&mut again).update(yes()).unwrap();

    assert_eq!(ran(&again, "sudo apt-get install"), [format!("{INSTALL} jq")]);
    assert!(
        ran(&again, "sudo /").is_empty(),
        "docker's repo is only set up to install from it"
    );
    assert!(recorded(&w).has("jq") && recorded(&w).has("git"));
}

#[test]
fn a_package_dropped_from_the_manifest_goes_only_when_kubaos_installed_it() {
    let w = World::new(Channel::Edge, true);
    w.push(&[(MANIFEST, &manifest(&["curl", "old-tool", "users-tool"]))], true);
    let mut m = fake(&["users-tool", "docker-ce", "libpam-u2f"]);
    w.updater(&mut m).update(yes()).unwrap();
    assert!(recorded(&w).has("old-tool") && !recorded(&w).has("users-tool"));

    w.push(&[(MANIFEST, &manifest(&["curl"]))], true);
    let mut again = Fake::new();
    again.dpkg = m.dpkg.take();
    let mut events = Vec::new();
    w.reporting(&mut again, &mut events).update(yes()).unwrap();

    assert_eq!(ran(&again, "sudo apt-get remove"), [format!("{REMOVE} old-tool")]);
    let dpkg = again.dpkg.as_ref().unwrap();
    assert!(!dpkg.installed.contains_key("old-tool"));
    assert!(
        dpkg.installed.contains_key("users-tool"),
        "the user's package is never touched"
    );
    assert!(!recorded(&w).has("old-tool"));
    assert_eq!(summary(&events).apt, [change("tools", &[], &["old-tool"])]);
}

#[test]
fn a_dropped_package_something_wanted_depends_on_stays() {
    let w = World::new(Channel::Edge, true);
    w.push(&[(MANIFEST, &manifest(&["curl"]))], true);
    put(&w.sys.apt_record(), "old-lib\ttools\ncurl\ttools\n");
    let mut m = fake(&["curl", "old-lib", "docker-ce", "libpam-u2f"]);
    m.dpkg
        .as_mut()
        .unwrap()
        .takes
        .insert("old-lib".into(), vec!["curl".into()]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(ran(&m, "sudo apt-get remove").is_empty());
    assert!(recorded(&w).has("old-lib"));
    assert!(
        warnings(&events).iter().any(|w| w.starts_with("kept old-lib")),
        "{events:?}"
    );
}

#[test]
fn a_group_left_out_at_install_stays_out_through_updates() {
    let w = World::new(Channel::Edge, true);
    w.push(&[(MANIFEST, &manifest(&["curl"]))], true);
    put(&w.sys.apt_choices(), "docker=off\n");
    let mut m = fake(&["curl", "libpam-u2f"]);

    w.updater(&mut m).update(yes()).unwrap();
    let mut again = Fake::new();
    again.dpkg = m.dpkg.take();
    w.updater(&mut again).update(yes()).unwrap();

    assert!(ran(&m, "sudo apt-get install").is_empty() && ran(&again, "sudo apt-get install").is_empty());
    assert!(ran(&m, "sudo /").is_empty(), "no repo for a group that is off");
}

#[test]
fn a_group_left_out_keeps_what_the_user_installed_of_it() {
    let w = World::new(Channel::Edge, true);
    w.push(&[(MANIFEST, &manifest(&["curl"]))], true);
    put(&w.sys.apt_choices(), "docker=off\n");
    put(&w.sys.apt_record(), "docker-ce\tdocker\n");
    let mut m = fake(&["curl", "docker-ce", "libpam-u2f"]);

    w.updater(&mut m).update(yes()).unwrap();

    assert!(
        ran(&m, "sudo apt-get remove").is_empty(),
        "only kubaos packages disable removes it"
    );
}

#[test]
fn a_held_group_is_unheld_to_install_and_held_after() {
    let w = World::new(Channel::Edge, true);
    let held = manifest(&["curl"]).replace("summary = \"Tools\"", "summary = \"Tools\"\nhold = true");
    w.push(&[(MANIFEST, &held)], true);
    let mut m = fake(&["docker-ce", "libpam-u2f"]);
    m.dpkg.as_mut().unwrap().held_missing.push("curl".into());

    w.updater(&mut m).update(yes()).unwrap();

    let marks: Vec<&str> = ran(&m, "sudo apt-mark");
    assert_eq!(marks, ["sudo apt-mark unhold curl", "sudo apt-mark hold curl"]);
    assert_eq!(m.dpkg.as_ref().unwrap().installed.get("curl"), Some(&true));
}

#[test]
fn a_package_apt_has_none_of_is_reported_and_the_rest_installed() {
    let w = World::new(Channel::Edge, true);
    w.push(&[(MANIFEST, &manifest(&["curl", "gone-upstream"]))], true);
    let mut m = fake(&["docker-ce", "libpam-u2f"]);
    m.dpkg.as_mut().unwrap().unavailable.push("gone-upstream".into());
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert_eq!(ran(&m, "sudo apt-get install"), [format!("{INSTALL} curl")]);
    assert!(warnings(&events).contains(&"apt has no gone-upstream (group tools): not installed"));
}

#[test]
fn packages_only_brings_the_installed_release_in_line_with_its_manifest() {
    let w = World::new(Channel::Edge, true);
    let head = w.push(&[(MANIFEST, &manifest(&["curl"]))], true);
    let mut first = fake(&["docker-ce", "libpam-u2f"]);
    w.updater(&mut first).update(yes()).unwrap();
    w.push(&[(MANIFEST, &manifest(&["curl", "jq"]))], true);
    let mut m = Fake::new();
    m.dpkg = first.dpkg.take();

    w.updater(&mut m).update(yes().packages_only()).unwrap();

    assert_eq!(w.release(), head);
    assert!(
        ran(&m, "sudo apt-get install").is_empty(),
        "jq is in the next release, not this one"
    );
}

#[test]
fn turning_a_group_off_runs_its_script_then_removes_what_kubaos_installed_for_it() {
    let w = World::new(Channel::Edge, true);
    w.push(&[(MANIFEST, &manifest(&["curl"]))], true);
    let mut first = fake(&["curl", "docker-ce"]);
    w.updater(&mut first).update(yes()).unwrap();
    assert!(recorded(&w).has("libpam-u2f"));
    let mut m = Fake::new();
    m.dpkg = first.dpkg.take();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).set_group("u2f", Switch::Off).unwrap();

    let script = w.sys.checkout().join("system/u2f-off.sh").display().to_string();
    let order: Vec<&str> = m
        .ran
        .iter()
        .filter(|l| l.starts_with(&script) || l.starts_with("sudo apt-get remove"))
        .map(String::as_str)
        .collect();
    assert_eq!(order, [script.clone(), format!("{REMOVE} libpam-u2f")]);
    assert_eq!(
        Choices::parse(&fs::read_to_string(w.sys.apt_choices()).unwrap()).get("u2f"),
        Some(Switch::Off)
    );
    assert!(!recorded(&w).has("libpam-u2f"));
    assert!(matches!(&events[0], UpdateEvent::Started { log: Some(_), .. }));
    assert_eq!(summary(&events).apt, [change("u2f", &[], &["libpam-u2f"])]);

    let mut later = Fake::new();
    later.dpkg = m.dpkg.take();
    w.updater(&mut later).update(yes()).unwrap();
    assert!(
        ran(&later, "sudo apt-get install").is_empty(),
        "an update leaves it off"
    );
}

#[test]
fn turning_a_group_on_installs_it_then_runs_its_script() {
    let w = World::new(Channel::Edge, true);
    w.push(&[(MANIFEST, &manifest(&["curl"]))], true);
    let mut first = fake(&["curl", "docker-ce"]);
    w.updater(&mut first).update(yes()).unwrap();
    put(&w.sys.apt_choices(), "u2f=off\n");
    let mut m = fake(&["curl", "docker-ce"]);

    w.updater(&mut m).set_group("u2f", Switch::On).unwrap();

    let script = w.sys.checkout().join("system/u2f-on.sh").display().to_string();
    let order: Vec<&str> = m
        .ran
        .iter()
        .filter(|l| l.starts_with(&script) || l.starts_with("sudo apt-get install"))
        .map(String::as_str)
        .collect();
    assert_eq!(order, [format!("{INSTALL} libpam-u2f"), format!("{script} --now")]);
    assert_eq!(
        Choices::parse(&fs::read_to_string(w.sys.apt_choices()).unwrap()).get("u2f"),
        Some(Switch::On)
    );
}

#[test]
fn turning_a_group_off_keeps_what_kubaos_didnt_install_and_says_so() {
    let w = World::new(Channel::Edge, true);
    w.push(&[(MANIFEST, &manifest(&["curl"]))], true);
    let mut first = fake(&["curl", "docker-ce", "libpam-u2f"]);
    w.updater(&mut first).update(yes()).unwrap();
    let mut m = Fake::new();
    m.dpkg = first.dpkg.take();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events)
        .set_group("docker", Switch::Off)
        .unwrap();

    assert!(ran(&m, "sudo apt-get remove").is_empty());
    assert!(
        warnings(&events).contains(
            &"kept docker-ce: KubaOS didn't install them, so it leaves them to you (sudo apt remove docker-ce)"
        ),
        "{events:?}"
    );
}

#[test]
fn a_failed_script_puts_the_old_choice_back() {
    let w = World::new(Channel::Edge, true);
    w.push(&[(MANIFEST, &manifest(&["curl"]))], true);
    let mut first = fake(&["curl", "docker-ce", "libpam-u2f"]);
    w.updater(&mut first).update(yes()).unwrap();
    let mut m = fake(&["curl", "docker-ce", "libpam-u2f"]);
    let script = w.sys.checkout().join("system/u2f-off.sh").display().to_string();
    m.failing.push(Box::leak(script.into_boxed_str()));

    let err = w.updater(&mut m).set_group("u2f", Switch::Off).unwrap_err();

    assert!(matches!(err, UpdateError::Step { .. }), "{err:?}");
    assert_eq!(fs::read_to_string(w.sys.apt_choices()).unwrap_or_default(), "");
    assert!(ran(&m, "sudo apt-get remove").is_empty());
}

#[test]
fn only_an_optional_group_can_be_turned_on_or_off() {
    let w = World::new(Channel::Edge, true);
    w.push(&[(MANIFEST, &manifest(&["curl"]))], true);
    w.updater(&mut fake(&[])).update(yes()).unwrap();

    let not_optional = w.updater(&mut fake(&[])).set_group("tools", Switch::Off).unwrap_err();
    let missing = w.updater(&mut fake(&[])).set_group("nope", Switch::On).unwrap_err();

    assert!(matches!(not_optional, UpdateError::NotOptional { .. }));
    assert!(matches!(missing, UpdateError::NoGroup { .. }));
}

#[test]
fn the_status_reports_what_dpkg_has_of_each_group() {
    let w = World::new(Channel::Edge, true);
    w.push(&[(MANIFEST, &manifest(&["curl", "git"]))], true);
    w.updater(&mut fake(&[])).update(yes()).unwrap();
    put(&w.sys.apt_choices(), "docker=off\n");
    put(&w.sys.apt_record(), "curl\ttools\nold-tool\ttools\n");
    let mut m = fake(&["curl", "old-tool", "libpam-u2f"]);
    m.dpkg.as_mut().unwrap().installed.insert("libpam-u2f".into(), true);

    let status = w.updater(&mut m).package_status().unwrap();

    let seen: Vec<(&str, PackageState)> = status
        .groups
        .iter()
        .flat_map(|g| g.packages.iter().map(|p| (p.name.as_str(), p.state)))
        .collect();
    assert_eq!(
        seen,
        [
            ("curl", PackageState::Installed),
            ("git", PackageState::Missing),
            ("docker-ce", PackageState::Skipped),
            ("libpam-u2f", PackageState::Held),
        ]
    );
    assert_eq!(status.extra[0].name, "old-tool");
    assert_eq!(status.arch, "amd64");
}

#[test]
fn a_release_without_a_manifest_has_no_status() {
    let w = World::new(Channel::Edge, true);

    let err = w.updater(&mut fake(&[])).package_status().unwrap_err();

    assert!(matches!(err, UpdateError::NoManifest { .. }));
}

fn preview(events: &[UpdateEvent]) -> &Preview {
    events
        .iter()
        .find_map(|e| match e {
            UpdateEvent::Preview(preview) => Some(preview),
            _ => None,
        })
        .expect("a preview")
}

#[test]
fn the_preview_lists_what_the_manifest_phase_will_install_by_group() {
    let w = World::new(Channel::Edge, true);
    w.push(&[(MANIFEST, &manifest(&["curl", "git", "gone-upstream"]))], true);
    let mut m = fake(&["curl", "libpam-u2f"]);
    // apt sees docker-ce only once the phase has set up docker's repository.
    m.dpkg
        .as_mut()
        .unwrap()
        .unavailable
        .extend(["gone-upstream".into(), "docker-ce".into()]);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert_eq!(
        preview(&events).apt,
        [change("docker", &["docker-ce"], &[]), change("tools", &["git"], &[])]
    );
}

#[test]
fn a_packages_only_preview_lists_the_installed_releases_manifest() {
    let w = World::new(Channel::Edge, true);
    w.push(&[(MANIFEST, &manifest(&["curl"]))], true);
    let mut first = fake(&["docker-ce", "libpam-u2f"]);
    let mut events = Vec::new();
    w.reporting(&mut first, &mut events).update(yes()).unwrap();
    assert_eq!(preview(&events).apt, [change("tools", &["curl"], &[])]);
    w.push(&[(MANIFEST, &manifest(&["curl", "jq"]))], true);
    let mut m = Fake::new();
    m.dpkg = first.dpkg.take();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes().packages_only()).unwrap();

    assert!(
        preview(&events).apt.is_empty(),
        "jq is in the next release, not this one"
    );
}
