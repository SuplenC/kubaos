//! A held item's sweep: a fill from the item's start to how far the hold has gone, with the item's
//! content again over it in the fill's own ink, cut at the sweep's edge, so the words read on
//! either side of it.
//!
//! # Owns
//!
//! - [`swept`], the item and its sweep laid out as one.
//!
//! # Not here
//!
//! - Timing the hold: the app's, which says how far along it is. The item: [`super::MenuItem`].

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Event;
use iced::Length;
use iced::Rectangle;
use iced::Renderer;
use iced::Size;
use iced::Theme;
use iced::advanced::Clipboard;
use iced::advanced::Layout;
use iced::advanced::Renderer as _;
use iced::advanced::Shell;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::Operation;
use iced::advanced::widget::Tree;

use crate::look::Corner;
use crate::look::radius;
use crate::scale::Scale;

/// `item` with its sweep over it: `fill` from its start to `reach` (0 to 1) of its width, rounded
/// at `corner`, and `copy` (the item's content in the fill's ink) cut to it. Only `item` takes
/// input and reports its boxes; the sweep is paint.
pub(super) fn swept<'a, M: 'a>(
    item: Element<'a, M>,
    copy: Element<'a, M>,
    reach: f32,
    fill: Color,
    corner: Corner,
    scale: Scale,
) -> Element<'a, M> {
    Element::new(Sweep {
        item,
        copy,
        reach: reach.clamp(0.0, 1.0),
        fill,
        corner,
        scale,
    })
}

struct Sweep<'a, M> {
    item: Element<'a, M>,
    copy: Element<'a, M>,
    reach: f32,
    fill: Color,
    corner: Corner,
    scale: Scale,
}

impl<M> Widget<M, Theme, Renderer> for Sweep<'_, M> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.item), Tree::new(&self.copy)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.children[0].diff(&self.item);
        tree.children[1].diff(&self.copy);
    }

    fn size(&self) -> Size<Length> {
        self.item.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.item.as_widget().size_hint()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let item = self
            .item
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        // The copy is laid out in exactly the item's box, so its words sit on the item's own.
        let exact = layout::Limits::new(item.size(), item.size());
        let copy = self
            .copy
            .as_widget_mut()
            .layout(&mut tree.children[1], renderer, &exact);
        layout::Node::with_children(item.size(), vec![item, copy])
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        if let Some(item) = layout.children().next() {
            self.item
                .as_widget_mut()
                .operate(&mut tree.children[0], item, renderer, operation);
        }
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
        if let Some(item) = layout.children().next() {
            self.item.as_widget_mut().update(
                &mut tree.children[0],
                event,
                item,
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
            .map_or_else(mouse::Interaction::default, |item| {
                self.item
                    .as_widget()
                    .mouse_interaction(&tree.children[0], item, cursor, viewport, renderer)
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
        let mut parts = layout.children();
        let (Some(item), Some(copy)) = (parts.next(), parts.next()) else {
            return;
        };
        self.item
            .as_widget()
            .draw(&tree.children[0], renderer, theme, style, item, cursor, viewport);
        let bounds = layout.bounds();
        let reach = self.scale.snap(bounds.width * self.reach);
        if reach <= 0.0 {
            return;
        }
        // The sweep's edge is straight: the item's rounded shape cut at a whole device pixel. A
        // layer's clip replaces its parent's, so it is held to the viewport by hand.
        let edge = Rectangle { width: reach, ..bounds };
        let Some(clip) = edge.intersection(viewport) else {
            return;
        };
        renderer.with_layer(clip, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border::default().rounded(radius(self.corner)),
                    snap: true,
                    ..renderer::Quad::default()
                },
                Background::Color(self.fill),
            );
            self.copy
                .as_widget()
                .draw(&tree.children[1], renderer, theme, style, copy, cursor, viewport);
        });
    }
}
