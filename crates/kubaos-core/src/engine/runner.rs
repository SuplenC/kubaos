//! Every command and every write to the target goes through `Runner`, so a
//! dry run prints instead of acting and tests can record what would happen.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

/// Set by the CLI's SIGINT/SIGTERM handler. The system runner refuses new
/// commands and kills a streaming one once this is set; the engine then
/// cleans up and reports `failed`.
pub static INTERRUPTED: AtomicBool = AtomicBool::new(false);

pub fn interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}

/// A string that never shows up in logs, `Debug` output or argv.
#[derive(Clone)]
pub struct Sensitive(pub String);

impl std::fmt::Debug for Sensitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<secret>")
    }
}

#[derive(Debug, Clone, Default)]
pub struct Cmd {
    pub program: String,
    pub args: Vec<String>,
    /// Fed to the child's stdin verbatim, then stdin is closed.
    pub stdin: Option<Sensitive>,
    pub env: Vec<(String, String)>,
    /// What a dry run returns as stdout, for commands whose output the next
    /// command needs (`blkid` for UUIDs, say).
    pub placeholder: Option<String>,
}

impl Cmd {
    pub fn new(program: impl Into<String>) -> Self {
        Cmd {
            program: program.into(),
            ..Default::default()
        }
    }

    pub fn arg(mut self, a: impl Into<String>) -> Self {
        self.args.push(a.into());
        self
    }

    pub fn args<I, S>(mut self, it: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(it.into_iter().map(Into::into));
        self
    }

    pub fn secret_stdin(mut self, s: &str) -> Self {
        self.stdin = Some(Sensitive(s.to_string()));
        self
    }

    pub fn env(mut self, k: &str, v: &str) -> Self {
        self.env.push((k.into(), v.into()));
        self
    }

    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = Some(p.into());
        self
    }

    /// Shell-quoted command line; stdin shows only as a marker.
    pub fn display(&self) -> String {
        let mut s = String::new();
        for (k, v) in &self.env {
            s.push_str(&format!("{k}={} ", quote(v)));
        }
        s.push_str(&quote(&self.program));
        for a in &self.args {
            s.push(' ');
            s.push_str(&quote(a));
        }
        if self.stdin.is_some() {
            s.push_str(" <<< (secret on stdin)");
        }
        s
    }
}

pub fn quote(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./=:,@+%^".contains(c))
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', r"'\''"))
    }
}

#[derive(Debug, Clone, Default)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stream {
    Line(String),
    /// Sent every 250 ms while the command runs, so callers can keep their
    /// progress events flowing even when the command is quiet.
    Tick,
}

pub trait Runner {
    fn is_dry_run(&self) -> bool;
    /// Run to completion. A non-zero exit is an error carrying the tail of
    /// stderr.
    fn run(&mut self, cmd: &Cmd) -> Result<Output, String>;
    /// Run, handing every stdout/stderr line (split on `\n` and `\r`) to `f`.
    fn stream(&mut self, cmd: &Cmd, f: &mut dyn FnMut(Stream)) -> Result<(), String>;
    /// Start in the background; returns a handle for `kill`.
    fn spawn(&mut self, cmd: &Cmd) -> Result<u32, String>;
    fn kill(&mut self, handle: u32);
    fn write_file(&mut self, path: &Path, contents: &[u8], mode: u32) -> Result<(), String>;
    fn mkdir_p(&mut self, path: &Path) -> Result<(), String>;
    /// Reads a file (on the target, mostly). `None` when it doesn't exist or,
    /// in a dry run, when it isn't there yet.
    fn read_file(&mut self, path: &Path) -> Option<String>;
    fn exists(&mut self, path: &Path) -> bool;
    /// Where the symlink at `path` points (unresolved). `None` when `path`
    /// isn't a symlink or doesn't exist.
    fn read_link(&mut self, _path: &Path) -> Option<PathBuf> {
        None
    }
}

// ---------------------------------------------------------------------------

/// Runs things for real.
#[derive(Default)]
pub struct SystemRunner {
    children: HashMap<u32, Child>,
}

impl SystemRunner {
    pub fn new() -> Self {
        Self::default()
    }

    fn command(cmd: &Cmd) -> Command {
        let mut c = Command::new(&cmd.program);
        c.args(&cmd.args);
        for (k, v) in &cmd.env {
            c.env(k, v);
        }
        c.stdin(if cmd.stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        });
        c
    }

    fn feed_stdin(child: &mut Child, cmd: &Cmd) -> Result<(), String> {
        if let (Some(secret), Some(mut stdin)) = (&cmd.stdin, child.stdin.take()) {
            stdin
                .write_all(secret.0.as_bytes())
                .map_err(|e| format!("{}: writing stdin: {e}", cmd.program))?;
        }
        Ok(())
    }
}

fn check_interrupted() -> Result<(), String> {
    if interrupted() {
        Err("interrupted".into())
    } else {
        Ok(())
    }
}

fn tail(s: &str, lines: usize) -> String {
    let v: Vec<&str> = s.lines().filter(|l| !l.trim().is_empty()).collect();
    v[v.len().saturating_sub(lines)..].join("\n")
}

impl Runner for SystemRunner {
    fn is_dry_run(&self) -> bool {
        false
    }

    fn run(&mut self, cmd: &Cmd) -> Result<Output, String> {
        check_interrupted()?;
        let mut child = Self::command(cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("{}: {e}", cmd.program))?;
        Self::feed_stdin(&mut child, cmd)?;
        let out = child
            .wait_with_output()
            .map_err(|e| format!("{}: {e}", cmd.program))?;
        let o = Output {
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        };
        if out.status.success() {
            Ok(o)
        } else {
            let detail = tail(&format!("{}\n{}", o.stdout, o.stderr), 6);
            Err(format!(
                "{} exited with {}: {}",
                cmd.program, out.status, detail
            ))
        }
    }

    fn stream(&mut self, cmd: &Cmd, f: &mut dyn FnMut(Stream)) -> Result<(), String> {
        check_interrupted()?;
        let mut child = Self::command(cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("{}: {e}", cmd.program))?;
        Self::feed_stdin(&mut child, cmd)?;
        let (tx, rx) = mpsc::channel::<Option<String>>();
        let mut readers = Vec::new();
        for pipe in [
            child
                .stdout
                .take()
                .map(|p| Box::new(p) as Box<dyn Read + Send>),
            child
                .stderr
                .take()
                .map(|p| Box::new(p) as Box<dyn Read + Send>),
        ]
        .into_iter()
        .flatten()
        {
            let tx = tx.clone();
            readers.push(std::thread::spawn(move || read_lines(pipe, &tx)));
        }
        drop(tx);
        let mut open = readers.len();
        let mut last: Vec<String> = Vec::new();
        while open > 0 {
            match rx.recv_timeout(Duration::from_millis(250)) {
                Ok(Some(line)) => {
                    last.push(line.clone());
                    if last.len() > 6 {
                        last.remove(0);
                    }
                    f(Stream::Line(line));
                }
                Ok(None) => open -= 1,
                Err(mpsc::RecvTimeoutError::Timeout) => f(Stream::Tick),
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
            if interrupted() {
                let _ = child.kill();
            }
        }
        for r in readers {
            let _ = r.join();
        }
        let status = child.wait().map_err(|e| format!("{}: {e}", cmd.program))?;
        check_interrupted()?;
        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "{} exited with {status}: {}",
                cmd.program,
                last.join("\n")
            ))
        }
    }

    fn spawn(&mut self, cmd: &Cmd) -> Result<u32, String> {
        check_interrupted()?;
        let mut child = Self::command(cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("{}: {e}", cmd.program))?;
        Self::feed_stdin(&mut child, cmd)?;
        let id = child.id();
        self.children.insert(id, child);
        Ok(id)
    }

    fn kill(&mut self, handle: u32) {
        if let Some(mut c) = self.children.remove(&handle) {
            // SIGTERM first so a daemon can clean up its socket.
            let _ = Command::new("kill").arg(handle.to_string()).status();
            for _ in 0..20 {
                if matches!(c.try_wait(), Ok(Some(_))) {
                    return;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let _ = c.kill();
            let _ = c.wait();
        }
    }

    fn write_file(&mut self, path: &Path, contents: &[u8], mode: u32) -> Result<(), String> {
        use std::os::unix::fs::PermissionsExt;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        // Create with the final mode right away: a 0600 file is never
        // briefly world-readable.
        let tmp = path.with_extension("dotinstall-tmp");
        {
            use std::os::unix::fs::OpenOptionsExt;
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(mode)
                .open(&tmp)
                .map_err(|e| format!("{}: {e}", tmp.display()))?;
            f.write_all(contents)
                .map_err(|e| format!("{}: {e}", tmp.display()))?;
        }
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(mode))
            .map_err(|e| format!("{}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))
    }

    fn mkdir_p(&mut self, path: &Path) -> Result<(), String> {
        std::fs::create_dir_all(path).map_err(|e| format!("{}: {e}", path.display()))
    }

    fn read_file(&mut self, path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    fn exists(&mut self, path: &Path) -> bool {
        path.symlink_metadata().is_ok()
    }

    fn read_link(&mut self, path: &Path) -> Option<PathBuf> {
        std::fs::read_link(path).ok()
    }
}

fn read_lines(mut pipe: Box<dyn Read + Send>, tx: &mpsc::Sender<Option<String>>) {
    let mut buf = [0u8; 8192];
    let mut line = Vec::new();
    loop {
        match pipe.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                for &b in &buf[..n] {
                    if b == b'\n' || b == b'\r' {
                        if !line.is_empty() {
                            let _ = tx.send(Some(String::from_utf8_lossy(&line).into_owned()));
                            line.clear();
                        }
                    } else {
                        line.push(b);
                    }
                }
            }
        }
    }
    if !line.is_empty() {
        let _ = tx.send(Some(String::from_utf8_lossy(&line).into_owned()));
    }
    let _ = tx.send(None);
}

// ---------------------------------------------------------------------------

/// What a dry run or a test did, in order.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Run(String),
    Spawn(String),
    Kill(u32),
    Write {
        path: PathBuf,
        mode: u32,
        contents: String,
    },
    Mkdir(PathBuf),
}

/// Touches nothing. Records every action; commands "succeed" with their
/// placeholder as stdout, or with a canned response a test registered.
#[derive(Default)]
pub struct DryRunner {
    pub actions: Vec<Action>,
    /// (program + args prefix, stdout). First match wins.
    pub responses: Vec<(String, String)>,
    /// Files `read_file` returns (tests); written files are readable too.
    pub files: HashMap<PathBuf, String>,
    /// Commands (by a substring of their display) that fail, for cleanup tests.
    pub fail_on: Vec<String>,
}

impl DryRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn commands(&self) -> Vec<String> {
        self.actions
            .iter()
            .filter_map(|a| match a {
                Action::Run(c) | Action::Spawn(c) => Some(c.clone()),
                _ => None,
            })
            .collect()
    }

    fn respond(&self, cmd: &Cmd) -> Result<Output, String> {
        let shown = cmd.display();
        if self.fail_on.iter().any(|p| shown.contains(p.as_str())) {
            return Err(format!("{} failed (test)", cmd.program));
        }
        let stdout = self
            .responses
            .iter()
            .find(|(p, _)| shown.starts_with(p.as_str()))
            .map(|(_, out)| out.clone())
            .or_else(|| cmd.placeholder.clone())
            .unwrap_or_default();
        Ok(Output {
            stdout,
            stderr: String::new(),
        })
    }
}

impl Runner for DryRunner {
    fn is_dry_run(&self) -> bool {
        true
    }

    fn run(&mut self, cmd: &Cmd) -> Result<Output, String> {
        self.actions.push(Action::Run(cmd.display()));
        self.respond(cmd)
    }

    fn stream(&mut self, cmd: &Cmd, f: &mut dyn FnMut(Stream)) -> Result<(), String> {
        self.actions.push(Action::Run(cmd.display()));
        let out = self.respond(cmd)?;
        for l in out.stdout.lines() {
            f(Stream::Line(l.to_string()));
        }
        Ok(())
    }

    fn spawn(&mut self, cmd: &Cmd) -> Result<u32, String> {
        self.actions.push(Action::Spawn(cmd.display()));
        self.respond(cmd).map(|_| 4242)
    }

    fn kill(&mut self, handle: u32) {
        self.actions.push(Action::Kill(handle));
    }

    fn write_file(&mut self, path: &Path, contents: &[u8], mode: u32) -> Result<(), String> {
        let contents = String::from_utf8_lossy(contents).into_owned();
        self.files.insert(path.to_path_buf(), contents.clone());
        self.actions.push(Action::Write {
            path: path.to_path_buf(),
            mode,
            contents,
        });
        Ok(())
    }

    fn mkdir_p(&mut self, path: &Path) -> Result<(), String> {
        self.actions.push(Action::Mkdir(path.to_path_buf()));
        Ok(())
    }

    fn read_file(&mut self, path: &Path) -> Option<String> {
        self.files.get(path).cloned()
    }

    fn exists(&mut self, path: &Path) -> bool {
        self.files.contains_key(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_quotes_and_hides_stdin() {
        let c = Cmd::new("cryptsetup")
            .args(["open", "--key-file=-", "/dev/sda3", "cryptroot"])
            .secret_stdin("hunter2");
        let d = c.display();
        assert_eq!(
            d,
            "cryptsetup open --key-file=- /dev/sda3 cryptroot <<< (secret on stdin)"
        );
        assert!(!format!("{c:?}").contains("hunter2"));
        assert_eq!(quote("a b"), "'a b'");
        assert_eq!(quote("it's"), r"'it'\''s'");
        assert_eq!(quote(""), "''");
    }

    #[test]
    fn dry_runner_records_and_answers() {
        let mut r = DryRunner::new();
        r.responses.push(("blkid".into(), "abcd\n".into()));
        let out = r.run(&Cmd::new("blkid").arg("/dev/x")).unwrap();
        assert_eq!(out.stdout, "abcd\n");
        let out = r.run(&Cmd::new("other").placeholder("<p>")).unwrap();
        assert_eq!(out.stdout, "<p>");
        assert_eq!(r.commands(), vec!["blkid /dev/x", "other"]);
    }
}
