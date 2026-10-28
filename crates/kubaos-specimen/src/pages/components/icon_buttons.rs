//! The components page's "Icon buttons" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/IconButtons.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::button::ButtonSize;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::Preview;
use kubaos_ui::component::button::icon_button;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::split;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use crate::specimen::Message;
use super::buttons::STATES;
use super::buttons::size_name;
use super::buttons::state_heads;
use super::Section;

pub const SECTION: Section = Section {
    title: "Icon buttons",
    lede: Some("Circles as tall as the label buttons of each size. The label is required: it shows in a hint on hover and while focused."),
    view,
};

/// An icon button of `kind` and `size` in the `state`th of [`STATES`].
fn icon_button_in<'a>(look: &Look, kind: Kind, size: ButtonSize, state: usize) -> Element<'a, Message> {
    let (glyph, label) = match kind {
        Kind::Filled => (Icon::ArrowForward, "Continue"),
        Kind::Tonal => (Icon::Settings, "Settings"),
        Kind::Text => (Icon::Close, "Close"),
        Kind::Danger => (Icon::PowerSettingsNew, "Shut down"),
    };
    let b = icon_button::<Message>(glyph, label).kind(kind).size(size);
    match state {
        0 => b.on_press(Message::Sample),
        1 => b.on_press(Message::Sample).preview(Preview::Hovered),
        2 => b.on_press(Message::Sample).preview(Preview::Pressed),
        3 => b.on_press(Message::Sample).focused(true),
        _ => b,
    }
    .view(look)
}

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let mut sizes = stack(look, Space::S32);
    for size in ButtonSize::ALL {
        // Rows 40 apart, so a focused button's hint has room under it. The columns are the label
        // buttons' above.
        let mut rows = stack(look, Space::S40).push(
            split(look, &[1, 2, 2, 2, 2, 2], Space::S16)
                .center()
                .extend(state_heads(look)),
        );
        for kind in Kind::ALL {
            let mut line = split(look, &[1, 2, 2, 2, 2, 2], Space::S16).center().push(text(
                format!("{kind:?}"),
                Type::Label,
                c.on_surface_variant,
            ));
            for state in 0..STATES.len() {
                line = line.push(icon_button_in(look, kind, size, state));
            }
            rows = rows.push(line);
        }
        sizes = sizes.push(
            stack(look, Space::S12)
                .push(text(size_name(size), Type::Subheading, c.on_surface))
                .push(rows),
        );
    }
    sizes.into()
}
