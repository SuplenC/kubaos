//! The machine itself: the `bw` CLI, the files, and the other programs.

use std::io::Write as _;
use std::os::unix::fs::FileTypeExt as _;
use std::os::unix::fs::OpenOptionsExt as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use super::Bw;
use super::BwError;
use super::Cmd;
use super::Host;
use super::Ran;
use super::Sensitive;
use super::Session;
use super::login::Login;

/// Where `system/70-bitwarden.sh` installs bw.
const BW: &str = "/usr/local/bin/bw";

/// bw and the host, for real.
#[derive(Debug, Clone)]
pub struct Real {
    bw: PathBuf,
}

impl Default for Real {
    fn default() -> Self {
        Self::new()
    }
}

impl Real {
    /// `/usr/local/bin/bw`, else the `bw` on `PATH`.
    #[must_use]
    pub fn new() -> Self {
        let bw = if Path::new(BW).exists() {
            PathBuf::from(BW)
        } else {
            find("bw").unwrap_or_else(|| PathBuf::from(BW))
        };
        Self { bw }
    }

    /// A bw that never prompts, with the session only in its environment. On failure the
    /// error is bw's stderr, without node's deprecation noise.
    fn quiet(&self, args: &[&str], session: Option<&Session>, env: &[(&str, &str)]) -> Result<String, BwError> {
        let mut cmd = Command::new(&self.bw);
        cmd.args(args)
            .env("BW_NOINTERACTION", "true")
            .env_remove("BW_SESSION")
            .stdin(Stdio::null());
        if let Some(s) = session {
            cmd.env("BW_SESSION", s.expose());
        }
        cmd.envs(env.iter().copied());
        let out = cmd
            .output()
            .map_err(|e| BwError::new(format!("can't run {}: {e}", self.bw.display())))?;
        if out.status.success() {
            return Ok(String::from_utf8_lossy(&out.stdout).into_owned());
        }
        let stderr = String::from_utf8_lossy(&out.stderr);
        let said: Vec<&str> = stderr
            .lines()
            .filter(|l| {
                let l = l.to_ascii_lowercase();
                !l.contains("punycode") && !l.contains("trace-deprecation") && !l.trim().is_empty()
            })
            .collect();
        Err(BwError::new(if said.is_empty() {
            format!("bw {} failed ({})", args.first().unwrap_or(&""), out.status)
        } else {
            said.join("\n")
        }))
    }

    /// A bw that asks on the terminal: stdin and stderr are the person's, stdout is ours.
    fn interactive(&self, args: &[&str]) -> Result<String, BwError> {
        let out = Command::new(&self.bw)
            .args(args)
            .env_remove("BW_SESSION")
            .stdin(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdout(Stdio::piped())
            .output()
            .map_err(|e| BwError::new(format!("can't run {}: {e}", self.bw.display())))?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).into_owned())
        } else {
            Err(BwError::new(format!(
                "bw {} failed ({})",
                args.first().unwrap_or(&""),
                out.status
            )))
        }
    }
}

fn session_of(stdout: &str) -> Option<Session> {
    let s = stdout.trim();
    (!s.is_empty()).then(|| Session::new(s))
}

impl Bw for Real {
    fn installed(&mut self) -> bool {
        self.bw.is_file()
    }

    fn status(&mut self, session: Option<&Session>) -> Result<String, BwError> {
        self.quiet(&["status"], session, &[])
    }

    fn set_server(&mut self, url: &str) -> Result<(), BwError> {
        self.quiet(&["config", "server", url], None, &[]).map(drop)
    }

    fn login(&mut self, how: &Login) -> Result<Option<Session>, BwError> {
        match how {
            Login::Totp { email } => self
                .interactive(&["login", email, "--method", "0", "--raw"])
                .map(|o| session_of(&o)),
            Login::ApiKeyPrompt => self.interactive(&["login", "--apikey"]).map(|_| None),
            Login::Sso => self.interactive(&["login", "--sso"]).map(|_| None),
            Login::Password {
                email,
                password,
                method,
                code,
            } => {
                let args = [
                    "login",
                    email,
                    "--passwordenv",
                    "BW_PASSWORD",
                    "--method",
                    method,
                    "--code",
                    code,
                    "--raw",
                ];
                self.quiet(&args, None, &[("BW_PASSWORD", &password.0)])
                    .map(|o| session_of(&o))
            }
            Login::ApiKey {
                client_id,
                client_secret,
            } => {
                let env = [
                    ("BW_CLIENTID", client_id.as_str()),
                    ("BW_CLIENTSECRET", client_secret.0.as_str()),
                ];
                self.quiet(&["login", "--apikey"], None, &env).map(|_| None)
            }
        }
    }

    fn unlock(&mut self, password: Option<&Sensitive>) -> Result<Session, BwError> {
        let out = match password {
            Some(pw) => self.quiet(
                &["unlock", "--passwordenv", "BW_PASSWORD", "--raw"],
                None,
                &[("BW_PASSWORD", &pw.0)],
            )?,
            None => self.interactive(&["unlock", "--raw"])?,
        };
        session_of(&out).ok_or_else(|| BwError::new("bw unlock printed no session"))
    }

    fn logout(&mut self) -> Result<(), BwError> {
        self.quiet(&["logout"], None, &[]).map(drop)
    }

    fn sync(&mut self, session: &Session) -> Result<(), BwError> {
        self.quiet(&["sync"], Some(session), &[]).map(drop)
    }

    fn folders(&mut self, session: &Session) -> Result<String, BwError> {
        self.quiet(&["list", "folders"], Some(session), &[])
    }

    fn items(&mut self, session: &Session) -> Result<String, BwError> {
        self.quiet(&["list", "items"], Some(session), &[])
    }
}

/// The program on `PATH`.
fn find(program: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|d| d.join(program))
        .find(|p| std::fs::metadata(p).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0))
}

impl Host for Real {
    fn read(&mut self, path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    fn write(&mut self, path: &Path, contents: &str, mode: u32) -> Result<(), String> {
        let dir = path.parent().ok_or("no parent directory")?;
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let tmp = dir.join(format!(".{name}.kubaos-{}", std::process::id()));
        let _ = std::fs::remove_file(&tmp);
        let written = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(mode)
            .open(&tmp)
            .and_then(|mut f| {
                f.write_all(contents.as_bytes())?;
                // The umask may have taken bits from the mode `open` was given.
                f.set_permissions(std::fs::Permissions::from_mode(mode))?;
                f.sync_all()
            })
            .and_then(|()| std::fs::rename(&tmp, path));
        if let Err(e) = written {
            let _ = std::fs::remove_file(&tmp);
            return Err(e.to_string());
        }
        Ok(())
    }

    fn mkdir(&mut self, path: &Path, mode: u32) -> Result<(), String> {
        std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).map_err(|e| e.to_string())
    }

    fn remove(&mut self, path: &Path) {
        let _ = std::fs::remove_file(path);
    }

    fn is_file(&mut self, path: &Path) -> bool {
        path.is_file()
    }

    fn exists(&mut self, path: &Path) -> bool {
        path.exists()
    }

    fn is_socket(&mut self, path: &Path) -> bool {
        std::fs::metadata(path).is_ok_and(|m| m.file_type().is_socket())
    }

    fn has(&mut self, program: &str) -> bool {
        find(program).is_some()
    }

    fn run(&mut self, cmd: &Cmd) -> Result<Ran, String> {
        let mut c = Command::new(&cmd.program);
        c.args(&cmd.args).env_remove("BW_SESSION");
        c.envs(cmd.env.iter().map(|(k, v)| (k, v)));
        c.stdin(if cmd.stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
        let mut child = c.spawn().map_err(|e| format!("can't run {}: {e}", cmd.program))?;
        if let (Some(input), Some(mut stdin)) = (&cmd.stdin, child.stdin.take()) {
            stdin
                .write_all(input.0.as_bytes())
                .map_err(|e| format!("{}: stdin: {e}", cmd.program))?;
        }
        let out = child.wait_with_output().map_err(|e| e.to_string())?;
        Ok(Ran {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
    }
}
