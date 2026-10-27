//! The state layer of everything pressable: the content colour laid over it while the pointer is on
//! it or presses it, fading in and out over [`Duration::Micro`].
//!
//! The layer is drawn over the pressable, content included. The layer is the content's own colour,
//! so over the label and icons it changes nothing, and over the fill it is the fill mixed toward
//! the content: what a layer under the content would show, without a hook into iced's button.

use std::slice;
use std::time::Instant;

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
use iced::advanced::widget::tree;

use crate::color;
use crate::look::Corner;
use crate::look::radius;
use crate::motion::Tween;
use crate::tokens::Duration;
use crate::tokens::Easing;
use crate::tokens::Opacity;

/// A state shown whatever the pointer does: the specimen's and screenshot tests' view of hover and
/// press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pin {
    Hovered,
    Pressed,
}

/// `content`, a pressable with corners `corner`, with the state layer of `colour` over it. A
/// disabled one has none.
pub(crate) fn layered<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    corner: impl Into<Corner>,
    colour: Color,
    enabled: bool,
    pin: Option<Pin>,
) -> Element<'a, M> {
    Element::new(Layer {
        content: content.into(),
        corner: corner.into(),
        colour,
        enabled,
        pin,
    })
}

struct Layer<'a, M> {
    content: Element<'a, M>,
    corner: Corner,
    colour: Color,
    enabled: bool,
    pin: Option<Pin>,
}

/// Where the pointer is, and how much of the layer shows.
struct State {
    hovered: bool,
    pressed: bool,
    shown: Tween,
}

impl<M> Layer<'_, M> {
    /// The layer's opacity for what the pointer does.
    fn target(&self, state: &State) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        let (hovered, pressed) = match self.pin {
            Some(Pin::Hovered) => (true, false),
            Some(Pin::Pressed) => (true, true),
            None => (state.hovered, state.pressed),
        };
        if pressed {
            Opacity::Press.value()
        } else if hovered {
            Opacity::Hover.value()
        } else {
            0.0
        }
    }
}

impl<M> Widget<M, Theme, Renderer> for Layer<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        let mut state = State {
            hovered: false,
            pressed: false,
            shown: Tween::new(0.0, Duration::Micro, Easing::Standard),
        };
        // A pinned state shows at once: nothing moved to it.
        state.shown = Tween::new(self.target(&state), Duration::Micro, Easing::Standard);
        tree::State::new(state)
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();
        let target = self.target(state);
        state.shown.to(target, Instant::now());
        tree.diff_children(slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
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
        let now = Instant::now();
        let over = cursor.is_over(layout.bounds());
        let state = tree.state.downcast_mut::<State>();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if over => state.pressed = true,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => state.pressed = false,
            Event::Window(iced::window::Event::RedrawRequested(at)) if state.shown.moving(*at) => {
                shell.request_redraw();
            }
            _ => {}
        }
        state.hovered = over;
        let target = self.target(state);
        if (target - state.shown.target()).abs() > f32::EPSILON {
            state.shown.to(target, now);
            shell.request_redraw();
        }
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
        let shown = tree.state.downcast_ref::<State>().shown.value(Instant::now());
        if shown <= 0.0 {
            return;
        }
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: Border::default().rounded(radius(self.corner)),
                snap: true,
                ..renderer::Quad::default()
            },
            Background::Color(color::alpha(self.colour, shown)),
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
