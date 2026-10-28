//! `kobo`: Kōbō, the graphical front end that puts KubaOS on a machine and sets it up afterwards
//! (docs/INSTALL-INTERNALS.md, owner: kobo). The install runs in this process, through
//! `kubaos_core::engine`.
//!
//!   kobo [--fullscreen]
//!   kobo --demo [--demo-speed N] [--demo-fail-at STEP]
//!   kobo --demo --screenshot-tour DIR
//!   kobo --setup [--first-boot] [--card CARD]   first-boot setup (setup/)
//!   kobo --setup --demo [--demo-machine tuxedo|vm|installed] [--screenshot-tour DIR]
//!
//! `--card` opens setup on one card, the way the Command menu opens the Bitwarden one.
//!
//! Also: `--size WxH` (window size), `--scale F` (UI scale; picked from the
//! window size when not given, see `app::auto_scale`), `--tour-pace F`
//! (slow the tour down for screen recordings).

mod app;
mod backend;
mod battery;
mod data;
mod edit;
mod model;
mod pointer;
mod progress;
mod reveal;
mod setup;
#[cfg(test)]
mod shots;
mod theme;
mod thumbs;
mod tour;
mod ui;
mod view;

use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use iced::Size;
use iced::window;
use kubaos_core::StepId;

use crate::app::App;
use crate::app::Options;
use crate::backend::Backend;
use crate::backend::Demo;
use crate::backend::Engine;

const USAGE: &str = "usage: kobo [--demo [--demo-speed N] [--demo-fail-at STEP]]
            [--fullscreen] [--size WxH] [--scale F] [--screenshot-tour DIR]
       kobo --setup [--first-boot] [--card CARD]
            [--demo [--demo-machine tuxedo|vm|installed]]";

#[expect(
    clippy::struct_excessive_bools,
    reason = "each bool is one command-line flag, read once at startup"
)]
struct Args {
    demo: bool,
    demo_speed: Option<f32>,
    demo_fail_at: Option<StepId>,
    fullscreen: bool,
    setup: bool,
    first_boot: bool,
    card: Option<kubaos_core::setup::CardId>,
    demo_machine: kubaos_core::setup::demo::Machine,
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
        card: None,
        demo_machine: kubaos_core::setup::demo::Machine::Tuxedo,
        // design-allow(size): the window's opening size; tiling sets the real one.
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
            "--demo-speed" => a.demo_speed = Some(val()?.parse().map_err(|_| "--demo-speed: not a number")?),
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
                use kubaos_core::setup::demo::Machine;
                a.demo_machine = match val()?.as_str() {
                    "vm" => Machine::Vm,
                    "tuxedo" => Machine::Tuxedo,
                    "installed" => Machine::Installed,
                    v => return Err(format!("--demo-machine: tuxedo, vm or installed, not {v}")),
                }
            }
            "--card" => {
                let v = val()?;
                a.card =
                    Some(kubaos_core::setup::CardId::parse(&v).ok_or_else(|| format!("--card: unknown card {v}"))?);
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
            "--tour-pace" => a.tour_pace = val()?.parse().map_err(|_| "--tour-pace: not a number")?,
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
    if log::set_logger(&Stderr).is_ok() {
        log::set_max_level(log::LevelFilter::Warn);
    }
    let args = match parse() {
        Ok(a) => a,
        Err(e) => {
            if !e.is_empty() {
                say(&format!("kobo: {e}"));
            }
            say(USAGE);
            return ExitCode::from(2);
        }
    };

    if args.setup {
        return run_setup(&args);
    }

    let backend: Arc<dyn Backend> = if args.demo {
        let speed = args.demo_speed.unwrap_or(if args.tour.is_some() { 4.0 } else { 1.0 });
        // The tour shows the Failed screen and Retry, so its first run fails.
        let fail_at = args.demo_fail_at.or(args.tour.is_some().then_some(StepId::Home));
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

    let mut app = iced::application(move || App::new(Arc::clone(&backend), &opts), App::update, view::view)
        .title(|app: &App| format!("Kōbō · KubaOS {}", app.codename()))
        .subscription(App::subscription)
        .theme(|app: &App| ui::theme(&app.look()))
        .window(window::Settings {
            size: args.size,
            min_size: Some(Size::new(900.0, 600.0)),
            // No client-side decorations: under cage (and in a tiling compositor)
            // winit's fallback title bar ate ~36 px off the bottom of the output.
            decorations: !cfg!(target_os = "linux"),
            ..window::Settings::default()
        })
        .scale_factor(|app: &App| app.ui_scale)
        .default_font(kubaos_ui::text::default_font())
        .antialiasing(true);
    for f in kubaos_ui::text::FONTS {
        app = app.font(*f);
    }
    match app.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            say(&format!("kobo: {e}"));
            ExitCode::FAILURE
        }
    }
}

/// `--setup`: the first-boot setup app (`setup.rs`), as the desktop user.
fn run_setup(args: &Args) -> ExitCode {
    use std::sync::Mutex;

    use kubaos_core::setup::demo;
    use setup::backend::Demo as SetupDemo;
    use setup::backend::Real;
    use setup::backend::SetupBackend;

    let backend: Arc<dyn SetupBackend> = if args.demo {
        let speed = args.demo_speed.unwrap_or(if args.tour.is_some() { 2.5 } else { 1.0 });
        Arc::new(SetupDemo {
            demo: demo::Demo::new(args.demo_machine, speed),
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
            .map(|d| setup::tour::Tour::new(d, args.tour_pace, args.demo_machine)),
    );
    let scale = args.scale;
    let card = args.card;
    let settings = window::Settings {
        size: args.size,
        min_size: Some(Size::new(900.0, 600.0)),
        decorations: !cfg!(target_os = "linux"),
        // Closing the window counts as "done for now" (setup writes its
        // marker first), so it handles the request itself.
        exit_on_close_request: false,
        ..window::Settings::default()
    };
    #[cfg(target_os = "linux")]
    let settings = window::Settings {
        platform_specific: window::settings::PlatformSpecific {
            application_id: "kobo".into(),
            ..window::settings::PlatformSpecific::default()
        },
        ..settings
    };
    let mut app = iced::application(
        move || {
            setup::Setup::new(
                Arc::clone(&backend),
                tour.lock().ok().and_then(|mut t| t.take()),
                card,
                scale.unwrap_or(1.0),
            )
        },
        setup::Setup::update,
        setup::view,
    )
    .title("Kōbō · Setup")
    .subscription(setup::Setup::subscription)
    .theme(|s: &setup::Setup| ui::theme(&s.look()))
    .window(settings)
    .scale_factor(move |_| scale.unwrap_or(1.0))
    .default_font(kubaos_ui::text::default_font())
    .antialiasing(true);
    for f in kubaos_ui::text::FONTS {
        app = app.font(*f);
    }
    match app.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            say(&format!("kobo: {e}"));
            ExitCode::FAILURE
        }
    }
}

/// One line to stderr: the usage, or the error Kōbō exits with.
#[expect(
    clippy::print_stderr,
    reason = "the usage and the error kobo exits with are for whoever started it, on its terminal"
)]
fn say(line: &str) {
    eprintln!("{line}");
}

/// Kōbō's own warnings, on stderr as `kobo: <message>`. iced, winit and wgpu log through the same
/// facade; only Kōbō's records are written, so theirs stay off the installer's terminal.
struct Stderr;

impl log::Log for Stderr {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Warn && metadata.target().starts_with(module_path!())
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            let _ = writeln!(io::stderr(), "kobo: {}", record.args());
        }
    }

    fn flush(&self) {}
}
