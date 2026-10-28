//! Tooltips: what a control does, on a card over it while the pointer is over it or the keyboard
//! is on it. Where a hint names a control in one line, a tooltip explains one: a title, the body
//! under it, and what it holds 8 below (its keys, as key hints).
//!
//! # Owns
//!
//! - [`tooltip`], the card, and [`with_tooltip`], which floats it over its control.
//!
//! # Not here
//!
//! - How a bubble floats beside its control: [`crate::component::hint`].

use iced::Element;

use crate::component::hint::Side;
use crate::component::hint::floating;
use crate::component::menu::popover;
use crate::layout::Extent;
use crate::layout::stack;
use crate::look::Look;
use crate::probe;
use crate::probe::Fixed;
use crate::text::text;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// The card: `title` in `body_strong`, `body` in `small` under it, and `extra` 8 below, on a
/// popover as wide as a menu.
pub fn tooltip<'a, M: 'a>(
    look: &Look,
    title: &'a str,
    body: &'a str,
    extra: Option<Element<'a, M>>,
) -> Element<'a, M> {
    let c = look.colors;
    let words = stack(look, Space::S4)
        .push(text(title, Type::BodyStrong, c.on_surface))
        .push(text(body, Type::Small, c.on_surface_variant));
    let mut content = stack(look, Space::S8).push(words);
    if let Some(extra) = extra {
        content = content.push(extra);
    }
    let card = popover(look, Extent::Fixed(Size::Menu), content);
    let fixed = Fixed {
        width: true,
        height: false,
        holds: false,
    };
    probe::component("Tooltip", ["", "", ""], fixed, card)
}

/// `content` with `tip` (a [`tooltip`]) floating 8 above it, starting where it starts and kept
/// inside the window, while hovered or while `shown` (the keyboard is on it).
pub fn with_tooltip<'a, M: 'a>(
    look: &Look,
    content: impl Into<Element<'a, M>>,
    tip: Element<'a, M>,
    shown: bool,
) -> Element<'a, M> {
    floating(look, content, tip, shown, Side::Above)
}
