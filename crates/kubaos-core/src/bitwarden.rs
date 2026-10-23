//! One Bitwarden vault for the machine's secrets (docs/SECRETS.md): `kubaos bitwarden` and
//! Kōbō's Bitwarden card both run this code.
//!
//! A pull reads the whole vault once and writes what the session needs from it: every SSH key
//! item's public half and Host block, commit signing with the key marked `signing=true`, the
//! `gh-token` item into gh's keyring entry, and every project's env into the GNOME keyring.
//! `folders=` in the config narrows the SSH keys to some folders.
//!
//! Private keys never leave the vault and nothing secret is written to disk or logged. The
//! session lives in a [`Session`] in this process and goes to `bw` alone, in its environment.
//! The master password and the two-step code go to bw's own prompt on a terminal, or, from
//! Kōbō's form, to bw's environment.
//!
//! # Owns
//!
//! - Logging in and unlocking through `bw` ([`login`], [`logout`]).
//! - What a pull writes ([`pull`]) and the read-only check ([`check`]).
//! - The project env cache in the keyring ([`env`]), which `use bwenv` in an `.envrc` reads.
//! - The non-secret config (`~/.config/dot-bw/config`) and what the last pull found
//!   ([`Overview`]), which Kōbō shows.
//!
//! # Not here
//!
//! - The command line: `crates/kubaos-cli/src/bitwarden.rs`.
//! - The first-boot card, its form and its events: `crate::setup` (`setup/actions.rs`).

pub mod agent;
mod base64;
mod config;
pub mod env;
#[cfg(test)]
pub(crate) mod fake;
mod gh;
mod git;
mod login;
mod real;
mod ssh;
mod vault;

use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

pub use crate::engine::runner::Cmd;
pub use crate::engine::runner::Sensitive;
pub use agent::Agent;
pub use config::Config;
pub use login::Ask;
pub use login::Form;
pub use login::FormMethod;
pub use login::Login;
pub use login::LoginRequest;
pub use login::Method;
pub use login::NoTerminal;
pub use login::login;
pub use login::logout;
pub use real::Real;
pub use vault::scope;

/// The session `bw unlock` or `bw login` hands out. It unlocks the whole vault, so it stays in
/// memory and goes to `bw` alone.
#[derive(Clone)]
pub struct Session(String);

impl Session {
    /// A session from `bw` or from `BW_SESSION`.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// The key, for `bw`'s environment.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Session(<redacted>)")
    }
}

/// `bw` failed. The text is what bw said on stderr, which names the cause.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct BwError(String);

impl BwError {
    /// bw's words.
    #[must_use]
    pub fn new(detail: impl Into<String>) -> Self {
        Self(detail.into())
    }

    pub fn detail(&self) -> &str {
        &self.0
    }
}

/// The vault, through the `bw` CLI. Every call is one bw process; none of them prompts unless
/// the call says so.
pub trait Bw {
    fn installed(&mut self) -> bool;
    /// `bw status` as JSON, with the session when there is one.
    ///
    /// # Errors
    ///
    /// bw failed.
    fn status(&mut self, session: Option<&Session>) -> Result<String, BwError>;
    /// `bw config server URL`.
    ///
    /// # Errors
    ///
    /// bw failed.
    fn set_server(&mut self, url: &str) -> Result<(), BwError>;
    /// `bw login`. The session, for the logins that print one.
    ///
    /// # Errors
    ///
    /// bw refused the login.
    fn login(&mut self, how: &Login) -> Result<Option<Session>, BwError>;
    /// `bw unlock`: with the master password given, or bw's prompt on the terminal.
    ///
    /// # Errors
    ///
    /// bw refused to unlock.
    fn unlock(&mut self, password: Option<&Sensitive>) -> Result<Session, BwError>;
    /// `bw logout`.
    ///
    /// # Errors
    ///
    /// bw failed.
    fn logout(&mut self) -> Result<(), BwError>;
    /// `bw sync`.
    ///
    /// # Errors
    ///
    /// bw failed.
    fn sync(&mut self, session: &Session) -> Result<(), BwError>;
    /// `bw list folders` as JSON.
    ///
    /// # Errors
    ///
    /// bw failed.
    fn folders(&mut self, session: &Session) -> Result<String, BwError>;
    /// `bw list items` as JSON: the whole vault, secrets included, parsed and dropped at once.
    ///
    /// # Errors
    ///
    /// bw failed.
    fn items(&mut self, session: &Session) -> Result<String, BwError>;
}

/// What a program besides `bw` did.
#[derive(Debug, Clone, Default)]
pub struct Ran {
    /// `None` when a signal ended it.
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl Ran {
    pub fn ok(&self) -> bool {
        self.code == Some(0)
    }
}

/// The machine a pull writes to: files and the programs besides `bw`.
pub trait Host {
    fn read(&mut self, path: &Path) -> Option<String>;
    /// Writes `contents` atomically with `mode`, making the parent directories.
    ///
    /// # Errors
    ///
    /// The file can't be written.
    fn write(&mut self, path: &Path, contents: &str, mode: u32) -> Result<(), String>;
    /// Makes the directory and its parents, and sets its mode.
    ///
    /// # Errors
    ///
    /// The directory can't be made.
    fn mkdir(&mut self, path: &Path, mode: u32) -> Result<(), String>;
    /// Removes a file; a missing one is fine.
    fn remove(&mut self, path: &Path);
    fn is_file(&mut self, path: &Path) -> bool;
    fn exists(&mut self, path: &Path) -> bool;
    fn is_socket(&mut self, path: &Path) -> bool;
    /// The program is on `PATH`.
    fn has(&mut self, program: &str) -> bool;
    /// Runs a program to its end.
    ///
    /// # Errors
    ///
    /// The program couldn't start.
    fn run(&mut self, cmd: &Cmd) -> Result<Ran, String>;
}

/// Both halves, which every operation here takes as one value.
pub trait Machine: Bw + Host {}

impl<T: Bw + Host> Machine for T {}

/// Where everything lives. The directory names are the old dotfiles' (`dot-bw`,
/// `git-bitwarden`): `config/git/config` includes the signing file by that path, and a rename
/// would orphan what machines already have.
#[derive(Debug, Clone)]
pub struct Paths {
    /// For messages, which print `~` for it.
    pub home: PathBuf,
    /// `server=`, `email=` and `folders=`, nothing secret.
    pub config: PathBuf,
    /// What the last pull wrote (`ssh-pubkeys`) and found (`overview.json`).
    pub state: PathBuf,
    pub git: PathBuf,
    pub ssh: PathBuf,
    /// Bitwarden desktop's SSH agent.
    pub agent: PathBuf,
    /// `SSH_AUTH_SOCK` as this process got it.
    pub auth_sock: Option<String>,
}

impl Paths {
    /// From the environment: `HOME`, the XDG directories, `BITWARDEN_SSH_AUTH_SOCK`.
    #[must_use]
    pub fn from_env() -> Self {
        let home = std::env::var_os("HOME").map_or_else(|| PathBuf::from("/"), PathBuf::from);
        let xdg = |var: &str, default: &str| {
            std::env::var_os(var)
                .map(PathBuf::from)
                .filter(|p| p.is_absolute())
                .unwrap_or_else(|| home.join(default))
        };
        let (config, state) = (xdg("XDG_CONFIG_HOME", ".config"), xdg("XDG_STATE_HOME", ".local/state"));
        let agent = std::env::var_os("BITWARDEN_SSH_AUTH_SOCK")
            .map_or_else(|| home.join(".bitwarden-ssh-agent.sock"), PathBuf::from);
        let auth_sock = std::env::var("SSH_AUTH_SOCK").ok();
        Self::new(&home, &config, &state, agent, auth_sock)
    }

    /// Under `home`, with the default XDG directories: what the tests use.
    #[must_use]
    pub fn under(home: &Path) -> Self {
        let agent = home.join(".bitwarden-ssh-agent.sock");
        let sock = agent.display().to_string();
        Self::new(home, &home.join(".config"), &home.join(".local/state"), agent, Some(sock))
    }

    fn new(home: &Path, config: &Path, state: &Path, agent: PathBuf, auth_sock: Option<String>) -> Self {
        Self {
            home: home.to_path_buf(),
            config: config.join("dot-bw/config"),
            state: state.join("dot-bw"),
            git: config.join("git-bitwarden"),
            ssh: home.join(".ssh"),
            agent,
            auth_sock,
        }
    }

    /// `path` with `~` for the home directory, for messages.
    pub fn show(&self, path: &Path) -> String {
        match path.strip_prefix(&self.home) {
            Ok(rest) => format!("~/{}", rest.display()),
            Err(_) => path.display().to_string(),
        }
    }

    fn ssh_config(&self) -> PathBuf {
        self.ssh.join("config.d/bitwarden")
    }

    fn signing(&self) -> PathBuf {
        self.git.join("signing.gitconfig")
    }

    fn pubkeys(&self) -> PathBuf {
        self.state.join("ssh-pubkeys")
    }

    fn overview(&self) -> PathBuf {
        self.state.join("overview.json")
    }
}

/// One line of progress, as the terminal shows it (`==>`, `ok`, `!!`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    /// A part starts.
    Step(String),
    Ok(String),
    Info(String),
    /// Something needs the person; it goes into [`Report::warnings`] too.
    Warn(String),
}

/// What a run leaves: the summary lines and everything that needs the person.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub summary: Vec<String>,
    pub warnings: Vec<String>,
}

/// Hands each line to the caller as it comes and keeps the report.
pub struct Progress<'a> {
    emit: &'a mut dyn FnMut(&Line),
    report: Report,
}

impl std::fmt::Debug for Progress<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Progress").field("report", &self.report).finish_non_exhaustive()
    }
}

impl<'a> Progress<'a> {
    #[must_use]
    pub fn new(emit: &'a mut dyn FnMut(&Line)) -> Self {
        Self {
            emit,
            report: Report::default(),
        }
    }

    pub fn finish(self) -> Report {
        self.report
    }

    fn line(&mut self, l: Line) {
        (self.emit)(&l);
        if let Line::Warn(w) = l {
            self.report.warnings.push(w);
        }
    }

    fn step(&mut self, s: impl Into<String>) {
        self.line(Line::Step(s.into()));
    }

    fn ok(&mut self, s: impl Into<String>) {
        self.line(Line::Ok(s.into()));
    }

    fn info(&mut self, s: impl Into<String>) {
        self.line(Line::Info(s.into()));
    }

    fn warn(&mut self, s: impl Into<String>) {
        self.line(Line::Warn(s.into()));
    }

    fn summary(&mut self, s: impl Into<String>) {
        self.report.summary.push(s.into());
    }

    /// Takes back the last warning, once what it warned about was dealt with.
    fn unwarn(&mut self) {
        self.report.warnings.pop();
    }
}

/// Why a Bitwarden operation stopped. The message is the terminal's last line; the cause, when
/// bw gave one, is the source.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("{0} not found: {1}")]
    Missing(&'static str, &'static str),
    #[error("not set up yet: run kubaos bitwarden setup")]
    NotSetUp,
    #[error("bw is logged out: run kubaos bitwarden setup to log in")]
    LoggedOut,
    #[error("bw is logged in to {current}, not {wanted}: run 'bw logout' first")]
    OtherServer { current: String, wanted: String },
    #[error("unexpected bw status '{0}'")]
    UnknownStatus(String),
    #[error("bw status failed")]
    Status(#[source] BwError),
    #[error("{0}")]
    Server(String),
    #[error("need an answer for '{0}' but stdin isn't a terminal")]
    NoTerminal(String),
    #[error("pick 1, 2 or 3")]
    Choice,
    #[error("not logged in")]
    NotLoggedIn,
    /// The form lacks something the login needs.
    #[error("{0}")]
    Incomplete(&'static str),
    #[error("{what}")]
    Login {
        what: &'static str,
        #[source]
        cause: BwError,
    },
    #[error("unlock failed")]
    Unlock(#[source] BwError),
    #[error("bw unlock failed (not logged in? run kubaos bitwarden setup)")]
    NotUnlocked(#[source] BwError),
    #[error("bw returned no session")]
    NoSession,
    #[error("bw config server failed")]
    SetServer(#[source] BwError),
    #[error("bw logout failed")]
    Logout(#[source] BwError),
    #[error("bw sync failed")]
    Sync(#[source] BwError),
    #[error("bw list {0} failed")]
    List(&'static str, #[source] BwError),
    #[error("bw list {0} printed something that isn't the vault")]
    Parse(&'static str, #[source] serde_json::Error),
    #[error("can't write {0}")]
    Write(String, #[source] HostError),
    #[error("bad project name '{0}' (letters, digits, . _ -)")]
    Project(String),
    #[error("can't read the keyring (gnome-keyring not running, or you dismissed its unlock prompt)")]
    Keyring(#[source] HostError),
}

/// The machine refused: a write, or a program that wouldn't start.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct HostError(String);

impl Error {
    fn write(path: &Path, e: String) -> Self {
        Self::Write(path.display().to_string(), HostError(e))
    }
}

/// Which parts a pull writes (`--only`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parts {
    pub ssh: bool,
    pub git: bool,
    pub gh: bool,
    pub env: bool,
}

impl Parts {
    pub const ALL: Parts = Parts {
        ssh: true,
        git: true,
        gh: true,
        env: true,
    };

    /// `ssh,git,gh,env`, or any of them.
    ///
    /// # Errors
    ///
    /// A word that isn't a part.
    pub fn parse(list: &str) -> Result<Parts, String> {
        let mut p = Parts {
            ssh: false,
            git: false,
            gh: false,
            env: false,
        };
        for w in list.split(',').map(str::trim).filter(|w| !w.is_empty()) {
            match w {
                "ssh" => p.ssh = true,
                "git" => p.git = true,
                "gh" => p.gh = true,
                "env" => p.env = true,
                _ => return Err(format!("{w} isn't a part: ssh, git, gh or env")),
            }
        }
        Ok(p)
    }
}

/// The vault's folders and its SSH keys as the last pull found them, for Kōbō's card. Names,
/// public-key fingerprints and host names only: nothing here is secret.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Overview {
    /// Every folder in the vault, by name, sorted.
    pub folders: Vec<String>,
    pub keys: Vec<KeyInfo>,
}

/// One SSH key item and what the pull made of it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyInfo {
    pub name: String,
    /// "no folder" for an item in none.
    pub folder: String,
    pub fingerprint: String,
    /// `~/.ssh/<file>.pub`, once written.
    pub file: Option<String>,
    /// From its `hosts` field.
    pub hosts: Vec<String>,
    /// It got a Host block in `~/.ssh/config.d/bitwarden`.
    pub block: bool,
    /// Why it has no Host block, or no public key on disk.
    pub note: Option<String>,
    /// git signs commits with it.
    pub signing: bool,
}

impl Overview {
    /// What the last pull left, or nothing yet.
    pub fn read(host: &mut dyn Host, paths: &Paths) -> Overview {
        host.read(&paths.overview())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
}

/// Checks the programs a pull runs, before anything logs in.
///
/// # Errors
///
/// One is missing.
pub fn preflight(m: &mut dyn Machine, parts: Parts) -> Result<(), Error> {
    for t in ["ssh-add", "ssh-keygen", "git"] {
        if !m.has(t) {
            return Err(Error::Missing(t, "it comes with the base system"));
        }
    }
    if !m.installed() {
        return Err(Error::Missing("bw", "run system/70-bitwarden.sh"));
    }
    if (parts.env || parts.gh) && !m.has("secret-tool") {
        return Err(Error::Missing("secret-tool", "run system/70-bitwarden.sh"));
    }
    Ok(())
}

/// Reads the vault once and writes every part in `parts`, then checks the agent.
///
/// # Errors
///
/// bw failed to read the vault, or a file couldn't be written.
pub fn pull(m: &mut dyn Machine, paths: &Paths, session: &Session, parts: Parts, p: &mut Progress) -> Result<(), Error> {
    p.step("Reading the vault");
    m.sync(session).map_err(Error::Sync)?;
    let folders = m.folders(session).map_err(|e| Error::List("folders", e))?;
    let folders = vault::folders(&folders).map_err(|e| Error::Parse("folders", e))?;
    let config = Config::read(m, paths);
    let filter = vault::Filter::pick(&folders, &config.folders(), p);
    let items = m.items(session).map_err(|e| Error::List("items", e))?;
    let vault = vault::Vault::parse(&items, &folders).map_err(|e| Error::Parse("items", e))?;
    drop(items);
    let keys = vault.ssh_keys(&filter);
    p.ok(format!("{} items, {} SSH key(s) in {}", vault.len(), keys.len(), filter.scope));
    let mut keys = ssh::name_files(keys, p);

    let mut overview = Overview {
        // bw's "No Folder" has no id: no filter can name it.
        folders: folders.iter().filter(|f| f.id.is_some()).map(|f| f.name.clone()).collect(),
        keys: vec![],
    };
    overview.folders.sort();
    overview.folders.dedup();
    if parts.ssh {
        overview.keys = ssh::apply(m, paths, &mut keys, &filter.scope, p)?;
    }
    if parts.git {
        git::apply(m, paths, &keys, &filter.scope, p)?;
    }
    if parts.gh {
        gh::apply(m, &vault, p);
    }
    if parts.env {
        env::setup(m, &vault, p);
    }
    let fingerprints: Vec<String> = keys.iter().filter_map(|k| k.item.fingerprint.clone()).collect();
    agent::check(m, paths, &fingerprints, &filter.scope, p);
    if parts.ssh {
        git::mark_signing(&mut overview.keys, &keys);
        // Strings, lists and booleans: serializing can't fail.
        let json = serde_json::to_string_pretty(&overview).expect("serialize the overview");
        let path = paths.overview();
        m.write(&path, &json, 0o600).map_err(|e| Error::write(&path, e))?;
    }
    Ok(())
}

/// No bw at all: is the agent up, and which files does the last pull's output have.
pub fn check(m: &mut dyn Machine, paths: &Paths, p: &mut Progress) {
    let config = Config::read(m, paths);
    let scope = vault::scope(&config.folders());
    let mut fingerprints = vec![];
    for f in m.read(&paths.pubkeys()).unwrap_or_default().lines().filter(|l| ssh::safe_file(l)) {
        let path = paths.ssh.join(f);
        if !m.is_file(&path) {
            continue;
        }
        let cmd = Cmd::new("ssh-keygen").arg("-lf").arg(path.display().to_string());
        if let Ok(r) = m.run(&cmd)
            && let Some(fp) = r.stdout.split_whitespace().nth(1)
        {
            fingerprints.push(fp.to_string());
        }
    }
    agent::check(m, paths, &fingerprints, &scope, p);
    for f in [paths.ssh_config(), paths.signing()] {
        let missing = if m.is_file(&f) { "" } else { " missing" };
        p.summary(format!("file   {}{missing}", paths.show(&f)));
    }
    if m.has("secret-tool")
        && let Ok(projects) = env::projects(m)
    {
        p.summary(format!("env    {}", projects.join(" ")));
    }
}

/// Writes `folders=` (or removes it, for the whole vault): where the next pull takes SSH keys
/// from.
///
/// # Errors
///
/// The config can't be written.
pub fn set_folders(m: &mut dyn Machine, paths: &Paths, folders: &[String]) -> Result<(), Error> {
    let mut c = Config::read(m, paths);
    if c.folders() == folders {
        return Ok(());
    }
    c.set_folders(folders);
    c.write(m, paths)
}

/// A name from the vault, safe to put on one line of a config file: a control character (a
/// newline would start a new directive) becomes a space.
fn one_line(s: &str) -> String {
    s.chars().map(|c| if c.is_control() { ' ' } else { c }).collect()
}

/// Words joined with ", ".
fn list(words: &[String]) -> String {
    words.join(", ")
}

#[cfg(test)]
mod tests;
