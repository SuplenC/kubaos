//! The keys a sidebar page answers, read from iced's keyboard events. What each does depends on
//! where the keyboard is, which [`super::SidebarPage`] decides.

use iced::keyboard;
use iced::keyboard::key::Named;

/// A key press, as the sidebar page reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    /// ↑, and Ctrl+P or Ctrl+K, which move in a list from inside a field too.
    Up,
    /// ↓, and Ctrl+N or Ctrl+J.
    Down,
    /// ←; with Ctrl or Alt, a word at a time in a field.
    Left {
        word: bool,
    },
    /// →; with Ctrl or Alt, a word at a time in a field.
    Right {
        word: bool,
    },
    Home,
    End,
    PageUp,
    PageDown,
    /// Once per press, never on key repeat; Shift+Enter steps back in the find box.
    Enter {
        shift: bool,
    },
    Space,
    Esc,
    Tab {
        shift: bool,
    },
    /// Ctrl+F. `/` is [`Key::Text`], since in a field it is typed.
    Find,
    /// Ctrl+Page Up and Ctrl+Page Down: the section before or after, from anywhere.
    PreviousSection,
    NextSection,
    Backspace {
        word: bool,
    },
    Delete {
        word: bool,
    },
    /// Ctrl+A.
    SelectAll,
    /// Ctrl+U: the field emptied.
    Clear,
    /// What a key types: in a field, text; off one, `/`, j, k, h and l are moves.
    Text(String),
}

/// The [`Key`] `event` is, if the sidebar page answers it.
pub fn translate(event: &keyboard::Event) -> Option<Key> {
    let keyboard::Event::KeyPressed {
        key,
        modifiers,
        repeat,
        text,
        ..
    } = event
    else {
        return None;
    };
    let ctrl = modifiers.control() || modifiers.logo();
    let word = modifiers.control() || modifiers.alt();
    let shift = modifiers.shift();
    match key.as_ref() {
        keyboard::Key::Named(named) => match named {
            Named::Enter => (!repeat).then_some(Key::Enter { shift }),
            Named::Escape => Some(Key::Esc),
            Named::Tab => Some(Key::Tab { shift }),
            Named::ArrowUp => Some(Key::Up),
            Named::ArrowDown => Some(Key::Down),
            Named::ArrowLeft => Some(Key::Left { word }),
            Named::ArrowRight => Some(Key::Right { word }),
            Named::Home => Some(Key::Home),
            Named::End => Some(Key::End),
            Named::PageUp if ctrl => Some(Key::PreviousSection),
            Named::PageDown if ctrl => Some(Key::NextSection),
            Named::PageUp => Some(Key::PageUp),
            Named::PageDown => Some(Key::PageDown),
            Named::Space if !ctrl => Some(Key::Space),
            Named::Backspace => Some(Key::Backspace { word }),
            Named::Delete => Some(Key::Delete { word }),
            _ => None,
        },
        keyboard::Key::Character(c) if ctrl => match c {
            "n" | "N" | "j" | "J" => Some(Key::Down),
            "p" | "P" | "k" | "K" => Some(Key::Up),
            "f" | "F" => Some(Key::Find),
            "a" | "A" => Some(Key::SelectAll),
            "u" | "U" => Some(Key::Clear),
            _ => None,
        },
        _ => text
            .as_ref()
            .map(ToString::to_string)
            .filter(|t| !t.is_empty() && !t.chars().any(char::is_control))
            .map(Key::Text),
    }
}

#[cfg(test)]
mod tests;
