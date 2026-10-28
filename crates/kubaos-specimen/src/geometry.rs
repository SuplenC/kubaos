//! The geometry check of the specimen: every page laid out at every supported scale, light and
//! dark, then held to the design language's rules (`kubaos-geometry`). And parity with QML: the
//! components page as `KubaOS.Components` lays it out, held to the same rules and to the iced
//! page's boxes.
//!
//! # Owns
//!
//! - The frames of every page ([`frames`]), the rules from the tokens ([`rules`]), and the parity
//!   report ([`parity`]).
//!
//! # Not here
//!
//! - Laying one page out: [`crate::shot::frame`]. The rules themselves: `kubaos-geometry`.

use std::fs;
use std::io;
use std::path::Path;

use kubaos_geometry::Frame;
use kubaos_geometry::Rules;
use kubaos_geometry::parity;
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
        container: tokens::CONTAINER
            .iter()
            .map(|&(from, padding)| (f32::from(from), f32::from(padding.px())))
            .collect(),
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

/// What the parity check found.
#[derive(Debug, Default)]
pub struct Parity {
    /// How many QML frames were compared.
    pub frames: usize,
    /// Nodes where QML and iced part ways.
    pub differences: usize,
    /// Geometry findings in the QML frames.
    pub findings: usize,
    /// What to print: the first differences of each frame, then the findings.
    pub lines: Vec<String>,
}

/// Why the QML frames couldn't be compared.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ParityError {
    #[error("reading {}", path.display())]
    Read {
        path: std::path::PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("reading the frame in {}", path.display())]
    Frame {
        path: std::path::PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("laying the iced page out")]
    Shot(#[source] shot::Error),
}

/// The components page as `KubaOS.Components` lays it out, read from `dir` as
/// `components-MODE-SCALEx.json` in both modes at every supported scale, held to the geometry rules
/// and to the iced page's boxes, node for node.
///
/// # Errors
///
/// When a QML frame can't be read, or the iced page can't be laid out.
pub fn parity(dir: &Path) -> Result<Parity, ParityError> {
    let mut report = Parity::default();
    let mut qml_frames = Vec::new();
    for mode in [Mode::Dark, Mode::Light] {
        for scale in Scale::SUPPORTED {
            let specimen = Specimen {
                page: Page::Components,
                mode,
                scale,
                seconds: 0.0,
                live: false,
            };
            let iced = shot::frame(&specimen, SHOT_WIDTH).map_err(ParityError::Shot)?;
            let path = dir.join(format!("{}.json", iced.name()));
            let text = fs::read_to_string(&path).map_err(|source| ParityError::Read {
                path: path.clone(),
                source,
            })?;
            let qml: Frame = serde_json::from_str(&text).map_err(|source| ParityError::Frame { path, source })?;
            let differences = parity::compare(&iced, &qml, ("iced", "QML"));
            report.differences += differences.len();
            report
                .lines
                .extend(differences.iter().take(parity::SHOWN).map(ToString::to_string));
            if differences.len() > parity::SHOWN {
                report.lines.push(format!(
                    "{}: {} more difference(s)",
                    iced.name(),
                    differences.len() - parity::SHOWN
                ));
            }
            report.frames += 1;
            qml_frames.push(qml);
        }
    }
    let findings = kubaos_geometry::check(&qml_frames, &rules());
    report.findings = findings.len();
    report.lines.extend(findings.iter().map(ToString::to_string));
    Ok(report)
}

#[cfg(test)]
mod tests;
