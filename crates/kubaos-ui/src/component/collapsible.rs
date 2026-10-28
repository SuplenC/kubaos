//! Collapsibles: a titled part that opens to show more, and closes to keep out of the way.
//!
//! The header is a small control as tall as a row of a list: its title, and a chevron at its end
//! that points down while it is closed and up while it is open. Open, the content sits 8 under
//! it.
//!
//! # Owns
//!
//! - [`collapsible`].
//!
//! # Not here
//!
//! - The motion as it opens: the shell's (`Collapsible` in `KubaOS.Components`); here it is drawn
//!   open or closed.

use iced::Border;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Theme;
use iced::widget;

use crate::component::layered;
use crate::focus::RingColor;
use crate::focus::focus_ring;
use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::Extent;
use crate::layout::Gap;
use crate::layout::Inset;
use crate::layout::row;
use crate::layout::stack;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::text::line;
use crate::tokens::Radius;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// `content` under a header reading `title`, shown while `open`, in line with the title. A press on
/// the header sends `on_toggle`; while it has the keyboard (`focused`) the focus tube lights the
/// header.
pub fn collapsible<'a, M: Clone + 'a>(
    look: &Look,
    title: impl Into<String>,
    open: bool,
    on_toggle: M,
    focused: bool,
    content: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    let c = look.colors;
    let chevron = if open { Icon::ExpandLess } else { Icon::ExpandMore };
    let parts = row(look, Space::S12)
        .center()
        .width(Extent::Fill)
        .height(Extent::Fill)
        .push(
            stack(look, Gap::NONE)
                .width(Extent::Fill)
                .push(line(title, Type::BodyStrong, c.on_surface)),
        )
        .push(icon(look, chevron, Type::BodyStrong.icon(), c.on_surface_variant));
    let header = widget::button(parts)
        .width(Length::Fill)
        .height(f32::from(Size::Row.px()))
        .padding(Inset::x(Space::S12).padding())
        .on_press(on_toggle)
        .style(move |_theme: &Theme, _status| widget::button::Style {
            background: None,
            text_color: c.on_surface,
            border: Border::default().rounded(radius(Radius::Control)),
            shadow: Shadow::default(),
            snap: true,
        });
    let header = layered(header, Radius::Control, c.on_surface, true, None);
    let header = focus_ring(look, header, Radius::Control, focused, RingColor::Accent);
    let mut view = stack(look, Space::S8).width(Extent::Fill).push(header);
    if open {
        // In line with the title: the header's own inset at the sides, once, so the content
        // brings none of its own.
        view = view.push(
            widget::container(content)
                .width(Length::Fill)
                .padding(Inset::x(Space::S12).padding()),
        );
    }
    probe::component(
        "Collapsible",
        [if open { "open" } else { "" }, "", ""],
        Fixed::HOLDS,
        view,
    )
}
