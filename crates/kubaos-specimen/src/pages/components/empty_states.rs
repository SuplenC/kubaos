//! The components page's "Empty states" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/EmptyStates.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::card::card;
use kubaos_ui::component::empty::empty_state;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::grid;
use kubaos_ui::tokens::Space;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Empty states",
    lede: None,
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    grid(look, 2, Space::S24)
        .push(card(
            look,
            empty_state(
                look,
                Icon::Wifi,
                "No networks",
                "Nothing in range. Plug in a cable, or skip it.",
                Some(button("Skip").kind(Kind::Tonal).on_press(Message::Sample).view(look)),
            ),
        ))
        .push(card(
            look,
            empty_state(look, Icon::Search, "Nothing matches", "Typo? Try fewer letters.", None),
        ))
        .into()
}
