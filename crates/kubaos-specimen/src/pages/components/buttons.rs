//! The components page's "Buttons" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Buttons.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::button::ButtonSize;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::Preview;
use kubaos_ui::component::button::button;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::split;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use crate::specimen::Message;
use super::Section;

pub const SECTION: Section = Section {
    title: "Buttons",
    lede: Some("Filled, Tonal, Text and Danger, in small, regular and large, each resting, hovered, pressed, focused and disabled."),
    view,
};

/// The states a button is shown in, left to right.
pub(super) const STATES: [&str; 5] = ["Resting", "Hovered", "Pressed", "Focused", "Disabled"];

pub(super) fn size_name(size: ButtonSize) -> String {
    let height = match size {
        ButtonSize::Sm => Size::ControlSm,
        ButtonSize::Md => Size::ControlMd,
        ButtonSize::Lg => Size::ControlLg,
    };
    format!("{size:?} · {}", height.px())
}

/// A label button of `kind` and `size` in the `state`th of [`STATES`].
pub(super) fn button_in<'a>(look: &Look, kind: Kind, size: ButtonSize, state: usize) -> Element<'a, Message> {
    let (label, glyph) = match kind {
        Kind::Filled => ("Continue", Icon::ArrowForward),
        Kind::Tonal => ("Save log", Icon::Save),
        Kind::Text => ("Back", Icon::ArrowBack),
        Kind::Danger => ("Erase disk", Icon::Warning),
    };
    let b = button::<Message>(label).kind(kind).size(size);
    let b = if kind == Kind::Filled {
        b.trailing_icon(glyph)
    } else {
        b.icon(glyph)
    };
    match state {
        0 => b.on_press(Message::Sample),
        1 => b.on_press(Message::Sample).preview(Preview::Hovered),
        2 => b.on_press(Message::Sample).preview(Preview::Pressed),
        3 => b.on_press(Message::Sample).focused(true),
        _ => b,
    }
    .view(look)
}

/// The states' names over a table's columns, after the kinds' column.
pub(super) fn state_heads<'a>(look: &Look) -> Vec<Element<'a, Message>> {
    let c = look.colors;
    std::iter::once(text("", Type::Label, c.on_surface_variant))
        .chain(STATES.map(|state| text(state, Type::Label, c.on_surface_variant)))
        .collect()
}

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let mut sizes = stack(look, Space::S32);
    for size in ButtonSize::ALL {
        let mut table = split(look, &[1, 2, 2, 2, 2, 2], Space::S16)
            .center()
            .extend(state_heads(look));
        for kind in Kind::ALL {
            table = table.push(text(format!("{kind:?}"), Type::Label, c.on_surface_variant));
            for state in 0..STATES.len() {
                table = table.push(button_in(look, kind, size, state));
            }
        }
        sizes = sizes.push(
            stack(look, Space::S12)
                .push(text(size_name(size), Type::Subheading, c.on_surface))
                .push(table),
        );
    }
    sizes.into()
}
