//! Dialogs: a question over a dimmed screen, the confirmation before a destructive action, and a
//! prompt that asks for something to be typed.

use iced::Background;
use iced::Border;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Vector;
use iced::widget;
use iced::widget::container;
use iced::widget::text::IntoFragment;

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
use crate::probe;
use crate::probe::Fixed;
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

impl Tone {
    /// The tone's name in the spec.
    pub fn name(self) -> &'static str {
        match self {
            Tone::Normal => "Normal",
            Tone::Danger => "Danger",
        }
    }
}

/// A dialog: `glyph` and `title` on the first line, `body` under them, then the buttons. A
/// dangerous one shows the warning sign instead of `glyph`. `dismiss` is the button on the left
/// that backs out (Cancel); `actions` sit on the right, the one Enter presses last.
pub fn dialog<'a, M: 'a>(
    look: &Look,
    tone: Tone,
    glyph: Icon,
    title: impl IntoFragment<'a>,
    body: impl IntoFragment<'a>,
    dismiss: Option<Element<'a, M>>,
    actions: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    compose(look, tone, glyph, title, body, None, dismiss, actions)
}

/// A dialog that asks for something to be typed: `dialog` in the `Normal` tone with `control` (a
/// labelled field) 16 under the body and 32 above the buttons. A prompt never destroys anything;
/// its action uses what was typed. The app owns the field's keys, so it sends the action's message
/// on Enter in the field too: Enter does what the action does.
pub fn prompt<'a, M: 'a>(
    look: &Look,
    glyph: Icon,
    title: impl IntoFragment<'a>,
    body: impl IntoFragment<'a>,
    control: impl Into<Element<'a, M>>,
    dismiss: Option<Element<'a, M>>,
    actions: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    compose(
        look,
        Tone::Normal,
        glyph,
        title,
        body,
        Some(control.into()),
        dismiss,
        actions,
    )
}

/// The dialog `dialog` and `prompt` draw: the head, the body, what it asks for if anything, then
/// the buttons.
#[expect(
    clippy::too_many_arguments,
    reason = "a dialog's parts, each an argument as `dialog` takes them, plus what it asks for"
)]
fn compose<'a, M: 'a>(
    look: &Look,
    tone: Tone,
    glyph: Icon,
    title: impl IntoFragment<'a>,
    body: impl IntoFragment<'a>,
    control: Option<Element<'a, M>>,
    dismiss: Option<Element<'a, M>>,
    actions: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    let variant = [tone.name(), "", ""];
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
    let mut content = stack(look, Gap::NONE)
        .push(
            row(look, Space::S12)
                .center()
                .push(icon(look, glyph, IconSize::S24, tint))
                .push(text(title, Type::Headline, c.on_surface)),
        )
        .push(gap(Space::S12))
        .push(text(body, Type::Body, c.on_surface_variant));
    if let Some(control) = control {
        content = content.push(gap(Space::S16)).push(control);
    }
    let content = content.push(gap(Space::S32)).push(buttons);
    let surface = c.surface_container_high;
    let shadow = c.shadow;
    let hairline = look.hairline();
    let view: Element<'a, M> = widget::container(content)
        .width(Length::Fixed(f32::from(Size::Dialog.px())))
        .padding(Inset::all(Space::S32).padding())
        .style(move |_theme| container::Style {
            background: Some(Background::Color(surface)),
            border: Border {
                color: edge,
                width: hairline,
                radius: radius(Radius::Surface).into(),
            },
            shadow: Shadow {
                color: color::alpha(shadow, ShadowToken::Dialog.opacity()),
                offset: Vector::new(0.0, f32::from(ShadowToken::Dialog.y())),
                blur_radius: f32::from(ShadowToken::Dialog.blur()),
            },
            ..container::Style::default()
        })
        .into();
    probe::component(
        "Dialog",
        variant,
        Fixed {
            width: true,
            height: false,
            holds: false,
        },
        view,
    )
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
