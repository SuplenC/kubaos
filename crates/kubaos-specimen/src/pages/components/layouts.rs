//! The components page's "Split, Flow and Gap" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Layouts.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The primitives: `kubaos_ui::layout`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::chip::ChipKind;
use kubaos_ui::component::chip::chip;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::flow;
use kubaos_ui::layout::gap;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::split;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Split, Flow and Gap",
    lede: Some(
        "Columns by their share of the width, a label a quarter wide beside its value; chips flowing onto a new line where the next doesn't fit; and one larger step in a stack.",
    ),
    view,
};

/// The language chips the flow wraps.
const LANGUAGES: [&str; 8] = [
    "English (US)",
    "Italian",
    "German",
    "Japanese",
    "Polish",
    "French",
    "Spanish",
    "Portuguese (Brazil)",
];

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let mut facts = split(look, &[1, 3], Space::S8).center();
    for (label, value) in [("Hostname", "tuxedo"), ("Kernel", "6.17.0-42"), ("Uptime", "3 days")] {
        facts = facts.push(text(label, Type::Small, c.on_surface_variant)).push(text(
            value,
            Type::BodyStrong,
            c.on_surface,
        ));
    }
    let chips = LANGUAGES.map(|label| chip::<Message>(ChipKind::Neutral, label).view(look));
    let stepped = stack(look, Gap::NONE)
        .push(text("A title", Type::BodyStrong, c.on_surface))
        .push(gap(Space::S16))
        .push(text(
            "16 under it, with no gap of the stack's own.",
            Type::Small,
            c.on_surface_variant,
        ));
    grid(look, 3, Space::S24)
        .push(facts)
        .push(flow(look, Space::S8, chips))
        .push(stepped)
        .into()
}
