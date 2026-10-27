//! Keycaps and key hints: a key drawn as a key, and a line of keys with what each does.
//!
//! A keycap gives the key its own fill and edge, so a hint is never text on text: the eye finds
//! the keys first and reads what they do after.
//!
//! # Owns
//!
//! - [`keycap`], one key, and [`key_hints`], keys and what they do side by side; [`keys`], a
//!   combination split into its keys.
//!
//! # Not here
//!
//! - The bubble naming an icon button: [`crate::component::hint`].

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Event;
use iced::Length;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Size as IcedSize;
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

use crate::layout::Gap;
use crate::layout::Inset;
use crate::layout::row;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::scale::Scale;
use crate::text::text;
use crate::tokens::Radius;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// `key` on a keycap, 20 tall: the `keycap` type on the highest container, a hairline edge with a
/// heavier bottom, the small radius, 4 either side of the key and never narrower than tall, so a
/// one-letter key is square. It sits inline with body and caption text without making the line
/// taller.
pub fn keycap<'a, M: 'a>(look: &Look, key: &'a str) -> Element<'a, M> {
    let c = look.colors;
    let label = row(look, Gap::NONE)
        .padding(Inset::x(Space::S4))
        .push(text(key, Type::Keycap, c.on_surface_variant));
    let face = Element::new(Face {
        content: label.into(),
        scale: look.scale,
        fill: c.surface_container_highest,
        edge: c.outline_variant,
    });
    probe::component("Keycap", ["", "", ""], Fixed::HEIGHT, face)
}

/// The symbols that are a key each even when written together: the modifiers' and the arrows'
/// glyphs, Return, Backspace and Delete. `⌃K` is two keys, `↑↓` two.
const GLYPHS: [char; 11] = ['⌃', '⇧', '⌥', '⌘', '↑', '↓', '←', '→', '↵', '⌫', '⌦'];

/// The keys of a combination, one keycap each: `Ctrl+K` is `Ctrl` and `K`, `⌃K` is `⌃` and `K`,
/// `← →` is `←` and `→`; a `+` alone is the plus key.
pub fn keys(combination: &str) -> Vec<&str> {
    if combination == "+" {
        return vec![combination];
    }
    let mut found = Vec::new();
    for part in combination.split(|c: char| c == '+' || c.is_whitespace()) {
        let mut word = 0;
        for (at, c) in part.char_indices() {
            if GLYPHS.contains(&c) {
                if word < at {
                    found.push(&part[word..at]);
                }
                found.push(&part[at..at + c.len_utf8()]);
                word = at + c.len_utf8();
            }
        }
        if word < part.len() {
            found.push(&part[word..]);
        }
    }
    found
}

/// Key hints side by side, 16 apart: each combination's keys as keycaps 4 apart, then what it does
/// in a `caption` line 8 after them, when it says. A combination is written `Ctrl+K`, and
/// each of its keys is a keycap of its own ([`keys`]).
pub fn key_hints<'a, M: 'a>(look: &Look, hints: &[(&'a str, &'a str)]) -> Element<'a, M> {
    let c = look.colors;
    let line = row(look, Space::S16)
        .center()
        .extend(hints.iter().map(|&(combination, what)| {
            let caps = row(look, Space::S4)
                .center()
                .extend(keys(combination).into_iter().map(|key| keycap(look, key)));
            let hint = row(look, Space::S8).center().push(caps);
            if what.is_empty() {
                hint.into()
            } else {
                hint.push(text(what, Type::Caption, c.on_surface_variant)).into()
            }
        }));
    probe::component("KeyHints", ["", "", ""], Fixed::NONE, line)
}

/// A keycap's face around its label: at least as wide as tall, the label centred across it on a
/// whole device pixel, drawn over the edge and the fill.
struct Face<'a, M> {
    content: Element<'a, M>,
    scale: Scale,
    fill: Color,
    edge: Color,
}

impl<M> Widget<M, Theme, Renderer> for Face<'_, M> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> IcedSize<Length> {
        IcedSize::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, _limits: &layout::Limits) -> layout::Node {
        let side = f32::from(Size::Keycap.px());
        // A key is one word that never wraps: it takes the width it needs, whatever the room.
        let unbounded = layout::Limits::new(IcedSize::ZERO, IcedSize::new(f32::INFINITY, side));
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &unbounded);
        let inner = node.size();
        let width = self.scale.snap_up(inner.width).max(side);
        let x = self.scale.snap((width - inner.width) / 2.0);
        layout::Node::with_children(IcedSize::new(width, side), vec![node.move_to(Point::new(x, 0.0))])
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
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
        use iced::advanced::Renderer as _;

        let bounds = layout.bounds();
        let hairline = self.scale.hairline();
        let corner = radius(Radius::Small);
        // The edge fills the whole key; the face stands on it one hairline short of the bottom,
        // so the edge shows a hairline at the top and sides and two at the bottom.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border::default().rounded(corner),
                snap: true,
                ..renderer::Quad::default()
            },
            Background::Color(self.edge),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    height: bounds.height - hairline,
                    ..bounds
                },
                border: Border {
                    color: self.edge,
                    width: hairline,
                    radius: corner.into(),
                },
                snap: true,
                ..renderer::Quad::default()
            },
            Background::Color(self.fill),
        );
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
