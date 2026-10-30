//! Colours in OKLCH, Björn Ottosson's perceptual space, for the derived roles whose recipes scale a
//! tone's lightness and chroma, and for the labels fitted to a contrast.
//!
//! # Owns
//!
//! - Converting between `#rrggbb` and OKLCH ([`Oklch`]), and fitting a colour into sRGB by lowering
//!   its chroma alone ([`Oklch::to_rgb`]), so its hue and lightness never shift: a yellow stays
//!   olive as it darkens, never orange.
//! - Fitting a label's lightness until it holds a contrast against every background it sits on
//!   ([`fit_text`]).
//!
//! # Not here
//!
//! - The recipes themselves: [`crate::model::Derived`]. The contrast ratio: [`crate::contrast`].
//!   The same arithmetic in the mockups: `design/mockups/src/site/derive.ts`.
//!
//! <https://bottosson.github.io/posts/oklab/>

use crate::contrast::channels;
use crate::contrast::ratio;
use crate::model::Rgb;

/// A colour in OKLCH: lightness 0 to 1, chroma from 0, hue in degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklch {
    pub l: f64,
    pub c: f64,
    pub h: f64,
}

/// How many halvings [`Oklch::to_rgb`] and [`fit_text`] search by: far below an 8 bit channel's
/// step.
const HALVINGS: u32 = 40;

fn linear(v: f64) -> f64 {
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn encoded(v: f64) -> f64 {
    if v <= 0.003_130_8 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

impl Oklch {
    /// `rgb` in OKLCH. A grey's hue is 0.
    #[must_use]
    pub fn of(rgb: Rgb) -> Oklch {
        let [red, green, blue] = [rgb.r, rgb.g, rgb.b].map(|c| linear(f64::from(c) / 255.0));
        let long = (0.412_221_470_8 * red + 0.536_332_536_3 * green + 0.051_445_992_9 * blue).cbrt();
        let medium = (0.211_903_498_2 * red + 0.680_699_545_1 * green + 0.107_396_956_6 * blue).cbrt();
        let short = (0.088_302_461_9 * red + 0.281_718_837_6 * green + 0.629_978_700_5 * blue).cbrt();
        let lightness = 0.210_454_255_3 * long + 0.793_617_785 * medium - 0.004_072_046_8 * short;
        let green_red = 1.977_998_495_1 * long - 2.428_592_205 * medium + 0.450_593_709_9 * short;
        let blue_yellow = 0.025_904_037_1 * long + 0.782_771_766_2 * medium - 0.808_675_766 * short;
        let chroma = green_red.hypot(blue_yellow);
        let hue = if chroma < 1e-6 {
            0.0
        } else {
            blue_yellow.atan2(green_red).to_degrees().rem_euclid(360.0)
        };
        Oklch {
            l: lightness,
            c: chroma,
            h: hue,
        }
    }

    /// Its linear sRGB channels, unclamped: outside 0 to 1 where it is out of gamut.
    fn linear(self) -> [f64; 3] {
        let (sin, cos) = self.h.to_radians().sin_cos();
        let (green_red, blue_yellow) = (self.c * cos, self.c * sin);
        let long = (self.l + 0.396_337_777_4 * green_red + 0.215_803_757_3 * blue_yellow).powi(3);
        let medium = (self.l - 0.105_561_345_8 * green_red - 0.063_854_172_8 * blue_yellow).powi(3);
        let short = (self.l - 0.089_484_177_5 * green_red - 1.291_485_548 * blue_yellow).powi(3);
        [
            4.076_741_662_1 * long - 3.307_711_591_3 * medium + 0.230_969_929_2 * short,
            -1.268_438_004_6 * long + 2.609_757_401_1 * medium - 0.341_319_396_5 * short,
            -0.004_196_086_3 * long - 0.703_418_614_7 * medium + 1.707_614_701 * short,
        ]
    }

    fn in_gamut(self) -> bool {
        self.linear().iter().all(|v| (-1e-9..=1.0 + 1e-9).contains(v))
    }

    /// The colour in sRGB, its chroma lowered at the same hue and lightness until it fits, each
    /// channel rounded half up.
    #[must_use]
    pub fn to_rgb(self) -> Rgb {
        let l = self.l.clamp(0.0, 1.0);
        let mut fitted = Oklch {
            l,
            c: self.c.max(0.0),
            h: self.h,
        };
        if !fitted.in_gamut() {
            let (mut inside, mut outside) = (0.0, fitted.c);
            for _ in 0..HALVINGS {
                let middle = f64::midpoint(inside, outside);
                if (Oklch { c: middle, ..fitted }).in_gamut() {
                    inside = middle;
                } else {
                    outside = middle;
                }
            }
            fitted.c = inside;
        }
        let channel = |v: f64| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "Clamped to 0..=255 and rounded first."
            )]
            let byte = (encoded(v.clamp(0.0, 1.0)) * 255.0 + 0.5).floor().clamp(0.0, 255.0) as u8;
            byte
        };
        let [red, green, blue] = fitted.linear().map(channel);
        Rgb {
            r: red,
            g: green,
            b: blue,
        }
    }
}

/// `colour` with its OKLCH lightness moved, at the same hue and chroma, only as far as it takes to
/// hold `minimum` against every one of `backgrounds`: up, toward white, when it starts lighter than
/// them, down otherwise, and the other way when that way can't hold (a dark label on a fill too
/// dark for any). It is `colour` itself when that already holds, and the end of the first way
/// when neither does, which the contrast check then refuses.
#[must_use]
pub fn fit_text(colour: Rgb, backgrounds: &[Rgb], minimum: f32) -> Rgb {
    let holds = |c: Rgb| {
        backgrounds
            .iter()
            .all(|&bg| ratio(channels(c), channels(bg)) >= minimum)
    };
    if holds(colour) {
        return colour;
    }
    let start = Oklch::of(colour);
    let lighter = backgrounds.iter().all(|&bg| Oklch::of(bg).l <= start.l);
    let at = |l: f64| Oklch { l, ..start }.to_rgb();
    let first = if lighter { 1.0 } else { 0.0 };
    let Some(end) = [first, 1.0 - first].into_iter().find(|&end| holds(at(end))) else {
        return at(first);
    };
    let (mut failing, mut holding) = (start.l, end);
    for _ in 0..HALVINGS {
        let middle = f64::midpoint(failing, holding);
        if holds(at(middle)) {
            holding = middle;
        } else {
            failing = middle;
        }
    }
    at(holding)
}

#[cfg(test)]
mod tests;
