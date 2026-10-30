//! Chips: a short fact or a pick, at the control radius, which on a chip's 24 is a pill.
//!
//! # Owns
//!
//! - [`chip`], built up with [`Chip`], and its [`ChipKind`].
//!
//! # Not here
//!
//! - A chip in a list row: [`crate::component::list`].

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Theme;
use iced::widget;
use iced::widget::container;
use iced::widget::text::Fragment;
use iced::widget::text::IntoFragment;

use crate::color;
use crate::component::layered;
use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::Gap;
use crate::layout::Inset;
use crate::layout::row;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::text::text;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// What a chip says about its content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipKind {
    /// A plain fact.
    Neutral,
    /// One of several things picked; the first of them, the default, shows a check.
    Selected,
    /// A tag that matters: Recommended, Default.
    Accent,
    /// The cold accent, rarely: a demo, a preview.
    Cold,
    /// The accent's own fill: the one thing still to do, an action such as Reboot among states.
    Filled,
}

impl ChipKind {
    pub const ALL: [ChipKind; 5] = [
        ChipKind::Neutral,
        ChipKind::Selected,
        ChipKind::Accent,
        ChipKind::Cold,
        ChipKind::Filled,
    ];

    /// The kind's name in the spec.
    pub fn name(self) -> &'static str {
        match self {
            ChipKind::Neutral => "Neutral",
            ChipKind::Selected => "Selected",
            ChipKind::Accent => "Accent",
            ChipKind::Cold => "Cold",
            ChipKind::Filled => "Filled",
        }
    }
}

/// A chip reading `label`, built up then drawn with [`Chip::view`].
#[derive(Debug)]
pub struct Chip<'a, M> {
    kind: ChipKind,
    label: Fragment<'a>,
    leading: Option<Icon>,
    on_press: Option<M>,
    on_remove: Option<M>,
    disabled: bool,
}

/// A `kind` chip reading `label`: a plain fact until it is given a press or a close button.
#[must_use]
pub fn chip<'a, M>(kind: ChipKind, label: impl IntoFragment<'a>) -> Chip<'a, M> {
    Chip {
        kind,
        label: label.into_fragment(),
        leading: None,
        on_press: None,
        on_remove: None,
        disabled: false,
    }
}

impl<'a, M: Clone + 'a> Chip<'a, M> {
    /// A symbol before the label.
    #[must_use]
    pub fn icon(self, glyph: Icon) -> Self {
        self.icon_maybe(Some(glyph))
    }

    /// [`Chip::icon`] when there is a symbol; the label alone otherwise.
    #[must_use]
    pub fn icon_maybe(self, glyph: Option<Icon>) -> Self {
        Chip { leading: glyph, ..self }
    }

    /// What a press on the chip sends: a pick made the default, a setting turned on.
    #[must_use]
    pub fn on_press(self, message: M) -> Self {
        self.on_press_maybe(Some(message))
    }

    /// [`Chip::on_press`] when there is a message; a plain chip otherwise.
    #[must_use]
    pub fn on_press_maybe(self, message: Option<M>) -> Self {
        Chip {
            on_press: message,
            ..self
        }
    }

    /// A close button at its end, sending `message`.
    #[must_use]
    pub fn on_remove(self, message: M) -> Self {
        self.on_remove_maybe(Some(message))
    }

    /// [`Chip::on_remove`] when there is a message; no close button otherwise.
    #[must_use]
    pub fn on_remove_maybe(self, message: Option<M>) -> Self {
        Chip {
            on_remove: message,
            ..self
        }
    }

    /// Whether it can't be used now: the content at the disabled opacity, the chip at its own,
    /// and neither its press nor its close button taking the pointer.
    #[must_use]
    pub fn disabled(self, disabled: bool) -> Self {
        Chip { disabled, ..self }
    }

    pub fn view(self, look: &Look) -> Element<'a, M> {
        let with_icon = if self.leading.is_some() { "icon" } else { "" };
        let removable = if self.on_remove.is_some() { "removable" } else { "" };
        let variant = [self.kind.name(), with_icon, removable];
        let c = look.colors;
        let (fill, content) = if self.disabled {
            (
                color::alpha(c.on_surface, Opacity::DisabledContainer.value()),
                color::alpha(c.on_surface, Opacity::DisabledContent.value()),
            )
        } else {
            match self.kind {
                ChipKind::Neutral => (c.surface_container_high, c.on_surface),
                ChipKind::Selected => (c.secondary_container, c.on_secondary_container),
                ChipKind::Accent => (color::alpha(c.primary, Opacity::Tint.value()), c.primary),
                ChipKind::Cold => (color::alpha(c.cold, Opacity::Tint.value()), c.cold),
                ChipKind::Filled => (c.primary, c.on_primary),
            }
        };
        let enabled = !self.disabled;
        // The label starts where the chip's curve ends; an icon sits closer in. The close button is
        // a circle as tall as the chip, so its hover layer shares the chip's curve at the end.
        let left = if self.leading.is_some() { Space::S8 } else { Space::S12 };
        let right = if self.on_remove.is_some() {
            Gap::NONE
        } else {
            Space::S12.into()
        };
        let mut body = row(look, Space::S4).center();
        if let Some(glyph) = self.leading {
            body = body.push(icon(look, glyph, Type::Label.icon(), content));
        }
        body = body.push(text(self.label, Type::Label, content));
        if let Some(on_remove) = self.on_remove {
            let close = widget::button(icon(look, Icon::Close, Type::Label.icon(), content))
                .padding(Inset::all(Space::S4).padding())
                .on_press_maybe(enabled.then_some(on_remove))
                .style(move |_theme: &Theme, _status| widget::button::Style {
                    background: None,
                    text_color: content,
                    border: Border::default().rounded(radius(Radius::Full)),
                    shadow: Shadow::default(),
                    snap: true,
                });
            body = body.push(layered(close, Radius::Full, content, enabled, None));
        }
        let face = widget::container(row(look, Gap::NONE).center().push(body))
            .height(Length::Fixed(f32::from(Size::Chip.px())))
            .center_y(Length::Fixed(f32::from(Size::Chip.px())))
            .padding(Inset::NONE.left(left).right(right).padding())
            .style(move |_theme| container::Style {
                background: Some(Background::Color(fill)),
                border: Border::default().rounded(radius(Radius::Control)),
                text_color: Some(Color { a: 1.0, ..content }),
                ..container::Style::default()
            });
        let Some(on_press) = self.on_press else {
            return probe::component("Chip", variant, Fixed::HEIGHT, face);
        };
        let pressable = widget::button(face)
            .padding(0)
            .on_press_maybe(enabled.then_some(on_press))
            .style(|_theme: &Theme, _status| widget::button::Style {
                background: None,
                ..widget::button::Style::default()
            });
        let view = layered(pressable, Radius::Control, content, enabled, None);
        probe::component("Chip", variant, Fixed::HEIGHT, view)
    }
}

#[cfg(test)]
mod tests;
