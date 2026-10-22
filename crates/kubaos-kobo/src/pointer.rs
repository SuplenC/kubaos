//! Keyboard mode: once you type, the pointer hides and stops hovering.
//!
//! In the live ISO the pointer sits wherever the last mouse event left it
//! (the middle of the screen, in a VM). A keyboard-only install would then
//! show a hover highlight on whatever row happens to be under it, all the
//! way through. This wrapper tells the UI there's no pointer while `idle`,
//! and hides it; moving or clicking the mouse brings it back.

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, mouse};
use iced::{Element, Event, Length, Rectangle, Renderer, Size, Theme, Vector, window};

pub struct Idle<'a, Message> {
    content: Element<'a, Message>,
    idle: bool,
}

/// `content`, pointerless while `idle`.
pub fn idle<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    idle: bool,
) -> Idle<'a, Message> {
    Idle {
        content: content.into(),
        idle,
    }
}

impl<Message> Idle<'_, Message> {
    fn cursor(&self, c: mouse::Cursor) -> mouse::Cursor {
        if self.idle {
            mouse::Cursor::Unavailable
        } else {
            c
        }
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Idle<'_, Message> {
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

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        // Widgets work out hover on redraw; real mouse events keep the
        // real position, so a click always lands.
        let cursor = match event {
            Event::Window(window::Event::RedrawRequested(_)) => self.cursor(cursor),
            _ => cursor,
        };
        self.content.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
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
        if self.idle {
            return mouse::Interaction::Hidden;
        }
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
        self.content.as_widget().draw(
            tree,
            renderer,
            theme,
            style,
            layout,
            self.cursor(cursor),
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

impl<'a, Message: 'a> From<Idle<'a, Message>> for Element<'a, Message> {
    fn from(i: Idle<'a, Message>) -> Self {
        Element::new(i)
    }
}
