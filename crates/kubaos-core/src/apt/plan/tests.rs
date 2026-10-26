use std::collections::BTreeSet;

use super::Facts;
use super::PackageState;
use super::Status;
use super::Want;
use super::missing;
use super::removable;
use super::repos_for;
use super::status_candidates;
use super::to_hold;
use crate::apt::choices::Choices;
use crate::apt::choices::Switch;
use crate::apt::dpkg::Dpkg;
use crate::apt::manifest::Manifest;
use crate::apt::record::Record;

const MANIFEST: &str = r#"
[repo.docker]
key_url = "https://example.com/gpg"
fingerprint = "AB"
keyring = "/etc/apt/keyrings/docker.gpg"

[group.tools]
summary = "Tools"
packages = ["curl", "git"]

[group.nix]
summary = "Nix"
packages = ["curl", "xz-utils"]

[group.docker]
summary = "Docker"
repo = "docker"
optional = "on"
packages = ["docker-ce", "containerd.io"]

[group.chrome]
summary = "Chrome"
arch = "amd64"
packages = ["google-chrome-stable"]

[group.hyprland]
summary = "Hyprland"
hold = true
if_available = true
packages = ["hyprland", "hyprland-guiutils"]
"#;

/// A machine's facts, owned, so a test can lend them out.
struct Machine {
    manifest: Manifest,
    choices: Choices,
    record: Record,
    arch: &'static str,
    dpkg: Dpkg,
}

impl Machine {
    fn new(dpkg: &str) -> Self {
        Self {
            manifest: Manifest::parse(MANIFEST).unwrap(),
            choices: Choices::default(),
            record: Record::default(),
            arch: "amd64",
            dpkg: Dpkg::parse(dpkg),
        }
    }

    fn facts(&self) -> Facts<'_> {
        Facts {
            manifest: &self.manifest,
            choices: &self.choices,
            record: &self.record,
            arch: self.arch,
            dpkg: &self.dpkg,
        }
    }
}

fn want(package: &str, group: &str) -> Want {
    Want {
        package: package.into(),
        group: group.into(),
    }
}

#[test]
fn missing_lists_each_wanted_package_once_under_its_first_group() {
    let m = Machine::new("installed install git 1\n");

    let wants = missing(&m.facts());

    assert_eq!(
        wants,
        [
            want("curl", "tools"),
            want("xz-utils", "nix"),
            want("docker-ce", "docker"),
            want("containerd.io", "docker"),
            want("google-chrome-stable", "chrome"),
            want("hyprland", "hyprland"),
            want("hyprland-guiutils", "hyprland"),
        ]
    );
    assert_eq!(repos_for(&m.manifest, &wants), ["docker"]);
}

#[test]
fn a_group_turned_off_or_for_another_architecture_wants_nothing() {
    let mut m = Machine::new("");
    m.choices.set("docker", Switch::Off);
    m.arch = "arm64";

    let packages: Vec<String> = missing(&m.facts()).into_iter().map(|w| w.package).collect();

    assert!(!packages.contains(&"docker-ce".to_owned()));
    assert!(!packages.contains(&"google-chrome-stable".to_owned()));
    assert!(packages.contains(&"curl".to_owned()));
}

#[test]
fn installed_packages_of_a_held_group_are_held_once() {
    let m = Machine::new("installed install hyprland 1\ninstalled hold hyprland-guiutils 1\n");

    assert_eq!(to_hold(&m.facts()), ["hyprland"]);
}

#[test]
fn only_a_package_kubaos_installed_and_no_group_lists_is_removable() {
    let mut m = Machine::new(
        "installed install old-tool 1\n\
         installed install users-tool 1\n\
         installed install curl 1\n\
         installed install docker-ce 1\n",
    );
    m.record.add("old-tool", "tools");
    m.record.add("curl", "tools");
    m.record.add("docker-ce", "docker");
    m.record.add("gone-already", "tools");

    assert_eq!(removable(&m.facts(), None), ["old-tool"]);
}

#[test]
fn a_group_turned_off_is_removable_only_while_it_is_being_turned_off() {
    let mut m = Machine::new("installed install docker-ce 1\ninstalled install curl 1\n");
    m.record.add("docker-ce", "docker");
    m.record.add("curl", "tools");
    m.choices.set("docker", Switch::Off);

    assert!(removable(&m.facts(), None).is_empty());
    assert_eq!(removable(&m.facts(), Some("docker")), ["docker-ce"]);
}

#[test]
fn a_package_another_active_group_lists_stays_when_its_group_turns_off() {
    let mut m = Machine::new("installed install curl 1\n");
    m.record.add("curl", "nix");
    m.manifest = Manifest::parse(&MANIFEST.replace("[group.nix]", "[group.nix]\noptional = \"on\"")).unwrap();
    m.choices.set("nix", Switch::Off);

    assert!(removable(&m.facts(), Some("nix")).is_empty());
}

#[test]
fn the_status_names_each_packages_state_version_and_whether_kubaos_installed_it() {
    let mut m = Machine::new(
        "installed install curl 8.1\n\
         installed hold hyprland 0.56\n\
         installed install old-tool 2.0\n",
    );
    m.record.add("curl", "tools");
    m.record.add("old-tool", "files");
    m.choices.set("docker", Switch::Off);
    let facts = m.facts();
    assert_eq!(status_candidates(&facts), ["hyprland-guiutils"]);

    let status = Status::new(&facts, &BTreeSet::new());

    let tools = &status.groups[0];
    assert_eq!((tools.name.as_str(), tools.active), ("tools", true));
    assert_eq!(tools.packages[0].state, PackageState::Installed);
    assert_eq!(tools.packages[0].version.as_deref(), Some("8.1"));
    assert!(tools.packages[0].kubaos);
    assert_eq!(tools.packages[1].state, PackageState::Missing);
    assert!(!tools.packages[1].kubaos);
    let docker = &status.groups[2];
    assert_eq!(
        (docker.active, docker.inactive.as_deref(), docker.optional),
        (false, Some("off"), Some(Switch::On))
    );
    assert_eq!(docker.packages[0].state, PackageState::Skipped);
    let hyprland = &status.groups[4];
    assert_eq!(hyprland.packages[0].state, PackageState::Held);
    assert_eq!(hyprland.packages[1].state, PackageState::Unavailable);
    assert_eq!(status.extra.len(), 1);
    assert_eq!(
        (
            status.extra[0].name.as_str(),
            status.extra[0].group.as_str(),
            status.extra[0].state
        ),
        ("old-tool", "files", PackageState::Extra)
    );
}

#[test]
fn a_package_of_a_group_for_another_architecture_is_skipped() {
    let mut m = Machine::new("");
    m.arch = "arm64";

    let status = Status::new(&m.facts(), &BTreeSet::new());

    let chrome = status.groups.iter().find(|g| g.name == "chrome").unwrap();
    assert_eq!(chrome.inactive.as_deref(), Some("amd64 only"));
    assert_eq!(chrome.packages[0].state, PackageState::Skipped);
}

#[test]
fn the_status_serializes_as_the_menu_reads_it() {
    let m = Machine::new("installed install curl 8.1\n");

    let json = serde_json::to_value(Status::new(&m.facts(), &BTreeSet::new())).unwrap();

    assert_eq!(json["arch"], "amd64");
    assert_eq!(json["groups"][0]["packages"][0]["state"], "installed");
    assert_eq!(json["groups"][2]["optional"], "on");
    assert!(json["groups"][0].get("inactive").is_none());
}
