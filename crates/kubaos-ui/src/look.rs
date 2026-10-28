//! What every component draws with: the colours, the mode and the display's scale, together.
//!
//! # Owns
//!
//! - [`Look`], and corners: [`Corner`], a role's or one derived by nesting ([`nested`],
//!   [`concentric`]), in pixels ([`radius`]), and one drawn around another ([`outset`]).
//!
//! # Not here
//!
//! - The colour roles' values: [`crate::tokens`]. Contrast: [`crate::color`].

use iced::Color;

use crate::color;
use crate::color::Colors;
use crate::motion::Motion;
use crate::scale::Scale;
use crate::tokens::Mode;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Space;

/// The colours, mode, scale and motion settings a view is drawn with. Every component takes one.
#[derive(Debug, Clone, Copy)]
pub struct Look {
    pub mode: Mode,
    pub colors: Colors,
    pub scale: Scale,
    pub motion: Motion,
}

impl Look {
    /// The fallback palette of `mode`, at `scale`.
    #[must_use]
    pub fn new(mode: Mode, scale: Scale) -> Look {
        Look {
            mode,
            colors: color::colors(mode),
            scale,
            motion: Motion::default(),
        }
    }

    /// The same, moving as `motion` says: the focus stutter, and reduced motion.
    #[must_use]
    pub fn with_motion(self, motion: Motion) -> Look {
        Look { motion, ..self }
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

    /// What a fill brightens toward under a state layer while it is held: the lighter end of the
    /// palette, `on_surface` in dark and `surface` in light, so a held fill lights up in both.
    pub fn bright(&self) -> Color {
        match self.mode {
            Mode::Dark => self.colors.on_surface,
            Mode::Light => self.colors.surface,
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

/// A corner as drawn: a [`Radius`] role's, or one derived from a role by [`nested`]. A component
/// never names a number for its corners; a corner that isn't a role comes out of nesting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Corner(Option<u16>);

impl Corner {
    /// The corner's radius in logical pixels, or `None` for a pill.
    pub const fn px(self) -> Option<u16> {
        self.0
    }
}

impl Radius {
    /// The role's corner.
    pub const fn corner(self) -> Corner {
        Corner(self.px())
    }
}

impl From<Radius> for Corner {
    fn from(role: Radius) -> Self {
        role.corner()
    }
}

/// `corner`'s radius in logical pixels.
pub fn radius(corner: impl Into<Corner>) -> f32 {
    corner.into().px().map_or(PILL, f32::from)
}

/// The corner of a shape that fills one with corner `outer` less `inset` on every side, so their
/// corners share a centre: `outer` minus `inset`. A pill in a pill is a pill. `None` when the inset
/// reaches the outer radius: nothing is left to round, and the shapes can't be concentric.
pub const fn concentric(outer: Corner, inset: Space) -> Option<Corner> {
    let Some(outer_px) = outer.px() else {
        return Some(outer);
    };
    let inset = inset.px();
    if inset >= outer_px {
        return None;
    }
    Some(Corner(Some(outer_px - inset)))
}

/// [`concentric`] for a nesting that must resolve: in a `const`, a nesting that doesn't stops the
/// build.
///
/// # Panics
///
/// When `inset` reaches `outer`'s radius. In a `const` that is a compile error.
pub const fn nested(outer: Corner, inset: Space) -> Corner {
    match concentric(outer, inset) {
        Some(inner) => inner,
        None => panic!("the inset reaches the outer radius: the corners can't be concentric"),
    }
}

/// The radius of a shape `by` logical pixels outside one with corner `inner` on every side,
/// concentric with it: a glow's ring around its control. A pill around a pill is a pill.
pub fn outset(inner: impl Into<Corner>, by: f32) -> f32 {
    inner.into().px().map_or(PILL, |r| f32::from(r) + by)
}

#[cfg(test)]
mod tests;
