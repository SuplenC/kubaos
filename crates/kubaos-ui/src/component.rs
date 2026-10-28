//! The closed component set. A new variant is a change to the design language first, then here.
//!
//! Every component takes the [`Look`](crate::Look) it is drawn with and a focus flag from the app,
//! which owns where the keyboard is; none takes a length, colour or duration of its own.
//!
//! # Owns
//!
//! - Buttons with a label or an icon alone ([`button`]) and the hint naming an icon button
//!   ([`hint`]), the text field ([`field`]), the switch ([`switch`]), the slider ([`slider`]),
//!   checkboxes and radio buttons ([`check`]), list rows and their well ([`list`]), chips
//!   ([`chip`]), cards ([`card`]), dialogs ([`dialog`]), toasts ([`toast`]), progress
//!   ([`progress`]), empty states ([`empty`]), errors with a hint and a log ([`error`]), keycaps
//!   and key hints ([`keycap`]), pictures in a box of a fixed aspect ([`media`]) and pictures to
//!   pick ([`picture`]).
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
pub mod keycap;
mod layer;
pub mod list;
pub mod media;
pub mod picture;
pub mod progress;
pub mod slider;
pub mod switch;
pub mod toast;

pub(crate) use layer::Pin;
pub(crate) use layer::layered;
