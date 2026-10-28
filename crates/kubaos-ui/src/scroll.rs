//! Scrolling: content taller than its room, with a thin thumb that only says where you are.
//!
//! A scroll clips what it scrolls, and a focused control's glow reaches only as far as the room it
//! has before the clip. A page's scroll therefore scrolls a box [`tokens::FOCUS_OFFSET`] larger
//! than itself on every side, with its content that band in: the content lines up with what is
//! around the scroll, and a control at its edge still glows that far.
//!
//! # Owns
//!
//! - [`scroll`], the thumb's look, and the band a page's scroll lends its content.
//!
//! # Not here
//!
//! - A list's scroll, inside its well: [`crate::component::list::scrolling_list`].

use std::fmt;
use std::slice;

use iced::Background;
use iced::Border;
use iced::Element;
use iced::Event;
use iced::Length;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Shadow;
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
use iced::widget::scrollable;

use crate::layout::Extent;
use crate::look::Look;
use crate::look::radius;
use crate::tokens;
use crate::tokens::Radius;
use crate::tokens::Size as SizeToken;
use crate::tokens::Space;

/// `content` scrolled when it is taller than its room: a page's scroll. Its thumb is
/// [`SizeToken::Scrollbar`] wide, [`Space::S4`] in from its edge, floating over the content so it
/// never takes width; content that runs to the edge leaves room for it on its right.
pub fn scroll<'a, M: 'a>(look: &Look, content: impl Into<Element<'a, M>>) -> Scroll<'a, M> {
    let band = f32::from(tokens::FOCUS_OFFSET);
    Scroll {
        inner: styled(look, widget::container(content).padding(band)),
        bleed: band,
    }
}

/// `content` scrolled in a box no larger than itself: a list's rows, inside their well, which
/// shows where the keyboard is without a ring.
pub(crate) fn flush<'a, M: 'a>(look: &Look, content: impl Into<Element<'a, M>>) -> Scroll<'a, M> {
    Scroll {
        inner: styled(look, content),
        bleed: 0.0,
    }
}

/// iced's scrollable with the thumb's look.
pub(crate) fn styled<'a, M: 'a>(look: &Look, content: impl Into<Element<'a, M>>) -> scrollable::Scrollable<'a, M> {
    let c = look.colors;
    let thumb = f32::from(SizeToken::Scrollbar.px());
    let bar = scrollable::Scrollbar::new()
        .width(thumb)
        .scroller_width(thumb)
        .margin(f32::from(Space::S4.px()));
    widget::scrollable(content)
        .direction(scrollable::Direction::Vertical(bar))
        .width(Length::Fill)
        .style(move |_theme, status| {
            let hot = matches!(
                status,
                scrollable::Status::Hovered {
                    is_vertical_scrollbar_hovered: true,
                    ..
                } | scrollable::Status::Dragged {
                    is_vertical_scrollbar_dragged: true,
                    ..
                }
            );
            let rail = scrollable::Rail {
                background: None,
                border: Border::default(),
                scroller: scrollable::Scroller {
                    background: Background::Color(if hot { c.outline } else { c.outline_variant }),
                    border: Border::default().rounded(radius(Radius::Full)),
                },
            };
            scrollable::Style {
                container: container::Style::default(),
                vertical_rail: rail,
                horizontal_rail: rail,
                gap: None,
                auto_scroll: scrollable::AutoScroll {
                    background: Background::Color(c.surface_container_high),
                    border: Border::default().rounded(radius(Radius::Full)),
                    shadow: Shadow::default(),
                    icon: c.on_surface,
                },
            }
        })
}

/// See [`scroll`].
pub struct Scroll<'a, M> {
    inner: scrollable::Scrollable<'a, M>,
    /// How far past its own box on every side it scrolls.
    bleed: f32,
}

impl<M> fmt::Debug for Scroll<'_, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scroll")
            .field("bleed", &self.bleed)
            .finish_non_exhaustive()
    }
}

impl<'a, M: 'a> Scroll<'a, M> {
    /// The id the app scrolls it by.
    #[must_use]
    pub fn id(self, id: impl Into<widget::Id>) -> Self {
        Scroll {
            inner: self.inner.id(id),
            ..self
        }
    }

    #[must_use]
    pub fn height(self, height: Extent) -> Self {
        Scroll {
            inner: self.inner.height(height.length()),
            ..self
        }
    }

    /// Held to its end: content that grows keeps its newest line in view.
    #[must_use]
    pub fn anchor_bottom(self) -> Self {
        Scroll {
            inner: self.inner.anchor_bottom(),
            ..self
        }
    }

    /// `on_scroll` with where it is scrolled to, each time it moves.
    #[must_use]
    pub fn on_scroll(self, on_scroll: impl Fn(scrollable::Viewport) -> M + 'a) -> Self {
        Scroll {
            inner: self.inner.on_scroll(on_scroll),
            ..self
        }
    }
}

impl<'a, M: 'a> From<Scroll<'a, M>> for Element<'a, M> {
    fn from(scroll: Scroll<'a, M>) -> Self {
        if scroll.bleed <= 0.0 {
            return scroll.inner.into();
        }
        Element::new(Bleed {
            content: scroll.inner.into(),
            by: scroll.bleed,
        })
    }
}

/// `content` laid out `by` larger than this box on every side, and placed `by` outside it: the box
/// is what lines up with its neighbours, and the content may draw in the band around it.
struct Bleed<'a, M> {
    content: Element<'a, M>,
    by: f32,
}

impl<M> Widget<M, Theme, Renderer> for Bleed<'_, M> {
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
        let both = Size::new(2.0 * self.by, 2.0 * self.by);
        let wider = layout::Limits::with_compression(limits.min() + both, limits.max() + both, limits.compression());
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &wider)
            .move_to(Point::new(-self.by, -self.by));
        let size = node.size();
        layout::Node::with_children(
            Size::new((size.width - both.width).max(0.0), (size.height - both.height).max(0.0)),
            vec![node],
        )
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
