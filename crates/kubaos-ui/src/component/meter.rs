//! Meters and sparklines: readings at a glance, as the system monitor shows them.
//!
//! A meter is a reading split into parts (memory as used and cache, a disk as used), on a progress
//! bar's track. A sparkline is the last minute of a reading as a line, filled under it.
//!
//! # Owns
//!
//! - [`meter`] and [`sparkline`], built up with [`Sparkline`].
//!
//! # Not here
//!
//! - Work that knows how far it is: [`crate::component::progress`].

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Length;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Size as IcedSize;
use iced::Theme;
use iced::advanced::Layout;
use iced::advanced::Renderer as _;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::renderer;
use iced::advanced::widget::Tree;
use iced::mouse;
use iced::widget::canvas;
use iced::widget::canvas::Path;
use iced::widget::canvas::Stroke;
use iced::widget::canvas::stroke;

use crate::color;
use crate::layout::Extent;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::scale::Scale;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Size;

/// A bar 8 tall split into `parts`, each a share from 0 to 1 and its colour, left to right; what
/// they leave is the track. Every part keeps the bar's rounded ends.
pub fn meter<'a, M: 'a>(look: &Look, parts: &[(f32, Color)]) -> Element<'a, M> {
    let mut end = 0.0_f32;
    let mut reaches: Vec<(f32, Color)> = parts
        .iter()
        .map(|&(share, colour)| {
            end = (end + share.max(0.0)).min(1.0);
            (end, colour)
        })
        .collect();
    // The last part first, so each one's rounded end shows over the one after it.
    reaches.reverse();
    let bar = Meter {
        reaches,
        track: color::alpha(look.colors.surface_container_highest, Opacity::Track.value()),
        scale: look.scale,
    };
    probe::component("Meter", ["", "", ""], Fixed::HEIGHT, Element::new(bar))
}

struct Meter {
    /// Where each part ends, from the bar's left edge, and its colour: the last part first.
    reaches: Vec<(f32, Color)>,
    track: Color,
    scale: Scale,
}

impl<M> Widget<M, Theme, Renderer> for Meter {
    fn size(&self) -> IcedSize<Length> {
        IcedSize::new(Length::Fill, Length::Fixed(f32::from(Size::Progress.px())))
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.resolve(
            Length::Fill,
            Length::Fixed(f32::from(Size::Progress.px())),
            IcedSize::ZERO,
        ))
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let pill = Border::default().rounded(radius(Radius::Full));
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: pill,
                snap: true,
                ..renderer::Quad::default()
            },
            Background::Color(self.track),
        );
        for &(reach, colour) in &self.reaches {
            let width = self.scale.snap_up(bounds.width * reach).min(bounds.width);
            if width <= 0.0 {
                continue;
            }
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle { width, ..bounds },
                    border: pill,
                    snap: true,
                    ..renderer::Quad::default()
                },
                Background::Color(colour),
            );
        }
    }
}

/// A sparkline of `samples`, oldest first, evenly spaced across, the newest at the right edge,
/// built up then drawn with [`Sparkline::view`].
#[derive(Debug, Clone)]
pub struct Sparkline {
    samples: Vec<f32>,
    second: Vec<f32>,
    max: f32,
    colour: Option<Color>,
    height: Extent,
}

/// The last minute of a reading as a line in the accent, filled under it with the accent's tint,
/// on a scale from 0 to 100, as wide as its room and [`Size::Sparkline`] tall.
#[must_use]
pub fn sparkline(samples: Vec<f32>) -> Sparkline {
    Sparkline {
        samples,
        second: Vec::new(),
        max: DEFAULT_MAX,
        colour: None,
        height: Extent::Fixed(Size::Sparkline),
    }
}

impl Sparkline {
    /// A second reading over the first, unfilled, in the tertiary colour: upload over download.
    #[must_use]
    pub fn second(self, second: Vec<f32>) -> Self {
        Sparkline { second, ..self }
    }

    /// The top of the scale, or 0 to fit the highest sample with room above it.
    #[must_use]
    pub fn max(self, max: f32) -> Self {
        Sparkline { max, ..self }
    }

    /// The line's colour in place of the accent: a reading near its limit, in the warning colour.
    #[must_use]
    pub fn colour(self, colour: Color) -> Self {
        Sparkline {
            colour: Some(colour),
            ..self
        }
    }

    #[must_use]
    pub fn height(self, height: Extent) -> Self {
        Sparkline { height, ..self }
    }

    pub fn view<'a, M: 'a>(self, look: &Look) -> Element<'a, M> {
        let c = look.colors;
        let top = if self.max > 0.0 {
            self.max
        } else {
            // Fitted, the highest sample sits a little under the top, so the line never runs
            // along the graph's edge.
            let highest = self.samples.iter().chain(&self.second).copied().fold(1.0_f32, f32::max);
            highest * FIT_HEADROOM
        };
        let graph = Graph {
            samples: self.samples,
            second: self.second,
            top,
            colour: self.colour.unwrap_or(c.primary),
            second_colour: c.tertiary,
            line: look.scale.thin(),
        };
        let drawn = canvas(graph).width(Length::Fill).height(self.height.length());
        probe::component("Sparkline", ["", "", ""], Fixed::HEIGHT, drawn)
    }
}

/// The top of a sparkline's scale unless set: a percentage's.
pub const DEFAULT_MAX: f32 = 100.0;

/// How far above the highest sample a fitted scale's top is.
const FIT_HEADROOM: f32 = 1.15;

struct Graph {
    samples: Vec<f32>,
    second: Vec<f32>,
    top: f32,
    colour: Color,
    second_colour: Color,
    /// The thin stroke, whole device pixels at the scale.
    line: f32,
}

impl Graph {
    /// `series` as a line across `bounds`, the oldest sample at the left edge.
    fn path(&self, series: &[f32], size: IcedSize, closed: bool) -> Path {
        let step = size.width / (series.len() - 1) as f32;
        let at = |i: usize, v: f32| {
            let share = (v / self.top).clamp(0.0, 1.0);
            Point::new(
                i as f32 * step,
                size.height - self.line / 2.0 - share * (size.height - self.line),
            )
        };
        Path::new(|b| {
            for (i, &v) in series.iter().enumerate() {
                if i == 0 {
                    b.move_to(at(i, v));
                } else {
                    b.line_to(at(i, v));
                }
            }
            if closed {
                b.line_to(Point::new(size.width, size.height));
                b.line_to(Point::new(0.0, size.height));
                b.close();
            }
        })
    }

    fn draw_series(&self, frame: &mut canvas::Frame, series: &[f32], colour: Color, filled: bool) {
        if series.len() < 2 {
            return;
        }
        let size = frame.size();
        if filled {
            frame.fill(
                &self.path(series, size, true),
                color::alpha(colour, Opacity::Tint.value()),
            );
        }
        frame.stroke(
            &self.path(series, size, false),
            Stroke::default()
                .with_color(colour)
                .with_width(self.line)
                .with_line_join(stroke::LineJoin::Round),
        );
    }
}

impl<M> canvas::Program<M> for Graph {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        self.draw_series(&mut frame, &self.samples, self.colour, true);
        self.draw_series(&mut frame, &self.second, self.second_colour, false);
        vec![frame.into_geometry()]
    }
}
