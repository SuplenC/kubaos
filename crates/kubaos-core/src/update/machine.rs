//! The updater's seam to the machine: running programs as the user or as root, the home-manager
//! switch, asking the user, and what the user reads. The `kubaos` command implements it on the
//! real machine; the tests run the updater on a fake that runs git and the migrations for real in
//! a temporary folder.
//!
//! # Owns
//!
//! - [`Machine`] and [`Cmd`], which says how a root command goes through `sudo`.
//!
//! # Not here
//!
//! - What the updater runs, and in which order: `updater.rs`.
//! - The real machine, and the switch through `switch.rs`'s queue: `kubaos-cli`'s `update.rs`.

use std::ffi::OsStr;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// Where Nix's own tools are when the login shell hasn't put them on `PATH`.
pub const NIX_BIN: &str = "/nix/var/nix/profiles/default/bin";

/// One program to run, as the user or as root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cmd {
    argv: Vec<OsString>,
    root: bool,
    stdin: Option<Vec<u8>>,
    env: Vec<(String, OsString)>,
    cwd: Option<PathBuf>,
}

impl Cmd {
    /// `program`, run as the user.
    #[must_use]
    pub fn user(program: impl AsRef<OsStr>) -> Self {
        Self {
            argv: vec![program.as_ref().to_owned()],
            root: false,
            stdin: None,
            env: Vec::new(),
            cwd: None,
        }
    }

    /// `program`, run as root.
    #[must_use]
    pub fn root(program: impl AsRef<OsStr>) -> Self {
        Self {
            root: true,
            ..Self::user(program)
        }
    }

    /// Adds one argument.
    #[must_use]
    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.argv.push(arg.as_ref().to_owned());
        self
    }

    /// Adds arguments.
    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.argv.extend(args.into_iter().map(|a| a.as_ref().to_owned()));
        self
    }

    /// Feeds `bytes` to its stdin.
    #[must_use]
    pub fn stdin(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.stdin = Some(bytes.into());
        self
    }

    /// Sets an environment variable for it.
    #[must_use]
    pub fn env(mut self, name: impl Into<String>, value: impl AsRef<OsStr>) -> Self {
        self.env.push((name.into(), value.as_ref().to_owned()));
        self
    }

    /// Runs it in `dir`.
    #[must_use]
    pub fn cwd(mut self, dir: impl Into<PathBuf>) -> Self {
        self.cwd = Some(dir.into());
        self
    }

    /// The program and its arguments, without `sudo`.
    pub fn argv(&self) -> &[OsString] {
        &self.argv
    }

    /// The program's name, for messages.
    pub fn program(&self) -> String {
        self.argv
            .first()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// Whether it runs as root.
    pub fn is_root(&self) -> bool {
        self.root
    }

    /// What it feeds to stdin.
    pub fn input(&self) -> Option<&[u8]> {
        self.stdin.as_deref()
    }

    /// Its environment variables.
    pub fn vars(&self) -> &[(String, OsString)] {
        &self.env
    }

    /// The folder it runs in.
    pub fn dir(&self) -> Option<&Path> {
        self.cwd.as_deref()
    }

    /// The command as a line, `sudo` first for a root one.
    pub fn display(&self) -> String {
        let words: Vec<String> = self.argv.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        let sudo = if self.root { "sudo " } else { "" };
        format!("{sudo}{}", words.join(" "))
    }

    /// The `std` command that runs it: through `sudo env` for a root one, unless this process is
    /// root already (`as_root`). `path` replaces `PATH` for a user one.
    #[must_use]
    pub fn command(&self, as_root: bool, path: Option<&OsStr>) -> Command {
        let sudo = self.root && !as_root;
        let mut argv: Vec<OsString> = Vec::new();
        if sudo {
            argv.push("sudo".into());
        }
        if self.root && !self.env.is_empty() {
            // HAZARD: sudo resets the environment, so a root command's variables go through
            // `env` on its command line, after sudo.
            argv.push("env".into());
            for (name, value) in &self.env {
                let mut pair = OsString::from(format!("{name}="));
                pair.push(value);
                argv.push(pair);
            }
        }
        argv.extend(self.argv.iter().cloned());
        let mut command = Command::new(&argv[0]);
        command.args(&argv[1..]);
        if !self.root {
            command.envs(self.env.iter().map(|(k, v)| (k, v)));
            if let Some(path) = path {
                command.env("PATH", path);
            }
        }
        if let Some(dir) = &self.cwd {
            command.current_dir(dir);
        }
        command
    }
}

/// A program the updater ran failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MachineError {
    /// It could not be started.
    #[error("starting {program}")]
    Spawn {
        program: String,
        #[source]
        source: io::Error,
    },
    /// It ran and failed, or was killed (no code).
    #[error("`{command}` exited with {}{}", code.map_or_else(|| "a signal".to_owned(), |c| format!("code {c}")),
        if stderr.is_empty() { String::new() } else { format!(": {stderr}") })]
    Exited {
        command: String,
        code: Option<i32>,
        stderr: String,
    },
    /// The home-manager switch failed.
    #[error("the home-manager switch failed: {0}")]
    Switch(String),
}

impl MachineError {
    /// `cmd` exited with `code`; `stderr` is the end of what it printed there.
    #[must_use]
    pub fn exited(cmd: &Cmd, code: Option<i32>, stderr: &str) -> Self {
        let lines: Vec<&str> = stderr.trim().lines().collect();
        let tail = lines[lines.len().saturating_sub(5)..].join("\n");
        Self::Exited {
            command: cmd.display(),
            code,
            stderr: tail,
        }
    }
}

/// What the updater needs from the machine.
pub trait Machine {
    /// Runs `cmd` to the end and returns its stdout.
    ///
    /// # Errors
    ///
    /// [`MachineError`] when it can't start or fails.
    fn output(&mut self, cmd: &Cmd) -> Result<String, MachineError>;

    /// Runs `cmd` to the end with its output on the terminal.
    ///
    /// # Errors
    ///
    /// [`MachineError`] when it can't start or fails.
    fn stream(&mut self, cmd: &Cmd) -> Result<(), MachineError>;

    /// Switches the user's home to the user flake, through the switch queue.
    ///
    /// # Errors
    ///
    /// [`MachineError::Switch`] when the switch fails.
    fn switch(&mut self) -> Result<(), MachineError>;

    /// Asks `question`, yes by default; whether the answer is yes.
    fn confirm(&mut self, question: &str) -> bool;

    /// The bytes free for the user on the filesystem of `path`.
    ///
    /// # Errors
    ///
    /// When the filesystem can't be asked.
    fn free_bytes(&self, path: &Path) -> io::Result<u64>;

    /// Tells the user one line.
    fn say(&mut self, line: &str);

    /// Warns the user with one line.
    fn warn(&mut self, line: &str);
}

#[cfg(test)]
mod tests;
