//! First-boot setup (docs/INSTALLER.md, "First boot"): the things the
//! payload can't carry because they need the hardware or the person.
//!
//! Five cards, each detected from the machine on every start, so re-running
//! never redoes a finished one: TUXEDO drivers (with the Secure Boot key),
//! the fingerprint reader, the YubiKey for sudo/polkit/lock screen, the
//! Bitwarden vault (SSH keys, git signing, gh, project env through
//! `dot-bw-setup`), and the name and email git commits with.
//!
//! Runs as the desktop user. The few steps that need root go through
//! `pkexec /usr/lib/kubaos/setup-helper`, a fixed root-owned script with
//! four verbs (installer/firstboot/setup-helper); the session's polkit agent
//! asks for the password. Secrets (master password, MOK password) travel on
//! stdin or in a child's environment, never in argv or a log.
//!
//! Everything that touches the machine goes through [`Sys`], so the
//! detection and the action logic run against a fake in the tests.

mod actions;
pub mod demo;
mod detect;
mod parse;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub use crate::engine::runner::{Cmd, Output, Stream};
use crate::event::Level;

pub use actions::{YKMAN_PREP, git_identity_ok, mok_password_ok, run};
pub use detect::{card_status, status};
pub use parse::*;

/// Where the engine puts the privileged helper (installer/firstboot/).
pub const HELPER: &str = "/usr/lib/kubaos/setup-helper";
/// KubaOS's root-owned checkout from the payload; the helper runs
/// `system/30-hardware.sh` from here, never from anything the user can write.
pub const SYSTEM_KUBAOS: &str = "/usr/share/kubaos";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardId {
    Tuxedo,
    Fingerprint,
    Yubikey,
    Bitwarden,
    Git,
}

impl CardId {
    pub const ALL: [CardId; 5] = [
        CardId::Tuxedo,
        CardId::Fingerprint,
        CardId::Yubikey,
        CardId::Bitwarden,
        CardId::Git,
    ];

    pub fn title(self) -> &'static str {
        match self {
            CardId::Tuxedo => "TUXEDO drivers",
            CardId::Fingerprint => "Fingerprint",
            CardId::Yubikey => "YubiKey",
            CardId::Bitwarden => "Bitwarden",
            CardId::Git => "Git",
        }
    }

    pub fn parse(s: &str) -> Option<CardId> {
        serde_json::from_value(serde_json::Value::String(s.into())).ok()
    }

    /// Needs the internet to do its thing (apt, the vault).
    pub fn needs_network(self) -> bool {
        !matches!(self, CardId::Yubikey | CardId::Git)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    /// Nothing left to do.
    Done,
    /// Can be done now.
    Needed,
    /// Done, but it only counts after a reboot (MOK enrolment, new modules).
    Reboot,
    /// Needs something first: the internet, a key plugged in.
    Waiting,
    /// Not on this machine. Shown, never nagged about.
    Absent,
}

impl State {
    /// Whether the card wants attention at all.
    pub fn open(self) -> bool {
        matches!(self, State::Needed | State::Waiting)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MokState {
    Enrolled,
    /// Imported, waiting for the blue MokManager screen on the next boot.
    Pending,
    NotEnrolled,
    /// Secure Boot is on but DKMS hasn't made its key yet.
    NoKey,
    Unknown,
}

/// What detection found, per card. The GUI words its screens from this.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(tag = "card", rename_all = "snake_case")]
pub enum Facts {
    #[default]
    None,
    Tuxedo {
        vendor: String,
        product: String,
        /// `None` without EFI variables.
        secure_boot: Option<bool>,
        drivers: bool,
        modules: Vec<String>,
        mok: Option<MokState>,
    },
    Fingerprint {
        /// "Goodix MOC Fingerprint Sensor (27c6:609c)".
        reader: Option<String>,
        fprintd: bool,
        fingers: Vec<String>,
        pam: bool,
    },
    Yubikey {
        /// The plugged-in YubiKeys, by USB product name.
        plugged: Vec<String>,
        /// Credentials in /etc/u2f_mappings for this user.
        enrolled: usize,
        /// The plugged-in key is one of them.
        this_key_enrolled: bool,
        pin: Option<bool>,
        fingerprints: Option<bool>,
    },
    Bitwarden {
        bw: bool,
        /// `bw status`: unauthenticated, locked or unlocked.
        login: String,
        email: Option<String>,
        server: Option<String>,
        /// `dot-bw-setup` has written its files.
        configured: bool,
        /// The desktop app (its SSH agent) is installed (amd64 only).
        desktop: bool,
    },
    Git {
        /// What git commits with now, from any of its config files.
        name: Option<String>,
        email: Option<String>,
        /// The account's full name (passwd), a guess for `name`.
        full_name: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardStatus {
    pub id: CardId,
    pub state: State,
    /// One line for the list.
    pub summary: String,
    #[serde(default)]
    pub facts: Facts,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pub online: bool,
    pub cards: Vec<CardStatus>,
}

impl Status {
    pub fn card(&self, id: CardId) -> Option<&CardStatus> {
        self.cards.iter().find(|c| c.id == id)
    }

    /// Setup has nothing to say any more: every card is done, absent, or
    /// waiting on a reboot it already asked for.
    pub fn settled(&self) -> bool {
        self.cards.iter().all(|c| !c.state.open())
    }

    pub fn reboot_pending(&self) -> bool {
        self.cards.iter().any(|c| c.state == State::Reboot)
    }
}

/// What the person has to do right now, shown big.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptKind {
    /// polkit's dialog is up: type your password (or touch the key).
    Password,
    /// The laptop's fingerprint reader.
    Finger,
    /// The YubiKey's sensor.
    Key,
    /// Nothing; the machine is busy.
    Wait,
}

/// The setup's event stream. `dotinstall setup run --events json` writes one
/// per line; the GUI gets the same values in-process.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SetupEvent {
    Started {
        card: CardId,
    },
    Prompt {
        card: CardId,
        kind: PromptKind,
        text: String,
        /// (done, of) for multi-touch enrolment.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        count: Option<(u32, u32)>,
    },
    Log {
        card: CardId,
        level: Level,
        line: String,
    },
    /// Last event on success: the card as detected afterwards.
    Finished {
        card: CardId,
        status: CardStatus,
    },
    /// Last event on failure.
    Failed {
        card: CardId,
        error: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        hint: Option<String>,
    },
}

impl SetupEvent {
    pub fn to_json_line(&self) -> String {
        serde_json::to_string(self).expect("events always serialize")
    }

    pub fn from_json_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BwMethod {
    /// Email, master password, authenticator-app code (`--method 0`).
    #[default]
    Totp,
    /// The code a YubiKey 5 types when touched (`--method 3`). The Bio can't.
    YubikeyOtp,
    /// Personal API key: no two-step code (docs/SECRETS.md).
    ApiKey,
}

/// What a card's action needs from the person. Never logged; `Debug` hides
/// every secret.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Inputs {
    /// One-time password for MokManager (Secure Boot on).
    pub mok_password: Option<String>,
    /// fprintd finger name, e.g. `right-index-finger`.
    pub finger: Option<String>,
    pub bw: Option<BwLogin>,
    pub git: Option<GitIdentity>,
}

/// The name and email git commits with. Not secret.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GitIdentity {
    pub name: String,
    pub email: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BwLogin {
    /// `com`, `eu` or an https:// URL.
    pub server: String,
    pub email: String,
    pub password: String,
    pub method: BwMethod,
    pub code: String,
    pub client_id: String,
    pub client_secret: String,
}

impl std::fmt::Debug for Inputs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inputs")
            .field(
                "mok_password",
                &self.mok_password.as_ref().map(|_| "<secret>"),
            )
            .field("finger", &self.finger)
            .field("bw", &self.bw)
            .field("git", &self.git)
            .finish()
    }
}

impl std::fmt::Debug for BwLogin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BwLogin")
            .field("server", &self.server)
            .field("email", &self.email)
            .field("method", &self.method)
            .finish_non_exhaustive()
    }
}

/// Who and where. `Env::current()` on the machine; tests build their own.
#[derive(Debug, Clone)]
pub struct Env {
    pub home: PathBuf,
    pub user: String,
    /// `pkexec <helper>`: what runs the privileged verbs.
    pub privileged: Vec<String>,
}

impl Env {
    pub fn current() -> Env {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("LOGNAME"))
            .unwrap_or_default();
        Env {
            home,
            user,
            privileged: vec!["pkexec".into(), HELPER.into()],
        }
    }

    pub fn home_path(&self, rel: &str) -> String {
        self.home.join(rel).display().to_string()
    }

    /// `$XDG_STATE_HOME/kubaos`, where the marker lives.
    pub fn state_dir(&self) -> PathBuf {
        std::env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .unwrap_or_else(|| self.home.join(".local/state"))
            .join("kubaos")
    }

    /// Present once setup was finished or dismissed; the login autostart
    /// (`kubaos-setup.service`) is conditioned on its absence.
    pub fn marker(&self) -> PathBuf {
        self.state_dir().join("setup-done")
    }

    /// A user script from the dotfiles (`~/.local/bin`, home-manager's live
    /// links), by absolute path so a stripped PATH doesn't matter.
    pub fn script(&self, name: &str) -> String {
        self.home_path(&format!(".local/bin/{name}"))
    }

    fn privileged(&self, verb: &str) -> Cmd {
        let mut c = Cmd::new(self.privileged[0].clone()).args(self.privileged[1..].iter().cloned());
        c = c.arg(verb);
        c
    }
}

/// Everything setup reads or runs. `RealSys` is the machine; the tests use
/// a fake with canned files and command output.
pub trait Sys {
    /// A file's text, `None` when missing or unreadable.
    fn read(&mut self, path: &str) -> Option<String>;
    fn exists(&mut self, path: &str) -> bool;
    /// Entry names in a directory (not paths), sorted.
    fn list(&mut self, dir: &str) -> Vec<String>;
    /// Run to completion; a non-zero exit is `Err` with the output's tail.
    fn run(&mut self, cmd: &Cmd) -> Result<Output, String>;
    /// Run, handing over each output line as it comes.
    fn stream(&mut self, cmd: &Cmd, f: &mut dyn FnMut(Stream)) -> Result<(), String>;
    /// Write a user-owned file (mode 0600 for anything under ~/.config).
    fn write(&mut self, path: &str, contents: &str) -> Result<(), String>;
    /// Last-resort online check when NetworkManager can't answer.
    fn reachable(&mut self) -> bool {
        false
    }
}

/// The real machine.
#[derive(Default)]
pub struct RealSys {
    runner: crate::engine::runner::SystemRunner,
}

impl RealSys {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Sys for RealSys {
    fn read(&mut self, path: &str) -> Option<String> {
        std::fs::read(path)
            .ok()
            .map(|b| String::from_utf8_lossy(&b).into_owned())
    }

    fn exists(&mut self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }

    fn list(&mut self, dir: &str) -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        v.sort();
        v
    }

    fn run(&mut self, cmd: &Cmd) -> Result<Output, String> {
        use crate::engine::runner::Runner;
        self.runner.run(cmd)
    }

    fn stream(&mut self, cmd: &Cmd, f: &mut dyn FnMut(Stream)) -> Result<(), String> {
        use crate::engine::runner::Runner;
        self.runner.stream(cmd, f)
    }

    fn write(&mut self, path: &str, contents: &str) -> Result<(), String> {
        use crate::engine::runner::Runner;
        self.runner
            .write_file(std::path::Path::new(path), contents.as_bytes(), 0o600)
    }

    fn reachable(&mut self) -> bool {
        use std::net::{SocketAddr, TcpStream};
        use std::time::Duration;
        // No DNS on purpose: it can't time out. Cloudflare's and Quad9's
        // resolvers answer on 443.
        ["1.1.1.1:443", "9.9.9.9:443"].iter().any(|a| {
            a.parse::<SocketAddr>()
                .is_ok_and(|a| TcpStream::connect_timeout(&a, Duration::from_secs(2)).is_ok())
        })
    }
}

/// Writes the marker. Setup then stays closed at login; `kubaos-setup`
/// still opens it.
pub fn mark_done(sys: &mut dyn Sys, env: &Env) -> Result<(), String> {
    let p = env.marker();
    sys.write(
        &p.display().to_string(),
        "KubaOS first-boot setup was closed. Delete this file to see it at the next login.\n",
    )
}

#[cfg(test)]
pub(crate) mod fake;
