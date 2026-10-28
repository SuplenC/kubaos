//! Selects: one of a list too long for radio rows or tabs, picked from a menu that drops from a
//! field-like control.
//!
//! # Owns
//!
//! - [`select`], the control, built up with [`Select`].
//!
//! # Not here
//!
//! - The menu it opens: [`crate::component::menu`], which the app shows on a popover under it.

use iced::Background;
use iced::Border;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Theme;
use iced::widget;

use crate::color;
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
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// A select showing `value`, built up then drawn with [`Select::view`].
#[derive(Debug)]
pub struct Select<M> {
    value: String,
    icon: Option<Icon>,
    open: bool,
    focused: bool,
    on_press: Option<M>,
}

/// A select showing `value`: as tall as a field, the control radius, a hairline, the value in
/// `body_large` and a chevron at its end.
pub fn select<M>(value: impl Into<String>) -> Select<M> {
    Select {
        value: value.into(),
        icon: None,
        open: false,
        focused: false,
        on_press: None,
    }
}

impl<M: Clone> Select<M> {
    /// A symbol before the value.
    #[must_use]
    pub fn icon(self, icon: Icon) -> Self {
        Select {
            icon: Some(icon),
            ..self
        }
    }

    /// Whether its menu is open: the hairline takes the accent and the chevron turns up.
    #[must_use]
    pub fn open(self, open: bool) -> Self {
        Select { open, ..self }
    }

    /// Whether the keyboard is on it: the focus tube lights its edge.
    #[must_use]
    pub fn focused(self, focused: bool) -> Self {
        Select { focused, ..self }
    }

    /// What a press sends: the app opens the menu. A select without one is disabled.
    #[must_use]
    pub fn on_press(self, message: M) -> Self {
        Select {
            on_press: Some(message),
            ..self
        }
    }

    pub fn view<'a>(self, look: &Look) -> Element<'a, M>
    where
        M: 'a,
    {
        let c = look.colors;
        let enabled = self.on_press.is_some();
        let ink = if enabled {
            c.on_surface
        } else {
            color::alpha(c.on_surface, Opacity::DisabledContent.value())
        };
        let muted = if enabled { c.on_surface_variant } else { ink };
        let edge = if self.open { c.primary } else { c.outline_variant };
        let fill = if self.open {
            c.surface_container_highest
        } else {
            c.surface_container_high
        };
        let glyph_size = Type::BodyLarge.icon();
        let mut parts = row(look, Space::S8).center().width(Extent::Fill).height(Extent::Fill);
        if let Some(glyph) = self.icon {
            parts = parts.push(icon(look, glyph, glyph_size, muted));
        }
        let chevron = if self.open { Icon::ExpandLess } else { Icon::ExpandMore };
        parts = parts
            .push(
                stack(look, Gap::NONE)
                    .width(Extent::Fill)
                    .push(line(self.value, Type::BodyLarge, ink)),
            )
            .push(icon(look, chevron, glyph_size, muted));
        let hairline = look.hairline();
        let pressable = widget::button(parts)
            .width(Length::Fill)
            .height(f32::from(Size::ControlMd.px()))
            .padding(Inset::x(Space::S12).padding())
            .on_press_maybe(self.on_press)
            .style(move |_theme: &Theme, _status| widget::button::Style {
                background: Some(Background::Color(fill)),
                text_color: ink,
                border: Border {
                    color: edge,
                    width: hairline,
                    radius: radius(Radius::Control).into(),
                },
                shadow: Shadow::default(),
                snap: true,
            });
        let layered = layered(pressable, Radius::Control, c.on_surface, enabled, None);
        let view: Element<'a, M> = focus_ring(look, layered, Radius::Control, self.focused, RingColor::Accent);
        let words = [if self.icon.is_some() { "icon" } else { "" }, "", ""];
        probe::component("Select", words, Fixed::HEIGHT, view)
    }
}
