//! State, messages and update logic. The views live in `view.rs`.
//!
//! Keyboard first (docs/BRAND.md): the app owns focus. Every screen has an
//! ordered list of focus targets (`App::focus_order`); Tab moves through it,
//! and keys act on whatever has focus. Text fields are `kubaos-ui`'s, which draw
//! and leave the editing to the app (`edit.rs`), so typing goes through `App::key` too. That keeps
//! one code path for real keys and the synthetic ones the key tour and the
//! tests send.
//!
//! The power menu opens over every screen (`F::Power`). While it's open it owns the keys and the
//! focus, and while an install runs it stops the install before the machine goes down.
//!
//! The battery is read on a thread of its own every `battery::POLL` (`battery_watch`), since a
//! read can wait on the firmware.

use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use iced::Size;
use iced::Subscription;
use iced::Task;
use iced::event;
use iced::keyboard;
use iced::keyboard::key::Named;
use iced::widget::image;
use iced::widget::operation;
use iced::window;
use kubaos_core::Probe;
use kubaos_ui::Look;
use kubaos_ui::Mode;
use kubaos_ui::Scale;
use kubaos_ui::component::field::Caret;
use kubaos_ui::component::field::Pointer;
use kubaos_ui::component::toast;
use kubaos_ui::motion::Motion;
use kubaos_ui::tokens;
use kubaos_ui::tokens::Easing;

use crate::backend::Backend;
use crate::backend::RunMsg;
use crate::battery;
use crate::battery::Battery;
use crate::edit;
use crate::edit::Edit;
use crate::model::DiskExt;
use crate::model::DiskMode;
use crate::model::Form;
use crate::progress::Install;
use crate::progress::Outcome;
use crate::theme;
use crate::tour::Tour;

/// Shown when the manifest doesn't name the release.
pub const CODENAME: &str = "Shinjuku";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    Language,
    Timezone,
    Wifi,
    Account,
    Disk,
    Git,
    Wallpapers,
    Summary,
    Installing,
    Done,
    Failed,
}

impl Screen {
    pub fn rail_label(self) -> &'static str {
        match self {
            Screen::Welcome => "Welcome",
            Screen::Language => "Language",
            Screen::Timezone => "Time zone",
            Screen::Wifi => "Network",
            Screen::Account => "You",
            Screen::Disk => "Disk",
            Screen::Git => "Git",
            Screen::Wallpapers => "Wallpapers",
            Screen::Summary => "Last chance",
            Screen::Installing | Screen::Done | Screen::Failed => "Install",
        }
    }

    /// Screens shown in the panel (everything but the full-bleed ones).
    pub fn in_panel(self) -> bool {
        !matches!(self, Screen::Welcome | Screen::Done)
    }
}

/// Everything that can have keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum F {
    Start,
    RetryProbe,
    LocaleQ,
    KeymapQ,
    KeyboardTest,
    TzQ,
    WifiList,
    WifiPsk,
    FullName,
    Username,
    Hostname,
    Password,
    PasswordConfirm,
    ShowSecrets,
    DiskList,
    DiskMode,
    Encrypt,
    LuksSame,
    Luks,
    LuksConfirm,
    GitName,
    GitEmail,
    /// A set's header on the Wallpapers screen: the manifest's set at this index.
    WallpaperSet(usize),
    /// A thumbnail on the Wallpapers screen: the set's file at the second index.
    Wallpaper(usize, usize),
    /// A step in the rail, to go back to.
    Step(Screen),
    Hold,
    ToggleLog,
    SaveLog,
    Retry,
    Reboot,
    Stay,
    Back,
    Next,
    /// Opens the power menu, on every screen.
    Power,
    /// The power menu's buttons.
    PowerCancel,
    PowerRestart,
    PowerOff,
}

impl F {
    pub fn is_field(self) -> bool {
        matches!(
            self,
            F::LocaleQ
                | F::KeymapQ
                | F::KeyboardTest
                | F::TzQ
                | F::WifiPsk
                | F::FullName
                | F::Username
                | F::Hostname
                | F::Password
                | F::PasswordConfirm
                | F::Luks
                | F::LuksConfirm
                | F::GitName
                | F::GitEmail
        )
    }

    fn in_power_menu(self) -> bool {
        matches!(self, F::PowerCancel | F::PowerRestart | F::PowerOff)
    }

    pub fn is_secret(self) -> bool {
        matches!(
            self,
            F::WifiPsk | F::Password | F::PasswordConfirm | F::Luks | F::LuksConfirm
        )
    }
}

pub mod ids {
    pub const LOCALE_LIST: &str = "locale-list";
    pub const KEYMAP_LIST: &str = "keymap-list";
    pub const TZ_LIST: &str = "tz-list";
    pub const WIFI_LIST: &str = "wifi-list";
    pub const LOG: &str = "log";
    pub const DISK_SCROLL: &str = "disk-scroll";
    pub const ACCOUNT_SCROLL: &str = "account-scroll";
    pub const WALLPAPER_SCROLL: &str = "wallpaper-scroll";
    pub const REVEAL_DISKS: &str = "reveal-disks";
    pub const REVEAL_MODE: &str = "reveal-mode";
    pub const REVEAL_ENCRYPT: &str = "reveal-encrypt";
    pub const REVEAL_LUKS_SAME: &str = "reveal-luks-same";
    pub const REVEAL_LUKS: &str = "reveal-luks";
    pub const REVEAL_NAMES: &str = "reveal-names";
    pub const REVEAL_HOST: &str = "reveal-host";
    pub const REVEAL_PASSWORDS: &str = "reveal-passwords";
    pub const SUMMARY_LANGUAGE: &str = "summary-language";
    pub const SUMMARY_ACCOUNT: &str = "summary-account";
    pub const SUMMARY_DISK: &str = "summary-disk";
    pub const SUMMARY_SYSTEM: &str = "summary-system";
    pub const SUMMARY_SCROLL: &str = "summary-scroll";
}

/// Which list the arrow keys move in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum List {
    Locale,
    Keymap,
    Timezone,
    Wifi,
    Disk,
}

/// A key, already translated from iced's event (see `on_event`).
#[derive(Debug, Clone, PartialEq)]
pub enum Key {
    Enter,
    EnterUp,
    Esc,
    Tab {
        shift: bool,
    },
    Up,
    Down,
    /// `word`: Ctrl (Alt on a Mac) jumps words. `shift` extends the selection.
    Left {
        word: bool,
        shift: bool,
    },
    Right {
        word: bool,
        shift: bool,
    },
    Home {
        shift: bool,
    },
    End {
        shift: bool,
    },
    PageUp,
    PageDown,
    Space,
    /// Shift+Space: on the keymap list, makes the row the default layout.
    ShiftSpace,
    Backspace {
        word: bool,
    },
    Delete {
        word: bool,
    },
    /// Ctrl+U: clear the field.
    Clear,
    SelectAll,
    Copy,
    Cut,
    Paste,
    Text(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
    Window(Option<window::Id>),
    Resized(Size),
    /// The display's own scale, under the UI's: on its first frame, and when it moves to another.
    Rescaled(f32),
    Key(Key),
    /// The mouse moved or clicked: show the pointer again.
    PointerMoved,
    Wallpaper(image::Handle),
    /// Thumbnails of the Wallpapers screen, decoded, keyed like `model::wallpaper_key`.
    Thumbs(Vec<(String, image::Handle)>),
    Pasted(Option<String>),

    /// Mouse: clicking a control focuses it and acts like Enter/Space on it.
    Click(F),
    Pick(List, String),
    /// A click on a picked layout's chip: make it the default.
    KeymapDefault(String),
    /// A click on a picked layout's remove button.
    KeymapDrop(String),
    /// A click or drag in a text field.
    FieldPointer(F, Pointer),
    /// A list scrolled: its id, offset and visible height.
    Scrolled(&'static str, f32, f32),
    /// Where the focused control sits in a scrollable (see `reveal`).
    Reveal(&'static str, crate::reveal::Found),
    WifiNone,
    DiskModePick(DiskMode),

    HoldStart,
    HoldEnd,
    /// `systemctl poweroff` or `reboot` answered.
    ShutDown(PowerAction, Result<(), String>),
    /// A fresh battery reading: `None` on a machine without one.
    Battery(Option<Battery>),

    TourShot(String, window::Screenshot),

    /// First-boot setup's own messages (`setup::Msg`).
    Setup(crate::setup::Msg),
}

#[derive(Debug)]
pub enum ProbeState {
    Loading,
    Failed(String),
    Ready(Box<Probe>),
}

#[derive(Debug, Clone)]
pub struct Options {
    pub tour: Option<PathBuf>,
    pub tour_pace: f32,
    pub fullscreen: bool,
    /// `--scale`: use this UI scale instead of picking one.
    pub scale: Option<f32>,
}

/// How long the reason a screen can't continue takes to fade back from the error colour.
pub const NUDGE: Duration = Duration::from_millis(tokens::Duration::Slow.ms() as u64);

/// How long the Summary's button (or Enter) must be held.
// design-allow(duration): how long a press must last before it erases a disk, a safety, not motion.
pub const HOLD: Duration = Duration::from_millis(1100);

/// How long a stopping install gets to unwind before the machine goes down anyway. A command that
/// doesn't stream (a `mkfs`, `grub-install`) runs to its end before the engine sees the stop, and
/// systemd unmounts whatever is left on the way down.
pub const STOP_GRACE: Duration = Duration::from_secs(30);

/// What the power menu does to the machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
    PowerOff,
    Restart,
}

/// The power menu, open over the screen it was opened on.
#[derive(Debug, Clone, Copy)]
pub struct PowerMenu {
    /// Where focus goes back to when the menu closes.
    return_focus: Option<F>,
    /// The install is being stopped, since then; the action follows once it has.
    pub stopping: Option<(PowerAction, Instant)>,
}

pub struct App {
    pub backend: Arc<dyn Backend>,
    pub probe: ProbeState,
    probe_rx: Option<mpsc::Receiver<Result<Probe, String>>>,
    pub form: Form,
    pub screen: Screen,
    /// Furthest screen reached, for the rail.
    pub reached: usize,
    pub transition: (Instant, bool),
    pub now: Instant,
    pub epoch: Instant,
    pub window: Option<window::Id>,
    pub size: Size,
    pub wallpaper: Option<(image::Handle, Instant)>,
    pub glass: Option<(Size, image::Handle)>,
    blur_src: Option<::image::RgbaImage>,
    /// Where the payload's wallpaper thumbnails are read from.
    pub payload_dir: PathBuf,
    /// The wallpaper thumbnails decoded so far, keyed like `model::wallpaper_key`.
    pub thumbs: HashMap<String, image::Handle>,

    pub locale_q: String,
    pub keymap_q: String,
    /// The keymap row the arrows have reached, which Space picks; `None` is the default layout's.
    pub keymap_cursor: Option<String>,
    pub tz_q: String,
    pub list: List,
    pub focus: Option<F>,
    /// When focus or the caret last moved; the caret blinks from here.
    pub focus_at: Instant,
    /// Caret and selection in the focused text field.
    pub caret: Caret,
    /// Typing hides the pointer and its hover until the mouse moves.
    pub pointer_idle: bool,
    /// Enter on a screen that can't continue yet: the reason flashes.
    pub nudged: Option<Instant>,
    /// UI scale on top of the system's (see `auto_scale`).
    pub ui_scale: f32,
    /// The display's own scale, under `ui_scale`: the two together are the device pixels per
    /// logical pixel that the views snap to.
    pub system_scale: f32,
    /// Light or dark. Dark on the live ISO; the screenshots draw both.
    pub mode: Mode,
    fixed_scale: Option<f32>,
    /// The switch that last flipped, and when (its knob slides).
    pub switched: Option<(F, Instant)>,
    /// Scroll offset and height of each list, as last reported.
    viewports: HashMap<&'static str, (f32, f32)>,
    /// A long screen gliding to the focused control.
    glide: Option<crate::reveal::Glide>,
    pub show_secrets: bool,

    pub hold: Option<Instant>,
    pub power: Option<PowerMenu>,
    /// Power off or restart asked of systemd: the system is going down.
    pub shutting_down: Option<PowerAction>,
    /// The last battery reading; `None` until the first one, and on a machine without a battery.
    pub battery: Option<Battery>,
    pub install: Option<Install>,
    run_rx: Option<mpsc::Receiver<RunMsg>>,
    pub log_open: (bool, Instant),
    pub toast: Option<(String, Instant)>,
    pub tour: Option<Tour>,
    last_tick: Instant,
    /// `--fullscreen`, applied after the first frame (see `tick`).
    want_fullscreen: bool,
}

impl App {
    pub fn new(backend: Arc<dyn Backend>, opts: &Options) -> (Self, Task<Message>) {
        let mut app = Self::headless(backend);
        app.blur_src = decode_blur();
        app.tour = opts.tour.clone().map(|d| Tour::new(d, opts.tour_pace));
        app.want_fullscreen = opts.fullscreen;
        app.fixed_scale = opts.scale;
        app.ui_scale = opts.scale.unwrap_or(1.0);
        let tasks = Task::batch([
            window::oldest().map(Message::Window),
            Task::perform(decode_wallpaper(), Message::Wallpaper),
        ]);
        (app, tasks)
    }

    /// The app without a window: what the keyboard tests drive.
    pub fn headless(backend: Arc<dyn Backend>) -> Self {
        let now = Instant::now();
        let (tx, rx) = mpsc::channel();
        let b = Arc::clone(&backend);
        std::thread::spawn(move || {
            let _ = tx.send(b.probe());
        });
        Self {
            backend,
            probe: ProbeState::Loading,
            probe_rx: Some(rx),
            form: Form::default(),
            screen: Screen::Welcome,
            reached: 0,
            transition: (now, true),
            now,
            epoch: now,
            window: None,
            // design-allow(size): the window's opening size, not layout.
            size: Size::new(1280.0, 800.0),
            wallpaper: None,
            glass: None,
            blur_src: None,
            payload_dir: PathBuf::from(crate::model::PAYLOAD_DIR),
            thumbs: HashMap::new(),
            locale_q: String::new(),
            keymap_q: String::new(),
            keymap_cursor: None,
            tz_q: String::new(),
            list: List::Locale,
            focus: Some(F::Start),
            focus_at: now,
            caret: Caret::default(),
            switched: None,
            ui_scale: 1.0,
            system_scale: 1.0,
            mode: Mode::Dark,
            fixed_scale: None,
            nudged: None,
            pointer_idle: false,
            viewports: HashMap::new(),
            glide: None,
            show_secrets: false,
            hold: None,
            power: None,
            shutting_down: None,
            battery: None,
            install: None,
            run_rx: None,
            log_open: (false, now),
            toast: None,
            tour: None,
            last_tick: now,
            want_fullscreen: false,
        }
    }

    pub fn ready_probe(&self) -> Option<&Probe> {
        match &self.probe {
            ProbeState::Ready(p) => Some(p),
            _ => None,
        }
    }

    /// The release name, from the manifest; "Shinjuku" for older images.
    pub fn codename(&self) -> &str {
        self.ready_probe()
            .and_then(|p| p.payload.as_ref())
            .and_then(|m| m.codename.as_deref())
            .unwrap_or(CODENAME)
    }

    /// The screens of this install, in order.
    pub fn flow(&self) -> Vec<Screen> {
        let online = self.ready_probe().is_some_and(|p| p.online);
        let mut v = vec![Screen::Welcome, Screen::Language, Screen::Timezone];
        if !online {
            v.push(Screen::Wifi);
        }
        v.extend([Screen::Account, Screen::Disk, Screen::Git]);
        // An older payload has no sets to pick from.
        if !self.wallpaper_sets().is_empty() {
            v.push(Screen::Wallpapers);
        }
        v.extend([Screen::Summary, Screen::Installing]);
        v
    }

    /// The payload's wallpaper sets, in its order.
    pub fn wallpaper_sets(&self) -> &[kubaos_core::probe::WallpaperSet] {
        self.ready_probe()
            .and_then(|p| p.payload.as_ref())
            .map_or(&[], |m| m.wallpaper_sets.as_slice())
    }

    pub fn flow_index(&self, s: Screen) -> usize {
        let s = if matches!(s, Screen::Done | Screen::Failed) {
            Screen::Installing
        } else {
            s
        };
        self.flow().iter().position(|x| *x == s).unwrap_or(0)
    }

    /// The focus targets of the current screen, or of the power menu when it's open, in Tab order.
    pub fn focus_order(&self) -> Vec<F> {
        if self.power.is_some() {
            return vec![F::PowerCancel, F::PowerRestart, F::PowerOff];
        }
        let mut v = self.screen_focus_order();
        v.push(F::Power);
        v
    }

    fn screen_focus_order(&self) -> Vec<F> {
        let f = &self.form;
        match self.screen {
            Screen::Welcome => match self.probe {
                ProbeState::Failed(_) => vec![F::RetryProbe],
                _ => vec![F::Start],
            },
            Screen::Language => vec![F::LocaleQ, F::KeymapQ, F::KeyboardTest, F::Back, F::Next],
            Screen::Timezone => vec![F::TzQ, F::Back, F::Next],
            Screen::Wifi => {
                let mut v = vec![F::WifiList];
                if self.wifi_secured(f.wifi_ssid.as_deref()) {
                    v.extend([F::WifiPsk, F::ShowSecrets]);
                }
                v.extend([F::Back, F::Next]);
                v
            }
            Screen::Account => {
                vec![
                    F::FullName,
                    F::Username,
                    F::Hostname,
                    F::Password,
                    F::PasswordConfirm,
                    F::ShowSecrets,
                    F::Back,
                    F::Next,
                ]
            }
            Screen::Disk => {
                let mut v = vec![F::DiskList, F::DiskMode, F::Encrypt];
                if f.encrypt {
                    v.push(F::LuksSame);
                    if !f.luks_same_as_password {
                        v.extend([F::Luks, F::LuksConfirm, F::ShowSecrets]);
                    }
                }
                v.extend([F::Back, F::Next]);
                v
            }
            Screen::Git => vec![F::GitName, F::GitEmail, F::Back, F::Next],
            Screen::Wallpapers => {
                // Tab goes from set to set; the arrows move through a set's thumbnails. The
                // focused thumbnail sits after its set's header, so Tab leaves it for the next set.
                let mut v = Vec::new();
                for s in 0..self.wallpaper_sets().len() {
                    v.push(F::WallpaperSet(s));
                    if let Some(f @ F::Wallpaper(fs, _)) = self.focus
                        && fs == s
                    {
                        v.push(f);
                    }
                }
                v.extend([F::Back, F::Next]);
                v
            }
            Screen::Summary => vec![F::Hold, F::Back],
            Screen::Installing => vec![F::ToggleLog],
            Screen::Failed => vec![F::Retry, F::SaveLog, F::ToggleLog, F::Back],
            Screen::Done => vec![F::Reboot, F::Stay],
        }
    }

    /// Where focus lands when a screen opens.
    fn initial_focus(&self) -> Option<F> {
        match self.screen {
            Screen::Account if !self.form.full_name.is_empty() => Some(F::Password),
            Screen::Git if !self.form.git_name.is_empty() && !self.form.git_email.is_empty() => Some(F::Next),
            Screen::Git if !self.form.git_name.is_empty() => Some(F::GitEmail),
            _ => self.screen_focus_order().first().copied(),
        }
    }

    /// Focuses where the current screen starts. With the power menu open, that's where focus goes
    /// back to when it closes.
    fn focus_screen(&mut self) {
        let f = self.initial_focus();
        match &mut self.power {
            Some(menu) => menu.return_focus = f,
            None => self.set_focus(f),
        }
    }

    pub fn is_focused(&self, f: F) -> bool {
        self.focus == Some(f)
    }

    /// Moves focus. A text field reached this way selects its whole value,
    /// so typing replaces it (the way Tab works everywhere else).
    fn set_focus(&mut self, f: Option<F>) {
        self.focus = f;
        self.focus_at = Instant::now();
        if let Some(f) = f.filter(|f| f.is_field()) {
            self.caret = Caret::all(self.field_value(f));
        }
        self.list = match f {
            Some(F::LocaleQ) => List::Locale,
            Some(F::KeymapQ | F::KeyboardTest) => List::Keymap,
            Some(F::TzQ) => List::Timezone,
            Some(F::WifiList) => List::Wifi,
            Some(F::DiskList) => List::Disk,
            _ => self.list,
        };
    }

    /// Why the current screen can't continue yet, if it can't.
    pub fn blocker(&self) -> Option<&'static str> {
        let probe = self.ready_probe();
        let r = match self.screen {
            Screen::Welcome => match &self.probe {
                ProbeState::Loading => Err("Poking at the hardware…"),
                ProbeState::Failed(_) => Err("Couldn't read this machine."),
                ProbeState::Ready(p) if p.payload.is_none() => Err("No system image on this stick."),
                ProbeState::Ready(_) => Ok(()),
            },
            Screen::Wifi => self.form.wifi_ok(self.wifi_secured(self.form.wifi_ssid.as_deref())),
            Screen::Account => self.form.account_ok(),
            Screen::Disk => probe.map_or(Err("No disks. None at all."), |p| self.form.disk_ok(p)),
            Screen::Git => self.form.git_ok(),
            Screen::Wallpapers => self.form.wallpapers_ok(self.wallpaper_sets()),
            _ => Ok(()),
        };
        r.err()
    }

    /// The look every view draws with: the mode, at the display's scale times the UI's.
    pub fn look(&self) -> Look {
        crate::ui::look(self.mode, self.system_scale * self.ui_scale)
    }

    /// How far the screen has arrived, 0 to 1: a page slides in over `long`.
    pub fn fade(&self) -> f32 {
        Motion::default().progress(
            tokens::Duration::Long,
            Easing::EmphasizedDecel,
            self.now - self.transition.0,
        )
    }

    pub fn animating(&self) -> bool {
        let t = self.now;
        let motion = Motion::default();
        t - self.transition.0 < motion.duration(tokens::Duration::Long)
            || self.hold.is_some()
            || t - self.log_open.1 < motion.duration(tokens::Duration::Medium)
            || self
                .switched
                .is_some_and(|(_, at)| t - at < motion.duration(tokens::Duration::Short))
            || self.nudged.is_some_and(|at| t - at < NUDGE)
            || self
                .wallpaper
                .as_ref()
                .is_some_and(|(_, at)| t - *at < motion.duration(tokens::Duration::Slow))
            || self.toast.is_some()
            || self.probe_rx.is_some()
            || self.run_rx.is_some()
            || self
                .install
                .as_ref()
                .is_some_and(|i| (i.shown - i.overall()).abs() > 0.0)
            || self.tour.is_some()
            || self.power.is_some_and(|m| m.stopping.is_some())
            || self.shutting_down.is_some()
            || self.glide.is_some()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            event::listen_with(on_event),
            window::resize_events().map(|(_, s)| Message::Resized(s)),
            Subscription::run_with(BatteryWatch(Arc::clone(&self.backend)), battery_watch).map(Message::Battery),
        ];
        if self.animating() {
            subs.push(window::frames().map(Message::Tick));
        }
        if self.pointer_idle {
            subs.push(event::listen_with(on_mouse));
        }
        Subscription::batch(subs)
    }

    pub(crate) fn go(&mut self, to: Screen) -> Task<Message> {
        if to == self.screen {
            return Task::none();
        }
        let forward = self.flow_index(to) >= self.flow_index(self.screen);
        self.screen = to;
        self.now = Instant::now();
        self.transition = (self.now, forward);
        self.hold = None;
        self.nudged = None;
        self.glide = None;
        self.reached = self.reached.max(self.flow_index(to));
        self.focus_screen();
        self.scroll_screen_to_selection()
    }

    pub fn next(&mut self) -> Task<Message> {
        if self.blocker().is_some() {
            self.now = Instant::now();
            self.nudged = Some(self.now);
            return Task::none();
        }
        let flow = self.flow();
        let i = self.flow_index(self.screen);
        match self.screen {
            Screen::Summary | Screen::Installing | Screen::Done | Screen::Failed => Task::none(),
            _ => self.go(flow[i + 1]),
        }
    }

    pub fn back(&mut self) -> Task<Message> {
        match self.screen {
            Screen::Welcome | Screen::Installing | Screen::Done => Task::none(),
            Screen::Failed => self.go(Screen::Summary),
            s => {
                let i = self.flow_index(s);
                self.go(self.flow()[i.saturating_sub(1)])
            }
        }
    }

    /// An install started and hasn't ended yet.
    pub fn install_running(&self) -> bool {
        self.install.as_ref().is_some_and(|i| i.outcome.is_none())
    }

    pub(crate) fn start_install(&mut self) -> Task<Message> {
        let Some(probe) = self.ready_probe() else {
            return Task::none();
        };
        if self.install_running() {
            return Task::none(); // one at a time
        }
        let config = self.form.to_config(probe);
        let secrets = self.form.to_secrets();
        let backend = Arc::clone(&self.backend);
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            backend.run(&config, &secrets, &mut |m| {
                let _ = tx.send(m);
            });
        });
        self.run_rx = Some(rx);
        self.install = Some(Install::new(Instant::now()));
        // The log starts closed, its close already over, so nothing animates.
        let closed_at = self
            .now
            .checked_sub(Motion::default().duration(tokens::Duration::Medium))
            .unwrap_or(self.now);
        self.log_open = (false, closed_at);
        self.go(Screen::Installing)
    }

    /// Decodes the wallpaper and glass right away (tests and the state
    /// screenshots, which have no event loop to do it in the background).
    #[cfg(test)]
    pub fn load_images_now(&mut self, size: Size) {
        let img = ::image::load_from_memory(theme::WALLPAPER)
            .expect("wallpaper")
            .to_rgba8();
        let h = image::Handle::from_rgba(img.width(), img.height(), img.into_raw());
        self.wallpaper = Some((
            h,
            self.epoch
                .checked_sub(Duration::from_secs(5))
                .expect("the clock is past 5 s"),
        ));
        self.blur_src = decode_blur();
        self.size = size;
        self.glass = None;
        self.ensure_glass();
    }

    /// Decodes the wallpaper thumbnails from `payload_dir` right away (the screenshots).
    #[cfg(test)]
    pub fn load_thumbs_now(&mut self) {
        let sets = self.wallpaper_sets().to_vec();
        for set in &sets {
            self.thumbs.extend(decoded_thumbs(&self.payload_dir, set));
        }
    }

    /// Pretends the probe already answered (tests and the screenshots).
    #[cfg(test)]
    pub fn set_probe(&mut self, p: Result<Probe, String>) {
        self.probe_rx = None;
        self.probe = match p {
            Ok(p) => {
                self.form = Form::new(&p);
                ProbeState::Ready(Box::new(p))
            }
            Err(e) => ProbeState::Failed(e),
        };
        self.focus_screen();
    }

    /// Glass under the panel: the blurred wallpaper cropped to the panel's
    /// rectangle. Rebuilt only when the window size changes.
    pub fn ensure_glass(&mut self) {
        if self.glass.as_ref().is_some_and(|(s, _)| *s == self.size) {
            return;
        }
        let Some(src) = &self.blur_src else { return };
        let panel = crate::view::panel_rect(self.size, self.look().scale);
        let (iw, ih) = (src.width() as f32, src.height() as f32);
        let zoom = (self.size.width / iw).max(self.size.height / ih);
        let ox = (iw * zoom - self.size.width) / 2.0;
        let oy = (ih * zoom - self.size.height) / 2.0;
        let x = ((panel.x + ox) / zoom).max(0.0) as u32;
        let y = ((panel.y + oy) / zoom).max(0.0) as u32;
        let w = ((panel.width / zoom) as u32).clamp(1, src.width() - x);
        let h = ((panel.height / zoom) as u32).clamp(1, src.height() - y);
        let crop = ::image::imageops::crop_imm(src, x, y, w, h).to_image();
        self.glass = Some((self.size, image::Handle::from_rgba(w, h, crop.into_raw())));
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        let task = self.update_inner(msg);
        if self.tour.is_some() {
            return Task::batch([task, crate::tour::drive(self)]);
        }
        task
    }

    #[expect(
        clippy::too_many_lines,
        reason = "One arm per message, each a few lines: the app's dispatch, read in one place."
    )]
    pub(crate) fn update_inner(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Tick(now) => self.tick(now),
            Message::Window(id) => {
                self.window = id;
                match id {
                    Some(id) => Task::batch([
                        window::size(id).map(Message::Resized),
                        window::scale_factor(id).map(Message::Rescaled),
                    ]),
                    None => Task::none(),
                }
            }
            Message::Resized(s) => {
                // `s` is in the current UI's units; back to the window's,
                // then pick the scale for that.
                let base = Size::new(s.width * self.ui_scale, s.height * self.ui_scale);
                self.ui_scale = self.fixed_scale.unwrap_or_else(|| auto_scale(base));
                self.size = Size::new(base.width / self.ui_scale, base.height / self.ui_scale);
                self.ensure_glass();
                Task::none()
            }
            Message::Rescaled(scale) => {
                self.system_scale = scale;
                Task::none()
            }
            Message::Wallpaper(h) => {
                self.wallpaper = Some((h, Instant::now()));
                self.now = Instant::now();
                Task::none()
            }
            Message::Thumbs(thumbs) => {
                self.thumbs.extend(thumbs);
                Task::none()
            }
            Message::Key(k) => {
                self.pointer_idle = true;
                self.key(k)
            }
            Message::PointerMoved => {
                self.pointer_idle = false;
                Task::none()
            }
            Message::Pasted(Some(s)) => {
                self.type_text(&s.replace(['\n', '\r'], ""));
                self.scroll_to_selection()
            }
            Message::Pasted(None) | Message::ShutDown(_, Ok(())) | Message::Setup(_) => Task::none(),
            Message::Click(f) if self.power.is_some() && !f.in_power_menu() => Task::none(),
            Message::Click(f) => {
                self.set_focus(Some(f));
                if f.is_field() || matches!(f, F::WifiList | F::DiskList | F::DiskMode) {
                    Task::none()
                } else {
                    self.activate(f)
                }
            }
            Message::FieldPointer(f, ptr) => {
                if self.focus != Some(f) {
                    self.set_focus(Some(f));
                }
                let value = self.field_value(f);
                self.caret = match ptr {
                    Pointer::Press { pos, extend: true } | Pointer::Drag(pos) => Caret {
                        pos,
                        anchor: self.caret.anchor,
                    },
                    Pointer::Press { pos, .. } => Caret::at(pos),
                    Pointer::Double(pos) => edit::word_at(value, pos),
                    Pointer::Triple => Caret::all(value),
                }
                .clamp(value);
                self.focus_at = Instant::now();
                Task::none()
            }
            Message::Scrolled(id, y, h) => {
                self.viewports.insert(id, (y, h));
                Task::none()
            }
            Message::Reveal(id, found) => {
                if let Some(to) = crate::reveal::target(found) {
                    self.now = Instant::now();
                    self.glide = Some(crate::reveal::Glide {
                        id,
                        from: found.offset,
                        to,
                        at: self.now,
                    });
                }
                Task::none()
            }
            Message::Pick(list, v) => {
                let f = match list {
                    List::Locale => F::LocaleQ,
                    List::Keymap => F::KeymapQ,
                    List::Timezone => F::TzQ,
                    List::Wifi => F::WifiList,
                    List::Disk => F::DiskList,
                };
                self.set_focus(Some(f));
                self.set_selected(list, v.clone());
                if list == List::Keymap {
                    self.toggle_keymap(&v);
                }
                Task::none()
            }
            Message::KeymapDefault(v) => {
                let r = self.form.make_default_keymap(&v);
                self.refused(r);
                Task::none()
            }
            Message::KeymapDrop(v) => {
                if self.form.keymap_picked(&v) {
                    self.toggle_keymap(&v);
                }
                Task::none()
            }
            Message::WifiNone => {
                self.set_focus(Some(F::WifiList));
                self.form.wifi_ssid = None;
                Task::none()
            }
            Message::DiskModePick(m) => {
                self.set_focus(Some(F::DiskMode));
                self.pick_mode(m);
                Task::none()
            }
            Message::HoldStart => {
                if self.screen == Screen::Summary && self.hold.is_none() {
                    self.set_focus(Some(F::Hold));
                    self.now = Instant::now();
                    self.hold = Some(self.now);
                }
                Task::none()
            }
            Message::ShutDown(action, Err(e)) => {
                self.shutting_down = None;
                if let Some(menu) = self.power.take() {
                    self.set_focus(menu.return_focus);
                }
                self.now = Instant::now();
                let msg = match action {
                    PowerAction::PowerOff => {
                        format!("Power off refused: {e}. Hold the power button, then.")
                    }
                    PowerAction::Restart => format!("Restart refused: {e}. Pull the plug, then."),
                };
                self.toast = Some((msg, self.now));
                Task::none()
            }
            Message::HoldEnd => {
                self.hold = None;
                Task::none()
            }
            Message::Battery(b) => {
                self.battery = b;
                Task::none()
            }
            Message::TourShot(name, shot) => {
                crate::tour::save(self, &name, &shot);
                if let Some(t) = &mut self.tour {
                    t.waiting_for_shot = false;
                }
                Task::none()
            }
        }
    }

    fn tick(&mut self, now: Instant) -> Task<Message> {
        let dt = now
            .saturating_duration_since(self.last_tick)
            // design-allow(duration): caps one frame's step; not a motion.
            .min(Duration::from_millis(100));
        self.last_tick = now;
        self.now = now;
        let mut task = Task::none();
        if let Some(rx) = &self.probe_rx
            && let Ok(r) = rx.try_recv()
        {
            self.probe_rx = None;
            self.probe = match r {
                Ok(p) => {
                    self.form = Form::new(&p);
                    ProbeState::Ready(Box::new(p))
                }
                Err(e) => ProbeState::Failed(e),
            };
            self.focus_screen();
            // Long before the Wallpapers screen opens.
            task = self.load_thumbs();
        }
        // Asking for fullscreen in the window settings makes winit request it
        // before the first frame, which kills cage 0.2.1. Ask once the window
        // is up instead (cage gives us the whole output anyway).
        if self.want_fullscreen
            && let Some(id) = self.window
        {
            self.want_fullscreen = false;
            task = window::set_mode(id, window::Mode::Fullscreen);
        }
        if let Some(rx) = &self.run_rx {
            let msgs: Vec<RunMsg> = rx.try_iter().collect();
            for m in msgs {
                task = Task::batch([task, self.on_run(m)]);
            }
        }
        if let Some(i) = &mut self.install {
            i.tick(dt, now);
        }
        if let Some(menu) = &mut self.power
            && let Some((action, since)) = menu.stopping
            && now.saturating_duration_since(since) >= STOP_GRACE
        {
            menu.stopping = None;
            task = Task::batch([task, self.shut_down(action)]);
        }
        if let Some(start) = self.hold
            && now - start >= HOLD
        {
            self.hold = None;
            return self.start_install();
        }
        if self
            .toast
            .as_ref()
            .is_some_and(|(_, at)| now - *at > toast::life(toast::DEFAULT_STAY))
        {
            self.toast = None;
        }
        if let Some(g) = self.glide {
            let (y, done) = g.at(now);
            if done {
                self.glide = None;
            }
            task = Task::batch([
                task,
                operation::scroll_to(g.id, operation::AbsoluteOffset { x: None, y: Some(y) }),
            ]);
        }
        task
    }

    fn on_run(&mut self, m: RunMsg) -> Task<Message> {
        let Some(install) = &mut self.install else {
            return Task::none();
        };
        let now = Instant::now();
        match m {
            RunMsg::Event(ev) => install.apply(&ev, now),
            RunMsg::Exited(r) => {
                install.exited(r);
                self.run_rx = None;
                let shown = if let Some(Outcome::Done { .. }) = install.outcome {
                    self.go(Screen::Done)
                } else {
                    self.log_open = (true, now);
                    self.go(Screen::Failed)
                };
                let Some(menu) = &mut self.power else {
                    return shown;
                };
                let Some((action, _)) = menu.stopping.take() else {
                    return shown;
                };
                return Task::batch([shown, self.shut_down(action)]);
            }
        }
        Task::none()
    }

    fn retry_probe(&mut self) {
        let (tx, rx) = mpsc::channel();
        let b = Arc::clone(&self.backend);
        std::thread::spawn(move || {
            let _ = tx.send(b.probe());
        });
        self.probe = ProbeState::Loading;
        self.probe_rx = Some(rx);
        self.set_focus(Some(F::Start));
    }

    /// Enter (or Space, or a click) on a button-like control.
    fn activate(&mut self, f: F) -> Task<Message> {
        match f {
            F::Start | F::Next => self.next(),
            F::Back => self.back(),
            F::RetryProbe => {
                self.retry_probe();
                Task::none()
            }
            F::ShowSecrets => {
                self.show_secrets = !self.show_secrets;
                Task::none()
            }
            F::Encrypt => {
                self.form.encrypt = !self.form.encrypt;
                self.flipped(f);
                Task::none()
            }
            F::LuksSame => {
                self.form.luks_same_as_password = !self.form.luks_same_as_password;
                self.flipped(f);
                Task::none()
            }
            F::WallpaperSet(s) => {
                if let Some(set) = self.wallpaper_sets().get(s).cloned() {
                    self.form.toggle_wallpaper_set(&set);
                }
                Task::none()
            }
            F::Wallpaper(s, i) => {
                if let Some(set) = self.wallpaper_sets().get(s).cloned()
                    && let Some(file) = set.files.get(i)
                {
                    self.form.toggle_wallpaper(&set, file);
                }
                Task::none()
            }
            F::Step(s) => {
                if self.install.as_ref().is_none_or(|i| i.outcome.is_some()) {
                    self.go(s)
                } else {
                    Task::none()
                }
            }
            F::ToggleLog => {
                let open = !self.log_open.0;
                self.now = Instant::now();
                self.log_open = (open, self.now);
                if open {
                    operation::snap_to_end(ids::LOG)
                } else {
                    Task::none()
                }
            }
            F::SaveLog => {
                let text = self.install.as_ref().map(Install::log_text).unwrap_or_default();
                self.now = Instant::now();
                self.toast = Some((
                    match save_log(&text) {
                        Ok(p) => format!("Log saved to {}.", p.display()),
                        Err(e) => format!("Couldn't save the log: {e}. It's also in /var/log/kubaos-install.log."),
                    },
                    self.now,
                ));
                Task::none()
            }
            F::Retry => self.start_install(),
            F::Reboot => self.shut_down(PowerAction::Restart),
            F::Stay => iced::exit(),
            F::Power => {
                self.open_power();
                Task::none()
            }
            F::PowerCancel => {
                self.close_power();
                Task::none()
            }
            F::PowerRestart => self.confirm_power(PowerAction::Restart),
            F::PowerOff => self.confirm_power(PowerAction::PowerOff),
            _ => Task::none(),
        }
    }

    fn open_power(&mut self) {
        if self.power.is_some() || self.shutting_down.is_some() {
            return;
        }
        self.hold = None;
        self.power = Some(PowerMenu {
            return_focus: self.focus,
            stopping: None,
        });
        // The safe answer first: Enter twice in a row never takes the machine down.
        self.set_focus(Some(F::PowerCancel));
    }

    /// Closes the power menu, unless it's already taking the machine down.
    fn close_power(&mut self) {
        let Some(menu) = self.power else {
            return;
        };
        if menu.stopping.is_some() || self.shutting_down.is_some() {
            return;
        }
        self.power = None;
        self.set_focus(menu.return_focus);
    }

    /// The menu's Power off or Restart: at once, or once the running install has stopped.
    fn confirm_power(&mut self, action: PowerAction) -> Task<Message> {
        if self.shutting_down.is_some() {
            return Task::none();
        }
        let running = self.install_running();
        let Some(menu) = &mut self.power else {
            return Task::none();
        };
        if menu.stopping.is_some() {
            return Task::none();
        }
        if !running {
            return self.shut_down(action);
        }
        self.now = Instant::now();
        menu.stopping = Some((action, self.now));
        self.backend.stop();
        Task::none()
    }

    /// Asks systemd to power off or restart, on a thread of its own. systemctl returns once
    /// systemd has the job and the machine takes a few seconds to go down, so the screen says so
    /// meanwhile.
    fn shut_down(&mut self, action: PowerAction) -> Task<Message> {
        if self.shutting_down.is_some() {
            return Task::none();
        }
        self.shutting_down = Some(action);
        self.now = Instant::now();
        let backend = Arc::clone(&self.backend);
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            let r = match action {
                PowerAction::PowerOff => backend.poweroff(),
                PowerAction::Restart => backend.reboot(),
            };
            let _ = tx.send(r);
        });
        Task::perform(
            async move { rx.await.unwrap_or_else(|_| Err("it crashed".into())) },
            move |r| Message::ShutDown(action, r),
        )
    }

    /// Keys while the power menu is open: it keeps them all, so nothing behind it moves.
    fn power_key(&mut self, k: &Key) -> Task<Message> {
        let busy = self.shutting_down.is_some() || self.power.is_some_and(|m| m.stopping.is_some());
        if busy {
            return Task::none();
        }
        match *k {
            Key::Esc => {
                self.close_power();
                Task::none()
            }
            Key::Tab { shift } => self.cycle_focus(shift),
            Key::Left { .. } => self.cycle_focus(true),
            Key::Right { .. } => self.cycle_focus(false),
            Key::Enter | Key::Space => match self.focus {
                Some(f) if f.in_power_menu() => self.activate(f),
                _ => Task::none(),
            },
            _ => Task::none(),
        }
    }

    /// Tab and Shift+Tab: the next or previous focus target, round the end.
    fn cycle_focus(&mut self, back: bool) -> Task<Message> {
        let order = self.focus_order();
        if order.is_empty() {
            return Task::none();
        }
        let n = order.len();
        let i = self.focus.and_then(|f| order.iter().position(|x| *x == f));
        let j = match (i, back) {
            (None, false) => 0,
            (None, true) => n - 1,
            (Some(i), false) => (i + 1) % n,
            (Some(i), true) => (i + n - 1) % n,
        };
        self.set_focus(Some(order[j]));
        self.reveal()
    }

    fn flipped(&mut self, f: F) {
        self.now = Instant::now();
        self.switched = Some((f, self.now));
    }

    /// How far the switch for `f` has slid to its current side, 0..=1.
    pub fn switch_progress(&self, f: F) -> f32 {
        match self.switched {
            Some((g, at)) if g == f => {
                Motion::default().progress(tokens::Duration::Short, Easing::Standard, self.now - at)
            }
            _ => 1.0,
        }
    }

    /// Applies an edit to the focused text field. False when no field has
    /// focus.
    fn edit(&mut self, e: Edit) -> bool {
        let Some(f) = self.focus.filter(|f| f.is_field()) else {
            return false;
        };
        let mut c = self.caret;
        let Some(s) = self.field_mut(f) else {
            return false;
        };
        let changed = edit::apply(s, &mut c, e);
        self.caret = c;
        self.focus_at = Instant::now();
        if changed {
            self.after_edit();
        }
        true
    }

    fn field_focused(&self) -> bool {
        self.focus.is_some_and(F::is_field)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "One arm per key, each a line or two: the keyboard's whole map, read in one place."
    )]
    pub fn key(&mut self, k: Key) -> Task<Message> {
        if self.power.is_some() {
            return self.power_key(&k);
        }
        let focus = self.focus;
        match k {
            Key::Tab { shift } => self.cycle_focus(shift),
            // A search typed: Enter picks the row it found, and clears it for the next one.
            Key::Enter if focus == Some(F::KeymapQ) && !self.keymap_q.trim().is_empty() => {
                let row = self.keymap_row();
                if !self.form.keymap_picked(&row) {
                    self.toggle_keymap(&row);
                }
                self.keymap_q.clear();
                self.caret = Caret::at(0);
                self.scroll_to_selection()
            }
            Key::Enter => match focus {
                Some(F::Hold) => self.update_inner(Message::HoldStart),
                _ if self.screen == Screen::Summary && !matches!(focus, Some(F::Back | F::Power)) => {
                    self.update_inner(Message::HoldStart)
                }
                Some(
                    f @ (F::Start
                    | F::RetryProbe
                    | F::Back
                    | F::Next
                    | F::ShowSecrets
                    | F::ToggleLog
                    | F::SaveLog
                    | F::Retry
                    | F::Reboot
                    | F::Stay
                    | F::Power),
                ) => self.activate(f),
                _ => self.next(),
            },
            Key::EnterUp => self.update_inner(Message::HoldEnd),
            Key::Esc => self.back(),
            Key::Space if self.keymap_list_keys() => {
                let row = self.keymap_row();
                self.toggle_keymap(&row);
                Task::none()
            }
            Key::ShiftSpace if self.keymap_list_keys() => {
                let row = self.keymap_row();
                let r = self.form.make_default_keymap(&row);
                self.refused(r);
                Task::none()
            }
            Key::ShiftSpace => self.key(Key::Space),
            Key::Space => match focus {
                Some(f) if f.is_field() => {
                    self.edit(Edit::Insert(" ".into()));
                    self.scroll_to_selection()
                }
                Some(F::DiskMode) => {
                    self.toggle_mode();
                    Task::none()
                }
                Some(F::Hold) => self.update_inner(Message::HoldStart),
                Some(F::WifiList | F::DiskList) | None => Task::none(),
                Some(f) => self.activate(f),
            },
            Key::Up | Key::Down | Key::PageUp | Key::PageDown => {
                let d = match k {
                    Key::Up => -1,
                    Key::Down => 1,
                    Key::PageUp => -PAGE,
                    _ => PAGE,
                };
                match focus {
                    Some(F::DiskMode) => {
                        self.toggle_mode();
                        Task::none()
                    }
                    Some(f @ (F::WallpaperSet(_) | F::Wallpaper(..))) if matches!(k, Key::Up | Key::Down) => {
                        self.move_in_wallpapers(f, if d < 0 { Dir::Up } else { Dir::Down })
                    }
                    Some(f) if f.lists() => self.move_selection(d),
                    _ => Task::none(),
                }
            }
            Key::Left { word, shift } | Key::Right { word, shift } => {
                let left = matches!(k, Key::Left { .. });
                if self.field_focused() {
                    self.edit(if left {
                        Edit::Left { word, select: shift }
                    } else {
                        Edit::Right { word, select: shift }
                    });
                } else if focus == Some(F::DiskMode) {
                    self.pick_mode(if left { DiskMode::FreeSpace } else { DiskMode::Whole });
                } else if let Some(f @ (F::WallpaperSet(_) | F::Wallpaper(..))) = focus {
                    return self.move_in_wallpapers(f, if left { Dir::Left } else { Dir::Right });
                }
                Task::none()
            }
            Key::Home { shift } | Key::End { shift } => {
                let home = matches!(k, Key::Home { .. });
                if self.field_focused() {
                    self.edit(if home {
                        Edit::Home { select: shift }
                    } else {
                        Edit::End { select: shift }
                    });
                    Task::none()
                } else if focus.is_some_and(F::lists) {
                    self.move_selection(if home { -100_000 } else { 100_000 })
                } else {
                    Task::none()
                }
            }
            Key::Backspace { word } => {
                self.edit(Edit::Backspace { word });
                self.scroll_to_selection()
            }
            Key::Delete { word } => {
                self.edit(Edit::Delete { word });
                self.scroll_to_selection()
            }
            Key::Clear => {
                self.edit(Edit::Clear);
                self.scroll_to_selection()
            }
            Key::SelectAll => {
                self.edit(Edit::SelectAll);
                Task::none()
            }
            Key::Copy | Key::Cut => {
                // Secrets stay where they are.
                let Some(f) = focus.filter(|f| f.is_field() && !f.is_secret()) else {
                    return Task::none();
                };
                let sel = edit::selected(self.field_value(f), self.caret).to_string();
                if sel.is_empty() {
                    return Task::none();
                }
                if k == Key::Cut {
                    self.edit(Edit::DeleteSelection);
                }
                iced::clipboard::write(sel)
            }
            Key::Paste => {
                if self.field_focused() {
                    iced::clipboard::read().map(Message::Pasted)
                } else {
                    Task::none()
                }
            }
            Key::Text(t) => {
                // Typing on a list screen goes to its search box, at the end
                // of what's already there.
                if !self.field_focused() {
                    let search = match self.screen {
                        Screen::Language if self.list == List::Keymap => Some(F::KeymapQ),
                        Screen::Language => Some(F::LocaleQ),
                        Screen::Timezone => Some(F::TzQ),
                        _ => None,
                    };
                    match search {
                        Some(f) => {
                            self.set_focus(Some(f));
                            self.caret = Caret::at(self.field_value(f).chars().count());
                        }
                        // Where no field takes the letters, h j k l move as the arrows do.
                        None => return vim_arrow(&t).map_or_else(Task::none, |k| self.key(k)),
                    }
                }
                self.type_text(&t);
                self.scroll_to_selection()
            }
        }
    }

    fn type_text(&mut self, t: &str) {
        self.edit(Edit::Insert(t.to_string()));
    }

    fn field_mut(&mut self, f: F) -> Option<&mut String> {
        let form = &mut self.form;
        Some(match f {
            F::LocaleQ => &mut self.locale_q,
            F::KeymapQ => &mut self.keymap_q,
            F::KeyboardTest => &mut form.keyboard_test,
            F::TzQ => &mut self.tz_q,
            F::WifiPsk => &mut form.wifi_psk,
            F::FullName => &mut form.full_name,
            F::Username => &mut form.username,
            F::Hostname => &mut form.hostname,
            F::Password => &mut form.password,
            F::PasswordConfirm => &mut form.password_confirm,
            F::Luks => &mut form.luks,
            F::LuksConfirm => &mut form.luks_confirm,
            F::GitName => &mut form.git_name,
            F::GitEmail => &mut form.git_email,
            _ => return None,
        })
    }

    pub fn field_value(&self, f: F) -> &str {
        let form = &self.form;
        match f {
            F::LocaleQ => &self.locale_q,
            F::KeymapQ => &self.keymap_q,
            F::KeyboardTest => &form.keyboard_test,
            F::TzQ => &self.tz_q,
            F::WifiPsk => &form.wifi_psk,
            F::FullName => &form.full_name,
            F::Username => &form.username,
            F::Hostname => &form.hostname,
            F::Password => &form.password,
            F::PasswordConfirm => &form.password_confirm,
            F::Luks => &form.luks,
            F::LuksConfirm => &form.luks_confirm,
            F::GitName => &form.git_name,
            F::GitEmail => &form.git_email,
            _ => "",
        }
    }

    /// Side effects of editing the focused field.
    fn after_edit(&mut self) {
        self.focus_at = Instant::now();
        match self.focus {
            Some(F::LocaleQ) => self.pick_first_match(List::Locale),
            Some(F::KeymapQ) => self.pick_first_match(List::Keymap),
            Some(F::TzQ) => self.pick_first_match(List::Timezone),
            Some(F::FullName) if !self.form.git_name_touched => {
                self.form.git_name = self.form.full_name.trim().to_string();
            }
            Some(F::GitName) => self.form.git_name_touched = true,
            _ => {}
        }
    }

    fn pick_mode(&mut self, m: DiskMode) {
        let ok = match m {
            DiskMode::Whole => true,
            DiskMode::FreeSpace => self
                .ready_probe()
                .and_then(|p| self.form.selected_disk(p).map(|d| d.fits_dual_boot(p)))
                .unwrap_or(false),
        };
        if ok {
            self.form.disk_mode = Some(m);
        }
    }

    fn toggle_mode(&mut self) {
        let m = match self.form.disk_mode {
            Some(DiskMode::Whole) => DiskMode::FreeSpace,
            _ => DiskMode::Whole,
        };
        self.pick_mode(m);
    }

    pub fn wifi_secured(&self, ssid: Option<&str>) -> bool {
        let (Some(p), Some(ssid)) = (self.ready_probe(), ssid) else {
            return false;
        };
        p.wifi.iter().any(|w| w.ssid == ssid && w.secured)
    }

    /// The entries a list shows, after filtering by its search box.
    pub fn list_items(&self, list: List) -> Vec<String> {
        let Some(p) = self.ready_probe() else {
            return vec![];
        };
        let (all, q): (Vec<String>, &str) = match list {
            List::Locale => (p.locales.clone(), &self.locale_q),
            // "custom" is xkb's slot for a hand-written layout: not one to pick.
            List::Keymap => (
                p.keymaps.iter().filter(|k| k.as_str() != "custom").cloned().collect(),
                &self.keymap_q,
            ),
            List::Timezone => (p.timezones.clone(), &self.tz_q),
            List::Wifi => (p.wifi.iter().map(|w| w.ssid.clone()).collect(), ""),
            List::Disk => (
                p.disks
                    .iter()
                    .filter(|d| d.is_target())
                    .map(|d| d.path.clone())
                    .collect(),
                "",
            ),
        };
        let name = |s: &str| match list {
            List::Locale => crate::data::locale_name(s),
            List::Keymap => crate::data::keymap_name(s),
            _ => "".into(),
        };
        // Languages and layouts read in name order, not code order.
        let mut all = all;
        if matches!(list, List::Locale | List::Keymap) {
            all.sort_by_cached_key(|s| name(s).to_lowercase());
        }
        let q = q.trim().to_lowercase();
        if q.is_empty() {
            return all;
        }
        let mut hits: Vec<(u8, String)> = all
            .into_iter()
            .filter_map(|s| {
                let key = s.to_lowercase();
                let disp = name(&s).to_lowercase();
                let leaf = key.rsplit('/').next().unwrap_or(&key).replace('_', " ");
                let rank = if leaf.starts_with(&q) || key.starts_with(&q) || disp.starts_with(&q) {
                    0
                } else if key.contains(&q) || disp.contains(&q) || leaf.contains(&q) {
                    1
                } else {
                    return None;
                };
                Some((rank, s))
            })
            .collect();
        hits.sort_by_key(|(r, _)| *r);
        hits.into_iter().map(|(_, s)| s).collect()
    }

    fn selected(&self, list: List) -> Option<String> {
        match list {
            List::Locale => Some(self.form.locale.clone()),
            List::Keymap => Some(self.keymap_row()),
            List::Timezone => Some(self.form.timezone.clone()),
            List::Wifi => self.form.wifi_ssid.clone(),
            List::Disk => self.form.disk.clone(),
        }
    }

    fn set_selected(&mut self, list: List, v: String) {
        match list {
            List::Locale => self.form.locale = v,
            List::Keymap => self.keymap_cursor = Some(v),
            List::Timezone => self.form.timezone = v,
            List::Wifi => {
                self.form.wifi_ssid = Some(v);
                self.form.wifi_psk.clear();
            }
            List::Disk => {
                self.form.disk = Some(v);
                if let Some(p) = self.ready_probe() {
                    let mode = self.form.selected_disk(p).map(|d| d.default_mode(p));
                    self.form.disk_mode = mode;
                }
            }
        }
    }

    /// The keymap row Space acts on: the cursor's, else the default layout's.
    pub fn keymap_row(&self) -> String {
        self.keymap_cursor
            .clone()
            .or_else(|| self.form.keymaps.first().cloned())
            .unwrap_or_default()
    }

    /// Space and Shift+Space act on the keymap list while its search box has the focus with
    /// nothing typed: a search never starts with a space.
    fn keymap_list_keys(&self) -> bool {
        self.focus == Some(F::KeymapQ) && self.keymap_q.is_empty()
    }

    fn toggle_keymap(&mut self, keymap: &str) {
        let r = self.form.toggle_keymap(keymap);
        self.refused(r);
    }

    /// Shows why a change to the form was refused.
    fn refused(&mut self, r: Result<(), &'static str>) {
        if let Err(why) = r {
            self.now = Instant::now();
            self.toast = Some((why.to_string(), self.now));
        }
    }

    fn pick_first_match(&mut self, list: List) {
        let items = self.list_items(list);
        let cur = self.selected(list);
        if !items.iter().any(|i| Some(i) == cur.as_ref())
            && let Some(first) = items.into_iter().next()
        {
            self.set_selected(list, first);
        }
    }

    fn move_selection(&mut self, d: i32) -> Task<Message> {
        let list = self.list;
        // Wi-Fi has a "Don't connect" row on top, standing for None.
        let mut items: Vec<Option<String>> = self.list_items(list).into_iter().map(Some).collect();
        if list == List::Wifi {
            items.insert(0, None);
        }
        if items.is_empty() {
            return Task::none();
        }
        let cur = self.selected(list);
        let i = items.iter().position(|x| *x == cur);
        let j = match i {
            None => 0,
            Some(i) => (i as i32 + d).clamp(0, items.len() as i32 - 1) as usize,
        };
        match items[j].clone() {
            Some(v) => self.set_selected(list, v),
            None => self.form.wifi_ssid = None,
        }
        self.scroll_to_selection()
    }

    /// Long screens: glide just far enough to show the focused control
    /// (src/reveal.rs). The page title sits outside the scrollable, so it
    /// never goes anywhere.
    fn reveal(&self) -> Task<Message> {
        let target = self.focus.and_then(|f| match wallpaper_item(f) {
            Some(item) => Some((ids::WALLPAPER_SCROLL, iced::widget::Id::from(item))),
            None => reveal_target(self.screen, f).map(|(scroll, item)| (scroll, iced::widget::Id::new(item))),
        });
        let Some((scroll, item)) = target else {
            return Task::none();
        };
        iced::advanced::widget::operate(crate::reveal::Find::new(scroll, item))
            .map(move |found| Message::Reveal(scroll, found))
    }

    /// The arrow keys on the Wallpapers screen: through a set's grid of thumbnails, from its
    /// header into it, and past its first or last row to the set above or below.
    fn move_in_wallpapers(&mut self, from: F, dir: Dir) -> Task<Message> {
        let Some(to) = self.wallpaper_neighbour(from, dir) else {
            return Task::none();
        };
        self.set_focus(Some(to));
        self.reveal()
    }

    /// Where an arrow key goes from a header or a thumbnail; `None` at the edge.
    pub(crate) fn wallpaper_neighbour(&self, from: F, dir: Dir) -> Option<F> {
        let sets = self.wallpaper_sets();
        let files = |s: usize| sets.get(s).map_or(0, |set| set.files.len());
        let cols = WALLPAPER_COLUMNS;
        // The set above's last row, at its first thumbnail; its header when it has none.
        let above = |s: usize| -> Option<F> {
            let s = s.checked_sub(1)?;
            Some(match files(s) {
                0 => F::WallpaperSet(s),
                n => F::Wallpaper(s, (n - 1) / cols * cols),
            })
        };
        let below = |s: usize| (s + 1 < sets.len()).then_some(F::WallpaperSet(s + 1));
        match (from, dir) {
            (F::WallpaperSet(s), Dir::Up) => above(s),
            (F::WallpaperSet(s), Dir::Down) => match files(s) {
                0 => below(s),
                _ => Some(F::Wallpaper(s, 0)),
            },
            (F::Wallpaper(s, i), Dir::Left) => i.checked_sub(1).map(|j| F::Wallpaper(s, j)),
            (F::Wallpaper(s, i), Dir::Right) => (i + 1 < files(s)).then_some(F::Wallpaper(s, i + 1)),
            (F::Wallpaper(s, i), Dir::Up) => Some(match i.checked_sub(cols) {
                Some(j) => F::Wallpaper(s, j),
                None => F::WallpaperSet(s),
            }),
            (F::Wallpaper(s, i), Dir::Down) => {
                let n = files(s);
                if i + cols < n {
                    Some(F::Wallpaper(s, i + cols))
                } else if i / cols < (n - 1) / cols {
                    // The row below is short: its last thumbnail.
                    Some(F::Wallpaper(s, n - 1))
                } else {
                    below(s)
                }
            }
            _ => None,
        }
    }

    /// Decodes every set's thumbnails, a thread per set, so the screen fills in as they come.
    fn load_thumbs(&self) -> Task<Message> {
        let dir = self.payload_dir.clone();
        Task::batch(
            self.wallpaper_sets()
                .iter()
                .cloned()
                .map(|set| Task::perform(load_thumbs(dir.clone(), set), Message::Thumbs)),
        )
    }

    /// Keeps the selected row of the active list in view, scrolling as
    /// little as possible (like any list does).
    fn scroll_to_selection(&self) -> Task<Message> {
        scroll_lists(self.selection_scroll(self.list))
    }

    /// Scrolls every list on the screen so its selected row shows. A list
    /// opens with a preselection (the probe's keymap, the time zone), and
    /// only one of a screen's lists has the focus.
    fn scroll_screen_to_selection(&self) -> Task<Message> {
        scroll_lists(self.selection_scrolls())
    }

    /// The scrollables on this screen that must move to show their selected
    /// row, with the offset each moves to.
    pub(crate) fn selection_scrolls(&self) -> Vec<(&'static str, f32)> {
        let lists: &[List] = match self.screen {
            Screen::Language => &[List::Locale, List::Keymap],
            Screen::Timezone => &[List::Timezone],
            Screen::Wifi => &[List::Wifi],
            _ => &[],
        };
        lists.iter().filter_map(|&l| self.selection_scroll(l)).collect()
    }

    /// Where `list`'s scrollable moves to show its selected row: `None` when
    /// the row already shows, or the list has no selection or no scrollable.
    fn selection_scroll(&self, list: List) -> Option<(&'static str, f32)> {
        let id = match list {
            List::Locale => ids::LOCALE_LIST,
            List::Keymap => ids::KEYMAP_LIST,
            List::Timezone => ids::TZ_LIST,
            List::Wifi => ids::WIFI_LIST,
            List::Disk => return None,
        };
        let items = self.list_items(list);
        let cur = self.selected(list);
        let i = match (list, &cur) {
            // Wi-Fi's first row is "Don't connect".
            (List::Wifi, None) => 0,
            (List::Wifi, Some(c)) => items.iter().position(|x| x == c)? + 1,
            (_, cur) => items.iter().position(|x| Some(x) == cur.as_ref())?,
        };
        // Rows are a row tall and a gap apart, after the well's padding, which scrolls with them.
        let height = f32::from(tokens::Size::Row.px());
        let row = height + f32::from(tokens::Space::S4.px());
        let top = f32::from(tokens::Space::S12.px()) + i as f32 * row;
        let bottom = top + height;
        let y = match self.viewports.get(id) {
            Some(&(off, h)) if h > row => {
                if top < off {
                    top
                } else if bottom > off + h {
                    bottom - h
                } else {
                    return None;
                }
            }
            // Not drawn yet: two rows of context above.
            _ => top - 2.0 * row,
        };
        Some((id, y.max(0.0)))
    }
}

/// The scrollable and the container that hold a control on a long screen.
/// The ids are on the containers in view.rs.
pub fn reveal_target(screen: Screen, f: F) -> Option<(&'static str, &'static str)> {
    let item = match (screen, f) {
        (Screen::Disk, F::DiskList) => ids::REVEAL_DISKS,
        (Screen::Disk, F::DiskMode) => ids::REVEAL_MODE,
        (Screen::Disk, F::Encrypt) => ids::REVEAL_ENCRYPT,
        (Screen::Disk, F::LuksSame) => ids::REVEAL_LUKS_SAME,
        (Screen::Disk, F::Luks | F::LuksConfirm | F::ShowSecrets) => ids::REVEAL_LUKS,
        (Screen::Account, F::FullName | F::Username) => ids::REVEAL_NAMES,
        (Screen::Account, F::Hostname) => ids::REVEAL_HOST,
        (Screen::Account, F::Password | F::PasswordConfirm | F::ShowSecrets) => ids::REVEAL_PASSWORDS,
        _ => return None,
    };
    let scroll = match screen {
        Screen::Disk => ids::DISK_SCROLL,
        _ => ids::ACCOUNT_SCROLL,
    };
    Some((scroll, item))
}

/// Rows Page Up and Page Down move in a list.
const PAGE: i32 = 8;

/// Thumbnails in a row on the Wallpapers screen (`view.rs`, `wallpapers`).
pub const WALLPAPER_COLUMNS: usize = 4;

/// An arrow key's direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

/// The container id of a header or thumbnail on the Wallpapers screen, which `reveal` scrolls to.
pub fn wallpaper_item(f: F) -> Option<String> {
    match f {
        F::WallpaperSet(s) => Some(format!("wallpaper-set-{s}")),
        F::Wallpaper(s, i) => Some(format!("wallpaper-{s}-{i}")),
        _ => None,
    }
}

impl F {
    /// Focus targets the arrow keys move a list selection for.
    pub fn lists(self) -> bool {
        matches!(
            self,
            F::LocaleQ | F::KeymapQ | F::KeyboardTest | F::TzQ | F::WifiList | F::DiskList
        )
    }
}

/// The UI is laid out for about 1280x800. cage gives us the whole output
/// at scale 1: 1920x1080 in the VM, 2880x1800 on the TUXEDO, where 14 px
/// text would be tiny. So scale up to fill the window, in quarter steps,
/// never below 1 (1024x768 stays 1).
/// The backend whose battery `battery_watch` reads. There is one backend per run, so its address
/// is the subscription's identity.
struct BatteryWatch(Arc<dyn Backend>);

impl Hash for BatteryWatch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).cast::<()>().hash(state);
    }
}

/// Reads the battery at once, then every `battery::POLL`, on a thread of its own. The thread ends
/// at the first reading after the subscription is dropped.
fn battery_watch(watch: &BatteryWatch) -> iced::futures::channel::mpsc::Receiver<Option<Battery>> {
    let (mut tx, rx) = iced::futures::channel::mpsc::channel(1);
    let backend = Arc::clone(&watch.0);
    thread::spawn(move || {
        loop {
            // A full channel means the app hasn't taken the last reading yet: this one is dropped
            // and the next comes a poll later.
            if let Err(e) = tx.try_send(backend.battery())
                && e.is_disconnected()
            {
                return;
            }
            thread::sleep(battery::POLL);
        }
    });
    rx
}

/// The UI scale for a window `window` big: the largest supported scale at which the window still
/// holds the 1280 × 800 the screens are laid out for, so every length lands on device pixels.
pub fn auto_scale(window: Size) -> f32 {
    let fits = (window.width / 1280.0).min(window.height / 800.0);
    Scale::SUPPORTED
        .iter()
        .rev()
        .map(|s| s.factor())
        .find(|&factor| factor <= fits)
        .unwrap_or(1.0)
}

/// h j k l as the arrows, for a list or grid with no field taking the keys.
fn vim_arrow(text: &str) -> Option<Key> {
    match text {
        "h" => Some(Key::Left {
            word: false,
            shift: false,
        }),
        "j" => Some(Key::Down),
        "k" => Some(Key::Up),
        "l" => Some(Key::Right {
            word: false,
            shift: false,
        }),
        _ => None,
    }
}

/// Moves each scrollable to its vertical offset.
fn scroll_lists(scrolls: impl IntoIterator<Item = (&'static str, f32)>) -> Task<Message> {
    Task::batch(
        scrolls
            .into_iter()
            .map(|(id, y)| operation::scroll_to(id, operation::AbsoluteOffset { x: None, y: Some(y) })),
    )
}

/// iced keyboard event -> our `Key`. The window subscription and the key
/// tour both go through here.
pub fn on_event(ev: iced::Event, _status: event::Status, _w: window::Id) -> Option<Message> {
    // The window moved to a display of another scale: the views snap to its device pixels.
    if let iced::Event::Window(window::Event::Rescaled(scale)) = ev {
        return Some(Message::Rescaled(scale));
    }
    let iced::Event::Keyboard(k) = ev else {
        return None;
    };
    translate(&k).map(Message::Key)
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "`event::listen_with` hands its function the event by value"
)]
pub fn on_mouse(ev: iced::Event, _status: event::Status, _w: window::Id) -> Option<Message> {
    match ev {
        iced::Event::Mouse(
            iced::mouse::Event::CursorMoved { .. }
            | iced::mouse::Event::ButtonPressed(_)
            | iced::mouse::Event::WheelScrolled { .. },
        ) => Some(Message::PointerMoved),
        _ => None,
    }
}

pub fn translate(k: &keyboard::Event) -> Option<Key> {
    match k {
        keyboard::Event::KeyPressed {
            key,
            modifiers,
            repeat,
            text,
            ..
        } => {
            // Shortcuts: Ctrl on Linux, Cmd on a Mac (both work everywhere).
            let ctrl = modifiers.control() || modifiers.logo();
            // Word jumps: Ctrl on Linux, Alt (Option) on a Mac.
            let word = modifiers.control() || modifiers.alt();
            let shift = modifiers.shift();
            match key.as_ref() {
                keyboard::Key::Named(n) => match n {
                    Named::Enter => (!repeat).then_some(Key::Enter),
                    Named::Escape => Some(Key::Esc),
                    Named::Tab => Some(Key::Tab { shift }),
                    Named::ArrowUp => Some(Key::Up),
                    Named::ArrowDown => Some(Key::Down),
                    // Cmd+arrows are Home and End on a Mac.
                    Named::ArrowLeft if modifiers.logo() => Some(Key::Home { shift }),
                    Named::ArrowRight if modifiers.logo() => Some(Key::End { shift }),
                    Named::ArrowLeft => Some(Key::Left { word, shift }),
                    Named::ArrowRight => Some(Key::Right { word, shift }),
                    Named::Home => Some(Key::Home { shift }),
                    Named::End => Some(Key::End { shift }),
                    Named::PageUp => Some(Key::PageUp),
                    Named::PageDown => Some(Key::PageDown),
                    Named::Space if shift => Some(Key::ShiftSpace),
                    Named::Space => Some(Key::Space),
                    Named::Backspace => Some(Key::Backspace { word }),
                    Named::Delete => Some(Key::Delete { word }),
                    Named::Insert if shift => Some(Key::Paste),
                    _ => None,
                },
                keyboard::Key::Character(c) if ctrl => match c {
                    "n" | "N" | "j" | "J" => Some(Key::Down),
                    "p" | "P" | "k" | "K" => Some(Key::Up),
                    "a" | "A" => Some(Key::SelectAll),
                    "c" | "C" => Some(Key::Copy),
                    "x" | "X" => Some(Key::Cut),
                    "v" | "V" => Some(Key::Paste),
                    "u" | "U" => Some(Key::Clear),
                    _ => None,
                },
                _ => text
                    .as_ref()
                    .map(ToString::to_string)
                    .filter(|t| !t.is_empty() && !t.chars().any(char::is_control))
                    .map(Key::Text),
            }
        }
        keyboard::Event::KeyReleased {
            key: keyboard::Key::Named(Named::Enter),
            ..
        } => Some(Key::EnterUp),
        _ => None,
    }
}

async fn decode_wallpaper() -> image::Handle {
    let (tx, rx) = iced::futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let h = match ::image::load_from_memory(theme::WALLPAPER) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                image::Handle::from_rgba(rgba.width(), rgba.height(), rgba.into_raw())
            }
            Err(_) => image::Handle::from_bytes(theme::WALLPAPER),
        };
        let _ = tx.send(h);
    });
    rx.await.unwrap_or_else(|_| image::Handle::from_bytes(theme::WALLPAPER))
}

/// A set's thumbnails, decoded on a thread of their own (`thumbs::decode`), so a few dozen of them
/// never hold up the UI thread's frames.
async fn load_thumbs(payload: PathBuf, set: kubaos_core::probe::WallpaperSet) -> Vec<(String, image::Handle)> {
    let (tx, rx) = iced::futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let _ = tx.send(decoded_thumbs(&payload, &set));
    });
    rx.await.unwrap_or_default()
}

fn decoded_thumbs(payload: &std::path::Path, set: &kubaos_core::probe::WallpaperSet) -> Vec<(String, image::Handle)> {
    crate::thumbs::decode(payload, std::path::Path::new(crate::thumbs::ORIGINALS), set)
        .into_iter()
        .map(|(key, img)| {
            // Every card's picture is the same shape, so the cards are all one height.
            let img = crate::thumbs::cropped(&img);
            let (w, h) = img.dimensions();
            (key, image::Handle::from_rgba(w, h, img.into_raw()))
        })
        .collect()
}

fn decode_blur() -> Option<::image::RgbaImage> {
    ::image::load_from_memory(theme::WALLPAPER_BLUR)
        .ok()
        .map(|i| i.to_rgba8())
}

/// Next to the user's home on the live system, else the temp dir.
fn save_log(text: &str) -> std::io::Result<PathBuf> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    let dir = std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .unwrap_or_else(std::env::temp_dir);
    let path = dir.join(format!("kubaos-install-{stamp}.log"));
    std::fs::write(&path, text)?;
    Ok(path)
}

#[cfg(test)]
mod tests;
