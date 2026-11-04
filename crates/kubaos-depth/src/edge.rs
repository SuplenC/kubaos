//! The sky's edge. Depth Anything sees the sky at 518 pixels across, 7 pixels a step on a 4K
//! picture, so its edge is coarse twice over: SAM, pointed at the sky, decides which side of the
//! edge each building falls on, small far towers included, and the sky is then keyed out of a
//! band along the edge at the picture's own resolution.
//!
//! SAM is taken only when it finds the same sky (an IoU of at least [`IOU_MIN`] with the depth's),
//! and only in a band along the coarse edge, so it can move the edge by the band's width at most
//! and never adds or drops a building elsewhere. A single object keeps BiRefNet's own edge, which
//! is finer than SAM's: `crate::run`.
//!
//! # Owns
//!
//! - The prompt SAM is pointed with ([`prompt_for`]), the merge of its answer ([`merge`]) and the
//!   key ([`key_sky`]).
//!
//! # Not here
//!
//! - Running SAM: `crate::models`.

use image::RgbImage;

use crate::choose::Method;
use crate::grid::Grid;
use crate::grid::Sums;
use crate::grid::box_mean;
use crate::models::Prompt;

/// The least share of SAM's region and the coarse one in common, over either (their IoU), for SAM's
/// edge to be taken: below it SAM found another region, and the coarse edge stays.
pub const IOU_MIN: f32 = 0.9;

/// The least fit SAM must claim for its own mask.
pub const FIT_MIN: f32 = 0.8;

/// The band SAM's edge is taken in, either side of the coarse edge, as a share of the mask's width:
/// 8 pixels of a 1024-wide mask, about Depth Anything's own pixel at that size.
const BAND: f32 = 8.0 / 1024.0;

/// How deep inside a region a point SAM is pointed at sits, as a share of the mask's width, so a
/// point never lands on the edge itself.
const DEPTH: f32 = 0.03;

/// The most points inside and outside the region SAM is pointed at.
const INSIDE_POINTS: usize = 6;
const OUTSIDE_POINTS: usize = 4;

/// The region SAM traces for a cut-out made by `method` from `mask`: the object, or the sky behind
/// the city.
pub fn region(method: Method, mask: &Grid) -> Grid {
    match method {
        Method::Object => mask.map(|p| if p > 0.5 { 1.0 } else { 0.0 }),
        Method::Sky => mask.map(|p| if p > 0.5 { 0.0 } else { 1.0 }),
    }
}

/// Points deep inside `region` (1 inside, 0 out) and deep outside it, spread over the picture, and
/// the object's box: a sky's box is the whole picture, which says nothing.
pub fn prompt_for(method: Method, region: &Grid) -> Prompt {
    let mut points = Vec::new();
    let inside = Sums::new(region);
    let outside = Sums::new(&region.map(|v| 1.0 - v));
    points.extend(
        deep_points(&inside, region, INSIDE_POINTS)
            .into_iter()
            .map(|(x, y)| (x, y, 1.0)),
    );
    points.extend(
        deep_points(&outside, region, OUTSIDE_POINTS)
            .into_iter()
            .map(|(x, y)| (x, y, 0.0)),
    );
    if method == Method::Object
        && let Some((x0, y0, x1, y1)) = bounds(region)
    {
        points.extend([(x0, y0, 2.0), (x1, y1, 3.0)]);
    }
    Prompt { points }
}

/// Up to `count` points, as shares of the picture, where `sums`' region holds a whole square
/// [`DEPTH`] across round them: the middle of the deepest such stretch in each cell of a 4 by 4
/// grid, the cells with the most of it first.
fn deep_points(sums: &Sums, grid: &Grid, count: usize) -> Vec<(f32, f32)> {
    const CELLS: usize = 4;
    let (width, height) = (grid.width(), grid.height());
    let reach = (DEPTH * width as f32 / 2.0).max(1.0);
    let mut cells: Vec<(usize, f32, f32)> = Vec::new();
    for cy in 0..CELLS {
        for cx in 0..CELLS {
            let (x0, x1) = (cx * width / CELLS, (cx + 1) * width / CELLS);
            let (y0, y1) = (cy * height / CELLS, (cy + 1) * height / CELLS);
            let (mut found, mut sum_x, mut sum_y) = (0usize, 0.0, 0.0);
            for y in (y0..y1).step_by(2) {
                for x in (x0..x1).step_by(2) {
                    let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
                    if sums.mean(fx - reach, fy - reach, fx + reach, fy + reach, 0.0) >= 1.0 {
                        found += 1;
                        sum_x += fx;
                        sum_y += fy;
                    }
                }
            }
            if found > 0 {
                let (mx, my) = (sum_x / found as f32, sum_y / found as f32);
                // The cell's mean may fall outside a curved region: take the deep point nearest it.
                if let Some((x, y)) = nearest_deep(sums, (x0, y0, x1, y1), reach, (mx, my)) {
                    cells.push((found, x / width as f32, y / height as f32));
                }
            }
        }
    }
    cells.sort_by_key(|&(found, _, _)| std::cmp::Reverse(found));
    cells.into_iter().take(count).map(|(_, x, y)| (x, y)).collect()
}

fn nearest_deep(
    sums: &Sums,
    (x0, y0, x1, y1): (usize, usize, usize, usize),
    reach: f32,
    (mx, my): (f32, f32),
) -> Option<(f32, f32)> {
    let mut best: Option<(f32, f32, f32)> = None;
    for y in (y0..y1).step_by(2) {
        for x in (x0..x1).step_by(2) {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            if sums.mean(fx - reach, fy - reach, fx + reach, fy + reach, 0.0) < 1.0 {
                continue;
            }
            let distance = (fx - mx).powi(2) + (fy - my).powi(2);
            if best.is_none_or(|(known, _, _)| distance < known) {
                best = Some((distance, fx, fy));
            }
        }
    }
    best.map(|(_, x, y)| (x, y))
}

/// The box round `region`'s pixels, as shares of the picture.
fn bounds(region: &Grid) -> Option<(f32, f32, f32, f32)> {
    let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
    for y in 0..region.height() {
        for x in 0..region.width() {
            if region.get(x, y) > 0.5 {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x + 1);
                y1 = y1.max(y + 1);
            }
        }
    }
    (x0 < x1).then(|| {
        let (w, h) = (region.width() as f32, region.height() as f32);
        (x0 as f32 / w, y0 as f32 / h, x1 as f32 / w, y1 as f32 / h)
    })
}

/// The share two masks' regions (above one half) have in common, over the two together.
pub fn iou(a: &Grid, b: &Grid) -> f32 {
    let (mut both, mut either) = (0usize, 0usize);
    for (&p, &q) in a.values().iter().zip(b.values()) {
        both += usize::from(p > 0.5 && q > 0.5);
        either += usize::from(p > 0.5 || q > 0.5);
    }
    if either == 0 { 1.0 } else { both as f32 / either as f32 }
}

/// The cut-out's mask with SAM's edge: `traced` (SAM's chance of the region, at the mask's size)
/// taken in the band along `mask`'s edge, as the cut-out's chance for `method`, and `mask` kept
/// everywhere else.
pub fn merge(method: Method, mask: &Grid, traced: &Grid) -> Grid {
    let hard = Sums::new(&mask.map(|p| if p > 0.5 { 1.0 } else { 0.0 }));
    let reach = (BAND * mask.width() as f32).max(1.0);
    Grid::from_fn(mask.width(), mask.height(), |x, y| {
        let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
        let around = hard.mean_inside(fx - reach, fy - reach, fx + reach, fy + reach);
        let in_band = around > 0.0 && around < 1.0;
        if !in_band {
            return mask.get(x, y);
        }
        let region = traced.get(x, y);
        match method {
            Method::Object => region,
            Method::Sky => 1.0 - region,
        }
    })
}

/// The band along the coarse edge the sky is keyed out in, either side of it, as a share of the
/// picture's width: 20 pixels of a 4K picture, wide enough to hold SAM's outline, which runs up to
/// 15 pixels into the sky, and Depth Anything's, 7 pixels a step there.
const KEY_REACH: f32 = 20.0 / 3840.0;

/// How far from the sky's colour, as a share of the way to the city's beside it, a pixel starts to
/// stand in front, and is opaque: under a fifth it is the sky's own gradient or haze; past a little
/// over half, a lit window or a dark wall, it is the city.
const KEY_FLOOR: f32 = 0.2;
const KEY_FULL: f32 = 0.6;

/// The least distance between the sky's colour and the city's, in sRGB's unit cube, for a key to
/// tell them apart: below it the coarse edge stays.
const KEY_MIN: f32 = 0.05;

/// `alpha` (the cut-out's chance at the picture's size, from depth and SAM, whose edges are 7 to 15
/// pixels coarse on a 4K picture) with the sky keyed out of a band along its edge, at the
/// picture's own resolution. A sky is smooth, so its colour beside each pixel of the band is the
/// mean of the sure sky round it; the city's is the mean of the sure city. A pixel is as opaque as
/// it is far from the sky's colour, from none at [`KEY_FLOOR`] of the way to the city's to whole
/// at [`KEY_FULL`], so a roofline, a crane or an antenna comes out at its own edge, and haze
/// between them stays sky.
pub fn key_sky(picture: &RgbImage, alpha: &Grid) -> Grid {
    let (width, height) = (alpha.width(), alpha.height());
    let reach = (KEY_REACH * width as f32).round().max(1.0) as usize;
    let hard: Vec<f32> = alpha
        .values()
        .iter()
        .map(|&p| if p > 0.5 { 1.0 } else { 0.0 })
        .collect();
    let around = box_mean(&hard, width, height, reach);
    let sure = |at: usize, city: bool| {
        if city {
            around[at] >= 1.0 - 1e-6
        } else {
            around[at] <= 1e-6
        }
    };
    let sky_weight: Vec<f32> = (0..hard.len()).map(|at| f32::from(u8::from(sure(at, false)))).collect();
    let city_weight: Vec<f32> = (0..hard.len()).map(|at| f32::from(u8::from(sure(at, true)))).collect();
    let near = Means::new(picture, &sky_weight, &city_weight, 2 * reach);
    let far = Means::new(picture, &sky_weight, &city_weight, 8 * reach);
    let mut keyed = alpha.values().to_vec();
    for (at, pixel) in picture.pixels().enumerate() {
        if sure(at, true) || sure(at, false) {
            continue;
        }
        // A gap between two towers may hold no sure sky near it: the sky farther round is the same.
        let Some(sky) = near.sky(at).or_else(|| far.sky(at)) else {
            continue;
        };
        let Some(city) = near.city(at).or_else(|| far.city(at)) else {
            continue;
        };
        let apart = distance(city, sky);
        if apart < KEY_MIN {
            continue;
        }
        let way = distance(pixel.0.map(|v| f32::from(v) / 255.0), sky) / apart;
        keyed[at] = ((way - KEY_FLOOR) / (KEY_FULL - KEY_FLOOR)).clamp(0.0, 1.0);
    }
    Grid::new(width, height, keyed)
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|c| (a[c] - b[c]).powi(2)).sum::<f32>().sqrt()
}

/// The mean colours of the sure sky and the sure city round each pixel, within a window.
struct Means {
    sky_share: Vec<f32>,
    city_share: Vec<f32>,
    sky: [Vec<f32>; 3],
    city: [Vec<f32>; 3],
}

impl Means {
    fn new(picture: &RgbImage, sky_weight: &[f32], city_weight: &[f32], radius: usize) -> Self {
        let (width, height) = (picture.width() as usize, picture.height() as usize);
        let mean = |values: &[f32]| box_mean(values, width, height, radius);
        let channel = |c: usize, weight: &[f32]| {
            let weighted: Vec<f32> = picture
                .pixels()
                .zip(weight)
                .map(|(p, w)| f32::from(p.0[c]) / 255.0 * w)
                .collect();
            mean(&weighted)
        };
        Self {
            sky_share: mean(sky_weight),
            city_share: mean(city_weight),
            sky: std::array::from_fn(|c| channel(c, sky_weight)),
            city: std::array::from_fn(|c| channel(c, city_weight)),
        }
    }

    fn sky(&self, at: usize) -> Option<[f32; 3]> {
        let share = self.sky_share[at];
        (share > 0.0).then(|| std::array::from_fn(|c| self.sky[c][at] / share))
    }

    fn city(&self, at: usize) -> Option<[f32; 3]> {
        let share = self.city_share[at];
        (share > 0.0).then(|| std::array::from_fn(|c| self.city[c][at] / share))
    }
}

#[cfg(test)]
mod tests;
