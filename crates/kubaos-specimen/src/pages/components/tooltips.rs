//! The components page's "Tooltips" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Tooltips.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::chip::ChipKind;
use kubaos_ui::component::chip::chip;
use kubaos_ui::component::keycap::key_hints;
use kubaos_ui::component::tooltip::tooltip;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::stack;
use kubaos_ui::tokens::Space;

use crate::specimen::Message;
use super::Section;

pub const SECTION: Section = Section {
    title: "Tooltips",
    lede: Some("What a control does, on a popover 8 above it while the pointer or the keyboard is on it; drawn here in place, over the chip it explains."),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let keys = stack(look, Space::S8)
        .push(key_hints(look, &[("Left Alt+E", "then"), ("E", "types é")]))
        .push(key_hints(look, &[("Left Alt+U", "then"), ("O", "types ö")]));
    let card = tooltip(
        look,
        "Mac accents on Left Alt",
        "Type accents as a Mac does: hold Left Alt with the accent's key, then type the letter.",
        Some(keys.into()),
    );
    stack(look, Space::S8)
        .push(card)
        .push(chip(
            look,
            ChipKind::Selected,
            "Mac accents",
            Some(Icon::KeyboardOptionKey),
            Some(Message::Sample),
            None,
        ))
        .into()
}
