use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use super::CurrentHome;
use super::Decision;
use super::Evaluated;
use super::Found;
use super::HeldFile;
use super::Holds;
use super::Prebuilt;
use super::decide;
use super::message;
use super::package_name;
use super::prebuilt;
use crate::state::StateDir;

const DISCORD_OLD: &str = "/nix/store/84k7g556fj4ib11413jna62l3hgfzyna-discord-1.0.159";
const DISCORD_NEW: &str = "/nix/store/gkm5zlch3916yfzbswzas38d5ysvl917-discord-1.0.159";

fn found(drv: &str, outputs: &[(&str, &str)], pname: &str) -> Found {
    Found {
        drv: drv.to_owned(),
        outputs: outputs
            .iter()
            .map(|(o, p)| ((*o).to_owned(), (*p).to_owned()))
            .collect(),
        pname: pname.to_owned(),
    }
}

fn evaluated(name: &str, found: Found) -> Evaluated {
    Evaluated {
        name: name.to_owned(),
        found: Some(found),
    }
}

fn package(name: &str, built: bool, pname: Option<&str>) -> Prebuilt {
    Prebuilt {
        name: name.to_owned(),
        built,
        pname: pname.map(str::to_owned),
    }
}

fn holds(pairs: &[(&str, &str)]) -> Holds {
    pairs.iter().map(|(n, p)| ((*n).to_owned(), (*p).to_owned())).collect()
}

fn recorded(pairs: &[(&str, &str)]) -> CurrentHome {
    CurrentHome::Recorded(pairs.iter().map(|(n, p)| ((*n).to_owned(), (*p).to_owned())).collect())
}

#[test]
fn the_message_names_the_package_as_listed() {
    assert_eq!(
        message("discord"),
        "discord stays at its current version: it couldn't be built"
    );
}

#[test]
fn a_derivation_builds_the_outputs_the_home_installs() {
    let found = found(
        "/nix/store/k8a-postgresql-17.12.drv",
        &[
            ("out", "/nix/store/0my-postgresql-17.12"),
            ("man", "/nix/store/hn3-postgresql-17.12-man"),
        ],
        "postgresql",
    );

    assert_eq!(found.installable(), "/nix/store/k8a-postgresql-17.12.drv^man,out");
}

#[test]
fn a_package_built_when_none_of_its_outputs_is_missing() {
    let listed = [
        evaluated(
            "discord",
            found("/nix/store/b2q-discord.drv", &[("out", DISCORD_NEW)], "discord"),
        ),
        evaluated(
            "jq",
            found("/nix/store/j-jq.drv", &[("bin", "/nix/store/j-jq-bin")], "jq"),
        ),
    ];

    let prebuilt = prebuilt(&listed, &BTreeSet::from([DISCORD_NEW.to_owned()]));

    assert_eq!(
        prebuilt,
        [
            package("discord", false, Some("discord")),
            package("jq", true, Some("jq"))
        ]
    );
}

#[test]
fn a_package_that_does_not_evaluate_did_not_build() {
    let prebuilt = prebuilt(
        &[Evaluated {
            name: "gone".to_owned(),
            found: None,
        }],
        &BTreeSet::new(),
    );

    assert_eq!(prebuilt, [package("gone", false, None)]);
}

#[test]
fn a_package_of_two_home_configurations_built_only_when_it_built_for_both() {
    let listed = [
        evaluated("jq", found("/nix/store/a-jq.drv", &[("out", "/nix/store/a-jq")], "jq")),
        evaluated("jq", found("/nix/store/b-jq.drv", &[("out", "/nix/store/b-jq")], "jq")),
    ];

    let prebuilt = prebuilt(&listed, &BTreeSet::from(["/nix/store/b-jq".to_owned()]));

    assert_eq!(prebuilt, [package("jq", false, Some("jq"))]);
}

#[test]
fn a_package_that_failed_and_that_the_home_has_is_held_at_its_store_path() {
    let prebuilt = [
        package("discord", false, Some("discord")),
        package("jq", true, Some("jq")),
    ];
    let current = recorded(&[("discord", DISCORD_OLD), ("jq", "/nix/store/j-jq")]);

    let decision = decide(&prebuilt, &current);

    assert_eq!(
        decision,
        Decision {
            held: holds(&[("discord", DISCORD_OLD)]),
            unkept: Vec::new()
        }
    );
}

#[test]
fn a_package_that_failed_and_that_the_home_lacks_is_not_held() {
    let prebuilt = [package("discord", false, Some("discord"))];

    for current in [recorded(&[("jq", "/nix/store/j-jq")]), CurrentHome::Nothing] {
        let decision = decide(&prebuilt, &current);

        assert!(decision.held.is_empty(), "{current:?}");
        assert_eq!(decision.unkept, ["discord"]);
    }
}

#[test]
fn a_package_that_builds_is_never_held() {
    let prebuilt = [package("discord", true, Some("discord"))];

    let decision = decide(&prebuilt, &recorded(&[("discord", DISCORD_OLD)]));

    assert_eq!(decision, Decision::default());
}

#[test]
fn a_generation_without_the_record_has_the_one_path_of_the_same_package() {
    let current = CurrentHome::Unrecorded(vec![
        "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-discord-ptb-0.0.170".to_owned(),
        DISCORD_OLD.to_owned(),
        "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-hm-session-vars.sh".to_owned(),
    ]);

    let decision = decide(&[package("discord", false, Some("discord"))], &current);

    assert_eq!(decision.held, holds(&[("discord", DISCORD_OLD)]));
}

#[test]
fn a_generation_without_the_record_and_two_versions_of_a_package_holds_neither() {
    let current = CurrentHome::Unrecorded(vec![
        DISCORD_OLD.to_owned(),
        "/nix/store/cccccccccccccccccccccccccccccccc-discord-1.0.158".to_owned(),
    ]);

    let decision = decide(&[package("discord", false, Some("discord"))], &current);

    assert!(decision.held.is_empty());
    assert_eq!(decision.unkept, ["discord"]);
}

#[test]
fn a_store_name_loses_its_version_as_nix_parses_it() {
    assert_eq!(package_name("discord-1.0.159"), "discord");
    assert_eq!(package_name("discord-ptb-0.0.170"), "discord-ptb");
    assert_eq!(package_name("yubioath-flutter-7.3.3"), "yubioath-flutter");
    assert_eq!(package_name("hm-session-vars.sh"), "hm-session-vars.sh");
}

fn held_file(dir: &Path) -> HeldFile {
    HeldFile::new(&StateDir::new(dir.join("home/kubaos.d")), dir.join("kubaos.d.lock"))
}

#[test]
fn a_missing_file_holds_nothing() {
    let tmp = tempfile::tempdir().unwrap();

    assert!(held_file(tmp.path()).read().unwrap().is_empty());
}

#[test]
fn the_holds_written_read_back_under_a_header() {
    let tmp = tempfile::tempdir().unwrap();
    let file = held_file(tmp.path());
    let holds = holds(&[
        ("discord", DISCORD_OLD),
        ("python3Packages.black", "/nix/store/b-black-24"),
    ]);

    let changed = file.write(&holds).unwrap();

    assert!(changed);
    assert_eq!(file.read().unwrap(), holds);
    let text = fs::read_to_string(file.state().held_file()).unwrap();
    assert!(text.starts_with("# Packages that couldn't be built"), "{text}");
    assert!(text.contains("[held]\n"), "{text}");
}

#[test]
fn the_same_holds_again_write_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let file = held_file(tmp.path());
    let holds = holds(&[("discord", DISCORD_OLD)]);
    file.write(&holds).unwrap();

    assert!(!file.write(&holds).unwrap());
}

#[test]
fn the_last_hold_gone_removes_the_file() {
    let tmp = tempfile::tempdir().unwrap();
    let file = held_file(tmp.path());
    file.write(&holds(&[("discord", DISCORD_OLD)])).unwrap();

    let changed = file.write(&Holds::new()).unwrap();

    assert!(changed);
    assert!(!file.state().held_file().exists());
    assert!(!file.write(&Holds::new()).unwrap());
}

#[test]
fn a_hold_written_in_a_flake_under_git_is_in_the_index_and_leaves_it_when_removed() {
    let tmp = tempfile::tempdir().unwrap();
    let flake = tmp.path().join("home");
    fs::create_dir_all(flake.join("kubaos.d")).unwrap();
    fs::write(flake.join("flake.nix"), "{ }\n").unwrap();
    crate::sandbox::git(&flake, &["init", "-q"]);
    let file = held_file(tmp.path());
    let tracked = || crate::sandbox::git(&flake, &["ls-files"]);

    file.write(&holds(&[("discord", DISCORD_OLD)])).unwrap();
    let with_hold = tracked();
    file.write(&Holds::new()).unwrap();

    assert!(with_hold.contains("kubaos.d/held.toml"), "{with_hold}");
    assert!(!tracked().contains("kubaos.d/held.toml"));
}

#[test]
fn a_record_reads_as_each_listed_package_to_its_path() {
    let record = r#"{"discord":"/nix/store/84k7g556fj4ib11413jna62l3hgfzyna-discord-1.0.159"}"#;

    let paths: BTreeMap<String, String> = serde_json::from_str(record).unwrap();

    assert_eq!(
        CurrentHome::Recorded(paths).path_of("discord", None).as_deref(),
        Some(DISCORD_OLD)
    );
}
