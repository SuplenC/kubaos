//! The slider: a value from 0 to 1, or one of a few stops, picked along a track.

use iced::Color;
use iced::Element;
use iced::Length;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Size as IcedSize;
use iced::Theme;
use iced::mouse;
use iced::widget::Action;
use iced::widget::canvas;
use iced::widget::canvas::Event;
use iced::widget::canvas::Frame;
use iced::widget::canvas::Geometry;
use iced::widget::canvas::Path;

use crate::color;
use crate::color::Colors;
use crate::focus::RingColor;
use crate::focus::focus_ring;
use crate::look::Look;
use crate::probe;
use crate::probe::Fixed;
use crate::scale::Scale;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Size;

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

    /// The value nearest `fraction` of the way along: itself for a continuous slider, the nearest
    /// stop for one with stops.
    fn at(self, fraction: f32) -> Value {
        let fraction = fraction.clamp(0.0, 1.0);
        match self {
            Value::Continuous(_) => Value::Continuous(fraction),
            Value::Stop { count, .. } => {
                let last = f32::from(count.saturating_sub(1));
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "rounded, clamped to 0..=last, and last fits a u16"
                )]
                let index = (fraction * last).round() as u16;
                Value::Stop { index, count }
            }
        }
    }
}

/// A slider at `value`, 24 tall: a progress bar's track filled up to a 16 thumb. A slider with
/// stops marks each one the fill hasn't reached. A press or a drag on it sends `on_change` with
/// the value under the pointer, at once for a continuous slider and on release for one with stops,
/// which shows the stop under the pointer while it is held; without `on_change` it is disabled.
/// Its focus ring is a pill round it all.
pub fn slider<'a, M: 'a>(
    look: &Look,
    value: Value,
    on_change: Option<Box<dyn Fn(Value) -> M + 'a>>,
    focused: bool,
) -> Element<'a, M> {
    slider_in(look, value, on_change, focused, false)
}

/// [`slider`] drawn as while a press holds it, its fill lit: the specimen's view of the state.
pub fn slider_held<'a, M: 'a>(
    look: &Look,
    value: Value,
    on_change: Option<Box<dyn Fn(Value) -> M + 'a>>,
) -> Element<'a, M> {
    slider_in(look, value, on_change, false, true)
}

fn slider_in<'a, M: 'a>(
    look: &Look,
    value: Value,
    on_change: Option<Box<dyn Fn(Value) -> M + 'a>>,
    focused: bool,
    held: bool,
) -> Element<'a, M> {
    let stepped = matches!(value, Value::Stop { count, .. } if count > 1);
    let track = canvas(Track {
        value,
        on_change,
        colors: look.colors,
        bright: look.bright(),
        scale: look.scale,
        held,
    })
    .width(Length::Fill)
    .height(f32::from(Size::Slider.px()));
    let view: Element<'a, M> = focus_ring(look, track, Radius::Full, focused, RingColor::Accent);
    let variant = if stepped { "stepped" } else { "" };
    probe::component("Slider", [variant, "", ""], Fixed::HEIGHT, view)
}

struct Track<'a, M> {
    value: Value,
    on_change: Option<Box<dyn Fn(Value) -> M + 'a>>,
    colors: Colors,
    /// What the fill brightens toward under the pointer and while held: [`Look::bright`].
    bright: Color,
    scale: Scale,
    /// Drawn as held whatever the pointer does.
    held: bool,
}

/// A press held on the slider: the value under the pointer.
#[derive(Debug, Default)]
struct Held(Option<Value>);

impl<M> Track<'_, M> {
    /// The fraction of the way along the track at `x`, the thumb's centre travelling from half a
    /// thumb in to half a thumb short of the end.
    fn fraction_at(bounds: Rectangle, x: f32) -> f32 {
        let thumb = f32::from(Size::SliderThumb.px());
        let travel = (bounds.width - thumb).max(1.0);
        (x - thumb / 2.0) / travel
    }
}

impl<M> canvas::Program<M> for Track<'_, M> {
    type State = Held;

    fn update(&self, held: &mut Held, event: &Event, bounds: Rectangle, cursor: mouse::Cursor) -> Option<Action<M>> {
        let on_change = self.on_change.as_ref()?;
        let continuous = matches!(self.value, Value::Continuous(_));
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let at = cursor.position_in(bounds)?;
                let value = self.value.at(Self::fraction_at(bounds, at.x));
                held.0 = Some(value);
                Some(if continuous {
                    Action::publish(on_change(value)).and_capture()
                } else {
                    Action::request_redraw().and_capture()
                })
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) if held.0.is_some() => {
                let value = self.value.at(Self::fraction_at(bounds, position.x - bounds.x));
                held.0 = Some(value);
                Some(if continuous {
                    Action::publish(on_change(value))
                } else {
                    Action::request_redraw()
                })
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let value = held.0.take()?;
                Some(if continuous {
                    Action::request_redraw()
                } else {
                    Action::publish(on_change(value))
                })
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        held: &Held,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let c = &self.colors;
        let enabled = self.on_change.is_some();
        let dim = if enabled { 1.0 } else { Opacity::DisabledContent.value() };
        let shown = match (held.0, self.value) {
            (Some(stop @ Value::Stop { .. }), _) => stop,
            (_, value) => value,
        };
        let width = bounds.width;
        let thumb = f32::from(Size::SliderThumb.px());
        let rail = f32::from(Size::Progress.px());
        let mark = f32::from(Size::SliderStop.px());
        let height = bounds.height;
        let travel = (width - thumb).max(0.0);
        let snap = |v: f32| self.scale.snap(v);
        let mut frame = Frame::new(renderer, bounds.size());

        let rail_y = snap((height - rail) / 2.0);
        let track = color::alpha(c.surface_container_highest, Opacity::Track.value());
        frame.fill(
            &Path::rounded_rectangle(Point::new(0.0, rail_y), IcedSize::new(width, rail), (rail / 2.0).into()),
            color::alpha(track, dim),
        );
        let layer = if !enabled {
            0.0
        } else if held.0.is_some() || self.held {
            Opacity::Press.value()
        } else if cursor.is_over(bounds) {
            Opacity::Hover.value()
        } else {
            0.0
        };
        let lift = if !enabled {
            0.0
        } else if held.0.is_some() || self.held {
            Opacity::Held.value()
        } else if cursor.is_over(bounds) {
            Opacity::Press.value()
        } else {
            0.0
        };
        let thumb_x = snap(travel * shown.fraction());
        // The fill lights up toward the palette's bright end: a little under the pointer, and
        // plainly while pressed or dragged, as a progress bar's fill glows.
        frame.fill(
            &Path::rounded_rectangle(
                Point::new(0.0, rail_y),
                IcedSize::new(thumb_x + thumb / 2.0, rail),
                (rail / 2.0).into(),
            ),
            color::alpha(color::mix(c.primary, self.bright, lift), dim),
        );
        if let Value::Stop { index, count } = shown
            && count > 1
        {
            let last = f32::from(count - 1);
            for stop in (index + 1)..count {
                let x = snap(travel * f32::from(stop) / last + (thumb - mark) / 2.0);
                let y = snap((height - mark) / 2.0);
                frame.fill(
                    &Path::circle(Point::new(x + mark / 2.0, y + mark / 2.0), mark / 2.0),
                    color::alpha(c.on_surface_variant, dim),
                );
            }
        }
        let thumb_y = snap((height - thumb) / 2.0);
        let centre = Point::new(thumb_x + thumb / 2.0, thumb_y + thumb / 2.0);
        let fill: Color = color::mix(
            c.primary,
            Color {
                a: c.primary.a,
                ..c.on_primary
            },
            layer,
        );
        frame.fill(&Path::circle(centre, thumb / 2.0), color::alpha(fill, dim));
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(&self, _held: &Held, bounds: Rectangle, cursor: mouse::Cursor) -> mouse::Interaction {
        if self.on_change.is_some() && cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}
