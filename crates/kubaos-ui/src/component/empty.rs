//! Empty states: what a place with nothing in it says, and what to do about it.

use iced::Element;

use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::Extent;
use crate::layout::Gap;
use crate::layout::gap;
use crate::layout::stack;
use crate::look::Look;
use crate::snap::centre_x;
use crate::text::text;
use crate::tokens::IconSize;
use crate::tokens::Space;
use crate::tokens::Type;

/// `icon` over `title` and `body`, centred, with an optional action under them.
pub fn empty_state<'a, M: 'a>(
    look: &Look,
    glyph: Icon,
    title: &'a str,
    body: &'a str,
    action: Option<Element<'a, M>>,
) -> Element<'a, M> {
    let c = look.colors;
    let line = |content: Element<'a, M>| centre_x(look.scale, content);
    let mut column = stack(look, Gap::NONE)
        .push(line(icon(look, glyph, IconSize::S32, c.on_surface_variant)))
        .push(gap(Space::S12))
        .push(line(text(title, Type::Subheading, c.on_surface).into()))
        .push(gap(Space::S4))
        .push(line(text(body, Type::Body, c.on_surface_variant).into()));
    if let Some(action) = action {
        column = column.push(gap(Space::S16)).push(line(action));
    }
    column.width(Extent::Fill).into()
}
