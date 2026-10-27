//! The closed component set. A new variant is a change to the design language first, then here.
//!
//! Every component takes the [`Look`](crate::Look) it is drawn with and a focus flag from the app,
//! which owns where the keyboard is; none takes a length, colour or duration of its own.
//!
//! # Owns
//!
//! - Buttons with a label or an icon alone ([`button`]) and the hint naming an icon button
//!   ([`hint`]), the text field ([`field`]), the switch ([`switch`]), the slider ([`slider`]),
//!   checkboxes and radio buttons ([`check`]), list rows and their well ([`list`]), chips ([`chip`]), cards ([`card`]),
//!   dialogs ([`dialog`]), toasts ([`toast`]), progress ([`progress`]), empty states ([`empty`]),
//!   errors with a hint and a log ([`error`]).
//!
//! # Not here
//!
//! - The focus ring: [`crate::focus`]. Layout: [`crate::layout`].

pub mod button;
pub mod card;
pub mod check;
pub mod chip;
pub mod dialog;
pub mod empty;
pub mod error;
pub mod field;
pub mod hint;
pub mod list;
pub mod progress;
pub mod slider;
pub mod switch;
pub mod toast;

use iced::Background;
use iced::Color;
use iced::widget::button as iced_button;

use crate::color;
use crate::tokens::Opacity;

/// The state layer of a pressable surface: `content` laid over `base` while hovered or pressed.
/// A surface with no fill of its own gets the layer alone.
pub(crate) fn state_layer(base: Option<Color>, content: Color, status: iced_button::Status) -> Option<Background> {
    let layer = match status {
        iced_button::Status::Hovered => Opacity::Hover.value(),
        iced_button::Status::Pressed => Opacity::Press.value(),
        iced_button::Status::Active | iced_button::Status::Disabled => 0.0,
    };
    match base {
        Some(base) => Some(Background::Color(color::mix(
            base,
            Color { a: base.a, ..content },
            layer,
        ))),
        None if layer > 0.0 => Some(Background::Color(color::alpha(content, layer))),
        None => None,
    }
}
