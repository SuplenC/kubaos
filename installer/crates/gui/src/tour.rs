//! `--screenshot-tour DIR`: install KubaOS from Welcome to Done with the
//! keyboard only, saving a PNG of each screen on the way, then exit.
//!
//! The script below is nothing but key presses (built as iced keyboard
//! events and translated by `app::translate`, the same function the window
//! uses) plus waits and screenshots. No step sends a click or calls into the
//! app's state, so if the flow ever needs the mouse, the tour gets stuck and
//! the `keyboard_only_install` test fails. Screenshots use iced's own window
//! capture, so this works the same on macOS, X11 and Wayland (cage).

use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use iced::keyboard::{self, Key, Location, Modifiers, key};
use iced::{Task, window};

use crate::app::{App, Message, ProbeState, Screen};

/// Keys the script and the tests press (the tests use more of them).
#[derive(Debug, Clone, Copy)]
#[cfg_attr(not(test), allow(dead_code))]
pub enum K {
    Enter,
    Esc,
    Tab,
    ShiftTab,
    Up,
    Down,
    Left,
    Right,
    ShiftLeft,
    CtrlLeft,
    Home,
    End,
    Space,
    Backspace,
    CtrlBackspace,
    CtrlA,
}

pub enum Step {
    /// Press and release.
    Tap(K),
    /// Press Enter and keep it down (the hold-to-install).
    EnterDown,
    EnterUp,
    Type(&'static str),
    Wait(u64),
    Until(&'static str, fn(&App) -> bool),
    Shot(&'static str),
    Exit,
}

const SETTLE: u64 = 750;

/// Welcome to Done, keys only. The demo backend fails the first install at
/// "Activating your home", so the Failed screen and Retry get a turn.
pub fn script() -> Vec<Step> {
    use K::*;
    use Step::*;
    vec![
        Until("probe", |a| matches!(a.probe, ProbeState::Ready(_))),
        Wait(900),
        Shot("01-welcome"),
        Tap(Enter),
        Until("language", |a| a.screen == Screen::Language),
        Tap(Tab),
        Type("us"),
        Tap(Tab),
        Type("Zażółć gęślą jaźń, 0123"),
        Wait(SETTLE),
        Shot("02-language"),
        Tap(Enter),
        Until("timezone", |a| a.screen == Screen::Timezone),
        Type("rom"),
        Wait(SETTLE),
        Shot("03-timezone"),
        Tap(Enter),
        Until("wifi", |a| a.screen == Screen::Wifi),
        Tap(Down),
        Tap(Tab),
        Type("misato-katsuragi"),
        Wait(SETTLE),
        Shot("04-wifi"),
        Tap(Enter),
        Until("account", |a| a.screen == Screen::Account),
        // The name is already there (from the image), focus is on the
        // password. Shift+Tab back selects the whole name; typing replaces it.
        Tap(ShiftTab),
        Type("Kuba Example"),
        Tap(Tab),
        Type("correct horse battery"),
        Tap(Tab),
        Type("correct horse battery"),
        Wait(SETTLE),
        Shot("05-account"),
        Tap(Enter),
        Until("disk", |a| a.screen == Screen::Disk),
        Wait(SETTLE),
        Shot("06-disk-alongside"),
        Tap(Tab),   // the alongside/erase choice
        Tap(Right), // erase
        Tap(Tab),   // encrypt
        Tap(Tab),   // use my account password
        Tap(Space), // off: own passphrase
        Tap(Tab),
        Type("a much longer boot passphrase"),
        Tap(Tab),
        Type("a much longer boot"),
        Wait(300),
        Shot("07-disk-erase"),
        Tap(ShiftTab),
        Tap(ShiftTab), // back on "use my account password"
        Tap(Space),    // on again
        Tap(Enter),
        Until("git", |a| a.screen == Screen::Git),
        Wait(SETTLE),
        Shot("08-git"),
        Tap(Enter),
        Until("summary", |a| a.screen == Screen::Summary),
        Wait(SETTLE),
        Shot("09-summary"),
        EnterDown,
        Until("holding", |a| {
            a.hold
                .is_some_and(|t| a.now - t >= crate::app::HOLD.mul_f32(0.6))
        }),
        Shot("10-summary-holding"),
        Until("installing", |a| a.screen == Screen::Installing),
        EnterUp,
        Until("42%", |a| {
            a.install.as_ref().is_some_and(|i| i.shown > 0.42)
        }),
        Shot("11-installing"),
        Tap(Enter), // show log
        Wait(450),
        Shot("12-installing-log"),
        Tap(Enter), // hide it
        Until("failed", |a| a.screen == Screen::Failed),
        Wait(SETTLE + 200),
        Shot("13-failed"),
        Tap(Enter), // try again
        Until("done", |a| a.screen == Screen::Done),
        Wait(SETTLE + 200),
        Shot("14-done"),
        Exit,
    ]
}

pub struct Tour {
    dir: PathBuf,
    steps: VecDeque<Step>,
    resume_at: Option<Instant>,
    /// Multiplies every wait; >1 for screen recordings.
    pace: f32,
    pub waiting_for_shot: bool,
}

impl Tour {
    pub fn new(dir: PathBuf, pace: f32) -> Self {
        Self {
            dir,
            steps: script().into(),
            resume_at: None,
            pace,
            waiting_for_shot: false,
        }
    }
}

fn press(key: Key, text: Option<&str>, modifiers: Modifiers) -> keyboard::Event {
    keyboard::Event::KeyPressed {
        modified_key: key.clone(),
        key,
        physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified),
        location: Location::Standard,
        modifiers,
        text: text.map(Into::into),
        repeat: false,
    }
}

fn release(key: Key) -> keyboard::Event {
    keyboard::Event::KeyReleased {
        modified_key: key.clone(),
        key,
        physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified),
        location: Location::Standard,
        modifiers: Modifiers::default(),
    }
}

/// The keyboard events one step sends, in order.
pub fn events(step: &Step) -> Vec<keyboard::Event> {
    use key::Named;
    let named = |n| Key::Named(n);
    match step {
        Step::Tap(k) => {
            let none = Modifiers::default();
            let (key, m, text) = match k {
                K::Enter => (named(Named::Enter), none, Some("\r")),
                K::Esc => (named(Named::Escape), none, None),
                K::Tab => (named(Named::Tab), none, Some("\t")),
                K::ShiftTab => (named(Named::Tab), Modifiers::SHIFT, Some("\t")),
                K::Up => (named(Named::ArrowUp), none, None),
                K::Down => (named(Named::ArrowDown), none, None),
                K::Left => (named(Named::ArrowLeft), none, None),
                K::Right => (named(Named::ArrowRight), none, None),
                K::ShiftLeft => (named(Named::ArrowLeft), Modifiers::SHIFT, None),
                K::CtrlLeft => (named(Named::ArrowLeft), Modifiers::CTRL, None),
                K::Home => (named(Named::Home), none, None),
                K::End => (named(Named::End), none, None),
                K::Space => (named(Named::Space), none, Some(" ")),
                K::Backspace => (named(Named::Backspace), none, None),
                K::CtrlBackspace => (named(Named::Backspace), Modifiers::CTRL, None),
                K::CtrlA => (Key::Character("a".into()), Modifiers::CTRL, Some("a")),
            };
            vec![press(key.clone(), text, m), release(key)]
        }
        Step::EnterDown => vec![press(named(Named::Enter), Some("\r"), Modifiers::default())],
        Step::EnterUp => vec![release(named(Named::Enter))],
        Step::Type(s) => s
            .chars()
            .flat_map(|c| {
                let k = if c == ' ' {
                    named(Named::Space)
                } else {
                    Key::Character(c.to_string().into())
                };
                [
                    press(k.clone(), Some(&c.to_string()), Modifiers::default()),
                    release(k),
                ]
            })
            .collect(),
        _ => vec![],
    }
}

/// Feeds keyboard events through the same translation the window uses.
pub fn send(app: &mut App, evs: Vec<keyboard::Event>) -> Task<Message> {
    let mut tasks = Vec::new();
    for ev in evs {
        if let Some(k) = crate::app::translate(&ev) {
            tasks.push(app.update_inner(Message::Key(k)));
        }
    }
    Task::batch(tasks)
}

/// Runs tour steps until one has to wait. Called after every update.
pub fn drive(app: &mut App) -> Task<Message> {
    let mut tasks = Vec::new();
    while let Some(tour) = &mut app.tour {
        if tour.waiting_for_shot {
            break;
        }
        let now = Instant::now();
        if let Some(t) = tour.resume_at {
            if now < t {
                break;
            }
            tour.resume_at = None;
        }
        let Some(step) = tour.steps.pop_front() else {
            break;
        };
        match step {
            Step::Wait(ms) => {
                tour.resume_at = Some(now + Duration::from_millis(ms).mul_f32(tour.pace))
            }
            Step::Until(name, f) => {
                // The window also has to be up, with its wallpaper, before
                // anything is worth a screenshot.
                let ready = app.window.is_some() && app.wallpaper.is_some() && app.glass.is_some();
                if !(ready && f(app)) {
                    if let Some(t) = &mut app.tour {
                        t.steps.push_front(Step::Until(name, f));
                    }
                    break;
                }
            }
            Step::Shot(name) => {
                let Some(id) = app.window else { break };
                tour.waiting_for_shot = true;
                tasks.push(
                    window::screenshot(id).map(move |s| Message::TourShot(name.to_string(), s)),
                );
                break;
            }
            Step::Exit => {
                tasks.push(iced::exit());
                break;
            }
            keys => {
                let evs = events(&keys);
                tasks.push(send(app, evs));
            }
        }
    }
    Task::batch(tasks)
}

pub fn save(app: &App, name: &str, shot: &window::Screenshot) {
    let Some(tour) = &app.tour else { return };
    let _ = std::fs::create_dir_all(&tour.dir);
    let path = tour.dir.join(format!("{name}.png"));
    match image::save_buffer(
        &path,
        &shot.rgba,
        shot.size.width,
        shot.size.height,
        image::ExtendedColorType::Rgba8,
    ) {
        Ok(()) => eprintln!("tour: {}", path.display()),
        Err(e) => eprintln!("tour: {}: {e}", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::F;
    use crate::backend::Demo;
    use crate::model::DiskMode;
    use dotinstall_core::StepId;
    use std::sync::Arc;

    /// The whole install with synthetic key events only, headless: no
    /// window, no clicks, no messages other than keys and frame ticks.
    #[test]
    fn keyboard_only_install() {
        let mut app = App::headless(Arc::new(Demo::new(40.0, Some(StepId::Home))));
        let mut seen = vec![app.screen];
        let tick = |app: &mut App| {
            let _ = app.update_inner(Message::Tick(Instant::now()));
        };
        for step in script() {
            match step {
                Step::Until(name, f) => {
                    let deadline = Instant::now() + Duration::from_secs(20);
                    while !f(&app) {
                        assert!(
                            Instant::now() < deadline,
                            "stuck waiting for {name} on {:?}",
                            app.screen
                        );
                        tick(&mut app);
                        std::thread::sleep(Duration::from_millis(2));
                    }
                }
                Step::Wait(_) | Step::Shot(_) => tick(&mut app),
                Step::Exit => break,
                keys => {
                    let _ = send(&mut app, events(&keys));
                    tick(&mut app);
                }
            }
            if seen.last() != Some(&app.screen) {
                seen.push(app.screen);
            }
        }
        use Screen::*;
        assert_eq!(
            seen,
            [
                Welcome, Language, Timezone, Wifi, Account, Disk, Git, Summary, Installing, Failed,
                Installing, Done
            ]
        );
        let f = &app.form;
        assert_eq!(f.keymap, "us");
        assert_eq!(f.keyboard_test, "Zażółć gęślą jaźń, 0123");
        assert_eq!(f.timezone, "Europe/Rome");
        assert_eq!(f.wifi_ssid.as_deref(), Some("Nerv HQ"));
        assert_eq!(f.wifi_psk, "misato-katsuragi");
        assert_eq!(f.full_name, "Kuba Example");
        assert_eq!(f.password, f.password_confirm);
        assert_eq!(f.disk_mode, Some(DiskMode::Whole));
        assert!(f.encrypt && f.luks_same_as_password);
        assert_eq!(
            (f.git_name.as_str(), f.git_email.as_str()),
            ("Kuba Example", "kuba@example.com")
        );
    }

    fn ready() -> App {
        let mut app = App::headless(Arc::new(Demo::new(40.0, None)));
        while !matches!(app.probe, ProbeState::Ready(_)) {
            let _ = app.update_inner(Message::Tick(Instant::now()));
        }
        app
    }

    fn keys(app: &mut App, steps: Vec<Step>) {
        for s in steps {
            let _ = send(app, events(&s));
        }
    }

    /// The text field behaves like a text field, from real key events:
    /// select-all on focus, caret movement, word delete, Ctrl+A.
    #[test]
    fn text_field_editing() {
        use K::*;
        use Step::*;
        let mut app = ready();
        let _ = app.go(Screen::Account);
        assert_eq!(
            app.form.full_name, "Kuba Example",
            "prefilled from the image"
        );
        assert_eq!(app.focus, Some(F::Password));
        // Back into the name: all of it selected, so typing replaces it.
        keys(&mut app, vec![Tap(ShiftTab), Type("Kuba")]);
        assert_eq!(app.form.full_name, "Kuba");
        // Caret moves: fix "Kuba" -> "Kuuba" -> "XKuuba!"
        keys(&mut app, vec![Tap(Left), Tap(Left), Tap(Left), Type("u")]);
        assert_eq!(app.form.full_name, "Kuuba");
        keys(&mut app, vec![Tap(Home), Type("X"), Tap(End), Type("!")]);
        assert_eq!(app.form.full_name, "XKuuba!");
        // Shift+Left selects, Backspace deletes the selection.
        keys(
            &mut app,
            vec![Tap(ShiftLeft), Tap(ShiftLeft), Tap(Backspace)],
        );
        assert_eq!(app.form.full_name, "XKuub");
        // Word delete and word jump.
        keys(&mut app, vec![Type(" Example"), Tap(CtrlBackspace)]);
        assert_eq!(app.form.full_name, "XKuub ");
        keys(&mut app, vec![Tap(CtrlLeft), Type("J")]);
        assert_eq!(app.form.full_name, "JXKuub ");
        // Ctrl+A, then typing replaces everything.
        keys(&mut app, vec![Tap(CtrlA), Type("Kuba Example")]);
        assert_eq!(app.form.full_name, "Kuba Example");
        // Space types a space in a field, and Esc still goes back.
        keys(&mut app, vec![Tap(Esc)]);
        assert_eq!(
            app.screen,
            Screen::Wifi,
            "the demo is offline: Wi-Fi comes first"
        );
    }

    /// Focus order matches reading order on every screen.
    #[test]
    fn focus_order_per_screen() {
        let mut app = ready();
        let expect: Vec<(Screen, Vec<F>)> = vec![
            (
                Screen::Language,
                vec![F::LocaleQ, F::KeymapQ, F::KeyboardTest, F::Back, F::Next],
            ),
            (Screen::Timezone, vec![F::TzQ, F::Back, F::Next]),
            (Screen::Wifi, vec![F::WifiList, F::Back, F::Next]),
            (
                Screen::Account,
                vec![
                    F::FullName,
                    F::Password,
                    F::PasswordConfirm,
                    F::ShowSecrets,
                    F::Back,
                    F::Next,
                ],
            ),
            (
                Screen::Disk,
                vec![
                    F::DiskList,
                    F::DiskMode,
                    F::Encrypt,
                    F::LuksSame,
                    F::Back,
                    F::Next,
                ],
            ),
            (Screen::Git, vec![F::GitName, F::GitEmail, F::Back, F::Next]),
        ];
        for (s, order) in expect {
            let _ = app.go(s);
            assert_eq!(app.focus_order(), order, "{s:?}");
        }
        // Own passphrase: its fields slot in after the switch that asks for them.
        let _ = app.go(Screen::Disk);
        keys(
            &mut app,
            vec![Step::Tap(K::Tab), Step::Tap(K::Tab), Step::Tap(K::Tab)],
        );
        assert_eq!(app.focus, Some(F::LuksSame));
        keys(&mut app, vec![Step::Tap(K::Space), Step::Tap(K::Tab)]);
        assert_eq!(app.focus, Some(F::Luks));
        assert_eq!(
            app.focus_order()[4..7],
            [F::Luks, F::LuksConfirm, F::ShowSecrets]
        );
    }

    /// Arrows, Home/End and Page keys move list selections, and typing on a
    /// list screen searches.
    #[test]
    fn list_keys() {
        use K::*;
        use Step::*;
        let mut app = ready();
        let _ = app.go(Screen::Timezone);
        app.focus = Some(F::Next);
        let before = app.form.timezone.clone();
        keys(&mut app, vec![Tap(Down)]);
        assert_eq!(
            app.form.timezone, before,
            "arrows on a button don't move the list"
        );
        keys(&mut app, vec![Type("tok")]);
        assert_eq!(app.focus, Some(F::TzQ));
        assert_eq!(app.form.timezone, "Asia/Tokyo");
        keys(&mut app, vec![Tap(CtrlA), Tap(Backspace)]);
        assert_eq!(app.field_value(F::TzQ), "");
        app.focus = Some(F::TzQ);
        // Up/Down in the search box move the list.
        keys(&mut app, vec![Tap(Down)]);
        assert_ne!(app.form.timezone, "Asia/Tokyo");
        // Wi-Fi: Up from the first network lands on "don't connect".
        let _ = app.go(Screen::Wifi);
        keys(&mut app, vec![Tap(Down), Tap(Up)]);
        assert_eq!(app.form.wifi_ssid, None);
        keys(&mut app, vec![Tap(End)]);
        assert_eq!(app.form.wifi_ssid.as_deref(), Some("DIRECT-roku-417"));
    }

    /// Every control on every screen is in the Tab order, and Tab and
    /// Shift+Tab walk it both ways.
    #[test]
    fn tab_reaches_everything() {
        let mut app = App::headless(Arc::new(Demo::new(40.0, None)));
        while !matches!(app.probe, ProbeState::Ready(_)) {
            let _ = app.update_inner(Message::Tick(Instant::now()));
        }
        let _ = app.go(Screen::Failed);
        let order = app.focus_order();
        for want in [F::Retry, F::SaveLog, F::ToggleLog, F::Back] {
            assert!(order.contains(&want), "{want:?} not reachable on Failed");
        }
        for s in [Screen::Language, Screen::Disk, Screen::Summary] {
            let _ = app.go(s);
            let order = app.focus_order();
            let start = app.focus;
            for _ in 0..order.len() {
                let _ = send(&mut app, events(&Step::Tap(K::Tab)));
            }
            assert_eq!(app.focus, start, "Tab doesn't cycle on {s:?}");
            let _ = send(&mut app, events(&Step::Tap(K::ShiftTab)));
            assert_eq!(
                app.focus,
                order.last().copied(),
                "Shift+Tab doesn't go back on {s:?}"
            );
        }
    }
}
