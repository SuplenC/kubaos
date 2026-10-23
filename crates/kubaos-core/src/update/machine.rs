//! The updater's seam to the machine: running programs as the user or as root with their output
//! captured, the home-manager switch, sudo's password and the answer to a question. The `kubaos`
//! command implements it on the real machine; the tests run the updater on a fake that runs git
//! and the migrations for real in a temporary folder.
//!
//! # Owns
//!
//! - [`Machine`] and [`Cmd`], which says how a root command goes through `sudo`.
//! - [`Lines`], which cuts a program's output into the lines the updater reports.
//!
//! # Not here
//!
//! - What the updater runs, and in which order: `updater.rs`. What it reports: `event.rs`.
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
    sudo_inside: bool,
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
            sudo_inside: false,
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

    /// Marks a user command that runs `sudo` itself, so sudo's password is asked before it starts
    /// rather than behind its captured output.
    #[must_use]
    pub fn runs_sudo(mut self) -> Self {
        self.sudo_inside = true;
        self
    }

    /// Whether it needs sudo: it runs as root, or runs `sudo` itself.
    pub fn uses_sudo(&self) -> bool {
        self.root || self.sudo_inside
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

/// What the updater needs from the machine. Nothing here writes to the terminal: the updater
/// reports what happens as events (`event.rs`).
pub trait Machine {
    /// Runs `cmd` to the end and returns its stdout.
    ///
    /// # Errors
    ///
    /// [`MachineError`] when it can't start or fails.
    fn output(&mut self, cmd: &Cmd) -> Result<String, MachineError>;

    /// Runs `cmd` to the end with no terminal input, handing each line it prints, on stdout or
    /// stderr, to `on_line` as it comes.
    ///
    /// # Errors
    ///
    /// [`MachineError`] when it can't start or fails.
    fn run(&mut self, cmd: &Cmd, on_line: &mut dyn FnMut(&str)) -> Result<(), MachineError>;

    /// Switches the user's home to the user flake, through the switch queue, handing each line of
    /// the switch's output to `on_line`.
    ///
    /// # Errors
    ///
    /// [`MachineError::Switch`] when the switch fails.
    fn switch(&mut self, on_line: &mut dyn FnMut(&str)) -> Result<(), MachineError>;

    /// Reads the answer to `question`, yes by default. The updater has reported the question as a
    /// prompt before it asks.
    fn confirm(&mut self, question: &str) -> bool;

    /// Whether sudo would ask for a password before a root command now.
    fn needs_password(&mut self) -> bool;

    /// Has sudo ask for the password on the terminal, so the root commands after it run without
    /// asking.
    ///
    /// # Errors
    ///
    /// [`MachineError`] when sudo can't start or the password is refused.
    fn authenticate(&mut self) -> Result<(), MachineError>;

    /// The bytes free for the user on the filesystem of `path`.
    ///
    /// # Errors
    ///
    /// When the filesystem can't be asked.
    fn free_bytes(&self, path: &Path) -> io::Result<u64>;
}

/// Cuts a program's output into lines as it arrives, at `\n` and at `\r`, with which apt and nix
/// redraw a progress line. Empty lines are dropped.
#[derive(Debug, Default)]
pub struct Lines {
    pending: Vec<u8>,
}

impl Lines {
    /// Takes the next bytes of the output and hands each line they finish to `on_line`.
    pub fn push(&mut self, bytes: &[u8], mut on_line: impl FnMut(&str)) {
        for &byte in bytes {
            if byte == b'\n' || byte == b'\r' {
                self.flush(&mut on_line);
            } else {
                self.pending.push(byte);
            }
        }
    }

    /// Hands the last line, one the output ended without a newline, to `on_line`.
    pub fn finish(&mut self, mut on_line: impl FnMut(&str)) {
        self.flush(&mut on_line);
    }

    fn flush(&mut self, on_line: &mut impl FnMut(&str)) {
        if !self.pending.is_empty() {
            on_line(&String::from_utf8_lossy(&self.pending));
            self.pending.clear();
        }
    }
}

#[cfg(test)]
mod tests;
