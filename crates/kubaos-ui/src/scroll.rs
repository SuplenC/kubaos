//! Scrolling: content taller than its room, with a thin thumb that only says where you are.
//!
//! A scroll clips what it scrolls, a focused control's glow included, only at an edge you can see.
//! A page's scroll has no edge of its own, so it reaches past its box into the room around it, as
//! far as a glow reaches ([`tokens::FOCUS_FAR`]) or up to the part of the page beside it, with its
//! content that far in: the content lines up with what is around the scroll, a resting control at
//! its edge keeps its whole glow, and the panel's or window's own edge cuts what passes it.
//!
//! # Owns
//!
//! - [`scroll`], the thumb's look, and the room a page's scroll reaches into.
//!
//! # Not here
//!
//! - A list's scroll, inside its well, whose outline is its edge:
//!   [`crate::component::list::scrolling_list`].

use std::fmt;
use std::slice;

use iced::Background;
use iced::Border;
use iced::Element;
use iced::Event;
use iced::Length;
use iced::Padding;
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
use crate::probe::Reach;
use crate::probe::Tag;
use crate::probe::probe;
use crate::tokens;
use crate::tokens::Radius;
use crate::tokens::Size as SizeToken;
use crate::tokens::Space;

/// The room content that runs to a page's scroll's right edge leaves there for the thumb.
const GUTTER: Space = Space::S16;

/// `content` scrolled when it is taller than its room: a page's scroll. Its thumb is
/// [`SizeToken::Scrollbar`] wide, on its box's right edge, floating over the content so it never
/// takes width; content that runs to the edge leaves [`GUTTER`] on its right for it.
///
/// It reaches past its box as far as a glow does on every side, the gutter less on the right,
/// where the content's own gutter makes up the rest: the thumb sits [`Space::S4`] in from the edge
/// it reaches, and a rail any wider would take the pointer from the content.
///
/// [`Scroll::room_above`] and [`Scroll::room_below`] set how far it reaches toward what the page
/// draws above and below it.
pub fn scroll<'a, M: 'a>(look: &Look, content: impl Into<Element<'a, M>>) -> Scroll<'a, M> {
    let glow = f32::from(tokens::FOCUS_FAR);
    Scroll {
        look: *look,
        content: content.into(),
        reach: Some(Padding {
            top: glow,
            right: (glow - f32::from(GUTTER.px())).max(0.0),
            bottom: glow,
            left: glow,
        }),
        id: None,
        height: None,
        anchor_bottom: false,
        moved: None,
    }
}

/// `content` scrolled in a box no larger than itself: a list's rows, inside their well, whose
/// outline is the edge a glow is cut at.
pub(crate) fn flush<'a, M: 'a>(look: &Look, content: impl Into<Element<'a, M>>) -> Scroll<'a, M> {
    Scroll {
        look: *look,
        content: content.into(),
        reach: None,
        id: None,
        height: None,
        anchor_bottom: false,
        moved: None,
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
    look: Look,
    content: Element<'a, M>,
    /// How far past its own box it reaches on each side; `None` for a scroll held to its box.
    reach: Option<Padding>,
    id: Option<widget::Id>,
    height: Option<Extent>,
    anchor_bottom: bool,
    moved: Option<Box<dyn Fn(scrollable::Viewport) -> M + 'a>>,
}

impl<M> fmt::Debug for Scroll<'_, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scroll")
            .field("reach", &self.reach)
            .field("height", &self.height)
            .field("anchor_bottom", &self.anchor_bottom)
            .finish_non_exhaustive()
    }
}

impl<'a, M: 'a> Scroll<'a, M> {
    /// The id the app scrolls it by.
    #[must_use]
    pub fn id(self, id: impl Into<widget::Id>) -> Self {
        Scroll {
            id: Some(id.into()),
            ..self
        }
    }

    #[must_use]
    pub fn height(self, height: Extent) -> Self {
        Scroll {
            height: Some(height),
            ..self
        }
    }

    /// Held to its end: content that grows keeps its newest line in view.
    #[must_use]
    pub fn anchor_bottom(self) -> Self {
        Scroll {
            anchor_bottom: true,
            ..self
        }
    }

    /// `on_scroll` with where it is scrolled to, each time it moves.
    #[must_use]
    pub fn on_scroll(self, on_scroll: impl Fn(scrollable::Viewport) -> M + 'a) -> Self {
        Scroll {
            moved: Some(Box::new(on_scroll)),
            ..self
        }
    }

    /// The room between it and what the page draws above it, all of which it reaches into:
    /// content scrolled up shows in that room and stops at that part, never over it.
    #[must_use]
    pub fn room_above(self, room: Space) -> Self {
        self.reaching(|reach| reach.top = f32::from(room.px()))
    }

    /// The room between it and what the page draws below it, all of which it reaches into.
    #[must_use]
    pub fn room_below(self, room: Space) -> Self {
        self.reaching(|reach| reach.bottom = f32::from(room.px()))
    }

    fn reaching(mut self, set: impl FnOnce(&mut Padding)) -> Self {
        if let Some(reach) = self.reach.as_mut() {
            set(reach);
        }
        self
    }
}

impl<'a, M: 'a> From<Scroll<'a, M>> for Element<'a, M> {
    fn from(scroll: Scroll<'a, M>) -> Self {
        let content = match scroll.reach {
            Some(reach) => widget::container(scroll.content).padding(reach).into(),
            None => scroll.content,
        };
        let mut inner = styled(&scroll.look, content);
        if let Some(id) = scroll.id {
            inner = inner.id(id);
        }
        // A set height is the box's, so the room it reaches into is added to it.
        if let Some(height) = scroll.height {
            inner = inner.height(match (height.length(), scroll.reach) {
                (Length::Fixed(box_height), Some(reach)) => Length::Fixed(box_height + reach.y()),
                (length, _) => length,
            });
        }
        if scroll.anchor_bottom {
            inner = inner.anchor_bottom();
        }
        if let Some(moved) = scroll.moved {
            inner = inner.on_scroll(moved);
        }
        match scroll.reach {
            Some(reach) => {
                let tag = Tag::Scroll(Reach {
                    above: reach.top,
                    below: reach.bottom,
                    left: reach.left,
                });
                probe(
                    tag,
                    Element::new(Bleed {
                        content: inner.into(),
                        by: reach,
                    }),
                )
            }
            None => inner.into(),
        }
    }
}

/// `content` laid out `by` larger than this box on each side, and placed that far outside it: the
/// box is what lines up with its neighbours, and the content may draw in the room around it.
struct Bleed<'a, M> {
    content: Element<'a, M>,
    by: Padding,
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
        let both = Size::new(self.by.x(), self.by.y());
        let wider = layout::Limits::with_compression(limits.min() + both, limits.max() + both, limits.compression());
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &wider)
            .move_to(Point::new(-self.by.left, -self.by.top));
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
