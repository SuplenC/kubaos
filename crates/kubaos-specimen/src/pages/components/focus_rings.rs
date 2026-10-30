//! The components page's "Focus" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/FocusRings.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The tube itself: `kubaos_ui::focus`.

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::widget;
use iced::widget::container;
use kubaos_ui::Look;
use kubaos_ui::focus::RingColor;
use kubaos_ui::focus::focus_ring;
use kubaos_ui::focus::focus_ring_on_fill;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::row;
use kubaos_ui::look::radius;
use kubaos_ui::snap::snap;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Radius;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Focus",
    lede: Some(
        "The neon tube on the edge of whatever the keyboard is on: the accent, the cold one where the accent means picked, the error colour on a wrong value, and the hot core outside a fill.",
    ),
    view,
};

/// A box reading `words` in `ink` on `fill`, at `corner`: what the tube lights.
fn target<'a>(look: &Look, words: &'a str, fill: Color, ink: Color, corner: Radius) -> Element<'a, Message> {
    widget::container(snap(look.scale, text(words, Type::Body, ink)))
        .padding(Inset::xy(Space::S16, Space::S8).padding())
        .style(move |_theme| container::Style {
            background: Some(Background::Color(fill)),
            border: Border::default().rounded(radius(corner)),
            ..container::Style::default()
        })
        .into()
}

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let high = c.surface_container_high;
    row(look, Space::S32)
        .center()
        .push(focus_ring(
            look,
            target(look, "A control", high, c.on_surface, Radius::Control),
            Radius::Control,
            true,
            RingColor::Accent,
        ))
        .push(focus_ring(
            look,
            target(
                look,
                "A picked card",
                c.secondary_container,
                c.on_secondary_container,
                Radius::Surface,
            ),
            Radius::Surface,
            true,
            RingColor::Cold,
        ))
        .push(focus_ring(
            look,
            target(look, "A wrong value", high, c.on_surface, Radius::Control),
            Radius::Control,
            true,
            RingColor::Error,
        ))
        .push(focus_ring_on_fill(
            look,
            target(look, "On a fill", c.primary, c.on_primary, Radius::Control),
            Radius::Control,
            true,
            RingColor::Accent,
        ))
        .into()
}
