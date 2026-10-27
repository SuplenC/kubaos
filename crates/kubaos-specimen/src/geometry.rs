//! The geometry check of the specimen: every page laid out at every supported scale, light and
//! dark, then held to the design language's rules (`kubaos-geometry`).
//!
//! # Owns
//!
//! - The frames of every page ([`frames`]) and the rules from the tokens ([`rules`]).
//!
//! # Not here
//!
//! - Laying one page out: [`crate::shot::frame`]. The rules themselves: `kubaos-geometry`.

use kubaos_geometry::Frame;
use kubaos_geometry::Rules;
use kubaos_ui::Mode;
use kubaos_ui::Scale;
use kubaos_ui::tokens;
use kubaos_ui::tokens::Space;

use crate::SHOT_WIDTH;
use crate::shot;
use crate::specimen::Page;
use crate::specimen::Specimen;

/// The rules of the tokens: the grid and the spacing scale.
pub fn rules() -> Rules {
    Rules {
        grid: f32::from(tokens::GRID),
        spacing: Space::ALL.iter().map(|space| f32::from(space.px())).collect(),
    }
}

/// Every page, dark then light, at every supported scale, laid out as the screenshots are.
///
/// # Errors
///
/// As [`shot::frame`].
pub fn frames() -> Result<Vec<Frame>, shot::Error> {
    let mut frames = Vec::new();
    for page in Page::ALL {
        for mode in [Mode::Dark, Mode::Light] {
            for scale in Scale::SUPPORTED {
                let specimen = Specimen {
                    page,
                    mode,
                    scale,
                    seconds: 0.0,
                    live: false,
                };
                frames.push(shot::frame(&specimen, SHOT_WIDTH)?);
            }
        }
    }
    Ok(frames)
}

#[cfg(test)]
mod tests;
