//! The components page's "Keycaps and key hints" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Keycaps.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::keycap::key_hints;
use kubaos_ui::component::keycap::keycap;
use kubaos_ui::layout::row;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use crate::specimen::Message;
use super::Section;

pub const SECTION: Section = Section {
    title: "Keycaps and key hints",
    lede: Some("Keys drawn as keys, as tall as a line of body text so they sit inline with it, then what each does."),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let caps = row(look, Space::S8).center().extend(
        ["Esc", "Tab", "Enter", "↵", "K", "Ctrl", "← →", "+"]
            .into_iter()
            .map(|key| keycap(look, key)),
    );
    let inline = row(look, Space::S4)
        .center()
        .push(text("Press", Type::Body, c.on_surface))
        .push(keycap(look, "Space"))
        .push(text("to pick a layout, or", Type::Body, c.on_surface))
        .push(keycap(look, "Del"))
        .push(text("to drop it.", Type::Body, c.on_surface));
    stack(look, Space::S16)
        .push(caps)
        .push(key_hints(
            look,
            &[("Enter", "continue"), ("Ctrl+K", "actions"), ("Esc", "back")],
        ))
        .push(key_hints(
            look,
            &[("↵", "Open"), ("Tab", "Enter app"), ("Ctrl+K", "Actions")],
        ))
        .push(inline)
        .into()
}
