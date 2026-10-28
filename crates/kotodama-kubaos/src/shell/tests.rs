use std::fs;

use super::*;

const SEED: &str = include_str!("../../../../user-layer/desktop/quickshell/settings.json");

fn parsed(text: &str) -> serde_json::Value {
    serde_json::from_str(&uncommented(text)).expect("the settings read as JSON")
}

#[test]
fn the_seed_leaves_the_stutter_on() {
    assert_eq!(stutter(SEED), None);
}

#[test]
fn setting_a_key_in_the_seed_keeps_its_comments_and_examples() {
    let set = with(SEED, STUTTER_KEY, false).unwrap();

    assert_eq!(stutter(&set), Some(false));
    assert_eq!(parsed(&set), serde_json::json!({ "focusStutter": false }));
    for line in SEED.lines().filter(|line| line.trim_start().starts_with("//")) {
        assert!(set.contains(line), "lost {line:?}");
    }
}

#[test]
fn setting_a_key_that_is_set_changes_its_value_in_place() {
    let text = "{\n  \"focusStutter\": false,\n  \"menuMotion\": \"fades\"\n}\n";

    let set = with(text, STUTTER_KEY, true).unwrap();

    assert_eq!(set, "{\n  \"focusStutter\": true,\n  \"menuMotion\": \"fades\"\n}\n");
}

#[test]
fn a_key_added_beside_others_takes_its_comma() {
    let text = "// mine\n{ \"menuMotion\": \"fades\" }\n";

    let set = with(text, STUTTER_KEY, false).unwrap();

    assert_eq!(
        parsed(&set),
        serde_json::json!({ "menuMotion": "fades", "focusStutter": false })
    );
    assert!(set.starts_with("// mine\n"));
}

#[test]
fn a_key_added_after_an_uncommented_example_takes_its_comma() {
    let text = SEED.replace("  // \"menuMotion\": \"full\"", "  \"menuMotion\": \"full\"");

    let set = with(&text, STUTTER_KEY, false).unwrap();

    assert_eq!(
        parsed(&set),
        serde_json::json!({ "menuMotion": "full", "focusStutter": false })
    );
}

#[test]
fn a_key_on_one_line_with_others_is_left_alone() {
    let one_line = "{\"menuMotion\": \"fades\", \"focusStutter\": true}\n";
    let leading = "{\n  \"focusStutter\": true, \"menuMotion\": \"full\"\n}\n";

    assert!(matches!(
        with(one_line, STUTTER_KEY, false),
        Err(SettingsError::Crowded)
    ));
    assert!(matches!(with(leading, STUTTER_KEY, false), Err(SettingsError::Crowded)));
}

#[test]
fn an_empty_file_gets_an_object_of_its_own() {
    let set = with("", STUTTER_KEY, false).unwrap();

    assert_eq!(parsed(&set), serde_json::json!({ "focusStutter": false }));
}

#[test]
fn a_file_that_doesnt_read_is_left_alone() {
    let broken = "{ \"menuMotion\": }\n";

    assert!(with(broken, STUTTER_KEY, false).is_err());
}

#[test]
fn the_item_turns_the_stutter_off_and_on_in_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("desktop/quickshell/settings.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, SEED).unwrap();

    run(STUTTER, OFF, &file).unwrap();
    assert_eq!(stutter(&fs::read_to_string(&file).unwrap()), Some(false));
    run(STUTTER, TOGGLE, &file).unwrap();
    assert_eq!(stutter(&fs::read_to_string(&file).unwrap()), Some(true));
    run(STUTTER, ON, &file).unwrap();
    assert_eq!(stutter(&fs::read_to_string(&file).unwrap()), Some(true));
}

#[test]
fn the_item_makes_the_file_when_there_is_none() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("desktop/quickshell/settings.json");

    run(STUTTER, OFF, &file).unwrap();

    assert_eq!(stutter(&fs::read_to_string(&file).unwrap()), Some(false));
}

#[test]
fn the_seed_leaves_the_bell_to_show_only_with_notifications() {
    assert_eq!(value(SEED, BELL_KEY), None);
}

#[test]
fn the_bell_item_turns_always_show_the_bell_on_and_off_in_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    let read = || value(&fs::read_to_string(&file).unwrap(), BELL_KEY);

    run(BELL, TOGGLE, &file).unwrap();
    assert_eq!(read(), Some(true));
    run(BELL, TOGGLE, &file).unwrap();
    assert_eq!(read(), Some(false));
    run(BELL, ON, &file).unwrap();
    assert_eq!(read(), Some(true));
}

#[test]
fn the_bell_item_leaves_the_stutter_as_it_is() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    run(STUTTER, OFF, &file).unwrap();

    run(BELL, ON, &file).unwrap();

    assert_eq!(stutter(&fs::read_to_string(&file).unwrap()), Some(false));
}

#[test]
fn the_bell_item_is_found_by_what_people_type() {
    let item = items().into_iter().find(|item| item.id == BELL).expect("the bell's Item");

    assert!(item.title.contains("Always show the bell"));
    assert!(item.keywords.iter().any(|word| word == "notifications"));
    assert_eq!(ids().len(), items().len());
}

#[test]
fn an_item_that_isnt_a_shell_setting_is_refused() {
    let dir = tempfile::tempdir().unwrap();

    assert!(run("wallpaper", TOGGLE, &dir.path().join("settings.json")).is_err());
}

#[test]
fn the_stutter_item_is_found_by_what_people_type() {
    let item = items().remove(0);

    assert_eq!(item.id, STUTTER);
    assert!(item.title.contains("Focus stutter"));
    assert!(item.keywords.iter().any(|word| word == "flicker"));
    assert_eq!(
        item.actions.iter().map(|action| action.id.as_str()).collect::<Vec<_>>(),
        [TOGGLE, ON, OFF]
    );
}
