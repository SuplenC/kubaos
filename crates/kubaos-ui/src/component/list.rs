//! List rows, and the well a list sits in.
//!
//! The well is a surface (24) with 12 inside, so the rows inside take 24 - 12 = 12, the control
//! radius: concentric with the well, and the radius of everything clicked. A list taller than its
//! room scrolls inside the well, its thumb in the well's padding at the right.

use std::fmt;

use iced::Background;
use iced::Border;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Theme;
use iced::widget;
use iced::widget::container;

use crate::color;
use crate::component::avatar::TileSize;
use crate::component::avatar::icon_tile;
use crate::component::chip::ChipKind;
use crate::component::chip::chip;
use crate::component::layered;
use crate::focus::RingColor;
use crate::focus::focus_ring;
use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::Extent;
use crate::layout::Inset;
use crate::layout::row;
use crate::layout::stack;
use crate::look::Corner;
use crate::look::Look;
use crate::look::nested;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::snap::centre_x;
use crate::text::line;
use crate::text::text;
use crate::tokens;
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

/// How a row's icon is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Face {
    /// The symbol alone.
    Bare,
    /// On an icon tile.
    Tile,
    /// On an icon tile, in the accent.
    AccentTile,
}

/// One row of a list, built up then drawn with [`ListRow::view`].
#[derive(Debug)]
pub struct ListRow<'a, M> {
    title: String,
    subtitle: Option<String>,
    detail: Option<String>,
    icon: Option<Icon>,
    face: Face,
    chips: Vec<(ChipKind, String)>,
    selected: bool,
    danger: bool,
    checked: Option<bool>,
    on_press: Option<M>,
    _text: std::marker::PhantomData<&'a str>,
}

/// A row reading `title`. Its lines stay one line each, ending in an ellipsis where they don't fit,
/// so every row of a kind is as tall as the others.
pub fn list_row<'a, M>(title: impl Into<String>) -> ListRow<'a, M> {
    ListRow {
        title: title.into(),
        subtitle: None,
        detail: None,
        icon: None,
        face: Face::Bare,
        chips: Vec::new(),
        selected: false,
        danger: false,
        checked: None,
        on_press: None,
        _text: std::marker::PhantomData,
    }
}

impl<'a, M: Clone + 'a> ListRow<'a, M> {
    /// A second line under the title: the row is two lines tall.
    #[must_use]
    pub fn subtitle(self, subtitle: impl Into<String>) -> Self {
        ListRow {
            subtitle: Some(subtitle.into()),
            ..self
        }
    }

    /// A short fact at the row's end, in mono: a device path, a size, a code.
    #[must_use]
    pub fn detail(self, detail: impl Into<String>) -> Self {
        ListRow {
            detail: Some(detail.into()),
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

    /// The icon on an icon tile, `Sm` beside one line and `Md` beside two, as the Command menu's
    /// rows show theirs: a list of things of one kind, where the row's symbol is what tells one
    /// from the others at a glance (the default keyboard layout's star).
    #[must_use]
    pub fn tile(self) -> Self {
        let face = match self.face {
            Face::Bare => Face::Tile,
            kept => kept,
        };
        ListRow { face, ..self }
    }

    /// The icon on a tile, its symbol in the accent: the one row of the list that comes first (the
    /// default).
    #[must_use]
    pub fn accent(self) -> Self {
        ListRow {
            face: Face::AccentTile,
            ..self
        }
    }

    /// A chip near the row's end, after the chips before it: a fact about the row that matters at
    /// a glance (Default, Too small, Done).
    #[must_use]
    pub fn chip(mut self, kind: ChipKind, label: impl Into<String>) -> Self {
        self.chips.push((kind, label.into()));
        self
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

    /// `message` when there is one; a row without one takes no press.
    #[must_use]
    pub fn on_press_maybe(self, message: Option<M>) -> Self {
        ListRow {
            on_press: message,
            ..self
        }
    }

    pub fn view(self, look: &Look) -> Element<'a, M> {
        let lines = if self.subtitle.is_some() {
            "two-line"
        } else {
            "one-line"
        };
        let symbol = match (self.icon, self.face) {
            (None, _) => "",
            (Some(_), Face::Bare) => "icon",
            (Some(_), Face::Tile | Face::AccentTile) => "tile",
        };
        let variant = [lines, symbol, ""];
        let c = look.colors;
        let (fill_colour, main, muted) = if self.selected && self.danger {
            (
                Some(c.error),
                c.on_error,
                color::alpha(c.on_error, Opacity::SubduedText.value()),
            )
        } else if self.selected {
            (
                Some(c.secondary_container),
                c.on_secondary_container,
                color::alpha(c.on_secondary_container, Opacity::SubduedText.value()),
            )
        } else {
            (None, c.on_surface, c.on_surface_variant)
        };
        let two_line = self.subtitle.is_some();
        let height = if two_line { Size::RowTwoLine } else { Size::Row };
        let mut words = stack(look, Space::S4)
            .width(Extent::Fill)
            .push(line(self.title, Type::BodyStrong, main));
        if let Some(subtitle) = self.subtitle {
            words = words.push(line(subtitle, Type::Small, muted));
        }
        // Each part centred on the row's whole height, on a whole device pixel, so a chip that
        // makes one row's parts taller moves none of the others: the title sits where every row's
        // does.
        let mut content = row(look, Space::S12).center().width(Extent::Fill).height(Extent::Fill);
        if let Some(glyph) = self.icon
            && self.face != Face::Bare
        {
            let size = if two_line { TileSize::Md } else { TileSize::Sm };
            let tile = icon_tile(glyph, size);
            let tile = if self.face == Face::AccentTile {
                tile.accent()
            } else {
                tile
            };
            content = content.push(tile.view(look));
        } else if let Some(glyph) = self.icon {
            // Beside a title over a subtitle, the icon spans both lines.
            let size = if two_line {
                tokens::TWO_LINE_ICON
            } else {
                Type::BodyStrong.icon()
            };
            content = content.push(icon(look, glyph, size, main));
        }
        content = content.push(words);
        for (kind, label) in self.chips {
            content = content.push(chip(look, kind, label, None, None, None));
        }
        if let Some(detail) = self.detail {
            content = content.push(text(detail, Type::Mono, muted));
        }
        if let Some(checked) = self.checked {
            content = content.push(if checked {
                Element::from(icon(look, Icon::Check, Type::BodyStrong.icon(), main))
            } else {
                widget::Space::new()
                    .width(f32::from(Type::BodyStrong.icon().px()))
                    .height(f32::from(Type::BodyStrong.icon().px()))
                    .into()
            });
        }
        let enabled = self.on_press.is_some();
        let pressable = widget::button(content)
            .width(Length::Fill)
            .height(f32::from(height.px()))
            .padding(Inset::x(Space::S12).padding())
            .on_press_maybe(self.on_press)
            .style(move |_theme: &Theme, _status| widget::button::Style {
                background: fill_colour.map(Background::Color),
                text_color: main,
                border: Border::default().rounded(radius(ROW_RADIUS)),
                shadow: Shadow::default(),
                snap: true,
            });
        let view = layered(pressable, ROW_RADIUS, c.on_surface, enabled, None);
        probe::component("ListRow", variant, Fixed::HEIGHT, view)
    }
}

/// `rows` in a well, its edge lit as the focus tube while the list has the keyboard. The rows show
/// which one the keyboard is on themselves.
pub fn list<'a, M: 'a>(look: &Look, rows: impl IntoIterator<Item = Element<'a, M>>, focused: bool) -> Element<'a, M> {
    well(
        look,
        stack(look, ROW_GAP).extend(rows),
        Inset::all(WELL_INSET),
        Extent::Shrink,
        focused,
        false,
    )
}

/// A list whose rows scroll inside its well, built up then drawn with [`ScrollingList::view`].
pub struct ScrollingList<'a, M> {
    rows: Vec<Element<'a, M>>,
    focused: bool,
    height: Extent,
    id: Option<widget::Id>,
    on_scroll: Option<Box<dyn Fn(widget::scrollable::Viewport) -> M + 'a>>,
}

impl<M> fmt::Debug for ScrollingList<'_, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScrollingList")
            .field("rows", &self.rows.len())
            .field("focused", &self.focused)
            .field("height", &self.height)
            .finish_non_exhaustive()
    }
}

/// `rows` in a well as [`list`] has them, scrolled inside it when they are taller than its room.
pub fn scrolling_list<'a, M: 'a>(
    rows: impl IntoIterator<Item = Element<'a, M>>,
    focused: bool,
) -> ScrollingList<'a, M> {
    ScrollingList {
        rows: rows.into_iter().collect(),
        focused,
        height: Extent::Fill,
        id: None,
        on_scroll: None,
    }
}

impl<'a, M: 'a> ScrollingList<'a, M> {
    /// How tall the well is; all the room there is unless set.
    #[must_use]
    pub fn height(self, height: Extent) -> Self {
        ScrollingList { height, ..self }
    }

    /// The id the app scrolls the rows by.
    #[must_use]
    pub fn id(self, id: impl Into<widget::Id>) -> Self {
        ScrollingList {
            id: Some(id.into()),
            ..self
        }
    }

    /// `on_scroll` with where the rows are scrolled to, each time they move.
    #[must_use]
    pub fn on_scroll(self, on_scroll: impl Fn(widget::scrollable::Viewport) -> M + 'a) -> Self {
        ScrollingList {
            on_scroll: Some(Box::new(on_scroll)),
            ..self
        }
    }

    pub fn view(self, look: &Look) -> Element<'a, M> {
        let rows = stack(look, ROW_GAP).extend(self.rows).padding(Inset::all(WELL_INSET));
        let mut scrolled = crate::scroll::flush(look, rows).height(Extent::Fill);
        if let Some(id) = self.id {
            scrolled = scrolled.id(id);
        }
        if let Some(on_scroll) = self.on_scroll {
            scrolled = scrolled.on_scroll(on_scroll);
        }
        well(look, scrolled, Inset::NONE, self.height, self.focused, true)
    }
}

/// `content` in the list's well, `inset` inside it and `height` tall, its edge lit as the focus
/// tube while `focused`. `scrolls` when the content is a scroll, whose rows pass under the edge.
fn well<'a, M: 'a>(
    look: &Look,
    content: impl Into<Element<'a, M>>,
    inset: Inset,
    height: Extent,
    focused: bool,
    scrolls: bool,
) -> Element<'a, M> {
    let c = look.colors;
    let surface = color::alpha(c.surface_container_low, Opacity::Well.value());
    let edge = look.outline_soft();
    let hairline = look.hairline();
    let border = Border {
        color: edge,
        width: hairline,
        radius: radius(WELL_RADIUS).into(),
    };
    let rows = widget::container(content)
        .padding(inset.padding())
        .width(Length::Fill)
        .height(height.length())
        .style(move |_theme| container::Style {
            background: Some(Background::Color(surface)),
            border: if scrolls {
                Border::default().rounded(radius(WELL_RADIUS))
            } else {
                border
            },
            ..container::Style::default()
        });
    let well: Element<'a, M> = if scrolls {
        // The edge is the well's, over the rows that scroll under it: drawn as the container's
        // border it would sit under them, since the scroll draws its rows in a layer of its own,
        // after the container's. A stack draws its upper child in a layer after the lower one's.
        let rim = widget::container(widget::Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme| container::Style {
                border,
                ..container::Style::default()
            });
        widget::Stack::with_children([rows.into(), rim.into()])
            .width(Length::Fill)
            .height(height.length())
            .into()
    } else {
        rows.into()
    };
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
