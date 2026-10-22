use super::*;
use crate::battery::Supply;

#[test]
fn scale_fits_the_output() {
    let at = |w, h| auto_scale(Size::new(w, h));
    assert_eq!(at(1024.0, 768.0), 1.0);
    assert_eq!(at(1280.0, 800.0), 1.0);
    assert_eq!(at(1440.0, 900.0), 1.0);
    assert_eq!(at(1920.0, 1080.0), 1.25);
    assert_eq!(at(1920.0, 1200.0), 1.5);
    assert_eq!(at(2880.0, 1800.0), 2.25);
    assert_eq!(at(3840.0, 2160.0), 2.5);
}

fn app_with_every_keymap() -> App {
    let mut p = crate::backend::demo_probe();
    p.keymaps = include_str!("../../assets/keymaps.tsv")
        .lines()
        .filter_map(|l| l.split('\t').next())
        .map(str::to_string)
        .collect();
    let mut app = App::headless(std::sync::Arc::new(crate::backend::Demo::new(40.0, None)));
    app.set_probe(Ok(p));
    app
}

fn shows_row(y: f32, row: usize) {
    let pitch = crate::widgets::ROW_HEIGHT + crate::widgets::ROW_GAP;
    let top = row as f32 * pitch;
    assert!(y <= top, "row {row} at {top} starts above the offset {y}");
    assert!(
        top + crate::widgets::ROW_HEIGHT <= y + 4.0 * pitch,
        "row {row} at {top} is below offset {y}"
    );
}

#[test]
fn the_language_screen_opens_with_the_preselected_layout_in_view() {
    let mut app = app_with_every_keymap();
    let row = app
        .list_items(List::Keymap)
        .iter()
        .position(|k| *k == app.form.keymaps[0])
        .expect("the preselected layout is listed");
    assert!(
        row > 10,
        "the preselected layout is near the top ({row}), so the test proves nothing"
    );

    let _ = app.go(Screen::Language);

    let scrolls = app.selection_scrolls();
    let &(_, y) = scrolls
        .iter()
        .find(|(id, _)| *id == ids::KEYMAP_LIST)
        .expect("the keymap list scrolls to its selection");
    shows_row(y, row);
}

/// A backend that records what the app asks of it. Its install runs until it's stopped, unless
/// it's `stubborn`, and then until the machine is told to go down.
#[derive(Default)]
struct Recorder {
    calls: std::sync::Mutex<Vec<&'static str>>,
    stubborn: bool,
}

impl Recorder {
    fn calls(&self) -> Vec<&'static str> {
        self.calls.lock().expect("calls mutex poisoned").clone()
    }

    fn record(&self, call: &'static str) {
        self.calls.lock().expect("calls mutex poisoned").push(call);
    }

    fn answered(&self, call: &str) -> bool {
        self.calls().contains(&call)
    }
}

impl Backend for Recorder {
    fn probe(&self) -> Result<Probe, String> {
        Ok(crate::backend::demo_probe())
    }

    fn run(
        &self,
        _config: &kubaos_core::InstallConfig,
        _secrets: &kubaos_core::Secrets,
        emit: &mut dyn FnMut(RunMsg),
    ) {
        use kubaos_core::{Event, StepId};
        emit(RunMsg::Event(Event::Plan {
            steps: StepId::ALL.iter().map(|s| s.info()).collect(),
        }));
        emit(RunMsg::Event(Event::StepStarted { id: StepId::Unpack }));
        let deadline = Instant::now() + Duration::from_secs(10);
        let done = |r: &Self| {
            if r.stubborn {
                r.answered("poweroff") || r.answered("reboot")
            } else {
                r.answered("stop")
            }
        };
        while !done(self) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(2));
        }
        emit(RunMsg::Event(Event::Failed {
            id: StepId::Unpack,
            error: "interrupted".into(),
            hint: None,
        }));
        emit(RunMsg::Exited(Ok(())));
    }

    fn stop(&self) {
        self.record("stop");
    }

    fn reboot(&self) -> Result<(), String> {
        self.record("reboot");
        Ok(())
    }

    fn poweroff(&self) -> Result<(), String> {
        self.record("poweroff");
        Ok(())
    }

    fn battery(&self) -> Option<Battery> {
        None
    }

    fn is_demo(&self) -> bool {
        false
    }
}

fn recorded(recorder: Recorder) -> (App, Arc<Recorder>) {
    let recorder = Arc::new(recorder);
    let backend: Arc<dyn Backend> = Arc::clone(&recorder) as Arc<dyn Backend>;
    let mut app = App::headless(backend);
    app.set_probe(Ok(crate::backend::demo_probe()));
    (app, recorder)
}

fn press(app: &mut App, keys: &[Key]) {
    for k in keys {
        let _ = app.update_inner(Message::Key(k.clone()));
    }
}

/// Ticks the app until `done`, failing after a few seconds.
fn tick_until(app: &mut App, what: &str, done: impl Fn(&App) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !done(app) {
        assert!(Instant::now() < deadline, "stuck waiting for {what}");
        let _ = app.update_inner(Message::Tick(Instant::now()));
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn wait_for(recorder: &Recorder, call: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !recorder.answered(call) {
        assert!(Instant::now() < deadline, "the backend never got {call}");
        std::thread::sleep(Duration::from_millis(2));
    }
}

const EVERY_SCREEN: [Screen; 12] = [
    Screen::Welcome,
    Screen::Language,
    Screen::Timezone,
    Screen::Wifi,
    Screen::Account,
    Screen::Disk,
    Screen::Git,
    Screen::Wallpapers,
    Screen::Summary,
    Screen::Installing,
    Screen::Failed,
    Screen::Done,
];

fn open_menu_with_keys(app: &mut App) {
    let order = app.focus_order();
    let start = order
        .iter()
        .position(|f| Some(*f) == app.focus)
        .unwrap_or(0);
    let tabs = order.len() - 1 - start;
    press(app, &vec![Key::Tab { shift: false }; tabs]);
    assert_eq!(
        app.focus,
        Some(F::Power),
        "Tab ends on the power button on {:?}",
        app.screen
    );
    press(app, &[Key::Enter]);
}

#[test]
fn every_screen_opens_the_power_menu_from_the_keyboard() {
    for screen in EVERY_SCREEN {
        let (mut app, recorder) = recorded(Recorder::default());
        let _ = app.go(screen);

        open_menu_with_keys(&mut app);

        assert!(app.power.is_some(), "no power menu on {screen:?}");
        assert_eq!(app.focus, Some(F::PowerCancel), "{screen:?}");
        assert_eq!(app.screen, screen, "opening the menu left {screen:?}");
        assert_eq!(
            app.hold, None,
            "Enter on the power button started the hold on {screen:?}"
        );
        assert!(recorder.calls().is_empty(), "{screen:?}");
    }
}

#[test]
fn shift_tab_from_a_screen_s_first_control_reaches_the_power_button() {
    let (mut app, _) = recorded(Recorder::default());
    let _ = app.go(Screen::Language);
    assert_eq!(app.focus, Some(F::LocaleQ));

    press(&mut app, &[Key::Tab { shift: true }]);

    assert_eq!(app.focus, Some(F::Power));
}

#[test]
fn every_screen_draws_a_clickable_power_button() {
    use iced_test::simulator::Simulator;
    for screen in EVERY_SCREEN {
        let (mut app, _) = recorded(Recorder::default());
        let _ = app.go(screen);
        let mut ui = Simulator::with_size(
            iced::Settings::default(),
            Size::new(1280.0, 800.0),
            crate::view::view(&app),
        );

        ui.click("Power")
            .unwrap_or_else(|e| panic!("no power button on {screen:?}: {e}"));

        let clicked = ui
            .into_messages()
            .any(|m| matches!(m, Message::Click(F::Power)));
        assert!(
            clicked,
            "the power button on {screen:?} doesn't open the menu"
        );
    }
}

#[test]
fn the_open_menu_draws_its_choices_and_keeps_clicks_from_the_screen_behind() {
    use iced_test::simulator::Simulator;
    let (mut app, _) = recorded(Recorder::default());
    let _ = app.go(Screen::Language);
    let _ = app.update_inner(Message::Click(F::Power));
    let mut ui = Simulator::with_size(
        iced::Settings::default(),
        Size::new(1280.0, 800.0),
        crate::view::view(&app),
    );

    ui.click("Power off").expect("a Power off button");
    ui.click("Restart").expect("a Restart button");
    ui.click("Cancel").expect("a Cancel button");
    let _ = ui.click("Continue");

    let clicks: Vec<F> = ui
        .into_messages()
        .filter_map(|m| match m {
            Message::Click(f) => Some(f),
            _ => None,
        })
        .collect();
    // The last click lands on the scrim over Continue, which closes the menu.
    assert_eq!(
        clicks,
        [F::PowerOff, F::PowerRestart, F::PowerCancel, F::PowerCancel]
    );
}

#[test]
fn a_menu_taking_the_machine_down_lets_no_click_through() {
    use iced_test::simulator::Simulator;
    let (mut app, _) = recorded(Recorder::default());
    let _ = app.go(Screen::Language);
    let _ = app.update_inner(Message::Click(F::Power));
    let _ = app.update_inner(Message::Click(F::PowerOff));
    let mut ui = Simulator::with_size(
        iced::Settings::default(),
        Size::new(1280.0, 800.0),
        crate::view::view(&app),
    );

    let _ = ui.click("Continue");
    let _ = ui.click("Back");

    assert_eq!(ui.into_messages().count(), 0);
}

#[test]
fn cancel_and_esc_close_the_menu_and_touch_nothing() {
    let (mut app, recorder) = recorded(Recorder::default());
    let _ = app.go(Screen::Timezone);
    app.focus = Some(F::Power);

    press(&mut app, &[Key::Enter, Key::Enter]);

    assert!(app.power.is_none(), "Enter on Cancel closes the menu");
    assert_eq!(app.focus, Some(F::Power), "focus goes back where it was");

    press(
        &mut app,
        &[Key::Enter, Key::Text("x".into()), Key::Down, Key::Esc],
    );

    assert!(app.power.is_none(), "Esc closes the menu");
    assert_eq!(
        app.screen,
        Screen::Timezone,
        "Esc in the menu doesn't go back a screen"
    );
    assert_eq!(
        app.field_value(F::TzQ),
        "",
        "typing in the menu doesn't reach the search box"
    );
    assert!(recorder.calls().is_empty());
    assert!(app.shutting_down.is_none());
}

#[test]
fn tab_and_arrows_stay_inside_the_menu() {
    let (mut app, _) = recorded(Recorder::default());
    let _ = app.go(Screen::Git);
    let _ = app.update_inner(Message::Click(F::Power));

    press(&mut app, &[Key::Tab { shift: false }]);
    assert_eq!(app.focus, Some(F::PowerRestart));
    press(
        &mut app,
        &[Key::Right {
            word: false,
            shift: false,
        }],
    );
    assert_eq!(app.focus, Some(F::PowerOff));
    press(&mut app, &[Key::Tab { shift: false }]);
    assert_eq!(app.focus, Some(F::PowerCancel), "Tab wraps inside the menu");
    press(
        &mut app,
        &[Key::Left {
            word: false,
            shift: false,
        }],
    );
    assert_eq!(app.focus, Some(F::PowerOff));
}

#[test]
fn power_off_before_the_install_powers_off_at_once() {
    let (mut app, recorder) = recorded(Recorder::default());
    let _ = app.go(Screen::Disk);
    let _ = app.update_inner(Message::Click(F::Power));

    let _ = app.update_inner(Message::Click(F::PowerOff));

    wait_for(&recorder, "poweroff");
    assert_eq!(recorder.calls(), ["poweroff"]);
    assert_eq!(app.shutting_down, Some(PowerAction::PowerOff));
    assert_eq!(
        crate::view::power_copy(&app).status.as_deref(),
        Some("Powering off…")
    );
}

#[test]
fn restart_before_the_install_restarts_at_once() {
    let (mut app, recorder) = recorded(Recorder::default());
    let _ = app.go(Screen::Welcome);
    let _ = app.update_inner(Message::Click(F::Power));

    press(&mut app, &[Key::Tab { shift: false }, Key::Enter]);

    wait_for(&recorder, "reboot");
    assert_eq!(recorder.calls(), ["reboot"]);
    assert_eq!(app.shutting_down, Some(PowerAction::Restart));
}

#[test]
fn the_menu_before_the_install_says_no_disk_was_touched() {
    let (mut app, _) = recorded(Recorder::default());
    let _ = app.go(Screen::Summary);
    let _ = app.update_inner(Message::Click(F::Power));

    let copy = crate::view::power_copy(&app);

    assert_eq!(copy.title, "Power off or restart?");
    assert!(
        copy.body.contains("Nothing has been written"),
        "{}",
        copy.body
    );
    assert!(!copy.danger);
    assert_eq!((copy.off, copy.restart), ("Power off", "Restart"));
}

fn installing(recorder: Recorder) -> (App, Arc<Recorder>) {
    let (mut app, recorder) = recorded(recorder);
    let _ = app.go(Screen::Summary);
    let _ = app.start_install();
    tick_until(&mut app, "the install to start", |a| {
        a.install.as_ref().is_some_and(|i| i.current().is_some())
    });
    (app, recorder)
}

#[test]
fn the_menu_while_installing_warns_the_disk_is_left_unbootable() {
    let (mut app, recorder) = installing(Recorder::default());
    let _ = app.update_inner(Message::Click(F::Power));
    let disk = app.form.disk.clone().expect("a disk is picked");

    let copy = crate::view::power_copy(&app);

    assert_eq!(copy.title, "Stop the install?");
    assert!(copy.danger);
    assert!(copy.body.contains(&disk), "{}", copy.body);
    assert!(copy.body.contains("won't boot"), "{}", copy.body);
    assert!(
        copy.body.contains("install KubaOS on it again"),
        "{}",
        copy.body
    );
    assert_eq!(
        (copy.off, copy.restart),
        ("Stop and power off", "Stop and restart")
    );
    assert!(
        recorder.calls().is_empty(),
        "opening the menu doesn't stop anything"
    );
}

#[test]
fn power_off_while_installing_stops_the_install_first() {
    let (mut app, recorder) = installing(Recorder::default());
    let _ = app.update_inner(Message::Click(F::Power));

    let _ = app.update_inner(Message::Click(F::PowerOff));

    assert_eq!(
        recorder.calls(),
        ["stop"],
        "nothing goes down before the install stops"
    );
    assert!(app.power.is_some_and(|m| m.stopping.is_some()));
    let status = crate::view::power_copy(&app).status.unwrap_or_default();
    assert!(status.starts_with("Stopping the install"), "{status}");
    press(&mut app, &[Key::Esc]);
    assert!(app.power.is_some(), "a stopping install can't be cancelled");

    tick_until(&mut app, "the install to end", |a| {
        a.shutting_down.is_some()
    });

    wait_for(&recorder, "poweroff");
    assert_eq!(recorder.calls(), ["stop", "poweroff"]);
    assert_eq!(app.shutting_down, Some(PowerAction::PowerOff));
    assert!(
        app.power.is_some(),
        "the menu stays up while the machine goes down"
    );
}

#[test]
fn restart_while_installing_stops_the_install_then_restarts() {
    let (mut app, recorder) = installing(Recorder::default());
    let _ = app.update_inner(Message::Click(F::Power));

    let _ = app.update_inner(Message::Click(F::PowerRestart));
    tick_until(&mut app, "the install to end", |a| {
        a.shutting_down.is_some()
    });

    wait_for(&recorder, "reboot");
    assert_eq!(recorder.calls(), ["stop", "reboot"]);
}

#[test]
fn an_install_that_won_t_stop_still_powers_off_after_the_grace_period() {
    let (mut app, recorder) = installing(Recorder {
        stubborn: true,
        ..Recorder::default()
    });
    let _ = app.update_inner(Message::Click(F::Power));
    let _ = app.update_inner(Message::Click(F::PowerOff));

    let _ = app.update_inner(Message::Tick(Instant::now() + STOP_GRACE / 2));
    assert!(
        app.shutting_down.is_none(),
        "too early to give up on the install"
    );
    let _ = app.update_inner(Message::Tick(Instant::now() + STOP_GRACE));

    wait_for(&recorder, "poweroff");
    assert_eq!(recorder.calls(), ["stop", "poweroff"]);
}

#[test]
fn a_refused_power_off_closes_the_menu_and_says_so() {
    let (mut app, _) = recorded(Recorder::default());
    let _ = app.go(Screen::Account);
    app.focus = Some(F::Power);
    let _ = app.update_inner(Message::Key(Key::Enter));
    let _ = app.update_inner(Message::Click(F::PowerOff));

    let _ = app.update_inner(Message::ShutDown(PowerAction::PowerOff, Err("no".into())));

    assert!(app.power.is_none());
    assert!(app.shutting_down.is_none());
    assert_eq!(app.focus, Some(F::Power));
    let (toast, _) = app.toast.as_ref().expect("a toast");
    assert!(toast.starts_with("Power off refused: no."), "{toast}");
}

#[test]
fn the_done_screen_s_reboot_restarts_without_asking() {
    let (mut app, recorder) = recorded(Recorder::default());
    let _ = app.go(Screen::Done);
    assert_eq!(app.focus, Some(F::Reboot));

    press(&mut app, &[Key::Enter]);

    wait_for(&recorder, "reboot");
    assert!(app.power.is_none());
    assert_eq!(app.shutting_down, Some(PowerAction::Restart));
}

fn on_battery(percent: u8, supply: Supply) -> Message {
    Message::Battery(Some(Battery { percent, supply }))
}

fn draws(app: &App, text: &str) -> bool {
    use iced_test::simulator::Simulator;
    let mut ui = Simulator::with_size(
        iced::Settings::default(),
        Size::new(1280.0, 800.0),
        crate::view::view(app),
    );
    ui.find(text).is_ok()
}

#[test]
fn every_screen_shows_the_battery_beside_the_power_button() {
    for screen in EVERY_SCREEN {
        let (mut app, _) = recorded(Recorder::default());
        let _ = app.go(screen);

        let _ = app.update_inner(on_battery(64, Supply::Discharging));

        assert!(draws(&app, "64%"), "no battery on {screen:?}");
    }
}

#[test]
fn a_machine_without_a_battery_shows_none() {
    let (mut app, _) = recorded(Recorder::default());
    let _ = app.go(Screen::Language);
    let _ = app.update_inner(on_battery(64, Supply::Discharging));

    let _ = app.update_inner(Message::Battery(None));

    assert!(!draws(&app, "64%"));
}

#[test]
fn a_low_battery_on_last_chance_asks_for_the_charger_and_still_installs() {
    let (mut app, _) = recorded(Recorder::default());
    let _ = app.go(Screen::Summary);

    let _ = app.update_inner(on_battery(14, Supply::Discharging));

    let note = crate::view::low_battery_note(&app).expect("a low battery note");
    assert_eq!(
        note,
        "Battery at 14%. Plug in the charger: the install can take a while."
    );
    assert!(draws(&app, &note));
    assert_eq!(app.blocker(), None);
}

#[test]
fn a_battery_on_the_charger_asks_for_nothing() {
    let (mut app, _) = recorded(Recorder::default());
    let _ = app.go(Screen::Summary);

    let _ = app.update_inner(on_battery(14, Supply::Charging));

    assert_eq!(crate::view::low_battery_note(&app), None);
    assert!(draws(&app, "14%"));
}

#[test]
fn the_demo_backend_has_a_battery_to_show() {
    let demo = crate::backend::Demo::new(40.0, None);

    assert_eq!(demo.battery(), Some(crate::backend::DEMO_BATTERY));
}

/// The sizes Kōbō lays out at: the gallery's, and what `auto_scale` leaves of larger outputs.
const LAYOUT_SIZES: [(f32, f32); 5] = [
    (1024.0, 768.0),
    (1280.0, 720.0),
    (1280.0, 800.0),
    (1440.0, 900.0),
    (1536.0, 864.0),
];

#[test]
fn last_chance_shows_its_cards_as_an_even_two_by_two_grid() {
    use iced::widget::Id;
    use iced_test::simulator::Simulator;
    let cards = [
        ids::SUMMARY_LANGUAGE,
        ids::SUMMARY_ACCOUNT,
        ids::SUMMARY_DISK,
        ids::SUMMARY_SYSTEM,
    ];
    for (w, h) in LAYOUT_SIZES {
        let (mut app, _) = recorded(Recorder::default());
        app.form.full_name = "Kuba Example".into();
        let _ = app.go(Screen::Summary);
        let mut ui = Simulator::with_size(
            iced::Settings::default(),
            Size::new(w, h),
            crate::view::view(&app),
        );

        let b: Vec<_> = cards
            .iter()
            .map(|id| ui.find(Id::new(id)).expect("a summary card").bounds())
            .collect();

        for card in &b {
            assert_eq!(card.width, b[0].width, "at {w}x{h}");
            assert_eq!(card.height, b[0].height, "at {w}x{h}");
        }
        assert_eq!(b[0].y, b[1].y, "at {w}x{h}");
        assert_eq!(b[2].y, b[3].y, "at {w}x{h}");
        assert_eq!(b[0].x, b[2].x, "at {w}x{h}");
        assert!(b[1].x > b[0].x + b[0].width, "at {w}x{h}");
        assert!(b[2].y > b[0].y + b[0].height, "at {w}x{h}");
    }
}

#[test]
fn last_chance_has_no_edit_buttons_and_tab_goes_from_hold_to_back() {
    let (mut app, _) = recorded(Recorder::default());
    let _ = app.go(Screen::Summary);

    assert_eq!(app.focus_order(), vec![F::Hold, F::Back, F::Power]);
    assert!(!draws(&app, "Edit"));
    assert!(
        !draws(&app, "edit"),
        "the rail's key hints offer no edit either"
    );
}

#[test]
fn the_rail_still_takes_you_back_from_last_chance() {
    let (mut app, _) = recorded(Recorder::default());
    let _ = app.go(Screen::Summary);

    let _ = app.update_inner(Message::Click(F::Step(Screen::Account)));

    assert_eq!(app.screen, Screen::Account);
}

#[test]
fn the_hostname_starts_as_the_payload_s_and_takes_what_you_type() {
    let (mut app, _) = recorded(Recorder::default());
    let _ = app.go(Screen::Account);
    assert_eq!(app.form.hostname, "tuxedo");
    assert!(draws(&app, "kuba@tuxedo"));

    press(
        &mut app,
        &[Key::Tab { shift: false }, Key::Tab { shift: false }],
    );
    assert_eq!(app.focus, Some(F::Hostname));
    press(
        &mut app,
        &[Key::Text("nerv-hq".into()), Key::Tab { shift: false }],
    );

    assert_eq!(app.form.hostname, "nerv-hq");
    assert_eq!(app.focus, Some(F::Password));
    assert!(draws(&app, "kuba@nerv-hq"));
    let probe = app.ready_probe().expect("the demo probe");
    assert_eq!(app.form.to_config(probe).host.hostname, "nerv-hq");
}

#[test]
fn a_bad_hostname_says_why_and_holds_continue() {
    let (mut app, _) = recorded(Recorder::default());
    app.form.full_name = "Kuba Example".into();
    app.form.password = "correct horse battery".into();
    app.form.password_confirm = app.form.password.clone();
    let _ = app.go(Screen::Account);
    assert_eq!(app.blocker(), None);

    app.form.hostname = "Nerv.HQ".into();

    assert!(app.blocker().is_some());
    assert!(draws(&app, crate::model::HOSTNAME_RULE));
}

fn on_wallpapers() -> App {
    let (mut app, _) = recorded(Recorder::default());
    let _ = app.go(Screen::Wallpapers);
    app
}

fn picked(app: &App) -> u32 {
    app.form.wallpapers_picked(app.wallpaper_sets())
}

const TAB: Key = Key::Tab { shift: false };
const SHIFT_TAB: Key = Key::Tab { shift: true };
const LEFT: Key = Key::Left {
    word: false,
    shift: false,
};
const RIGHT: Key = Key::Right {
    word: false,
    shift: false,
};

#[test]
fn the_wallpapers_screen_opens_on_the_first_set_with_every_wallpaper_picked() {
    let app = on_wallpapers();

    assert_eq!(app.focus, Some(F::WallpaperSet(0)));
    assert_eq!(picked(&app), 10);
    assert_eq!(app.blocker(), None);
}

#[test]
fn tab_goes_from_set_to_set_and_the_arrows_through_a_sets_thumbnails() {
    let mut app = on_wallpapers();

    press(&mut app, &[TAB, TAB]);
    assert_eq!(app.focus, Some(F::WallpaperSet(2)), "shinjuku, the third set");
    press(&mut app, &[Key::Down]);
    assert_eq!(app.focus, Some(F::Wallpaper(2, 0)));
    press(&mut app, &[RIGHT, RIGHT, RIGHT, RIGHT]);
    assert_eq!(app.focus, Some(F::Wallpaper(2, 4)), "the second row");
    press(&mut app, &[Key::Up]);
    assert_eq!(app.focus, Some(F::Wallpaper(2, 0)), "the column above");
    press(&mut app, &[Key::Up]);
    assert_eq!(app.focus, Some(F::WallpaperSet(2)), "out of the grid to its header");
    press(&mut app, &[Key::Up]);
    assert_eq!(app.focus, Some(F::Wallpaper(1, 0)), "the set above's last row");

    press(&mut app, &[RIGHT, TAB]);
    assert_eq!(app.focus, Some(F::WallpaperSet(2)), "Tab leaves a thumbnail for the next set");
    press(&mut app, &[Key::Down, SHIFT_TAB]);
    assert_eq!(app.focus, Some(F::WallpaperSet(2)), "Shift+Tab, back to its own header");
}

#[test]
fn the_arrows_stop_at_the_edges_and_down_from_a_short_last_row_goes_to_the_next_set() {
    let app = on_wallpapers();

    assert_eq!(app.wallpaper_neighbour(F::Wallpaper(0, 2), Dir::Down), Some(F::WallpaperSet(1)));
    assert_eq!(app.wallpaper_neighbour(F::Wallpaper(2, 1), Dir::Down), Some(F::Wallpaper(2, 4)));
    assert_eq!(app.wallpaper_neighbour(F::Wallpaper(2, 4), Dir::Down), None, "the last set");
    assert_eq!(app.wallpaper_neighbour(F::Wallpaper(2, 4), Dir::Right), None);
    assert_eq!(app.wallpaper_neighbour(F::Wallpaper(2, 0), Dir::Left), None);
    assert_eq!(app.wallpaper_neighbour(F::WallpaperSet(0), Dir::Up), None);
    assert_eq!(app.wallpaper_neighbour(F::WallpaperSet(1), Dir::Left), None);
}

#[test]
fn space_drops_and_picks_a_thumbnail_and_a_whole_set() {
    let mut app = on_wallpapers();

    press(&mut app, &[Key::Down, Key::Space]);
    assert_eq!(picked(&app), 9, "cyberpunk's first dropped");
    press(&mut app, &[Key::Up, Key::Space]);
    assert_eq!(picked(&app), 10, "the header picks the rest of the set");
    press(&mut app, &[Key::Space]);
    assert_eq!(picked(&app), 7, "and drops it when it's all picked");
    press(&mut app, &[Key::Down, LEFT, Key::Space]);
    assert_eq!(picked(&app), 8, "Left at a row's start stays put");
}

#[test]
fn continue_waits_for_at_least_one_wallpaper() {
    let mut app = on_wallpapers();
    press(&mut app, &[Key::Space, TAB, Key::Space, TAB, Key::Space]);
    assert_eq!(picked(&app), 0);

    press(&mut app, &[Key::Enter]);

    assert_eq!(app.screen, Screen::Wallpapers);
    assert_eq!(app.blocker(), Some("Pick at least one wallpaper."));
    assert!(app.nudged.is_some(), "the reason flashes");

    let _ = app.update_inner(Message::Click(F::Wallpaper(1, 1)));
    let _ = app.update_inner(Message::Click(F::Next));

    assert_eq!(app.screen, Screen::Summary);
}

#[test]
fn a_click_on_a_thumbnail_focuses_and_toggles_it() {
    use iced_test::simulator::Simulator;
    let mut app = on_wallpapers();
    let mut ui = Simulator::with_size(
        iced::Settings::default(),
        Size::new(1280.0, 800.0),
        crate::view::view(&app),
    );

    let tokyo_tower = wallpaper_item(F::Wallpaper(1, 1)).unwrap();
    ui.click(iced::widget::Id::from(tokyo_tower)).expect("the thumbnail drawn");

    for m in ui.into_messages().collect::<Vec<_>>() {
        let _ = app.update_inner(m);
    }
    assert_eq!(app.focus, Some(F::Wallpaper(1, 1)));
    assert_eq!(picked(&app), 9);
}

#[test]
fn the_wallpapers_screen_counts_what_is_picked() {
    let mut app = on_wallpapers();
    assert!(draws(&app, "All 10 picked."));

    press(&mut app, &[TAB, Key::Space]);

    assert!(draws(&app, "8 of 10 picked."));
    assert!(draws(&app, "2 wallpapers"), "the dropped set's header");
}

/// The demo machine on the Language screen, the keymap search focused.
fn on_the_keymap_list() -> App {
    let (mut app, _) = recorded(Recorder::default());
    let _ = app.go(Screen::Language);
    press(&mut app, &[Key::Tab { shift: false }]);
    assert_eq!(app.focus, Some(F::KeymapQ));
    app
}

#[test]
fn space_on_the_keymap_list_picks_the_row_and_again_unpicks_it() {
    let mut app = on_the_keymap_list();
    assert_eq!(app.form.keymaps, ["us"]);

    press(&mut app, &[Key::Down, Key::Space]);
    let row = app.keymap_row();
    assert_ne!(row, "us", "Down moves off the default layout");
    assert_eq!(app.form.keymaps, ["us".to_string(), row.clone()]);
    assert!(
        draws(&app, "Default"),
        "with two picked, the first says it is the default"
    );

    press(&mut app, &[Key::Space]);
    assert_eq!(app.form.keymaps, ["us"]);
}

#[test]
fn typing_a_layout_and_enter_picks_it_and_clears_the_search() {
    let mut app = on_the_keymap_list();

    press(&mut app, &[Key::Text("ital".into()), Key::Enter]);

    assert_eq!(app.form.keymaps, ["us", "it"]);
    assert_eq!(app.keymap_q, "");
    assert_eq!(app.screen, Screen::Language, "the first Enter picks");
    press(&mut app, &[Key::Enter]);
    assert_eq!(
        app.screen,
        Screen::Timezone,
        "with nothing typed, Enter goes on"
    );
}

#[test]
fn shift_space_makes_the_row_the_default_layout() {
    let mut app = on_the_keymap_list();

    press(
        &mut app,
        &[Key::Text("ital".into()), Key::Enter, Key::ShiftSpace],
    );
    assert_eq!(
        app.form.keymaps,
        ["it", "us"],
        "the row Enter picked is still the one Space acts on"
    );

    let probe = app.ready_probe().expect("the demo probe").clone();
    let cfg = app.form.to_config(&probe);
    assert_eq!(cfg.locale.keyboard_layout, ["it", "us"]);
    let problems = kubaos_core::engine::validate::problems(&cfg, &probe);
    assert!(
        !problems.iter().any(|p| p.contains("keyboard")),
        "{problems:#?}"
    );
}

#[test]
fn a_search_still_takes_a_space() {
    let mut app = on_the_keymap_list();

    press(
        &mut app,
        &[Key::Text("english".into()), Key::Space, Key::ShiftSpace],
    );

    assert_eq!(app.keymap_q, "english  ");
    assert_eq!(app.form.keymaps, ["us"]);
}

#[test]
fn a_click_on_a_row_toggles_it_and_the_chips_reorder_and_drop() {
    let mut app = on_the_keymap_list();

    let _ = app.update_inner(Message::Pick(List::Keymap, "de".into()));
    let _ = app.update_inner(Message::Pick(List::Keymap, "pl".into()));
    assert_eq!(app.form.keymaps, ["us", "de", "pl"]);
    let _ = app.update_inner(Message::KeymapDefault("pl".into()));
    assert_eq!(app.form.keymaps, ["pl", "us", "de"]);
    let _ = app.update_inner(Message::KeymapDrop("us".into()));
    assert_eq!(app.form.keymaps, ["pl", "de"]);
    let _ = app.update_inner(Message::Pick(List::Keymap, "de".into()));

    assert_eq!(app.form.keymaps, ["pl"]);
}

#[test]
fn the_last_layout_stays_and_a_toast_says_why() {
    let mut app = on_the_keymap_list();

    press(&mut app, &[Key::Space]);

    assert_eq!(app.form.keymaps, ["us"]);
    let toast = app.toast.as_ref().map(|(msg, _)| msg.as_str());
    assert_eq!(toast, Some("Keep one layout at least."));
}

#[test]
fn last_chance_lists_every_layout_the_default_first() {
    let mut app = on_the_keymap_list();
    let _ = app.update_inner(Message::Pick(List::Keymap, "it".into()));

    let _ = app.go(Screen::Summary);

    assert!(draws(&app, "English (US), Italian"));
}
