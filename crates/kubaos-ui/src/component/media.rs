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
use iced::Color;
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

use crate::color;
use crate::look::Corner;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::scale::Scale;
use crate::tokens;
pub use crate::tokens::Aspect;
use crate::tokens::Opacity;

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
    media_under(look, picture, aspect, corner, [])
}

/// [`media`] with `over` laid over the picture, each filling the box: a name on a scrim, a chip
/// in a corner. They are the box's own, so they never move it either.
pub fn media_under<'a, M: 'a>(
    look: &Look,
    picture: Option<image::Handle>,
    aspect: Aspect,
    corner: impl Into<Corner>,
    over: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    media_drawn(look, picture, aspect, corner, over, None)
}

/// [`media_under`] for a control that can't be used now: the picture at the disabled opacity, and
/// the placeholder the disabled container.
pub(crate) fn media_disabled<'a, M: 'a>(
    look: &Look,
    picture: Option<image::Handle>,
    aspect: Aspect,
    corner: impl Into<Corner>,
    over: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    let c = look.colors;
    let dimmed = Dimmed {
        picture: Opacity::DisabledContent.value(),
        placeholder: color::alpha(c.on_surface, Opacity::DisabledContainer.value()),
    };
    media_drawn(look, picture, aspect, corner, over, Some(dimmed))
}

/// How a disabled box draws its picture and its placeholder.
#[derive(Debug, Clone, Copy)]
struct Dimmed {
    picture: f32,
    placeholder: Color,
}

fn media_drawn<'a, M: 'a>(
    look: &Look,
    picture: Option<image::Handle>,
    aspect: Aspect,
    corner: impl Into<Corner>,
    over: impl IntoIterator<Item = Element<'a, M>>,
    dimmed: Option<Dimmed>,
) -> Element<'a, M> {
    let corner = radius(corner);
    let placeholder = dimmed.map_or(look.colors.surface_container_high, |d| d.placeholder);
    let inner: Element<'a, M> = if let Some(handle) = picture {
        widget::image(handle)
            .opacity(dimmed.map_or(1.0, |d| d.picture))
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
    let layers = widget::Stack::with_children(std::iter::once(inner).chain(over))
        .width(Length::Fill)
        .height(Length::Fill);
    let view = Element::new(Framed {
        content: layers.into(),
        aspect,
        scale: look.scale,
    });
    probe::component("Media", ["", "", ""], Fixed::NONE, view)
}

/// The height of a box `width` wide at `aspect`, down to the 4 px grid, so it lands on whole device
/// pixels at every scale. Boxes side by side in a grid, whose columns differ by a device pixel at
/// most, then come out one height, and what sits on their foot lines up across the row.
pub fn height(width: f32, aspect: Aspect, _scale: Scale) -> f32 {
    let [w, h] = aspect.ratio();
    let grid = f32::from(tokens::GRID);
    (width * f32::from(h) / f32::from(w) / grid).floor() * grid
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
