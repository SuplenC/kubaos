//! The one-line text field, and its label and note.

use std::fmt;

use iced::Background;
use iced::Border;
use iced::Element;
use iced::Length;
use iced::Theme;
use iced::widget;
use iced::widget::text_input;

use crate::color;
use crate::focus::RingColor;
use crate::focus::focus_ring;
use crate::icon::Icon;
use crate::layout::Gap;
use crate::layout::Inset;
use crate::layout::fill;
use crate::layout::row;
use crate::layout::stack;
use crate::look::Look;
use crate::look::radius;
use crate::text;
use crate::tokens::Family;
use crate::tokens::IconSize;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;
use crate::tokens::Weight;

/// How a field shows its value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Editable.
    Normal,
    /// The value is wrong: the error colour, with the reason in the note under it.
    Invalid,
    /// Read-only: dimmed, never focused.
    Locked,
}

/// A text field, built up then drawn with [`Field::view`].
pub struct Field<'a, M> {
    value: &'a str,
    placeholder: &'a str,
    icon: Option<Icon>,
    secret: bool,
    state: State,
    focused: bool,
    id: Option<widget::Id>,
    on_input: Option<Box<dyn Fn(String) -> M + 'a>>,
}

impl<M> fmt::Debug for Field<'_, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Field")
            .field("value", if self.secret { &"<secret>" } else { &self.value })
            .field("placeholder", &self.placeholder)
            .field("icon", &self.icon)
            .field("state", &self.state)
            .field("focused", &self.focused)
            .finish_non_exhaustive()
    }
}

/// A field showing `value`, or the dimmed `placeholder` while it is empty: a hint, never a value
/// that looks typed.
pub fn field<'a, M>(value: &'a str, placeholder: &'a str) -> Field<'a, M> {
    Field {
        value,
        placeholder,
        icon: None,
        secret: false,
        state: State::Normal,
        focused: false,
        id: None,
        on_input: None,
    }
}

impl<'a, M: Clone + 'a> Field<'a, M> {
    /// `icon` inside the field, before the text.
    #[must_use]
    pub fn icon(self, icon: Icon) -> Self {
        Field {
            icon: Some(icon),
            ..self
        }
    }

    /// Bullets instead of the value.
    #[must_use]
    pub fn secret(self) -> Self {
        Field { secret: true, ..self }
    }

    #[must_use]
    pub fn state(self, state: State) -> Self {
        Field { state, ..self }
    }

    #[must_use]
    pub fn focused(self, focused: bool) -> Self {
        Field { focused, ..self }
    }

    /// The id the app focuses the field's caret by.
    #[must_use]
    pub fn id(self, id: impl Into<widget::Id>) -> Self {
        Field {
            id: Some(id.into()),
            ..self
        }
    }

    #[must_use]
    pub fn on_input(self, on_input: impl Fn(String) -> M + 'a) -> Self {
        Field {
            on_input: Some(Box::new(on_input)),
            ..self
        }
    }

    pub fn view(self, look: &Look) -> Element<'a, M> {
        let c = look.colors;
        let hairline = look.hairline();
        let focused = self.focused && self.state != State::Locked;
        let state = self.state;
        let style = move |_theme: &Theme, status: text_input::Status| {
            let hovered = matches!(status, text_input::Status::Hovered);
            let (edge, fill) = match state {
                State::Locked => (
                    color::alpha(c.outline_variant, Opacity::Locked.value()),
                    color::alpha(c.surface_container_high, Opacity::Locked.value()),
                ),
                State::Invalid => (c.error, c.surface_container_high),
                State::Normal if focused => (c.primary, c.surface_container_highest),
                State::Normal if hovered => (c.outline, c.surface_container_highest),
                State::Normal => (c.outline_variant, c.surface_container_high),
            };
            let content = if state == State::Locked {
                c.on_surface_variant
            } else {
                c.on_surface
            };
            text_input::Style {
                background: Background::Color(fill),
                border: Border {
                    color: edge,
                    width: hairline,
                    radius: radius(Radius::R12).into(),
                },
                icon: c.on_surface_variant,
                placeholder: color::alpha(c.on_surface_variant, Opacity::Placeholder.value()),
                value: content,
                selection: color::alpha(c.primary, Opacity::Selection.value()),
            }
        };
        // A 24 px line and 8 above and below: as tall as a regular button.
        let style_type = Type::BodyLarge;
        let mut input = widget::text_input(self.placeholder, self.value)
            .font(text::font(style_type))
            .size(f32::from(style_type.size()))
            .line_height(text::line_height(style_type))
            .padding(Inset::xy(Space::S12, Space::S8).padding())
            .width(Length::Fill)
            .secure(self.secret)
            .style(style);
        if let Some(glyph) = self.icon {
            input = input.icon(text_input::Icon {
                font: text::family_font(Family::Icons, Weight::Regular),
                code_point: glyph.glyph(),
                size: Some(f32::from(IconSize::S20.px()).into()),
                spacing: f32::from(Space::S8.px()),
                side: text_input::Side::Left,
            });
        }
        if let Some(id) = self.id {
            input = input.id(id);
        }
        if self.state != State::Locked
            && let Some(on_input) = self.on_input
        {
            input = input.on_input(on_input);
        }
        focus_ring(look, input, Radius::R12, focused, RingColor::Accent)
    }
}

/// `control` under its label, with an optional small action at the label's right (a Show toggle,
/// a small button) and an optional note under it. The label's row is as tall as a small button
/// whether or not there is one, so fields side by side line up.
pub fn labelled<'a, M: 'a>(
    look: &Look,
    label: &'a str,
    action: Option<Element<'a, M>>,
    control: impl Into<Element<'a, M>>,
    note: Option<Element<'a, M>>,
) -> Element<'a, M> {
    let c = look.colors;
    let mut head = row(look, Gap::NONE)
        .center()
        .height(crate::layout::Extent::Fixed(Size::ControlSm))
        .push(text::text(label, Type::Label, c.on_surface_variant))
        .push(fill());
    if let Some(action) = action {
        head = head.push(action);
    }
    let mut column = stack(look, Space::S4).push(head).push(control);
    if let Some(note) = note {
        column = column.push(note);
    }
    column.into()
}

/// A note under a field: `message` in the error colour when `invalid`, otherwise muted.
pub fn note<'a, M: 'a>(look: &Look, message: &'a str, invalid: bool) -> Element<'a, M> {
    let c = look.colors;
    text::text(
        message,
        Type::Small,
        if invalid { c.error } else { c.on_surface_variant },
    )
    .into()
}
