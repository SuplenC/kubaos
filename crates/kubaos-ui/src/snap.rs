//! Boxes rounded up to whole device pixels.
//!
//! A box sized by its text (a label, a toast, a chip) is as wide as the text's advance, a fraction
//! of a pixel; its far edge then blurs, and so does everything placed after it. [`snap`] lays its
//! content out as usual and rounds the box's width and height up to the next device pixel, keeping
//! the content where it was. Rows and stacks snap every child, so on the grid a sibling's position
//! is always whole.
//!
//! Centring has the same trap: half of an odd number of device pixels is a half pixel. [`centre`]
//! and [`centre_x`] round the offset to a whole one.
//!
//! # Owns
//!
//! - [`snap`], a transparent wrapper: the content keeps its own state and layout children; only its
//!   box grows by under one device pixel. [`snapped`], the same for a layout node. [`placed`], the
//!   same with each child moved to the nearest device pixel.
//! - [`centre`] and [`centre_x`]: content centred on whole device pixels.
//!
//! # Not here
//!
//! - Columns of set shares: [`crate::grid`], which splits the width in device pixels itself.

use iced::Element;
use iced::Event;
use iced::Length;
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
use iced::advanced::widget::tree;

use crate::scale::Scale;

/// `content` with its box rounded up to whole device pixels at `scale`.
pub fn snap<'a, M: 'a>(scale: Scale, content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    Element::new(Snap {
        content: content.into(),
        scale,
        place: false,
    })
}

/// `content`, a row or another box of children, with its box rounded up to whole device pixels
/// and each child moved to the nearest one: iced centres a row's children down it by half the
/// difference in height, which is half a device pixel whenever that is odd.
pub fn placed<'a, M: 'a>(scale: Scale, content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    Element::new(Snap {
        content: content.into(),
        scale,
        place: true,
    })
}

/// `node` with its size rounded up to whole device pixels at `scale`, its children where they were.
pub fn snapped(scale: Scale, node: &layout::Node) -> layout::Node {
    let size = node.size();
    let rounded = Size::new(scale.snap_up(size.width), scale.snap_up(size.height));
    layout::Node::with_children(rounded, node.children().to_vec()).move_to(node.bounds().position())
}

/// `content` in the middle of all the room there is, both ways, on whole device pixels.
pub fn centre<'a, M: 'a>(scale: Scale, content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    Element::new(Centre {
        content: content.into(),
        scale,
        vertical: true,
    })
}

/// `content` in the middle of the width there is, on a whole device pixel, as tall as itself.
pub fn centre_x<'a, M: 'a>(scale: Scale, content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    Element::new(Centre {
        content: content.into(),
        scale,
        vertical: false,
    })
}

struct Snap<'a, M> {
    content: Element<'a, M>,
    scale: Scale,
    /// The children moved to the nearest device pixel, too.
    place: bool,
}

impl<M> Widget<M, Theme, Renderer> for Snap<'_, M> {
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let node = self.content.as_widget_mut().layout(tree, renderer, limits);
        if !self.place {
            return snapped(self.scale, &node);
        }
        let scale = self.scale;
        let children = node
            .children()
            .iter()
            .map(|child| {
                let at = child.bounds().position();
                child
                    .clone()
                    .move_to(iced::Point::new(scale.snap(at.x), scale.snap(at.y)))
            })
            .collect();
        let placed = layout::Node::with_children(node.size(), children).move_to(node.bounds().position());
        snapped(scale, &placed)
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        // The slot is reported even when its content reports nothing (a canvas, an image), so the
        // geometry check sees every child of a row or stack.
        operation.container(None, layout.bounds());
        let content = &mut self.content;
        operation.traverse(&mut |operation| {
            content.as_widget_mut().operate(tree, layout, renderer, operation);
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
        self.content
            .as_widget_mut()
            .update(tree, event, layout, cursor, renderer, clipboard, shell, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
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
        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

struct Centre<'a, M> {
    content: Element<'a, M>,
    scale: Scale,
    /// Centred down as well as across.
    vertical: bool,
}

impl<M> Widget<M, Theme, Renderer> for Centre<'_, M> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, if self.vertical { Length::Fill } else { Length::Shrink })
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let room = limits.width(Length::Fill).max();
        let node = snapped(
            self.scale,
            &self
                .content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &limits.loose()),
        );
        let size = node.size();
        let height = if self.vertical { room.height } else { size.height };
        // Across, as a stack centres its children: ties go down, as `Stack` in QML puts them.
        let spare = room.width - size.width;
        let x = if self.vertical {
            self.scale.snap(spare / 2.0)
        } else {
            self.scale.centre(spare)
        }
        .max(0.0);
        let y = if self.vertical {
            self.scale.snap((room.height - size.height) / 2.0).max(0.0)
        } else {
            0.0
        };
        layout::Node::with_children(
            Size::new(room.width, height),
            vec![node.move_to(iced::Point::new(x, y))],
        )
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        let Some(content) = layout.children().next() else {
            return;
        };
        // The room the content is centred in, so the geometry check can see it is.
        operation.container(None, layout.bounds());
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
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
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
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        layout
            .children()
            .next()
            .map_or_else(mouse::Interaction::default, |content| {
                self.content
                    .as_widget()
                    .mouse_interaction(&tree.children[0], content, cursor, viewport, renderer)
            })
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
        if let Some(content) = layout.children().next() {
            self.content
                .as_widget()
                .draw(&tree.children[0], renderer, theme, style, content, cursor, viewport);
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
        let content = layout.children().next()?;
        self.content
            .as_widget_mut()
            .overlay(&mut tree.children[0], content, renderer, viewport, translation)
    }
}

#[cfg(test)]
mod tests;
