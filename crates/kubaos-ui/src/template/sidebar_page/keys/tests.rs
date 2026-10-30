use iced::keyboard;
use iced::keyboard::Modifiers;
use iced::keyboard::key::Named;
use iced::keyboard::key::NativeCode;
use iced::keyboard::key::Physical;

use super::Key;
use super::translate;

fn press(key: keyboard::Key, modifiers: Modifiers, text: Option<&str>, repeat: bool) -> keyboard::Event {
    keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: Physical::Unidentified(NativeCode::Unidentified),
        location: keyboard::Location::Standard,
        modifiers,
        text: text.map(Into::into),
        repeat,
    }
}

fn named(named: Named, modifiers: Modifiers) -> Option<Key> {
    translate(&press(keyboard::Key::Named(named), modifiers, None, false))
}

fn character(c: &str, modifiers: Modifiers) -> Option<Key> {
    translate(&press(keyboard::Key::Character(c.into()), modifiers, Some(c), false))
}

#[test]
fn the_shell_lists_control_keys_move_up_and_down() {
    for c in ["n", "j"] {
        assert_eq!(character(c, Modifiers::CTRL), Some(Key::Down));
    }
    for c in ["p", "k"] {
        assert_eq!(character(c, Modifiers::CTRL), Some(Key::Up));
    }
}

#[test]
fn plain_letters_are_text_so_a_field_can_type_them() {
    assert_eq!(character("j", Modifiers::empty()), Some(Key::Text("j".into())));
    assert_eq!(character("/", Modifiers::empty()), Some(Key::Text("/".into())));
}

#[test]
fn control_page_keys_switch_section_and_plain_ones_page() {
    assert_eq!(named(Named::PageDown, Modifiers::CTRL), Some(Key::NextSection));
    assert_eq!(named(Named::PageUp, Modifiers::CTRL), Some(Key::PreviousSection));
    assert_eq!(named(Named::PageDown, Modifiers::empty()), Some(Key::PageDown));
}

#[test]
fn control_f_opens_find() {
    assert_eq!(character("f", Modifiers::CTRL), Some(Key::Find));
}

#[test]
fn enter_fires_once_per_press_and_carries_shift() {
    let held = press(keyboard::Key::Named(Named::Enter), Modifiers::empty(), None, true);
    assert_eq!(translate(&held), None);
    assert_eq!(named(Named::Enter, Modifiers::SHIFT), Some(Key::Enter { shift: true }));
}

#[test]
fn tab_carries_shift() {
    assert_eq!(named(Named::Tab, Modifiers::SHIFT), Some(Key::Tab { shift: true }));
}
