//! Dialogs: a question over a dimmed screen, and the confirmation before a destructive action.

use iced::Background;
use iced::Border;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Vector;
use iced::widget;
use iced::widget::container;

use crate::color;
use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::Gap;
use crate::layout::Inset;
use crate::layout::fill;
use crate::layout::gap;
use crate::layout::row;
use crate::layout::stack;
use crate::look::Look;
use crate::look::radius;
use crate::snap::centre;
use crate::text::text;
use crate::tokens::IconSize;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Shadow as ShadowToken;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// Whether the dialog's action can't be undone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// The accent: a question with no harm in either answer.
    Normal,
    /// The error colour and a warning sign: the next press destroys something.
    Danger,
}

/// A dialog: `glyph` and `title` on the first line, `body` under them, then the buttons. A
/// dangerous one shows the warning sign instead of `glyph`. `dismiss` is the button on the left
/// that backs out (Cancel); `actions` sit on the right, the one Enter presses last.
pub fn dialog<'a, M: 'a>(
    look: &Look,
    tone: Tone,
    glyph: Icon,
    title: &'a str,
    body: &'a str,
    dismiss: Option<Element<'a, M>>,
    actions: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    let c = look.colors;
    let (tint, glyph, edge) = match tone {
        Tone::Normal => (c.primary, glyph, color::alpha(c.primary, Opacity::DialogEdge.value())),
        Tone::Danger => (
            c.error,
            Icon::Warning,
            color::alpha(c.error, Opacity::DangerEdge.value()),
        ),
    };
    let mut buttons = row(look, Space::S8).center();
    if let Some(dismiss) = dismiss {
        buttons = buttons.push(dismiss);
    }
    buttons = buttons.push(fill()).extend(actions);
    let content = stack(look, Gap::NONE)
        .push(
            row(look, Space::S12)
                .center()
                .push(icon(glyph, IconSize::S24, tint))
                .push(text(title, Type::Headline, c.on_surface)),
        )
        .push(gap(Space::S12))
        .push(text(body, Type::Body, c.on_surface_variant))
        .push(gap(Space::S32))
        .push(buttons);
    let surface = c.surface_container_high;
    let shadow = c.shadow;
    let hairline = look.hairline();
    widget::container(content)
        .width(Length::Fixed(f32::from(Size::Dialog.px())))
        .padding(Inset::all(Space::S32).padding())
        .style(move |_theme| container::Style {
            background: Some(Background::Color(surface)),
            border: Border {
                color: edge,
                width: hairline,
                radius: radius(Radius::R24).into(),
            },
            shadow: Shadow {
                color: color::alpha(shadow, ShadowToken::Dialog.opacity()),
                offset: Vector::new(0.0, f32::from(ShadowToken::Dialog.y())),
                blur_radius: f32::from(ShadowToken::Dialog.blur()),
            },
            ..container::Style::default()
        })
        .into()
}

/// `dialog` centred over the screen, everything behind it dimmed.
pub fn scrim<'a, M: 'a>(look: &Look, dialog: impl Into<Element<'a, M>>) -> Element<'a, M> {
    let dim = color::alpha(look.colors.scrim, Opacity::Scrim.value());
    widget::container(centre(look.scale, dialog))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style::default().background(dim))
        .into()
}
