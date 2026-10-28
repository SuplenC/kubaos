//! Kōbō's screens without a window: every stop of the install and setup tours, and the states the
//! tours never reach, laid out by iced at each supported scale, light and dark.
//!
//! `every_screen_keeps_to_the_grid` holds every one of them to the design language's geometry
//! rules (docs/DESIGN.md, "The geometry check"), as `kubaos-specimen check` holds the specimen.
//!
//!   KOBO_SHOTS=DIR cargo test -p kubaos-kobo --release shots -- --ignored
//!
//! draws them too, with iced's tiny-skia renderer, as `SCREEN-MODE-SCALEx.png` in DIR.
//! `GALLERY_PAYLOAD` names a payload directory with the wallpaper thumbnails
//! (`wallpapers/<set>/<file>.jpg`); without it, the cards show their placeholder.

use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use iced::Element;
use iced::Event;
use iced::Pixels;
use iced::Renderer;
use iced::Size;
use iced::Theme;
use iced::advanced::clipboard;
use iced::advanced::graphics::text::font_system;
use iced::advanced::renderer;
use iced::advanced::renderer::Headless;
use iced::mouse;
use iced::window;
use iced_runtime::UserInterface;
use iced_runtime::user_interface;
use kubaos_core::Event as EngineEvent;
use kubaos_core::Probe;
use kubaos_core::StepId;
use kubaos_core::probe::DiskInfo;
use kubaos_core::probe::PartitionInfo;
use kubaos_core::probe::Region;
use kubaos_core::probe::WifiNetwork;
use kubaos_core::setup::demo;
use kubaos_geometry::Frame;
use kubaos_geometry::Rules;
use kubaos_ui::Look;
use kubaos_ui::Mode;
use kubaos_ui::Scale;
use kubaos_ui::probe::Collect;
use kubaos_ui::tokens;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use crate::app::App;
use crate::app::F;
use crate::app::List;
use crate::app::Message;
use crate::app::Screen;
use crate::backend::DEMO_BATTERY;
use crate::backend::Demo;
use crate::backend::demo_probe;
use crate::battery::Battery;
use crate::battery::Supply;
use crate::progress::Install;
use crate::setup::Setup;

/// The window every screen is laid out in, in logical pixels: Kōbō's opening size.
// design-allow(size): the window the screens are drawn in, not layout.
const WINDOW: Size = Size::new(1280.0, 800.0);

const MODES: [Mode; 2] = [Mode::Dark, Mode::Light];

/// What each stop is turned into.
struct Out {
    /// Where the screenshots go; none, for the geometry check alone.
    shots: Option<PathBuf>,
    frames: Vec<Frame>,
}

fn load_fonts() {
    let mut system = font_system().write().expect("font system lock poisoned");
    for font in kubaos_ui::text::FONTS {
        system.load_font(Cow::Borrowed(font));
    }
}

fn headless() -> Renderer {
    iced::futures::executor::block_on(<Renderer as Headless>::new(
        kubaos_ui::text::default_font(),
        Pixels(f32::from(Type::Body.size())),
        Some("tiny-skia"),
    ))
    .expect("the tiny-skia renderer")
}

fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::Dark => "dark",
        Mode::Light => "light",
    }
}

/// `view` laid out in [`WINDOW`] at `look`'s scale: its boxes, as the geometry check reads them,
/// and, when `shot` names a file, its pixels written there.
fn take(view: Element<'_, Message>, look: &Look, page: &str, shot: Option<&Path>) -> Frame {
    let mut renderer = headless();
    let factor = look.scale.factor();
    let mut ui = UserInterface::build(view, WINDOW, user_interface::Cache::default(), &mut renderer);
    let mut collect = Collect::new();
    ui.operate(&renderer, &mut collect);
    if let Some(path) = shot {
        let mut messages = Vec::new();
        let _ = ui.update(
            &[Event::Window(window::Event::RedrawRequested(Instant::now()))],
            mouse::Cursor::Unavailable,
            &mut renderer,
            &mut clipboard::Null,
            &mut messages,
        );
        ui.draw(
            &mut renderer,
            &Theme::Dark,
            &renderer::Style {
                text_color: look.colors.on_surface,
            },
            mouse::Cursor::Unavailable,
        );
        let physical = Size::new(
            (WINDOW.width * factor).round() as u32,
            (WINDOW.height * factor).round() as u32,
        );
        let rgba = renderer.screenshot(physical, factor, look.colors.surface);
        image::save_buffer(
            path,
            &rgba,
            physical.width,
            physical.height,
            image::ExtendedColorType::Rgba8,
        )
        .expect("write the screenshot");
        eprintln!("shots: {}", path.display());
    }
    Frame {
        page: page.to_owned(),
        mode: mode_name(look.mode).to_owned(),
        scale: factor,
        root: collect.into_tree().expect("the screen reports its boxes"),
    }
}

/// Every running animation finished, and the clock where the shots read it.
fn settle_app(app: &mut App) {
    let now = Instant::now();
    let held = app.hold.map(|at| app.now - at);
    app.now = now;
    app.transition.0 = now - Duration::from_secs(5);
    app.log_open.1 = now - Duration::from_secs(5);
    app.switched = None;
    app.nudged = None;
    app.hold = held.map(|held| now - held);
    if let Some((_, at)) = &mut app.wallpaper {
        *at = now - Duration::from_secs(5);
    }
}

fn settle_setup(s: &mut Setup) {
    let now = Instant::now();
    s.now = now;
    s.changed = now - Duration::from_secs(5);
}

fn take_app(out: &mut Out, name: &str, app: &mut App) {
    settle_app(app);
    let page = format!("install-{name}");
    for mode in MODES {
        for scale in Scale::SUPPORTED {
            app.mode = mode;
            app.system_scale = scale.factor();
            let look = app.look();
            let shot = out
                .shots
                .as_ref()
                .map(|dir| dir.join(format!("{page}-{}-{}x.png", mode_name(mode), scale.factor())));
            let frame = take(crate::view::view(app), &look, &page, shot.as_deref());
            out.frames.push(frame);
        }
    }
    app.mode = Mode::Dark;
    app.system_scale = 1.0;
}

fn take_setup(out: &mut Out, name: &str, s: &mut Setup) {
    settle_setup(s);
    let page = format!("setup-{name}");
    for mode in MODES {
        for scale in Scale::SUPPORTED {
            s.mode = mode;
            s.system_scale = scale.factor();
            let look = s.look();
            let shot = out
                .shots
                .as_ref()
                .map(|dir| dir.join(format!("{page}-{}-{}x.png", mode_name(mode), scale.factor())));
            let frame = take(crate::setup::view(s), &look, &page, shot.as_deref());
            out.frames.push(frame);
        }
    }
    s.mode = Mode::Dark;
    s.system_scale = 1.0;
}

fn wait_until<T>(state: &mut T, what: &str, done: impl Fn(&T) -> bool, tick: impl Fn(&mut T)) {
    let deadline = Instant::now() + Duration::from_secs(120);
    while !done(state) {
        assert!(Instant::now() < deadline, "stuck waiting for {what}");
        tick(state);
        // design-allow(duration): how often a tour checks what it waits for, not motion.
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn tick_app(app: &mut App) {
    let _ = app.update_inner(Message::Tick(Instant::now()));
}

fn tick_setup(s: &mut Setup) {
    let _ = s.update_inner(Message::Tick(Instant::now()));
}

/// The install tour, taken at each of its stops. The demo runs slowly enough that taking a stop at
/// every scale never outlasts the step it shows, even with the other tests running beside it.
fn install_tour(out: &mut Out) {
    use crate::tour::Step;
    let mut app = App::headless(Arc::new(Demo::new(4.0, Some(StepId::Home))));
    app.battery = Some(DEMO_BATTERY);
    app.load_images_now(WINDOW);
    load_payload(&mut app);
    for step in crate::tour::script() {
        match step {
            Step::Until(name, done) => wait_until(&mut app, name, done, tick_app),
            Step::Wait(_) => tick_app(&mut app),
            Step::Shot(name) => {
                tick_app(&mut app);
                // The probe names the wallpaper sets once it answers, partway into the tour.
                if app.thumbs.is_empty() {
                    load_payload(&mut app);
                }
                take_app(out, name, &mut app);
            }
            Step::Exit => break,
            keys => {
                let _ = crate::tour::send(&mut app, crate::tour::events(&keys));
                tick_app(&mut app);
            }
        }
    }
}

/// A setup tour on `machine`, taken at each of its stops.
fn setup_tour(out: &mut Out, machine: demo::Machine, script: Vec<crate::setup::tour::Step>) {
    use crate::setup::tour::Step;
    let mut s = Setup::headless(Arc::new(crate::setup::backend::Demo {
        demo: demo::Demo::new(machine, 200.0),
    }));
    for step in script {
        match step {
            Step::Until(name, done) => wait_until(&mut s, name, done, tick_setup),
            Step::Wait(_) => tick_setup(&mut s),
            Step::Shot(name) => {
                tick_setup(&mut s);
                take_setup(out, name, &mut s);
            }
            Step::Exit => break,
            keys => {
                let _ = crate::setup::tour::send(&mut s, crate::setup::tour::events(&keys));
                tick_setup(&mut s);
            }
        }
    }
}

fn load_payload(app: &mut App) {
    if let Some(payload) = std::env::var_os("GALLERY_PAYLOAD") {
        app.payload_dir = PathBuf::from(payload);
        app.load_thumbs_now();
    }
}

fn app_with(probe: Result<Probe, String>) -> App {
    let mut app = App::headless(Arc::new(Demo::new(40.0, None)));
    app.set_probe(probe);
    // The battery watch is a subscription, which a headless app never runs.
    app.battery = Some(DEMO_BATTERY);
    app.load_images_now(WINDOW);
    load_payload(&mut app);
    app
}

const GB: u64 = 1_000_000_000;

/// The demo machine, but harder: a long SSID, more networks, a disk with a long name and one too
/// small.
fn hard_probe() -> Probe {
    let mut p = demo_probe();
    p.wifi.insert(
        1,
        WifiNetwork {
            ssid: "FRITZ!Box 7590 XL Gästenetz OG".into(),
            signal: 80,
            secured: true,
        },
    );
    for i in 0..6 {
        p.wifi.push(WifiNetwork {
            ssid: format!("Neighbour-{i}"),
            signal: 10 + i * 5,
            secured: i % 2 == 0,
        });
    }
    p.disks.push(DiskInfo {
        path: "/dev/nvme1n1".into(),
        model: "WD_BLACK SN850X 4000GB with Heatsink, Gaming Edition, Firmware 620361WD".into(),
        size_bytes: 4000 * GB,
        removable: false,
        is_boot_medium: false,
        partitions: vec![PartitionInfo {
            path: "/dev/nvme1n1p1".into(),
            size_bytes: 4000 * GB - 2_000_000,
            fstype: Some("btrfs".into()),
            label: Some("a-very-long-filesystem-label-that-goes-on".into()),
            os_hint: Some("Linux (some other distro, unfortunately)".into()),
        }],
        free_regions: vec![Region {
            start: 0,
            end: 1_000_000,
        }],
    });
    p.disks.push(DiskInfo {
        path: "/dev/mmcblk0".into(),
        model: String::new(),
        size_bytes: 8 * GB,
        removable: true,
        is_boot_medium: false,
        partitions: vec![],
        free_regions: vec![],
    });
    p
}

/// An install that has planned its steps and started `at`, `ago` back.
fn install_at(at: StepId, ago: Duration) -> Install {
    let now = Instant::now();
    let mut install = Install::new(now - ago);
    install.apply(
        &EngineEvent::Plan {
            steps: StepId::ALL.iter().map(|s| s.info()).collect(),
        },
        now,
    );
    install.apply(&EngineEvent::StepStarted { id: at }, now);
    install
}

/// The states the install tour never reaches.
fn install_states(out: &mut Out) {
    let mut p = demo_probe();
    p.payload = None;
    take_app(out, "welcome-no-image", &mut app_with(Ok(p)));
    let mut app = app_with(Err("probe failed: lsblk: command not found".into()));
    take_app(out, "welcome-probe-failed", &mut app);

    let mut two = app_with(Ok(demo_probe()));
    let _ = two.go(Screen::Language);
    let _ = two.update_inner(Message::Pick(List::Keymap, "fr".into()));
    take_app(out, "language-two-layouts", &mut two);

    let mut app = app_with(Ok(hard_probe()));
    let _ = app.go(Screen::Wifi);
    let _ = app.update_inner(Message::Pick(List::Wifi, "FRITZ!Box 7590 XL Gästenetz OG".into()));
    take_app(out, "wifi-long", &mut app);

    app.form.full_name = "Kuba Example".into();
    app.form.password = "hunter2".into();
    app.form.password_confirm = "hunter3".into();
    app.form.hostname = "Nerv.HQ".into();
    let _ = app.go(Screen::Account);
    app.focus = Some(F::PasswordConfirm);
    take_app(out, "account-mismatch", &mut app);
    app.form.password_confirm = "hunter2".into();
    app.form.hostname = "tuxedo".into();

    let _ = app.go(Screen::Disk);
    let _ = app.update_inner(Message::Pick(List::Disk, "/dev/mmcblk0".into()));
    app.form.encrypt = false;
    take_app(out, "disk-too-small", &mut app);
    app.form.encrypt = true;
    let _ = app.update_inner(Message::Pick(List::Disk, "/dev/nvme0n1".into()));

    app.form.git_email = "not-an-email".into();
    let _ = app.go(Screen::Git);
    app.focus = Some(F::GitEmail);
    let _ = app.update_inner(Message::Key(crate::app::Key::Enter));
    take_app(out, "git-bad-email", &mut app);
    app.form.git_email = "kuba@example.com".into();

    let _ = app.go(Screen::Wallpapers);
    let picked = app.form.wallpapers.clone();
    app.form.wallpapers.clear();
    app.focus = Some(F::WallpaperSet(0));
    take_app(out, "wallpapers-none", &mut app);
    app.form.wallpapers = picked;

    let _ = app.go(Screen::Summary);
    app.battery = Some(Battery {
        percent: 14,
        supply: Supply::Discharging,
    });
    take_app(out, "summary-low-battery", &mut app);
    app.battery = Some(DEMO_BATTERY);
    let _ = app.update_inner(Message::Click(F::Power));
    take_app(out, "power-menu", &mut app);
    let _ = app.update_inner(Message::Click(F::PowerCancel));

    app.install = Some(install_at(StepId::Unpack, Duration::from_secs(90)));
    let _ = app.go(Screen::Installing);
    let _ = app.update_inner(Message::Click(F::Power));
    take_app(out, "power-menu-installing", &mut app);
    let _ = app.update_inner(Message::Click(F::PowerCancel));

    let mut install = install_at(StepId::Bootloader, Duration::from_secs(40));
    install.apply(
        &EngineEvent::Failed {
            id: StepId::Bootloader,
            error: "grub-install --target=x86_64-efi --efi-directory=/boot/efi --bootloader-id=ubuntu failed with \
                    exit status 1: Installing for x86_64-efi platform. grub-install: error: efibootmgr failed to \
                    register the boot entry: Operation not permitted."
                .into(),
            hint: Some(
                "The firmware refused a new boot entry. Turn off Secure Boot's \"lock boot order\" in the setup \
                 menu and try again."
                    .into(),
            ),
        },
        Instant::now(),
    );
    app.install = Some(install);
    app.log_open = (true, Instant::now());
    let _ = app.go(Screen::Failed);
    take_app(out, "failed-long", &mut app);
    // Halfway through its stay: arrived, not yet leaving.
    let shown = Instant::now() - kubaos_ui::component::toast::DEFAULT_STAY / 2;
    app.toast = Some(("Log saved to /root/kubaos-install.log.".into(), shown));
    take_app(out, "failed-log-saved", &mut app);
}

/// Every stop and state, taken into `out`.
fn every_screen(out: &mut Out) {
    load_fonts();
    install_tour(out);
    install_states(out);
    setup_tour(out, demo::Machine::Tuxedo, crate::setup::tour::script());
    setup_tour(out, demo::Machine::Vm, crate::setup::tour::script_vm());
    setup_tour(out, demo::Machine::Installed, crate::setup::tour::script_bitwarden());
}

/// The rules of the tokens: the grid and the spacing scale.
fn rules() -> Rules {
    Rules {
        grid: f32::from(tokens::GRID),
        spacing: Space::ALL.iter().map(|space| f32::from(space.px())).collect(),
        container: tokens::CONTAINER
            .iter()
            .map(|&(from, padding)| (f32::from(from), f32::from(padding.px())))
            .collect(),
    }
}

#[test]
fn every_screen_keeps_to_the_grid() {
    let mut out = Out {
        shots: None,
        frames: Vec::new(),
    };
    every_screen(&mut out);

    let findings = kubaos_geometry::check(&out.frames, &rules());

    let shown: Vec<String> = findings.iter().take(4000).map(ToString::to_string).collect();
    assert!(
        findings.is_empty(),
        "{} frame(s), {} finding(s):\n{}",
        out.frames.len(),
        findings.len(),
        shown.join("\n")
    );
}

#[test]
#[ignore = "writes screenshots: KOBO_SHOTS=DIR"]
fn shots() {
    let Some(dir) = std::env::var_os("KOBO_SHOTS") else {
        eprintln!("set KOBO_SHOTS=DIR");
        return;
    };
    let dir = PathBuf::from(dir);
    std::fs::create_dir_all(&dir).expect("create the shots directory");
    let mut out = Out {
        shots: Some(dir),
        frames: Vec::new(),
    };
    every_screen(&mut out);
}
