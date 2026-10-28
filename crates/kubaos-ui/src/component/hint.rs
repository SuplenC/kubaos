//! Hints: a control's name in a small bubble under it, while the pointer is over it or the keyboard
//! is on it. An icon-only button has no other words, so its hint is its accessible label.
//!
//! # Owns
//!
//! - [`hint`], and the floating that a tooltip's card shares ([`floating`]).
//!
//! # Not here
//!
//! - The tooltip's card: [`crate::component::tooltip`].

use iced::Background;
use iced::Border;
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
use iced::advanced::widget::tree;
use iced::widget;
use iced::widget::container;

use crate::layout::Inset;
use crate::look::Look;
use crate::look::radius;
use crate::scale::Scale;
use crate::text::text;
use crate::tokens::Radius;
use crate::tokens::Space;
use crate::tokens::Type;

/// `content` with `label` in a bubble under it while hovered, or while `shown` (the keyboard is on
/// it). The bubble is the toast's inverse colours, 8 below the control.
pub fn hint<'a, M: 'a>(look: &Look, content: impl Into<Element<'a, M>>, label: &'a str, shown: bool) -> Element<'a, M> {
    let c = look.colors;
    let bubble = widget::container(text(label, Type::Label, c.surface))
        .padding(Inset::xy(Space::S8, Space::S4).padding())
        .style(move |_theme| container::Style {
            background: Some(Background::Color(c.on_surface)),
            border: Border::default().rounded(radius(Radius::Small)),
            ..container::Style::default()
        });
    floating(look, content, bubble, shown, Side::Below)
}

/// Which side of its control a floating bubble takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Side {
    /// Under it and centred on it: a hint, which names the control.
    Below,
    /// Over it, starting where it starts: a tooltip, whose card is taller than the room a
    /// control's foot usually leaves, and wider than a chip at a toolbar's start, so centred it
    /// would hang past the panel. With no room above it, under it instead, still starting where
    /// it starts: a control in a header at the window's top.
    Above,
}

/// `content` with `bubble` floating 8 from it on `side`, kept inside the window, while hovered or
/// while `shown`.
pub(crate) fn floating<'a, M: 'a>(
    look: &Look,
    content: impl Into<Element<'a, M>>,
    bubble: impl Into<Element<'a, M>>,
    shown: bool,
    side: Side,
) -> Element<'a, M> {
    Element::new(Hint {
        content: content.into(),
        bubble: bubble.into(),
        shown,
        side,
        scale: look.scale,
    })
}

struct Hint<'a, M> {
    content: Element<'a, M>,
    bubble: Element<'a, M>,
    shown: bool,
    side: Side,
    scale: Scale,
}

/// Whether the pointer is over the control.
#[derive(Default)]
struct Hovered(bool);

impl<M> Widget<M, Theme, Renderer> for Hint<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Hovered>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Hovered::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content), Tree::new(&self.bubble)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.content, &self.bubble].map(Element::as_widget));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        let over = cursor.is_over(layout.bounds());
        let hovered = tree.state.downcast_mut::<Hovered>();
        if hovered.0 != over {
            hovered.0 = over;
            shell.invalidate_layout();
            shell.request_redraw();
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
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
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
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
            .draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        let open = self.shown || tree.state.downcast_ref::<Hovered>().0;
        let (content_tree, rest) = tree.children.split_at_mut(1);
        let inner = self
            .content
            .as_widget_mut()
            .overlay(&mut content_tree[0], layout, renderer, viewport, translation);
        let bubble = open.then(|| {
            overlay::Element::new(Box::new(Bubble {
                content: &mut self.bubble,
                tree: &mut rest[0],
                anchor: layout.bounds() + translation,
                side: self.side,
                scale: self.scale,
            }))
        });
        let all: Vec<_> = inner.into_iter().chain(bubble).collect();
        (!all.is_empty()).then(|| overlay::Group::with_children(all).overlay())
    }
}

struct Bubble<'a, 'b, M> {
    content: &'b mut Element<'a, M>,
    tree: &'b mut Tree,
    /// The control's bounds on the window.
    anchor: Rectangle,
    side: Side,
    scale: Scale,
}

impl<M> overlay::Overlay<M, Theme, Renderer> for Bubble<'_, '_, M> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let node = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &layout::Limits::new(Size::ZERO, bounds));
        let size = node.size();
        let size = Size::new(self.scale.snap_up(size.width), self.scale.snap_up(size.height));
        // Beside the control, on a whole device pixel and kept inside the window.
        let apart = f32::from(Space::S8.px());
        let below = self.anchor.y + self.anchor.height + apart;
        let (x, y) = match self.side {
            Side::Below => (self.anchor.center_x() - size.width / 2.0, below),
            Side::Above => {
                let above = self.anchor.y - apart - size.height;
                (self.anchor.x, if above < 0.0 { below } else { above })
            }
        };
        let x = self.scale.snap(x).clamp(0.0, (bounds.width - size.width).max(0.0));
        let y = self.scale.snap(y).max(0.0);
        layout::Node::with_children(size, vec![node]).move_to(Point::new(x, y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        if let Some(bubble) = layout.children().next() {
            self.content.as_widget().draw(
                self.tree,
                renderer,
                theme,
                style,
                bubble,
                cursor,
                &Rectangle::with_size(Size::INFINITE),
            );
        }
    }
}
