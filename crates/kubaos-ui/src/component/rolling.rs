//! A count that rolls as it changes, as an odometer does.
//!
//! Its figures are tabular, each in a slot one figure wide and the style's line tall that cuts what
//! leaves it: the figure's advance rounded up to a whole device pixel, so each figure starts on
//! one. The box is as many slots wide as the widest value it is given, its figures held to its end,
//! so nothing beside it moves as the count grows a figure.
//!
//! Only the figures that change roll. Going up, the new one comes in from below as the old one
//! leaves upward; going down, from above as the old one leaves downward, the pair moving together
//! over `short` on `standard`. A figure the count grows comes in from nothing, and one it sheds
//! leaves into nothing. With reduce motion the figures change in place.
//!
//! # Owns
//!
//! - [`rolling_number`], built up with [`RollingNumber`].
//!
//! # Not here
//!
//! - The count, and the buttons that change it: the app's.

use std::time::Instant;

use iced::Color;
use iced::Element;
use iced::Event;
use iced::Font;
use iced::Length;
use iced::Pixels;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Size;
use iced::Theme;
use iced::advanced::Clipboard;
use iced::advanced::Layout;
use iced::advanced::Shell;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::text;
use iced::advanced::text::Paragraph as _;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree;
use iced::alignment;

use crate::look::Look;
use crate::motion::Motion;
use crate::probe;
use crate::probe::Fixed;
use crate::scale::Scale;
use crate::text::font;
use crate::text::line_height;
use crate::tokens::Duration;
use crate::tokens::Easing;
use crate::tokens::Type;

type Paragraph = <Renderer as text::Renderer>::Paragraph;

/// A count of `value`, built up then drawn with [`RollingNumber::view`].
#[derive(Debug, Clone, Copy)]
pub struct RollingNumber<'a> {
    value: u64,
    widest: &'a str,
    style: Type,
    color: Option<Color>,
}

/// `value` in `mono_large`, `on_surface`, in a box one figure wide until it is given a wider one.
pub fn rolling_number<'a>(value: u64) -> RollingNumber<'a> {
    RollingNumber {
        value,
        widest: "0",
        style: Type::MonoLarge,
        color: None,
    }
}

impl<'a> RollingNumber<'a> {
    /// The widest value it shows, its figures all 0: `"000"` for a count up to 999. A value wider
    /// than it widens the box.
    #[must_use]
    pub fn widest(self, widest: &'a str) -> Self {
        RollingNumber { widest, ..self }
    }

    /// The type style its figures are in.
    #[must_use]
    pub fn style(self, style: Type) -> Self {
        RollingNumber { style, ..self }
    }

    #[must_use]
    pub fn color(self, color: Color) -> Self {
        RollingNumber {
            color: Some(color),
            ..self
        }
    }

    pub fn view<M: 'a>(self, look: &Look) -> Element<'a, M> {
        let rolling = Element::new(Rolling {
            figures: figures(self.value),
            value: self.value,
            widest: self.widest.chars().count(),
            style: self.style,
            color: self.color.unwrap_or(look.colors.on_surface),
            scale: look.scale,
            motion: look.motion,
        });
        probe::component("RollingNumber", ["", "", ""], Fixed::HEIGHT, rolling)
    }
}

/// `count`'s figures. A count is an integer, never grouped or padded, so its figures are the same
/// in every locale this formats for.
fn figures(count: u64) -> Vec<char> {
    count.to_string().chars().collect()
}

/// The slots going from `was` to `now`, each held to the same end: a slot whose figure differs
/// rolls, the old figure leaving it, unless motion is `reduced`, which changes it in place.
fn changed(was: &[Option<char>], now: &[Option<char>], reduced: bool) -> Vec<Slot> {
    was.iter()
        .zip(now)
        .map(|(&was, &figure)| {
            let rolls = was != figure && !reduced;
            Slot {
                figure,
                leaving: if rolls { was } else { None },
                rolls,
            }
        })
        .collect()
}

struct Rolling {
    figures: Vec<char>,
    value: u64,
    widest: usize,
    style: Type,
    color: Color,
    scale: Scale,
    motion: Motion,
}

/// One slot's figure, the one rolling out of it, and whether the slot rolls on the last change.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Slot {
    figure: Option<char>,
    leaving: Option<char>,
    rolls: bool,
}

/// What a rolling number shows, and how far its last change has rolled.
struct State {
    value: u64,
    /// Every slot, from the box's start.
    slots: Vec<Slot>,
    /// Which way the last change went: 1 up, -1 down.
    way: f32,
    started: Option<Instant>,
    /// A figure's advance, and the style and scale it was measured at.
    advance: Option<(Type, f32, f32)>,
}

impl Rolling {
    fn slot_count(&self) -> usize {
        self.widest.max(self.figures.len())
    }

    /// The figures held to the end of `count` slots.
    fn placed(&self, count: usize) -> Vec<Option<char>> {
        let blank = count.saturating_sub(self.figures.len());
        (0..count)
            .map(|at| at.checked_sub(blank).map(|index| self.figures[index]))
            .collect()
    }

    fn shaped<'b>(&self, content: &'b str) -> text::Text<&'b str, Font> {
        text::Text {
            content,
            bounds: Size::INFINITE,
            size: Pixels(f32::from(self.style.size())),
            line_height: line_height(self.style),
            font: font(self.style),
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Top,
            shaping: text::Shaping::Basic,
            wrapping: text::Wrapping::None,
        }
    }

    /// A slot's width: a figure's advance rounded up to a whole device pixel.
    fn slot_width(&self, state: &mut State) -> f32 {
        let key = (self.style, self.scale.factor());
        let advance = match state.advance {
            Some((style, factor, advance)) if (style, factor) == key => advance,
            _ => {
                let advance = Paragraph::with_text(self.shaped("0")).min_width();
                state.advance = Some((self.style, self.scale.factor(), advance));
                advance
            }
        };
        self.scale.snap_up(advance)
    }

    /// How far the last change has rolled at `now`, 0 to 1.
    fn rolled(&self, state: &State, now: Instant) -> f32 {
        state.started.map_or(1.0, |at| {
            self.motion
                .progress(Duration::Short, Easing::Standard, now.saturating_duration_since(at))
        })
    }
}

impl<M> Widget<M, Theme, Renderer> for Rolling {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        let count = self.slot_count();
        tree::State::new(State {
            value: self.value,
            slots: self
                .placed(count)
                .into_iter()
                .map(|figure| Slot {
                    figure,
                    leaving: None,
                    rolls: false,
                })
                .collect(),
            way: 1.0,
            started: None,
            advance: None,
        })
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();
        if state.value == self.value {
            return;
        }
        let reduced = self.motion.is_reduced();
        let shown: Vec<Option<char>> = state.slots.iter().map(|slot| slot.figure).collect();
        // Both held to the end of the wider of the two, so each figure meets its own place.
        let width = self.slot_count().max(shown.len());
        let was: Vec<Option<char>> = std::iter::repeat_n(None, width - shown.len()).chain(shown).collect();
        state.slots = changed(&was, &self.placed(width), reduced);
        // A slot the count shed keeps its room until its figure has rolled away; changed in
        // place, it gives it back at once.
        if reduced {
            let shed = state.slots.len() - self.slot_count();
            state.slots.drain(..shed);
        }
        state.way = if self.value > state.value { 1.0 } else { -1.0 };
        state.value = self.value;
        state.started = (!reduced).then(Instant::now);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&mut self, tree: &mut Tree, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        let slot = self.slot_width(state);
        let count = state.slots.len().max(self.slot_count());
        #[expect(clippy::cast_precision_loss, reason = "A count's figures are a few.")]
        let width = slot * count as f32;
        layout::Node::new(limits.resolve(
            Length::Shrink,
            Length::Shrink,
            Size::new(width, f32::from(self.style.line())),
        ))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        _viewport: &Rectangle,
    ) {
        let Event::Window(iced::window::Event::RedrawRequested(at)) = event else {
            return;
        };
        let state = tree.state.downcast_mut::<State>();
        if state.started.is_none() {
            return;
        }
        if self.rolled(state, *at) < 1.0 {
            shell.request_redraw();
            return;
        }
        // Rolled: the leaving figures are gone, and a shed slot gives its room back.
        state.started = None;
        let count = self.slot_count();
        let shed = state.slots.len().saturating_sub(count);
        state.slots.drain(..shed);
        for slot in &mut state.slots {
            slot.leaving = None;
            slot.rolls = false;
        }
        if shed > 0 {
            shell.invalidate_layout();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::text::Renderer as _;

        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        #[expect(clippy::cast_precision_loss, reason = "A count's figures are a few.")]
        let slot = bounds.width / state.slots.len().max(1) as f32;
        let rolled = self.rolled(state, Instant::now());
        let line = bounds.height;
        let mut buffer = [0_u8; 4];
        let mut figure = |renderer: &mut Renderer, char: char, x: f32, y: f32, clip: Rectangle| {
            let content = char.encode_utf8(&mut buffer);
            let shaped = self.shaped(content);
            renderer.fill_text(
                text::Text {
                    content: shaped.content.to_owned(),
                    bounds: Size::new(slot, line),
                    size: shaped.size,
                    line_height: shaped.line_height,
                    font: shaped.font,
                    align_x: shaped.align_x,
                    align_y: shaped.align_y,
                    shaping: shaped.shaping,
                    wrapping: shaped.wrapping,
                },
                Point::new(x, y),
                self.color,
                clip,
            );
        };
        for (index, each) in state.slots.iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "A count's figures are a few.")]
            let x = bounds.x + slot * index as f32;
            let cell = Rectangle {
                x,
                y: bounds.y,
                width: slot,
                height: line,
            };
            let Some(clip) = cell.intersection(viewport) else {
                continue;
            };
            let rolling = each.rolls && rolled < 1.0;
            if let Some(leaving) = each.leaving.filter(|_| rolling) {
                let y = self.scale.snap(-rolled * state.way * line);
                figure(renderer, leaving, x, bounds.y + y, clip);
            }
            if let Some(shown) = each.figure {
                let y = if rolling {
                    self.scale.snap((1.0 - rolled) * state.way * line)
                } else {
                    0.0
                };
                figure(renderer, shown, x, bounds.y + y, clip);
            }
        }
    }
}

#[cfg(test)]
mod tests;
