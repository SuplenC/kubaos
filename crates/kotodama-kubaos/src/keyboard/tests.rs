use std::collections::BTreeMap;
use std::time::Duration;

use super::*;
use crate::fake::FakeRunner;

const LST: &str = "! layout\n  us              English (US)\n  it              Italian\n  de              German\n\n\
                   ! variant\n  intl            us: English (US, intl., with dead keys)\n";

fn keyboards(layout: &str, variant: &str) -> Keyboards {
    Keyboards::parse(&format!("XKBLAYOUT=\"{layout}\"\nXKBVARIANT=\"{variant}\"\n"), LST)
}

fn none() -> BTreeMap<String, String> {
    BTreeMap::new()
}

fn titles(keyboards: &Keyboards) -> Vec<String> {
    keyboards.items().into_iter().map(|item| item.title).collect()
}

fn actions(keyboards: &Keyboards, id: &str) -> Vec<String> {
    let items = keyboards.items();
    let item = items.iter().find(|item| item.id == id).expect("the Item");
    item.actions.iter().map(|action| action.id.clone()).collect()
}

fn launched(runner: &FakeRunner) -> Vec<String> {
    runner
        .launched
        .borrow()
        .iter()
        .map(|(argv, delay)| {
            assert_eq!(*delay, Duration::ZERO);
            argv.join(" ")
        })
        .collect()
}

#[test]
fn two_layouts_are_each_by_its_name_and_add() {
    let keyboards = keyboards("us,it", ",");

    assert_eq!(
        titles(&keyboards),
        ["Keyboard: English (US)", "Keyboard: Italian", "Keyboard: add a layout"]
    );
    let items = keyboards.items();
    assert!(items.iter().all(|item| item.kind.name() == KIND));
    assert_eq!(items[0].id, "keyboard:us");
    assert_eq!(items[0].subtitle.as_deref(), Some("The default keyboard layout, us"));
}

#[test]
fn a_variant_is_named_and_spelled_in_the_id() {
    let keyboards = keyboards("it,us", ",intl");

    let items = keyboards.items();

    assert_eq!(items[1].id, "keyboard:us(intl)");
    assert_eq!(items[1].title, "Keyboard: English (US, intl., with dead keys)");
}

#[test]
fn one_layout_cannot_be_removed() {
    let keyboards = keyboards("us", "");

    assert_eq!(titles(&keyboards), ["Keyboard: English (US)", "Keyboard: add a layout"]);
    assert_eq!(actions(&keyboards, "keyboard:us"), [RUN], "nor moved");
}

#[test]
fn only_a_layout_after_the_first_can_be_made_the_default() {
    let keyboards = keyboards("us,it", "");

    assert_eq!(actions(&keyboards, "keyboard:us"), [RUN, MOVE_DOWN, REMOVE]);
    assert_eq!(actions(&keyboards, "keyboard:it"), [RUN, MAKE_DEFAULT, MOVE_UP, REMOVE]);
}

#[test]
fn a_layout_moves_up_unless_first_and_down_unless_last() {
    let keyboards = keyboards("us,it,de", "");

    assert_eq!(actions(&keyboards, "keyboard:us"), [RUN, MOVE_DOWN, REMOVE]);
    assert_eq!(
        actions(&keyboards, "keyboard:it"),
        [RUN, MAKE_DEFAULT, MOVE_UP, MOVE_DOWN, REMOVE]
    );
    assert_eq!(actions(&keyboards, "keyboard:de"), [RUN, MAKE_DEFAULT, MOVE_UP, REMOVE]);
}

#[test]
fn moving_up_and_down_launches_kubaos_keyboard_move_with_the_new_place() {
    let runner = FakeRunner::default();
    let keyboards = keyboards("us,it,de", "");

    assert_eq!(keyboards.run("keyboard:it", MOVE_UP, &none(), &runner), Ok(Ran::Close));
    assert_eq!(
        keyboards.run("keyboard:it", MOVE_DOWN, &none(), &runner),
        Ok(Ran::Close)
    );

    assert_eq!(
        launched(&runner),
        ["kubaos keyboard move it 1", "kubaos keyboard move it 3"]
    );
}

#[test]
fn a_move_past_either_end_is_refused_and_runs_nothing() {
    let runner = FakeRunner::default();
    let keyboards = keyboards("us,it", "");

    assert_eq!(
        keyboards.run("keyboard:us", MOVE_UP, &none(), &runner),
        Err("`us` is the first keyboard layout already".to_owned())
    );
    assert_eq!(
        keyboards.run("keyboard:it", MOVE_DOWN, &none(), &runner),
        Err("`it` is the last keyboard layout already".to_owned())
    );
    assert!(runner.launched.borrow().is_empty());
}

#[test]
fn the_users_layouts_win_over_the_systems_file() {
    let dir = tempfile::tempdir().unwrap();
    let (mine, system, lst) = (
        dir.path().join("keyboard.toml"),
        dir.path().join("keyboard"),
        dir.path().join("evdev.lst"),
    );
    fs::write(&system, "XKBLAYOUT=\"us,it\"\n").unwrap();
    fs::write(&lst, LST).unwrap();

    let before = Keyboards::read(&mine, &system, &lst);
    fs::write(&mine, "layouts = [\"de\", \"us(intl)\"]\n").unwrap();
    let after = Keyboards::read(&mine, &system, &lst);

    assert_eq!(titles(&before)[..2], ["Keyboard: English (US)", "Keyboard: Italian"]);
    assert_eq!(
        titles(&after)[..2],
        ["Keyboard: German", "Keyboard: English (US, intl., with dead keys)"]
    );
}

#[test]
fn without_the_file_there_are_no_keyboard_items() {
    let keyboards = Keyboards::parse("", LST);

    assert_eq!(titles(&keyboards), [] as [std::string::String; 0]);
}

#[test]
fn a_layout_switches_to_its_index() {
    let runner = FakeRunner::default();

    let ran = keyboards("us,it", "").run("keyboard:it", RUN, &none(), &runner);

    assert_eq!(ran, Ok(Ran::Close));
    assert_eq!(runner.ran(), ["hyprctl switchxkblayout all 1"]);
}

#[test]
fn a_failing_switch_says_why() {
    let runner = FakeRunner {
        failing: Some("no such device".to_owned()),
        ..FakeRunner::default()
    };

    let ran = keyboards("us,it", "").run("keyboard:it", RUN, &none(), &runner);

    assert!(ran.is_err_and(|message| message.contains("no such device")));
}

#[test]
fn make_default_and_remove_launch_kubaos_keyboard() {
    let runner = FakeRunner::default();
    let keyboards = keyboards("us,it,de", "");

    assert_eq!(
        keyboards.run("keyboard:it", MAKE_DEFAULT, &none(), &runner),
        Ok(Ran::Close)
    );
    assert_eq!(keyboards.run("keyboard:de", REMOVE, &none(), &runner), Ok(Ran::Close));

    assert_eq!(
        launched(&runner),
        ["kubaos keyboard default it", "kubaos keyboard remove de"]
    );
    assert_eq!(runner.ran(), [] as [std::string::String; 0]);
}

#[test]
fn add_launches_kubaos_keyboard_with_the_typed_layout() {
    let runner = FakeRunner::default();
    let typed = BTreeMap::from([(LAYOUT.to_owned(), " us(intl) ".to_owned())]);

    let ran = keyboards("it", "").run(ADD, RUN, &typed, &runner);

    assert_eq!(ran, Ok(Ran::Close));
    assert_eq!(launched(&runner), ["kubaos keyboard add us(intl)"]);
}

#[test]
fn a_change_the_rules_refuse_is_read_in_the_menu_and_runs_nothing() {
    let runner = FakeRunner::default();
    let keyboards = keyboards("us,it", "");
    let add = |typed: &str| BTreeMap::from([(LAYOUT.to_owned(), typed.to_owned())]);

    assert_eq!(
        keyboards.run(ADD, RUN, &add("xx"), &runner),
        Err("Unknown keyboard layout \"xx\"".to_owned())
    );
    assert_eq!(
        keyboards.run(ADD, RUN, &add("it"), &runner),
        Err("Keyboard layout \"it\" is listed twice".to_owned())
    );
    assert_eq!(
        keyboards.run(ADD, RUN, &add(""), &runner),
        Err("Odd keyboard layout \"\"".to_owned())
    );
    assert!(runner.launched.borrow().is_empty());
}

#[test]
fn a_fifth_layout_is_refused() {
    let runner = FakeRunner::default();
    let typed = BTreeMap::from([(LAYOUT.to_owned(), "de(nodeadkeys)".to_owned())]);

    let ran = keyboards("us,it,de,us", ",,,intl").run(ADD, RUN, &typed, &runner);

    assert!(ran.is_err_and(|message| message.starts_with("At most 4 keyboard layouts")));
}

#[test]
fn a_layout_that_is_gone_is_refused() {
    let runner = FakeRunner::default();

    let ran = keyboards("us,it", "").run("keyboard:de", RUN, &none(), &runner);

    assert_eq!(ran, Err("`de` isn't one of the keyboard layouts".to_owned()));
    assert_eq!(runner.ran(), [] as [std::string::String; 0]);
}

#[test]
fn an_unknown_action_is_refused() {
    let runner = FakeRunner::default();

    assert!(keyboards("us,it", "").run(ADD, REMOVE, &none(), &runner).is_err());
    assert!(
        keyboards("us,it", "")
            .run("keyboard-next", RUN, &none(), &runner)
            .is_err()
    );
    assert!(keyboards("us,it", "").run("wifi", RUN, &none(), &runner).is_err());
}
