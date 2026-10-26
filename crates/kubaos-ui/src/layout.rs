//! Layout on the grid: pages, sections, rows, stacks and grids whose gaps, paddings and fixed sizes
//! are tokens. None of them takes a number of pixels, so a view built from them can't leave the
//! spacing scale.
//!
//! # Owns
//!
//! - [`Gap`], [`Inset`] and [`Extent`], the only spacing and sizes the primitives take, and the
//!   primitives: [`page`], [`header`], [`section`], [`row`], [`stack`] and [`grid`].
//!
//! # Not here
//!
//! - The grid's cell arithmetic and widget: [`crate::grid`]. Components: [`crate::component`].

use std::fmt;

use iced::Alignment;
use iced::Background;
use iced::Element;
use iced::Length;
use iced::Padding;
use iced::widget;
use iced::widget::container;

use crate::grid::Grid;
use crate::look::Look;
use crate::scale::Scale;
use crate::snap::centre_x;
use crate::snap::snap;
use crate::text::text;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// The space between siblings: a step of the spacing scale, or none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Gap(Option<Space>);

impl Gap {
    pub const NONE: Gap = Gap(None);

    /// The gap in logical pixels.
    pub fn px(self) -> f32 {
        self.0.map_or(0.0, |space| f32::from(space.px()))
    }
}

impl From<Space> for Gap {
    fn from(space: Space) -> Self {
        Gap(Some(space))
    }
}

/// Padding inside each edge, from the spacing scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Inset {
    top: Gap,
    right: Gap,
    bottom: Gap,
    left: Gap,
}

impl Inset {
    pub const NONE: Inset = Inset {
        top: Gap::NONE,
        right: Gap::NONE,
        bottom: Gap::NONE,
        left: Gap::NONE,
    };

    /// `space` inside every edge.
    #[must_use]
    pub fn all(space: Space) -> Inset {
        Inset::xy(space, space)
    }

    /// `x` inside the left and right edges, `y` inside the top and bottom.
    #[must_use]
    pub fn xy(x: Space, y: Space) -> Inset {
        Inset {
            top: y.into(),
            right: x.into(),
            bottom: y.into(),
            left: x.into(),
        }
    }

    /// `x` inside the left and right edges only.
    #[must_use]
    pub fn x(x: Space) -> Inset {
        Inset {
            right: x.into(),
            left: x.into(),
            ..Inset::NONE
        }
    }

    /// `y` inside the top and bottom edges only.
    #[must_use]
    pub fn y(y: Space) -> Inset {
        Inset {
            top: y.into(),
            bottom: y.into(),
            ..Inset::NONE
        }
    }

    #[must_use]
    pub fn top(self, gap: impl Into<Gap>) -> Inset {
        Inset {
            top: gap.into(),
            ..self
        }
    }

    #[must_use]
    pub fn right(self, gap: impl Into<Gap>) -> Inset {
        Inset {
            right: gap.into(),
            ..self
        }
    }

    #[must_use]
    pub fn bottom(self, gap: impl Into<Gap>) -> Inset {
        Inset {
            bottom: gap.into(),
            ..self
        }
    }

    #[must_use]
    pub fn left(self, gap: impl Into<Gap>) -> Inset {
        Inset {
            left: gap.into(),
            ..self
        }
    }

    /// The inset as iced padding, in logical pixels.
    pub fn padding(self) -> Padding {
        Padding {
            top: self.top.px(),
            right: self.right.px(),
            bottom: self.bottom.px(),
            left: self.left.px(),
        }
    }
}

/// How wide or tall a primitive is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Extent {
    /// All the room there is, shared with other fills. Columns of set shares are a [`grid`]'s,
    /// which keeps their edges on device pixels.
    Fill,
    /// As much as the content needs.
    Shrink,
    /// A component's fixed size.
    Fixed(Size),
}

impl Extent {
    pub fn length(self) -> Length {
        match self {
            Extent::Fill => Length::Fill,
            Extent::Shrink => Length::Shrink,
            Extent::Fixed(size) => Length::Fixed(f32::from(size.px())),
        }
    }
}

/// Children side by side, `gap` apart, each box rounded up to whole device pixels at `look`'s
/// scale, so every child starts on one.
pub fn row<'a, M: 'a>(look: &Look, gap: impl Into<Gap>) -> Row<'a, M> {
    Row {
        inner: widget::Row::new().spacing(gap.into().px()),
        scale: look.scale,
    }
}

/// See [`row`].
pub struct Row<'a, M> {
    inner: widget::Row<'a, M>,
    scale: Scale,
}

impl<M> fmt::Debug for Row<'_, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Row").finish_non_exhaustive()
    }
}

impl<'a, M: 'a> Row<'a, M> {
    #[must_use]
    pub fn push(self, child: impl Into<Element<'a, M>>) -> Self {
        Row {
            inner: self.inner.push(snap(self.scale, child)),
            ..self
        }
    }

    #[must_use]
    pub fn extend(self, children: impl IntoIterator<Item = Element<'a, M>>) -> Self {
        let scale = self.scale;
        Row {
            inner: self.inner.extend(children.into_iter().map(|child| snap(scale, child))),
            ..self
        }
    }

    /// The children centred on the row's height.
    #[must_use]
    pub fn center(self) -> Self {
        Row {
            inner: self.inner.align_y(Alignment::Center),
            ..self
        }
    }

    /// The children on the row's bottom edge.
    #[must_use]
    pub fn bottom(self) -> Self {
        Row {
            inner: self.inner.align_y(Alignment::End),
            ..self
        }
    }

    #[must_use]
    pub fn width(self, width: Extent) -> Self {
        Row {
            inner: self.inner.width(width.length()),
            ..self
        }
    }

    #[must_use]
    pub fn height(self, height: Extent) -> Self {
        Row {
            inner: self.inner.height(height.length()),
            ..self
        }
    }

    #[must_use]
    pub fn padding(self, inset: Inset) -> Self {
        Row {
            inner: self.inner.padding(inset.padding()),
            ..self
        }
    }
}

impl<'a, M: 'a> From<Row<'a, M>> for Element<'a, M> {
    fn from(row: Row<'a, M>) -> Self {
        row.inner.into()
    }
}

/// Children one above the other, `gap` apart, each box rounded up to whole device pixels at
/// `look`'s scale.
pub fn stack<'a, M: 'a>(look: &Look, gap: impl Into<Gap>) -> Stack<'a, M> {
    Stack {
        children: Vec::new(),
        gap: gap.into(),
        scale: look.scale,
        centred: false,
        width: None,
        height: None,
        inset: Inset::NONE,
    }
}

/// See [`stack`].
pub struct Stack<'a, M> {
    children: Vec<Element<'a, M>>,
    gap: Gap,
    scale: Scale,
    centred: bool,
    /// Set only when asked for: by default the stack grows to fit a child that fills, as iced's
    /// column does.
    width: Option<Extent>,
    height: Option<Extent>,
    inset: Inset,
}

impl<M> fmt::Debug for Stack<'_, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stack")
            .field("children", &self.children.len())
            .field("gap", &self.gap)
            .field("centred", &self.centred)
            .finish_non_exhaustive()
    }
}

impl<'a, M: 'a> Stack<'a, M> {
    #[must_use]
    pub fn push(mut self, child: impl Into<Element<'a, M>>) -> Self {
        self.children.push(child.into());
        self
    }

    #[must_use]
    pub fn extend(mut self, children: impl IntoIterator<Item = Element<'a, M>>) -> Self {
        self.children.extend(children);
        self
    }

    /// Each child centred across the stack, on a whole device pixel. The stack takes the whole
    /// width, so the centre is the room's.
    #[must_use]
    pub fn center(self) -> Self {
        Stack {
            centred: true,
            width: Some(Extent::Fill),
            ..self
        }
    }

    #[must_use]
    pub fn width(self, width: Extent) -> Self {
        Stack {
            width: Some(width),
            ..self
        }
    }

    #[must_use]
    pub fn height(self, height: Extent) -> Self {
        Stack {
            height: Some(height),
            ..self
        }
    }

    #[must_use]
    pub fn padding(self, inset: Inset) -> Self {
        Stack { inset, ..self }
    }
}

impl<'a, M: 'a> From<Stack<'a, M>> for Element<'a, M> {
    fn from(stack: Stack<'a, M>) -> Self {
        let Stack {
            children,
            gap,
            scale,
            centred,
            width,
            height,
            inset,
        } = stack;
        let mut column = widget::Column::with_children(children.into_iter().map(|child| {
            if centred {
                centre_x(scale, child)
            } else {
                snap(scale, child)
            }
        }))
        .spacing(gap.px())
        .padding(inset.padding());
        if let Some(width) = width {
            column = column.width(width.length());
        }
        if let Some(height) = height {
            column = column.height(height.length());
        }
        column.into()
    }
}

/// `columns` equal columns, `gap` apart both ways, filled row by row. Every cell's edges land on
/// whole device pixels at `look`'s scale.
pub fn grid<'a, M: 'a>(look: &Look, columns: u16, gap: Space) -> Grid<'a, M> {
    Grid::new(&vec![1; usize::from(columns.max(1))], gap, look.scale)
}

/// Columns as wide as their share of `weights`, `gap` apart, filled row by row: `[1, 3]` is a label
/// column a quarter wide beside the rest. Every edge lands on a whole device pixel.
pub fn split<'a, M: 'a>(look: &Look, weights: &[u16], gap: Space) -> Grid<'a, M> {
    Grid::new(weights, gap, look.scale)
}

/// Room that grows across a row, pushing what follows to its far end. It never grows down, so the
/// row stays as tall as its content.
pub fn fill<'a, M: 'a>() -> Element<'a, M> {
    widget::Space::new().width(Length::Fill).into()
}

/// Empty room of `space` both ways.
pub fn gap<'a, M: 'a>(space: Space) -> Element<'a, M> {
    let px = f32::from(space.px());
    widget::Space::new().width(px).height(px).into()
}

/// A whole page: the surface, with the page's insets around `content`.
///
/// The insets are Kōbō's panel content, on the grid: 40 at the sides, 32 above, 24 below, where the
/// footer's buttons sit.
pub fn page<'a, M: 'a>(look: &Look, content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    let surface = look.colors.surface;
    widget::container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Inset::x(Space::S40).top(Space::S32).bottom(Space::S24).padding())
        .style(move |_theme| container::Style {
            background: Some(Background::Color(surface)),
            ..container::Style::default()
        })
        .into()
}

/// A page's header: the Japanese accent, the title, and one line of explanation under it.
pub fn header<'a, M: 'a>(look: &Look, accent: &'a str, title: &'a str, lede: &'a str) -> Element<'a, M> {
    let c = &look.colors;
    stack(look, Gap::NONE)
        .push(text(accent, Type::Accent, c.primary))
        .push(text(title, Type::Title, c.on_surface))
        .push(gap(Space::S8))
        .push(text(lede, Type::Body, c.on_surface_variant))
        .into()
}

/// A titled part of a page: the title, an optional line under it, then `content` below.
pub fn section<'a, M: 'a>(
    look: &Look,
    title: &'a str,
    description: Option<&'a str>,
    content: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    let c = &look.colors;
    let mut head = stack(look, Space::S4).push(text(title, Type::Heading, c.on_surface));
    if let Some(description) = description {
        head = head.push(text(description, Type::Body, c.on_surface_variant));
    }
    stack(look, Space::S16).push(head).push(content).into()
}

/// A hairline across the width, in the soft outline colour.
pub fn divider<'a, M: 'a>(look: &Look) -> Element<'a, M> {
    let soft = look.outline_soft();
    widget::container(widget::Space::new())
        .width(Length::Fill)
        .height(look.hairline())
        .style(move |_theme| container::Style::default().background(soft))
        .into()
}

#[cfg(test)]
mod tests;
