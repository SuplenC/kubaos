use super::*;
use crate::fake::FakeRunner;

const LST: &str = "! layout\n  us              English (US)\n  it              Italian\n  cz              Czech\n\n\
                   ! variant\n  intl            us: English (US, intl., with dead keys)\n";

fn app(layouts: &[&str]) -> LayoutsApp {
    LayoutsApp::new(layouts.iter().map(|spelled| Layout::parse(spelled)).collect(), LST)
}

#[test]
fn every_layout_not_in_use_is_an_item_by_name_with_its_spelling() {
    let items = app(&["us"]).items();

    let rows: Vec<(&str, &str, Option<&str>)> = items
        .iter()
        .map(|item| (item.id.as_str(), item.title.as_str(), item.subtitle.as_deref()))
        .collect();
    assert_eq!(
        rows,
        [
            ("layout:cz", "Czech", Some("cz")),
            ("layout:us(intl)", "English (US, intl., with dead keys)", Some("us(intl)")),
            ("layout:it", "Italian", Some("it")),
        ]
    );
}

#[test]
fn the_items_stay_in_the_app_and_are_found_by_their_spelling() {
    let items = app(&["us"]).items();
    let czech = items.iter().find(|item| item.id == "layout:cz").unwrap();

    assert!(czech.app_only);
    assert!(czech.keywords.contains(&"cz".to_owned()));
    assert_eq!(czech.kind.name(), KIND);
    assert_eq!(czech.actions.iter().map(|action| action.id.as_str()).collect::<Vec<_>>(), [ADD]);
}

#[test]
fn add_runs_kubaos_keyboard_add_and_closes() {
    let runner = FakeRunner::default();

    let ran = app(&["us"]).run("layout:us(intl)", ADD, &runner);

    assert!(matches!(ran, Ok(Ran::Close)));
    assert_eq!(runner.ran(), ["kubaos keyboard add us(intl)"]);
}

#[test]
fn a_refused_add_reads_as_kubaos_keyboards_last_line() {
    let runner = FakeRunner {
        failing: Some("kubaos: at most 4 keyboard layouts, XKB holds no more\n".to_owned()),
        ..FakeRunner::default()
    };

    let ran = app(&["us"]).run("layout:it", ADD, &runner);

    assert_eq!(ran.err().as_deref(), Some("At most 4 keyboard layouts, XKB holds no more"));
}

#[test]
fn an_unknown_item_or_action_is_refused_and_runs_nothing() {
    let runner = FakeRunner::default();

    assert!(app(&["us"]).run("keyboard-add", ADD, &runner).is_err());
    assert!(app(&["us"]).run("layout:it", "remove", &runner).is_err());
    assert!(runner.ran().is_empty());
}
