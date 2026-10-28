//! The slider: a pill whose fill is the value, from 0 to 1 or one of a few stops.
//!
//! The fill's round end marks the value: the value is that end circle's centre, and at 0 the fill
//! is the circle behind the icon. The icon sits in that circle; the value, or a name for it, sits
//! at the far end, in the track's colours over the track and the fill's over the fill.
//!
//! # Owns
//!
//! - [`slider`], built up with [`Slider`], in two [`Size`]s.
//! - Where it stands ([`Value`]) and how it moves: an arrow key's [`Value::step`], Home and End
//!   ([`Value::end`]), the stop a released drag settles on ([`Value::at`]), the icon's walk down
//!   the stops and back ([`Value::turn`]) and a sticky point ([`stick`]).
//! - Its geometry and its marks ([`Track`]).
//!
//! # Not here
//!
//! - Drawing it and the pointer on it: `slider/pill.rs`. The keys: the app, which moves the
//!   [`Value`] it gives.

use iced::Element;

use crate::focus::RingColor;
use crate::focus::focus_ring;
use crate::icon::Icon;
use crate::look::Look;
use crate::probe;
use crate::probe::Fixed;
use crate::tokens::Radius;
use crate::tokens::Size as SizeToken;
use crate::tokens::Space;

mod pill;

/// An arrow key's step along a continuous slider: a twentieth, 5% of a volume.
const NUDGE: f32 = 0.05;

/// Which way the icon of a slider with stops moves it next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Toward the first stop: from Max Performance toward Super Saver.
    Down,
    /// Toward the last stop.
    Up,
}

/// Where the icon's walk starts: from high to low, so a press from Max Performance eases off.
pub const DEFAULT_DIRECTION: Direction = Direction::Down;

impl Default for Direction {
    fn default() -> Self {
        DEFAULT_DIRECTION
    }
}

/// Where a drag at `value` stands with a sticky point at `point`: on the point while within
/// `reach` of it, so the value holds there until pulled further. All three are fractions of the
/// track.
#[must_use]
pub fn stick(value: f32, point: f32, reach: f32) -> f32 {
    if (value - point).abs() <= reach { point } else { value }
}

/// Where a slider stands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    /// Anywhere from 0 to 1.
    Continuous(f32),
    /// Stop `index` of `count`, from 0.
    Stop {
        /// The stop it is on.
        index: u16,
        /// How many stops there are; two or more.
        count: u16,
    },
}

impl Value {
    /// How far along the track the thumb stands, 0 to 1.
    pub fn fraction(self) -> f32 {
        match self {
            Value::Continuous(v) => v.clamp(0.0, 1.0),
            Value::Stop { index, count } if count > 1 => f32::from(index.min(count - 1)) / f32::from(count - 1),
            Value::Stop { .. } => 0.0,
        }
    }

    /// One arrow key's move, toward the end when `forward`: a twentieth of the track, or a stop.
    /// It stops at either end.
    #[must_use]
    pub fn step(self, forward: bool) -> Value {
        match self {
            Value::Continuous(v) => {
                let by = if forward { NUDGE } else { -NUDGE };
                Value::Continuous((v + by).clamp(0.0, 1.0))
            }
            Value::Stop { index, count } => {
                let last = count.saturating_sub(1);
                let index = if forward {
                    index.saturating_add(1).min(last)
                } else {
                    index.saturating_sub(1)
                };
                Value::Stop { index, count }
            }
        }
    }

    /// Home's move to the start, or End's to the end when `end`.
    #[must_use]
    pub fn end(self, end: bool) -> Value {
        self.at(if end { 1.0 } else { 0.0 })
    }

    /// The icon's press on a slider with stops: one stop `direction`'s way, turning around at
    /// either end, so presses walk the stops down and back up. Gives the stop and the direction
    /// the next press takes; a slider without stops stays where it is.
    #[must_use]
    pub fn turn(self, direction: Direction) -> (Value, Direction) {
        let Value::Stop { index, count } = self else {
            return (self, direction);
        };
        if count < 2 {
            return (self, direction);
        }
        let last = count - 1;
        let direction = match direction {
            Direction::Down if index == 0 => Direction::Up,
            Direction::Up if index >= last => Direction::Down,
            kept => kept,
        };
        let index = match direction {
            Direction::Down => index.min(last) - 1,
            Direction::Up => index + 1,
        };
        (Value::Stop { index, count }, direction)
    }

    /// The value nearest `fraction` of the way along: itself for a continuous slider, the nearest
    /// stop for one with stops, where a released drag settles.
    #[must_use]
    pub fn at(self, fraction: f32) -> Value {
        let fraction = fraction.clamp(0.0, 1.0);
        match self {
            Value::Continuous(_) => Value::Continuous(fraction),
            Value::Stop { count, .. } => {
                let last = f32::from(count.saturating_sub(1));
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "Rounded, clamped to 0..=last, and last fits a u16."
                )]
                let index = (fraction * last).round() as u16;
                Value::Stop { index, count }
            }
        }
    }
}

/// A slider's height.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Size {
    /// 36 tall, its value always shown.
    #[default]
    Regular,
    /// 28 tall, for a dense panel: a number shows only under the pointer, while held or focused;
    /// a name always shows.
    Compact,
}

impl Size {
    /// How tall it is.
    pub fn height(self) -> f32 {
        f32::from(
            match self {
                Size::Regular => SizeToken::Slider,
                Size::Compact => SizeToken::SliderCompact,
            }
            .px(),
        )
    }

    /// The dot of the mark the value rests on.
    pub fn anchor(self) -> f32 {
        f32::from(
            match self {
                Size::Regular => SizeToken::SliderAnchor,
                Size::Compact => SizeToken::SliderAnchorCompact,
            }
            .px(),
        )
    }

    fn word(self) -> &'static str {
        match self {
            Size::Regular => "regular",
            Size::Compact => "compact",
        }
    }
}

/// A slider's track as its marks are placed on it: its size, its marks' and where its label
/// starts. Logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Track {
    pub width: f32,
    pub height: f32,
    /// A step's dot.
    pub stop: f32,
    /// The anchor dot.
    pub anchor: f32,
    /// The room kept clear between a mark and the icon or the label.
    pub gap: f32,
    /// Where the label's box starts, from the left.
    pub label: f32,
}

/// A dot drawn on a slider: a step, or the anchor of the mark the value rests on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mark {
    /// Its centre, from the left.
    pub x: f32,
    /// Whether it sits on the fill, so it takes the fill's content colour; the track's otherwise.
    pub on_fill: bool,
    /// Whether it is the solid dot of the mark the value rests on.
    pub anchor: bool,
}

impl Track {
    /// The track of `size`, `width` wide, its label's box starting at `label`.
    pub fn new(size: Size, width: f32, label: f32) -> Track {
        Track {
            width,
            height: size.height(),
            stop: f32::from(SizeToken::SliderStop.px()),
            anchor: size.anchor(),
            gap: f32::from(Space::S4.px()),
            label,
        }
    }

    /// Where the value `fraction` stands: the centre of the fill's round end, from half the height
    /// in at 0 to half the height short of the end at 1.
    pub fn centre(&self, fraction: f32) -> f32 {
        self.height / 2.0 + (self.width - self.height).max(0.0) * fraction.clamp(0.0, 1.0)
    }

    /// The fraction whose centre is at `x`: the pointer maps to the fill's end circle.
    pub fn fraction_at(&self, x: f32) -> f32 {
        let span = (self.width - self.height).max(1.0);
        ((x - self.height / 2.0) / span).clamp(0.0, 1.0)
    }

    /// The dots of `points`, fractions along the track, with the value at `shown` and resting on
    /// `resting` when it does. A point the value rests on is the anchor; a step under the moving
    /// value's end hides, and so does one the fill's edge crosses, which would take one side's
    /// colour over both; no dot is drawn under the icon or the label, which mark that place
    /// already.
    pub fn marks(&self, points: &[f32], shown: f32, resting: Option<f32>) -> Vec<Mark> {
        let value = self.centre(shown);
        let fill_end = value + self.height / 2.0;
        let (left, right) = (self.height + self.gap, self.label - self.gap);
        points
            .iter()
            .filter_map(|&point| {
                let x = self.centre(point);
                let anchor = resting.is_some_and(|rest| (rest - point).abs() < 1e-4);
                let size = if anchor { self.anchor } else { self.stop };
                let clear = x - size / 2.0 >= left && x + size / 2.0 <= right;
                let under_value = !anchor && (x - value).abs() < self.anchor / 2.0;
                let astride = !anchor && (x - fill_end).abs() < size / 2.0;
                (clear && !under_value && !astride).then_some(Mark {
                    x,
                    on_fill: x < fill_end,
                    anchor,
                })
            })
            .collect()
    }
}

/// What a slider shows at its end.
#[derive(Debug, Clone, PartialEq)]
enum Label {
    /// A number: the value as a percentage, or the owner's own. A compact slider shows it only
    /// under the pointer, while held or while focused.
    Number(Option<String>),
    /// A name, always shown: "Muted", "Auto · 55%".
    Words(String),
    /// Each stop's name, the one nearest the value shown.
    Stops(Vec<String>),
}

/// How a slider is drawn whatever the pointer does: the specimen's view of a state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Preview {
    #[default]
    None,
    Hovered,
    Held,
}

/// A slider showing `glyph`, at `value`, built up then drawn with [`Slider::view`].
pub struct Slider<'a, M> {
    glyph: Icon,
    value: Value,
    size: Size,
    label: Label,
    widest: String,
    soft: bool,
    sticky: Option<f32>,
    on_change: Option<Box<dyn Fn(Value) -> M + 'a>>,
    on_icon: Option<M>,
    focused: bool,
    preview: Preview,
}

impl<M> std::fmt::Debug for Slider<'_, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Slider")
            .field("glyph", &self.glyph)
            .field("value", &self.value)
            .field("size", &self.size)
            .field("label", &self.label)
            .field("soft", &self.soft)
            .field("sticky", &self.sticky)
            .field("enabled", &self.on_change.is_some())
            .field("focused", &self.focused)
            .finish_non_exhaustive()
    }
}

/// The widest a percentage reads: its number's box is this wide, so the number ends in one place
/// as it ticks. iced's text has no font features, so its figures are proportional, not tabular.
const DEFAULT_WIDEST: &str = "100%";

/// A regular slider showing `glyph` at `value`, its value as a percentage; disabled until it has
/// [`Slider::on_change`].
pub fn slider<'a, M>(glyph: Icon, value: Value) -> Slider<'a, M> {
    Slider {
        glyph,
        value,
        size: Size::Regular,
        label: Label::Number(None),
        widest: DEFAULT_WIDEST.to_owned(),
        soft: false,
        sticky: None,
        on_change: None,
        on_icon: None,
        focused: false,
        preview: Preview::None,
    }
}

impl<'a, M: Clone + 'a> Slider<'a, M> {
    /// 28 tall, for a dense panel.
    #[must_use]
    pub fn compact(self) -> Self {
        Slider {
            size: Size::Compact,
            ..self
        }
    }

    /// The number shown in place of the percentage, as the owner reads the value: "100%" on a
    /// track that runs to 150%.
    #[must_use]
    pub fn number(self, number: impl Into<String>) -> Self {
        Slider {
            label: Label::Number(Some(number.into())),
            ..self
        }
    }

    /// A name shown in place of the value, always: "Muted".
    #[must_use]
    pub fn words(self, words: impl Into<String>) -> Self {
        Slider {
            label: Label::Words(words.into()),
            ..self
        }
    }

    /// Each stop's name, for a slider with stops: the one nearest the value shows.
    #[must_use]
    pub fn stops(self, names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Slider {
            label: Label::Stops(names.into_iter().map(Into::into).collect()),
            ..self
        }
    }

    /// The widest the number reads, for a slider whose number runs past 100%.
    #[must_use]
    pub fn widest(self, widest: impl Into<String>) -> Self {
        Slider {
            widest: widest.into(),
            ..self
        }
    }

    /// The fill in the softer, desaturated accent: a level something else picked, as Auto
    /// brightness's.
    #[must_use]
    pub fn soft(self) -> Self {
        Slider { soft: true, ..self }
    }

    /// A sticky point, a fraction along the track: a drag within [`SizeToken::SliderHold`] of it
    /// holds there until pulled further, and its dot shows while the value rests on it.
    #[must_use]
    pub fn sticky(self, point: f32) -> Self {
        Slider {
            sticky: Some(point.clamp(0.0, 1.0)),
            ..self
        }
    }

    /// What a press or a drag sends: the value under the pointer, at once on a continuous slider
    /// and on release on one with stops, which then settles on the nearest. A slider with stops
    /// also sends the icon's step. Without it the slider is disabled.
    #[must_use]
    pub fn on_change(self, on_change: impl Fn(Value) -> M + 'a) -> Self {
        Slider {
            on_change: Some(Box::new(on_change)),
            ..self
        }
    }

    /// What a press on the icon sends: the icon is then a button, as volume's mutes.
    #[must_use]
    pub fn on_icon(self, message: M) -> Self {
        Slider {
            on_icon: Some(message),
            ..self
        }
    }

    /// Whether the keyboard is on it: the focus tube lights its edge.
    #[must_use]
    pub fn focused(self, focused: bool) -> Self {
        Slider { focused, ..self }
    }

    /// Drawn as under the pointer, whatever the pointer does: the specimen's view of the state.
    #[must_use]
    pub fn hovered(self) -> Self {
        Slider {
            preview: Preview::Hovered,
            ..self
        }
    }

    /// Drawn as while a press holds it, its fill lit: the specimen's view of the state.
    #[must_use]
    pub fn held(self) -> Self {
        Slider {
            preview: Preview::Held,
            ..self
        }
    }

    pub fn view(self, look: &Look) -> Element<'a, M> {
        let stepped = matches!(self.value, Value::Stop { count, .. } if count > 1);
        let size = self.size;
        let focused = self.focused;
        let pill: Element<'a, M> = Element::new(pill::Pill::new(look, self));
        let view: Element<'a, M> = focus_ring(look, pill, Radius::Full, focused, RingColor::Accent);
        let variant = [size.word(), if stepped { "stepped" } else { "" }, ""];
        probe::component("Slider", variant, Fixed::HEIGHT, view)
    }
}

#[cfg(test)]
mod tests;
