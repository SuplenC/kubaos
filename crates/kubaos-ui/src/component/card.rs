//! Cards: a surface inside the page, plain or one of a few to pick from.

use iced::Background;
use iced::Border;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Theme;
use iced::widget;
use iced::widget::container;

use crate::color;
use crate::focus::RingColor;
use crate::focus::focus_ring;
use crate::layout::Extent;
use crate::layout::Inset;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Space;

const CARD_RADIUS: Radius = Radius::Surface;

/// A plain card around `content`: the well's surface and soft hairline, 16 inside.
pub fn card<'a, M: 'a>(look: &Look, content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    card_sized(look, content, Extent::Shrink)
}

/// [`card`], `height` tall: `Fill` stretches it to its row in a grid, so cards side by side end
/// together.
pub fn card_sized<'a, M: 'a>(look: &Look, content: impl Into<Element<'a, M>>, height: Extent) -> Element<'a, M> {
    let c = look.colors;
    let surface = color::alpha(c.surface_container_low, Opacity::Well.value());
    let edge = look.outline_soft();
    let hairline = look.hairline();
    let view: Element<'a, M> = widget::container(content)
        .padding(Inset::all(Space::S16).padding())
        .width(Length::Fill)
        .height(height.length())
        .style(move |_theme| container::Style {
            background: Some(Background::Color(surface)),
            border: Border {
                color: edge,
                width: hairline,
                radius: radius(CARD_RADIUS).into(),
            },
            ..container::Style::default()
        })
        .into();
    probe::component("Card", ["", "", ""], Fixed::HOLDS, view)
}

/// One of a few cards to pick from. The picked one takes the secondary container and an accent
/// hairline; focus is the cold ring, since the accent already means picked here. Without `on_pick`
/// it is disabled: the card is the disabled container, with no hover and no ring, and the caller
/// builds `content` at the disabled opacity (`look.faded(Opacity::DisabledContent.value())`), since
/// iced can't dim what it is given.
pub fn choice_card<'a, M: Clone + 'a>(
    look: &Look,
    content: impl Into<Element<'a, M>>,
    picked: bool,
    on_pick: Option<M>,
    focused: bool,
) -> Element<'a, M> {
    let c = look.colors;
    let hairline = look.hairline();
    let enabled = on_pick.is_some();
    let pressable = widget::button(content)
        .width(Length::Fill)
        .padding(Inset::all(Space::S16).padding())
        .on_press_maybe(on_pick)
        .style(move |_theme: &Theme, status| {
            let fill = if !enabled {
                color::alpha(c.on_surface, Opacity::DisabledContainer.value())
            } else if picked {
                c.secondary_container
            } else if enabled && matches!(status, widget::button::Status::Hovered) {
                color::mix(c.surface_container_low, c.on_surface, Opacity::Hover.value())
            } else {
                color::alpha(c.surface_container_low, Opacity::Well.value())
            };
            widget::button::Style {
                background: Some(Background::Color(fill)),
                text_color: c.on_surface,
                border: Border {
                    color: if picked && enabled {
                        c.primary
                    } else {
                        c.outline_variant
                    },
                    width: hairline,
                    radius: radius(CARD_RADIUS).into(),
                },
                shadow: Shadow::default(),
                snap: true,
            }
        });
    let view: Element<'a, M> = focus_ring(look, pressable, CARD_RADIUS, focused && enabled, RingColor::Cold);
    probe::component("ChoiceCard", ["", "", ""], Fixed::HOLDS, view)
}
