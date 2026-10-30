//! The components page's "Hints" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Hints.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::icon_button;
use kubaos_ui::component::chip::ChipKind;
use kubaos_ui::component::chip::chip;
use kubaos_ui::component::hint::hint;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::row;
use kubaos_ui::tokens::Space;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Hints",
    lede: Some(
        "A control's name in a small bubble 8 under it, while the pointer or the keyboard is on it: an icon button's label, a chip's tooltip.",
    ),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let settings = icon_button(Icon::Settings, "Settings")
        .kind(Kind::Tonal)
        .on_press(Message::Sample)
        .focused(true)
        .view(look);
    let day_first = chip(ChipKind::Neutral, "Day first")
        .on_press(Message::Sample)
        .view(look);
    row(look, Space::S32)
        .center()
        .push(settings)
        .push(hint(look, day_first, "Your locale's order", true))
        .into()
}
