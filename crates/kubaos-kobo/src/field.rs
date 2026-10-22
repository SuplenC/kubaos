//! The one-line text field, as its own iced widget.
//!
//! Not iced's `text_input`: the app owns focus and editing (`edit.rs`), so
//! real keys, the key tour and the tests all go through one code path. This
//! widget only draws (value or placeholder, selection, a blinking caret,
//! horizontal scroll that keeps the caret in view) and turns mouse presses
//! into caret positions (`Pointer`), measured on the same text it draws.

use std::time::{Duration, Instant};

use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse::click;
use iced::advanced::renderer::{self, Quad};
use iced::advanced::text::{self, Paragraph as _, Text, paragraph::Plain};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, mouse};
use iced::{
    Background, Border, Element, Event, Font, Length, Pixels, Point, Rectangle, Renderer, Size,
    Theme, alignment, keyboard, window,
};

use crate::edit::Caret;
use crate::theme::{self, Palette, alpha, radius};

/// What the mouse did, in chars of the value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pointer {
    Press { pos: usize, extend: bool },
    Drag(usize),
    Double(usize),
    Triple,
}

/// The caret blinks with this half-period, and stays solid while typing.
pub const BLINK: Duration = Duration::from_millis(530);
const PAD_X: f32 = 14.0;
const PAD_Y: f32 = 11.0;
const ICON: f32 = 20.0;
const GAP: f32 = 10.0;

pub struct Field<'a, Message> {
    pub shown: String,
    pub placeholder: &'a str,
    pub size: f32,
    pub font: Font,
    pub icon: Option<char>,
    pub locked: bool,
    pub error: bool,
    /// Some when focused.
    pub caret: Option<Caret>,
    /// When the caret last moved; it's solid for one half-period from here.
    pub blink_from: Instant,
    pub palette: Palette,
    pub on_pointer: Option<Box<dyn Fn(Pointer) -> Message + 'a>>,
}

struct State {
    value: Plain<<Renderer as text::Renderer>::Paragraph>,
    placeholder: Plain<<Renderer as text::Renderer>::Paragraph>,
    icon: Plain<<Renderer as text::Renderer>::Paragraph>,
    lock: Plain<<Renderer as text::Renderer>::Paragraph>,
    last_click: Option<click::Click>,
    dragging: bool,
    shift: bool,
    hovered: bool,
    now: Instant,
}

impl Default for State {
    fn default() -> Self {
        Self {
            value: Plain::default(),
            placeholder: Plain::default(),
            icon: Plain::default(),
            lock: Plain::default(),
            last_click: None,
            dragging: false,
            shift: false,
            hovered: false,
            now: Instant::now(),
        }
    }
}

impl<Message> Field<'_, Message> {
    fn line_height(&self) -> f32 {
        (self.size * 1.3).round()
    }

    fn text<'b>(&self, content: &'b str, font: Font, size: f32) -> Text<&'b str, Font> {
        Text {
            content,
            bounds: Size::new(f32::INFINITY, self.line_height()),
            size: Pixels(size),
            line_height: text::LineHeight::Absolute(Pixels(self.line_height())),
            font,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Center,
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::None,
        }
    }

    /// Where the text goes inside the field's bounds.
    fn text_bounds(&self, b: Rectangle) -> Rectangle {
        let left = PAD_X + if self.icon.is_some() { ICON + GAP } else { 0.0 };
        let right = PAD_X + if self.locked { 18.0 + GAP } else { 0.0 };
        Rectangle {
            x: b.x + left,
            y: b.y + PAD_Y,
            width: (b.width - left - right).max(0.0),
            height: self.line_height(),
        }
    }

    /// Scrolls the text left just enough to keep the caret in view.
    fn offset(&self, state: &State, tb: Rectangle) -> f32 {
        let Some(c) = self.caret else { return 0.0 };
        let x = caret_x(state, c.pos);
        (x + 3.0 - tb.width).max(0.0)
    }

    fn pos_at(&self, state: &State, tb: Rectangle, x: f32) -> usize {
        let local = x - tb.x + self.offset(state, tb);
        let n = self.shown.chars().count();
        if local <= 0.0 {
            return 0;
        }
        if local >= state.value.min_width() {
            return n;
        }
        state
            .value
            .raw()
            .hit_test(Point::new(local, tb.height / 2.0))
            .map(|h| {
                let byte = h.cursor().min(self.shown.len());
                self.shown
                    .get(..byte)
                    .map_or(n, |s| s.chars().count())
                    .min(n)
            })
            .unwrap_or(n)
    }
}

fn caret_x(state: &State, pos: usize) -> f32 {
    if pos == 0 {
        return 0.0;
    }
    state
        .value
        .raw()
        .grapheme_position(0, pos)
        .map_or_else(|| state.value.min_width(), |p| p.x)
}

impl<Message> Widget<Message, Theme, Renderer> for Field<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        state
            .value
            .update(self.text(&self.shown, self.font, self.size));
        state
            .placeholder
            .update(self.text(self.placeholder, self.font, self.size));
        let mut buf = [0u8; 4];
        if let Some(i) = self.icon {
            state
                .icon
                .update(self.text(i.encode_utf8(&mut buf), theme::ICONS, ICON));
        }
        let lock = theme::Icon::Lock.glyph();
        state
            .lock
            .update(self.text(lock.encode_utf8(&mut buf), theme::ICONS, 18.0));
        let h = self.line_height() + 2.0 * PAD_Y;
        layout::Node::new(limits.resolve(Length::Fill, h, Size::new(0.0, h)))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let b = layout.bounds();
        let tb = self.text_bounds(b);
        match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(m)) => state.shift = m.shift(),
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let (Some(on), Some(p)) = (&self.on_pointer, cursor.position_over(b)) else {
                    return;
                };
                let click = click::Click::new(p, mouse::Button::Left, state.last_click);
                state.last_click = Some(click);
                let pos = self.pos_at(state, tb, p.x);
                let msg = match click.kind() {
                    click::Kind::Single => Pointer::Press {
                        pos,
                        extend: state.shift,
                    },
                    click::Kind::Double => Pointer::Double(pos),
                    click::Kind::Triple => Pointer::Triple,
                };
                state.dragging = matches!(msg, Pointer::Press { .. });
                shell.publish(on(msg));
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let hovered = b.contains(*position);
                if hovered != state.hovered {
                    state.hovered = hovered;
                    shell.request_redraw();
                }
                if state.dragging
                    && let Some(on) = &self.on_pointer
                {
                    let pos = self.pos_at(state, tb, position.x);
                    if self.caret.is_none_or(|c| c.pos != pos) {
                        shell.publish(on(Pointer::Drag(pos)));
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.dragging = false;
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                state.now = *now;
                if self.caret.is_some_and(|c| !c.has_selection()) {
                    let since = now.saturating_duration_since(self.blink_from).as_millis();
                    let half = BLINK.as_millis();
                    let wait = half - since % half;
                    shell.request_redraw_at(*now + Duration::from_millis(wait as u64));
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.on_pointer.is_some() && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Text
        } else {
            mouse::Interaction::None
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        use iced::advanced::text::Renderer as _;
        let state = tree.state.downcast_ref::<State>();
        let p = self.palette;
        let b = layout.bounds();
        let focused = self.caret.is_some();
        let hovered = !self.locked && cursor.is_over(b);
        let (border, width, bg) = if self.error && !focused {
            (alpha(p.error, 0.8), 1.0, p.surface_container_high)
        } else if focused {
            (
                if self.error { p.error } else { p.primary },
                2.0,
                p.surface_container_highest,
            )
        } else if self.locked {
            (
                alpha(p.outline_variant, 0.5),
                1.0,
                alpha(p.surface_container_high, 0.5),
            )
        } else if hovered {
            (p.outline, 1.0, p.surface_container_highest)
        } else {
            (p.outline_variant, 1.0, p.surface_container_high)
        };
        renderer.fill_quad(
            Quad {
                bounds: b,
                border: Border {
                    color: border,
                    width,
                    radius: radius::MD.into(),
                },
                ..Quad::default()
            },
            Background::Color(alpha(bg, 0.9)),
        );
        let tb = self.text_bounds(b);
        let middle = |h: f32| tb.y + (tb.height - h) / 2.0;
        if self.icon.is_some() {
            let ib = state.icon.min_bounds();
            renderer.fill_paragraph(
                state.icon.raw(),
                Point::new(b.x + PAD_X + (ICON - ib.width) / 2.0, middle(ib.height)),
                p.on_surface_variant,
                *viewport,
            );
        }
        if self.locked {
            let lb = state.lock.min_bounds();
            renderer.fill_paragraph(
                state.lock.raw(),
                Point::new(b.x + b.width - PAD_X - lb.width, middle(lb.height)),
                p.outline,
                *viewport,
            );
        }
        let offset = self.offset(state, tb);
        let clip = Rectangle {
            x: tb.x - 2.0,
            width: tb.width + 4.0,
            ..tb
        };
        renderer.with_layer(clip, |r| {
            if self.shown.is_empty() {
                let pb = state.placeholder.min_bounds();
                r.fill_paragraph(
                    state.placeholder.raw(),
                    Point::new(tb.x, middle(pb.height)),
                    alpha(p.on_surface_variant, 0.55),
                    clip,
                );
            } else {
                if let Some(c) = self.caret.filter(|c| c.has_selection()) {
                    let (a, z) = c.range();
                    let (xa, xz) = (caret_x(state, a), caret_x(state, z));
                    r.fill_quad(
                        Quad {
                            bounds: Rectangle {
                                x: tb.x + xa - offset,
                                y: tb.y - 1.0,
                                width: xz - xa,
                                height: tb.height + 2.0,
                            },
                            border: Border::default().rounded(3.0),
                            ..Quad::default()
                        },
                        Background::Color(alpha(p.primary, 0.32)),
                    );
                }
                let vb = state.value.min_bounds();
                r.fill_paragraph(
                    state.value.raw(),
                    Point::new(tb.x - offset, middle(vb.height)),
                    if self.locked {
                        p.on_surface_variant
                    } else {
                        p.on_surface
                    },
                    clip,
                );
            }
            if let Some(c) = self.caret.filter(|c| !c.has_selection()) {
                let since = state.now.saturating_duration_since(self.blink_from);
                if (since.as_millis() / BLINK.as_millis()).is_multiple_of(2) {
                    let x = (tb.x + caret_x(state, c.pos) - offset).round();
                    r.fill_quad(
                        Quad {
                            bounds: Rectangle {
                                x: x - 1.0,
                                y: tb.y,
                                width: 2.0,
                                height: tb.height,
                            },
                            border: Border::default().rounded(1.0),
                            ..Quad::default()
                        },
                        Background::Color(p.primary),
                    );
                }
            }
        });
    }
}

impl<'a, Message: 'a> From<Field<'a, Message>> for Element<'a, Message> {
    fn from(f: Field<'a, Message>) -> Self {
        Element::new(f)
    }
}
