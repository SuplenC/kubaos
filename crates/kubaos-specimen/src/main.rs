//! `kubaos-specimen`: every token and component of `kubaos-ui`, in a window or as screenshots.
//!
//! ```text
//! kubaos-specimen                  the live specimen, in a window
//! kubaos-specimen shots DIR        every page, light and dark, at 1, 1.25, 1.5 and 2, as PNG in DIR
//! ```
//!
//! # Owns
//!
//! - The command line and its exit codes: 2 for a usage error, 1 when the window or a screenshot
//!   fails.
//!
//! # Not here
//!
//! - The window: [`app`]. The screenshots: [`shot`]. The pages: [`pages`].

#![forbid(unsafe_code)]

mod app;
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

With no arguments, the live specimen opens in a window: Tab or the arrows move between pages,
M switches light and dark, Esc closes it.

shots  every page in light and dark at 1, 1.25, 1.5 and 2, as PNG files in DIR,
       named PAGE-MODE-SCALE.png (type-dark-1.25x.png), each listing the boxes whose
       edges fall between device pixels";

/// How many of a screenshot's off-grid boxes are listed; the count says how many there are.
const AUDIT_SHOWN: usize = 5;

/// The width the screenshots are laid out at, in logical pixels: Kōbō's panel on a 1366 px laptop.
const SHOT_WIDTH: f32 = 1280.0;

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
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

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
                let mode_name = match mode {
                    Mode::Dark => "dark",
                    Mode::Light => "light",
                };
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
