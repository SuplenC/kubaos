//! The contrast of every pair in `[contrast]`, in both fallback palettes, by WCAG 2.1.
//!
//! # Owns
//!
//! - The contrast ratio of two colours, a foreground drawn at an opacity over an opaque background,
//!   and reading every pair's ratio in each mode ([`readings`]).
//!
//! # Not here
//!
//! - Refusing a pair under its minimum: [`crate::rules`]. The same arithmetic on iced colours:
//!   `kubaos_ui::color`.

use std::collections::BTreeMap;

use crate::model::ContrastPair;
use crate::model::Rgb;
use crate::model::Tokens;

/// A fallback palette, by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Dark,
    Light,
}

impl Mode {
    /// Both palettes, dark first.
    pub const ALL: [Mode; 2] = [Mode::Dark, Mode::Light];

    /// The palette's table in `[color]`: `dark` or `light`.
    pub fn name(self) -> &'static str {
        match self {
            Mode::Dark => "dark",
            Mode::Light => "light",
        }
    }
}

/// One pair's contrast in one mode.
#[derive(Debug, Clone)]
pub struct Reading<'a> {
    /// The pair as `[contrast]` lists it.
    pub pair: &'a ContrastPair,
    /// The palette it was read in.
    pub mode: Mode,
    /// The ratio, 1 to 21, or `None` when a role or the opacity the pair names doesn't exist.
    pub ratio: Option<f32>,
}

impl Reading<'_> {
    /// Whether the pair has the contrast it needs.
    pub fn holds(&self) -> bool {
        self.ratio.is_some_and(|ratio| ratio >= self.pair.need.minimum())
    }
}

/// Every pair's contrast in each mode, pair by pair.
pub fn readings(tokens: &Tokens) -> Vec<Reading<'_>> {
    let mut found = Vec::new();
    for pair in &tokens.contrast.pairs {
        for mode in Mode::ALL {
            let palette = match mode {
                Mode::Dark => &tokens.color.dark,
                Mode::Light => &tokens.color.light,
            };
            found.push(Reading {
                pair,
                mode,
                ratio: ratio_of(pair, palette, &tokens.opacity),
            });
        }
    }
    found
}

fn ratio_of(pair: &ContrastPair, palette: &BTreeMap<String, Rgb>, opacity: &BTreeMap<String, f32>) -> Option<f32> {
    let fg = *palette.get(&pair.fg)?;
    let bg = *palette.get(&pair.bg)?;
    let alpha = match &pair.opacity {
        Some(name) => *opacity.get(name)?,
        None => 1.0,
    };
    Some(ratio(over(fg, bg, alpha), channels(bg)))
}

/// A colour's sRGB channels, 0 to 1, as written: gamma encoded, so blending them is what the
/// toolkits do, and [`luminance`] linearises them.
pub type Srgb = [f32; 3];

pub(crate) fn channels(c: Rgb) -> Srgb {
    [c.r, c.g, c.b].map(|v| f32::from(v) / 255.0)
}

/// `fg` at `alpha` over the opaque `bg`: what the eye sees, blended in sRGB as the toolkits blend.
fn over(fg: Rgb, bg: Rgb, alpha: f32) -> Srgb {
    let (f, b) = (channels(fg), channels(bg));
    [0, 1, 2].map(|i| b[i] + (f[i] - b[i]) * alpha)
}

/// WCAG 2.1's relative luminance of an sRGB colour.
fn luminance(c: Srgb) -> f32 {
    let channel = |v: f32| {
        if v <= 0.040_45 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(c[0]) + 0.7152 * channel(c[1]) + 0.0722 * channel(c[2])
}

/// WCAG 2.1's contrast ratio of two opaque colours, 1 to 21.
pub fn ratio(a: Srgb, b: Srgb) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    let (light, dark) = if la > lb { (la, lb) } else { (lb, la) };
    (light + 0.05) / (dark + 0.05)
}

#[cfg(test)]
mod tests;
