//! `kubaos depth`: the wallpaper-depth settings, and the Suite's runs they start and stop.
//!
//!   kubaos depth on               depth for your own pictures; queues the current wallpaper
//!   kubaos depth off              no depth for them; stops every waiting or running run
//!   kubaos depth redo [PICTURE]   make the cut-out again, even when it is up to date
//!   kubaos depth skip [PICTURE]   no cut-out on the picture; stops its run, removes its results
//!   kubaos depth unskip [PICTURE] cut the picture out again; queues it
//!   kubaos depth push <0-0.05>    how far the wallpaper comes toward you in the lock motion
//!   kubaos depth show             print the settings, then the skipped pictures, one a line
//!
//! PICTURE is the current wallpaper (`dot-theme current`) when left out. The settings are
//! `~/.config/kubaos/desktop/wallpapers/depth.toml`; each command rewrites its line there and
//! applies at once. The runs belong to the Suite's `kubaos-depth-queue`
//! (`home/suites/wallpaper-depth-queue.sh`), which asks `kubaos depth wants PICTURE` before it
//! queues one; without the Suite, a command changes the file alone, and `redo` refuses. The
//! Command menu's wallpaper-depth Items run these.
//! Exit codes: 0 ok, 1 the file or the queue failed, or the picture gets no cut-out, 2 usage.
//!
//! # Owns
//!
//! - Where the files are, and which of the queue's commands each change runs.
//!
//! # Not here
//!
//! - The settings file and which pictures get a cut-out: `kubaos_core::wallpaper_depth`.
//! - The units, the results' folders and the tool: `kubaos-depth-queue` and `crates/kubaos-depth`.

use std::env;
use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;

use clap::Subcommand;
use kubaos_core::wallpaper_depth;
use kubaos_core::wallpaper_depth::Settings;
use kubaos_core::wallpaper_depth::SettingsError;

use crate::job::chain;
use crate::out::err;
use crate::out::out;
use crate::usage_error;

/// The Suite's queue command, on PATH while the Suite is installed.
const QUEUE: &str = "kubaos-depth-queue";

/// KubaOS's wallpaper sets, as `scripts/dot-theme` finds them (`SETS_DIR`).
const SETS_DIR: &str = "/usr/share/kubaos/wallpapers";

#[derive(Subcommand)]
pub enum DepthCmd {
    /// Turn depth on for your own pictures, and queue the current wallpaper.
    On,
    /// Turn depth off for your own pictures, and stop every run waiting or running.
    Off,
    /// Make the picture's cut-out again, even when it is up to date (the current wallpaper by
    /// default).
    Redo { picture: Option<PathBuf> },
    /// Give the picture no cut-out, shipped or your own (the current wallpaper by default).
    Skip { picture: Option<PathBuf> },
    /// Cut the picture out again after `skip` (the current wallpaper by default).
    Unskip { picture: Option<PathBuf> },
    /// How far the wallpaper comes toward you as the lock comes up: the share the picture grows
    /// by, 0 to 0.05.
    Push { share: f64 },
    /// Print the settings, then the skipped pictures, one a line.
    Show,
    /// Exit 0 when the Suite cuts PICTURE out; else print why and exit 1.
    #[command(hide = true)]
    Wants { picture: PathBuf },
}

pub fn main(cmd: DepthCmd) -> ExitCode {
    let paths = match Paths::from_env() {
        Ok(paths) => paths,
        Err(msg) => return usage_error(msg),
    };
    match cmd {
        DepthCmd::On => on(&paths),
        DepthCmd::Off => off(&paths),
        DepthCmd::Redo { picture } => with_picture(&paths, picture, redo),
        DepthCmd::Skip { picture } => with_picture(&paths, picture, skip),
        DepthCmd::Unskip { picture } => with_picture(&paths, picture, unskip),
        DepthCmd::Push { share } => match wallpaper_depth::set_push(&paths.settings, share) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e @ SettingsError::Push(_)) => usage_error(e),
            Err(e) => failed(&e),
        },
        DepthCmd::Show => show(&paths),
        DepthCmd::Wants { picture } => wants(&paths, &picture),
    }
}

/// Where the settings and the current wallpaper are.
struct Paths {
    /// `~/.config/kubaos/desktop/wallpapers/depth.toml`.
    settings: PathBuf,
    /// The theme's record of the current wallpaper.
    wallpaper: PathBuf,
    /// KubaOS's wallpaper sets.
    sets: PathBuf,
}

impl Paths {
    fn from_env() -> Result<Self, String> {
        let home = env::var_os("HOME")
            .filter(|home| !home.is_empty())
            .map(PathBuf::from)
            .ok_or("HOME isn't set")?;
        let config = kubaos_user_layer::config_home().ok_or("HOME isn't set")?;
        Ok(Paths {
            settings: wallpaper_depth::path(&config),
            wallpaper: wallpaper_depth::wallpaper_file(&home),
            sets: env::var_os("KUBAOS_WALLPAPERS")
                .filter(|dir| !dir.is_empty())
                .map_or_else(|| PathBuf::from(SETS_DIR), PathBuf::from),
        })
    }
}

fn load_settings(paths: &Paths) -> Settings {
    Settings::load(&paths.settings).unwrap_or_else(|e| {
        err!("kubaos: {}; wallpaper depth takes its defaults", chain(&e));
        Settings::default()
    })
}

fn failed(e: &dyn std::error::Error) -> ExitCode {
    err!("kubaos: {}", chain(e));
    ExitCode::FAILURE
}

/// Runs `command` on `picture`, made absolute, or on the current wallpaper without one.
fn with_picture(paths: &Paths, picture: Option<PathBuf>, command: fn(&Paths, &Path) -> ExitCode) -> ExitCode {
    let picture = match picture {
        Some(picture) => match std::path::absolute(&picture) {
            Ok(picture) => picture,
            Err(e) => return usage_error(format!("{}: {e}", picture.display())),
        },
        None => match wallpaper_depth::current_wallpaper(&paths.wallpaper) {
            Ok(Some(picture)) => picture,
            Ok(None) => {
                err!("kubaos: no wallpaper is set");
                return ExitCode::FAILURE;
            }
            Err(e) => {
                err!(
                    "kubaos: reading the current wallpaper from {}: {e}",
                    paths.wallpaper.display()
                );
                return ExitCode::FAILURE;
            }
        },
    };
    command(paths, &picture)
}

fn on(paths: &Paths) -> ExitCode {
    if let Err(e) = wallpaper_depth::set_enabled(&paths.settings, true) {
        return failed(&e);
    }
    match wallpaper_depth::current_wallpaper(&paths.wallpaper) {
        Ok(Some(picture)) if load_settings(paths).refusal(&picture, &paths.sets).is_none() => {
            queue_quietly(&[picture.as_os_str()])
        }
        _ => ExitCode::SUCCESS,
    }
}

fn off(paths: &Paths) -> ExitCode {
    if let Err(e) = wallpaper_depth::set_enabled(&paths.settings, false) {
        return failed(&e);
    }
    queue_quietly(&["stop".as_ref()])
}

fn redo(paths: &Paths, picture: &Path) -> ExitCode {
    if let Some(refusal) = load_settings(paths).refusal(picture, &paths.sets) {
        err!("kubaos: no cut-out for {}: {refusal}", picture.display());
        return ExitCode::FAILURE;
    }
    match run_queue(&["redo".as_ref(), picture.as_os_str()]) {
        Ok(code) => code,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            err!("kubaos: the wallpaper-depth Suite isn't installed (kubaos suite add wallpaper-depth)");
            ExitCode::FAILURE
        }
        Err(e) => failed(&e),
    }
}

fn skip(paths: &Paths, picture: &Path) -> ExitCode {
    if let Err(e) = wallpaper_depth::skip(&paths.settings, picture) {
        return failed(&e);
    }
    queue_quietly(&["forget".as_ref(), picture.as_os_str()])
}

fn unskip(paths: &Paths, picture: &Path) -> ExitCode {
    if let Err(e) = wallpaper_depth::unskip(&paths.settings, picture) {
        return failed(&e);
    }
    if load_settings(paths).refusal(picture, &paths.sets).is_some() {
        return ExitCode::SUCCESS;
    }
    queue_quietly(&[picture.as_os_str()])
}

fn show(paths: &Paths) -> ExitCode {
    let settings = load_settings(paths);
    out!("enabled={} push={}", settings.enabled, settings.push);
    for picture in &settings.skip {
        out!("{}", picture.display());
    }
    ExitCode::SUCCESS
}

fn wants(paths: &Paths, picture: &Path) -> ExitCode {
    match load_settings(paths).refusal(picture, &paths.sets) {
        None => ExitCode::SUCCESS,
        Some(refusal) => {
            out!("{refusal}");
            ExitCode::FAILURE
        }
    }
}

/// Runs the queue with `args`, its output the caller's; its exit code, or why it didn't start.
fn run_queue(args: &[&OsStr]) -> io::Result<ExitCode> {
    let status = Command::new(QUEUE).args(args).status()?;
    Ok(if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

/// [`run_queue`], with no Suite meaning nothing to queue or stop.
fn queue_quietly(args: &[&OsStr]) -> ExitCode {
    match run_queue(args) {
        Ok(code) => code,
        Err(e) if e.kind() == io::ErrorKind::NotFound => ExitCode::SUCCESS,
        Err(e) => {
            err!("kubaos: running {QUEUE}: {e}");
            ExitCode::FAILURE
        }
    }
}
