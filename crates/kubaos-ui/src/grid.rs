//! The grid primitive: columns whose edges land on whole device pixels.
//!
//! iced's own grid and its rows' shares of the room split the width in fractions of a pixel, so a
//! column's edge, and everything in the column, falls between two device pixels and blurs. This one
//! splits the width in device pixels instead: each column gets its weight's share rounded down, the
//! leftover pixels go one each to the first columns, and together they fill the width exactly.
//!
//! # Owns
//!
//! - The column arithmetic ([`columns`], [`weighted`]) and the [`Grid`] widget that lays its
//!   children out by it.
//!
//! # Not here
//!
//! - The other layout primitives: [`crate::layout`].

use std::fmt;

use iced::Element;
use iced::Event;
use iced::Length;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Size;
use iced::Theme;
use iced::Vector;
use iced::advanced::Clipboard;
use iced::advanced::Layout;
use iced::advanced::Shell;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::Operation;
use iced::advanced::widget::Tree;

use crate::probe::Align;
use crate::probe::Tag;
use crate::probe::probe;
use crate::scale::Scale;
use crate::snap::snapped;
use crate::tokens::Space;

/// A column's left edge and width, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub x: f32,
    pub width: f32,
}

/// The `count` equal columns of a grid `width` logical pixels wide, `gap` apart, at `scale`. Every
/// edge is on a whole device pixel; the widths differ by at most one device pixel.
pub fn columns(width: f32, count: u16, gap: Space, scale: Scale) -> Vec<Span> {
    weighted(width, &vec![1; usize::from(count.max(1))], gap, scale)
}

/// Columns of a grid `width` logical pixels wide, `gap` apart, at `scale`, each as wide as its
/// share of `weights`. Every edge is on a whole device pixel; each width is within one device pixel
/// of its exact share.
pub fn weighted(width: f32, weights: &[u16], gap: Space, scale: Scale) -> Vec<Span> {
    let weights: Vec<u32> = if weights.is_empty() {
        vec![1]
    } else {
        weights.iter().map(|&w| u32::from(w.max(1))).collect()
    };
    let count = weights.len() as u32;
    let factor = scale.factor();
    // Whole device pixels: a width a hair under a pixel from float error still counts as it.
    let total = (width * factor + 1e-3).floor().max(0.0) as u32;
    let gap = (f32::from(gap.px()) * factor).round() as u32;
    let room = total.saturating_sub(gap * (count - 1));
    let shares: u32 = weights.iter().sum();
    let mut widths: Vec<u32> = weights.iter().map(|w| room * w / shares).collect();
    let leftover = room - widths.iter().sum::<u32>();
    for w in widths.iter_mut().take(leftover as usize) {
        *w += 1;
    }
    let mut x = 0;
    widths
        .into_iter()
        .map(|w| {
            let span = Span {
                x: x as f32 / factor,
                width: w as f32 / factor,
            };
            x += w + gap;
            span
        })
        .collect()
}

/// Children in columns, row by row, each at most its column's width and its box rounded up to whole
/// device pixels. Each row is as tall as its tallest child, and a child whose height is `Fill` is
/// stretched to it, so cards side by side end together.
pub struct Grid<'a, M> {
    children: Vec<Element<'a, M>>,
    weights: Vec<u16>,
    gap: Space,
    scale: Scale,
    centred: bool,
    /// `Fill`: the rows share all the height there is, instead of each being as tall as its tallest
    /// child.
    fill_height: bool,
    /// Every row as tall as the tallest, so cards in two rows end up one size.
    uniform: bool,
}

impl<M> fmt::Debug for Grid<'_, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Grid")
            .field("children", &self.children.len())
            .field("weights", &self.weights)
            .field("gap", &self.gap)
            .field("scale", &self.scale)
            .field("centred", &self.centred)
            .field("fill_height", &self.fill_height)
            .field("uniform", &self.uniform)
            .finish()
    }
}

impl<'a, M: 'a> Grid<'a, M> {
    /// Columns as wide as their share of `weights`: `[1, 3]` is a quarter and three quarters.
    #[must_use]
    pub fn new(weights: &[u16], gap: Space, scale: Scale) -> Self {
        Grid {
            children: Vec::new(),
            weights: if weights.is_empty() { vec![1] } else { weights.to_vec() },
            gap,
            scale,
            centred: false,
            fill_height: false,
            uniform: false,
        }
    }

    /// Every row as tall as the tallest row: a grid of cards that are all one size, whichever row
    /// holds the most.
    #[must_use]
    pub fn uniform_rows(self) -> Self {
        Grid { uniform: true, ..self }
    }

    /// All the height there is, shared by the rows: columns of lists that run to the page's end.
    #[must_use]
    pub fn fill_height(self) -> Self {
        Grid {
            fill_height: true,
            ..self
        }
    }

    /// Each child centred on its row's height, on a whole device pixel, instead of at its top.
    #[must_use]
    pub fn center(self) -> Self {
        Grid { centred: true, ..self }
    }

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
}

impl<M> Widget<M, Theme, Renderer> for Grid<'_, M> {
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn size(&self) -> Size<Length> {
        Size::new(
            Length::Fill,
            if self.fill_height { Length::Fill } else { Length::Shrink },
        )
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let width = limits.width(Length::Fill).max().width;
        let spans = weighted(width, &self.weights, self.gap, self.scale);
        let gap = f32::from(self.gap.px());
        let mut nodes = Vec::with_capacity(self.children.len());
        let mut y = 0.0;
        let mut rows = self.children.iter_mut().zip(&mut tree.children).collect::<Vec<_>>();
        let count = rows.chunks(spans.len()).count();
        // A row's share of the height when the grid fills it. Inside a scroll the height is
        // unbounded, and each row keeps the height it needs instead.
        let room = limits.max().height;
        let share = (self.fill_height && room.is_finite()).then(|| {
            let gaps = gap * count.saturating_sub(1) as f32;
            self.scale.snap((room - gaps).max(0.0) / count.max(1) as f32)
        });
        // Measuring compresses the height, so a `Fill` child reports its content's height instead
        // of the infinite height a scrolled page offers.
        let mut heights = Vec::with_capacity(count);
        for cells in rows.chunks_mut(spans.len()) {
            let mut height: f32 = 0.0;
            for ((child, state), span) in cells.iter_mut().zip(&spans) {
                let cell = layout::Limits::with_compression(
                    Size::ZERO,
                    Size::new(span.width, share.unwrap_or(f32::INFINITY)),
                    Size::new(false, true),
                );
                let node = snapped(self.scale, &child.as_widget_mut().layout(state, renderer, &cell));
                height = height.max(node.size().height);
                nodes.push(node);
            }
            heights.push(share.unwrap_or_else(|| self.scale.snap_up(height)));
        }
        if self.uniform {
            let tallest = heights.iter().copied().fold(0.0, f32::max);
            heights.fill(tallest);
        }
        let mut first = 0;
        for ((index, cells), height) in rows.chunks_mut(spans.len()).enumerate().zip(heights) {
            if index > 0 {
                y += gap;
            }
            let placed = &mut nodes[first..first + cells.len()];
            for (((child, state), span), node) in cells.iter_mut().zip(&spans).zip(placed.iter_mut()) {
                if child.as_widget().size().height.is_fill() {
                    let exact = layout::Limits::new(Size::new(0.0, height), Size::new(span.width, height));
                    *node = snapped(self.scale, &child.as_widget_mut().layout(state, renderer, &exact));
                }
                let down = if self.centred {
                    self.scale.snap((height - node.size().height) / 2.0)
                } else {
                    0.0
                };
                node.move_to_mut(Point::new(span.x, y + down));
            }
            first += cells.len();
            y += height;
        }
        layout::Node::with_children(Size::new(width, y), nodes)
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            for ((child, state), layout) in self.children.iter_mut().zip(&mut tree.children).zip(layout.children()) {
                // Each cell is reported even when its content reports nothing (a canvas, an image),
                // so the geometry check counts the cells into their columns.
                operation.container(None, layout.bounds());
                operation.traverse(&mut |operation| {
                    child.as_widget_mut().operate(state, layout, renderer, operation);
                });
            }
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        for ((child, state), layout) in self.children.iter_mut().zip(&mut tree.children).zip(layout.children()) {
            child
                .as_widget_mut()
                .update(state, event, layout, cursor, renderer, clipboard, shell, viewport);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((child, state), layout) in self.children.iter().zip(&tree.children).zip(layout.children()) {
            child
                .as_widget()
                .draw(state, renderer, theme, style, layout, cursor, viewport);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        overlay::from_children(&mut self.children, tree, layout, renderer, viewport, translation)
    }
}

impl<'a, M: 'a> From<Grid<'a, M>> for Element<'a, M> {
    fn from(grid: Grid<'a, M>) -> Self {
        let tag = Tag::Grid {
            columns: grid.weights.len() as u16,
            gap: f32::from(grid.gap.px()),
            align: if grid.centred { Align::Centre } else { Align::Start },
        };
        probe(tag, Element::new(grid))
    }
}

#[cfg(test)]
mod tests;
