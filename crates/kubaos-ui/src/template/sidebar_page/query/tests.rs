use super::Query;
use crate::component::field::Caret;
use crate::component::field::Pointer;

#[test]
fn typing_replaces_the_selection() {
    let mut query = Query::new("dark mode");
    query.select_all();

    query.insert("lid");

    assert_eq!(query, Query::new("lid"));
}

#[test]
fn backspace_takes_a_character_or_a_word() {
    let mut query = Query::new("turn screens off");
    query.backspace(false);
    assert_eq!(query.text, "turn screens of");

    query.backspace(true);

    assert_eq!(query.text, "turn screens ");
    assert_eq!(query.caret, Caret::at(13));
}

#[test]
fn edits_count_characters_not_bytes() {
    let mut query = Query::new("ząb");
    query.left(false);

    query.backspace(false);

    assert_eq!(query.text, "zb");
    assert_eq!(query.caret, Caret::at(1));
}

#[test]
fn a_double_press_selects_the_word_under_the_pointer() {
    let mut query = Query::new("sleep and lock");

    query.point(Pointer::Double(7));

    assert_eq!(query.caret.range(), (6, 9));
}

#[test]
fn clearing_empties_the_field() {
    let mut query = Query::new("battery");

    query.clear();

    assert!(query.is_empty());
}
