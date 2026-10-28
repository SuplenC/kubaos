//! The components page's "Skeletons" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Skeletons.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::skeleton::Bone;
use kubaos_ui::component::skeleton::skeleton;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::row;
use kubaos_ui::layout::stack;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Radius;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Skeletons",
    lede: Some(
        "The shape of what is loading, in the placeholder colour, so what arrives moves nothing: a row being scanned, a thumbnail being decoded.",
    ),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let scanning = row(look, Space::S12)
        .center()
        .push(skeleton(look, Bone::Circle(IconSize::S24)))
        .push(
            stack(look, Space::S4)
                .width(Extent::Fill)
                .push(skeleton(look, Bone::Line(Type::BodyStrong)))
                .push(skeleton(look, Bone::Line(Type::Small))),
        );
    grid(look, 2, Space::S24)
        .push(scanning)
        .push(skeleton(look, Bone::Block(Size::ControlLg, Radius::Control)))
        .into()
}
