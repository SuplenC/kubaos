//! The components page's "Notes" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Notes.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::field::note;
use kubaos_ui::layout::stack;
use kubaos_ui::tokens::Space;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Notes",
    lede: Some("The small line under a field: muted, or the error colour when the value is wrong."),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    stack(look, Space::S8)
        .push(note(look, "Letters, digits and dashes.", false))
        .push(note(look, "That's not an email. Typo?", true))
        .into()
}
