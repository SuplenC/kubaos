//! `kubaos-specimen`: every token and component of `kubaos-ui`, in a window or as screenshots.
//!
//! ```text
//! kubaos-specimen                  the live specimen, in a window
//! kubaos-specimen shots DIR        every page, light and dark, at 1, 1.25, 1.5 and 2, as PNG in DIR
//! kubaos-specimen check [DIR]      the same pages held to the geometry rules; DIR gets their frames
//! kubaos-specimen parity DIR       the QML specimen's frames in DIR held to the components page's
//! ```
//!
//! # Owns
//!
//! - The command line and its exit codes: 2 for a usage error, 1 when the window or a screenshot
//!   fails, or a page breaks a geometry rule.
//!
//! # Not here
//!
//! - The window: [`app`]. The screenshots: [`shot`]. The geometry check and parity with QML:
//!   [`geometry`]. The pages: [`pages`].

mod app;
mod geometry;
mod pages;
mod shot;
mod specimen;

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use kubaos_ui::Mode;
use kubaos_ui::Scale;

use crate::specimen::Page;
use crate::specimen::Specimen;

const USAGE: &str = "\
usage: kubaos-specimen
       kubaos-specimen shots DIR
       kubaos-specimen check [DIR]
       kubaos-specimen parity DIR

With no arguments, the live specimen opens in a window: Tab or the arrows move between pages,
M switches light and dark, Esc closes it.

shots  every page in light and dark at 1, 1.25, 1.5 and 2, as PNG files in DIR,
       named PAGE-MODE-SCALE.png (type-dark-1.25x.png), each listing the boxes whose
       edges fall between device pixels
check  the same pages, light and dark at every scale, held to the geometry rules (docs/DESIGN.md,
       \"Checks\"); exit 1 on any finding. With DIR, each page's frame is written there as
       PAGE-MODE-SCALE.json, the input `kubaos-design geometry` reads
parity the components page as `KubaOS.Components` draws it (qml/specimen/), read from DIR as
       components-MODE-SCALEx.json for both modes at every scale: each frame held to the geometry
       rules and to the iced page's boxes, node for node; exit 1 on any difference";

/// How many of a screenshot's off-grid boxes are listed; the count says how many there are.
const AUDIT_SHOWN: usize = 5;

/// The width the screenshots are laid out at, in logical pixels: Kōbō's panel on a 1366 px laptop.
pub const SHOT_WIDTH: f32 = 1280.0;

#[expect(
    clippy::print_stderr,
    reason = "The failure and the usage are the command's answer to the developer who ran it."
)]
fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        [] => match app::run() {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("kubaos-specimen: {err}");
                ExitCode::FAILURE
            }
        },
        ["shots", dir] => shots(Path::new(dir)),
        ["check"] => check(None),
        ["check", dir] => check(Some(Path::new(dir))),
        ["parity", dir] => parity(Path::new(dir)),
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The shots written and their off-grid boxes are the command's report, for the developer who ran it."
)]
fn shots(dir: &Path) -> ExitCode {
    if let Err(err) = fs::create_dir_all(dir) {
        eprintln!("kubaos-specimen: creating {}: {err}", dir.display());
        return ExitCode::FAILURE;
    }
    shot::load_fonts();
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
                let mode_name = shot::mode_name(mode);
                let path = dir.join(format!("{}-{mode_name}-{}x.png", page.slug(), scale.factor()));
                let result = shot::render(&specimen, SHOT_WIDTH)
                    .and_then(|image| shot::write_png(&path, &image).map(|()| image.off_grid));
                match result {
                    Ok(off_grid) if off_grid.is_empty() => println!("{}", path.display()),
                    Ok(off_grid) => {
                        println!("{}: {} boxes off the device grid", path.display(), off_grid.len());
                        for bounds in off_grid.iter().take(AUDIT_SHOWN) {
                            println!("  {bounds:?}");
                        }
                    }
                    Err(err) => {
                        eprintln!("kubaos-specimen: {err}");
                        return ExitCode::FAILURE;
                    }
                }
            }
        }
    }
    ExitCode::SUCCESS
}

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The findings and the verdict are the check's output, for the developer or the CI log."
)]
fn check(dir: Option<&Path>) -> ExitCode {
    shot::load_fonts();
    let frames = match geometry::frames() {
        Ok(frames) => frames,
        Err(err) => {
            eprintln!("kubaos-specimen: {err}");
            return ExitCode::FAILURE;
        }
    };
    if let Some(dir) = dir
        && let Err(err) = write_frames(dir, &frames)
    {
        eprintln!("kubaos-specimen: writing the frames to {}: {err}", dir.display());
        return ExitCode::FAILURE;
    }
    let findings = kubaos_geometry::check(&frames, &geometry::rules());
    for finding in &findings {
        println!("{finding}");
    }
    eprintln!(
        "kubaos-specimen check: {} frame(s), {} finding(s)",
        frames.len(),
        findings.len()
    );
    if findings.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The differences and the verdict are the check's output, for the developer or the CI log."
)]
fn parity(dir: &Path) -> ExitCode {
    shot::load_fonts();
    match geometry::parity(dir) {
        Ok(report) => {
            for line in &report.lines {
                println!("{line}");
            }
            eprintln!(
                "kubaos-specimen parity: {} frame(s), {} difference(s), {} finding(s)",
                report.frames, report.differences, report.findings
            );
            if report.differences == 0 && report.findings == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(err) => {
            eprintln!("kubaos-specimen: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Each frame as `PAGE-MODE-SCALE.json` in `dir`.
fn write_frames(dir: &Path, frames: &[kubaos_geometry::Frame]) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    for frame in frames {
        let json = serde_json::to_string(frame).map_err(std::io::Error::other)?;
        fs::write(dir.join(format!("{}.json", frame.name())), json)?;
    }
    Ok(())
}
