use std::fs;

use super::super::tests::DEMO_MISE;
use super::super::tests::Home;
use super::super::tests::fragment;
use super::super::tests::home;
use super::super::tests::requests;
use super::super::tests::suite_file;
use super::super::tests::write;
use super::Available;
use super::Reach;
use super::newer;
use crate::mise::Mise;
use crate::suites::OptionsChange;
use crate::suites::Request;
use crate::suites::Suite;
use crate::suites::Tool;
use crate::suites::catalog;
use crate::suites::major_minor;

/// Sets `tool`'s entry in the lockfile to `version` and leaves that version the one installed,
/// as an install from before its newer patch release would.
pub(in crate::suites) fn lock_at(h: &Home, tool: &str, version: &str) {
    let path = h.paths.mise_lock();
    let mut doc: toml_edit::DocumentMut = fs::read_to_string(&path).unwrap().parse().unwrap();
    let entries = doc["tools"][tool].as_array_of_tables_mut().unwrap();
    entries.get_mut(0).unwrap()["version"] = toml_edit::value(version);
    fs::write(&path, doc.to_string()).unwrap();
    h.mise.preinstall(tool, version);
    h.mise.prune().unwrap();
}

/// The demo Suite installed with its language at 1.0.0 and its language server at 0.5.0, each a
/// patch release behind.
fn behind() -> Home {
    let h = home();
    h.suites.add(&DEMO_MISE, &OptionsChange::default()).unwrap();
    lock_at(&h, "demo-lang", "1.0.0");
    lock_at(&h, "demo-ls", "0.5.0");
    h
}

fn available(tool: &str, request: &str, current: &str, latest: &str, target: Option<&str>) -> Available {
    Available {
        tool: tool.into(),
        request: request.into(),
        current: Some(current.into()),
        latest: latest.into(),
        target: target.map(Into::into),
    }
}

#[test]
fn an_update_moves_each_tool_to_its_newest_patch_and_prunes() {
    let h = behind();
    let fragment_before = fs::read_to_string(fragment(&h)).unwrap();
    let record_before = fs::read_to_string(suite_file(&h)).unwrap();

    let moved = h.suites.update(&[&DEMO_MISE]).unwrap();

    assert_eq!(moved.len(), 1);
    assert_eq!(
        moved[0].tools,
        [
            available("demo-lang", "1.0", "1.0.0", "1.0.3", None),
            available("demo-ls", "0.5", "0.5.0", "0.5.2", None),
        ]
    );
    assert_eq!(h.mise.installed(), ["demo-lang@1.0.3", "demo-ls@0.5.2"]);
    let calls = h.mise.calls();
    assert_eq!(
        calls[calls.len() - 3..],
        [
            "mise outdated --json",
            "mise upgrade demo-lang demo-ls",
            "mise prune --tools"
        ]
    );
    assert_eq!(fs::read_to_string(fragment(&h)).unwrap(), fragment_before);
    assert_eq!(fs::read_to_string(suite_file(&h)).unwrap(), record_before);
}

#[test]
fn an_update_with_nothing_newer_moves_nothing() {
    let h = home();
    h.suites.add(&DEMO_MISE, &OptionsChange::default()).unwrap();

    let moved = h.suites.update(&[&DEMO_MISE]).unwrap();

    assert!(moved.is_empty(), "{moved:?}");
    assert!(!h.mise.calls().iter().any(|c| c.starts_with("mise upgrade")));
}

#[test]
fn a_suite_without_a_fragment_runs_no_mise() {
    let h = home();

    assert!(h.suites.available(Reach::Upgrade).unwrap().is_empty());
    assert!(h.suites.update(&[&DEMO_MISE]).unwrap().is_empty());
    assert!(h.mise.calls().is_empty(), "{:?}", h.mise.calls());
}

#[test]
fn an_upgrade_names_each_target_and_moves_the_main_toolchain_to_the_newest_line_the_suite_has() {
    let h = home();
    h.suites.add(&DEMO_MISE, &OptionsChange::default()).unwrap();

    let upgraded = h.suites.upgrade(&[&DEMO_MISE]).unwrap();

    assert_eq!(
        upgraded.moved[0].tools,
        [
            available("demo-lang", "1.0", "1.0.3", "2.1.0", Some("2.1")),
            available("demo-ls", "0.5", "0.5.2", "0.6.1", Some("0.6")),
        ]
    );
    assert!(
        h.mise
            .calls()
            .contains(&"mise upgrade --bump demo-lang@2.1 demo-ls@0.6".to_owned()),
        "{:?}",
        h.mise.calls()
    );
    let text = fs::read_to_string(fragment(&h)).unwrap();
    assert!(text.starts_with('#'), "mise keeps the header: {text}");
    assert!(text.contains("demo-lang = \"2.1\"\ndemo-ls = \"0.6\""), "{text}");
    assert_eq!(h.mise.installed(), ["demo-lang@2.1.0", "demo-ls@0.6.1"]);
    let config = h.suites.config(&DEMO_MISE).unwrap().unwrap();
    assert_eq!(config.version.as_deref(), Some("2"), "the record takes the new line");
    let [(suite, before)] = upgraded.lines.as_slice() else {
        panic!("{:?}", upgraded.lines);
    };
    assert_eq!(suite.name, "demo");
    assert_eq!(before.tools, requests(&[("demo-lang", "1.0"), ("demo-ls", "0.5")]));
    assert_eq!(before.config.as_ref().unwrap().version.as_deref(), Some("1"));
}

#[test]
fn an_upgrade_of_a_main_toolchain_on_its_newest_line_offers_nothing_past_it() {
    let h = home();
    h.suites
        .add(
            &DEMO_MISE,
            &OptionsChange {
                version: Some("2".into()),
                ..OptionsChange::default()
            },
        )
        .unwrap();

    let upgraded = h.suites.upgrade(&[&DEMO_MISE]).unwrap();

    assert_eq!(
        upgraded.moved[0].tools,
        [available_ls()],
        "demo-lang 3 and 4 are on no line the Suite has"
    );
    assert!(upgraded.lines.is_empty());
}

fn available_ls() -> Available {
    available("demo-ls", "0.5", "0.5.2", "0.6.1", Some("0.6"))
}

#[test]
fn an_upgrade_of_the_test_suite_moves_its_record_to_the_line_mise_bumps_to() {
    let h = home();
    h.suites.add(&catalog::TEST, &OptionsChange::default()).unwrap();

    let available = h.suites.available(Reach::Upgrade).unwrap();
    let upgraded = h.suites.upgrade(&[&catalog::TEST]).unwrap();

    assert_eq!(available.len(), 1);
    assert_eq!(available[0].suite.name, "test");
    assert_eq!(
        available[0].tools,
        [Available {
            tool: "kubaos-test-tool".into(),
            request: "1.0".into(),
            current: Some("1.0.1".into()),
            latest: "3.0.0".into(),
            target: Some("3.0".into()),
        }]
    );
    assert_eq!(upgraded.lines.len(), 1);
    let config = h.suites.config(&catalog::TEST).unwrap().unwrap();
    assert_eq!(config.version.as_deref(), Some("3"));
    assert_eq!(h.mise.installed(), ["kubaos-test-tool@3.0.0"]);
}

#[test]
fn a_restore_after_a_failed_switch_takes_an_upgrade_back() {
    let h = home();
    h.suites.add(&DEMO_MISE, &OptionsChange::default()).unwrap();
    let record = fs::read_to_string(suite_file(&h)).unwrap();
    let upgraded = h.suites.upgrade(&[&DEMO_MISE]).unwrap();

    for (suite, before) in &upgraded.lines {
        h.suites.restore(suite, before).unwrap();
    }

    assert_eq!(fs::read_to_string(suite_file(&h)).unwrap(), record);
    let text = fs::read_to_string(fragment(&h)).unwrap();
    assert!(text.contains("demo-lang = \"1.0\"\ndemo-ls = \"0.5\""), "{text}");
    assert_eq!(h.mise.installed(), ["demo-lang@1.0.3", "demo-ls@0.5.2"]);
}

#[test]
fn update_and_upgrade_leave_a_project_s_pinned_version_and_its_file_alone() {
    let h = behind();
    let project = h.paths.home().join("Projects/acme/mise.toml");
    write(&project, "[tools]\ndemo-ls = \"0.4.1\"\n");
    h.mise.preinstall("demo-ls", "0.4.1");
    h.mise.track(project.clone());

    h.suites.update(&[&DEMO_MISE]).unwrap();
    h.suites.upgrade(&[&DEMO_MISE]).unwrap();

    assert_eq!(fs::read_to_string(&project).unwrap(), "[tools]\ndemo-ls = \"0.4.1\"\n");
    assert!(
        h.mise.installed().contains(&"demo-ls@0.4.1".to_owned()),
        "{:?}",
        h.mise.installed()
    );
    assert!(
        !h.mise.calls().iter().any(|c| c.contains("acme")),
        "{:?}",
        h.mise.calls()
    );
}

#[test]
fn a_tool_another_config_asks_for_is_left_out() {
    let h = behind();
    write(
        &h.paths.mise_config().join("conf.d/zz-mine.toml"),
        "[tools]\ndemo-ls = \"0.4\"\n",
    );
    h.mise.preinstall("demo-ls", "0.4.1");

    let moved = h.suites.update(&[&DEMO_MISE]).unwrap();

    let tools: Vec<&str> = moved.iter().flat_map(|s| &s.tools).map(|t| t.tool.as_str()).collect();
    assert_eq!(
        tools,
        ["demo-lang"],
        "zz-mine.toml loads later, so demo-ls is the user's"
    );
    assert!(h.mise.calls().contains(&"mise upgrade demo-lang".to_owned()));
}

/// A Suite whose main toolchain has releases named by channel, not by number.
const NIGHTLY: Suite = Suite {
    versions: &["nightly"],
    mise: &[Tool {
        name: "demo-nightly",
        request: Request::Suite,
        extra: None,
    }],
    ..DEMO_MISE
};

#[test]
fn a_main_toolchain_whose_versions_are_not_numbered_keeps_the_line_as_chosen() {
    let h = home();

    let added = h.suites.add(&NIGHTLY, &OptionsChange::default()).unwrap();

    assert_eq!(added.tools, requests(&[("demo-nightly", "nightly")]));
}

#[test]
fn major_minor_takes_the_first_two_numbered_parts_of_a_three_part_version() {
    assert_eq!(major_minor("24.22.1").as_deref(), Some("24.22"));
    assert_eq!(major_minor("0.13.2-rc.1").as_deref(), Some("0.13"));
    assert_eq!(major_minor("1.27"), None);
    assert_eq!(major_minor("nightly"), None);
    assert_eq!(major_minor("v1.2.3"), None);
}

#[test]
fn a_version_is_newer_by_its_numbered_parts() {
    assert!(newer("2.10.0", "2.9.4"));
    assert!(newer("26.1.0", "24.22.1"));
    assert!(!newer("1.0.3", "1.0.3"));
    assert!(!newer("1.0", "1.0.1"));
}
