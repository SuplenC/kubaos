//! Pictures that never move what is around them: a box of a fixed aspect ratio, reserved before the
//! picture has loaded, with a placeholder in it until then.
//!
//! A picture sized by its own pixels is nothing tall until it decodes, then pushes everything after
//! it down: a grid of thumbnails jumps as each arrives. [`media`] takes its height from its width
//! and an [`Aspect`] token instead, so the box is the same before and after the load.
//!
//! # Owns
//!
//! - [`media`], a picture or its placeholder in a box of an aspect ratio.
//!
//! # Not here
//!
//! - Decoding and caching pictures: the app's, which hands over a handle once one is ready.

use iced::Background;
use iced::ContentFit;
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
use iced::widget;
use iced::widget::container;
use iced::widget::image;

use crate::look::Corner;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::scale::Scale;
pub use crate::tokens::Aspect;

/// `picture` filling a box as wide as the room (a thumbnail's width where the room has no end) and
/// as tall as `aspect` makes it, cut to cover it, with `corner`'s radius; without a picture yet,
/// the box holds the placeholder, the high container. Its height lands on a whole device pixel, so
/// what follows it does too.
pub fn media<'a, M: 'a>(
    look: &Look,
    picture: Option<image::Handle>,
    aspect: Aspect,
    corner: impl Into<Corner>,
) -> Element<'a, M> {
    let corner = radius(corner);
    let placeholder = look.colors.surface_container_high;
    let inner: Element<'a, M> = if let Some(handle) = picture {
        widget::image(handle)
            .content_fit(ContentFit::Cover)
            .width(Length::Fill)
            .height(Length::Fill)
            .border_radius(corner)
            .into()
    } else {
        widget::container(widget::Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(Background::Color(placeholder)),
                border: iced::Border::default().rounded(corner),
                ..container::Style::default()
            })
            .into()
    };
    let view = Element::new(Framed {
        content: inner,
        aspect,
        scale: look.scale,
    });
    probe::component("Media", ["", "", ""], Fixed::NONE, view)
}

/// The height of a box `width` wide at `aspect`, on a whole device pixel at `scale`.
pub fn height(width: f32, aspect: Aspect, scale: Scale) -> f32 {
    let [w, h] = aspect.ratio();
    scale.snap(width * f32::from(h) / f32::from(w))
}

/// A box as wide as the room and as tall as its aspect makes it, its content laid out to fill it.
struct Framed<'a, M> {
    content: Element<'a, M>,
    aspect: Aspect,
    scale: Scale,
}

impl<M> Widget<M, Theme, Renderer> for Framed<'_, M> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        // In a parent with no end across (a shrinking row, a sideways scroll) the room is
        // infinite, and so would the height be: the box then takes a thumbnail's width.
        let room = limits.width(Length::Fill).max().width;
        let width = if room.is_finite() {
            room
        } else {
            f32::from(crate::tokens::Size::ThumbnailWidth.px())
        };
        let size = Size::new(width, height(width, self.aspect, self.scale));
        let node =
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &layout::Limits::new(size, size));
        layout::Node::with_children(size, vec![node])
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
