//! The rest clock on a picture: where its digits and its kicker (the date line) fall for a place,
//! on each screen the picture may be shown on, and the place that lets a cut-out cover part of the
//! digits and never the kicker.
//!
//! The geometry is the Lock page's in `design/mockups/` (`lib/lock.ts`, `lib/lock.css`): the screen
//! crops the picture as `object-fit: cover` does, the place is the middle of the digits' baseline
//! in shares of the picture, the digits' box runs from their cap height down to their baseline, and
//! the kicker stands `space.s16` above it, centred. The sizes are logical pixels:
//!
//! | What | Value | From |
//! |---|---|---|
//! | The digits' size | 384 | `type.clock_rest` |
//! | Their cap height, a share of it | 0.71 | Geist Light's `OS/2` table |
//! | Their width, in their size | 2.395 | Geist Light's advances for `21:47` |
//! | The kicker's gap above the digits | 16 | `space.s16` |
//! | The kicker's height | 24 | `type.clock_place`'s line |
//! | The kicker's width | 480 | the longest English kicker, a third longer for a translation |
//! | The halo round the kicker's letters | 6 | `size.halo` |
//! | The screen's edge the clock keeps clear of | 32 | `space.s32` |
//!
//! # Owns
//!
//! - The clock's boxes for a place ([`boxes`]), the place for a cut-out ([`place_for`]) and the
//!   place without one ([`DEFAULT_PLACE`]).
//!
//! # Not here
//!
//! - Whether a cut-out is good enough to keep: `crate::choose`. The clock's colour: `crate::tint`.

use serde::Deserialize;
use serde::Serialize;

use crate::grid::Sums;

/// The rest digits' size, `type.clock_rest`.
const DIGITS_SIZE: f32 = 384.0;
/// Geist Light's cap height, a share of its size: the digits' box is trimmed to it.
const CAP_HEIGHT: f32 = 0.71;
/// `21:47` in Geist Light, in its size: the mockup's time. The figures are proportional, so the box
/// is a typical time's; `00:00`, the widest, runs a fifth wider.
const DIGITS_WIDTH: f32 = 2.395;
/// `space.s16`, between the kicker and the digits' tops.
const KICKER_GAP: f32 = 16.0;
/// The kicker's line, `type.clock_place`.
const KICKER_HEIGHT: f32 = 24.0;
/// `東京・月曜日 · Wednesday, 30 September`, about 360, a third longer for a translation.
const KICKER_WIDTH: f32 = 480.0;
/// `size.halo`: the kicker's letters are lifted off the picture this far round, and the cut-out
/// stays out of it too.
const HALO: f32 = 6.0;
/// `space.s32`, the screen's edge the clock keeps clear of.
const MARGIN: f32 = 32.0;

/// A screen in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Screen {
    pub width: f32,
    pub height: f32,
}

/// The screen the clock's coverage is judged on: the Lock page's, a 16:10 panel 1536 by 960, a
/// 2560 by 1600 laptop at 1.67 or a 1920 by 1200 one at 1.25.
pub const PRIMARY: Screen = Screen {
    width: 1536.0,
    height: 960.0,
};

/// The screens the clock must fit on and the kicker stay clear on: 16:10 and 16:9 panels, from the
/// Lock page's up, in logical pixels. A larger one shows the clock smaller against the picture, so
/// the kicker sits nearer the baseline; a wider one crops more of the picture's top and bottom.
pub const SCREENS: [Screen; 5] = [
    PRIMARY,
    Screen {
        width: 1920.0,
        height: 1200.0,
    },
    Screen {
        width: 1536.0,
        height: 864.0,
    },
    Screen {
        width: 1920.0,
        height: 1080.0,
    },
    Screen {
        width: 2560.0,
        height: 1440.0,
    },
];

/// Where the rest clock stands: the middle of its digits' baseline, as shares of the picture's
/// width and height, so it keeps its place against the city however a screen crops the picture.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Place {
    pub x: f32,
    pub y: f32,
}

/// The place on a picture with no cut-out: centred, the baseline a little below the middle, so the
/// digits' weight sits on it, as on the Lock page's Shibuya.
pub const DEFAULT_PLACE: Place = Place { x: 0.5, y: 0.56 };

/// A box in a picture's pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl Rect {
    fn scaled(self, by: f32) -> Self {
        Self {
            x0: self.x0 * by,
            y0: self.y0 * by,
            x1: self.x1 * by,
            y1: self.y1 * by,
        }
    }
}

/// The clock's two boxes on a picture, in its pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Boxes {
    /// The digits, cap height to baseline.
    pub digits: Rect,
    /// The kicker with its halo: what the cut-out must never cover.
    pub kicker: Rect,
}

/// The clock at `place` on a `width` by `height` picture shown on `screen`: its boxes in the
/// picture's pixels, and whether it fits inside the screen's margins there.
pub fn boxes(place: Place, width: f32, height: f32, screen: Screen) -> (Boxes, bool) {
    let scale = (screen.width / width).max(screen.height / height);
    let left = (screen.width - width * scale) / 2.0;
    let top = (screen.height - height * scale) / 2.0;
    let cx = left + place.x * width * scale;
    let base = top + place.y * height * scale;
    let digits_half = DIGITS_WIDTH * DIGITS_SIZE / 2.0;
    let digits_top = base - CAP_HEIGHT * DIGITS_SIZE;
    let kicker_top = digits_top - KICKER_GAP - KICKER_HEIGHT;
    let half = digits_half.max(KICKER_WIDTH / 2.0);
    let fits = cx - half >= MARGIN
        && cx + half <= screen.width - MARGIN
        && kicker_top >= MARGIN
        && base <= screen.height - MARGIN;
    let rect = |x0: f32, y0: f32, x1: f32, y1: f32| Rect {
        x0: (x0 - left) / scale,
        y0: (y0 - top) / scale,
        x1: (x1 - left) / scale,
        y1: (y1 - top) / scale,
    };
    let boxes = Boxes {
        digits: rect(cx - digits_half, digits_top, cx + digits_half, base),
        kicker: rect(
            cx - KICKER_WIDTH / 2.0 - HALO,
            kicker_top - HALO,
            cx + KICKER_WIDTH / 2.0 + HALO,
            digits_top - KICKER_GAP + HALO,
        ),
    };
    (boxes, fits)
}

/// How much of the digits a cut-out may cover: from a spire crossing one figure, as on the Lock
/// page's Tokyo Tower, to a skyline up to the figures' middles, so the time still reads.
pub const COVER_MIN: f32 = 0.01;
pub const COVER_MAX: f32 = 0.45;

/// The cover a place is drawn to, and how strongly against its distance from the picture's middle:
/// a fifth of the digits behind the city reads as the clock standing in it, but a clock pulled far
/// off the middle to reach it reads as misplaced.
const COVER_AIM: f32 = 0.2;
const COVER_WEIGHT: f32 = 0.5;

/// The most of the kicker's box a cut-out may cover: a stray pixel of a soft edge, never a letter.
const KICKER_COVER_MAX: f32 = 0.001;

/// The steps the place is searched in, as shares of the picture.
const STEP: f32 = 0.005;

/// A place for the clock on a mask and how it sits there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    pub place: Place,
    /// The share of the digits the mask covers, on [`PRIMARY`].
    pub cover: f32,
}

/// The place for the clock on a picture whose cut-out is `mask` (a summed-area table of its hard
/// mask, 1 where the cut-out stands, over a picture `width` by `height`, which the mask may be a
/// smaller copy of): the mask covers between [`COVER_MIN`] and [`COVER_MAX`] of the digits on
/// [`PRIMARY`] and none of the kicker on any of [`SCREENS`], and of those places the one with the
/// least distance from the picture's middle plus [`COVER_WEIGHT`] times its cover's from
/// [`COVER_AIM`]. None when there is no such place.
pub fn place_for(mask: &Sums, mask_width: usize, width: f32, height: f32) -> Option<Placed> {
    let to_mask = mask_width as f32 / width;
    let mut best: Option<(f32, Placed)> = None;
    let steps = (1.0 / STEP).round() as usize;
    for ix in 0..=steps {
        for iy in 0..=steps {
            let place = Place {
                x: ix as f32 * STEP,
                y: iy as f32 * STEP,
            };
            let Some(cover) = cover_on(mask, to_mask, place, width, height) else {
                continue;
            };
            let cost = COVER_WEIGHT * (cover - COVER_AIM).abs() + (place.x - 0.5).abs() + (place.y - 0.5).abs();
            if best.is_none_or(|(known, _)| cost < known) {
                best = Some((cost, Placed { place, cover }));
            }
        }
    }
    best.map(|(_, placed)| placed)
}

/// The share of the digits the mask covers at `place` on [`PRIMARY`], or none when the clock
/// doesn't fit there, the cover is out of bounds or the mask reaches the kicker on any screen.
fn cover_on(mask: &Sums, to_mask: f32, place: Place, width: f32, height: f32) -> Option<f32> {
    let (primary, fits) = boxes(place, width, height, PRIMARY);
    if !fits {
        return None;
    }
    let digits = primary.digits.scaled(to_mask);
    let cover = mask.mean(digits.x0, digits.y0, digits.x1, digits.y1, 0.0);
    if !(COVER_MIN..=COVER_MAX).contains(&cover) {
        return None;
    }
    for screen in SCREENS {
        let (on_screen, fits) = boxes(place, width, height, screen);
        let kicker = on_screen.kicker.scaled(to_mask);
        if !fits || mask.mean(kicker.x0, kicker.y0, kicker.x1, kicker.y1, 0.0) > KICKER_COVER_MAX {
            return None;
        }
    }
    Some(cover)
}

#[cfg(test)]
mod tests;
