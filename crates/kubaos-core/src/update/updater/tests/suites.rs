use std::fs;
use std::sync::Arc;

use super::Fake;
use super::World;
use super::notes;
use super::phases;
use super::summary;
use super::warnings;
use super::yes;
use crate::mise::Mise;
use crate::mise::fake::FakeMise;
use crate::suites::OptionsChange;
use crate::suites::Suites;
use crate::suites::catalog::TEST;
use crate::update::event::Phase;
use crate::update::event::SuiteTool;
use crate::update::event::UpdateEvent;
use crate::update::record::Channel;
use crate::update::record::parse_history;
use crate::update::updater::Confirm;

/// The test Suite installed in `w`'s home at 1.0.0, a patch release behind its newest, 1.0.1.
fn test_suite_behind(w: &World) -> Arc<FakeMise> {
    let mise = Arc::new(FakeMise::new(w.user.mise_config().join("conf.d")));
    let suites = Suites::with_mise(&w.user, Arc::clone(&mise) as Arc<dyn Mise>);
    suites.add(&TEST, &OptionsChange::default()).unwrap();
    let path = w.user.mise_lock();
    let text = fs::read_to_string(&path).unwrap().replace("\"1.0.1\"", "\"1.0.0\"");
    fs::write(&path, text).unwrap();
    mise.preinstall("kubaos-test-tool", "1.0.0");
    mise.prune().unwrap();
    mise
}

fn tool(from: Option<&str>, to: &str) -> SuiteTool {
    SuiteTool {
        suite: "test".into(),
        tool: "kubaos-test-tool".into(),
        from: from.map(Into::into),
        to: to.into(),
    }
}

#[test]
fn an_update_moves_the_suites_tools_after_the_switch_and_lists_their_upgrades() {
    let w = World::new(Channel::Edge, true);
    w.push(&[("VERSION", "0.1.1\n")], true);
    let mise = test_suite_behind(&w);
    let mut m = Fake::new();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events)
        .with_mise(Arc::clone(&mise) as Arc<dyn Mise>)
        .update(yes())
        .unwrap();

    let phases = phases(&events);
    let home = phases.iter().position(|p| *p == Phase::Home).unwrap();
    assert_eq!(phases.get(home + 1), Some(&Phase::Suites), "{phases:?}");
    assert_eq!(mise.installed(), ["kubaos-test-tool@1.0.1"]);
    let suites = &summary(&events).suites;
    assert_eq!(suites.updated, [tool(Some("1.0.0"), "1.0.1")]);
    assert_eq!(suites.upgrades, [tool(Some("1.0"), "3.0")]);
    assert_eq!(suites.error, None);
    let notes = notes(&events);
    assert!(notes.contains(&"test: kubaos-test-tool 1.0 → 3.0"), "{notes:?}");
    assert!(notes.contains(&"Upgrade them with kubaos suite upgrade"), "{notes:?}");
    let history = parse_history(&fs::read_to_string(w.sys.history()).unwrap());
    let saved = history[0].mise_lock.clone().expect("the Suites' entries are saved");
    assert!(fs::read_to_string(saved).unwrap().contains("\"1.0.0\""));
    assert_eq!(
        fs::read_to_string(w.user.mise_fragment("test")).unwrap().lines().last(),
        Some("kubaos-test-tool = \"1.0\""),
        "an update leaves the fragment alone"
    );
}

#[test]
fn a_rollback_puts_the_suites_tools_back_at_their_versions_from_before_the_update() {
    let w = World::new(Channel::Edge, true);
    w.push(&[("VERSION", "0.1.1\n")], true);
    let mise = test_suite_behind(&w);
    w.updater(&mut Fake::new())
        .with_mise(Arc::clone(&mise) as Arc<dyn Mise>)
        .update(yes())
        .unwrap();
    let mut events = Vec::new();

    w.reporting(&mut Fake::new(), &mut events)
        .with_mise(Arc::clone(&mise) as Arc<dyn Mise>)
        .rollback(Confirm::AssumeYes)
        .unwrap();

    assert_eq!(mise.installed(), ["kubaos-test-tool@1.0.0"]);
    assert_eq!(phases(&events).last(), Some(&Phase::Suites));
    assert_eq!(summary(&events).suites.restored, [tool(None, "1.0.0")]);
    assert!(
        notes(&events).contains(&"Your Suites' tools go back to the versions they had before that update"),
        "{:?}",
        notes(&events)
    );
}

#[test]
fn a_failed_suite_step_is_a_warning_and_the_update_goes_on() {
    let w = World::new(Channel::Edge, true);
    w.push(&[("VERSION", "0.1.1\n")], true);
    let mise = test_suite_behind(&w);
    mise.fail_on("kubaos-test-tool");
    let mut events = Vec::new();

    w.reporting(&mut Fake::new(), &mut events)
        .with_mise(Arc::clone(&mise) as Arc<dyn Mise>)
        .update(yes())
        .unwrap();

    assert_eq!(w.installed().version.to_string(), "0.1.1");
    assert!(summary(&events).suites.error.is_some());
    assert!(
        warnings(&events).iter().any(|w| w.starts_with("Suites: ")),
        "{:?}",
        warnings(&events)
    );
    assert!(events.iter().any(|e| matches!(
        e,
        UpdateEvent::PhaseFinished {
            phase: Phase::Suites,
            ..
        }
    )));
}
