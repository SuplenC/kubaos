//! Chips: a short fact or a pick, at the control radius, which on a chip's 24 is a pill.

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Theme;
use iced::widget;
use iced::widget::container;
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
}

impl ChipKind {
    pub const ALL: [ChipKind; 4] = [ChipKind::Neutral, ChipKind::Selected, ChipKind::Accent, ChipKind::Cold];

    /// The kind's name in the spec.
    pub fn name(self) -> &'static str {
        match self {
            ChipKind::Neutral => "Neutral",
            ChipKind::Selected => "Selected",
            ChipKind::Accent => "Accent",
            ChipKind::Cold => "Cold",
        }
    }
}

/// A chip reading `label`, with an optional leading icon. Pressing it sends `on_press` when there
/// is one (a pick made the default), and its close button sends `on_remove` when there is one.
pub fn chip<'a, M: Clone + 'a>(
    look: &Look,
    kind: ChipKind,
    label: impl IntoFragment<'a>,
    leading: Option<Icon>,
    on_press: Option<M>,
    on_remove: Option<M>,
) -> Element<'a, M> {
    let with_icon = if leading.is_some() { "icon" } else { "" };
    let removable = if on_remove.is_some() { "removable" } else { "" };
    let variant = [kind.name(), with_icon, removable];
    let c = look.colors;
    let (fill, content) = match kind {
        ChipKind::Neutral => (c.surface_container_high, c.on_surface),
        ChipKind::Selected => (c.secondary_container, c.on_secondary_container),
        ChipKind::Accent => (color::alpha(c.primary, Opacity::Tint.value()), c.primary),
        ChipKind::Cold => (color::alpha(c.cold, Opacity::Tint.value()), c.cold),
    };
    // The label starts where the chip's curve ends; an icon sits closer in. The close button is a
    // circle as tall as the chip, so its hover layer shares the chip's curve at the end.
    let left = if leading.is_some() { Space::S8 } else { Space::S12 };
    let right = if on_remove.is_some() {
        Gap::NONE
    } else {
        Space::S12.into()
    };
    let mut body = row(look, Space::S4).center();
    if let Some(glyph) = leading {
        body = body.push(icon(look, glyph, Type::Label.icon(), content));
    }
    body = body.push(text(label, Type::Label, content));
    if let Some(on_remove) = on_remove {
        let close = widget::button(icon(look, Icon::Close, Type::Label.icon(), content))
            .padding(Inset::all(Space::S4).padding())
            .on_press(on_remove)
            .style(move |_theme: &Theme, _status| widget::button::Style {
                background: None,
                text_color: content,
                border: Border::default().rounded(radius(Radius::Full)),
                shadow: Shadow::default(),
                snap: true,
            });
        body = body.push(layered(close, Radius::Full, content, true, None));
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
    let Some(on_press) = on_press else {
        return probe::component("Chip", variant, Fixed::HEIGHT, face);
    };
    let pressable = widget::button(face)
        .padding(0)
        .on_press(on_press)
        .style(|_theme: &Theme, _status| widget::button::Style {
            background: None,
            ..widget::button::Style::default()
        });
    let view = layered(pressable, Radius::Control, content, true, None);
    probe::component("Chip", variant, Fixed::HEIGHT, view)
}
