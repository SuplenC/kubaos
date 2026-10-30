//! Colour roles as iced colours, blending, and the contrast check.
//!
//! # Owns
//!
//! - [`Colors`], a palette of iced colours; alpha and mixing; the WCAG 2.1 contrast ratio and the
//!   minimums every pair of roles is held to ([`Contrast`]).
//!
//! # Not here
//!
//! - The roles' values: [`crate::tokens`], from `design/tokens.toml`.

use iced::Color;

use crate::tokens;
use crate::tokens::Derived;
use crate::tokens::Mode;
use crate::tokens::Palette;
use crate::tokens::Rgb;

/// Every colour role, as iced colours.
pub type Colors = Palette<Color>;

/// The fallback palette of `mode`, as iced colours.
pub fn colors(mode: Mode) -> Colors {
    tokens::palette(mode).map(rgb)
}

pub fn rgb(c: Rgb) -> Color {
    Color::from_rgb8(c.r, c.g, c.b)
}

/// The derived role `role` in `colors`: its tone there taken `role.white` of the way to white,
/// each 8-bit channel rounded half up as the tokens compute it, so it follows whichever palette is
/// drawn.
pub fn derived(colors: &Colors, role: Derived) -> Color {
    let tone = role.tone.of(colors);
    let channel = |c: f32| {
        let byte = (c * 255.0).round();
        (byte + (255.0 - byte) * role.white + 0.5).floor() / 255.0
    };
    Color::from_rgb(channel(tone.r), channel(tone.g), channel(tone.b))
}

/// `c` with its alpha multiplied by `a`.
pub fn alpha(c: Color, a: f32) -> Color {
    Color { a: c.a * a, ..c }
}

/// The linear blend from `a` to `b`, `t` of the way.
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

/// `fg` drawn over the opaque `bg`: what the eye sees.
pub fn over(fg: Color, bg: Color) -> Color {
    Color {
        a: 1.0,
        ..mix(bg, Color { a: 1.0, ..fg }, fg.a)
    }
}

/// Every role with its alpha multiplied by `a`, for a screen fading in.
pub fn faded(colors: Colors, a: f32) -> Colors {
    let a = a.clamp(0.0, 1.0);
    colors.map(|c| alpha(c, a))
}

/// How much contrast a pair of colours needs, by what the foreground is (WCAG 2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Contrast {
    /// Body text, labels: 4.5:1.
    Text,
    /// Text of 24 px and up, or 19 px semibold and up: 3:1.
    LargeText,
    /// Icons, outlines and other shapes that carry meaning: 3:1.
    Graphic,
}

impl Contrast {
    /// The lowest ratio that passes.
    pub fn minimum(self) -> f32 {
        match self {
            Contrast::Text => 4.5,
            Contrast::LargeText | Contrast::Graphic => 3.0,
        }
    }

    /// Whether `fg` over the opaque `bg` has enough contrast.
    pub fn holds(self, fg: Color, bg: Color) -> bool {
        contrast_ratio(over(fg, bg), bg) >= self.minimum()
    }
}

impl From<tokens::Need> for Contrast {
    fn from(need: tokens::Need) -> Self {
        match need {
            tokens::Need::Text => Contrast::Text,
            tokens::Need::LargeText => Contrast::LargeText,
            tokens::Need::Graphic => Contrast::Graphic,
        }
    }
}

/// `pair`'s contrast ratio in `colors`, its foreground drawn at its opacity over its background.
pub fn pair_ratio(pair: &tokens::ContrastPair, colors: &Colors) -> f32 {
    let bg = pair.bg.of(colors);
    let fg = alpha(pair.fg.of(colors), pair.opacity.map_or(1.0, tokens::Opacity::value));
    contrast_ratio(over(fg, bg), bg)
}

/// WCAG 2.1's contrast ratio of two opaque colours, 1 to 21.
pub fn contrast_ratio(a: Color, b: Color) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    let (light, dark) = if la > lb { (la, lb) } else { (lb, la) };
    (light + 0.05) / (dark + 0.05)
}

/// WCAG 2.1's relative luminance of an sRGB colour.
pub fn luminance(c: Color) -> f32 {
    let channel = |v: f32| {
        if v <= 0.040_45 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(c.r) + 0.7152 * channel(c.g) + 0.0722 * channel(c.b)
}

#[cfg(test)]
mod tests;
