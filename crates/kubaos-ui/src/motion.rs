//! Motion: how far along an animation is, from its duration, its curve and the clock.
//!
//! # Owns
//!
//! - Evaluating an [`Easing`] curve, and the speed and reduce-motion settings applied to a
//!   [`Duration`].
//! - A neon tube striking ([`Motion::strike`]), and the stutter setting that turns it off.
//! - [`Tween`]: a value a widget keeps in its state, moving to each new one its view gives it.
//!
//! # Not here
//!
//! - Which duration and curve an animation takes: their docs in [`crate::tokens`].

use std::time;
use std::time::Instant;

use crate::tokens;
use crate::tokens::Duration;
use crate::tokens::Easing;

/// How the user wants things to move.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Motion {
    /// Every duration is divided by this: 2 is twice as fast. 0.5 to 2.
    speed: f32,
    /// Moves become short fades, or nothing.
    reduced: bool,
    /// The focus tube strikes on as focus arrives, as a neon sign does.
    stutter: bool,
}

/// Speed 1: the durations as the tokens give them.
pub const DEFAULT_SPEED: f32 = 1.0;

/// The focus tube strikes on: it is the sign's own flicker, and the user may turn it off.
pub const DEFAULT_STUTTER: bool = true;

impl Default for Motion {
    fn default() -> Self {
        Motion {
            speed: DEFAULT_SPEED,
            reduced: false,
            stutter: DEFAULT_STUTTER,
        }
    }
}

impl Motion {
    /// `speed` held to the setting's range, 0.5 to 2.
    #[must_use]
    pub fn speed(self, speed: f32) -> Self {
        Motion {
            speed: speed.clamp(0.5, 2.0),
            ..self
        }
    }

    /// Moves as short fades.
    #[must_use]
    pub fn reduced(self) -> Self {
        Motion { reduced: true, ..self }
    }

    /// Moves as they are.
    #[must_use]
    pub fn no_reduced(self) -> Self {
        Motion { reduced: false, ..self }
    }

    pub fn is_reduced(self) -> bool {
        self.reduced
    }

    /// The focus tube strikes on as focus arrives.
    #[must_use]
    pub fn stutter(self) -> Self {
        Motion { stutter: true, ..self }
    }

    /// The focus tube fades in as focus arrives.
    #[must_use]
    pub fn no_stutter(self) -> Self {
        Motion { stutter: false, ..self }
    }

    /// Whether a tube strikes on: the stutter is on and motion isn't reduced.
    pub fn strikes(self) -> bool {
        self.stutter && !self.reduced
    }

    /// A tube's brightness, 0 to 1, `elapsed` after it starts striking: through
    /// [`tokens::NEON_STRIKE`]'s even steps over [`Duration::Medium`], then lit.
    pub fn strike(self, elapsed: time::Duration) -> f32 {
        let total = self.duration(Duration::Medium).as_secs_f32();
        let steps = tokens::NEON_STRIKE.len();
        if total <= 0.0 || elapsed.as_secs_f32() >= total {
            return 1.0;
        }
        let step = (elapsed.as_secs_f32() / total * steps as f32) as usize;
        tokens::NEON_STRIKE[step.min(steps - 1)]
    }

    /// How long `duration` runs under these settings. With reduced motion anything longer than
    /// [`Duration::Short`] becomes a short fade.
    pub fn duration(self, duration: Duration) -> time::Duration {
        let ms = if self.reduced {
            duration.ms().min(Duration::Short.ms())
        } else {
            duration.ms()
        };
        time::Duration::from_micros((f64::from(ms) * 1000.0 / f64::from(self.speed)).round() as u64)
    }

    /// How far along, 0 to 1, an animation of `duration` eased by `easing` is `elapsed` after it
    /// started.
    pub fn progress(self, duration: Duration, easing: Easing, elapsed: time::Duration) -> f32 {
        let total = self.duration(duration).as_secs_f32();
        if total <= 0.0 {
            return 1.0;
        }
        ease(easing, elapsed.as_secs_f32() / total)
    }
}

/// `easing` at `x`, 0 to 1: the share of the way an animation has covered when `x` of its time
/// has gone.
pub fn ease(easing: Easing, x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.0 || x >= 1.0 {
        return x;
    }
    let [x1, y1, x2, y2] = easing.curve();
    let bezier = |a: f32, b: f32, t: f32| {
        let u = 1.0 - t;
        3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
    };
    // x(t) rises monotonically for x control points in 0..=1, so bisection finds the t for x; 24
    // rounds narrow it to under a millionth.
    let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
    let mut t = x;
    for _ in 0..24 {
        let at = bezier(x1, x2, t);
        if (at - x).abs() < 1e-6 {
            break;
        }
        if at < x {
            lo = t;
        } else {
            hi = t;
        }
        t = f32::midpoint(lo, hi);
    }
    bezier(y1, y2, t)
}

/// A value moving to the last one it was given, over a duration and a curve: what a widget keeps
/// in its state so that a change in its view animates instead of jumping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tween {
    from: f32,
    to: f32,
    at: Instant,
    duration: Duration,
    easing: Easing,
}

impl Tween {
    /// Resting at `value`, moving over `duration` on `easing` when it is next given another.
    pub fn new(value: f32, duration: Duration, easing: Easing) -> Tween {
        Tween {
            from: value,
            to: value,
            at: Instant::now(),
            duration,
            easing,
        }
    }

    /// Moves to `to` from wherever it is at `now`. The same target again changes nothing, so a view
    /// can give its value on every frame.
    pub fn to(&mut self, to: f32, now: Instant) {
        if (to - self.to).abs() <= f32::EPSILON {
            return;
        }
        self.from = self.value(now);
        self.to = to;
        self.at = now;
    }

    /// Where it ends up.
    pub fn target(&self) -> f32 {
        self.to
    }

    /// Where it is at `now`.
    pub fn value(&self, now: Instant) -> f32 {
        let t = Motion::default().progress(self.duration, self.easing, now.saturating_duration_since(self.at));
        self.from + (self.to - self.from) * t
    }

    /// Whether it is still on its way at `now`.
    pub fn moving(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.at) < Motion::default().duration(self.duration)
            && (self.to - self.from).abs() > f32::EPSILON
    }
}

#[cfg(test)]
mod tests;
