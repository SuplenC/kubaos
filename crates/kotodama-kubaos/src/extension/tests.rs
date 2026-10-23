use std::collections::BTreeMap;
use std::time::Duration;

use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;

use super::*;
use crate::fake::FakeBus;
use crate::fake::FakeRunner;

fn configured(text: &str) -> Extension<FakeBus, FakeRunner> {
    let mut extension = Extension::new(FakeBus::default(), FakeRunner::default());
    let problems = extension.configure(&toml::from_str(text).expect("the test's config"));
    assert!(problems.is_empty(), "{problems:?}");
    extension
}

fn item(kind: &str, id: &str) -> ItemRef {
    ItemRef {
        id: id.to_owned(),
        kind: ItemKind::Namespaced(NamespacedKind::new(kind).unwrap()),
    }
}

fn ids(extension: &Extension<FakeBus, FakeRunner>) -> Vec<String> {
    extension.items().map(|item| item.id).collect()
}

#[test]
fn without_a_config_there_are_the_toggles() {
    let extension = Extension::new(FakeBus::default(), FakeRunner::default());

    assert_eq!(ids(&extension), toggles::ids());
}

#[test]
fn the_actions_come_before_the_toggles() {
    let extension = configured("[actions.mine]\ntitle = \"Mine\"\nrun = [\"true\"]");

    assert_eq!(ids(&extension).first().map(String::as_str), Some("mine"));
    assert_eq!(ids(&extension).len(), toggles::ids().len() + 1);
}

#[test]
fn hide_takes_out_a_toggle_too() {
    let extension = configured(r#"hide = ["bluetooth"]"#);

    assert!(!ids(&extension).contains(&"bluetooth".to_owned()));
    assert!(ids(&extension).contains(&"wifi".to_owned()));
}

#[test]
fn a_hidden_toggle_does_not_run() {
    let mut extension = configured(r#"hide = ["mute"]"#);

    let ran = extension.run(toggles::RUN, &item(toggles::KIND, "mute"), &BTreeMap::new());

    assert!(ran.is_err());
    assert!(extension.runner.ran().is_empty());
}

#[test]
fn an_action_launches_its_command_and_closes_the_menu() {
    let mut extension = configured(
        r#"
        [actions.shot]
        title = "Screenshot"
        run = ["dot-hypr-screenshot", "region"]
        delay_ms = 400
        "#,
    );

    let ran = extension.run(registry::RUN, &item(registry::KIND, "shot"), &BTreeMap::new());

    assert_eq!(ran, Ok(Ran::Close));
    assert_eq!(
        *extension.runner.launched.borrow(),
        [(
            vec!["dot-hypr-screenshot".to_owned(), "region".to_owned()],
            Duration::from_millis(400)
        )]
    );
}

#[test]
fn an_action_that_cannot_start_says_why() {
    let mut extension = configured("[actions.mine]\ntitle = \"Mine\"\nrun = [\"nothing-here\"]");
    extension.runner.failing = Some("no".into());

    let ran = extension.run(registry::RUN, &item(registry::KIND, "mine"), &BTreeMap::new());

    assert_eq!(ran, Err("couldn't start nothing-here".to_owned()));
}

#[test]
fn a_new_config_replaces_the_actions() {
    let mut extension = configured("[actions.old]\ntitle = \"Old\"\nrun = [\"true\"]");

    extension.configure(&toml::from_str("[actions.new]\ntitle = \"New\"\nrun = [\"true\"]").unwrap());

    assert!(ids(&extension).contains(&"new".to_owned()));
    assert!(!ids(&extension).contains(&"old".to_owned()));
}

fn with_layouts(text: &str) -> Extension<FakeBus, FakeRunner> {
    let mut extension = configured(text);
    extension.set_keyboards(Keyboards::parse(
        "XKBLAYOUT=\"us,it\"\nXKBVARIANT=\",\"\n",
        "! layout\n  us  English (US)\n  it  Italian\n  de  German\n",
    ));
    extension
}

#[test]
fn the_keyboard_items_come_after_the_toggles() {
    let extension = with_layouts("");

    let ids = ids(&extension);

    assert_eq!(ids[..toggles::ids().len()], toggles::ids());
    assert_eq!(
        ids[toggles::ids().len()..],
        ["keyboard:us", "keyboard:it", "keyboard-add"]
    );
}

#[test]
fn a_keyboard_item_runs_through_the_extension() {
    let mut extension = with_layouts("");

    let ran = extension.run(keyboard::RUN, &item(keyboard::KIND, "keyboard:it"), &BTreeMap::new());

    assert_eq!(ran, Ok(Ran::Close));
    assert_eq!(extension.runner.ran(), ["hyprctl switchxkblayout all 1"]);
}

#[test]
fn hide_takes_out_a_keyboard_item_and_its_run() {
    let mut extension = with_layouts(r#"hide = ["keyboard-add"]"#);
    let typed = BTreeMap::from([("layout".to_owned(), "de".to_owned())]);

    let ran = extension.run(keyboard::RUN, &item(keyboard::KIND, keyboard::ADD), &typed);

    assert!(!ids(&extension).contains(&"keyboard-add".to_owned()));
    assert!(ids(&extension).contains(&"keyboard:us".to_owned()));
    assert!(ran.is_err());
    assert!(extension.runner.launched.borrow().is_empty());
}

#[test]
fn an_action_may_not_take_a_keyboard_items_id() {
    let mut extension = Extension::new(FakeBus::default(), FakeRunner::default());

    let problems = extension.configure(
        &toml::from_str("[actions.keyboard-add]\ntitle = \"Mine\"\nrun = [\"true\"]").expect("the test's config"),
    );

    assert_eq!(problems.len(), 1, "{problems:?}");
}

#[test]
fn an_unknown_item_or_action_says_so() {
    let mut extension = configured("");

    let unknown_item = extension.run(registry::RUN, &item(registry::KIND, "nope"), &BTreeMap::new());
    let unknown_action = extension.run("explode", &item(toggles::KIND, "wifi"), &BTreeMap::new());

    assert_eq!(unknown_item, Err("`nope` has no Action `run`".to_owned()));
    assert!(unknown_action.is_err());
}
