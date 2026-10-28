//! The components page's "Page scroll" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/PageScroll.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::button::ButtonSize;
use kubaos_ui::component::button::Kind;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::scroll;
use kubaos_ui::layout::stack;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;

use super::Section;
use super::buttons::button_in;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Page scroll",
    lede: Some(
        "A page's or a panel's content taller than its room. It has no edge of its own, so it reaches past its box into the room around it, as far as a glow does and up to the text above it, with its content that far in: the focused button at its top keeps its whole glow, and what runs past that room is cut there.",
    ),
    view,
};

/// Every kind of button, twice, in a page's scroll a log tall: more than it shows, the first
/// focused at its top edge.
fn view<'a>(look: &Look) -> Element<'a, Message> {
    let mut content = stack(look, Space::S8);
    for (i, kind) in Kind::ALL.into_iter().chain(Kind::ALL).enumerate() {
        // The first is focused, STATES' fourth; the rest are resting.
        let state = if i == 0 { 3 } else { 0 };
        content = content.push(button_in(look, kind, ButtonSize::Md, state));
    }
    grid(look, 3, Space::S24)
        .push(
            scroll(look, content)
                .height(Extent::Fixed(Size::Log))
                .room_above(Space::S16),
        )
        .into()
}
