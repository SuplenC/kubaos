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

/// A row of `choices`, the `picked` one showing, built up then drawn with [`Tabs::view`].
#[derive(Debug)]
pub struct Tabs<'a, 'c, F> {
    choices: &'c [Tab<'a>],
    picked: usize,
    on_pick: F,
    focused: bool,
}

/// `choices` side by side, 4 apart, as small buttons: the `picked` one tonal, the rest text. A
/// press sends `on_pick` with the choice's index. The row is one stop for the keyboard; its owner
/// moves the pick with ← and →.
pub fn tabs<'a, 'c, M, F: Fn(usize) -> M>(choices: &'c [Tab<'a>], picked: usize, on_pick: F) -> Tabs<'a, 'c, F> {
    Tabs {
        choices,
        picked,
        on_pick,
        focused: false,
    }
}

impl<'a, M: Clone + 'a, F: Fn(usize) -> M> Tabs<'a, '_, F> {
    /// Whether the row has the keyboard: the focus ring goes round the picked one.
    #[must_use]
    pub fn focused(self, focused: bool) -> Self {
        Tabs { focused, ..self }
    }

    pub fn view(self, look: &Look) -> Element<'a, M> {
        let buttons = self.choices.iter().enumerate().map(|(index, choice)| {
            let current = index == self.picked;
            let kind = if current { Kind::Tonal } else { Kind::Text };
            let b = button(choice.label)
                .kind(kind)
                .size(ButtonSize::Sm)
                .on_press((self.on_pick)(index))
                .focused(self.focused && current);
            match choice.icon {
                Some(glyph) => b.icon(glyph),
                None => b,
            }
            .view(look)
        });
        let line = row(look, Space::S4).center().extend(buttons);
        probe::component("Tabs", ["", "", ""], Fixed::HEIGHT, line)
    }
}
