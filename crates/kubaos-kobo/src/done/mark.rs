//! The クバ mark as neon tubes: its strokes in the order they are written, flattened to
//! polylines in the sign-off's logical pixels, so a stroke can be drawn part of the way.
//!
//! # Owns
//!
//! - The strokes' paths, order and tones, their widths, and the glow's band and blur around them.
//!
//! # Not here
//!
//! - Where each stroke is at a moment: `super::Sequence`. The glow baked around them: `super::art`.

use std::sync::LazyLock;

use iced::Color;
use iced::Point;
use iced::widget::canvas::Path;
use kubaos_ui::color::Colors;
use kubaos_ui::tokens::Size;

/// A tube's colour family: the warm amber of ク and バ, or the cold blue of the dakuten.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Warm,
    Cold,
}

impl Tone {
    /// The tube's colour and its white-hot core's.
    pub fn colors(self, c: &Colors) -> (Color, Color) {
        match self {
            Tone::Warm => (c.primary, c.primary_core),
            Tone::Cold => (c.cold, c.cold_core),
        }
    }

    /// The tube's width in the drawing's units: the logo's (`branding/make.py`), the dakuten a
    /// fifth thinner.
    fn width(self) -> f32 {
        match self {
            Tone::Warm => 9.5,
            Tone::Cold => 7.6,
        }
    }
}

/// The strokes in the order a hand writes them: ク's tick and sweep, バ's two strokes, then the
/// dakuten. The path data is `branding/make.py`'s, in its 120 unit drawing.
const STROKES: [(&str, Tone); 6] = [
    ("M29 13 L17 33", Tone::Warm),
    ("M24 24 H47 Q51 24 49.6 28 L46 39 Q38 66 14 83", Tone::Warm),
    ("M66 41 Q64.5 63 53 81", Tone::Warm),
    ("M76.5 39 Q84 61 94 79", Tone::Warm),
    ("M81 14 L85 25", Tone::Cold),
    ("M90 11 L94 22", Tone::Cold),
];

/// How many strokes the mark has.
pub const COUNT: usize = STROKES.len();

/// The logo's drawing is 120 units square, its strokes inside a 10 unit margin.
const UNITS: f32 = 120.0;
const MARGIN: f32 = 10.0;
/// The glow is a band this many times the tube's width, blurred by [`GLOW_SIGMA`] units: the
/// draft Kuba picked.
const GLOW_WIDTH: f32 = 1.6;
const GLOW_SIGMA: f32 = 6.0;
/// The core's width, of the tube's: the logo's.
const CORE: f32 = 0.22;
/// Line pieces per quadratic curve: under a tenth of a pixel from the curve at 2×.
const CURVE_PIECES: u16 = 24;

/// Logical pixels per unit of the drawing.
pub fn unit() -> f32 {
    f32::from(Size::NeonMark.px()) / UNITS
}

/// One stroke of the mark, as a polyline in logical pixels from the sign-off's top left.
#[derive(Debug, Clone)]
pub struct Stroke {
    pub tone: Tone,
    points: Vec<Point>,
    /// The distance along the stroke to each point.
    along: Vec<f32>,
}

impl Stroke {
    /// The tube's width in logical pixels.
    pub fn width(&self) -> f32 {
        self.tone.width() * unit()
    }

    /// The core's width in logical pixels.
    pub fn core_width(&self) -> f32 {
        self.width() * CORE
    }

    /// The half width of the glow's band, and the blur over it, in logical pixels.
    pub fn glow(&self) -> (f32, f32) {
        (self.width() * GLOW_WIDTH / 2.0, GLOW_SIGMA * unit())
    }

    pub fn length(&self) -> f32 {
        self.along.last().copied().unwrap_or(0.0)
    }

    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// The stroke written `drawn` of the way, 0 to 1, from its start; nothing at 0.
    pub fn path(&self, drawn: f32) -> Option<Path> {
        let reach = self.length() * drawn.clamp(0.0, 1.0);
        if reach <= 0.0 {
            return None;
        }
        Some(Path::new(|b| {
            b.move_to(self.points[0]);
            for (i, &point) in self.points.iter().enumerate().skip(1) {
                if self.along[i] <= reach {
                    b.line_to(point);
                    continue;
                }
                let from = self.points[i - 1];
                let t = (reach - self.along[i - 1]) / (self.along[i] - self.along[i - 1]);
                b.line_to(from + (point - from) * t);
                break;
            }
        }))
    }
}

/// Every stroke, in writing order.
pub fn strokes() -> &'static [Stroke] {
    static ALL: LazyLock<Vec<Stroke>> = LazyLock::new(|| {
        STROKES
            .iter()
            .map(|&(data, tone)| {
                let points = flatten(data);
                let mut along = vec![0.0];
                for pair in points.windows(2) {
                    along.push(along[along.len() - 1] + pair[0].distance(pair[1]));
                }
                Stroke { tone, points, along }
            })
            .collect()
    });
    &ALL
}

/// The centre of the strokes' bounds, in logical pixels: where the glow behind the mark sits.
pub fn centre() -> Point {
    let (mut lo, mut hi) = (Point::new(f32::MAX, f32::MAX), Point::new(f32::MIN, f32::MIN));
    for p in strokes().iter().flat_map(Stroke::points) {
        lo = Point::new(lo.x.min(p.x), lo.y.min(p.y));
        hi = Point::new(hi.x.max(p.x), hi.y.max(p.y));
    }
    Point::new(f32::midpoint(lo.x, hi.x), f32::midpoint(lo.y, hi.y))
}

/// The polyline of an SVG path made of absolute `M`, `L`, `H` and `Q` commands, the only ones the
/// logo's strokes use, in logical pixels. Each command letter starts a word.
fn flatten(data: &str) -> Vec<Point> {
    let to_px = |(x, y): (f32, f32)| Point::new((x + MARGIN) * unit(), (y + MARGIN) * unit());
    let mut words = data.split_whitespace();
    let mut out = Vec::new();
    let mut at = (0.0, 0.0);
    while let Some(word) = words.next() {
        let (command, first) = word.split_at(1);
        let mut numbers = std::iter::once(first).chain(&mut words).map(|w| {
            w.parse::<f32>()
                .expect("the mark's path data is numbers after each command")
        });
        let mut next = || {
            numbers
                .next()
                .expect("the mark's path data has every number a command takes")
        };
        match command {
            "M" | "L" => {
                at = (next(), next());
                out.push(to_px(at));
            }
            "H" => {
                at.0 = next();
                out.push(to_px(at));
            }
            "Q" => {
                let (control, end) = ((next(), next()), (next(), next()));
                let pieces = f32::from(CURVE_PIECES);
                out.extend((1..=CURVE_PIECES).map(|i| to_px(quadratic(at, control, end, f32::from(i) / pieces))));
                at = end;
            }
            other => panic!("the mark's path data uses only M, L, H and Q, not {other}"),
        }
    }
    out
}

/// The quadratic Bézier from `a` through `control` to `b`, `t` of the way.
fn quadratic(a: (f32, f32), control: (f32, f32), b: (f32, f32), t: f32) -> (f32, f32) {
    let u = 1.0 - t;
    let at = |a: f32, c: f32, b: f32| u * u * a + 2.0 * u * t * c + t * t * b;
    (at(a.0, control.0, b.0), at(a.1, control.1, b.1))
}
