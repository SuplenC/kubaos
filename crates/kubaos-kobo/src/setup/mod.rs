//! `kobo --setup`: first-boot setup on the installed system
//! (docs/INSTALL-INTERNALS.md, "First boot"). Its own iced application, sharing the
//! install screens' theme, widgets, key translation and message type.
//!
//! A checklist: the cards on the left, the selected one on the right. Each
//! card is detected fresh on every start (`kubaos_core::setup`), so
//! re-running never redoes a finished one. Keyboard first, like the
//! install screens: Up/Down pick a card, Enter or Tab go into it, Esc comes back,
//! Space toggles, Left/Right switch choices.
//!
//! The phone card checks again on its own while it waits for a phone, since the
//! pairing request arrives from the phone, not from a key pressed here.

pub mod backend;
pub mod tour;
mod view;

use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use iced::{Size, Subscription, Task, event, window};
use kubaos_core::Level;
use kubaos_core::setup::{
    BwLogin, BwMethod, CardId, CardStatus, Facts, GitIdentity, Inputs, MokState, PhoneStep,
    PromptKind, SetupEvent, State, Status, YKMAN_PREP, git_identity_ok, mok_password_ok,
    next_finger,
};

pub use view::view;

use crate::app::{Key, Message};
use crate::edit::{self, Caret, Edit};
use crate::field::Pointer;
use crate::theme::motion;
use backend::SetupBackend;

/// What the vault's project env folders start with. The folder list is drawn every frame, so the
/// environment is read once.
static ENV_PREFIX: LazyLock<String> = LazyLock::new(|| kubaos_core::bitwarden::Names::from_env().env_prefix);

/// How often the phone card checks again while it waits: a pairing request shows within this.
const PHONE_POLL: Duration = Duration::from_secs(2);

/// How long the phone card waits for a phone before it explains adding the laptop by address.
/// The laptop shows up in the phone's app within seconds on a home network; one that blocks
/// discovery never shows it.
const BY_ADDRESS_AFTER: Duration = Duration::from_secs(30);

/// Everything that can have focus on the setup screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SF {
    Cards,
    Mok,
    MokAgain,
    Server,
    Url,
    Email,
    Password,
    Method,
    Code,
    ClientId,
    ClientSecret,
    /// The vault's folders, one ticked or not under a cursor.
    Folders,
    LogOut,
    /// The phone card's way out: the link off, KDE Connect stopped.
    TurnOff,
    GitName,
    GitEmail,
    ShowSecrets,
    Primary,
    Secondary,
    Skip,
    Log,
    RebootAll,
    Finish,
}

impl SF {
    pub fn is_field(self) -> bool {
        matches!(
            self,
            SF::Mok
                | SF::MokAgain
                | SF::Url
                | SF::Email
                | SF::Password
                | SF::Code
                | SF::ClientId
                | SF::ClientSecret
                | SF::GitName
                | SF::GitEmail
        )
    }

    pub fn is_secret(self) -> bool {
        matches!(
            self,
            SF::Mok | SF::MokAgain | SF::Password | SF::ClientSecret
        )
    }
}

#[derive(Debug, Clone)]
pub enum Msg {
    Click(SF),
    /// A click or drag in a text field.
    Pointer(SF, Pointer),
    Pick(CardId),
    /// A click on one of the vault's folders.
    Folder(String),
    CloseRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Server {
    Com,
    #[default]
    Eu,
    Custom,
}

impl Server {
    pub fn label(self) -> &'static str {
        match self {
            Server::Com => "bitwarden.com",
            Server::Eu => "bitwarden.eu",
            Server::Custom => "Self-hosted",
        }
    }
}

pub fn method_label(m: BwMethod) -> &'static str {
    match m {
        BwMethod::Totp => "Authenticator app",
        BwMethod::YubikeyOtp => "YubiKey OTP",
        BwMethod::ApiKey => "API key",
    }
}

/// What's typed into the cards. Secrets are cleared once a run succeeds.
#[derive(Default)]
pub struct Form {
    pub mok: String,
    pub mok_again: String,
    pub server: Server,
    pub url: String,
    pub email: String,
    pub password: String,
    pub method: BwMethod,
    pub code: String,
    pub client_id: String,
    pub client_secret: String,
    /// The folders SSH keys come from; none for the whole vault.
    pub folders: Vec<String>,
    /// Which of the vault's folders the cursor is on.
    pub folder_at: usize,
    pub git_name: String,
    pub git_email: String,
    prefilled: bool,
}

/// What a card's buttons do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    Run,
    Reboot,
    Terminal,
    Recheck,
    LogOut,
    /// Accept the pairing request the phone card shows.
    Accept,
    /// The test copy to the phone.
    Test,
    /// What the phone sent back.
    Check,
    TurnOff,
}

#[derive(Debug, Clone)]
pub enum Outcome {
    Done(String),
    Failed { error: String, hint: Option<String> },
}

/// What the person is asked to do: kind, words, (done, of) touches.
pub type Prompt = (PromptKind, String, Option<(u32, u32)>);

/// One card's action, running or finished.
pub struct Run {
    pub card: CardId,
    pub prompt: Option<Prompt>,
    pub prompt_at: Instant,
    pub log: Vec<(Level, String)>,
    pub outcome: Option<Outcome>,
    /// A run that leaves its card to do on purpose (a log out, the phone link off), with what
    /// to say once it's done.
    pub undo: Option<&'static str>,
}

impl Run {
    pub fn running(&self) -> bool {
        self.outcome.is_none()
    }
}

pub struct Setup {
    pub backend: Arc<dyn SetupBackend>,
    pub status: Option<Status>,
    status_rx: Option<mpsc::Receiver<Status>>,
    pub sel: CardId,
    pub focus: Option<SF>,
    pub focus_at: Instant,
    /// Caret and selection in the focused field.
    pub caret: Caret,
    /// Typing hides the pointer and its hover until the mouse moves.
    pub pointer_idle: bool,
    pub form: Form,
    pub show_secrets: bool,
    pub run: Option<Run>,
    run_rx: Option<mpsc::Receiver<SetupEvent>>,
    pub log_open: bool,
    pub toast: Option<(String, Instant)>,
    /// A blocker flashed after Enter on a button that can't go yet.
    pub flash: Option<Instant>,

    pub now: Instant,
    pub epoch: Instant,
    /// When the selected card last changed (the detail fades in).
    pub changed: Instant,
    pub window: Option<window::Id>,
    pub size: Size,
    pub tour: Option<tour::Tour>,
    /// The card to open on, instead of the first that wants something.
    pub start: Option<CardId>,
    /// The phone card detected again, while it waits for a phone.
    phone_rx: Option<mpsc::Receiver<CardStatus>>,
    phone_polled: Instant,
    /// Since when the phone card has waited for a phone, for the by-address help.
    pub phone_waiting_since: Option<Instant>,
}

impl Setup {
    pub fn new(
        backend: Arc<dyn SetupBackend>,
        tour: Option<tour::Tour>,
        start: Option<CardId>,
    ) -> (Self, Task<Message>) {
        let mut s = Self::headless(backend);
        s.tour = tour;
        s.start = start;
        (s, window::oldest().map(Message::Window))
    }

    /// No window: what the keyboard tests drive.
    pub fn headless(backend: Arc<dyn SetupBackend>) -> Self {
        let now = Instant::now();
        let mut s = Self {
            backend,
            status: None,
            status_rx: None,
            sel: CardId::Tuxedo,
            focus: Some(SF::Cards),
            focus_at: now,
            caret: Caret::default(),
            pointer_idle: false,
            form: Form::default(),
            show_secrets: false,
            run: None,
            run_rx: None,
            log_open: false,
            toast: None,
            flash: None,
            now,
            epoch: now,
            changed: now,
            window: None,
            // design-allow(size): the window's opening size, not layout.
            size: Size::new(1280.0, 800.0),
            tour: None,
            start: None,
            phone_rx: None,
            phone_polled: now,
            phone_waiting_since: None,
        };
        s.detect();
        s
    }

    fn detect(&mut self) {
        let (tx, rx) = mpsc::channel();
        let b = self.backend.clone();
        std::thread::spawn(move || {
            let _ = tx.send(b.status());
        });
        self.status_rx = Some(rx);
    }

    pub fn detecting(&self) -> bool {
        self.status_rx.is_some()
    }

    pub fn card(&self, id: CardId) -> Option<&CardStatus> {
        self.status.as_ref().and_then(|s| s.card(id))
    }

    pub fn selected(&self) -> Option<&CardStatus> {
        self.card(self.sel)
    }

    pub fn running(&self) -> bool {
        self.run.as_ref().is_some_and(Run::running)
    }

    /// The run shown on the selected card (its last one, if any).
    pub fn shown_run(&self) -> Option<&Run> {
        self.run.as_ref().filter(|r| r.card == self.sel)
    }

    pub fn settled(&self) -> bool {
        self.status.as_ref().is_some_and(Status::settled)
    }

    pub fn reboot_pending(&self) -> bool {
        self.status.as_ref().is_some_and(Status::reboot_pending)
    }

    /// The footer's line on what only takes effect after the reboot.
    pub fn reboot_nudge(&self) -> Option<String> {
        let waiting: Vec<CardId> = self
            .status
            .as_ref()?
            .cards
            .iter()
            .filter(|c| c.state == State::Reboot)
            .map(|c| c.id)
            .collect();
        match waiting.as_slice() {
            [] => None,
            [CardId::Tuxedo] => Some("The TUXEDO drivers load after a reboot.".into()),
            [one] => Some(format!("{} waits on a reboot.", one.title())),
            more => Some(format!("{} things wait on a reboot.", more.len())),
        }
    }

    // ------------------------------------------------------------ cards

    /// Secure Boot on and a key still to enroll: the TUXEDO card asks for
    /// the one-time MokManager password.
    pub fn wants_mok(&self) -> bool {
        match self.card(CardId::Tuxedo).map(|c| (&c.facts, c.state)) {
            Some((
                Facts::Tuxedo {
                    secure_boot: Some(true),
                    drivers,
                    mok,
                    ..
                },
                State::Needed,
            )) => !drivers || matches!(mok, Some(MokState::NotEnrolled | MokState::NoKey)),
            _ => false,
        }
    }

    /// The vault's folders a person can tick for SSH keys. The `env/` ones ([`ENV_PREFIX`]) hold
    /// project variables, never SSH keys, so ticking one would only empty the keys.
    pub fn bw_folders(&self) -> Vec<&str> {
        match self.card(CardId::Bitwarden).map(|c| &c.facts) {
            Some(Facts::Bitwarden { overview, .. }) => overview
                .folders
                .iter()
                .map(String::as_str)
                .filter(|f| !f.starts_with(ENV_PREFIX.as_str()))
                .collect(),
            _ => vec![],
        }
    }

    /// Ticks or unticks a folder; none ticked is the whole vault.
    fn toggle_folder(&mut self, name: &str) {
        let f = &mut self.form.folders;
        match f.iter().position(|x| x == name) {
            Some(i) => {
                f.remove(i);
            }
            None => f.push(name.to_string()),
        }
    }

    pub fn bw_logged_in(&self) -> bool {
        matches!(
            self.card(CardId::Bitwarden).map(|c| &c.facts),
            Some(Facts::Bitwarden { login, .. }) if login == "locked" || login == "unlocked"
        )
    }

    /// The YubiKey has no PIN or no fingerprint yet: that's a terminal job.
    pub fn key_needs_prep(&self) -> bool {
        matches!(
            self.card(CardId::Yubikey).map(|c| &c.facts),
            Some(Facts::Yubikey { pin, fingerprints, .. })
                if *pin == Some(false) || *fingerprints == Some(false)
        )
    }

    pub fn next_finger(&self) -> &'static str {
        match self.card(CardId::Fingerprint).map(|c| &c.facts) {
            Some(Facts::Fingerprint { fingers, .. }) => next_finger(fingers),
            _ => "right-index-finger",
        }
    }

    /// The phone link as the phone card last saw it.
    pub fn phone_link(&self) -> Option<&kubaos_core::phone::Status> {
        match self.card(CardId::Phone).map(|c| &c.facts) {
            Some(Facts::Phone { link }) => Some(link),
            _ => None,
        }
    }

    /// The phone card offers to turn the link off once it is on.
    pub fn can_turn_off(&self) -> bool {
        self.sel == CardId::Phone && self.phone_link().is_some_and(|l| l.on)
    }

    /// The phone card waits on the phone: for a pairing request, or for the user to accept one
    /// (the phone may withdraw it).
    fn polls_phone(&self) -> bool {
        self.sel == CardId::Phone
            && !self.running()
            && !self.detecting()
            && self.phone_link().is_some_and(|l| l.on && l.running)
            && self.card(CardId::Phone).is_some_and(|c| c.state.open())
    }

    /// No phone has shown up for a while: the network may block discovery.
    pub fn phone_by_address(&self) -> bool {
        self.phone_waiting_since
            .is_some_and(|since| self.now.saturating_duration_since(since) >= BY_ADDRESS_AFTER)
    }

    /// The selected card's main button.
    pub fn primary(&self) -> Option<(&'static str, Act)> {
        let c = self.selected()?;
        let s = c.state;
        let phone = self.phone_link();
        let r = match (c.id, s) {
            (CardId::Phone, State::Needed) if phone.is_some_and(|l| l.request().is_some()) => {
                ("Accept the pairing", Act::Accept)
            }
            (CardId::Phone, State::Needed) if phone.is_some_and(|l| l.on) => {
                ("Start KDE Connect again", Act::Run)
            }
            (CardId::Phone, State::Needed) => ("Set up a phone", Act::Run),
            (CardId::Phone, State::Done) => ("Send a test copy", Act::Test),
            (_, State::Waiting) => ("Check again", Act::Recheck),
            (CardId::Tuxedo, State::Needed) => {
                let drivers = matches!(c.facts, Facts::Tuxedo { drivers: true, .. });
                if drivers {
                    ("Enroll the key", Act::Run)
                } else {
                    ("Install the drivers", Act::Run)
                }
            }
            (CardId::Tuxedo, State::Reboot) => ("Reboot now", Act::Reboot),
            (CardId::Fingerprint, State::Needed) => ("Enroll a finger", Act::Run),
            (CardId::Yubikey, State::Needed) if self.key_needs_prep() => {
                ("Set it up in a terminal", Act::Terminal)
            }
            (CardId::Yubikey, State::Needed) => ("Enroll this key", Act::Run),
            (CardId::Bitwarden, State::Needed) if self.bw_logged_in() => {
                ("Unlock and pull", Act::Run)
            }
            (CardId::Bitwarden, State::Needed) => ("Log in and pull", Act::Run),
            (CardId::Git, State::Needed) => ("Save", Act::Run),
            _ => return None,
        };
        Some(r)
    }

    /// The quieter second button.
    pub fn secondary(&self) -> Option<(&'static str, Act)> {
        let c = self.selected()?;
        match (c.id, c.state) {
            (CardId::Fingerprint, State::Done) => Some(("Add another finger", Act::Run)),
            (CardId::Yubikey, State::Needed | State::Done) => Some(("Check again", Act::Recheck)),
            (CardId::Bitwarden, State::Done) => Some(("Pull again", Act::Run)),
            (CardId::Git, State::Done) => Some(("Save changes", Act::Run)),
            (CardId::Phone, State::Done) => Some(("Check what the phone sent", Act::Check)),
            _ => None,
        }
    }

    /// The text fields and choices the selected card shows.
    pub fn fields(&self) -> Vec<SF> {
        let Some(c) = self.selected() else {
            return vec![];
        };
        match (c.id, c.state) {
            (CardId::Tuxedo, State::Needed) if self.wants_mok() => vec![SF::Mok, SF::MokAgain],
            (CardId::Bitwarden, State::Needed | State::Done)
                if self.bw_logged_in() || c.state == State::Done =>
            {
                let mut v = vec![];
                if !self.bw_folders().is_empty() {
                    v.push(SF::Folders);
                }
                v.push(SF::Password);
                v
            }
            (CardId::Bitwarden, State::Needed) => {
                let mut v = vec![SF::Server];
                if self.form.server == Server::Custom {
                    v.push(SF::Url);
                }
                v.push(SF::Method);
                match self.form.method {
                    BwMethod::ApiKey => v.extend([SF::ClientId, SF::ClientSecret, SF::Password]),
                    _ => v.extend([SF::Email, SF::Password, SF::Code]),
                }
                v
            }
            (CardId::Git, State::Needed | State::Done) => vec![SF::GitName, SF::GitEmail],
            _ => vec![],
        }
    }

    pub fn focus_order(&self) -> Vec<SF> {
        let mut v = vec![SF::Cards];
        // The form hides while its card runs.
        let fields = if self.running() {
            vec![]
        } else {
            self.fields()
        };
        let secret = fields.iter().any(|f| f.is_secret());
        v.extend(fields);
        if secret {
            v.push(SF::ShowSecrets);
        }
        if self.primary().is_some() {
            v.push(SF::Primary);
        }
        if self.secondary().is_some() {
            v.push(SF::Secondary);
        }
        if self.can_log_out() {
            v.push(SF::LogOut);
        }
        if self.can_turn_off() {
            v.push(SF::TurnOff);
        }
        if self.status.is_some() {
            v.push(SF::Skip);
        }
        if self.shown_run().is_some_and(|r| !r.log.is_empty()) {
            v.push(SF::Log);
        }
        if self.reboot_pending() {
            v.push(SF::RebootAll);
        }
        v.push(SF::Finish);
        v
    }

    /// A logged-in Bitwarden card offers to log out, for another account or server.
    pub fn can_log_out(&self) -> bool {
        self.sel == CardId::Bitwarden && self.bw_logged_in()
    }

    /// Why the selected card's Run can't start yet.
    pub fn blocker(&self) -> Option<&'static str> {
        if self.running() {
            return Some("One thing at a time.");
        }
        let f = &self.form;
        match self.sel {
            CardId::Tuxedo if self.wants_mok() => {
                if let Err(e) = mok_password_ok(&f.mok) {
                    return Some(e);
                }
                if f.mok != f.mok_again {
                    return Some("The two passwords don't match.");
                }
                None
            }
            CardId::Bitwarden => {
                if f.password.is_empty() {
                    return Some("The master password, please.");
                }
                if self.bw_logged_in() || self.selected().is_some_and(|c| c.state == State::Done) {
                    return None;
                }
                if f.server == Server::Custom && !f.url.trim().starts_with("https://") {
                    return Some("Your server's https:// address.");
                }
                match f.method {
                    BwMethod::ApiKey if f.client_id.trim().is_empty() => {
                        Some("client_id from the web vault.")
                    }
                    BwMethod::ApiKey if f.client_secret.trim().is_empty() => {
                        Some("client_secret from the web vault.")
                    }
                    BwMethod::ApiKey => None,
                    _ if !f.email.contains('@') => Some("Your Bitwarden email."),
                    _ if f.code.trim().is_empty() => Some("The two-step code."),
                    _ => None,
                }
            }
            CardId::Git => git_identity_ok(&f.git_name, &f.git_email).err(),
            _ => None,
        }
    }

    fn inputs(&self) -> Inputs {
        let f = &self.form;
        let mut i = Inputs::default();
        match self.sel {
            CardId::Tuxedo if self.wants_mok() => i.mok_password = Some(f.mok.clone()),
            CardId::Fingerprint => i.finger = Some(self.next_finger().to_string()),
            CardId::Bitwarden => {
                if !self.bw_folders().is_empty() {
                    i.bw_folders = Some(f.folders.clone());
                }
                i.bw = Some(BwLogin {
                    server: match f.server {
                        Server::Com => "com".into(),
                        Server::Eu => "eu".into(),
                        Server::Custom => f.url.trim().to_string(),
                    },
                    email: f.email.trim().to_string(),
                    password: f.password.clone(),
                    method: f.method,
                    code: f.code.trim().to_string(),
                    client_id: f.client_id.trim().to_string(),
                    client_secret: f.client_secret.trim().to_string(),
                })
            }
            CardId::Git => {
                i.git = Some(GitIdentity {
                    name: f.git_name.trim().to_string(),
                    email: f.git_email.trim().to_string(),
                })
            }
            CardId::Phone => i.phone = Some(PhoneStep::On),
            _ => {}
        }
        i
    }

    /// Email, server and folders from what's on the machine (bw, the bitwarden
    /// config), and git's name and email (or the account's full name), once.
    fn prefill(&mut self) {
        if self.form.prefilled {
            return;
        }
        self.form.prefilled = true;
        if let Some(Facts::Git {
            name,
            email,
            full_name,
        }) = self.card(CardId::Git).map(|c| c.facts.clone())
        {
            self.form.git_name = name.or(full_name).unwrap_or_default();
            self.form.git_email = email.unwrap_or_default();
        }
        if let Some(Facts::Bitwarden {
            email,
            server,
            folders,
            ..
        }) = self.card(CardId::Bitwarden).map(|c| c.facts.clone())
        {
            self.form.folders = folders;
            if let Some(e) = email {
                self.form.email = e;
            }
            match server.as_deref() {
                Some(s) if s.contains("bitwarden.eu") => self.form.server = Server::Eu,
                Some(s) if s.contains("bitwarden.com") => self.form.server = Server::Com,
                Some(s) if s.starts_with("https://") => {
                    self.form.server = Server::Custom;
                    self.form.url = s.to_string();
                }
                _ => {}
            }
        }
    }

    // ----------------------------------------------------------- update

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            event::listen_with(crate::app::on_event),
            window::resize_events().map(|(_, s)| Message::Resized(s)),
            window::close_requests().map(|_| Message::Setup(Msg::CloseRequested)),
        ];
        if self.animating() {
            subs.push(window::frames().map(Message::Tick));
        }
        if self.pointer_idle {
            subs.push(event::listen_with(crate::app::on_mouse));
        }
        Subscription::batch(subs)
    }

    fn animating(&self) -> bool {
        let t = self.now;
        self.status_rx.is_some()
            || self.run_rx.is_some()
            || self.running()
            || t - self.changed < motion::LARGE
            || self.toast.is_some()
            || self
                .flash
                .is_some_and(|f| t - f < Duration::from_millis(900))
            || self.tour.is_some()
            || self.phone_rx.is_some()
            || self.polls_phone()
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        let task = self.update_inner(msg);
        if self.tour.is_some() {
            return Task::batch([task, tour::drive(self)]);
        }
        task
    }

    pub(crate) fn update_inner(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Tick(now) => self.tick(now),
            Message::Window(id) => {
                self.window = id;
                match id {
                    Some(id) => window::size(id).map(Message::Resized),
                    None => Task::none(),
                }
            }
            Message::Resized(s) => {
                self.size = s;
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
            Message::Setup(Msg::Pointer(f, ptr)) => {
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
            Message::Pasted(Some(s)) => {
                self.type_text(&s.replace(['\n', '\r'], ""));
                Task::none()
            }
            Message::Setup(Msg::Click(f)) => {
                self.set_focus(Some(f));
                if f.is_field() {
                    Task::none()
                } else {
                    self.activate(f)
                }
            }
            Message::Setup(Msg::Folder(name)) => {
                self.set_focus(Some(SF::Folders));
                if let Some(i) = self.bw_folders().iter().position(|f| *f == name) {
                    self.form.folder_at = i;
                }
                self.toggle_folder(&name);
                Task::none()
            }
            Message::Setup(Msg::Pick(id)) => {
                self.set_focus(Some(SF::Cards));
                self.select(id);
                Task::none()
            }
            Message::Setup(Msg::CloseRequested) => self.finish(),
            Message::TourShot(name, shot) => {
                tour::save(self, &name, &shot);
                if let Some(t) = &mut self.tour {
                    t.waiting_for_shot = false;
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn tick(&mut self, now: Instant) -> Task<Message> {
        self.now = now;
        if let Some(rx) = &self.status_rx
            && let Ok(st) = rx.try_recv()
        {
            self.status_rx = None;
            let first = self.status.is_none();
            self.status = Some(st);
            self.prefill();
            if first {
                // Start on the card asked for, else the first that wants something.
                if let Some(c) = self.start.or_else(|| {
                    self.status
                        .as_ref()
                        .and_then(|s| s.cards.iter().find(|c| c.state.open()))
                        .map(|c| c.id)
                }) {
                    self.sel = c;
                }
                self.changed = now;
            }
        }
        if let Some(rx) = &self.run_rx {
            let evs: Vec<SetupEvent> = rx.try_iter().collect();
            let closed = matches!(rx.try_recv(), Err(mpsc::TryRecvError::Disconnected));
            for e in evs {
                self.on_event(e);
            }
            if closed {
                self.run_rx = None;
                if self.running()
                    && let Some(r) = &mut self.run
                {
                    r.outcome = Some(Outcome::Failed {
                        error: "setup's worker died without a word".into(),
                        hint: Some("Run it again; the log has whatever it said.".into()),
                    });
                }
            }
        }
        self.poll_phone(now);
        if self
            .toast
            .as_ref()
            .is_some_and(|(_, at)| now - *at > Duration::from_secs(4))
        {
            self.toast = None;
        }
        Task::none()
    }

    /// Detects the phone card again every [`PHONE_POLL`] while it waits on the phone, and keeps
    /// the time it has waited for one.
    fn poll_phone(&mut self, now: Instant) {
        if let Some(rx) = &self.phone_rx
            && let Ok(card) = rx.try_recv()
        {
            self.phone_rx = None;
            if !self.running()
                && let Some(st) = &mut self.status
                && let Some(c) = st.cards.iter_mut().find(|c| c.id == CardId::Phone)
            {
                *c = card;
            }
        }
        let waiting = self.card(CardId::Phone).is_some_and(|c| c.state == State::Waiting);
        match (waiting, self.phone_waiting_since) {
            (true, None) => self.phone_waiting_since = Some(now),
            (false, Some(_)) => self.phone_waiting_since = None,
            _ => {}
        }
        let due = now.saturating_duration_since(self.phone_polled) >= PHONE_POLL;
        if due && self.phone_rx.is_none() && self.polls_phone() {
            self.phone_polled = now;
            let (tx, rx) = mpsc::channel();
            let b = Arc::clone(&self.backend);
            std::thread::spawn(move || {
                let _ = tx.send(b.card(CardId::Phone));
            });
            self.phone_rx = Some(rx);
        }
    }

    fn on_event(&mut self, e: SetupEvent) {
        let now = Instant::now();
        let Some(run) = &mut self.run else { return };
        match e {
            SetupEvent::Started { .. } => {}
            SetupEvent::Prompt {
                kind, text, count, ..
            } => {
                run.prompt = Some((kind, text, count));
                run.prompt_at = now;
            }
            SetupEvent::Log { level, line, .. } => run.log.push((level, line)),
            // Logged out, or the phone link off: the card is to do again on purpose.
            SetupEvent::Finished { status, .. } if run.undo.is_some() => {
                run.outcome = Some(Outcome::Done(run.undo.unwrap_or_default().into()));
                run.prompt = None;
                if let Some(st) = &mut self.status
                    && let Some(c) = st.cards.iter_mut().find(|c| c.id == status.id)
                {
                    *c = status;
                }
                self.form.password.clear();
                self.set_focus(Some(SF::Cards));
            }
            // A run that left its card still to do didn't work, whatever it exited with:
            // showing "Done." over a cleared form would ask for the same thing again in silence.
            SetupEvent::Finished { status, .. } if left_to_do(&status) => {
                run.outcome = Some(Outcome::Failed {
                    error: "it ran, but this still isn't done".into(),
                    hint: Some(format!("{} The log has what it said.", status.summary)),
                });
                run.prompt = None;
                if let Some(st) = &mut self.status
                    && let Some(c) = st.cards.iter_mut().find(|c| c.id == status.id)
                {
                    *c = status;
                }
                self.log_open = true;
            }
            SetupEvent::Finished { status, .. } => {
                run.outcome = Some(Outcome::Done(status.summary.clone()));
                run.prompt = None;
                if let Some(st) = &mut self.status
                    && let Some(c) = st.cards.iter_mut().find(|c| c.id == status.id)
                {
                    *c = status;
                }
                if let Some(Facts::Bitwarden { folders, .. }) = self.card(CardId::Bitwarden).map(|c| &c.facts) {
                    self.form.folders = folders.clone();
                }
                // Spent: never keep a secret longer than its run.
                let f = &mut self.form;
                for s in [
                    &mut f.mok,
                    &mut f.mok_again,
                    &mut f.password,
                    &mut f.code,
                    &mut f.client_secret,
                ] {
                    s.clear();
                }
                let next = if self.primary().is_some() {
                    SF::Primary
                } else {
                    SF::Skip
                };
                self.set_focus(Some(next));
            }
            SetupEvent::Failed { error, hint, .. } => {
                run.outcome = Some(Outcome::Failed { error, hint });
                run.prompt = None;
                self.form.code.clear();
                self.log_open = true;
            }
        }
    }

    fn start(&mut self) -> Task<Message> {
        if self.blocker().is_some() {
            self.flash = Some(Instant::now());
            return Task::none();
        }
        let inputs = self.inputs();
        self.spawn(inputs, None)
    }

    /// Logs out of Bitwarden: no form needed, the session goes with it.
    fn log_out(&mut self) -> Task<Message> {
        if self.running() {
            self.flash = Some(Instant::now());
            return Task::none();
        }
        let inputs = Inputs {
            bw_logout: true,
            ..Inputs::default()
        };
        self.spawn(inputs, Some("Logged out. Log in again, to this server or another."))
    }

    /// One step of the phone card that needs no form.
    fn phone_step(&mut self, step: PhoneStep, undo: Option<&'static str>) -> Task<Message> {
        if self.running() {
            self.flash = Some(Instant::now());
            return Task::none();
        }
        let inputs = Inputs {
            phone: Some(step),
            ..Inputs::default()
        };
        self.spawn(inputs, undo)
    }

    fn spawn(&mut self, inputs: Inputs, undo: Option<&'static str>) -> Task<Message> {
        let card = self.sel;
        let (tx, rx) = mpsc::channel();
        let b = self.backend.clone();
        std::thread::spawn(move || {
            b.run(card, &inputs, &mut |e| {
                let _ = tx.send(e);
            });
        });
        self.run_rx = Some(rx);
        // A phone card detected before this run would land after it, stale.
        self.phone_rx = None;
        self.run = Some(Run {
            card,
            prompt: None,
            prompt_at: Instant::now(),
            log: vec![],
            outcome: None,
            undo,
        });
        self.log_open = false;
        self.now = Instant::now();
        Task::none()
    }

    fn act(&mut self, a: Act) -> Task<Message> {
        match a {
            Act::Run => self.start(),
            Act::LogOut => self.log_out(),
            Act::Accept => {
                let Some(device) = self.phone_link().and_then(|l| l.request()).map(|d| d.id.clone()) else {
                    return Task::none();
                };
                self.phone_step(PhoneStep::Accept { device }, None)
            }
            Act::Test => self.phone_step(PhoneStep::Test, None),
            Act::Check => self.phone_step(PhoneStep::Check, None),
            Act::TurnOff => self.phone_step(
                PhoneStep::Off,
                Some("Phone link off. KDE Connect stopped; set a phone up again any time."),
            ),
            Act::Recheck => {
                if !self.detecting() && !self.running() {
                    self.detect();
                }
                Task::none()
            }
            Act::Reboot => {
                // Write the marker first only if nothing waits on the reboot;
                // a queued MOK brings setup back to confirm the drivers.
                if let Err(e) = self.backend.reboot() {
                    self.toast(format!("Reboot refused: {e}."));
                }
                Task::none()
            }
            Act::Terminal => {
                let pin = matches!(
                    self.card(CardId::Yubikey).map(|c| &c.facts),
                    Some(Facts::Yubikey {
                        pin: Some(false),
                        ..
                    })
                );
                let mut script = String::from("set -e\n");
                if pin {
                    script.push_str(
                        "echo 'The key needs its own FIDO2 PIN first. 6 to 8 characters if you might ever use PIV.'\nykman fido access change-pin\n",
                    );
                }
                script.push_str(
                    "echo 'Now two or three fingers on the key, one from the other hand.'\nykman fido fingerprints add right-index\nykman fido fingerprints add left-index\n",
                );
                if let Err(e) = self.backend.terminal(&script) {
                    self.toast(format!("{e}. In a terminal: {YKMAN_PREP}."));
                }
                Task::none()
            }
        }
    }

    fn toast(&mut self, s: String) {
        self.now = Instant::now();
        self.toast = Some((s, self.now));
    }

    /// Finish (or the window closing): the marker keeps setup closed at the
    /// next login, unless a reboot is pending that setup should confirm.
    fn finish(&mut self) -> Task<Message> {
        let pending = self.status.as_ref().is_some_and(Status::reboot_pending);
        if !pending && let Err(e) = self.backend.finish() {
            eprintln!("kobo: can't write the marker: {e}");
        }
        self.backend.start_bitwarden();
        iced::exit()
    }

    fn select(&mut self, id: CardId) {
        if id != self.sel {
            self.sel = id;
            self.changed = Instant::now();
            self.now = self.changed;
            self.show_secrets = false;
        }
    }

    fn move_card(&mut self, d: i32) {
        let all = CardId::ALL;
        let i = all.iter().position(|c| *c == self.sel).unwrap_or(0) as i32;
        let j = (i + d).clamp(0, all.len() as i32 - 1) as usize;
        self.select(all[j]);
    }

    /// Skip / Next: the next card that still wants something, else Finish.
    fn skip(&mut self) {
        let all = CardId::ALL;
        let i = all.iter().position(|c| *c == self.sel).unwrap_or(0);
        let next = all[i + 1..]
            .iter()
            .chain(all[..i].iter())
            .find(|c| self.card(**c).is_some_and(|s| s.state.open()))
            .copied();
        match next {
            Some(c) => {
                self.select(c);
                self.set_focus(Some(SF::Cards));
            }
            None => self.set_focus(Some(SF::Finish)),
        }
    }

    fn activate(&mut self, f: SF) -> Task<Message> {
        match f {
            SF::Cards => {
                // Into the card: its first field, else its button.
                let order = self.focus_order();
                let first = order.get(1).copied().filter(|f| *f != SF::Finish);
                self.set_focus(first.or(Some(SF::Finish)));
                Task::none()
            }
            SF::Primary => match self.primary() {
                Some((_, a)) if !self.running() => self.act(a),
                _ => Task::none(),
            },
            SF::Secondary => match self.secondary() {
                Some((_, a)) if !self.running() => self.act(a),
                _ => Task::none(),
            },
            SF::Skip => {
                self.skip();
                Task::none()
            }
            SF::ShowSecrets => {
                self.show_secrets = !self.show_secrets;
                Task::none()
            }
            SF::Server => {
                self.cycle_server(1);
                Task::none()
            }
            SF::Method => {
                self.cycle_method(1);
                Task::none()
            }
            SF::Log => {
                self.log_open = !self.log_open;
                Task::none()
            }
            SF::LogOut => self.act(Act::LogOut),
            SF::TurnOff => self.act(Act::TurnOff),
            SF::Folders => {
                if let Some(name) = self.bw_folders().get(self.form.folder_at).map(|f| f.to_string()) {
                    self.toggle_folder(&name);
                }
                Task::none()
            }
            SF::RebootAll => self.act(Act::Reboot),
            SF::Finish => self.finish(),
            _ => Task::none(),
        }
    }

    fn move_folder(&mut self, d: i32) {
        let n = self.bw_folders().len();
        if n > 0 {
            let i = self.form.folder_at as i32 + d;
            self.form.folder_at = i.clamp(0, n as i32 - 1) as usize;
        }
    }

    fn cycle_server(&mut self, d: i32) {
        let all = [Server::Com, Server::Eu, Server::Custom];
        let i = all.iter().position(|s| *s == self.form.server).unwrap_or(0) as i32;
        self.form.server = all[(i + d).rem_euclid(3) as usize];
    }

    fn cycle_method(&mut self, d: i32) {
        let all = [BwMethod::Totp, BwMethod::YubikeyOtp, BwMethod::ApiKey];
        let i = all.iter().position(|m| *m == self.form.method).unwrap_or(0) as i32;
        self.form.method = all[(i + d).rem_euclid(3) as usize];
    }

    pub fn is_focused(&self, f: SF) -> bool {
        self.focus == Some(f)
    }

    /// Moves focus; a field reached this way selects its whole value.
    fn set_focus(&mut self, f: Option<SF>) {
        self.focus = f;
        self.focus_at = Instant::now();
        if let Some(f) = f.filter(|f| f.is_field()) {
            self.caret = Caret::all(self.field_value(f));
        }
    }

    /// Applies an edit to the focused field; false when none has focus.
    fn edit(&mut self, e: Edit) -> bool {
        let Some(f) = self.focus.filter(|f| f.is_field()) else {
            return false;
        };
        let mut c = self.caret;
        let Some(s) = self.field_mut(f) else {
            return false;
        };
        edit::apply(s, &mut c, e);
        self.caret = c;
        self.focus_at = Instant::now();
        true
    }

    pub fn key(&mut self, k: Key) -> Task<Message> {
        let focus = self.focus;
        // Focus can point at something the card no longer has (it changed
        // state): fall back to the list.
        if let Some(f) = focus
            && !self.focus_order().contains(&f)
        {
            self.set_focus(Some(SF::Cards));
        }
        let focus = self.focus;
        match k {
            Key::Tab { shift } => {
                let order = self.focus_order();
                let n = order.len();
                let i = focus.and_then(|f| order.iter().position(|x| *x == f));
                let j = match (i, shift) {
                    (None, false) => 0,
                    (None, true) => n - 1,
                    (Some(i), false) => (i + 1) % n,
                    (Some(i), true) => (i + n - 1) % n,
                };
                self.set_focus(Some(order[j]));
                Task::none()
            }
            Key::Enter => match focus {
                // Enter in a field: on to the next one, and from the last
                // one, the card's button.
                Some(f) if f.is_field() => {
                    let fields = self.fields();
                    let i = fields.iter().position(|x| *x == f).unwrap_or(0);
                    match fields.get(i + 1) {
                        Some(n) => self.set_focus(Some(*n)),
                        None if self.primary().is_some() => {
                            self.set_focus(Some(SF::Primary));
                            return self.activate(SF::Primary);
                        }
                        // A done card's button is its second one: Pull again.
                        None if self.secondary().is_some() => {
                            self.set_focus(Some(SF::Secondary));
                            return self.activate(SF::Secondary);
                        }
                        None => {}
                    }
                    Task::none()
                }
                Some(f) => self.activate(f),
                None => Task::none(),
            },
            Key::EnterUp => Task::none(),
            Key::Esc => {
                if focus != Some(SF::Cards) {
                    self.set_focus(Some(SF::Cards));
                }
                Task::none()
            }
            Key::Space | Key::ShiftSpace => match focus {
                Some(f) if f.is_field() => {
                    self.type_text(" ");
                    Task::none()
                }
                Some(SF::Cards) | None => Task::none(),
                Some(f) => self.activate(f),
            },
            Key::Up | Key::Down | Key::PageUp | Key::PageDown => {
                let d = if matches!(k, Key::Up | Key::PageUp) {
                    -1
                } else {
                    1
                };
                match focus {
                    Some(SF::Cards) => self.move_card(d),
                    Some(SF::Server) => self.cycle_server(d),
                    Some(SF::Method) => self.cycle_method(d),
                    _ => {}
                }
                Task::none()
            }
            Key::Home { shift } | Key::End { shift } => {
                let home = matches!(k, Key::Home { .. });
                if !self.edit(if home {
                    Edit::Home { select: shift }
                } else {
                    Edit::End { select: shift }
                }) && focus == Some(SF::Cards)
                {
                    self.move_card(if home { -100 } else { 100 });
                }
                Task::none()
            }
            Key::Left { word, shift } | Key::Right { word, shift } => {
                let left = matches!(k, Key::Left { .. });
                let e = if left {
                    Edit::Left {
                        word,
                        select: shift,
                    }
                } else {
                    Edit::Right {
                        word,
                        select: shift,
                    }
                };
                if !self.edit(e) {
                    let d = if left { -1 } else { 1 };
                    match focus {
                        Some(SF::Server) => self.cycle_server(d),
                        Some(SF::Method) => self.cycle_method(d),
                        Some(SF::Folders) => self.move_folder(d),
                        _ => {}
                    }
                }
                Task::none()
            }
            Key::Backspace { word } => {
                self.edit(Edit::Backspace { word });
                Task::none()
            }
            Key::Delete { word } => {
                self.edit(Edit::Delete { word });
                Task::none()
            }
            Key::Clear => {
                self.edit(Edit::Clear);
                Task::none()
            }
            Key::SelectAll => {
                self.edit(Edit::SelectAll);
                Task::none()
            }
            Key::Copy | Key::Cut => {
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
                if focus.is_some_and(SF::is_field) {
                    iced::clipboard::read().map(Message::Pasted)
                } else {
                    Task::none()
                }
            }
            Key::Text(t) => {
                // A YubiKey typing its OTP lands in the code field wherever
                // focus was on the Bitwarden card.
                self.type_text(&t);
                Task::none()
            }
        }
    }

    fn type_text(&mut self, t: &str) {
        self.edit(Edit::Insert(t.to_string()));
    }

    fn field_mut(&mut self, f: SF) -> Option<&mut String> {
        let form = &mut self.form;
        Some(match f {
            SF::Mok => &mut form.mok,
            SF::MokAgain => &mut form.mok_again,
            SF::Url => &mut form.url,
            SF::Email => &mut form.email,
            SF::Password => &mut form.password,
            SF::Code => &mut form.code,
            SF::ClientId => &mut form.client_id,
            SF::ClientSecret => &mut form.client_secret,
            SF::GitName => &mut form.git_name,
            SF::GitEmail => &mut form.git_email,
            _ => return None,
        })
    }

    pub fn field_value(&self, f: SF) -> &str {
        let form = &self.form;
        match f {
            SF::Mok => &form.mok,
            SF::MokAgain => &form.mok_again,
            SF::Url => &form.url,
            SF::Email => &form.email,
            SF::Password => &form.password,
            SF::Code => &form.code,
            SF::ClientId => &form.client_id,
            SF::ClientSecret => &form.client_secret,
            SF::GitName => &form.git_name,
            SF::GitEmail => &form.git_email,
            _ => "",
        }
    }
}

/// A run that left its card still to do didn't work, whatever it exited with. The phone card is
/// the exception once its link runs: what is left (accepting a pairing) is up to the phone.
fn left_to_do(status: &CardStatus) -> bool {
    status.state == State::Needed && !matches!(&status.facts, Facts::Phone { link } if link.running)
}
