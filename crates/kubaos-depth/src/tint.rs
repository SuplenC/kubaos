//! The rest clock's tint: a light colour of the picture under its digits, so the clock reads as
//! part of the picture rather than as words laid on it. The kicker takes it too.
//!
//! The rule is the Lock page's (`design/mockups/src/lib/lock-tint.ts`, `tintFor`), so the lock and
//! the mockups agree: every other picture pixel under the digits counts, each pixel's OKLCH hue by
//! its chroma squared, so the signs and the glow of a sky decide it and grey haze and dark walls
//! barely do; the chroma is the pixels' own, weighted the same way. That hue and chroma, the chroma
//! raised by [`CHROMA`], are set to [`LIGHTNESS`] and fitted into sRGB by lowering the chroma
//! alone.
//!
//! # Owns
//!
//! - The tint ([`tint_under`]) and the OKLCH conversions it needs.
//!
//! # Not here
//!
//! - Where the digits are: `crate::clock`.

use image::RgbImage;

use crate::clock::Rect;

/// The tint's OKLCH lightness: light, as a sky's highlight is.
const LIGHTNESS: f64 = 0.84;

/// How much the tint raises the picture's chroma, so a hazy sky still gives a colour.
const CHROMA: f64 = 1.2;

/// The colour in OKLCH: lightness, chroma, hue in degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Lch {
    l: f64,
    c: f64,
    h: f64,
}

/// The tint of the picture under `digits` (in its pixels), as `#rrggbb`.
pub fn tint_under(picture: &RgbImage, digits: Rect) -> String {
    let clamp = |v: f32, len: u32| (v.round().max(0.0) as u32).min(len - 1);
    let (x0, x1) = (clamp(digits.x0, picture.width()), clamp(digits.x1, picture.width()));
    let (y0, y1) = (clamp(digits.y0, picture.height()), clamp(digits.y1, picture.height()));
    let (mut a, mut b, mut weight, mut chroma) = (0.0, 0.0, 0.0, 0.0);
    for y in (y0..=y1).step_by(2) {
        for x in (x0..=x1).step_by(2) {
            let lch = to_lch(picture.get_pixel(x, y).0);
            let squared = lch.c * lch.c;
            a += squared * lch.h.to_radians().cos();
            b += squared * lch.h.to_radians().sin();
            weight += squared;
            chroma += squared * lch.c;
        }
    }
    if weight == 0.0 {
        return to_hex(Lch {
            l: LIGHTNESS,
            c: 0.0,
            h: 0.0,
        });
    }
    let hue = (b.atan2(a).to_degrees() + 360.0) % 360.0;
    to_hex(Lch {
        l: LIGHTNESS,
        c: chroma / weight * CHROMA,
        h: hue,
    })
}

fn linear(v: f64) -> f64 {
    if v <= 0.04045 {
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

fn to_lch(rgb: [u8; 3]) -> Lch {
    let [red, green, blue] = rgb.map(|v| linear(f64::from(v) / 255.0));
    let long = (0.412_221_470_8 * red + 0.536_332_536_3 * green + 0.051_445_992_9 * blue).cbrt();
    let medium = (0.211_903_498_2 * red + 0.680_699_545_1 * green + 0.107_396_956_6 * blue).cbrt();
    let short = (0.088_302_461_9 * red + 0.281_718_837_6 * green + 0.629_978_700_5 * blue).cbrt();
    let lightness = 0.210_454_255_3 * long + 0.793_617_785 * medium - 0.004_072_046_8 * short;
    let green_red = 1.977_998_495_1 * long - 2.428_592_205 * medium + 0.450_593_709_9 * short;
    let blue_yellow = 0.025_904_037_1 * long + 0.782_771_766_2 * medium - 0.808_675_766 * short;
    let c = green_red.hypot(blue_yellow);
    let h = if c < 1e-6 {
        0.0
    } else {
        (blue_yellow.atan2(green_red).to_degrees() % 360.0 + 360.0) % 360.0
    };
    Lch { l: lightness, c, h }
}

fn linear_of(lch: Lch) -> [f64; 3] {
    let green_red = lch.c * lch.h.to_radians().cos();
    let blue_yellow = lch.c * lch.h.to_radians().sin();
    let long = (lch.l + 0.396_337_777_4 * green_red + 0.215_803_757_3 * blue_yellow).powi(3);
    let medium = (lch.l - 0.105_561_345_8 * green_red - 0.063_854_172_8 * blue_yellow).powi(3);
    let short = (lch.l - 0.089_484_177_5 * green_red - 1.291_485_548 * blue_yellow).powi(3);
    [
        4.076_741_662_1 * long - 3.307_711_591_3 * medium + 0.230_969_929_2 * short,
        -1.268_438_004_6 * long + 2.609_757_401_1 * medium - 0.341_319_396_5 * short,
        -0.004_196_086_3 * long - 0.703_418_614_7 * medium + 1.707_614_701 * short,
    ]
}

fn in_gamut(lch: Lch) -> bool {
    linear_of(lch).iter().all(|&v| (-1e-9..=1.0 + 1e-9).contains(&v))
}

/// `lch` in sRGB, `#rrggbb`, its chroma lowered at the same hue and lightness until it fits.
fn to_hex(lch: Lch) -> String {
    let mut fitted = Lch {
        l: lch.l.clamp(0.0, 1.0),
        c: lch.c.max(0.0),
        h: lch.h,
    };
    if !in_gamut(fitted) {
        let (mut inside, mut outside) = (0.0, fitted.c);
        for _ in 0..40 {
            let middle = f64::midpoint(inside, outside);
            if in_gamut(Lch { c: middle, ..fitted }) {
                inside = middle;
            } else {
                outside = middle;
            }
        }
        fitted.c = inside;
    }
    let byte = |v: f64| (encoded(v.clamp(0.0, 1.0)) * 255.0 + 0.5).floor().clamp(0.0, 255.0) as u8;
    let [r, g, b] = linear_of(fitted).map(byte);
    format!("#{r:02x}{g:02x}{b:02x}")
}

#[cfg(test)]
mod tests;
