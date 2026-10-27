//! List rows, and the well a list sits in.
//!
//! The well is a surface (24) with 12 inside, so the rows inside take 24 - 12 = 12, the control
//! radius: concentric with the well, and the radius of everything clicked.

use iced::Alignment;
use iced::Background;
use iced::Border;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Theme;
use iced::widget;
use iced::widget::container;

use crate::color;
use crate::component::state_layer;
use crate::focus::RingColor;
use crate::focus::focus_ring;
use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::Inset;
use crate::layout::fill;
use crate::layout::row;
use crate::layout::stack;
use crate::look::Corner;
use crate::look::Look;
use crate::look::nested;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::snap::centre_x;
use crate::text::text;
use crate::tokens::IconSize;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// The well's corners: it holds content.
const WELL_RADIUS: Radius = Radius::Surface;
/// The well's padding.
const WELL_INSET: Space = Space::S12;
/// The gap between rows.
const ROW_GAP: Space = Space::S4;
/// The rows' corners, concentric inside the well; a well and inset that can't be stop the build.
const ROW_RADIUS: Corner = nested(WELL_RADIUS.corner(), WELL_INSET);

/// One row of a list, built up then drawn with [`ListRow::view`].
#[derive(Debug)]
pub struct ListRow<'a, M> {
    title: &'a str,
    subtitle: Option<&'a str>,
    detail: Option<&'a str>,
    icon: Option<Icon>,
    selected: bool,
    danger: bool,
    checked: Option<bool>,
    on_press: Option<M>,
}

/// A row reading `title`.
pub fn list_row<M>(title: &str) -> ListRow<'_, M> {
    ListRow {
        title,
        subtitle: None,
        detail: None,
        icon: None,
        selected: false,
        danger: false,
        checked: None,
        on_press: None,
    }
}

impl<'a, M: Clone + 'a> ListRow<'a, M> {
    /// A second line under the title: the row is two lines tall.
    #[must_use]
    pub fn subtitle(self, subtitle: &'a str) -> Self {
        ListRow {
            subtitle: Some(subtitle),
            ..self
        }
    }

    /// A short fact at the row's end, in mono: a device path, a size, a code.
    #[must_use]
    pub fn detail(self, detail: &'a str) -> Self {
        ListRow {
            detail: Some(detail),
            ..self
        }
    }

    #[must_use]
    pub fn icon(self, icon: Icon) -> Self {
        ListRow {
            icon: Some(icon),
            ..self
        }
    }

    /// The row the keyboard is on, or the one picked in a list of one.
    #[must_use]
    pub fn selected(self, selected: bool) -> Self {
        ListRow { selected, ..self }
    }

    /// A row whose action can't be taken back (Shut down): selected, it takes the error colour
    /// instead of the secondary container.
    #[must_use]
    pub fn danger(self) -> Self {
        ListRow { danger: true, ..self }
    }

    /// A check mark at the end when picked; the room for it is kept when not, so the rows' ends
    /// line up.
    #[must_use]
    pub fn checked(self, checked: bool) -> Self {
        ListRow {
            checked: Some(checked),
            ..self
        }
    }

    #[must_use]
    pub fn on_press(self, message: M) -> Self {
        ListRow {
            on_press: Some(message),
            ..self
        }
    }

    pub fn view(self, look: &Look) -> Element<'a, M> {
        let lines = if self.subtitle.is_some() {
            "two-line"
        } else {
            "one-line"
        };
        let variant = [lines, if self.icon.is_some() { "icon" } else { "" }, ""];
        let c = look.colors;
        let (fill_colour, main, muted) = if self.selected && self.danger {
            (
                Some(c.error),
                c.on_error,
                color::alpha(c.on_error, Opacity::Subdued.value()),
            )
        } else if self.selected {
            (
                Some(c.secondary_container),
                c.on_secondary_container,
                color::alpha(c.on_secondary_container, Opacity::Subdued.value()),
            )
        } else {
            (None, c.on_surface, c.on_surface_variant)
        };
        let height = if self.subtitle.is_some() {
            Size::RowTwoLine
        } else {
            Size::Row
        };
        let mut lines = stack(look, Space::S4).push(text(self.title, Type::BodyStrong, main));
        if let Some(subtitle) = self.subtitle {
            lines = lines.push(text(subtitle, Type::Small, muted));
        }
        let mut content = row(look, Space::S12).center();
        if let Some(glyph) = self.icon {
            content = content.push(icon(look, glyph, IconSize::S20, main));
        }
        content = content.push(lines).push(fill());
        if let Some(detail) = self.detail {
            content = content.push(text(detail, Type::Mono, muted));
        }
        if let Some(checked) = self.checked {
            content = content.push(if checked {
                Element::from(icon(look, Icon::Check, IconSize::S20, main))
            } else {
                widget::Space::new()
                    .width(f32::from(IconSize::S20.px()))
                    .height(f32::from(IconSize::S20.px()))
                    .into()
            });
        }
        let corner = ROW_RADIUS;
        let on_surface = c.on_surface;
        let view: Element<'a, M> = widget::button(
            widget::container(content)
                .height(Length::Fill)
                .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .height(f32::from(height.px()))
        .padding(Inset::x(Space::S12).padding())
        .on_press_maybe(self.on_press)
        .style(move |_theme: &Theme, status| widget::button::Style {
            background: state_layer(fill_colour, on_surface, status),
            text_color: main,
            border: Border::default().rounded(radius(corner)),
            shadow: Shadow::default(),
            snap: true,
        })
        .into();
        probe::component("ListRow", variant, Fixed::HEIGHT, view)
    }
}

/// `rows` in a well, the focus ring around it while the list has the keyboard. The rows show
/// which one the keyboard is on themselves.
pub fn list<'a, M: 'a>(look: &Look, rows: impl IntoIterator<Item = Element<'a, M>>, focused: bool) -> Element<'a, M> {
    let c = look.colors;
    let surface = color::alpha(c.surface_container_low, Opacity::Well.value());
    let edge = look.outline_soft();
    let hairline = look.hairline();
    let well = widget::container(stack(look, ROW_GAP).extend(rows))
        .padding(Inset::all(WELL_INSET).padding())
        .width(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(surface)),
            border: Border {
                color: edge,
                width: hairline,
                radius: radius(WELL_RADIUS).into(),
            },
            ..container::Style::default()
        });
    let view: Element<'a, M> = focus_ring(look, well, WELL_RADIUS, focused, RingColor::Accent);
    probe::component("List", ["", "", ""], Fixed::NONE, view)
}

/// The well with nothing in it: `message`, centred.
pub fn empty_list<'a, M: 'a>(look: &Look, message: &'a str) -> Element<'a, M> {
    let c = look.colors;
    let view: Element<'a, M> = list(
        look,
        [
            widget::container(centre_x(look.scale, text(message, Type::Body, c.on_surface_variant)))
                .padding(Inset::y(Space::S12).padding())
                .into(),
        ],
        false,
    );
    probe::component("EmptyList", ["", "", ""], Fixed::NONE, view)
}

#[cfg(test)]
mod tests;
