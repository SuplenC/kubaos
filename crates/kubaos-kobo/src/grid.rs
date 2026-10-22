//! A grid of cells that are all the same size: an equal share of the width,
//! and as tall as the tallest cell.
//!
//! iced's own `grid` either fixes the cell height (an aspect ratio, or the
//! height shared out) or lets every cell keep its own, and a `row` of
//! `Fill`-height cards inside a scrollable shrinks each card to its content.
//! Cards laid out side by side then end at different heights.
//!
//! # Owns
//! - Splitting the width into equal columns and stretching every cell to the
//!   tallest one's height. A cell fills it when its height is `Fill`.
//!
//! # Not here
//! - The cards themselves (`src/view.rs`, `summary`).

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, mouse, overlay};
use iced::{Element, Event, Length, Rectangle, Renderer, Size, Theme, Vector};

pub struct Grid<'a, Message> {
    columns: usize,
    spacing: f32,
    cells: Vec<Element<'a, Message>>,
}

/// `cells`, `columns` to a row, `spacing` apart both ways.
pub fn grid<'a, Message>(
    columns: usize,
    spacing: f32,
    cells: impl IntoIterator<Item = Element<'a, Message>>,
) -> Grid<'a, Message> {
    Grid {
        columns: columns.max(1),
        spacing,
        cells: cells.into_iter().collect(),
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Grid<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        self.cells.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.cells);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width = limits.max().width;
        #[expect(clippy::cast_precision_loss, reason = "a handful of columns")]
        let gaps = self.spacing * (self.columns - 1) as f32;
        #[expect(clippy::cast_precision_loss, reason = "a handful of columns")]
        let cell_width = ((width - gaps) / self.columns as f32).max(0.0);
        // Measuring compresses the height, so a `Fill` cell reports its
        // content's height instead of the infinite height a scrollable offers.
        let measure = layout::Limits::with_compression(
            Size::new(cell_width, 0.0),
            Size::new(cell_width, f32::INFINITY),
            Size::new(false, true),
        );
        let height = self
            .cells
            .iter_mut()
            .zip(&mut tree.children)
            .map(|(cell, tree)| {
                cell.as_widget_mut()
                    .layout(tree, renderer, &measure)
                    .size()
                    .height
            })
            .fold(0.0, f32::max);
        let exact =
            layout::Limits::new(Size::new(cell_width, height), Size::new(cell_width, height));
        let nodes: Vec<_> = self
            .cells
            .iter_mut()
            .zip(&mut tree.children)
            .enumerate()
            .map(|(i, (cell, tree))| {
                #[expect(clippy::cast_precision_loss, reason = "a handful of cells")]
                let (col, row) = ((i % self.columns) as f32, (i / self.columns) as f32);
                let at = (
                    col * (cell_width + self.spacing),
                    row * (height + self.spacing),
                );
                cell.as_widget_mut()
                    .layout(tree, renderer, &exact)
                    .move_to(at)
            })
            .collect();
        #[expect(clippy::cast_precision_loss, reason = "a handful of rows")]
        let rows = self.cells.len().div_ceil(self.columns) as f32;
        let height = (rows * (height + self.spacing) - self.spacing).max(0.0);
        layout::Node::with_children(Size::new(width, height), nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            for ((cell, tree), layout) in self
                .cells
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
            {
                cell.as_widget_mut()
                    .operate(tree, layout, renderer, operation);
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
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((cell, tree), layout) in self
            .cells
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            cell.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
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
        self.cells
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((cell, tree), layout)| {
                cell.as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
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
        for ((cell, tree), layout) in self.cells.iter().zip(&tree.children).zip(layout.children()) {
            cell.as_widget()
                .draw(tree, renderer, theme, style, layout, cursor, viewport);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.cells,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: 'a> From<Grid<'a, Message>> for Element<'a, Message> {
    fn from(g: Grid<'a, Message>) -> Self {
        Element::new(g)
    }
}

#[cfg(test)]
mod tests;
