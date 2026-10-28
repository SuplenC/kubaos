//! The components page's "Chips" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Chips.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::chip::ChipKind;
use kubaos_ui::component::chip::chip;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::row;
use kubaos_ui::tokens::Space;

use crate::specimen::Message;
use super::Section;

pub const SECTION: Section = Section {
    title: "Chips",
    lede: None,
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let mut line = row(look, Space::S8).center();
    for kind in ChipKind::ALL {
        let label = match kind {
            ChipKind::Neutral => "2 TB",
            ChipKind::Selected => "English (US)",
            ChipKind::Accent => "Recommended",
            ChipKind::Cold => "Demo",
            ChipKind::Filled => "Reboot",
        };
        line = line.push(chip(look, kind, label, None, None, None::<Message>));
    }
    line = line
        .push(chip(
            look,
            ChipKind::Selected,
            "Default",
            Some(Icon::Check),
            None,
            None::<Message>,
        ))
        .push(chip(
            look,
            ChipKind::Cold,
            "Demo",
            Some(Icon::Bolt),
            None,
            None::<Message>,
        ))
        .push(chip(
            look,
            ChipKind::Neutral,
            "Italian",
            None,
            Some(Message::Sample),
            Some(Message::Sample),
        ))
        .push(chip(
            look,
            ChipKind::Selected,
            "English (US)",
            Some(Icon::Check),
            None,
            Some(Message::Sample),
        ));
    line.into()
}
