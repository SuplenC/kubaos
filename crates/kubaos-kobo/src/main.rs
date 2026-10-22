//! `kobo`: Kōbō, the graphical front end that puts KubaOS on a machine and sets it up afterwards
//! (docs/INSTALL-INTERNALS.md, owner: kobo). The install runs in this process, through
//! `kubaos_core::engine`.
//!
//!   kobo [--fullscreen]
//!   kobo --demo [--demo-speed N] [--demo-fail-at STEP]
//!   kobo --demo --screenshot-tour DIR
//!   kobo --setup [--first-boot]        first-boot setup (setup/)
//!   kobo --setup --demo [--demo-machine tuxedo|vm] [--screenshot-tour DIR]
//!
//! Also: --size WxH (window size), --scale F (UI scale; picked from the
//! window size when not given, see `app::auto_scale`), --tour-pace F
//! (slow the tour down for screen recordings).

mod app;
mod backend;
mod battery;
mod data;
mod edit;
mod elide;
mod field;
#[cfg(test)]
mod gallery;
mod model;
mod pointer;
mod progress;
mod reveal;
mod setup;
mod theme;
mod tour;
mod view;
mod widgets;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use iced::{Size, Theme, window};
use kubaos_core::StepId;

use crate::app::{App, Options};
use crate::backend::{Backend, Demo, Engine};

const USAGE: &str = "usage: kobo [--demo [--demo-speed N] [--demo-fail-at STEP]]
            [--fullscreen] [--size WxH] [--scale F] [--screenshot-tour DIR]
       kobo --setup [--first-boot] [--demo [--demo-machine tuxedo|vm]]";

struct Args {
    demo: bool,
    demo_speed: Option<f32>,
    demo_fail_at: Option<StepId>,
    fullscreen: bool,
    setup: bool,
    first_boot: bool,
    demo_vm: bool,
    size: Size,
    scale: Option<f32>,
    tour: Option<PathBuf>,
    tour_pace: f32,
}

fn parse() -> Result<Args, String> {
    let mut a = Args {
        demo: false,
        demo_speed: None,
        demo_fail_at: None,
        fullscreen: false,
        setup: false,
        first_boot: false,
        demo_vm: false,
        size: Size::new(1280.0, 800.0),
        scale: None,
        tour: None,
        tour_pace: 1.0,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut val = || it.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--demo" => a.demo = true,
            "--demo-speed" => {
                a.demo_speed = Some(val()?.parse().map_err(|_| "--demo-speed: not a number")?)
            }
            "--demo-fail-at" => {
                let v = val()?;
                a.demo_fail_at = Some(
                    serde_json::from_value(serde_json::Value::String(v.clone()))
                        .map_err(|_| format!("--demo-fail-at: unknown step {v}"))?,
                );
            }
            "--fullscreen" => a.fullscreen = true,
            "--setup" => a.setup = true,
            "--first-boot" => a.first_boot = true,
            "--demo-machine" => {
                a.demo_vm = match val()?.as_str() {
                    "vm" => true,
                    "tuxedo" => false,
                    v => return Err(format!("--demo-machine: tuxedo or vm, not {v}")),
                }
            }
            "--size" => {
                let v = val()?;
                let (w, h) = v.split_once('x').ok_or("--size: expected WxH")?;
                a.size = Size::new(
                    w.parse().map_err(|_| "--size: bad width")?,
                    h.parse().map_err(|_| "--size: bad height")?,
                );
            }
            "--scale" => a.scale = Some(val()?.parse().map_err(|_| "--scale: not a number")?),
            "--screenshot-tour" => a.tour = Some(PathBuf::from(val()?)),
            "--tour-pace" => {
                a.tour_pace = val()?.parse().map_err(|_| "--tour-pace: not a number")?
            }
            "-h" | "--help" => return Err(String::new()),
            other => return Err(format!("unknown flag {other}")),
        }
    }
    if a.tour.is_some() && !a.demo {
        return Err("--screenshot-tour needs --demo".into());
    }
    Ok(a)
}

fn main() -> ExitCode {
    let args = match parse() {
        Ok(a) => a,
        Err(e) => {
            if !e.is_empty() {
                eprintln!("kobo: {e}");
            }
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };

    if args.setup {
        return run_setup(args);
    }

    let backend: Arc<dyn Backend> = if args.demo {
        let speed = args
            .demo_speed
            .unwrap_or(if args.tour.is_some() { 4.0 } else { 1.0 });
        // The tour shows the Failed screen and Retry, so its first run fails.
        let fail_at = args
            .demo_fail_at
            .or(args.tour.is_some().then_some(StepId::Home));
        Arc::new(Demo::new(speed, fail_at))
    } else {
        backend::stop_install_on_signals();
        Arc::new(Engine::live())
    };
    let opts = Options {
        tour: args.tour.clone(),
        tour_pace: args.tour_pace,
        fullscreen: args.fullscreen,
        scale: args.scale,
    };

    let mut app = iced::application(
        move || App::new(backend.clone(), opts.clone()),
        App::update,
        view::view,
    )
    .title(|app: &App| format!("Kōbō · KubaOS {}", app.codename()))
    .subscription(App::subscription)
    .theme(|_: &App| Theme::custom("dotfiles", theme::iced_palette()))
    .window(window::Settings {
        size: args.size,
        min_size: Some(Size::new(900.0, 600.0)),
        // No client-side decorations: under cage (and in a tiling compositor)
        // winit's fallback title bar ate ~36 px off the bottom of the output.
        decorations: !cfg!(target_os = "linux"),
        ..window::Settings::default()
    })
    .scale_factor(|app: &App| app.ui_scale)
    .default_font(theme::SANS)
    .antialiasing(true);
    for f in theme::FONTS {
        app = app.font(*f);
    }
    match app.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("kobo: {e}");
            ExitCode::FAILURE
        }
    }
}

/// `--setup`: the first-boot setup app (setup/mod.rs), as the desktop user.
fn run_setup(args: Args) -> ExitCode {
    use kubaos_core::setup::demo;
    use setup::backend::{Demo as SetupDemo, Real, SetupBackend};
    use std::sync::Mutex;

    let backend: Arc<dyn SetupBackend> = if args.demo {
        let speed = args
            .demo_speed
            .unwrap_or(if args.tour.is_some() { 2.5 } else { 1.0 });
        let machine = if args.demo_vm {
            demo::Machine::Vm
        } else {
            demo::Machine::Tuxedo
        };
        Arc::new(SetupDemo {
            demo: demo::Demo::new(machine, speed),
        })
    } else {
        Arc::new(Real::new())
    };
    // The login autostart: already finished or dismissed, stay closed.
    if args.first_boot && backend.marker_exists() {
        return ExitCode::SUCCESS;
    }
    let tour = Mutex::new(
        args.tour
            .clone()
            .map(|d| setup::tour::Tour::new(d, args.tour_pace, args.demo_vm)),
    );
    let scale = args.scale;
    #[allow(unused_mut)]
    let mut settings = window::Settings {
        size: args.size,
        min_size: Some(Size::new(900.0, 600.0)),
        decorations: !cfg!(target_os = "linux"),
        // Closing the window counts as "done for now" (setup writes its
        // marker first), so it handles the request itself.
        exit_on_close_request: false,
        ..window::Settings::default()
    };
    #[cfg(target_os = "linux")]
    {
        settings.platform_specific.application_id = "kobo".into();
    }
    let mut app = iced::application(
        move || setup::Setup::new(backend.clone(), tour.lock().ok().and_then(|mut t| t.take())),
        setup::Setup::update,
        setup::view,
    )
    .title("Kōbō · Setup")
    .subscription(setup::Setup::subscription)
    .theme(|_: &setup::Setup| Theme::custom("dotfiles", theme::iced_palette()))
    .window(settings)
    .scale_factor(move |_| scale.unwrap_or(1.0))
    .default_font(theme::SANS)
    .antialiasing(true);
    for f in theme::FONTS {
        app = app.font(*f);
    }
    match app.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("kobo: {e}");
            ExitCode::FAILURE
        }
    }
}
