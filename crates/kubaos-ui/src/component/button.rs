//! Buttons: four kinds, two sizes.

use iced::Border;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Theme;
use iced::widget;

use crate::color;
use crate::component::state_layer;
use crate::focus::RingColor;
use crate::focus::focus_ring;
use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::Inset;
use crate::layout::row;
use crate::look::Look;
use crate::look::radius;
use crate::text::text;
use crate::tokens::IconSize;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// What a button is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The accent pill: the one thing to do next on a screen.
    Filled,
    /// The secondary container: an alternative to the filled one.
    Tonal,
    /// Text alone: Back, Skip, Cancel, Show log.
    Text,
    /// The error colour: the action that destroys something, after a confirmation.
    Danger,
}

impl Kind {
    pub const ALL: [Kind; 4] = [Kind::Filled, Kind::Tonal, Kind::Text, Kind::Danger];
}

/// How big a button is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    /// 32 tall: the chrome around a screen, beside a field's label.
    Sm,
    /// 40 tall, as tall as a field: a screen's own actions.
    Md,
}

/// A button, built up then drawn with [`Button::view`].
#[derive(Debug)]
pub struct Button<'a, M> {
    label: &'a str,
    kind: Kind,
    size: ButtonSize,
    icon: Option<Icon>,
    trailing: bool,
    on_press: Option<M>,
    focused: bool,
}

/// A [`Kind::Filled`], [`ButtonSize::Md`] button reading `label`, disabled until it gets an action.
pub fn button<M>(label: &str) -> Button<'_, M> {
    Button {
        label,
        kind: Kind::Filled,
        size: ButtonSize::Md,
        icon: None,
        trailing: false,
        on_press: None,
        focused: false,
    }
}

impl<'a, M: Clone + 'a> Button<'a, M> {
    #[must_use]
    pub fn kind(self, kind: Kind) -> Self {
        Button { kind, ..self }
    }

    #[must_use]
    pub fn size(self, size: ButtonSize) -> Self {
        Button { size, ..self }
    }

    /// `icon` before the label.
    #[must_use]
    pub fn icon(self, icon: Icon) -> Self {
        Button {
            icon: Some(icon),
            trailing: false,
            ..self
        }
    }

    /// `icon` after the label: a button that goes forward.
    #[must_use]
    pub fn trailing_icon(self, icon: Icon) -> Self {
        Button {
            icon: Some(icon),
            trailing: true,
            ..self
        }
    }

    #[must_use]
    pub fn on_press(self, message: M) -> Self {
        Button {
            on_press: Some(message),
            ..self
        }
    }

    /// `message` when there is one; disabled when `None`.
    #[must_use]
    pub fn on_press_maybe(self, message: Option<M>) -> Self {
        Button {
            on_press: message,
            ..self
        }
    }

    /// Whether the keyboard is on it.
    #[must_use]
    pub fn focused(self, focused: bool) -> Self {
        Button { focused, ..self }
    }

    pub fn view(self, look: &Look) -> Element<'a, M> {
        let c = look.colors;
        let enabled = self.on_press.is_some();
        let (fill, content) = match self.kind {
            Kind::Filled => (Some(c.primary), c.on_primary),
            Kind::Tonal => (Some(c.secondary_container), c.on_secondary_container),
            Kind::Text => (None, c.primary),
            Kind::Danger => (Some(c.error), c.on_error),
        };
        let shown = if enabled {
            content
        } else {
            color::alpha(c.on_surface, Opacity::DisabledContent.value())
        };
        let fill = if enabled {
            fill
        } else {
            fill.map(|_| color::alpha(c.on_surface, Opacity::DisabledContainer.value()))
        };
        let (label_style, icon_size, gap, height) = match self.size {
            ButtonSize::Md => (Type::Button, IconSize::S20, Space::S8, Size::ControlMd),
            ButtonSize::Sm => (Type::ButtonSmall, IconSize::S16, Space::S4, Size::ControlSm),
        };
        // The label's line box fills the control's content height, so the label centres on the grid.
        let pad_x = match (self.kind, self.size) {
            (Kind::Text, ButtonSize::Md) | (_, ButtonSize::Sm) => Space::S12,
            (_, ButtonSize::Md) => Space::S24,
        };
        let pad_y = (height.px() - label_style.line()) / 2;
        let label = text(self.label, label_style, shown);
        let mut content_row = row(look, gap).center();
        content_row = match (self.icon, self.trailing) {
            (None, _) => content_row.push(label),
            (Some(glyph), false) => content_row.push(icon(glyph, icon_size, shown)).push(label),
            (Some(glyph), true) => content_row.push(label).push(icon(glyph, icon_size, shown)),
        };
        let pressable = widget::button(content_row)
            .height(Length::Fixed(f32::from(height.px())))
            .padding(iced::Padding {
                top: f32::from(pad_y),
                bottom: f32::from(pad_y),
                ..Inset::x(pad_x).padding()
            })
            .on_press_maybe(self.on_press)
            .style(move |_theme: &Theme, status| widget::button::Style {
                background: state_layer(fill, content, status),
                text_color: shown,
                border: Border::default().rounded(radius(Radius::Full)),
                shadow: Shadow::default(),
                snap: true,
            });
        focus_ring(look, pressable, Radius::Full, self.focused, RingColor::Accent)
    }
}

#[cfg(test)]
mod tests;
