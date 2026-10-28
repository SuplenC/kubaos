//! Menus: a few actions or choices on a popover, as a context menu or a select's list.
//!
//! A popover is the floating surface: the surface radius, a soft hairline and the panel's
//! shadow, 12 inside, so the items in it take 24 − 12 = 12, the control radius. A menu is its
//! items one under the other, each a small control tall, with separators between groups.
//!
//! # Owns
//!
//! - [`popover`], [`menu`], [`menu_item`] (built up with [`MenuItem`]) and [`menu_separator`].
//!
//! # Not here
//!
//! - Floating the popover over the view at its anchor: the app's. The shell's popovers float over
//!   the window (`Popover` in `KubaOS.Components`); here one is drawn where it is put.
//! - The control a select's menu drops from: [`crate::component::select`].

use iced::Background;
use iced::Border;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Theme;
use iced::Vector;
use iced::widget;
use iced::widget::container;

use crate::color;
use crate::component::keycap::keycap;
use crate::component::layered;
use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::Extent;
use crate::layout::Gap;
use crate::layout::Inset;
use crate::layout::divider;
use crate::layout::row;
use crate::layout::stack;
use crate::look::Corner;
use crate::look::Look;
use crate::look::nested;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::text::line;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Shadow as ShadowToken;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// The popover's corners: it holds content.
const SURFACE: Radius = Radius::Surface;
/// The popover's padding.
const INSET: Space = Space::S12;
/// An item's corners, concentric inside the popover; a pair that can't be stops the build.
const ITEM_RADIUS: Corner = nested(SURFACE.corner(), INSET);

/// `content` on a popover `width` wide: the surface radius, a soft hairline, the panel's shadow,
/// 12 inside.
pub fn popover<'a, M: 'a>(look: &Look, width: Extent, content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    let c = look.colors;
    let fill = c.surface_container;
    let edge = look.outline_soft();
    let hairline = look.hairline();
    let shade = color::alpha(c.shadow, ShadowToken::Panel.opacity());
    let view: Element<'a, M> = widget::container(content)
        .padding(Inset::all(INSET).padding())
        .width(width.length())
        .style(move |_theme| container::Style {
            background: Some(Background::Color(fill)),
            border: Border {
                color: edge,
                width: hairline,
                radius: radius(SURFACE).into(),
            },
            shadow: Shadow {
                color: shade,
                offset: Vector::new(0.0, f32::from(ShadowToken::Panel.y())),
                blur_radius: f32::from(ShadowToken::Panel.blur()),
            },
            ..container::Style::default()
        })
        .into();
    probe::component("Popover", ["", "", ""], Fixed::HOLDS, view)
}

/// `entries` one under the other: [`menu_item`]s and [`menu_separator`]s.
pub fn menu<'a, M: 'a>(look: &Look, entries: impl IntoIterator<Item = Element<'a, M>>) -> Element<'a, M> {
    let view = stack(look, Gap::NONE).width(Extent::Fill).extend(entries);
    probe::component("Menu", ["", "", ""], Fixed::NONE, view)
}

/// A hairline between two groups of items, 4 clear of each.
pub fn menu_separator<'a, M: 'a>(look: &Look) -> Element<'a, M> {
    stack(look, Gap::NONE)
        .width(Extent::Fill)
        .padding(Inset::y(Space::S4))
        .push(divider(look))
        .into()
}

/// One item of a menu reading `label`, built up then drawn with [`MenuItem::view`].
#[derive(Debug)]
pub struct MenuItem<'a, M> {
    label: String,
    icon: Option<Icon>,
    shortcut: &'a [&'a str],
    checked: Option<bool>,
    danger: bool,
    highlighted: bool,
    on_press: Option<M>,
}

/// An item reading `label` on one line, ending in an ellipsis where it doesn't fit.
pub fn menu_item<'a, M>(label: impl Into<String>) -> MenuItem<'a, M> {
    MenuItem {
        label: label.into(),
        icon: None,
        shortcut: &[],
        checked: None,
        danger: false,
        highlighted: false,
        on_press: None,
    }
}

impl<'a, M: Clone + 'a> MenuItem<'a, M> {
    /// A symbol before the label.
    #[must_use]
    pub fn icon(self, icon: Icon) -> Self {
        MenuItem {
            icon: Some(icon),
            ..self
        }
    }

    /// The keys that do the same, one keycap each, at the item's end.
    #[must_use]
    pub fn shortcut(self, keys: &'a [&'a str]) -> Self {
        MenuItem { shortcut: keys, ..self }
    }

    /// A check at the end when picked; the room for it is kept when not, so the items' ends line
    /// up.
    #[must_use]
    pub fn checked(self, checked: bool) -> Self {
        MenuItem {
            checked: Some(checked),
            ..self
        }
    }

    /// An item that destroys something (Kill, Delete): its words and symbol in the error colour.
    #[must_use]
    pub fn danger(self) -> Self {
        MenuItem { danger: true, ..self }
    }

    /// The item the keyboard is on.
    #[must_use]
    pub fn highlighted(self, highlighted: bool) -> Self {
        MenuItem { highlighted, ..self }
    }

    /// What a press sends; an item without one is disabled.
    #[must_use]
    pub fn on_press(self, message: M) -> Self {
        MenuItem {
            on_press: Some(message),
            ..self
        }
    }

    pub fn view(self, look: &Look) -> Element<'a, M> {
        let c = look.colors;
        let enabled = self.on_press.is_some();
        let (fill, ink) = if !enabled {
            (None, color::alpha(c.on_surface, Opacity::DisabledContent.value()))
        } else if self.highlighted {
            (Some(c.secondary_container), c.on_secondary_container)
        } else if self.danger {
            (None, c.error)
        } else {
            (None, c.on_surface)
        };
        let glyph_size = Type::Body.icon();
        let mut parts = row(look, Space::S12).center().width(Extent::Fill).height(Extent::Fill);
        if let Some(glyph) = self.icon {
            parts = parts.push(icon(look, glyph, glyph_size, ink));
        }
        parts = parts.push(
            stack(look, Gap::NONE)
                .width(Extent::Fill)
                .push(line(self.label, Type::Body, ink)),
        );
        if !self.shortcut.is_empty() {
            parts = parts.push(
                row(look, Space::S4)
                    .center()
                    .extend(self.shortcut.iter().map(|key| keycap(look, key))),
            );
        }
        if let Some(checked) = self.checked {
            parts = parts.push(if checked {
                icon(look, Icon::Check, glyph_size, ink)
            } else {
                widget::Space::new()
                    .width(f32::from(glyph_size.px()))
                    .height(f32::from(glyph_size.px()))
                    .into()
            });
        }
        let pressable = widget::button(parts)
            .width(Length::Fill)
            .height(f32::from(Size::ControlSm.px()))
            .padding(Inset::x(Space::S12).padding())
            .on_press_maybe(self.on_press)
            .style(move |_theme: &Theme, _status| widget::button::Style {
                background: fill.map(Background::Color),
                text_color: ink,
                border: Border::default().rounded(radius(ITEM_RADIUS)),
                shadow: Shadow::default(),
                snap: true,
            });
        let view = layered(pressable, ITEM_RADIUS, ink, enabled, None);
        let words = [
            if self.icon.is_some() { "icon" } else { "" },
            if self.checked.is_some() { "check" } else { "" },
            if self.danger { "danger" } else { "" },
        ];
        probe::component("MenuItem", words, Fixed::HEIGHT, view)
    }
}
