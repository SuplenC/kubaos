//! The components page's "Rolling numbers" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/RollingNumbers.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::rolling::rolling_number;
use kubaos_ui::layout::row;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Rolling numbers",
    lede: Some(
        "A count that rolls only the figures that change, each in a slot a whole device pixel wide, in a box as wide as its widest value.",
    ),
    view,
};

/// The widest a counter's value is shown: three figures.
const WIDEST: &str = "000";

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    row(look, Space::S32)
        .center()
        .extend([0, 42, 999, 1000].map(|value| rolling_number(value).widest(WIDEST).view(look)))
        .push(
            row(look, Space::S8)
                .center()
                .push(rolling_number(128).widest(WIDEST).view(look))
                .push(text("glasses", Type::Body, c.on_surface_variant)),
        )
        .into()
}
