//! Drawing a slider, and following the pointer on it.
//!
//! The pill is one widget, so the geometry check sees one box, as the QML slider reports it: its
//! icon and its label are drawn by it, not laid out beside it.

use std::time::Instant;

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Event;
use iced::Length;
use iced::Pixels;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Size as IcedSize;
use iced::Theme;
use iced::advanced::Clipboard;
use iced::advanced::Layout;
use iced::advanced::Shell;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::svg;
use iced::advanced::text;
use iced::advanced::text::Paragraph as _;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree;
use iced::alignment;

use super::Direction;
use super::Label;
use super::Preview;
use super::Size;
use super::Slider;
use super::Track;
use super::Value;
use super::stick;
use crate::color;
use crate::color::Colors;
use crate::icon::symbol;
use crate::look::Look;
use crate::motion::Tween;
use crate::scale::Scale;
use crate::text::font;
use crate::text::line_height;
use crate::tokens::Duration;
use crate::tokens::Easing;
use crate::tokens::Opacity;
use crate::tokens::Size as SizeToken;
use crate::tokens::Space;
use crate::tokens::Type;

type Paragraph = <Renderer as text::Renderer>::Paragraph;

pub(super) struct Pill<'a, M> {
    slider: Slider<'a, M>,
    colors: Colors,
    /// What the fill brightens toward under the pointer and while held: [`Look::bright`].
    bright: Color,
    scale: Scale,
}

impl<'a, M> Pill<'a, M> {
    pub(super) fn new(look: &Look, slider: Slider<'a, M>) -> Self {
        Pill {
            slider,
            colors: look.colors,
            bright: look.bright(),
            scale: look.scale,
        }
    }
}

/// What the pill keeps between frames.
struct State {
    /// The fraction under the pointer while a press on the track holds it.
    drag: Option<f32>,
    /// The fraction the fill shows when no drag holds it: it eases to a new value, and settles on
    /// a stop after a drag.
    shown: Tween,
    /// The anchor dot's scale, popping in from nothing when the value comes to rest on a mark.
    anchor: Tween,
    /// The mark the value last rested on, so a new one pops.
    resting: Option<f32>,
    /// Which way the icon steps a slider with stops next.
    direction: Direction,
    /// A compact slider's number fading in under the pointer and out after.
    number: Tween,
    hovered: bool,
    over_icon: bool,
    pressing_icon: bool,
    /// Each label it may show, shaped: one, or every stop's name.
    labels: Vec<(String, Paragraph)>,
    /// The widest the number reads, and its width.
    widest: (String, f32),
}

/// A label `content` in `style`, shaped on one line.
fn shaped(content: &str, style: Type) -> Paragraph {
    Paragraph::with_text(text::Text {
        content,
        bounds: IcedSize::INFINITE,
        size: Pixels(f32::from(style.size())),
        line_height: line_height(style),
        font: font(style),
        align_x: text::Alignment::Default,
        align_y: alignment::Vertical::Top,
        shaping: text::Shaping::Auto,
        wrapping: text::Wrapping::None,
    })
}

/// The value as a whole percentage.
fn percent(fraction: f32) -> String {
    format!("{:.0}%", fraction * 100.0)
}

impl<M> Pill<'_, M> {
    fn stepped(&self) -> bool {
        matches!(self.slider.value, Value::Stop { count, .. } if count > 1)
    }

    fn enabled(&self) -> bool {
        self.slider.on_change.is_some()
    }

    /// Whether the icon is a button: it steps a slider with stops, or sends [`Slider::on_icon`].
    fn icon_presses(&self) -> bool {
        self.enabled() && (self.slider.on_icon.is_some() || self.stepped())
    }

    fn style(&self) -> Type {
        match self.slider.label {
            Label::Number(_) => Type::LabelNumber,
            Label::Words(_) | Label::Stops(_) => Type::Label,
        }
    }

    /// The labels it may show, in order: a stop's name is the one at its index.
    fn label_texts(&self) -> Vec<String> {
        match &self.slider.label {
            Label::Number(None) => vec![percent(self.slider.value.fraction())],
            Label::Number(Some(number)) => vec![number.clone()],
            Label::Words(words) => vec![words.clone()],
            Label::Stops(names) => names.clone(),
        }
    }

    /// The sticky point's pull, as a fraction of a track `width` wide.
    fn hold(&self, width: f32) -> f32 {
        f32::from(SizeToken::SliderHold.px()) / (width - self.slider.size.height()).max(1.0)
    }

    /// The fraction under the pointer at `x` on a track `width` wide, held on the sticky point
    /// while near it.
    fn fraction_at(&self, width: f32, x: f32) -> f32 {
        let fraction = Track::new(self.slider.size, width, width).fraction_at(x);
        match self.slider.sticky {
            Some(point) => stick(fraction, point, self.hold(width)),
            None => fraction,
        }
    }

    /// The mark the value rests on at `now`: a slider with stops at rest on its stop, or a sticky
    /// point the value holds at.
    fn resting(&self, state: &State, now: Instant) -> Option<f32> {
        if self.stepped() {
            return (state.drag.is_none() && !state.shown.moving(now)).then(|| self.slider.value.fraction());
        }
        let at = state.drag.unwrap_or_else(|| self.slider.value.fraction());
        self.slider.sticky.filter(|point| (at - point).abs() < 1e-4)
    }

    /// Whether a compact slider's number shows: under the pointer, while held, or focused.
    fn number_wanted(&self, state: &State) -> bool {
        self.slider.size == Size::Regular
            || state.hovered
            || state.drag.is_some()
            || self.slider.focused
            || self.slider.preview != Preview::None
    }

    /// Pops the anchor in when the value has come to rest on a new mark, and fades a compact
    /// slider's number toward what the pointer asks.
    fn refresh(&self, state: &mut State, now: Instant) {
        let resting = self.resting(state, now);
        if resting != state.resting {
            state.anchor = Tween::new(0.0, Duration::Short, Easing::Pop);
            state.anchor.to(1.0, now);
            state.resting = resting;
        }
        let number = if self.number_wanted(state) { 1.0 } else { 0.0 };
        state.number.to(number, now);
    }

    fn moving(state: &State, now: Instant) -> bool {
        state.shown.moving(now) || state.anchor.moving(now) || state.number.moving(now)
    }

    /// Whether `at`, a point in the pill, is on the icon's circle.
    fn on_icon(&self, at: Point) -> bool {
        let radius = self.slider.size.height() / 2.0;
        (at.x - radius).hypot(at.y - radius) <= radius
    }

    fn easing(&self) -> Easing {
        if self.stepped() {
            Easing::StandardDecel
        } else {
            Easing::Standard
        }
    }

    /// What one frame draws with, worked out once.
    fn paint<'s>(&self, state: &'s State, now: Instant, bounds: Rectangle) -> Paint<'s> {
        let c = &self.colors;
        let enabled = self.enabled();
        let dim = if enabled { 1.0 } else { Opacity::DisabledContent.value() };
        let fraction = state.drag.unwrap_or_else(|| state.shown.value(now));
        let label = match &self.slider.label {
            Label::Stops(_) => {
                let last = state.labels.len().saturating_sub(1);
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    clippy::cast_precision_loss,
                    reason = "Rounded and clamped to 0..=last, a handful of stops."
                )]
                let index = ((fraction * last as f32).round() as usize).min(last);
                state.labels.get(index)
            }
            Label::Number(_) | Label::Words(_) => state.labels.first(),
        };
        let pad = f32::from(Space::S12.px());
        let text_width = label.map_or(0.0, |(_, paragraph)| paragraph.min_width());
        let box_width = if self.style().tabular() {
            text_width.max(state.widest.1)
        } else {
            text_width
        };
        let track = Track::new(self.slider.size, bounds.width, bounds.width - pad - box_width);
        let fill_end = self
            .scale
            .snap(track.centre(fraction) + bounds.height / 2.0)
            .min(bounds.width);
        let held = enabled && (state.drag.is_some() || self.slider.preview == Preview::Held);
        let hovered = enabled && ((state.hovered && !state.over_icon) || self.slider.preview == Preview::Hovered);
        let lift = if held {
            Opacity::Held.value()
        } else if hovered {
            Opacity::Press.value()
        } else {
            0.0
        };
        let (fill, on_fill) = if self.slider.soft {
            (c.secondary_container, c.on_secondary_container)
        } else {
            (c.primary, c.on_primary)
        };
        Paint {
            fraction,
            track,
            fill_end,
            fill: color::alpha(color::mix(fill, self.bright, lift), dim),
            on_fill: color::alpha(on_fill, dim),
            dim,
            label,
            text_width,
            pad,
        }
    }

    fn draw_track(&self, renderer: &mut Renderer, bounds: Rectangle, paint: &Paint<'_>) {
        use iced::advanced::Renderer as _;

        let pill = Border::default().rounded(bounds.height / 2.0);
        let track = color::alpha(self.colors.surface_container_highest, Opacity::Track.value());
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: pill,
                snap: true,
                ..renderer::Quad::default()
            },
            Background::Color(color::alpha(track, paint.dim)),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    width: paint.fill_end,
                    ..bounds
                },
                border: pill,
                snap: true,
                ..renderer::Quad::default()
            },
            Background::Color(paint.fill),
        );
    }

    /// The steps on either side of the fill, each in its side's colour, and the anchor of the mark
    /// the value rests on.
    fn draw_marks(&self, renderer: &mut Renderer, state: &State, now: Instant, bounds: Rectangle, paint: &Paint<'_>) {
        use iced::advanced::Renderer as _;

        let points: Vec<f32> = match self.slider.value {
            Value::Stop { count, .. } if count > 1 => {
                let last = f32::from(count - 1);
                (0..count).map(|stop| f32::from(stop) / last).collect()
            }
            Value::Stop { .. } | Value::Continuous(_) => self.slider.sticky.into_iter().collect(),
        };
        let track = &paint.track;
        for mark in track.marks(&points, paint.fraction, self.resting(state, now)) {
            let (side, colour) = if mark.anchor {
                (track.anchor * state.anchor.value(now).max(0.0), paint.on_fill)
            } else if mark.on_fill {
                (track.stop, color::alpha(paint.on_fill, Opacity::Subdued.value()))
            } else {
                (track.stop, color::alpha(self.colors.on_surface_variant, paint.dim))
            };
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        Point::new(bounds.x + mark.x - side / 2.0, bounds.y + (bounds.height - side) / 2.0),
                        IcedSize::new(side, side),
                    ),
                    border: Border::default().rounded(side / 2.0),
                    ..renderer::Quad::default()
                },
                Background::Color(colour),
            );
        }
    }

    /// The icon in the circle the fill starts with, always on the fill; its state layer when it
    /// is a button under the pointer.
    fn draw_icon(
        &self,
        renderer: &mut Renderer,
        state: &State,
        bounds: Rectangle,
        paint: &Paint<'_>,
        viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        use iced::advanced::svg::Renderer as _;

        let height = bounds.height;
        let icon = Type::Label.icon();
        let side = f32::from(icon.px());
        let inset = self.scale.snap((height - side) / 2.0);
        renderer.draw_svg(
            svg::Svg::new(symbol(self.slider.glyph, icon, self.scale))
                .color(Color {
                    a: 1.0,
                    ..paint.on_fill
                })
                .opacity(paint.on_fill.a),
            Rectangle::new(
                Point::new(bounds.x + inset, bounds.y + inset),
                IcedSize::new(side, side),
            ),
            *viewport,
        );
        if !(self.icon_presses() && (state.over_icon || state.pressing_icon)) {
            return;
        }
        let layer = if state.pressing_icon {
            Opacity::Press
        } else {
            Opacity::Hover
        };
        // The state layer goes over its content, as every state layer does: in a layer of its own,
        // since a picture draws over the quads of its own layer.
        renderer.with_layer(*viewport, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(bounds.position(), IcedSize::new(height, height)),
                    border: Border::default().rounded(height / 2.0),
                    snap: true,
                    ..renderer::Quad::default()
                },
                Background::Color(color::alpha(paint.on_fill, layer.value())),
            );
        });
    }

    /// The label at the far end: in the track's content colour over the track and the fill's over
    /// the fill, split where the fill ends.
    fn draw_label(
        &self,
        renderer: &mut Renderer,
        state: &State,
        now: Instant,
        bounds: Rectangle,
        paint: &Paint<'_>,
        viewport: &Rectangle,
    ) {
        use iced::advanced::text::Renderer as _;

        let Some((_, paragraph)) = paint.label else {
            return;
        };
        let opacity = match self.slider.label {
            Label::Number(_) if self.slider.size == Size::Compact => state.number.value(now).clamp(0.0, 1.0),
            Label::Number(_) | Label::Words(_) | Label::Stops(_) => 1.0,
        };
        if opacity <= 0.0 {
            return;
        }
        let line = f32::from(self.style().line());
        let at = Point::new(
            bounds.x + bounds.width - paint.pad - paint.text_width,
            bounds.y + self.scale.snap((bounds.height - line) / 2.0),
        );
        let split = bounds.x + paint.fill_end;
        let over_track = Rectangle {
            x: split,
            width: (bounds.x + bounds.width - split).max(0.0),
            ..bounds
        };
        let over_fill = Rectangle {
            width: paint.fill_end,
            ..bounds
        };
        let on_track = color::alpha(self.colors.on_surface, paint.dim);
        for (clip, colour) in [(over_track, on_track), (over_fill, paint.on_fill)] {
            if let Some(clip) = clip.intersection(viewport) {
                renderer.fill_paragraph(paragraph, at, color::alpha(colour, opacity), clip);
            }
        }
    }
}

/// What a frame of the pill draws with.
struct Paint<'s> {
    /// The fraction the fill shows.
    fraction: f32,
    track: Track,
    /// Where the fill ends, on a device pixel.
    fill_end: f32,
    fill: Color,
    /// The content colour over the fill.
    on_fill: Color,
    /// The disabled dimming, 1 when enabled.
    dim: f32,
    label: Option<&'s (String, Paragraph)>,
    text_width: f32,
    /// The label's room from the far end.
    pad: f32,
}

impl<M: Clone> Widget<M, Theme, Renderer> for Pill<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        let fraction = self.slider.value.fraction();
        let mut state = State {
            drag: None,
            shown: Tween::new(fraction, Duration::Short, self.easing()),
            anchor: Tween::new(1.0, Duration::Short, Easing::Pop),
            resting: None,
            direction: Direction::default(),
            number: Tween::new(0.0, Duration::Micro, Easing::Standard),
            hovered: false,
            over_icon: false,
            pressing_icon: false,
            labels: Vec::new(),
            widest: (String::new(), 0.0),
        };
        state.resting = self.resting(&state, Instant::now());
        let number = if self.number_wanted(&state) { 1.0 } else { 0.0 };
        state.number = Tween::new(number, Duration::Micro, Easing::Standard);
        tree::State::new(state)
    }

    fn diff(&self, tree: &mut Tree) {
        let now = Instant::now();
        let state = tree.state.downcast_mut::<State>();
        if state.drag.is_none() {
            state.shown.to(self.slider.value.fraction(), now);
        }
        self.refresh(state, now);
    }

    fn size(&self) -> IcedSize<Length> {
        IcedSize::new(Length::Fill, Length::Fixed(self.slider.size.height()))
    }

    fn layout(&mut self, tree: &mut Tree, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let style = self.style();
        let state = tree.state.downcast_mut::<State>();
        let texts = self.label_texts();
        let same = state.labels.len() == texts.len() && state.labels.iter().zip(&texts).all(|((a, _), b)| a == b);
        if !same {
            state.labels = texts.into_iter().map(|t| (t.clone(), shaped(&t, style))).collect();
        }
        if state.widest.0 != self.slider.widest {
            let width = shaped(&self.slider.widest, style).min_width();
            state.widest = (self.slider.widest.clone(), width);
        }
        layout::Node::new(limits.resolve(Length::Fill, Length::Fixed(self.slider.size.height()), IcedSize::ZERO))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let now = Instant::now();
        let state = tree.state.downcast_mut::<State>();
        match event {
            Event::Window(iced::window::Event::RedrawRequested(at)) => {
                self.refresh(state, *at);
                if Self::moving(state, *at) {
                    shell.request_redraw();
                }
                return;
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let over = cursor.is_over(bounds);
                let over_icon = over && self.on_icon(Point::new(position.x - bounds.x, position.y - bounds.y));
                if over != state.hovered || over_icon != state.over_icon {
                    state.hovered = over;
                    state.over_icon = over_icon;
                    shell.request_redraw();
                }
                if state.drag.is_some() {
                    let fraction = self.fraction_at(bounds.width, position.x - bounds.x);
                    state.drag = Some(fraction);
                    if !self.stepped()
                        && let Some(on_change) = &self.slider.on_change
                    {
                        shell.publish(on_change(Value::Continuous(fraction)));
                    }
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if self.enabled() => {
                let Some(at) = cursor.position_in(bounds) else {
                    return;
                };
                if self.icon_presses() && self.on_icon(at) {
                    state.pressing_icon = true;
                } else {
                    let fraction = self.fraction_at(bounds.width, at.x);
                    state.drag = Some(fraction);
                    if !self.stepped()
                        && let Some(on_change) = &self.slider.on_change
                    {
                        shell.publish(on_change(Value::Continuous(fraction)));
                    }
                }
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if std::mem::take(&mut state.pressing_icon) {
                    let on_icon = cursor.position_in(bounds).is_some_and(|at| self.on_icon(at));
                    if on_icon {
                        if self.stepped() {
                            let (next, direction) = self.slider.value.turn(state.direction);
                            state.direction = direction;
                            if let Some(on_change) = &self.slider.on_change {
                                shell.publish(on_change(next));
                            }
                        } else if let Some(message) = &self.slider.on_icon {
                            shell.publish(message.clone());
                        }
                    }
                    shell.request_redraw();
                }
                if let Some(fraction) = state.drag.take() {
                    state.shown = Tween::new(fraction, Duration::Short, self.easing());
                    if self.stepped() {
                        let settled = self.slider.value.at(fraction);
                        state.shown.to(settled.fraction(), now);
                        if let Some(on_change) = &self.slider.on_change {
                            shell.publish(on_change(settled));
                        }
                    }
                    shell.request_redraw();
                }
            }
            _ => return,
        }
        self.refresh(state, now);
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.enabled() && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
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
        let state = tree.state.downcast_ref::<State>();
        let now = Instant::now();
        let bounds = layout.bounds();
        let paint = self.paint(state, now, bounds);
        self.draw_track(renderer, bounds, &paint);
        self.draw_marks(renderer, state, now, bounds, &paint);
        self.draw_icon(renderer, state, bounds, &paint, viewport);
        self.draw_label(renderer, state, now, bounds, &paint, viewport);
    }
}
