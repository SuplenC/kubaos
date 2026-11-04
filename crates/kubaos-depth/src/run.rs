//! One picture's run: the models, the choice, the edge, the clock, and the files written.
//!
//! # Owns
//!
//! - [`run`], which writes a picture's folder, and its error ([`Error`]).
//!
//! # Not here
//!
//! - The command line and the process's priority: `src/main.rs`.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

use image::ExtendedColorType;
use image::ImageEncoder;
use image::RgbImage;
use image::codecs::png::CompressionType;
use image::codecs::png::FilterType;
use image::codecs::png::PngEncoder;

use crate::choose;
use crate::choose::Candidate;
use crate::choose::Method;
use crate::clock;
use crate::clock::DEFAULT_PLACE;
use crate::clock::Placed;
use crate::edge;
use crate::grid::Grid;
use crate::grid::Sums;
use crate::models;
use crate::models::Models;
use crate::models::Sessions;
use crate::sidecar::CUTOUT_FILE;
use crate::sidecar::Sidecar;
use crate::tint;

/// The width the masks are compared, scored and merged at: SAM's and BiRefNet's own.
const WORK_WIDTH: usize = 1024;

/// A picture's run failed.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("reading {}", .0.display())]
    Picture(PathBuf, #[source] image::ImageError),
    #[error("finding the models")]
    Models(#[source] models::ModelsError),
    #[error("running the models")]
    Model(#[source] models::RunError),
    #[error("writing {}", .0.display())]
    Write(PathBuf, #[source] io::Error),
}

/// Makes `picture`'s files in `out` (created if it isn't there) with the models in `models`, the
/// runtime on `threads` of the CPU, and returns the sidecar written. A cut-out from an earlier run
/// is removed when none is kept this time.
///
/// # Errors
///
/// [`Error`] for the step that failed. The folder's files are each written whole or not at all.
pub fn run(picture: &Path, out: &Path, models: &Path, threads: usize) -> Result<Sidecar, Error> {
    let started = Instant::now();
    let models = Models::in_dir(models).map_err(Error::Models)?;
    let image = image::open(picture)
        .map_err(|e| Error::Picture(picture.to_owned(), e))?
        .to_rgb8();
    let (width, height) = (image.width() as f32, image.height() as f32);
    log::info!("{}: {}×{}", picture.display(), image.width(), image.height());
    let mut sessions = Sessions::load(&models, threads).map_err(Error::Model)?;
    let work = (
        WORK_WIDTH,
        ((WORK_WIDTH as f32 * height / width).round() as usize).max(1),
    );
    log::info!("models loaded in {:.1} s", started.elapsed().as_secs_f32());

    let step = Instant::now();
    let depth = sessions.depth(&image).map_err(Error::Model)?.resized(work.0, work.1);
    log::info!("depth in {:.1} s", step.elapsed().as_secs_f32());
    let step = Instant::now();
    let object = sessions.object(&image).map_err(Error::Model)?.resized(work.0, work.1);
    log::info!("object in {:.1} s", step.elapsed().as_secs_f32());

    let candidates: Vec<Candidate> = [choose::sky(&depth), choose::object(&object)]
        .into_iter()
        .flatten()
        .collect();
    let scored: Vec<_> = candidates
        .iter()
        .map(|c| choose::candidate_score(c, width, height))
        .collect();
    for s in &scored {
        log::info!(
            "{:?}: score {:.3}, cover {:?}",
            s.method,
            s.score,
            s.placed.map(|p| p.cover)
        );
    }
    let (kept, best) = choose::choose(&scored);
    let best_score = best.map_or(0.0, |s| s.score);
    let cutout_path = out.join(CUTOUT_FILE);
    fs::create_dir_all(out).map_err(|e| Error::Write(out.to_owned(), e))?;

    let kept = kept.and_then(|kept| {
        let candidate = candidates.iter().find(|c| c.method == kept.method)?;
        Some((candidate, kept.placed?))
    });
    let (method, place) = if let Some((candidate, placed)) = kept {
        let step = Instant::now();
        let (mask, placed) = traced(&mut sessions, &image, candidate, placed).map_err(Error::Model)?;
        log::info!("edge in {:.1} s", step.elapsed().as_secs_f32());
        let step = Instant::now();
        drop(sessions);
        write_cutout(&image, candidate.method, &mask, &cutout_path)
            .map_err(|e| Error::Write(cutout_path.clone(), e))?;
        log::info!("cut-out written in {:.1} s", step.elapsed().as_secs_f32());
        (Some(candidate.method), placed.place)
    } else {
        log::info!(
            "no cut-out: the best score is {best_score:.3}, under {}",
            choose::SCORE_MIN
        );
        if let Err(e) = fs::remove_file(&cutout_path)
            && e.kind() != io::ErrorKind::NotFound
        {
            return Err(Error::Write(cutout_path, e));
        }
        (None, DEFAULT_PLACE)
    };

    let digits = clock::boxes(place, width, height, clock::PRIMARY).0.digits;
    let sidecar = Sidecar::new(method, best_score, place, tint::tint_under(&image, digits));
    sidecar
        .write(out)
        .map_err(|e| Error::Write(out.join(crate::sidecar::SIDECAR_FILE), e))?;
    log::info!("done in {:.1} s", started.elapsed().as_secs_f32());
    Ok(sidecar)
}

/// The kept candidate's mask with SAM's edge, when SAM finds the same region and the clock still
/// has a place on it, and the clock's place on the mask returned; else the coarse mask and its
/// place.
///
/// Only the sky's is traced. BiRefNet's edge is finer than SAM's: SAM's mask comes from 256-pixel
/// logits, BiRefNet's from 1024, and on the shipped wallpapers SAM's tower lost its spire's lattice
/// and smeared its legs.
fn traced(
    sessions: &mut Sessions,
    image: &RgbImage,
    candidate: &Candidate,
    placed: Placed,
) -> Result<(Grid, Placed), models::RunError> {
    if candidate.method == Method::Object {
        return Ok((candidate.mask.clone(), placed));
    }
    let region = edge::region(candidate.method, &candidate.mask);
    let prompt = edge::prompt_for(candidate.method, &region);
    let (traced, fit) = sessions.edge(image, &prompt)?;
    let traced = traced.resized(region.width(), region.height());
    let iou = edge::iou(&traced, &region);
    log::info!("SAM: fit {fit:.3}, IoU with the coarse mask {iou:.3}");
    if fit < edge::FIT_MIN || iou < edge::IOU_MIN {
        log::info!("SAM's region isn't the cut-out's: the coarse edge stays");
        return Ok((candidate.mask.clone(), placed));
    }
    let merged = edge::merge(candidate.method, &candidate.mask, &traced);
    let hard = Sums::new(&merged.map(|p| if p > 0.5 { 1.0 } else { 0.0 }));
    if let Some(placed) = clock::place_for(&hard, merged.width(), image.width() as f32, image.height() as f32) {
        Ok((merged, placed))
    } else {
        log::info!("the clock has no place on SAM's edge: the coarse edge stays");
        Ok((candidate.mask.clone(), placed))
    }
}

/// Writes `image` with `mask` as its alpha, at the image's size, to `path` through a temporary
/// file. Where the cut-out is clear its colour is black, which the lock never shows and the PNG
/// compresses to almost nothing.
fn write_cutout(image: &RgbImage, method: Method, mask: &Grid, path: &Path) -> io::Result<()> {
    let (width, height) = (image.width() as usize, image.height() as usize);
    let alpha = mask.resized(width, height);
    let alpha = match method {
        Method::Sky => edge::key_sky(image, &alpha),
        Method::Object => alpha,
    };
    let mut rgba = Vec::with_capacity(width * height * 4);
    for (pixel, &a) in image.pixels().zip(alpha.values()) {
        let a = (a.clamp(0.0, 1.0) * 255.0).round() as u8;
        if a == 0 {
            rgba.extend([0, 0, 0, 0]);
        } else {
            rgba.extend([pixel.0[0], pixel.0[1], pixel.0[2], a]);
        }
    }
    let partial = path.with_extension("png.partial");
    let file = io::BufWriter::new(fs::File::create(&partial)?);
    PngEncoder::new_with_quality(file, CompressionType::Fast, FilterType::Adaptive)
        .write_image(&rgba, image.width(), image.height(), ExtendedColorType::Rgba8)
        .map_err(io::Error::other)?;
    fs::rename(partial, path)
}
