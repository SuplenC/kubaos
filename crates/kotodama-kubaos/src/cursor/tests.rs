use super::*;
use crate::fake::FakeRunner;

fn settings(style: Style, size: u32) -> CursorSettings {
    CursorSettings { style, size }
}

fn action_ids(item: &Item) -> Vec<&str> {
    item.actions.iter().map(|action| action.id.as_str()).collect()
}

#[test]
fn the_subtitle_says_the_style_and_the_size() {
    let item = item(Some(settings(Style::Accent, 32)));

    assert_eq!(item.title, "Cursor");
    assert_eq!(
        item.subtitle.as_deref(),
        Some("Accent, 32 px: the pointer in the wallpaper's accent colour")
    );
}

#[test]
fn the_next_style_comes_first_then_the_others_then_the_sizes() {
    let dark = item(Some(settings(Style::Dark, 24)));
    let classic = item(Some(settings(Style::Classic, 24)));

    assert_eq!(
        action_ids(&dark),
        ["accent", "dark", "classic", "size-24", "size-32", "size-48", "size"]
    );
    assert_eq!(action_ids(&classic)[..3], ["dark", "accent", "classic"]);
}

#[test]
fn while_the_settings_cant_be_read_the_styles_keep_their_order() {
    let item = item(None);

    assert_eq!(action_ids(&item)[..3], ["dark", "accent", "classic"]);
    assert!(item.subtitle.is_some());
}

#[test]
fn every_action_has_its_own_verbs() {
    let item = item(None);
    let mut verbs: Vec<&String> = item.actions.iter().flat_map(|action| &action.keywords).collect();
    let count = verbs.len();

    verbs.sort();
    verbs.dedup();

    assert!(item.actions.iter().all(|action| !action.keywords.is_empty()));
    assert_eq!(verbs.len(), count);
}

#[test]
fn each_action_sets_its_style_or_size() {
    let runner = FakeRunner::default();
    let given = BTreeMap::from([(PIXELS.to_owned(), " 40 ".to_owned())]);

    for action in ["classic", "size-48", SIZE] {
        assert_eq!(run(action, &given, &runner), Ok(Ran::Close), "{action}");
    }

    let launched: Vec<Vec<String>> = runner.launched.borrow().iter().map(|(argv, _)| argv.clone()).collect();
    assert_eq!(
        launched,
        [
            ["kubaos", "cursor", "style", "classic"],
            ["kubaos", "cursor", "size", "48"],
            ["kubaos", "cursor", "size", "40"],
        ]
    );
}

#[test]
fn a_size_that_isnt_a_number_or_an_unknown_action_says_why() {
    let runner = FakeRunner::default();
    let given = BTreeMap::from([(PIXELS.to_owned(), "huge".to_owned())]);

    assert!(run(SIZE, &given, &runner).is_err());
    assert!(run("neon", &BTreeMap::new(), &runner).is_err());
    assert!(runner.launched.borrow().is_empty());
}
