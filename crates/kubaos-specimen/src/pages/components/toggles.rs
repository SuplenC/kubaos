//! The components page's "Switches, checkboxes and radio buttons" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Toggles.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::check::Tick;
use kubaos_ui::component::check::checkbox;
use kubaos_ui::component::check::radio;
use kubaos_ui::component::switch::switch;
use kubaos_ui::layout::row;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Switches, checkboxes and radio buttons",
    lede: None,
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let label = |s: &'static str| text(s, Type::Small, c.on_surface_variant);
    let switches = row(look, Space::S32)
        .center()
        .push(
            stack(look, Space::S8)
                .push(switch(look, false, 1.0, Some(Message::Sample), false))
                .push(label("Off")),
        )
        .push(
            stack(look, Space::S8)
                .push(switch(look, true, 1.0, Some(Message::Sample), false))
                .push(label("On")),
        )
        .push(
            stack(look, Space::S8)
                .push(switch(look, true, 0.5, Some(Message::Sample), false))
                .push(label("Halfway")),
        )
        .push(
            stack(look, Space::S8)
                .push(switch(look, true, 1.0, Some(Message::Sample), true))
                .push(label("Focused")),
        )
        .push(
            stack(look, Space::S8)
                .push(switch::<Message>(look, false, 1.0, None, false))
                .push(label("Disabled off")),
        )
        .push(
            stack(look, Space::S8)
                .push(switch::<Message>(look, true, 1.0, None, false))
                .push(label("Disabled on")),
        );
    let checks = row(look, Space::S32)
        .center()
        .push(checkbox(look, "Bluetooth", Tick::Empty, Some(Message::Sample), false))
        .push(checkbox(
            look,
            "Some wallpapers",
            Tick::Partial,
            Some(Message::Sample),
            false,
        ))
        .push(checkbox(
            look,
            "Encrypt the disk",
            Tick::Full,
            Some(Message::Sample),
            false,
        ))
        .push(checkbox(look, "Focused", Tick::Full, Some(Message::Sample), true))
        .push(checkbox::<Message>(look, "Disabled", Tick::Full, None, false));
    let radios = row(look, Space::S32)
        .center()
        .push(radio(look, "Keep Windows", false, Some(Message::Sample), false))
        .push(radio(look, "Erase everything", true, Some(Message::Sample), false))
        .push(radio(look, "Focused", false, Some(Message::Sample), true))
        .push(radio::<Message>(look, "Disabled", true, None, false));
    stack(look, Space::S24).push(switches).push(checks).push(radios).into()
}
