//! A machine in memory for the tests: a bw with a vault in JSON, files, sockets, a keyring
//! that answers like `secret-tool`, and canned answers for the other programs.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use super::Bw;
use super::BwError;
use super::Cmd;
use super::Host;
use super::Ran;
use super::Sensitive;
use super::Session;
use super::login::Login;

pub const SESSION: &str = "SESSION-KEY==";
pub const PASSWORD: &str = "hunter2-master";

/// One keyring entry: its label and its secret.
#[derive(Debug, Clone)]
pub struct Secret {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Fake {
    pub installed: bool,
    /// unauthenticated, locked.
    pub state: String,
    pub server: Option<String>,
    pub email: String,
    pub folders: String,
    pub items: String,
    /// The next login fails with this.
    pub login_error: Option<String>,
    /// Every bw call, secrets left out.
    pub bw: Vec<String>,
    /// What the logins got, for the checks that secrets went only there.
    pub logins: Vec<Login>,
    pub files: BTreeMap<PathBuf, (String, u32)>,
    pub dirs: BTreeMap<PathBuf, u32>,
    pub sockets: BTreeSet<PathBuf>,
    pub programs: BTreeSet<String>,
    /// Canned answers by the start of the command line.
    pub answers: Vec<(String, Ran)>,
    /// Every other program run.
    pub ran: Vec<Cmd>,
    /// (project, name) to the entry.
    pub keyring: BTreeMap<(String, String), Secret>,
}

impl Default for Fake {
    fn default() -> Self {
        Self::new()
    }
}

impl Fake {
    /// Logged out of no server yet, with an empty vault and every program installed.
    pub fn new() -> Self {
        Self {
            installed: true,
            state: "unauthenticated".into(),
            server: None,
            email: String::new(),
            folders: "[]".into(),
            items: "[]".into(),
            login_error: None,
            bw: vec![],
            logins: vec![],
            files: BTreeMap::new(),
            dirs: BTreeMap::new(),
            sockets: BTreeSet::new(),
            programs: ["ssh-add", "ssh-keygen", "git", "secret-tool", "gh", "timeout"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            answers: vec![],
            ran: vec![],
            keyring: BTreeMap::new(),
        }
    }

    /// Logged in and locked, on `server`.
    pub fn locked(mut self, server: &str, email: &str) -> Self {
        self.state = "locked".into();
        self.server = Some(server.into());
        self.email = email.into();
        self
    }

    pub fn vault(mut self, folders: &str, items: &str) -> Self {
        self.folders = folders.into();
        self.items = items.into();
        self
    }

    pub fn file(mut self, path: &str, contents: &str) -> Self {
        self.files.insert(PathBuf::from(path), (contents.into(), 0o644));
        self
    }

    pub fn answer(mut self, prefix: &str, code: i32, stdout: &str) -> Self {
        self.answers.push((
            prefix.into(),
            Ran {
                code: Some(code),
                stdout: stdout.into(),
                stderr: String::new(),
            },
        ));
        self
    }

    pub fn text(&self, path: &str) -> Option<&str> {
        self.files.get(Path::new(path)).map(|(s, _)| s.as_str())
    }

    pub fn mode(&self, path: &str) -> Option<u32> {
        self.files.get(Path::new(path)).map(|(_, m)| *m)
    }

    /// The command lines run, `KEY=value` env first.
    pub fn lines(&self) -> Vec<String> {
        self.ran.iter().map(Cmd::display).collect()
    }

    fn session_ok(&self, s: Option<&Session>) -> bool {
        self.state == "locked" && s.is_some_and(|s| s.expose() == SESSION)
    }

    fn need(&self, s: &Session) -> Result<(), BwError> {
        if self.session_ok(Some(s)) {
            Ok(())
        } else {
            Err(BwError::new("You are not logged in."))
        }
    }

    fn secret_tool(&mut self, args: &[String], stdin: Option<&Sensitive>) -> Ran {
        let attr = |k: &str| args.windows(2).find(|w| w[0] == k).map(|w| w[1].clone());
        let (project, name) = (attr("project"), attr("name"));
        let ours = attr("service").as_deref() == Some(super::env::SERVICE);
        let mut ran = Ran {
            code: Some(0),
            ..Ran::default()
        };
        match args.first().map(String::as_str) {
            Some("store") => {
                let label = args
                    .iter()
                    .find_map(|a| a.strip_prefix("--label="))
                    .unwrap_or_default()
                    .to_string();
                if let (true, Some(p), Some(n)) = (ours, project, name) {
                    let value = stdin.map(|s| s.0.clone()).unwrap_or_default();
                    self.keyring.insert((p, n), Secret { label, value });
                }
            }
            Some("clear") if ours => self.keyring.retain(|(p, _), _| project.as_ref().is_some_and(|w| w != p)),
            Some("search") => {
                let hits: Vec<&Secret> = self
                    .keyring
                    .iter()
                    .filter(|((p, _), _)| ours && project.as_ref().is_none_or(|w| w == p))
                    .map(|(_, s)| s)
                    .collect();
                if hits.is_empty() {
                    ran.code = Some(1);
                }
                for s in hits {
                    ran.stdout.push_str(&format!("[/1]\nlabel = {}\nsecret = {}\n", s.label, s.value));
                    ran.stderr.push_str("attribute.service = dotfiles-env\n");
                }
            }
            _ => {}
        }
        ran
    }
}

impl Bw for Fake {
    fn installed(&mut self) -> bool {
        self.installed
    }

    fn status(&mut self, session: Option<&Session>) -> Result<String, BwError> {
        self.bw.push("status".into());
        let status = if self.session_ok(session) { "unlocked" } else { self.state.as_str() };
        let server = self.server.as_ref().map_or("null".to_string(), |s| format!("\"{s}\""));
        Ok(format!(
            r#"{{"serverUrl":{server},"userEmail":"{}","status":"{status}"}}"#,
            self.email
        ))
    }

    fn set_server(&mut self, url: &str) -> Result<(), BwError> {
        self.bw.push(format!("config server {url}"));
        self.server = Some(url.into());
        Ok(())
    }

    fn login(&mut self, how: &Login) -> Result<Option<Session>, BwError> {
        self.bw.push("login".into());
        self.logins.push(how.clone());
        if let Some(e) = self.login_error.take() {
            return Err(BwError::new(e));
        }
        self.state = "locked".into();
        let session = Some(Session::new(SESSION));
        match how {
            Login::Totp { email } => {
                self.email.clone_from(email);
                Ok(session)
            }
            Login::Password { email, password, .. } => {
                if password.0 != PASSWORD {
                    self.state = "unauthenticated".into();
                    return Err(BwError::new("Username or password is incorrect. Try again."));
                }
                self.email.clone_from(email);
                Ok(session)
            }
            Login::ApiKeyPrompt | Login::Sso | Login::ApiKey { .. } => Ok(None),
        }
    }

    fn unlock(&mut self, password: Option<&Sensitive>) -> Result<Session, BwError> {
        self.bw.push("unlock".into());
        if self.state != "locked" {
            return Err(BwError::new("You are not logged in."));
        }
        if password.is_some_and(|p| p.0 != PASSWORD) {
            return Err(BwError::new("Invalid master password."));
        }
        Ok(Session::new(SESSION))
    }

    fn logout(&mut self) -> Result<(), BwError> {
        self.bw.push("logout".into());
        self.state = "unauthenticated".into();
        self.email.clear();
        Ok(())
    }

    fn sync(&mut self, session: &Session) -> Result<(), BwError> {
        self.bw.push("sync".into());
        self.need(session)
    }

    fn folders(&mut self, session: &Session) -> Result<String, BwError> {
        self.bw.push("list folders".into());
        self.need(session)?;
        Ok(self.folders.clone())
    }

    fn items(&mut self, session: &Session) -> Result<String, BwError> {
        self.bw.push("list items".into());
        self.need(session)?;
        Ok(self.items.clone())
    }
}

impl Host for Fake {
    fn read(&mut self, path: &Path) -> Option<String> {
        self.files.get(path).map(|(s, _)| s.clone())
    }

    fn write(&mut self, path: &Path, contents: &str, mode: u32) -> Result<(), String> {
        self.files.insert(path.to_path_buf(), (contents.into(), mode));
        Ok(())
    }

    fn mkdir(&mut self, path: &Path, mode: u32) -> Result<(), String> {
        self.dirs.insert(path.to_path_buf(), mode);
        Ok(())
    }

    fn remove(&mut self, path: &Path) {
        self.files.remove(path);
    }

    fn is_file(&mut self, path: &Path) -> bool {
        self.files.contains_key(path)
    }

    fn exists(&mut self, path: &Path) -> bool {
        self.files.contains_key(path) || self.dirs.contains_key(path) || self.sockets.contains(path)
    }

    fn is_socket(&mut self, path: &Path) -> bool {
        self.sockets.contains(path)
    }

    fn has(&mut self, program: &str) -> bool {
        self.programs.contains(program)
    }

    fn run(&mut self, cmd: &Cmd) -> Result<Ran, String> {
        self.ran.push(cmd.clone());
        if !self.programs.contains(&cmd.program) {
            return Err(format!("{}: not found", cmd.program));
        }
        if cmd.program == "secret-tool" {
            return Ok(self.secret_tool(&cmd.args, cmd.stdin.as_ref()));
        }
        let line = std::iter::once(cmd.program.as_str())
            .chain(cmd.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ");
        Ok(self
            .answers
            .iter()
            .rev()
            .find(|(prefix, _)| line.starts_with(prefix.as_str()))
            .map_or_else(
                || Ran {
                    code: Some(1),
                    ..Ran::default()
                },
                |(_, r)| r.clone(),
            ))
    }
}
