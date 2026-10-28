//! Tabs: one of a few views or values, picked from a row of small buttons. The same row is the
//! segmented control: a setting's few choices side by side (Always, On AC, Never).
//!
//! # Owns
//!
//! - [`tabs`], the row, and [`Tab`], one of its choices.
//!
//! # Not here
//!
//! - The buttons it is made of: [`crate::component::button`].

use iced::Element;

use crate::component::button::ButtonSize;
use crate::component::button::Kind;
use crate::component::button::button;
use crate::icon::Icon;
use crate::layout::row;
use crate::look::Look;
use crate::probe;
use crate::probe::Fixed;
use crate::tokens::Space;

/// One choice of a [`tabs`] row: its label and an optional symbol before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tab<'a> {
    label: &'a str,
    icon: Option<Icon>,
}

impl<'a> Tab<'a> {
    #[must_use]
    pub fn new(label: &'a str) -> Tab<'a> {
        Tab { label, icon: None }
    }

    /// A symbol before the label.
    #[must_use]
    pub fn icon(self, icon: Icon) -> Tab<'a> {
        Tab {
            icon: Some(icon),
            ..self
        }
    }
}

/// `choices` side by side, 4 apart, as small buttons: the `picked` one tonal, the rest text. A
/// press sends `on_pick` with the choice's index. The row is one stop for the keyboard, which
/// moves the pick with ← and →; while it has the keyboard (`focused`) the ring goes round the
/// picked one.
pub fn tabs<'a, M: Clone + 'a>(
    look: &Look,
    choices: &[Tab<'a>],
    picked: usize,
    on_pick: impl Fn(usize) -> M,
    focused: bool,
) -> Element<'a, M> {
    let buttons = choices.iter().enumerate().map(|(index, choice)| {
        let current = index == picked;
        let kind = if current { Kind::Tonal } else { Kind::Text };
        let b = button(choice.label)
            .kind(kind)
            .size(ButtonSize::Sm)
            .on_press(on_pick(index))
            .focused(focused && current);
        match choice.icon {
            Some(glyph) => b.icon(glyph),
            None => b,
        }
        .view(look)
    });
    let line = row(look, Space::S4).center().extend(buttons);
    probe::component("Tabs", ["", "", ""], Fixed::HEIGHT, line)
}
