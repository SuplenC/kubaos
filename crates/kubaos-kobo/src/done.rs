//! The done screen's sign-off: the クバ mark writes itself in as neon tubes that stutter on and
//! hum, the 完了 hanko stamps down beside it and the mark surges on impact, while the words
//! settle in under them. Everything is in place 3.4 s in, the glow behind the mark gone by 3.6 s,
//! and nothing waits for it: Reboot works from the first frame.
//!
//! ```text
//! 0.30 s  the strokes write in, 0.22 s apart, each stuttering on as it ends; then the hum
//! 2.00 s  the headline, a word every 0.09 s
//! 2.40 s  the lede
//! 2.60 s  the actions; the hanko comes down
//! 2.78 s  impact: the mark flares white-hot, the glow behind it fades, the page jolts
//! 2.90 s  the key hints
//! ```
//!
//! # Owns
//!
//! - [`Sequence`]: where every part is at a moment, and when the whole has settled.
//! - The sign-off ([`sign_off`]): two canvases, the glow under the tubes and the hanko; its strokes
//!   (`mark`) and its baked pictures (`art`).
//! - [`shift`], an element drawn off its place with its layout left where it is.
//!
//! # Not here
//!
//! - The rest of the screen, its copy and its actions: `crate::view`.

mod art;
mod mark;
mod shift;

use std::cell::Cell;
use std::cell::RefCell;
use std::f32::consts::TAU;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use iced::Color;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Size as Area;
use iced::Theme;
use iced::Vector;
use iced::mouse;
use iced::widget;
use iced::widget::canvas;
use iced::widget::canvas::Action;
use iced::widget::canvas::Cache;
use iced::widget::canvas::Frame;
use iced::widget::canvas::Geometry;
use iced::widget::canvas::LineCap;
use iced::widget::canvas::LineJoin;
use iced::widget::canvas::Stroke;
use iced::widget::image::FilterMethod;
use iced::window;
use kubaos_ui::Look;
use kubaos_ui::color;
use kubaos_ui::color::Colors;
use kubaos_ui::motion::Motion;
use kubaos_ui::motion::ease;
use kubaos_ui::tokens;
use kubaos_ui::tokens::Easing;
use kubaos_ui::tokens::Opacity;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;

use self::art::Art;
use self::art::Picture;
pub use self::shift::shift;
use crate::ui::El;

// The cues: when each part starts, from the screen's arrival, as the draft Kuba picked set them
// (docs/DESIGN.md, "Motion"). How long a part then moves, and on which curve, is a token.

/// The first stroke, and the gap to each next one.
// design-allow(duration): a cue, when the strokes start, not how long they move.
const STROKES_AT: Duration = Duration::from_millis(300);
// design-allow(duration): a cue, the stagger between strokes, not how long one moves.
const STROKE_GAP: Duration = Duration::from_millis(220);
/// The headline's first word, and the gap to each next one.
const HEADLINE_AT: Duration = Duration::from_secs(2);
// design-allow(duration): a cue, the stagger between words, not how long one moves.
const WORD_GAP: Duration = Duration::from_millis(90);
const LEDE_AT: Duration = Duration::from_millis(2400);
const ACTIONS_AT: Duration = Duration::from_millis(2600);
const STAMP_AT: Duration = Duration::from_millis(2600);
/// Impact: the hanko lands, the mark surges and the page jolts.
const IMPACT_AT: Duration = Duration::from_millis(2780);
const HINTS_AT: Duration = Duration::from_millis(2900);

/// One breath of the hum, longer than any motion token: a slow pulse, not a move.
const HUM_PERIOD: Duration = Duration::from_millis(2400);
/// The glow's opacity at the bottom and the top of the hum.
const HUM_LOW: f32 = 0.55;
const HUM_HIGH: f32 = 0.9;
/// The hum redraws when the glow at its brightest has changed by this many of 255 levels, and
/// not sooner than a frame: about 30 redraws a second on the GPU, fewer at the turns.
const HUM_LEVELS: f32 = 2.0;
/// The same in software, about 10 a second. There every redraw rasterises the whole wallpaper
/// under the damaged region again: about 10 ms, so 30 a second held a core at 28%.
const HUM_LEVELS_SOFTWARE: f32 = 6.0;
/// The brightest the glow picture gets, at the middle of a tube.
const GLOW_PEAK: f32 = 0.8;
/// A frame at 60 Hz, the soonest the hum asks for the next.
// design-allow(duration): a display's frame, not motion.
const FRAME: Duration = Duration::from_micros(16_667);
/// The longest the hum waits between redraws, near its turns.
// design-allow(duration): how long a redraw may wait, not motion.
const HUM_WAIT: Duration = Duration::from_millis(250);

/// A tube striking: its brightness through ten even steps over `medium`, the boot splash's
/// stutter (`system/files/plymouth/kubaos/kubaos.script`).
const STRIKE: [f32; 10] = [0.6, 0.0, 0.9, 0.2, 0.0, 1.0, 0.4, 1.0, 0.8, 1.0];

/// The page's jolt on impact, in logical pixels: the draft's, through `short`.
const JOLT: [(f32, f32); 5] = [(0.0, 0.0), (1.0, 2.0), (-1.0, -1.0), (1.0, 0.0), (0.0, 0.0)];

/// The tubes at their whitest, of the way to the core's colour, and the core as wide as this much
/// of its tube.
const FLARE_CORE: f32 = 0.6;

/// How far a part has arrived: its opacity, and how far below its place it still is, in logical
/// pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reveal {
    pub shown: f32,
    pub below: f32,
}

/// The hanko on its way down.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Stamp {
    /// Its size, of its size at rest.
    scale: f32,
    lean: iced::Radians,
    shown: f32,
    landed: bool,
}

/// Where the sign-off and the words are at a moment.
#[derive(Debug, Clone, Copy)]
pub struct Sequence {
    since: Duration,
    motion: Motion,
}

impl Sequence {
    /// The sequence `now`, for a screen that arrived at `arrived`.
    pub fn new(arrived: Instant, now: Instant, motion: Motion) -> Sequence {
        Sequence {
            since: now.saturating_duration_since(arrived),
            motion,
        }
    }

    /// Reduce motion shows every part in place from the start.
    fn still(&self) -> bool {
        self.motion.is_reduced()
    }

    /// `cue` at the motion's speed: the speed setting divides every duration, cues too.
    fn at(&self, cue: Duration) -> Duration {
        let scene = tokens::Duration::Scene;
        cue.mul_f32(self.motion.duration(scene).as_secs_f32() * 1000.0 / scene.ms() as f32)
    }

    /// The time since `cue`, or `None` before it.
    fn after(&self, cue: Duration) -> Option<Duration> {
        self.since.checked_sub(self.at(cue))
    }

    /// How far, 0 to 1, a part that starts at `cue` and takes `duration` on `easing` has come.
    fn progress(&self, cue: Duration, duration: tokens::Duration, easing: Easing) -> f32 {
        if self.still() {
            return 1.0;
        }
        self.after(cue)
            .map_or(0.0, |elapsed| self.motion.progress(duration, easing, elapsed))
    }

    /// When the last part has settled; only the hum goes on.
    fn settled_at(&self) -> Duration {
        let ends = [
            self.at(IMPACT_AT) + self.motion.duration(tokens::Duration::Slow),
            self.at(HINTS_AT) + self.motion.duration(tokens::Duration::Long),
            self.at(STAMP_AT) + self.motion.duration(tokens::Duration::Medium),
            self.at(IMPACT_AT) + self.motion.duration(tokens::Duration::Short),
        ];
        ends.into_iter().max().unwrap_or_default()
    }

    /// Whether anything but the hum still moves: frames are needed until it has settled.
    pub fn moving(&self) -> bool {
        !self.still() && self.since < self.settled_at()
    }

    fn rise(&self, cue: Duration, by: Space) -> Reveal {
        let shown = self.progress(cue, tokens::Duration::Long, Easing::EmphasizedDecel);
        Reveal {
            shown,
            below: (1.0 - shown) * f32::from(by.px()),
        }
    }

    /// The headline's `i`th word, rising into place.
    pub fn word(&self, i: u16) -> Reveal {
        self.rise(HEADLINE_AT + WORD_GAP * u32::from(i), Space::S24)
    }

    pub fn lede(&self) -> Reveal {
        self.rise(LEDE_AT, Space::S16)
    }

    pub fn actions(&self) -> Reveal {
        self.rise(ACTIONS_AT, Space::S16)
    }

    /// The key hints only fade in: they are the quietest line.
    pub fn hints(&self) -> Reveal {
        Reveal {
            shown: self.progress(HINTS_AT, tokens::Duration::Long, Easing::Standard),
            below: 0.0,
        }
    }

    /// How far the page is off its place in the jolt, in logical pixels.
    pub fn jolt(&self) -> Vector {
        let Some(elapsed) = self.after(IMPACT_AT).filter(|_| !self.still()) else {
            return Vector::ZERO;
        };
        let x = elapsed.as_secs_f32() / self.motion.duration(tokens::Duration::Short).as_secs_f32();
        if x >= 1.0 {
            return Vector::ZERO;
        }
        let steps = (JOLT.len() - 1) as f32;
        let i = ((x * steps) as usize).min(JOLT.len() - 2);
        let t = ease(Easing::Standard, x * steps - i as f32);
        let ((x0, y0), (x1, y1)) = (JOLT[i], JOLT[i + 1]);
        Vector::new(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t)
    }

    /// The unlit glass, fading in as the screen arrives.
    fn dead(&self) -> f32 {
        self.progress(Duration::ZERO, tokens::Duration::Long, Easing::EmphasizedDecel)
    }

    fn stroke_at(k: usize) -> Duration {
        STROKES_AT + STROKE_GAP * k as u32
    }

    /// How much of stroke `k` is written, 0 to 1.
    fn drawn(&self, k: usize) -> f32 {
        self.progress(Self::stroke_at(k), tokens::Duration::Medium, Easing::EmphasizedDecel)
    }

    /// How brightly stroke `k` is lit: dark, striking as its writing ends, then on.
    fn lit(&self, k: usize) -> f32 {
        if self.still() {
            return 1.0;
        }
        let Some(elapsed) = self.after(Self::stroke_at(k) + STROKE_GAP) else {
            return 0.0;
        };
        let x = elapsed.as_secs_f32() / self.motion.duration(tokens::Duration::Medium).as_secs_f32();
        STRIKE.get((x * STRIKE.len() as f32) as usize).copied().unwrap_or(1.0)
    }

    /// When the last tube has struck, and the hum starts.
    fn hum_at(&self) -> Duration {
        self.at(Self::stroke_at(mark::COUNT - 1) + STROKE_GAP) + self.motion.duration(tokens::Duration::Medium)
    }

    /// The glow's opacity in the hum: at its top until the hum starts, so it starts without a step.
    /// Reduce motion holds it still, between the two.
    fn hum(&self) -> f32 {
        let (mid, swing) = (f32::midpoint(HUM_LOW, HUM_HIGH), (HUM_HIGH - HUM_LOW) / 2.0);
        if self.still() {
            return mid;
        }
        let Some(elapsed) = self.since.checked_sub(self.hum_at()) else {
            return HUM_HIGH;
        };
        mid + swing * (TAU * elapsed.as_secs_f32() / HUM_PERIOD.as_secs_f32()).cos()
    }

    /// How long until the hum has changed enough to redraw: the glow at its brightest `levels`
    /// further on, at least a frame and at most [`HUM_WAIT`] away.
    fn next_hum(&self, levels: f32) -> Duration {
        let swing = (HUM_HIGH - HUM_LOW) / 2.0;
        let omega = TAU / HUM_PERIOD.as_secs_f32();
        let phase = self.since.saturating_sub(self.hum_at()).as_secs_f32() * omega;
        let rate = swing * omega * phase.sin().abs() * GLOW_PEAK * 255.0;
        let wait = if rate > 0.0 { levels / rate } else { f32::MAX };
        Duration::from_secs_f32(wait.min(HUM_WAIT.as_secs_f32())).max(FRAME)
    }

    /// The white-hot flare on impact, 1 to 0 over `micro`.
    fn flare(&self) -> f32 {
        if self.still() || self.after(IMPACT_AT).is_none() {
            return 0.0;
        }
        1.0 - self.progress(IMPACT_AT, tokens::Duration::Micro, Easing::StandardAccel)
    }

    /// The glow behind the mark on impact, 1 to 0 over `slow`.
    fn halo(&self) -> f32 {
        if self.still() || self.after(IMPACT_AT).is_none() {
            return 0.0;
        }
        1.0 - self.progress(IMPACT_AT, tokens::Duration::Slow, Easing::Standard)
    }

    /// The hanko: nothing before it comes down, then shrinking from [`art::STAMP_FROM`] and
    /// turning to its lean over `medium`.
    fn stamp(&self) -> Option<Stamp> {
        let landed = Stamp {
            scale: 1.0,
            lean: art::LEAN,
            shown: 1.0,
            landed: true,
        };
        if self.still() {
            return Some(landed);
        }
        self.after(STAMP_AT)?;
        let p = self.progress(STAMP_AT, tokens::Duration::Medium, Easing::EmphasizedDecel);
        if p >= 1.0 {
            return Some(landed);
        }
        // It comes down turned a little further than it lands: the draft's -14° to -8°.
        let from = art::LEAN.0 * 1.75;
        Some(Stamp {
            scale: art::STAMP_FROM + (1.0 - art::STAMP_FROM) * p,
            lean: iced::Radians(from + (art::LEAN.0 - from) * p),
            shown: (p / 0.6).min(1.0),
            landed: false,
        })
    }
}

/// The sign-off as it stands in `seq`: the mark and the hanko beside it, `Size::NeonMark` tall.
/// `arrived` and `now` place the hum, which redraws the glow by itself once the rest has settled.
pub fn sign_off<'a>(look: &Look, arrived: Instant, now: Instant, motion: Motion) -> El<'a> {
    let seq = Sequence::new(arrived, now, motion);
    // The glows are needed from the first strike; until then the bake runs on its own thread.
    let needed = seq.still() || seq.after(STROKES_AT).is_some();
    let art = art::art(look, needed);
    let height = f32::from(Size::NeonMark.px());
    let width = height + f32::from(Space::S32.px()) + f32::from(Size::Hanko.px());
    let glow = layer(Glow {
        arrived,
        now,
        motion,
        colors: look.colors,
        art: art.clone(),
    })
    .width(width)
    .height(height);
    let sign = layer(Sign {
        seq,
        colors: look.colors,
        art,
    })
    .width(width)
    .height(height);
    widget::stack![glow, sign].into()
}

/// Starts baking the sign-off's pictures for `look` on a thread of its own, long before the done
/// screen draws them, so its first frames never wait for the bake.
pub fn prepare(look: &Look) {
    let _ = art::art(look, false);
}

/// Whether the done screen still needs frames `since` it arrived.
pub fn moving(arrived: Instant, now: Instant, motion: Motion) -> bool {
    Sequence::new(arrived, now, motion).moving()
}

fn layer<P: canvas::Program<crate::app::Message>>(program: P) -> canvas::Canvas<P, crate::app::Message> {
    canvas::Canvas::new(program)
}

/// The frame the sign-off draws in.
fn frame(renderer: &Renderer, bounds: Rectangle) -> Frame {
    Frame::with_bounds(renderer, frame_bounds(bounds))
}

/// What the sign-off draws in: its box, and room around it as wide as the mark for the halo,
/// which reaches past the box.
fn frame_bounds(bounds: Rectangle) -> Rectangle {
    let r = f32::from(Size::NeonMark.px());
    Rectangle::new(
        Point::new(-r, -r),
        Area::new(bounds.width + 2.0 * r, bounds.height + 2.0 * r),
    )
}

/// A baked picture at `opacity`, pixel for pixel: it was baked on the device pixels it covers.
/// It is clipped to its own box, so the software renderer fills only that box and not all the
/// sign-off's frame, which is most of what a redraw of the hum costs there.
fn picture(frame: &mut Frame, picture: &Picture, opacity: f32) {
    if opacity <= 0.0 {
        return;
    }
    frame.with_clip(picture.bounds, |frame| {
        frame.draw_image(
            picture.bounds,
            canvas::Image::new(picture.handle.clone())
                .opacity(opacity.min(1.0))
                .filter_method(FilterMethod::Nearest)
                .snap(true),
        );
    });
}

/// The layer under the tubes: the unlit glass, the glow around each tube and the halo behind the
/// mark. Once the rest has settled it keeps its own clock for the hum and asks for its frames
/// itself, so the hum never rebuilds the view.
struct Glow {
    arrived: Instant,
    now: Instant,
    motion: Motion,
    colors: Colors,
    art: Option<Arc<Art>>,
}

#[derive(Default)]
struct GlowState {
    /// The time of the last frame drawn, which the hum reads once the view no longer changes.
    clock: Cell<Option<Instant>>,
    /// The unlit glass, kept once it has faded in.
    glass: Cache,
    glass_colour: Cell<Option<[u8; 4]>>,
    /// Whether the last frame was drawn in software, by tiny-skia rather than the GPU.
    software: Cell<bool>,
}

impl Glow {
    fn sequence(&self, state: &GlowState) -> Sequence {
        let now = state.clock.get().map_or(self.now, |clock| clock.max(self.now));
        Sequence::new(self.arrived, now, self.motion)
    }
}

impl canvas::Program<crate::app::Message> for Glow {
    type State = GlowState;

    fn update(
        &self,
        state: &mut GlowState,
        event: &canvas::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<Action<crate::app::Message>> {
        let canvas::Event::Window(window::Event::RedrawRequested(at)) = event else {
            return None;
        };
        state.clock.set(Some(*at));
        let seq = self.sequence(state);
        // While anything else moves the app draws every frame; the still hum asks for its own.
        if seq.still() || seq.moving() {
            return None;
        }
        let levels = if state.software.get() {
            HUM_LEVELS_SOFTWARE
        } else {
            HUM_LEVELS
        };
        Some(Action::request_redraw_at(*at + seq.next_hum(levels)))
    }

    fn draw(
        &self,
        state: &GlowState,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        state.software.set(matches!(renderer, Renderer::Secondary(_)));
        let seq = self.sequence(state);
        let c = self.colors;
        let dead = seq.dead();
        let glass_colour = color::alpha(c.surface_container_highest, Opacity::Placeholder.value() * dead);
        let draw_glass = |frame: &mut Frame| {
            for stroke in mark::strokes() {
                if let Some(path) = stroke.path(1.0) {
                    frame.stroke(&path, tube(glass_colour, stroke.width()));
                }
            }
        };
        let mut layers = Vec::with_capacity(2);
        if dead >= 1.0 {
            let key = Some(glass_colour.into_rgba8());
            if state.glass_colour.replace(key) != key {
                state.glass.clear();
            }
            layers.push(state.glass.draw_with_bounds(renderer, frame_bounds(bounds), draw_glass));
        } else {
            let mut glass = frame(renderer, bounds);
            draw_glass(&mut glass);
            layers.push(glass.into_geometry());
        }
        let Some(art) = &self.art else {
            return layers;
        };
        let mut lit = frame(renderer, bounds);
        picture(&mut lit, &art.halo, seq.halo());
        let (hum, flare) = (seq.hum(), seq.flare());
        for (k, glow) in art.glows.iter().enumerate() {
            let shown = seq.lit(k) * hum;
            picture(&mut lit, glow, shown + (1.0 - shown) * flare * seq.lit(k));
        }
        layers.push(lit.into_geometry());
        layers
    }
}

/// A tube's stroke: round at its ends and its bends, as glass is bent.
fn tube(colour: Color, width: f32) -> Stroke<'static> {
    Stroke::default()
        .with_color(colour)
        .with_width(width)
        .with_line_cap(LineCap::Round)
        .with_line_join(LineJoin::Round)
}

/// The tubes and the hanko, over the glow. It changes only while the sequence runs, so once it
/// has settled it is drawn from its cache.
struct Sign {
    seq: Sequence,
    colors: Colors,
    art: Option<Arc<Art>>,
}

#[derive(Default)]
struct SignState {
    cache: Cache,
    /// What the cache was drawn with: the tubes' colours, and the baked pictures, which hold the
    /// hanko's ink at the scale it was drawn at.
    drawn: RefCell<Option<Drawn>>,
}

/// The tubes' four colours, and the address of the baked pictures, if there were any yet.
type Drawn = ([[u8; 4]; 4], Option<usize>);

impl canvas::Program<crate::app::Message> for Sign {
    type State = SignState;

    fn draw(
        &self,
        state: &SignState,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let draw = |frame: &mut Frame| self.draw_into(frame);
        if self.seq.moving() {
            let mut frame = frame(renderer, bounds);
            draw(&mut frame);
            return vec![frame.into_geometry()];
        }
        let c = self.colors;
        let key = Some((
            [c.primary, c.primary_core, c.cold, c.cold_core].map(Color::into_rgba8),
            self.art.as_ref().map(|art| Arc::as_ptr(art).addr()),
        ));
        if state.drawn.replace(key) != key {
            state.cache.clear();
        }
        vec![state.cache.draw_with_bounds(renderer, frame_bounds(bounds), draw)]
    }
}

impl Sign {
    fn draw_into(&self, frame: &mut Frame) {
        let seq = &self.seq;
        let flare = seq.flare();
        for (k, stroke) in mark::strokes().iter().enumerate() {
            let Some(path) = stroke.path(seq.drawn(k)) else {
                continue;
            };
            let (glass, core) = stroke.tone.colors(&self.colors);
            frame.stroke(&path, tube(color::mix(glass, core, flare * FLARE_CORE), stroke.width()));
            let lit = seq.lit(k);
            if lit > 0.0 {
                let width = stroke.core_width() + (stroke.width() * FLARE_CORE - stroke.core_width()) * flare;
                frame.stroke(&path, tube(color::alpha(core, lit), width));
            }
        }
        let (Some(art), Some(stamp)) = (&self.art, seq.stamp()) else {
            return;
        };
        if stamp.landed {
            picture(frame, &art.hanko, 1.0);
            return;
        }
        let rest = art.stamp.bounds;
        let size = Area::new(rest.width * stamp.scale, rest.height * stamp.scale);
        let at = rest.center() - Vector::new(size.width / 2.0, size.height / 2.0);
        frame.draw_image(
            Rectangle::new(at, size),
            canvas::Image::new(art.stamp.handle.clone())
                .rotation(stamp.lean)
                .opacity(stamp.shown)
                .filter_method(FilterMethod::Linear),
        );
    }
}

#[cfg(test)]
mod tests;
