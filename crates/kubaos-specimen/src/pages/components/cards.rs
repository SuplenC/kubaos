//! The components page's "Cards" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Cards.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::card::card;
use kubaos_ui::component::card::choice_card;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Opacity;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Cards",
    lede: Some(
        "Picked cards take the accent, so their focus tube is the cold one. A card that can't be picked now is disabled.",
    ),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let mode = |title: &'static str, sub: &'static str| -> Element<'a, Message> {
        stack(look, Space::S4)
            .push(text(title, Type::Subheading, c.on_surface))
            .push(text(sub, Type::Small, c.on_surface_variant))
            .into()
    };
    let disabled_mode = |title: &'static str, sub: &'static str| -> Element<'a, Message> {
        let dim = look.faded(Opacity::DisabledContent.value());
        let c = dim.colors;
        stack(&dim, Space::S4)
            .push(text(title, Type::Subheading, c.on_surface))
            .push(text(sub, Type::Small, c.on_surface_variant))
            .into()
    };
    grid(look, 4, Space::S24)
        .push(card(
            look,
            stack(look, Space::S4)
                .push(text("Plain card", Type::Subheading, c.on_surface))
                .push(text(
                    "The well's surface and a soft hairline, 16 inside.",
                    Type::Small,
                    c.on_surface_variant,
                )),
        ))
        .push(choice_card(
            look,
            mode("Erase the disk", "Everything on it goes."),
            true,
            Some(Message::Sample),
            false,
        ))
        .push(choice_card(
            look,
            mode("Install alongside", "Keep Windows, shrink it."),
            false,
            Some(Message::Sample),
            false,
        ))
        .push(choice_card(
            look,
            mode("Focused, picked", "The cold tube."),
            true,
            Some(Message::Sample),
            true,
        ))
        .push(choice_card(
            look,
            disabled_mode("Keep it as it is", "No room left on this disk."),
            false,
            None,
            false,
        ))
        .into()
}
