//! What every component draws with: the colours, the mode and the display's scale, together.
//!
//! # Owns
//!
//! - [`Look`], and the radius arithmetic shared by nested shapes: [`radius`], [`concentric`] and
//!   [`ring_radius`].
//!
//! # Not here
//!
//! - The colour roles' values: [`crate::tokens`]. Contrast: [`crate::color`].

use iced::Color;

use crate::color;
use crate::color::Colors;
use crate::scale::Scale;
use crate::tokens;
use crate::tokens::Mode;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Space;

/// The colours, mode and scale a view is drawn with. Every component takes one.
#[derive(Debug, Clone, Copy)]
pub struct Look {
    pub mode: Mode,
    pub colors: Colors,
    pub scale: Scale,
}

impl Look {
    /// The fallback palette of `mode`, at `scale`.
    #[must_use]
    pub fn new(mode: Mode, scale: Scale) -> Look {
        Look {
            mode,
            colors: color::colors(mode),
            scale,
        }
    }

    /// The same, with matugen's colours from the wallpaper.
    #[must_use]
    pub fn with_colors(self, colors: Colors) -> Look {
        Look { colors, ..self }
    }

    /// Every colour faded to `a`, for a screen fading in.
    #[must_use]
    pub fn faded(self, a: f32) -> Look {
        Look {
            colors: color::faded(self.colors, a),
            ..self
        }
    }

    /// One device pixel, in logical pixels.
    pub fn hairline(&self) -> f32 {
        self.scale.hairline()
    }

    /// The soft hairline around wells, cards and logs.
    pub fn outline_soft(&self) -> Color {
        color::alpha(self.colors.outline_variant, Opacity::OutlineSoft.value())
    }
}

/// A pill's radius: larger than any shape, which the renderer clamps to half its shorter side.
const PILL: f32 = 4096.0;

/// `r` in logical pixels.
pub fn radius(r: Radius) -> f32 {
    r.px().map_or(PILL, f32::from)
}

/// The radius of a shape nested `inset` inside a shape of radius `outer`, so their corners share a
/// centre: `outer` minus `inset`. `None` when that is no radius on the scale, so the nesting is
/// either too deep for concentric corners or off the scale.
pub const fn concentric(outer: Radius, inset: Space) -> Option<Radius> {
    if matches!(outer, Radius::Full) {
        return Some(Radius::Full);
    }
    let Some(outer_px) = outer.px() else {
        return None;
    };
    let Some(inner) = outer_px.checked_sub(inset.px()) else {
        return None;
    };
    let mut at = 0;
    while at < Radius::ALL.len() {
        if let Some(px) = Radius::ALL[at].px()
            && px == inner
        {
            return Some(Radius::ALL[at]);
        }
        at += 1;
    }
    None
}

/// [`concentric`] for a nesting that must be: in a `const`, a nesting off the radius scale stops
/// the build.
///
/// # Panics
///
/// When `outer` minus `inset` is no radius on the scale. In a `const` that is a compile error.
pub const fn nested(outer: Radius, inset: Space) -> Radius {
    match concentric(outer, inset) {
        Some(inner) => inner,
        None => panic!("the nested shape's radius is off the radius scale: the corners can't be concentric"),
    }
}

/// The radius of the focus ring's outer edge around a control of radius `control`: concentric with
/// it, [`tokens::FOCUS_OFFSET`] further out.
pub fn ring_radius(control: Radius) -> f32 {
    control.px().map_or(PILL, |r| f32::from(r + tokens::FOCUS_OFFSET))
}

#[cfg(test)]
mod tests;
