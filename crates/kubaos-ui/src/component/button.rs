//! Buttons: four kinds in three sizes, with a label or with an icon alone.

use iced::Border;
use iced::Color;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Theme;
use iced::widget;

use crate::color;
use crate::component::hint::hint;
use crate::component::state_layer;
use crate::focus::RingColor;
use crate::focus::focus_ring;
use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::Inset;
use crate::layout::row;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::snap::centre;
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

    /// The kind's name in the spec.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Filled => "Filled",
            Kind::Tonal => "Tonal",
            Kind::Text => "Text",
            Kind::Danger => "Danger",
        }
    }
}

/// How big a button is. A label's button and an icon's are the same height at each size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    /// 32 tall: the chrome around a screen, beside a field's label.
    Sm,
    /// 40 tall, as tall as a field: a screen's own actions.
    Md,
    /// 48 tall: the one main action of a big page, Kōbō's Install.
    Lg,
}

impl ButtonSize {
    pub const ALL: [ButtonSize; 3] = [ButtonSize::Sm, ButtonSize::Md, ButtonSize::Lg];

    /// The size's name in the spec.
    pub fn name(self) -> &'static str {
        match self {
            ButtonSize::Sm => "Sm",
            ButtonSize::Md => "Md",
            ButtonSize::Lg => "Lg",
        }
    }
}

/// A size's measures, all from the scale.
struct Metrics {
    height: Size,
    label: Type,
    icon: IconSize,
    gap: Space,
    /// Inside the left and right of a filled, tonal or danger button.
    padding: Space,
    /// Inside a text button, which has no edge to keep the label from.
    text_padding: Space,
}

fn metrics(size: ButtonSize) -> Metrics {
    match size {
        ButtonSize::Sm => Metrics {
            height: Size::ControlSm,
            label: Type::ButtonSmall,
            icon: IconSize::S16,
            gap: Space::S4,
            padding: Space::S12,
            text_padding: Space::S12,
        },
        ButtonSize::Md => Metrics {
            height: Size::ControlMd,
            label: Type::Button,
            icon: IconSize::S20,
            gap: Space::S8,
            padding: Space::S24,
            text_padding: Space::S12,
        },
        ButtonSize::Lg => Metrics {
            height: Size::ControlLg,
            label: Type::ButtonLarge,
            icon: IconSize::S24,
            gap: Space::S8,
            padding: Space::S32,
            text_padding: Space::S16,
        },
    }
}

/// A state shown as if the pointer were on the button, whatever the pointer does: the specimen's
/// and screenshot tests' view of hover and press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preview {
    Hovered,
    Pressed,
}

/// A kind's fill and content colours, dimmed when disabled.
fn colours(look: &Look, kind: Kind, enabled: bool) -> (Option<Color>, Color, Color) {
    let c = look.colors;
    let (fill, content) = match kind {
        Kind::Filled => (Some(c.primary), c.on_primary),
        Kind::Tonal => (Some(c.secondary_container), c.on_secondary_container),
        Kind::Text => (None, c.primary),
        Kind::Danger => (Some(c.error), c.on_error),
    };
    if enabled {
        (fill, content, content)
    } else {
        let dim = color::alpha(c.on_surface, Opacity::DisabledContent.value());
        let fill = fill.map(|_| color::alpha(c.on_surface, Opacity::DisabledContainer.value()));
        (fill, content, dim)
    }
}

/// The pill's style, with the state layer for the status iced reports or the one `preview` pins.
fn pill(
    fill: Option<Color>,
    content: Color,
    shown: Color,
    enabled: bool,
    preview: Option<Preview>,
) -> impl Fn(&Theme, widget::button::Status) -> widget::button::Style {
    move |_theme, status| {
        let status = match preview {
            Some(Preview::Hovered) if enabled => widget::button::Status::Hovered,
            Some(Preview::Pressed) if enabled => widget::button::Status::Pressed,
            _ => status,
        };
        widget::button::Style {
            background: state_layer(fill, content, status),
            text_color: shown,
            border: Border::default().rounded(radius(Radius::Full)),
            shadow: Shadow::default(),
            snap: true,
        }
    }
}

/// `glyph` at `size`, centred on a whole device pixel in a box `line` tall, so it shares the
/// label's line box and the row needs no centring of its own.
fn icon_in_line<'a, M: 'a>(look: &Look, glyph: Icon, size: IconSize, line: u16, colour: Color) -> Element<'a, M> {
    widget::container(centre(look.scale, icon(look, glyph, size, colour)))
        .width(f32::from(size.px()))
        .height(f32::from(line))
        .into()
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
    preview: Option<Preview>,
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
        preview: None,
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

    /// Shown hovered or pressed whatever the pointer does.
    #[must_use]
    pub fn preview(self, preview: Preview) -> Self {
        Button {
            preview: Some(preview),
            ..self
        }
    }

    pub fn view(self, look: &Look) -> Element<'a, M> {
        let placement = match (self.icon, self.trailing) {
            (None, _) => "",
            (Some(_), false) => "leading-icon",
            (Some(_), true) => "trailing-icon",
        };
        let variant = [self.kind.name(), self.size.name(), placement];
        let m = metrics(self.size);
        let enabled = self.on_press.is_some();
        let (fill, content, shown) = colours(look, self.kind, enabled);
        let padding = if self.kind == Kind::Text {
            m.text_padding
        } else {
            m.padding
        };
        let line = m.label.line();
        // The label's line box fills the control's content height, so the label centres on the grid.
        let pad_y = (m.height.px() - line) / 2;
        let label = text(self.label, m.label, shown);
        let mut content_row = row(look, m.gap);
        content_row = match (self.icon, self.trailing) {
            (None, _) => content_row.push(label),
            (Some(glyph), false) => content_row
                .push(icon_in_line(look, glyph, m.icon, line, shown))
                .push(label),
            (Some(glyph), true) => content_row
                .push(label)
                .push(icon_in_line(look, glyph, m.icon, line, shown)),
        };
        let pressable = widget::button(content_row)
            .height(Length::Fixed(f32::from(m.height.px())))
            .padding(iced::Padding {
                top: f32::from(pad_y),
                bottom: f32::from(pad_y),
                ..Inset::x(padding).padding()
            })
            .on_press_maybe(self.on_press)
            .style(pill(fill, content, shown, enabled, self.preview));
        let view: Element<'a, M> = focus_ring(look, pressable, Radius::Full, self.focused, RingColor::Accent);
        probe::component("Button", variant, Fixed::HEIGHT, view)
    }
}

/// An icon-only button, built up then drawn with [`IconButton::view`].
#[derive(Debug)]
pub struct IconButton<'a, M> {
    glyph: Icon,
    label: &'a str,
    kind: Kind,
    size: ButtonSize,
    on_press: Option<M>,
    focused: bool,
    preview: Option<Preview>,
}

/// A circular [`Kind::Filled`], [`ButtonSize::Md`] button showing `glyph`. `label` is its name,
/// shown in a hint while it is hovered or focused, since the icon says nothing out loud.
pub fn icon_button<M>(glyph: Icon, label: &str) -> IconButton<'_, M> {
    IconButton {
        glyph,
        label,
        kind: Kind::Filled,
        size: ButtonSize::Md,
        on_press: None,
        focused: false,
        preview: None,
    }
}

impl<'a, M: Clone + 'a> IconButton<'a, M> {
    #[must_use]
    pub fn kind(self, kind: Kind) -> Self {
        IconButton { kind, ..self }
    }

    #[must_use]
    pub fn size(self, size: ButtonSize) -> Self {
        IconButton { size, ..self }
    }

    #[must_use]
    pub fn on_press(self, message: M) -> Self {
        IconButton {
            on_press: Some(message),
            ..self
        }
    }

    /// `message` when there is one; disabled when `None`.
    #[must_use]
    pub fn on_press_maybe(self, message: Option<M>) -> Self {
        IconButton {
            on_press: message,
            ..self
        }
    }

    /// Whether the keyboard is on it; its hint shows while it is.
    #[must_use]
    pub fn focused(self, focused: bool) -> Self {
        IconButton { focused, ..self }
    }

    /// Shown hovered or pressed whatever the pointer does.
    #[must_use]
    pub fn preview(self, preview: Preview) -> Self {
        IconButton {
            preview: Some(preview),
            ..self
        }
    }

    pub fn view(self, look: &Look) -> Element<'a, M> {
        let variant = [self.kind.name(), self.size.name(), ""];
        let m = metrics(self.size);
        let enabled = self.on_press.is_some();
        let (fill, content, shown) = colours(look, self.kind, enabled);
        let side = f32::from(m.height.px());
        let pressable = widget::button(centre(look.scale, icon(look, self.glyph, m.icon, shown)))
            .width(side)
            .height(side)
            .padding(0)
            .on_press_maybe(self.on_press)
            .style(pill(fill, content, shown, enabled, self.preview));
        let ringed = focus_ring(look, pressable, Radius::Full, self.focused, RingColor::Accent);
        let view: Element<'a, M> = hint(look, ringed, self.label, self.focused);
        probe::component("IconButton", variant, Fixed::BOTH, view)
    }
}

#[cfg(test)]
mod tests;
