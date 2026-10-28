//! Kōbō's screens without a window: every stop of the install and setup tours, and the states the
//! tours never reach, laid out by iced at each supported scale, light and dark.
//!
//! `every_screen_keeps_to_the_grid` holds every one of them to the design language's geometry
//! rules (docs/DESIGN.md, "The geometry check"), as `kubaos-specimen check` holds the specimen.
//!
//!   `KOBO_SHOTS=DIR cargo test -p kubaos-kobo --release shots -- --ignored`
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
use kubaos_ui::motion::Motion;
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
use crate::progress::Outcome;
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
/// and, when `shot` names a file, its pixels written there, drawn as a frame at `now`.
fn take(view: Element<'_, Message>, look: &Look, page: &str, shot: Option<&Path>, now: Instant) -> Frame {
    take_scrolled(view, look, page, shot, now, None)
}

/// How a shot scrolls a scrollable before it is taken: a shot runs no tasks.
enum Scrolled {
    /// The item named scrolled into view in its scrollable, as the live app's reveal task does.
    Reveal(&'static str, iced::widget::Id),
    /// The scrollable scrolled down this many logical pixels, its content cut by both its edges.
    By(&'static str, f32),
}

/// [`take`], with a scrollable scrolled first.
fn take_scrolled(
    view: Element<'_, Message>,
    look: &Look,
    page: &str,
    shot: Option<&Path>,
    now: Instant,
    scrolled: Option<Scrolled>,
) -> Frame {
    let mut renderer = headless();
    let factor = look.scale.factor();
    let mut ui = UserInterface::build(view, WINDOW, user_interface::Cache::default(), &mut renderer);
    let offset = match scrolled {
        None => None,
        Some(Scrolled::By(scroll, y)) => Some((scroll, y)),
        Some(Scrolled::Reveal(scroll, item)) => {
            let mut find = Unit(crate::reveal::Find::new(scroll, item));
            ui.operate(&renderer, &mut find);
            match iced::advanced::widget::Operation::<crate::reveal::Found>::finish(&find.0) {
                iced::advanced::widget::operation::Outcome::Some(found) => {
                    crate::reveal::target(found).map(|y| (scroll, y))
                }
                _ => None,
            }
        }
    };
    if let Some((scroll, y)) = offset {
        let mut to = iced::advanced::widget::operation::scrollable::scroll_to(
            iced::widget::Id::new(scroll),
            iced::widget::operation::AbsoluteOffset { x: None, y: Some(y) },
        );
        ui.operate(&renderer, &mut to);
    }
    let mut collect = Collect::new();
    ui.operate(&renderer, &mut collect);
    if let Some(path) = shot {
        let mut messages = Vec::new();
        let _ = ui.update(
            &[Event::Window(window::Event::RedrawRequested(now))],
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
        say(&format!("shots: {}", path.display()));
    }
    Frame {
        page: page.to_owned(),
        mode: mode_name(look.mode).to_owned(),
        scale: factor,
        root: collect.into_tree().expect("the screen reports its boxes"),
    }
}

/// The reveal's search, run where only operations that report nothing are taken; what it found is
/// read from it afterwards.
struct Unit(crate::reveal::Find);

impl iced::advanced::widget::Operation for Unit {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn iced::advanced::widget::Operation)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&iced::widget::Id>, bounds: iced::Rectangle) {
        iced::advanced::widget::Operation::<crate::reveal::Found>::container(&mut self.0, id, bounds);
    }

    fn scrollable(
        &mut self,
        id: Option<&iced::widget::Id>,
        bounds: iced::Rectangle,
        content_bounds: iced::Rectangle,
        translation: iced::Vector,
        state: &mut dyn iced::advanced::widget::operation::Scrollable,
    ) {
        iced::advanced::widget::Operation::<crate::reveal::Found>::scrollable(
            &mut self.0,
            id,
            bounds,
            content_bounds,
            translation,
            state,
        );
    }
}

/// Every running animation finished, and the clock where the shots read it.
fn settle_app(app: &mut App) {
    let now = Instant::now();
    let held = app.hold.map(|at| app.now.saturating_duration_since(at));
    app.now = now;
    app.transition.0 = ago(now, Duration::from_secs(5));
    app.log_open.1 = ago(now, Duration::from_secs(5));
    app.switched = None;
    app.nudged = None;
    app.hold = held.map(|held| ago(now, held));
    if let Some((_, at)) = &mut app.wallpaper {
        *at = ago(now, Duration::from_secs(5));
    }
}

/// A line for whoever ran the shots from a terminal: where a shot went, or how to ask for them.
#[expect(clippy::print_stderr, reason = "The shots are run by hand, from a terminal.")]
fn say(line: &str) {
    eprintln!("{line}");
}

/// `by` before `now`.
fn ago(now: Instant, by: Duration) -> Instant {
    now.checked_sub(by)
        .expect("the monotonic clock has run longer than the shots look back")
}

fn settle_setup(s: &mut Setup) {
    let now = Instant::now();
    s.now = now;
    s.changed = ago(now, Duration::from_secs(5));
}

fn take_app(out: &mut Out, name: &str, app: &mut App) {
    settle_app(app);
    take_every_look(out, name, app);
}

/// The screen `since` it arrived, everything else settled: a moment of its own sequence.
fn take_app_at(out: &mut Out, name: &str, app: &mut App, since: Duration) {
    settle_app(app);
    app.transition.0 = ago(app.now, since);
    take_every_look(out, name, app);
}

/// The screen with the scrollable `scroll` scrolled down `by` logical pixels, everything else
/// settled.
fn take_app_scrolled(out: &mut Out, name: &str, app: &mut App, scroll: &'static str, by: f32) {
    settle_app(app);
    take_every_look_scrolled(out, name, app, Some((scroll, by)));
}

fn take_every_look(out: &mut Out, name: &str, app: &mut App) {
    take_every_look_scrolled(out, name, app, None);
}

fn take_every_look_scrolled(out: &mut Out, name: &str, app: &mut App, scrolled: Option<(&'static str, f32)>) {
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
            let scrolled = scrolled.map(|(scroll, by)| Scrolled::By(scroll, by));
            let frame = take_scrolled(crate::view::view(app), &look, &page, shot.as_deref(), app.now, scrolled);
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
            let reveal = Scrolled::Reveal(crate::setup::CARDS_SCROLL, crate::setup::card_item(s.sel));
            let frame = take_scrolled(
                crate::setup::view(s),
                &look,
                &page,
                shot.as_deref(),
                s.now,
                Some(reveal),
            );
            out.frames.push(frame);
        }
    }
    s.mode = Mode::Dark;
    s.system_scale = 1.0;
}

fn wait_until<T>(state: &mut T, what: &str, done: impl Fn(&T) -> bool, tick: impl Fn(&mut T)) {
    let deadline = Instant::now() + Duration::from_mins(2);
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
    let mut install = Install::new(self::ago(now, ago));
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

    // Scrolled partway, so a scroll's edges cut through rows and pictures: nothing a control
    // draws, its fill and glow included, shows past the edge or over what is beside it.
    let mut scrolled = app_with(Ok(demo_probe()));
    let _ = scrolled.go(Screen::Timezone);
    let first = scrolled.list_items(List::Timezone).into_iter().next();
    if let Some(first) = first {
        let _ = scrolled.update_inner(Message::Pick(List::Timezone, first));
    }
    take_app_scrolled(out, "timezone-scrolled", &mut scrolled, crate::app::ids::TZ_LIST, 30.0);
    let _ = scrolled.go(Screen::Wallpapers);
    scrolled.focus = Some(F::Wallpaper(0, 0));
    take_app_scrolled(
        out,
        "wallpapers-scrolled",
        &mut scrolled,
        crate::app::ids::WALLPAPER_SCROLL,
        60.0,
    );

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
    let shown = ago(Instant::now(), kubaos_ui::component::toast::DEFAULT_STAY / 2);
    app.toast = Some(("Log saved to /root/kubaos-install.log.".into(), shown));
    take_app(out, "failed-log-saved", &mut app);
}

/// The done screen as an install of `took` leaves it, just arrived.
fn done_app(took: Duration) -> App {
    let mut app = app_with(Ok(demo_probe()));
    let mut install = Install::new(Instant::now());
    install.outcome = Some(Outcome::Done { elapsed: took });
    app.install = Some(install);
    app.form.username = "kuba".into();
    let _ = app.go(Screen::Done);
    app
}

/// The done screen through its sequence: the strokes writing in, the headline arriving, the hanko
/// coming down, just after impact; then with reduce motion, everything in place from the start.
/// The tour's stop is the settled screen.
fn done_states(out: &mut Out) {
    let mut app = done_app(Duration::from_secs(108));
    for (name, ms) in [("0.5s", 500), ("1.5s", 1500), ("2.7s", 2700), ("3.2s", 3200)] {
        take_app_at(out, &format!("done-{name}"), &mut app, Duration::from_millis(ms));
    }
    app.motion = Motion::default().reduced();
    take_app_at(out, "done-reduced-motion", &mut app, Duration::ZERO);
}

/// Every stop and state, taken into `out`.
fn every_screen(out: &mut Out) {
    load_fonts();
    install_tour(out);
    install_states(out);
    done_states(out);
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
        say("set KOBO_SHOTS=DIR");
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

/// The done screen's sequence frame by frame, for a recording: `done-NNN.png` in the directory,
/// at `KOBO_DONE_FPS` frames a second (30 by default) for four seconds, at 1×, dark.
#[test]
#[ignore = "writes frames: KOBO_DONE_FRAMES=DIR"]
fn done_frames() {
    let Some(dir) = std::env::var_os("KOBO_DONE_FRAMES") else {
        say("set KOBO_DONE_FRAMES=DIR");
        return;
    };
    let dir = PathBuf::from(dir);
    std::fs::create_dir_all(&dir).expect("create the frames directory");
    let fps: u32 = std::env::var("KOBO_DONE_FPS")
        .ok()
        .and_then(|f| f.parse().ok())
        .unwrap_or(30);
    load_fonts();
    let mut app = done_app(Duration::from_secs(108));
    settle_app(&mut app);
    let arrived = ago(app.now, Duration::from_secs(5));
    for frame in 0..4 * fps {
        app.transition.0 = arrived;
        // design-allow(duration): a second divided into the recording's frames, not motion.
        app.now = arrived + Duration::from_secs(1) * frame / fps;
        let look = app.look();
        let path = dir.join(format!("done-{frame:03}.png"));
        take(crate::view::view(&app), &look, "done", Some(&path), app.now);
    }
}

/// Every box in `node`'s tree, `node` first.
fn boxes(node: &kubaos_geometry::Node) -> Vec<&kubaos_geometry::Node> {
    let mut all = vec![node];
    for child in &node.children {
        all.extend(boxes(child));
    }
    all
}

/// The Wallpapers page's note on how many are picked sits in the footer, just left of Continue
/// and level with it, below everything the page's scroll reaches: nothing loose under its cut.
#[test]
fn wallpapers_note_sits_in_the_footer_beside_continue() {
    use kubaos_geometry::Role;
    load_fonts();
    let mut app = app_with(Ok(demo_probe()));
    let _ = app.go(Screen::Wallpapers);
    settle_app(&mut app);
    let look = app.look();
    let frame = take(crate::view::view(&app), &look, "install-wallpapers", None, app.now);
    let all = boxes(&frame.root);
    let text = |says: &dyn Fn(&str) -> bool| {
        all.iter()
            .find(|n| matches!(&n.role, Role::Text { content, .. } if says(content)))
            .map(|n| n.bounds)
    };
    let note = text(&|t| t.ends_with(" picked.")).expect("the note on how many are picked");
    let next = text(&|t| t == "Continue").expect("Continue");
    let (scroll, reach) = all
        .iter()
        .find_map(|n| match &n.role {
            Role::Scroll { reach: Some(reach) } => Some((n.bounds, *reach)),
            _ => None,
        })
        .expect("the page's scroll, with its reach");

    assert!(
        note.y >= scroll.y + scroll.height + reach.below,
        "the note at {note:?} is under the scroll at {scroll:?}, which reaches {reach:?}"
    );
    let centre = |r: kubaos_geometry::Rect| r.y + r.height / 2.0;
    assert!(
        (centre(note) - centre(next)).abs() <= 1.0,
        "the note at {note:?} is not level with Continue at {next:?}"
    );
    assert!(
        note.x + note.width < next.x,
        "the note at {note:?} is not left of Continue at {next:?}"
    );
    assert!(
        next.x - (note.x + note.width) <= f32::from(Space::S40.px()),
        "the note at {note:?} is not beside Continue at {next:?}"
    );
}
