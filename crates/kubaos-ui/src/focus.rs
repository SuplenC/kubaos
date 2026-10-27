//! The focus ring: the keyboard's place on screen.
//!
//! The ring is drawn outside its control, in a band [`tokens::FOCUS_OFFSET`] wide around it, and
//! takes no layout space: focus moving never moves anything, and a focusable control's edge lines
//! up with a plain one's. A parent that clips (a scrolled list) keeps at least that band of padding
//! so the ring shows whole.
//!
//! # Owns
//!
//! - [`focus_ring`], and the ring's geometry at a scale ([`ring_bounds`]).
//!
//! # Not here
//!
//! - Which control has focus: the app, which moves it with Tab and the arrows.

use std::slice;

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
use iced::Vector;
use iced::advanced::Clipboard;
use iced::advanced::Layout;
use iced::advanced::Renderer as _;
use iced::advanced::Shell;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::Operation;
use iced::advanced::widget::Tree;

use crate::look::Corner;
use crate::look::Look;
use crate::look::ring_radius;
use crate::scale::Scale;
use crate::snap::snapped;
use crate::tokens;

/// Which colour the ring takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingColor {
    /// The accent: every control.
    Accent,
    /// The cold accent, on cards where the accent already means picked.
    Cold,
}

/// `content`, a control whose corners have radius `radius`, with the focus ring around it when
/// `focused`.
pub fn focus_ring<'a, M: 'a>(
    look: &Look,
    content: impl Into<Element<'a, M>>,
    radius: impl Into<Corner>,
    focused: bool,
    color: RingColor,
) -> Element<'a, M> {
    let tint = match color {
        RingColor::Accent => look.colors.primary,
        RingColor::Cold => look.colors.cold,
    };
    Element::new(Ring {
        content: content.into(),
        color: focused.then_some(tint),
        radius: radius.into(),
        width: look.scale.thin(),
        scale: look.scale,
    })
}

/// The rectangle whose border is the ring around a control with `bounds`.
pub fn ring_bounds(bounds: Rectangle) -> Rectangle {
    bounds.expand(f32::from(tokens::FOCUS_OFFSET))
}

struct Ring<'a, M> {
    content: Element<'a, M>,
    /// Some while focused.
    color: Option<Color>,
    radius: Corner,
    /// The ring's stroke in logical pixels, whole device pixels at the scale.
    width: f32,
    /// The control's box is rounded up to it, so the ring around it lands on device pixels too.
    scale: Scale,
}

impl<M> Widget<M, Theme, Renderer> for Ring<'_, M> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let node = snapped(
            self.scale,
            &self
                .content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits),
        );
        layout::Node::with_children(node.size(), vec![node])
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        if let Some(content) = layout.children().next() {
            self.content
                .as_widget_mut()
                .operate(&mut tree.children[0], content, renderer, operation);
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
        let Some(color) = self.color else {
            return;
        };
        renderer.fill_quad(
            renderer::Quad {
                bounds: ring_bounds(layout.bounds()),
                border: Border {
                    color,
                    width: self.width,
                    radius: ring_radius(self.radius).into(),
                },
                ..renderer::Quad::default()
            },
            Background::Color(Color::TRANSPARENT),
        );
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
