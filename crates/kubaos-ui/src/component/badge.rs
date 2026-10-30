//! Badges: a count, or a dot, over an icon's corner, saying what is new about it.
//!
//! # Owns
//!
//! - [`badge`] and [`accent_badge`], and [`MOST`], the most one counts to.
//!
//! # Not here
//!
//! - The icon tile it sits on: [`crate::component::icon_tile`].

use iced::Background;
use iced::Border;
use iced::Element;
use iced::widget;
use iced::widget::container;

use crate::layout::Inset;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::snap::centre_x;
use crate::snap::snap;
use crate::text::text;
use crate::tokens::Radius;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// The most a badge counts to; past it, it reads as this and a plus.
pub const MOST: u32 = 99;

/// A badge: `count` in a pill a line of label text tall, or a dot when there is no count, in the
/// error colour so it reads over any icon. One digit makes a circle; more pad 4 either side.
pub fn badge<'a, M: 'a>(look: &Look, count: Option<u32>) -> Element<'a, M> {
    toned_badge(look, count, false)
}

/// A [`badge`] in the accent: something unread rather than something wrong, as an unread
/// notification's dot on its tile.
pub fn accent_badge<'a, M: 'a>(look: &Look, count: Option<u32>) -> Element<'a, M> {
    toned_badge(look, count, true)
}

fn toned_badge<'a, M: 'a>(look: &Look, count: Option<u32>, accent: bool) -> Element<'a, M> {
    let c = look.colors;
    let (fill, ink) = if accent {
        (c.primary, c.on_primary)
    } else {
        (c.error, c.on_error)
    };
    let tone = if accent { "accent" } else { "" };
    let pill_style = move |_theme: &iced::Theme| container::Style {
        background: Some(Background::Color(fill)),
        border: Border::default().rounded(radius(Radius::Full)),
        ..container::Style::default()
    };
    let Some(n) = count else {
        let side = f32::from(Size::BadgeDot.px());
        let dot = widget::container(widget::Space::new())
            .width(side)
            .height(side)
            .style(pill_style);
        return probe::component("Badge", ["dot", tone, ""], Fixed::HEIGHT, dot);
    };
    let words = if n > MOST { format!("{MOST}+") } else { n.to_string() };
    let height = f32::from(Size::Badge.px());
    let label = snap(look.scale, text(words, Type::Label, ink));
    let pill = if n < 10 {
        widget::container(centre_x(look.scale, label)).width(height)
    } else {
        widget::container(label).padding(Inset::x(Space::S4).padding())
    };
    let view = pill.height(height).style(pill_style);
    probe::component("Badge", ["count", tone, ""], Fixed::HEIGHT, view)
}

#[cfg(test)]
mod tests;
