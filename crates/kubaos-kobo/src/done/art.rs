//! The sign-off's pictures, baked once per palette and scale: each tube's glow, the glow behind
//! the mark on impact, and the 完了 hanko with its ink.
//!
//! The renderers have no blur and no radial gradient, and a glow drawn as strokes would be
//! tessellated again on every frame of the hum. Baked, the hum only changes an image's opacity.
//! Every picture is at device pixels, and the ones that stay on screen sit on whole device pixels,
//! so nothing is resampled once the sequence has settled.
//!
//! # Owns
//!
//! - The bakes ([`art`]), done once per palette and scale on a thread of their own, and kept.
//! - How the glow falls off, the halo's shape, and the hanko's border, characters and ink.
//!
//! # Not here
//!
//! - When each picture shows, and how strongly: `super::Sequence`. The tubes: `super::mark`.

use std::collections::HashMap;
use std::f32::consts::FRAC_1_SQRT_2;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::OnceLock;

use iced::Color;
use iced::Pixels;
use iced::Point;
use iced::Radians;
use iced::Rectangle;
use iced::Size as Area;
use iced::Vector;
use iced::alignment;
use iced::widget::canvas;
use iced::widget::canvas::path::lyon_path;
use iced::widget::image;
use iced::widget::text::LineHeight;
use kubaos_ui::Look;
use kubaos_ui::look::nested;
use kubaos_ui::look::radius;
use kubaos_ui::text::family_font;
use kubaos_ui::tokens::Family;
use kubaos_ui::tokens::Radius;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;
use kubaos_ui::tokens::Weight;

use super::mark;
use super::mark::Stroke;
use crate::theme::jp;

/// A baked picture and where it goes, in the sign-off's logical pixels.
#[derive(Debug, Clone)]
pub struct Picture {
    pub handle: image::Handle,
    pub bounds: Rectangle,
}

/// Every picture of the sign-off, for one palette at one scale.
#[derive(Debug)]
pub struct Art {
    /// Each stroke's glow, in writing order.
    pub glows: Vec<Picture>,
    /// The glow thrown behind the mark on impact.
    pub halo: Picture,
    /// The hanko as it lands: turned, at device pixels, on whole device pixels.
    pub hanko: Picture,
    /// The hanko upright and at its largest, for the way down; its bounds are its box at rest.
    pub stamp: Picture,
}

/// How far the hanko leans once it has landed.
pub const LEAN: Radians = Radians(-8.0 * std::f32::consts::PI / 180.0);
/// How large the hanko starts, of its size at rest.
pub const STAMP_FROM: f32 = 1.9;

/// The halo's opacity at its middle: soft, so it lights the wall without hiding the words.
const HALO_PEAK: f32 = 0.45;
/// Room around the hanko's box for its rough edge.
const INK_ROOM: Space = Space::S4;
/// The ink's edge wanders up to this many logical pixels either way, over features about two
/// pixels across: the draft's displacement of the stamp's outline.
const ROUGH_REACH: f32 = 2.0;
const ROUGH_FREQUENCY: f32 = 0.6;

/// What a bake depends on: the colours it draws in, and the scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    colours: [[u8; 4]; 3],
    /// Device pixels per logical pixel, in thousandths.
    scale: u32,
}

/// The colours a bake draws in: the tubes' glows, and the hanko's ink.
#[derive(Debug, Clone, Copy)]
struct Inks {
    primary: Color,
    cold: Color,
    ink: Color,
}

impl Inks {
    fn of(look: &Look) -> Inks {
        let c = look.colors;
        Inks {
            primary: c.primary,
            cold: c.cold,
            ink: c.primary_container,
        }
    }

    fn key(self, scale: f32) -> Key {
        Key {
            colours: [self.primary, self.cold, self.ink].map(Color::into_rgba8),
            scale: (scale * 1000.0).round() as u32,
        }
    }
}

type Slot = Arc<OnceLock<Arc<Art>>>;

/// Bakes started or done. A few at most: Kōbō draws one palette at one scale, the shots eight.
static BAKED: LazyLock<Mutex<HashMap<Key, Slot>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
const KEPT: usize = 8;

/// The sign-off's pictures for `look`. The first ask starts the bake on a thread of its own and
/// returns `None` until it is done; with `wait`, it returns them, baking them here if no thread
/// has yet.
pub fn art(look: &Look, wait: bool) -> Option<Arc<Art>> {
    let (inks, scale) = (Inks::of(look), look.scale.factor());
    let key = inks.key(scale);
    let (slot, fresh) = {
        let mut baked = BAKED.lock().expect("the sign-off's bake list poisoned");
        // A bake still running keeps its slot, so whoever waits on it gets it and none runs twice.
        if baked.len() >= KEPT && !baked.contains_key(&key) {
            baked.retain(|_, slot| slot.get().is_none());
        }
        let fresh = !baked.contains_key(&key);
        (Arc::clone(baked.entry(key).or_default()), fresh)
    };
    if wait {
        return Some(Arc::clone(slot.get_or_init(|| Arc::new(bake(inks, scale)))));
    }
    if fresh {
        let baking = Arc::clone(&slot);
        std::thread::spawn(move || {
            baking.get_or_init(|| Arc::new(bake(inks, scale)));
        });
    }
    slot.get().cloned()
}

fn bake(inks: Inks, scale: f32) -> Art {
    let glows = mark::strokes()
        .iter()
        .map(|stroke| {
            let colour = match stroke.tone {
                mark::Tone::Warm => inks.primary,
                mark::Tone::Cold => inks.cold,
            };
            glow(stroke, colour, scale)
        })
        .collect();
    Art {
        glows,
        halo: halo(inks.primary, scale),
        hanko: hanko(inks.ink, scale, LEAN),
        stamp: hanko(inks.ink, scale * STAMP_FROM, Radians(0.0)),
    }
}

/// The sign-off's hanko: where its box sits beside the mark, in logical pixels.
pub fn hanko_box() -> Rectangle {
    let mark = f32::from(Size::NeonMark.px());
    let side = f32::from(Size::Hanko.px());
    Rectangle::new(
        Point::new(mark + f32::from(Space::S32.px()), (mark - side) / 2.0),
        Area::new(side, side),
    )
}

/// The device pixels around `bounds` (logical) at `scale`, as whole pixels: their origin and size.
fn device_box(bounds: Rectangle, scale: f32) -> (i32, i32, u32, u32) {
    let (x0, y0) = ((bounds.x * scale).floor(), (bounds.y * scale).floor());
    let (x1, y1) = (
        ((bounds.x + bounds.width) * scale).ceil(),
        ((bounds.y + bounds.height) * scale).ceil(),
    );
    (x0 as i32, y0 as i32, (x1 - x0) as u32, (y1 - y0) as u32)
}

/// A picture of `colour` at the alpha `coverage` gives each device pixel of `bounds`, at `scale`.
/// Each pixel's centre is passed in logical pixels. Alpha is dithered, so a soft falloff doesn't
/// band.
fn paint(bounds: Rectangle, scale: f32, colour: Color, coverage: impl Fn(Point) -> f32) -> Picture {
    let (left, top, width, height) = device_box(bounds, scale);
    let [red, green, blue, _] = colour.into_rgba8();
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for row in 0..height {
        for column in 0..width {
            let (x, y) = (left + column as i32, top + row as i32);
            // design-allow(size): a device pixel's centre is half a pixel into it, not a length.
            let centre = Point::new((x as f32 + 0.5) / scale, (y as f32 + 0.5) / scale);
            let alpha = coverage(centre).clamp(0.0, 1.0) * 255.0 + dither(x, y);
            rgba.extend([red, green, blue, alpha.round().clamp(0.0, 255.0) as u8]);
        }
    }
    Picture {
        handle: image::Handle::from_rgba(width, height, rgba),
        bounds: Rectangle::new(
            Point::new(left as f32 / scale, top as f32 / scale),
            Area::new(width as f32 / scale, height as f32 / scale),
        ),
    }
}

/// A stroke's glow: its tube widened into a band and blurred, as an SVG Gaussian blur draws it.
fn glow(stroke: &Stroke, colour: Color, scale: f32) -> Picture {
    let (half, sigma) = stroke.glow();
    let reach = half + 3.0 * sigma;
    let points = stroke.points();
    let (mut lo, mut hi) = (points[0], points[0]);
    for p in points {
        lo = Point::new(lo.x.min(p.x), lo.y.min(p.y));
        hi = Point::new(hi.x.max(p.x), hi.y.max(p.y));
    }
    let bounds = Rectangle::new(
        lo - Vector::new(reach, reach),
        Area::new(hi.x - lo.x + 2.0 * reach, hi.y - lo.y + 2.0 * reach),
    );
    paint(bounds, scale, colour, |p| band(distance(p, points), half, sigma))
}

/// The halo behind the mark: a radial falloff as wide as the mark, flat at its middle and
/// reaching nothing at its rim.
fn halo(colour: Color, scale: f32) -> Picture {
    let centre = mark::centre();
    let r = f32::from(Size::NeonMark.px());
    let bounds = Rectangle::new(centre - Vector::new(r, r), Area::new(2.0 * r, 2.0 * r));
    paint(bounds, scale, colour, |p| {
        let x = (p.distance(centre) / r).min(1.0);
        HALO_PEAK * (1.0 - x * x).powi(3)
    })
}

/// The hanko at `out` device pixels per logical pixel, turned by `lean`: its border and 完了 in
/// `ink`, with a rough edge and the ink's grain. Its bounds are its box at rest whatever `out` is.
fn hanko(ink: Color, out: f32, lean: Radians) -> Picture {
    let at = hanko_box();
    let half = at.width / 2.0 + f32::from(INK_ROOM.px());
    let centre = at.center();
    let source = Coverage::new(half, out * 2.0);
    let (sin, cos) = lean.0.sin_cos();
    let reach = half * (cos.abs() + sin.abs());
    let bounds = Rectangle::new(centre - Vector::new(reach, reach), Area::new(2.0 * reach, 2.0 * reach));
    paint(bounds, out, ink, |p| {
        let v = p - centre;
        // The pixel in the hanko's own frame: turned back by its lean, nudged by the ink's edge.
        let local = Vector::new(v.x * cos + v.y * sin, -v.x * sin + v.y * cos);
        let rough = Vector::new(
            fbm(local, ROUGH_FREQUENCY, 11) - 0.5,
            fbm(local, ROUGH_FREQUENCY, 23) - 0.5,
        );
        source.at(local + rough * ROUGH_REACH) * grain(local)
    })
}

/// How much of the ink takes at a point of the hanko, in its own frame: patchy over a few pixels,
/// with specks where none took, and a slow unevenness over the whole.
fn grain(p: Vector) -> f32 {
    let patchy = 1.0 - 0.45 * smoothstep(0.6, 0.85, fbm(p, 0.25, 3));
    let specks = 1.0 - 0.7 * smoothstep(0.78, 0.95, fbm(p, 0.9, 7));
    let uneven = 0.86 + 0.14 * fbm(p, 0.04, 5);
    patchy * specks * uneven
}

/// The hanko's border and characters as coverage, rasterised upright at `density` pixels per
/// logical pixel, `half` logical pixels from its centre to each edge.
struct Coverage {
    alpha: Vec<u8>,
    side: u32,
    density: f32,
    half: f32,
}

impl Coverage {
    fn new(half: f32, density: f32) -> Coverage {
        let side = (2.0 * half * density).ceil() as u32;
        let mut pixmap = tiny_skia::Pixmap::new(side, side).expect("the hanko's pixmap has a size");
        let to_px = tiny_skia::Transform::from_scale(density, density).pre_translate(half, half);
        let mut paint = tiny_skia::Paint::default();
        paint.set_color_rgba8(0, 0, 0, 255);
        let box_half = f32::from(Size::Hanko.px()) / 2.0;
        let border = rounded_square(box_half, radius(Radius::Control));
        let inset = f32::from(Space::S4.px());
        let inner = rounded_square(box_half - inset, radius(nested(Radius::Control.corner(), Space::S4)));
        let mut frame = tiny_skia::PathBuilder::new();
        frame.push_path(&border);
        frame.push_path(&inner);
        let frame = frame.finish().expect("the hanko's border is a path");
        pixmap.fill_path(&frame, &paint, tiny_skia::FillRule::EvenOdd, to_px, None);
        let glyphs = characters();
        pixmap.fill_path(&glyphs, &paint, tiny_skia::FillRule::Winding, to_px, None);
        // M PLUS 1 ships here in one weight; a hanko's carved characters are heavier, so the
        // outline is inked too.
        let heavier = tiny_skia::Stroke {
            width: f32::from(Space::S4.px()) / 2.0,
            line_join: tiny_skia::LineJoin::Round,
            ..tiny_skia::Stroke::default()
        };
        pixmap.stroke_path(&glyphs, &paint, &heavier, to_px, None);
        Coverage {
            alpha: pixmap.pixels().iter().map(|p| p.alpha()).collect(),
            side,
            density,
            half,
        }
    }

    /// The coverage at `p`, logical pixels from the centre, interpolated between pixels.
    fn at(&self, p: Vector) -> f32 {
        let x = (p.x + self.half) * self.density - 0.5;
        let y = (p.y + self.half) * self.density - 0.5;
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let px = |x: f32, y: f32| -> f32 {
            if x < 0.0 || y < 0.0 || x >= self.side as f32 || y >= self.side as f32 {
                return 0.0;
            }
            f32::from(self.alpha[(y as u32 * self.side + x as u32) as usize]) / 255.0
        };
        let top = px(x0, y0) * (1.0 - fx) + px(x0 + 1.0, y0) * fx;
        let bottom = px(x0, y0 + 1.0) * (1.0 - fx) + px(x0 + 1.0, y0 + 1.0) * fx;
        top * (1.0 - fy) + bottom * fy
    }
}

/// 完了 set top to bottom, as a hanko is carved: each character at the display size, a small
/// gap apart, centred on the hanko.
fn characters() -> tiny_skia::Path {
    let size = f32::from(Type::Display.size());
    let pitch = size + f32::from(Space::S4.px());
    let mut out = tiny_skia::PathBuilder::new();
    for (i, glyph) in jp::DONE.chars().enumerate() {
        let offset = (i as f32 - 0.5) * pitch;
        let text = canvas::Text {
            content: glyph.to_string(),
            position: Point::new(0.0, offset),
            size: Pixels(size),
            line_height: LineHeight::Absolute(Pixels(size)),
            font: family_font(Family::Jp, Weight::Medium),
            align_x: iced::widget::text::Alignment::Center,
            align_y: alignment::Vertical::Center,
            ..canvas::Text::default()
        };
        text.draw_with(|path, _| push_lyon(&mut out, path.raw()));
    }
    out.finish().expect("完了 has outlines in M PLUS 1")
}

/// `path` added to `out`.
fn push_lyon(out: &mut tiny_skia::PathBuilder, path: &lyon_path::Path) {
    for event in path {
        match event {
            lyon_path::Event::Begin { at } => out.move_to(at.x, at.y),
            lyon_path::Event::Line { to, .. } => out.line_to(to.x, to.y),
            lyon_path::Event::Quadratic { ctrl, to, .. } => out.quad_to(ctrl.x, ctrl.y, to.x, to.y),
            lyon_path::Event::Cubic { ctrl1, ctrl2, to, .. } => {
                out.cubic_to(ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y);
            }
            lyon_path::Event::End { close, .. } => {
                if close {
                    out.close();
                }
            }
        }
    }
}

/// A square `half` from its centre to each edge, its corners rounded by `r`.
fn rounded_square(half: f32, r: f32) -> tiny_skia::Path {
    // The control points' distance along a tangent for a quarter circle drawn as one cubic.
    const KAPPA: f32 = 0.552_284_8;
    let (a, k) = (half, r * (1.0 - KAPPA));
    let mut pb = tiny_skia::PathBuilder::new();
    pb.move_to(-a + r, -a);
    pb.line_to(a - r, -a);
    pb.cubic_to(a - k, -a, a, -a + k, a, -a + r);
    pb.line_to(a, a - r);
    pb.cubic_to(a, a - k, a - k, a, a - r, a);
    pb.line_to(-a + r, a);
    pb.cubic_to(-a + k, a, -a, a - k, -a, a - r);
    pb.line_to(-a, -a + r);
    pb.cubic_to(-a, -a + k, -a + k, -a, -a + r, -a);
    pb.close();
    pb.finish().expect("a rounded square is a path")
}

/// How much of a band `half` wide on either side of a line, blurred by a Gaussian of `sigma`,
/// reaches `d` from the line: exact for a straight band, and close round the tube's bends.
fn band(d: f32, half: f32, sigma: f32) -> f32 {
    let k = FRAC_1_SQRT_2 / sigma;
    0.5 * (erf((half - d) * k) + erf((half + d) * k))
}

/// The distance from `p` to the polyline through `points`.
fn distance(p: Point, points: &[Point]) -> f32 {
    points
        .windows(2)
        .map(|pair| {
            let (a, b) = (pair[0], pair[1]);
            let (ab, ap) = (b - a, p - a);
            let len = ab.x * ab.x + ab.y * ab.y;
            let t = if len > 0.0 {
                ((ap.x * ab.x + ap.y * ab.y) / len).clamp(0.0, 1.0)
            } else {
                0.0
            };
            p.distance(a + ab * t)
        })
        .fold(f32::MAX, f32::min)
}

/// The error function, to within 1.5e-7 (Abramowitz and Stegun, 7.1.26).
fn erf(x: f32) -> f32 {
    let t = 1.0 / (1.0 + 0.327_591_1 * x.abs());
    let poly = t * (0.254_829_6 + t * (-0.284_496_7 + t * (1.421_413_7 + t * (-1.453_152 + t * 1.061_405_4))));
    let y = 1.0 - poly * (-x * x).exp();
    y.copysign(x)
}

fn smoothstep(lo: f32, hi: f32, x: f32) -> f32 {
    let t = ((x - lo) / (hi - lo)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Two octaves of value noise at `p`, `frequency` cells per logical pixel, 0 to 1. The same seed
/// gives the same texture every time: the ink is baked, not random.
fn fbm(p: Vector, frequency: f32, seed: u32) -> f32 {
    let q = p * frequency;
    (2.0 * value_noise(q.x, q.y, seed) + value_noise(2.0 * q.x, 2.0 * q.y, seed + 1)) / 3.0
}

fn value_noise(x: f32, y: f32, seed: u32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (ix, iy) = (x0 as i32, y0 as i32);
    let at = |dx: i32, dy: i32| unit_hash(ix + dx, iy + dy, seed);
    let top = at(0, 0) + (at(1, 0) - at(0, 0)) * sx;
    let bottom = at(0, 1) + (at(1, 1) - at(0, 1)) * sx;
    top + (bottom - top) * sy
}

/// A number from 0 to 1 for a lattice point: a 32-bit integer hash (Chris Wellons' `lowbias32`).
fn unit_hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut h =
        (x as u32).wrapping_mul(0x9e37_79b9) ^ (y as u32).wrapping_mul(0x85eb_ca6b) ^ seed.wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^= h >> 16;
    (h >> 8) as f32 / (1u32 << 24) as f32
}

/// Under half a level of noise for a device pixel, -0.5 to 0.5.
fn dither(x: i32, y: i32) -> f32 {
    unit_hash(x, y, 0x5eed) - 0.5
}

#[cfg(test)]
mod tests;
