//! The choice of cut-out: the sky's, from depth, or the single object's, from BiRefNet, scored
//! under the clock, and one of them kept or none.
//!
//! A candidate is a soft mask of what stands in front of the clock, a chance in `[0, 1]` per pixel,
//! and the **separation** of its two sides in the values it was cut from ([`sky`], [`object`]). The
//! rule the crate's doc states is applied here: [`candidate_score`] and [`choose`].
//!
//! # Owns
//!
//! - The sky's candidate from a depth map ([`sky`]), the object's from BiRefNet's chances
//!   ([`object`]), each one's score under the clock and the choice between them.
//!
//! # Not here
//!
//! - Running the models: `crate::models`. The clock's geometry: `crate::clock`.

use serde::Deserialize;
use serde::Serialize;

use crate::clock;
use crate::clock::Placed;
use crate::grid::Grid;
use crate::grid::Sums;
use crate::grid::otsu;

/// Which model's cut-out was kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    /// Everything nearer than the sky, from the depth map's farthest layer.
    Sky,
    /// The single object BiRefNet finds.
    Object,
}

/// A soft mask of what stands in front of the clock, and how cleanly it was told apart.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub method: Method,
    /// The chance each pixel stands in front of the clock.
    pub mask: Grid,
    /// How cleanly the mask's two sides are told apart in the values it was cut from, in `[0, 1]`.
    pub separation: f32,
}

/// The least separation a candidate needs, so the mask cuts along a real edge between two layers.
pub const SEPARATION_MIN: f32 = 0.85;

/// The least score a candidate needs to be kept: below it, no cut-out is written.
pub const SCORE_MIN: f32 = 0.7;

/// The sky's depth: Depth Anything puts it at the far plane, so it is the depth map's floor, the
/// pixels within this share of the depth's range from the farthest. On the shipped wallpapers with
/// a sky, a fifth to a half of the picture sits there and the city starts past 0.03.
const SKY_DEPTH: f32 = 0.01;

/// The depth past the sky's in which a pixel counts against its separation: the haze, a far
/// mountain or a far skyline, which the sky's edge can't be sure of.
const SKY_NEAR: f32 = 0.05;

/// How far past [`SKY_DEPTH`] a pixel's chance of standing in front goes from one half to 1.
const SKY_RAMP: f32 = 0.04;

/// The sky's least and greatest share of the picture: less is a glimpse between buildings, more
/// leaves no city to stand in front of the clock.
const SKY_SHARE: (f32, f32) = (0.1, 0.8);

/// The least share of the picture's top row the sky holds, joined to it: a sky is above the city.
const SKY_TOP_ROW: f32 = 0.5;

/// The object's least and greatest share of the picture: less is a speck, more is a scene.
const OBJECT_SHARE: (f32, f32) = (0.005, 0.6);

/// The sky's candidate from `depth`, normalised to `[0, 1]` with 1 nearest: the pixels at the
/// depth's floor ([`SKY_DEPTH`]) are the sky and everything else stands in front of it. Its
/// separation is how alone the sky sits at its depth: 1 less the pixels just past it (up to
/// [`SKY_NEAR`]) for each of its own. None when the depth holds no sky: too little or too much of
/// the picture, or not joined to its top edge.
pub fn sky(depth: &Grid) -> Option<Candidate> {
    let joined = sky_from_top(depth, SKY_DEPTH);
    let share = joined.iter().filter(|&&s| s).count() as f32 / joined.len() as f32;
    let top_row = joined[..depth.width()].iter().filter(|&&s| s).count() as f32 / depth.width() as f32;
    let floor = depth.values().iter().filter(|&&d| d < SKY_DEPTH).count();
    let near = depth
        .values()
        .iter()
        .filter(|&&d| (SKY_DEPTH..SKY_NEAR).contains(&d))
        .count();
    let separation = (1.0 - near as f32 / floor.max(1) as f32).max(0.0);
    log::debug!("sky: separation {separation:.3}, share {share:.3}, top row {top_row:.3}");
    if !(SKY_SHARE.0..=SKY_SHARE.1).contains(&share) || top_row < SKY_TOP_ROW {
        return None;
    }
    let mask = depth.map(|d| {
        if d < SKY_DEPTH {
            0.5 * d / SKY_DEPTH
        } else {
            (0.5 + 0.5 * (d - SKY_DEPTH) / SKY_RAMP).min(1.0)
        }
    });
    Some(Candidate {
        method: Method::Sky,
        mask,
        separation,
    })
}

/// The pixels farther than `threshold` joined to the top edge through each other, four ways.
fn sky_from_top(depth: &Grid, threshold: f32) -> Vec<bool> {
    let (width, height) = (depth.width(), depth.height());
    let far = |at: usize| depth.values()[at] < threshold;
    let mut sky = vec![false; width * height];
    let mut stack: Vec<usize> = (0..width).filter(|&x| far(x)).collect();
    for &at in &stack {
        sky[at] = true;
    }
    while let Some(at) = stack.pop() {
        let (x, y) = (at % width, at / width);
        let mut visit = |next: usize| {
            if !sky[next] && far(next) {
                sky[next] = true;
                stack.push(next);
            }
        };
        if x > 0 {
            visit(at - 1);
        }
        if x + 1 < width {
            visit(at + 1);
        }
        if y > 0 {
            visit(at - width);
        }
        if y + 1 < height {
            visit(at + width);
        }
    }
    sky
}

/// The object's candidate from BiRefNet's chances, its separation Otsu's between-class variance
/// of the chances as a share of their whole (`crate::grid::otsu`): 1 for an object cut clean, 0.75
/// for chances spread evenly. None when it finds no object, or one too small or too large to be a
/// single object.
pub fn object(chances: &Grid) -> Option<Candidate> {
    let (_, separation) = otsu(chances.values());
    let share = chances.share_above_half();
    log::debug!("object: separation {separation:.3}, share {share:.3}");
    if !(OBJECT_SHARE.0..=OBJECT_SHARE.1).contains(&share) {
        return None;
    }
    Some(Candidate {
        method: Method::Object,
        mask: chances.clone(),
        separation,
    })
}

/// A candidate's score and the clock's place on it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scored {
    pub method: Method,
    pub score: f32,
    /// Where the clock goes on this cut-out; none when it has no place, and its score is 0.
    pub placed: Option<Placed>,
}

/// `candidate`'s score on a `width` by `height` picture: its separation times its certainty
/// under the digits at the clock's place, the mean of `|2p - 1|` over the digits' box. 0 when the
/// separation is under [`SEPARATION_MIN`] or the clock has no place on it.
pub fn candidate_score(candidate: &Candidate, width: f32, height: f32) -> Scored {
    let none = Scored {
        method: candidate.method,
        score: 0.0,
        placed: None,
    };
    if candidate.separation < SEPARATION_MIN {
        return none;
    }
    let hard = Sums::new(&candidate.mask.map(|p| if p > 0.5 { 1.0 } else { 0.0 }));
    let Some(placed) = clock::place_for(&hard, candidate.mask.width(), width, height) else {
        return none;
    };
    let digits = clock::boxes(placed.place, width, height, clock::PRIMARY).0.digits;
    let to_mask = candidate.mask.width() as f32 / width;
    let certainty = Sums::new(&candidate.mask.map(|p| (2.0 * p - 1.0).abs())).mean(
        digits.x0 * to_mask,
        digits.y0 * to_mask,
        digits.x1 * to_mask,
        digits.y1 * to_mask,
        1.0,
    );
    Scored {
        method: candidate.method,
        score: candidate.separation * certainty,
        placed: Some(placed),
    }
}

/// The kept candidate among `scored`: the best, when it reaches [`SCORE_MIN`]. The best of all
/// is returned beside it, kept or not, for the sidecar and the log.
pub fn choose(scored: &[Scored]) -> (Option<Scored>, Option<Scored>) {
    let best = scored.iter().copied().max_by(|a, b| a.score.total_cmp(&b.score));
    (best.filter(|s| s.score >= SCORE_MIN && s.placed.is_some()), best)
}

#[cfg(test)]
mod tests;
