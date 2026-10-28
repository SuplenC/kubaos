//! The components page's "Sheets" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Sheets.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use iced::Length;
use iced::border;
use iced::widget;
use iced::widget::container;
use kubaos_ui::Look;
use kubaos_ui::component::sheet;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::stack;
use kubaos_ui::look::radius;
use kubaos_ui::snap::centre_x;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Opacity;
use kubaos_ui::tokens::Radius;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use crate::specimen::Message;
use super::Section;

pub const SECTION: Section = Section {
    title: "Sheets",
    lede: Some("A panel growing out of the frame's edge, one surface with it: concave fillets where it meets the edge, the surface radius at its far corners."),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let frame = sheet::surface(look);
    let corner = radius(Radius::Surface);
    // The frame's top edge, which the sheet hangs from.
    let edge = widget::container(widget::Space::new())
        .width(Length::Fill)
        .height(f32::from(Space::S8.px()))
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(frame)),
            border: iced::Border {
                radius: border::Radius::default().top(corner),
                ..iced::Border::default()
            },
            ..container::Style::default()
        });
    let body = stack(look, Space::S4)
        .padding(Inset::all(Space::S16))
        .push(text("Control center", Type::Subheading, c.on_surface))
        .push(text(
            "It grows out of the frame's edge.",
            Type::Small,
            c.on_surface_variant,
        ));
    let backdrop = kubaos_ui::color::alpha(c.surface_container_highest, Opacity::Well.value());
    let hanging = widget::container(centre_x(look.scale, sheet::sheet(look, body)))
        .padding(Inset::NONE.top(Space::S8).bottom(Space::S32).padding());
    // The hanging sheet first: a stack is as large as its first layer.
    widget::container(widget::Stack::with_children([hanging.into(), edge.into()]))
        .width(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(backdrop)),
            border: iced::Border::default().rounded(corner),
            ..container::Style::default()
        })
        .into()
}
