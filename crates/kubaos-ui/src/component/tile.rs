//! Tiles: a quick toggle with a symbol, a title and a status line, as the control center's Wi-Fi
//! and Bluetooth are. On, it fills with the accent. A tile with more behind it ends in a detail
//! arrow, which opens it.
//!
//! # Owns
//!
//! - [`tile`], built up with [`Tile`].
//!
//! # Not here
//!
//! - A row of a list, which is picked rather than toggled: [`crate::component::list`].

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Theme;
use iced::alignment;
use iced::widget;

use crate::color;
use crate::component::layered;
use crate::focus::RingColor;
use crate::focus::focus_ring;
use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::Extent;
use crate::layout::Inset;
use crate::layout::row;
use crate::layout::stack;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::snap::centre;
use crate::text::line;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// A tile is clicked, so it takes the control's corners.
const CORNER: Radius = Radius::Control;

/// A tile reading `title`, built up then drawn with [`Tile::view`].
#[derive(Debug)]
pub struct Tile<M> {
    glyph: Icon,
    title: String,
    subtitle: String,
    checked: bool,
    on_press: Option<M>,
    on_detail: Option<M>,
    focused: bool,
}

/// A tile showing `glyph` and `title`, off. It is as tall as a two-line row, its two lines one
/// line each, ending in an ellipsis where they don't fit.
pub fn tile<M>(glyph: Icon, title: impl Into<String>) -> Tile<M> {
    Tile {
        glyph,
        title: title.into(),
        subtitle: String::new(),
        checked: false,
        on_press: None,
        on_detail: None,
        focused: false,
    }
}

impl<M: Clone> Tile<M> {
    /// The status line under the title: the network, "Off", "2 connected".
    #[must_use]
    pub fn subtitle(self, subtitle: impl Into<String>) -> Self {
        Tile {
            subtitle: subtitle.into(),
            ..self
        }
    }

    /// Whether it is on.
    #[must_use]
    pub fn checked(self, checked: bool) -> Self {
        Tile { checked, ..self }
    }

    /// What a press on the tile toggles; a tile without one is disabled.
    #[must_use]
    pub fn on_press(self, message: M) -> Self {
        Tile {
            on_press: Some(message),
            ..self
        }
    }

    /// The detail arrow at the tile's end, and what it opens: the tile's list or settings.
    #[must_use]
    pub fn on_detail(self, message: M) -> Self {
        Tile {
            on_detail: Some(message),
            ..self
        }
    }

    /// Whether the keyboard is on it: the focus tube lights its edge.
    #[must_use]
    pub fn focused(self, focused: bool) -> Self {
        Tile { focused, ..self }
    }

    pub fn view<'a>(self, look: &Look) -> Element<'a, M>
    where
        M: 'a,
    {
        let c = look.colors;
        let enabled = self.on_press.is_some();
        let (fill, content, muted, glyph_colour) = if !enabled {
            let dim = color::alpha(c.on_surface, Opacity::DisabledContent.value());
            (
                color::alpha(c.on_surface, Opacity::DisabledContainer.value()),
                dim,
                dim,
                dim,
            )
        } else if self.checked {
            let muted = color::alpha(c.on_primary, Opacity::SubduedText.value());
            (c.primary, c.on_primary, muted, c.on_primary)
        } else {
            (
                c.surface_container_high,
                c.on_surface,
                c.on_surface_variant,
                c.on_surface_variant,
            )
        };
        let words = stack(look, Space::S4)
            .width(Extent::Fill)
            .push(line(self.title, Type::BodyStrong, content))
            .push(line(self.subtitle, Type::Small, muted));
        let parts = row(look, Space::S12)
            .center()
            .width(Extent::Fill)
            .height(Extent::Fill)
            .push(icon(look, self.glyph, Type::BodyStrong.icon(), glyph_colour))
            .push(words);
        let detail = self.on_detail.is_some();
        // The words end before the arrow and its inset, or at the tile's own inset.
        let inset = Inset::x(Space::S12).padding();
        let end = if detail {
            inset.right * 2.0 + f32::from(Size::ControlSm.px())
        } else {
            inset.right
        };
        let pressable = widget::button(parts)
            .width(Length::Fill)
            .height(f32::from(Size::RowTwoLine.px()))
            .padding(iced::Padding { right: end, ..inset })
            .on_press_maybe(self.on_press)
            .style(move |_theme: &Theme, _status| widget::button::Style {
                background: Some(Background::Color(fill)),
                text_color: content,
                border: Border::default().rounded(radius(CORNER)),
                shadow: Shadow::default(),
                snap: true,
            });
        let mut tile: Element<'a, M> = layered(pressable, CORNER, content, enabled, None);
        if let Some(message) = self.on_detail {
            // Over the tile, not in it: the stack hides the pointer from the tile while it is on
            // the arrow, so only the arrow's own state layer shows there.
            let over = widget::container(arrow(look, message, glyph_colour, enabled))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Right)
                .align_y(alignment::Vertical::Center)
                .padding(Inset::NONE.right(Space::S12).padding());
            tile = widget::Stack::with_children([tile, over.into()]).into();
        }
        let view: Element<'a, M> = focus_ring(look, tile, CORNER, self.focused, RingColor::Accent);
        probe::component(
            "Tile",
            [if detail { "detail" } else { "" }, "", ""],
            Fixed::HEIGHT,
            view,
        )
    }
}

/// The detail arrow: a small control that stays quiet, a circle with no fill, in the tile's own
/// content colour, which only its state layer shows under the pointer.
fn arrow<'a, M: Clone + 'a>(look: &Look, message: M, colour: Color, enabled: bool) -> Element<'a, M> {
    let side = f32::from(Size::ControlSm.px());
    let pressable = widget::button(centre(
        look.scale,
        icon(look, Icon::ChevronRight, Type::ButtonSmall.icon(), colour),
    ))
    .width(side)
    .height(side)
    .padding(0)
    .on_press_maybe(enabled.then_some(message))
    .style(move |_theme: &Theme, _status| widget::button::Style {
        background: None,
        text_color: colour,
        border: Border::default().rounded(radius(Radius::Full)),
        shadow: Shadow::default(),
        snap: true,
    });
    layered(pressable, Radius::Full, colour, enabled, None)
}

#[cfg(test)]
mod tests;
