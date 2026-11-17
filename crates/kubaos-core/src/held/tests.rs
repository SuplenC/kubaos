use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use super::Build;
use super::CurrentHome;
use super::Decision;
use super::Evaluated;
use super::Found;
use super::HeldFile;
use super::Hold;
use super::Holds;
use super::Outcome;
use super::Prebuilt;
use super::decide;
use super::failed_derivations;
use super::message;
use super::package_name;
use super::prebuilt;
use super::unchanged;
use super::unkept_message;
use crate::state::StateDir;

const DISCORD_OLD: &str = "/nix/store/84k7g556fj4ib11413jna62l3hgfzyna-discord-1.0.159";
const DISCORD_NEW: &str = "/nix/store/gkm5zlch3916yfzbswzas38d5ysvl917-discord-1.0.159";
const DISCORD_DRV: &str = "/nix/store/b2qzjq7gkvybdpdf7hh2hsi6x3cpag3s-discord-1.0.159.drv";

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

fn discord() -> Found {
    found(DISCORD_DRV, &[("out", DISCORD_NEW)], "discord")
}

fn evaluated(name: &str, found: Found) -> Evaluated {
    Evaluated {
        name: name.to_owned(),
        found: Some(found),
    }
}

fn prebuilt_as(name: &str, outcome: Outcome, found: Option<Found>) -> Prebuilt {
    Prebuilt {
        name: name.to_owned(),
        outcome,
        found,
    }
}

/// Holds of `pairs`, each since the derivation `failed`.
fn holds(pairs: &[(&str, &str)]) -> Holds {
    pairs
        .iter()
        .map(|(n, p)| {
            let hold = Hold {
                path: (*p).to_owned(),
                failed: DISCORD_DRV.to_owned(),
            };
            ((*n).to_owned(), hold)
        })
        .collect()
}

fn held_since(drv: &str) -> Holds {
    let hold = Hold {
        path: DISCORD_OLD.to_owned(),
        failed: drv.to_owned(),
    };
    Holds::from([("discord".to_owned(), hold)])
}

fn recorded(pairs: &[(&str, &str)]) -> CurrentHome {
    CurrentHome::Recorded(pairs.iter().map(|(n, p)| ((*n).to_owned(), (*p).to_owned())).collect())
}

fn discord_failed() -> Build {
    Build {
        missing: BTreeSet::from([DISCORD_NEW.to_owned()]),
        failed: BTreeSet::from([DISCORD_DRV.to_owned()]),
    }
}

#[test]
fn the_messages_name_the_package_as_listed() {
    assert_eq!(
        message("discord"),
        "discord stays at its current version: it couldn't be built"
    );
    assert_eq!(
        unkept_message("discord"),
        "discord couldn't be built, and your home has no version of it to keep"
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
fn nix_names_the_derivations_that_failed_and_the_package_whose_fetch_failed() {
    let stderr = "\
building '/nix/store/xxizsc33mlz3ii71dzafvgrmf584nlpa-full.distro.drv'...
error: Cannot build '/nix/store/xxizsc33mlz3ii71dzafvgrmf584nlpa-full.distro.drv'.
       Reason: builder failed with exit code 1.
error: Cannot build '/nix/store/b2qzjq7gkvybdpdf7hh2hsi6x3cpag3s-discord-1.0.159.drv'.
       Reason: 14 dependencies failed.
       Output paths:
         /nix/store/gkm5zlch3916yfzbswzas38d5ysvl917-discord-1.0.159
error: builder for '/nix/store/aaa-old.drv' failed with exit code 2
warning: '/nix/store/bbb-not-an-error.drv' is just a warning
";

    let failed = failed_derivations(stderr);

    assert_eq!(
        failed,
        BTreeSet::from([
            "/nix/store/aaa-old.drv".to_owned(),
            DISCORD_DRV.to_owned(),
            "/nix/store/xxizsc33mlz3ii71dzafvgrmf584nlpa-full.distro.drv".to_owned(),
        ])
    );
}

#[test]
fn a_package_nix_says_failed_failed_and_one_with_its_outputs_built() {
    let jq = found("/nix/store/j-jq.drv", &[("bin", "/nix/store/j-jq-bin")], "jq");
    let listed = [evaluated("discord", discord()), evaluated("jq", jq.clone())];

    let prebuilt = prebuilt(&listed, &discord_failed(), &BTreeSet::new()).unwrap();

    assert_eq!(
        prebuilt,
        [
            prebuilt_as("discord", Outcome::Failed, Some(discord())),
            prebuilt_as("jq", Outcome::Built, Some(jq)),
        ]
    );
}

#[test]
fn outputs_missing_that_nix_did_not_say_failed_are_no_failure_to_hold() {
    let build = Build {
        missing: BTreeSet::from([DISCORD_NEW.to_owned()]),
        failed: BTreeSet::new(),
    };

    let err = prebuilt(&[evaluated("discord", discord())], &build, &BTreeSet::new()).unwrap_err();

    assert_eq!(
        err.to_string(),
        "the build stopped before it built discord, without saying why"
    );
}

#[test]
fn a_package_that_does_not_evaluate_is_unevaluated() {
    let gone = Evaluated {
        name: "gone".to_owned(),
        found: None,
    };

    let prebuilt = prebuilt(&[gone], &Build::default(), &BTreeSet::new()).unwrap();

    assert_eq!(prebuilt, [prebuilt_as("gone", Outcome::Unevaluated, None)]);
}

#[test]
fn a_package_that_failed_and_that_the_home_has_is_held_with_its_failed_derivation() {
    let prebuilt = [prebuilt_as("discord", Outcome::Failed, Some(discord()))];

    let decision = decide(&prebuilt, &recorded(&[("discord", DISCORD_OLD)]), &Holds::new());

    assert_eq!(
        decision,
        Decision {
            held: held_since(DISCORD_DRV),
            unkept: Vec::new()
        }
    );
}

#[test]
fn a_package_that_failed_and_that_the_home_lacks_is_unkept() {
    let prebuilt = [prebuilt_as("discord", Outcome::Failed, Some(discord()))];

    for current in [recorded(&[("jq", "/nix/store/j-jq")]), CurrentHome::Nothing] {
        let decision = decide(&prebuilt, &current, &Holds::new());

        assert!(decision.held.is_empty(), "{current:?}");
        assert_eq!(decision.unkept, ["discord"]);
    }
}

#[test]
fn a_held_package_that_no_longer_evaluates_or_that_builds_loses_its_hold() {
    let current = recorded(&[("discord", DISCORD_OLD)]);

    for outcome in [Outcome::Unevaluated, Outcome::Built] {
        let found = (outcome == Outcome::Built).then(discord);
        let prebuilt = [prebuilt_as("discord", outcome, found)];

        let decision = decide(&prebuilt, &current, &held_since(DISCORD_DRV));

        assert_eq!(decision, Decision::default());
    }
}

#[test]
fn a_generation_without_the_record_has_the_one_path_of_the_same_package() {
    let current = CurrentHome::Unrecorded(vec![
        "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-discord-ptb-0.0.170".to_owned(),
        DISCORD_OLD.to_owned(),
        "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-hm-session-vars.sh".to_owned(),
    ]);
    let newer = found("/nix/store/n3w-discord-1.0.160.drv", &[("out", DISCORD_NEW)], "discord");

    let decision = decide(
        &[prebuilt_as("discord", Outcome::Failed, Some(newer))],
        &current,
        &Holds::new(),
    );

    assert_eq!(decision.held["discord"].path, DISCORD_OLD);
}

#[test]
fn a_generation_without_the_record_holds_a_multi_output_package_at_its_main_output() {
    let main = "/nix/store/0mygjlbcn7lhq0zvz7jvyqmzv2nvj7s6-postgresql-17.12";
    let current = CurrentHome::Unrecorded(vec![
        main.to_owned(),
        "/nix/store/hn37fswnp3x9vinr83z4q7y7vpb385v9-postgresql-17.12-man".to_owned(),
    ]);
    let outputs = [
        ("out", "/nix/store/new-postgresql-17.13"),
        ("man", "/nix/store/new-postgresql-17.13-man"),
    ];
    let same = found(
        "/nix/store/k8a6g93hxqnnj05pk9yv39pll8i467zv-postgresql-17.12.drv",
        &outputs,
        "postgresql",
    );
    let newer = found(
        "/nix/store/z9z9z9z9z9z9z9z9z9z9z9z9z9z9z9z9-postgresql-17.13.drv",
        &outputs,
        "postgresql",
    );

    for found in [same, newer] {
        let decision = decide(
            &[prebuilt_as("postgresql_17", Outcome::Failed, Some(found))],
            &current,
            &Holds::new(),
        );

        assert_eq!(decision.held["postgresql_17"].path, main);
    }
}

#[test]
fn a_generation_without_the_record_and_two_versions_of_a_package_holds_neither() {
    let current = CurrentHome::Unrecorded(vec![
        DISCORD_OLD.to_owned(),
        "/nix/store/cccccccccccccccccccccccccccccccc-discord-1.0.158".to_owned(),
    ]);
    let newer = found("/nix/store/n3w-discord-1.0.160.drv", &[("out", DISCORD_NEW)], "discord");

    let decision = decide(
        &[prebuilt_as("discord", Outcome::Failed, Some(newer))],
        &current,
        &Holds::new(),
    );

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

#[test]
fn a_held_package_whose_derivation_is_the_one_that_failed_is_unchanged() {
    let listed = [evaluated("discord", discord())];

    assert_eq!(
        unchanged(&listed, &held_since(DISCORD_DRV)),
        BTreeSet::from(["discord".to_owned()])
    );
}

#[test]
fn a_held_package_with_another_derivation_is_built_again() {
    let newer = found("/nix/store/n3w-discord-1.0.160.drv", &[("out", DISCORD_NEW)], "discord");

    assert!(unchanged(&[evaluated("discord", newer)], &held_since(DISCORD_DRV)).is_empty());
}

#[test]
fn an_unchanged_package_still_missing_keeps_its_hold_as_it_was() {
    let build = Build {
        missing: BTreeSet::from([DISCORD_NEW.to_owned()]),
        failed: BTreeSet::new(),
    };
    let skipped = BTreeSet::from(["discord".to_owned()]);

    let prebuilt = prebuilt(&[evaluated("discord", discord())], &build, &skipped).unwrap();
    let decision = decide(&prebuilt, &CurrentHome::Nothing, &held_since(DISCORD_DRV));

    assert_eq!(prebuilt[0].outcome, Outcome::Unchanged);
    assert_eq!(decision.held, held_since(DISCORD_DRV));
}

#[test]
fn an_unchanged_package_whose_outputs_turned_up_built() {
    let skipped = BTreeSet::from(["discord".to_owned()]);

    let prebuilt = prebuilt(&[evaluated("discord", discord())], &Build::default(), &skipped).unwrap();

    assert_eq!(prebuilt[0].outcome, Outcome::Built);
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
    assert!(text.contains("[held.\"python3Packages.black\"]\n"), "{text}");
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
fn a_hold_and_its_failed_derivation_read_back_from_the_file() {
    let tmp = tempfile::tempdir().unwrap();
    let file = held_file(tmp.path());

    file.write(&held_since(DISCORD_DRV)).unwrap();

    assert_eq!(file.read().unwrap(), held_since(DISCORD_DRV));
    let text = fs::read_to_string(file.state().held_file()).unwrap();
    assert!(
        text.contains(&format!(
            "[held.discord]\npath = \"{DISCORD_OLD}\"\nfailed = \"{DISCORD_DRV}\"\n"
        )),
        "{text}"
    );
}

#[test]
fn a_record_reads_as_each_listed_package_to_its_path() {
    let record = format!(r#"{{"discord":"{DISCORD_OLD}"}}"#);

    let paths: BTreeMap<String, String> = serde_json::from_str(&record).unwrap();

    assert_eq!(
        CurrentHome::Recorded(paths).path_of("discord", &discord()).as_deref(),
        Some(DISCORD_OLD)
    );
}
