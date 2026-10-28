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
use kubaos_ui::layout::split;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::Section;
use super::samples::STATES;
use super::samples::button_in;
use super::samples::size_name;
use super::samples::state_heads;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Buttons",
    lede: Some(
        "Filled, Tonal, Text and Danger, in small, regular and large, each resting, hovered, pressed, focused and disabled.",
    ),
    view,
};

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
