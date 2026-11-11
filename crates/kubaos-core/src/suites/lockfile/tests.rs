use std::fs;
use std::path::PathBuf;

use super::super::tests::Home;
use super::super::tests::home;
use super::super::updates::tests::lock_at;
use crate::mise::Mise;
use crate::mise::ToolRequest;
use crate::suites::CachePolicy;
use crate::suites::OptionsChange;
use crate::suites::catalog::TEST;

/// The test Suite installed at 1.0.0, a patch release behind, with its lockfile entries saved
/// as an update saves them before moving it.
fn saved_behind() -> (Home, PathBuf) {
    let h = home();
    h.suites.add(&TEST, &OptionsChange::default()).unwrap();
    lock_at(&h, "kubaos-test-tool", "1.0.0");
    let saved = h.paths.saved_locks().join("1.mise.lock");
    assert!(h.suites.save_locked(&saved).unwrap());
    (h, saved)
}

fn lockfile(h: &Home) -> String {
    fs::read_to_string(h.paths.mise_lock()).unwrap()
}

#[test]
fn saving_keeps_only_the_entries_of_the_suites_tools() {
    let h = home();
    h.suites.add(&TEST, &OptionsChange::default()).unwrap();
    let mut text = lockfile(&h);
    text.push_str("\n[[tools.someone-elses]]\nversion = \"9.9.9\"\nbackend = \"asdf:someone-elses\"\n");
    fs::write(h.paths.mise_lock(), text).unwrap();
    let saved = h.paths.saved_locks().join("1.mise.lock");

    assert!(h.suites.save_locked(&saved).unwrap());

    let text = fs::read_to_string(&saved).unwrap();
    assert!(text.starts_with('#'), "it says what it is: {text}");
    let doc: toml::Table = toml::from_str(&text).unwrap();
    let tools = doc["tools"].as_table().unwrap();
    assert_eq!(tools.keys().collect::<Vec<_>>(), ["kubaos-test-tool"]);
    assert_eq!(tools["kubaos-test-tool"][0]["version"].as_str(), Some("1.0.1"));
}

#[test]
fn saving_with_no_suite_tools_writes_nothing() {
    let h = home();
    let saved = h.paths.saved_locks().join("1.mise.lock");

    assert!(!h.suites.save_locked(&saved).unwrap());
    assert!(!saved.exists());
    assert!(h.mise.calls().is_empty(), "{:?}", h.mise.calls());
}

#[test]
fn restoring_puts_the_version_from_before_the_update_back_and_prunes_the_newer_one() {
    let (h, saved) = saved_behind();
    h.suites.update(&[&TEST]).unwrap();
    assert_eq!(h.mise.installed(), ["kubaos-test-tool@1.0.1"]);
    let fragment = fs::read_to_string(h.paths.mise_fragment("test")).unwrap();

    let restored = h.suites.restore_locked(&saved).unwrap();

    assert_eq!(
        restored,
        [(
            "test",
            ToolRequest {
                tool: "kubaos-test-tool".into(),
                request: "1.0.0".into(),
            }
        )]
    );
    assert_eq!(h.mise.installed(), ["kubaos-test-tool@1.0.0"]);
    let calls = h.mise.calls();
    assert_eq!(
        calls[calls.len() - 2..],
        ["mise install --locked", "mise prune --tools"]
    );
    assert_eq!(
        fs::read_to_string(h.paths.mise_fragment("test")).unwrap(),
        fragment,
        "the fragment stays"
    );
}

#[test]
fn restoring_after_the_suite_is_removed_puts_back_no_stale_entry() {
    let (h, saved) = saved_behind();
    h.suites.update(&[&TEST]).unwrap();
    h.suites.remove(&TEST, CachePolicy::Keep).unwrap();
    let before = lockfile(&h);
    let calls = h.mise.calls().len();

    let restored = h.suites.restore_locked(&saved).unwrap();

    assert!(restored.is_empty(), "{restored:?}");
    assert_eq!(lockfile(&h), before);
    assert_eq!(h.mise.calls().len(), calls, "{:?}", h.mise.calls());
    assert!(h.mise.installed().is_empty(), "{:?}", h.mise.installed());
}

#[test]
fn restoring_leaves_a_tool_an_upgrade_moved_past_the_saved_version() {
    let (h, saved) = saved_behind();
    h.suites.upgrade(&[&TEST]).unwrap();
    let before = lockfile(&h);

    let restored = h.suites.restore_locked(&saved).unwrap();

    assert!(restored.is_empty(), "{restored:?}");
    assert_eq!(lockfile(&h), before);
    assert_eq!(h.mise.installed(), ["kubaos-test-tool@3.0.0"]);
}

#[test]
fn restoring_from_nothing_saved_changes_nothing() {
    let h = home();
    h.suites.add(&TEST, &OptionsChange::default()).unwrap();
    let before = lockfile(&h);

    let restored = h
        .suites
        .restore_locked(&h.paths.saved_locks().join("none.mise.lock"))
        .unwrap();

    assert!(restored.is_empty(), "{restored:?}");
    assert_eq!(lockfile(&h), before);
    assert!(!h.mise.calls().iter().any(|c| c.contains("--locked")));
    h.mise.prune().unwrap();
}
