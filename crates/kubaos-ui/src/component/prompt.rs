//! Prompts: one value asked for in a popover.
//!
//! A prompt is its label in `label` type with an Enter keycap at the label row's end, the field
//! under it, and the field's note under that when there is something to say: [`labelled`],
//! [`keycap`] and the app's [`field`](crate::component::field::field) and
//! [`note`](crate::component::field::note), nothing of its own. Floating, it is on a popover of its
//! own, `size.menu` wide, `space.s8` under the text it changes, its label and field lined up with
//! that text's start, or its end for text at a panel's end. Inside a menu's popover, between its
//! separators, it has no popover of its own, so its field's edges land where a floating one's do.
//!
//! It opens with the keyboard in its field and the whole value selected ([`Caret::all`]). Enter
//! takes the value; Esc, or a click outside a floating one, cancels. A value it can't take turns
//! the field [`State::Invalid`] with the reason in its note, and it stays open until the value is
//! one it takes. Floating, it fades in over `short` on `emphasized_decel` and out over `micro` on
//! `emphasized_accel`.
//!
//! [`Caret::all`]: crate::component::field::Caret::all
//! [`State::Invalid`]: crate::component::field::State::Invalid
//!
//! # Owns
//!
//! - [`prompt`], built up with [`Prompt`].
//!
//! # Not here
//!
//! - The value, checking it, the keys and floating the popover at its anchor: the app's, as with
//!   every field and popover. The shell's prompts do all of it themselves (`Prompt` in
//!   `KubaOS.Components`).

use std::fmt;

use iced::Element;

use crate::component::field::labelled;
use crate::component::keycap::keycap;
use crate::component::menu::popover;
use crate::layout::Extent;
use crate::look::Look;
use crate::probe;
use crate::probe::Fixed;
use crate::tokens::Size;

/// A prompt asking for `label`, built up then drawn with [`Prompt::view`].
pub struct Prompt<'a, M> {
    label: &'a str,
    field: Element<'a, M>,
    note: Option<Element<'a, M>>,
    in_menu: bool,
}

impl<M> fmt::Debug for Prompt<'_, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Prompt")
            .field("label", &self.label)
            .field("note", &self.note.is_some())
            .field("in_menu", &self.in_menu)
            .finish_non_exhaustive()
    }
}

/// A floating prompt asking for `label` in `field`: a [`field`](crate::component::field::field)
/// holding the value.
pub fn prompt<'a, M>(label: &'a str, field: impl Into<Element<'a, M>>) -> Prompt<'a, M> {
    Prompt {
        label,
        field: field.into(),
        note: None,
        in_menu: false,
    }
}

impl<'a, M: 'a> Prompt<'a, M> {
    /// The field's [`note`](crate::component::field::note): the reason a value can't be taken, or
    /// what one reads as.
    #[must_use]
    pub fn note(self, note: impl Into<Element<'a, M>>) -> Self {
        Prompt {
            note: Some(note.into()),
            ..self
        }
    }

    /// Inside a menu's popover, between its separators: no popover of its own.
    #[must_use]
    pub fn in_menu(self) -> Self {
        Prompt { in_menu: true, ..self }
    }

    pub fn view(self, look: &Look) -> Element<'a, M> {
        let asked = labelled(look, self.label, Some(keycap(look, "Enter")), self.field, self.note);
        let fixed = Fixed {
            width: true,
            ..Fixed::NONE
        };
        if self.in_menu {
            probe::component("Prompt", ["menu", "", ""], fixed, asked)
        } else {
            let card = popover(look, Extent::Fixed(Size::Menu), asked);
            probe::component("Prompt", ["", "", ""], fixed, card)
        }
    }
}
