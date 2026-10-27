//! Layout on the grid: pages, sections, rows, stacks and grids whose gaps, paddings and fixed sizes
//! are tokens. None of them takes a number of pixels, so a view built from them can't leave the
//! spacing scale.
//!
//! # Owns
//!
//! - [`Gap`], [`Inset`] and [`Extent`], the only spacing and sizes the primitives take, and the
//!   primitives: [`page`], [`container`], [`header`], [`section`], [`row`], [`stack`] and [`grid`].
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
use iced::advanced;
use iced::advanced::widget::Tree;
use iced::widget;
use iced::widget::text::IntoFragment;

use crate::grid::Grid;
use crate::look::Look;
use crate::probe;
use crate::probe::Align;
use crate::probe::Axis;
use crate::probe::Fixed;
use crate::probe::Tag;
use crate::probe::probe;
use crate::scale::Scale;
pub use crate::scroll::Scroll;
pub use crate::scroll::scroll;
use crate::snap::centre_x;
use crate::snap::placed;
use crate::snap::snap;
use crate::text::text;
use crate::tokens;
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
    let gap = gap.into();
    Row {
        inner: widget::Row::new().spacing(gap.px()),
        scale: look.scale,
        gap,
        align: Align::Start,
    }
}

/// See [`row`].
pub struct Row<'a, M> {
    inner: widget::Row<'a, M>,
    scale: Scale,
    gap: Gap,
    /// How the children line up down the row, for the geometry check.
    align: Align,
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
            align: Align::Centre,
            ..self
        }
    }

    /// The children on the row's bottom edge.
    #[must_use]
    pub fn bottom(self) -> Self {
        Row {
            inner: self.inner.align_y(Alignment::End),
            align: Align::End,
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
        let tag = Tag::Layout {
            axis: Axis::X,
            gap: row.gap.px(),
            align: row.align,
        };
        probe(tag, placed(row.scale, row.inner))
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
        let tag = Tag::Layout {
            axis: Axis::Y,
            gap: gap.px(),
            align: if centred { Align::Centre } else { Align::Start },
        };
        probe(tag, column)
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
    probe(Tag::Spacer { fill: true }, widget::Space::new().width(Length::Fill))
}

/// Empty room of `space` both ways.
pub fn gap<'a, M: 'a>(space: Space) -> Element<'a, M> {
    let px = f32::from(space.px());
    probe(Tag::Spacer { fill: false }, widget::Space::new().width(px).height(px))
}

/// A whole page: the surface, with `content` the container padding in at the sides
/// ([`container_padding`]), 32 below the page's top and 24 above its foot, where the footer's
/// buttons sit (Kōbō's panel), filling the room both ways.
pub fn page<'a, M: 'a>(look: &Look, content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    page_over(content, Some(look.colors.surface))
}

/// [`page`] with nothing behind it, over a surface of its own: Kōbō's glass panel, a pane beside a
/// rail.
pub fn page_clear<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    page_over(content, None)
}

fn page_over<'a, M: 'a>(content: impl Into<Element<'a, M>>, surface: Option<iced::Color>) -> Element<'a, M> {
    let view = Element::new(Container {
        content: content.into(),
        height: Length::Fill,
        top: Space::S32.into(),
        bottom: Space::S24.into(),
        surface,
    });
    probe::component("Page", ["", "", ""], Fixed::HOLDS, view)
}

/// A panel's body or a toolbar band: `content` the container padding in at the sides, as wide as
/// the room and as tall as the content. Above and below, the content keeps its own spacing.
pub fn container<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    let view = Element::new(Container {
        content: content.into(),
        height: Length::Shrink,
        top: Gap::NONE,
        bottom: Gap::NONE,
        surface: None,
    });
    probe::component("Container", ["", "", ""], Fixed::HOLDS, view)
}

/// The padding at the sides of a page or a panel's body `width` wide: Tailwind's container
/// (`px-4 sm:px-6 lg:px-8`), 16 under 640, 24 from 640, 32 from 1024 (`tokens::CONTAINER`).
pub fn container_padding(width: f32) -> Space {
    tokens::CONTAINER
        .iter()
        .rev()
        .find(|&&(from, _)| width >= f32::from(from))
        .map_or(tokens::CONTAINER[0].1, |&(_, padding)| padding)
}

/// Content padded at the sides by [`container_padding`] for the width it is given.
struct Container<'a, M> {
    content: Element<'a, M>,
    height: Length,
    top: Gap,
    bottom: Gap,
    surface: Option<iced::Color>,
}

impl<M> iced::advanced::Widget<M, iced::Theme, iced::Renderer> for Container<'_, M> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> iced::Size<Length> {
        iced::Size::new(Length::Fill, self.height)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &advanced::layout::Limits,
    ) -> advanced::layout::Node {
        let width = limits.width(Length::Fill).max().width;
        let side = f32::from(container_padding(width).px());
        let padding = Padding {
            top: self.top.px(),
            right: side,
            bottom: self.bottom.px(),
            left: side,
        };
        let content = &mut self.content;
        advanced::layout::padded(limits, Length::Fill, self.height, padding, |limits| {
            content.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
        })
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn advanced::widget::Operation,
    ) {
        let Some(content) = layout.children().next() else {
            return;
        };
        let widget = &mut self.content;
        operation.traverse(&mut |operation| {
            widget
                .as_widget_mut()
                .operate(&mut tree.children[0], content, renderer, operation);
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: advanced::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, M>,
        viewport: &iced::Rectangle,
    ) {
        if let Some(content) = layout.children().next() {
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                content,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: advanced::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> advanced::mouse::Interaction {
        layout
            .children()
            .next()
            .map_or_else(advanced::mouse::Interaction::default, |content| {
                self.content
                    .as_widget()
                    .mouse_interaction(&tree.children[0], content, cursor, viewport, renderer)
            })
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &advanced::renderer::Style,
        layout: advanced::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        use iced::advanced::Renderer as _;

        if let Some(surface) = self.surface {
            renderer.fill_quad(
                advanced::renderer::Quad {
                    bounds: layout.bounds(),
                    snap: true,
                    ..advanced::renderer::Quad::default()
                },
                Background::Color(surface),
            );
        }
        if let Some(content) = layout.children().next() {
            self.content
                .as_widget()
                .draw(&tree.children[0], renderer, theme, style, content, cursor, viewport);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: advanced::Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &iced::Rectangle,
        translation: iced::Vector,
    ) -> Option<advanced::overlay::Element<'b, M, iced::Theme, iced::Renderer>> {
        let content = layout.children().next()?;
        self.content
            .as_widget_mut()
            .overlay(&mut tree.children[0], content, renderer, viewport, translation)
    }
}

/// `children` side by side, `gap` apart both ways, a new line started where the next one doesn't
/// fit: chips, tags. Each box is rounded up to whole device pixels, so every child starts on one.
/// The lines tell the geometry check nothing; each child still says what it is.
pub fn flow<'a, M: 'a>(look: &Look, gap: Space, children: impl IntoIterator<Item = Element<'a, M>>) -> Element<'a, M> {
    let scale = look.scale;
    let px = f32::from(gap.px());
    widget::Row::with_children(children.into_iter().map(|child| snap(scale, child)))
        .spacing(px)
        .wrap()
        .vertical_spacing(px)
        .into()
}

/// A page's header: the Japanese accent, the title, and one line of explanation under it.
pub fn header<'a, M: 'a>(
    look: &Look,
    accent: impl IntoFragment<'a>,
    title: impl IntoFragment<'a>,
    lede: impl IntoFragment<'a>,
) -> Element<'a, M> {
    let c = &look.colors;
    let view: Element<'a, M> = stack(look, Gap::NONE)
        .push(text(accent, Type::Accent, c.primary))
        .push(text(title, Type::Title, c.on_surface))
        .push(gap(Space::S8))
        .push(text(lede, Type::Body, c.on_surface_variant))
        .into();
    probe::component("Header", ["", "", ""], Fixed::NONE, view)
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
    let view: Element<'a, M> = stack(look, Space::S16).push(head).push(content).into();
    probe::component("Section", ["", "", ""], Fixed::HOLDS, view)
}

/// A hairline across the width, in the soft outline colour.
pub fn divider<'a, M: 'a>(look: &Look) -> Element<'a, M> {
    let soft = look.outline_soft();
    let view: Element<'a, M> = widget::container(widget::Space::new())
        .width(Length::Fill)
        .height(look.hairline())
        .style(move |_theme| widget::container::Style::default().background(soft))
        .into();
    probe::component("Divider", ["", "", ""], Fixed::NONE, view)
}

#[cfg(test)]
mod tests;
