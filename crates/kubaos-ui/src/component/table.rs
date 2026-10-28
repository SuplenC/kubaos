//! Tables: readings in columns, a dense row each, as the system monitor's processes are.
//!
//! A row is a line of body text with 4 above and below ([`Size::RowCompact`]), so many fit, and
//! the whole row is one control: the pointer over it lays the state layer across all its cells,
//! and the picked row takes the secondary container. Numbers sit at their column's right edge in
//! `mono_large`, so their digits line up.
//!
//! # Owns
//!
//! - [`table`], its [`Column`]s, and [`table_row`], built up with [`TableRow`].
//!
//! # Not here
//!
//! - Label and value pairs: a card's facts.

use iced::Background;
use iced::Border;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Theme;
use iced::alignment;
use iced::widget;

use crate::color;
use crate::component::layered;
use crate::layout::Extent;
use crate::layout::Gap;
use crate::layout::Inset;
use crate::layout::split;
use crate::layout::stack;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::snap::snap;
use crate::text::line;
use crate::text::text;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// One column of a [`table`]: its title, its share of the width, and whether it holds numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Column<'a> {
    title: &'a str,
    weight: u16,
    numeric: bool,
}

impl<'a> Column<'a> {
    /// A column of words titled `title`, `weight` shares of the width.
    #[must_use]
    pub fn new(title: &'a str, weight: u16) -> Column<'a> {
        Column {
            title,
            weight,
            numeric: false,
        }
    }

    /// A column of numbers: at its right edge, in mono.
    #[must_use]
    pub fn numeric(self) -> Column<'a> {
        Column { numeric: true, ..self }
    }
}

/// `rows` under a header naming `columns`.
pub fn table<'a, M: 'a>(
    look: &Look,
    columns: &[Column<'_>],
    rows: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    let c = look.colors;
    let weights: Vec<u16> = columns.iter().map(|column| column.weight).collect();
    let titles = columns.iter().map(|column| {
        cell(
            look,
            text(column.title.to_owned(), Type::Label, c.on_surface_variant),
            column.numeric,
        )
    });
    let header = widget::container(split(look, &weights, Space::S12).extend(titles))
        .width(Length::Fill)
        .padding(Inset::xy(Space::S12, Space::S4).padding());
    let view = stack(look, Gap::NONE).width(Extent::Fill).push(header).extend(rows);
    probe::component("Table", ["", "", ""], Fixed::NONE, view)
}

/// One row of a table, its `cells` left to right, built up then drawn with [`TableRow::view`].
#[derive(Debug)]
pub struct TableRow<M> {
    cells: Vec<String>,
    selected: bool,
    on_press: Option<M>,
}

/// A row of `cells`, one per column, each one line ending in an ellipsis where it doesn't fit.
pub fn table_row<M>(cells: Vec<String>) -> TableRow<M> {
    TableRow {
        cells,
        selected: false,
        on_press: None,
    }
}

impl<M: Clone> TableRow<M> {
    /// The picked row: the secondary container.
    #[must_use]
    pub fn selected(self, selected: bool) -> Self {
        TableRow { selected, ..self }
    }

    #[must_use]
    pub fn on_press(self, message: M) -> Self {
        TableRow {
            on_press: Some(message),
            ..self
        }
    }

    /// The row laid out in `columns`, the table's.
    pub fn view<'a>(self, look: &Look, columns: &[Column<'_>]) -> Element<'a, M>
    where
        M: 'a,
    {
        let c = look.colors;
        let (fill, main, muted) = if self.selected {
            (
                Some(c.secondary_container),
                c.on_secondary_container,
                color::alpha(c.on_secondary_container, Opacity::SubduedText.value()),
            )
        } else {
            (None, c.on_surface, c.on_surface_variant)
        };
        let weights: Vec<u16> = columns.iter().map(|column| column.weight).collect();
        let cells = self.cells.into_iter().zip(columns).map(|(words, column)| {
            if column.numeric {
                cell(look, text(words, Type::MonoLarge, muted), true)
            } else {
                line(words, Type::Body, main)
            }
        });
        let enabled = self.on_press.is_some();
        let pressable = widget::button(split(look, &weights, Space::S12).extend(cells))
            .width(Length::Fill)
            .height(f32::from(Size::RowCompact.px()))
            .padding(Inset::xy(Space::S12, Space::S4).padding())
            .on_press_maybe(self.on_press)
            .style(move |_theme: &Theme, _status| widget::button::Style {
                background: fill.map(Background::Color),
                text_color: main,
                border: Border::default().rounded(radius(Radius::Control)),
                shadow: Shadow::default(),
                snap: true,
            });
        let view = layered(pressable, Radius::Control, c.on_surface, enabled, None);
        probe::component("TableRow", ["", "", ""], Fixed::HEIGHT, view)
    }
}

/// `content` in its cell: at the right edge when it is a number, its box rounded up to whole device
/// pixels so its left edge lands on one, else where it starts.
fn cell<'a, M: 'a>(look: &Look, content: Element<'a, M>, numeric: bool) -> Element<'a, M> {
    if !numeric {
        return content;
    }
    widget::container(snap(look.scale, content))
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Right)
        .into()
}

#[cfg(test)]
mod tests;
