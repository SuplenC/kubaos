//! The components page's "Tiles" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Tiles.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::tile::tile;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::grid;
use kubaos_ui::tokens::Space;

use crate::specimen::Message;
use super::Section;

pub const SECTION: Section = Section {
    title: "Tiles",
    lede: Some("Quick toggles as tall as a two-line row, filled with the accent while on. The detail arrow stays quiet: no fill, a state layer under the pointer."),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    grid(look, 4, Space::S16)
        .push(
            tile(Icon::SignalWifi4Bar, "Wi-Fi")
                .subtitle("Shinjuku-5G")
                .checked(true)
                .on_press(Message::Sample)
                .view(look),
        )
        .push(
            tile(Icon::Bluetooth, "Bluetooth")
                .subtitle("Off")
                .on_press(Message::Sample)
                .view(look),
        )
        .push(
            tile(Icon::Keyboard, "Keyboard")
                .subtitle("US · 1 of 2")
                .checked(true)
                .on_press(Message::Sample)
                .on_detail(Message::Sample)
                .view(look),
        )
        .push(
            tile(Icon::Key, "Proxy")
                .subtitle("Off")
                .on_press(Message::Sample)
                .on_detail(Message::Sample)
                .focused(true)
                .view(look),
        )
        .into()
}
