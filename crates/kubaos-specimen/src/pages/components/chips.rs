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

use super::Section;
use crate::specimen::Message;

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
        line = line.push(chip::<Message>(kind, label).view(look));
    }
    line = line
        .push(
            chip::<Message>(ChipKind::Selected, "Default")
                .icon(Icon::Check)
                .view(look),
        )
        .push(chip::<Message>(ChipKind::Cold, "Demo").icon(Icon::Bolt).view(look))
        .push(
            chip(ChipKind::Neutral, "Italian")
                .on_press(Message::Sample)
                .on_remove(Message::Sample)
                .view(look),
        )
        .push(
            chip(ChipKind::Selected, "English (US)")
                .icon(Icon::Check)
                .on_remove(Message::Sample)
                .view(look),
        )
        .push(
            chip(ChipKind::Neutral, "German")
                .on_press(Message::Sample)
                .on_remove(Message::Sample)
                .disabled(true)
                .view(look),
        );
    line.into()
}
