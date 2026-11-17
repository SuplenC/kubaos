//! The root half of an app's install: the requests the user's side sends the root helper
//! (`kubaos-apps serve`), and the helper's answers.
//!
//! A request names an app id, a setup id and a step number (or, for an adoption, whether KubaOS
//! put the app there), nothing else. The helper reads the
//! action from the root-owned catalog, refuses one that isn't root's, and undoes only what root's
//! journal, which only root writes, recorded. One JSON line a request, one a reply; the helper
//! says `ready` once it runs, which is how the user's side knows the password was taken.
//!
//! # Owns
//!
//! - [`Request`], [`Reply`], [`RootSide`] (the user's end), [`ProcessRoot`] (the helper as a child
//!   process) and [`Server`] (the helper's end), with its refusals.
//!
//! # Not here
//!
//! - The steps themselves: `engine.rs`'s `Keeper`. The binary: `kubaos-cli`'s `apps_root.rs`.

use std::ffi::OsString;
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;

use serde::Deserialize;
use serde::Serialize;

use super::engine::Keeper;
use super::engine::Outcome;
use super::engine::StepError;
use super::folder::is_app_id;
use crate::update::event::chain;

/// What the user's side asks root to do.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    /// Whether the install's root steps may write what they write.
    Preflight {
        app: String,
        #[serde(default)]
        setup: Option<String>,
    },
    /// One root step of the install.
    Step {
        app: String,
        #[serde(default)]
        setup: Option<String>,
        step: usize,
        windows: bool,
    },
    /// One step of root's journal of the app, undone.
    Undo { app: String, step: usize },
    /// A root step this run undid, done again for a removal rolled back, from root's own record
    /// of the undo.
    Redo {
        app: String,
        #[serde(default)]
        setup: Option<String>,
        step: usize,
    },
    /// Root's steps of an app the machine had before it had a journal, recorded from what their
    /// checks find, nothing run: owned when KubaOS put the app there.
    Adopt {
        app: String,
        #[serde(default)]
        setup: Option<String>,
        owned: bool,
    },
    /// The vendor repository a settled `apt` step of root's journal of the app set up, set up
    /// again, so a key that nears its expiry is fetched again.
    Refresh { app: String, step: usize },
}

/// Root's answer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reply", rename_all = "snake_case")]
pub enum Reply {
    /// The helper runs, or a preflight found nothing in the way.
    Ready,
    Step {
        outcome: Outcome,
        /// What the user has to know of the step: what it left to them, what its undo kept.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        notes: Vec<String>,
    },
    /// The request isn't one the helper takes, or the install may not write what it writes.
    Refused {
        why: Vec<String>,
    },
    Failed {
        why: String,
    },
}

/// The root helper couldn't be reached.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RootError {
    #[error("the password wasn't given")]
    Declined,
    #[error("starting {program}")]
    Spawn {
        program: String,
        #[source]
        source: io::Error,
    },
    #[error("it stopped answering")]
    Lost(#[source] io::Error),
    #[error("its answer isn't one")]
    Garbled(#[source] serde_json::Error),
    #[error("it isn't running")]
    NotStarted,
    #[error("no program was named to start it")]
    NoProgram,
}

/// The user's end of the root helper.
pub trait RootSide {
    /// Starts it: the one password prompt of a run.
    ///
    /// # Errors
    ///
    /// [`RootError::Declined`] when the user said no, else why it couldn't start.
    fn start(&mut self) -> Result<(), RootError>;

    /// Sends it `request`.
    ///
    /// # Errors
    ///
    /// [`RootError`] when it can't be reached.
    fn call(&mut self, request: &Request) -> Result<Reply, RootError>;
}

/// The root helper as a child process: `pkexec /usr/local/libexec/kubaos-apps serve`, or the same
/// through sudo.
#[derive(Debug)]
pub struct ProcessRoot {
    argv: Vec<OsString>,
    running: Option<(Child, ChildStdin, BufReader<ChildStdout>)>,
}

/// The exit codes pkexec gives a prompt that was dismissed (126) or failed (127).
const PKEXEC_DECLINED: [i32; 2] = [126, 127];

impl ProcessRoot {
    /// The helper `argv` runs, not yet started.
    #[must_use]
    pub fn new(argv: Vec<OsString>) -> Self {
        Self { argv, running: None }
    }
}

impl RootSide for ProcessRoot {
    fn start(&mut self) -> Result<(), RootError> {
        let Some((first, args)) = self.argv.split_first() else {
            return Err(RootError::NoProgram);
        };
        let program = first.to_string_lossy().into_owned();
        let mut child = Command::new(first)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|source| RootError::Spawn { program, source })?;
        let (Some(stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
            return Err(RootError::NotStarted);
        };
        let mut stdout = BufReader::new(stdout);
        let mut line = String::new();
        let read = stdout.read_line(&mut line).map_err(RootError::Lost)?;
        if read == 0 {
            let status = child.wait().map_err(RootError::Lost)?;
            return Err(match status.code() {
                Some(code) if PKEXEC_DECLINED.contains(&code) => RootError::Declined,
                _ => RootError::Lost(io::Error::other(format!("it exited with {status}"))),
            });
        }
        match serde_json::from_str(&line).map_err(RootError::Garbled)? {
            Reply::Ready => {
                self.running = Some((child, stdin, stdout));
                Ok(())
            }
            _ => Err(RootError::NotStarted),
        }
    }

    fn call(&mut self, request: &Request) -> Result<Reply, RootError> {
        let (_, stdin, stdout) = self.running.as_mut().ok_or(RootError::NotStarted)?;
        let mut line = serde_json::to_string(request).map_err(RootError::Garbled)?;
        line.push('\n');
        stdin.write_all(line.as_bytes()).map_err(RootError::Lost)?;
        stdin.flush().map_err(RootError::Lost)?;
        let mut answer = String::new();
        if stdout.read_line(&mut answer).map_err(RootError::Lost)? == 0 {
            return Err(RootError::Lost(io::Error::from(io::ErrorKind::UnexpectedEof)));
        }
        serde_json::from_str(&answer).map_err(RootError::Garbled)
    }
}

impl Drop for ProcessRoot {
    fn drop(&mut self) {
        if let Some((mut child, stdin, _)) = self.running.take() {
            drop(stdin);
            let _ = child.wait();
        }
    }
}

/// The root helper's end: requests in, replies out, each checked against the root-owned catalog.
#[derive(Debug)]
pub struct Server<'h> {
    keeper: Keeper<'h>,
    catalog: PathBuf,
    /// Who must own the catalog: root, or the tests' own user for their checkout.
    owner: u32,
}

impl<'h> Server<'h> {
    /// Serves `keeper`, root's side, for the catalog `catalog`, which `owner` must own.
    pub fn new(keeper: Keeper<'h>, catalog: impl Into<PathBuf>, owner: u32) -> Self {
        Self {
            keeper,
            catalog: catalog.into(),
            owner,
        }
    }

    /// Its keeper, given back once the requests are answered.
    pub fn into_keeper(self) -> Keeper<'h> {
        self.keeper
    }

    /// Answers one request line.
    pub fn handle(&mut self, line: &str) -> Reply {
        let refuse = |why: String| Reply::Refused { why: vec![why] };
        let request: Request = match serde_json::from_str(line) {
            Ok(request) => request,
            Err(e) => return refuse(format!("not a request: {e}")),
        };
        let (app, setup) = match &request {
            Request::Preflight { app, setup, .. }
            | Request::Step { app, setup, .. }
            | Request::Adopt { app, setup, .. }
            | Request::Redo { app, setup, .. } => (app, setup.as_deref()),
            Request::Undo { app, .. } | Request::Refresh { app, .. } => (app, None),
        };
        if !is_app_id(app) {
            return refuse(format!("{app:?} is not an app id"));
        }
        if let Some(setup) = setup
            && !is_app_id(setup)
        {
            return refuse(format!("{setup:?} is not a setup id"));
        }
        // An undo of an app the catalog dropped, and a refresh, read only root's journal, which
        // root alone writes; a refresh runs the release's repository setup all the same.
        let checked = match &request {
            Request::Refresh { .. } => self.release_only(),
            Request::Undo { .. } if !self.catalog.join(app).exists() => Ok(()),
            _ => self.root_only(app),
        };
        if let Err(why) = checked {
            return refuse(why);
        }
        match request {
            Request::Preflight { app, setup } => match self.keeper.preflight(&app, setup.as_deref()) {
                Ok(why) if why.is_empty() => Reply::Ready,
                Ok(why) => Reply::Refused { why },
                Err(e) => answer(&e),
            },
            Request::Step {
                app,
                setup,
                step,
                windows,
            } => match self.keeper.step(&app, setup.as_deref(), step, windows) {
                Ok(outcome) => Reply::Step {
                    outcome,
                    notes: self.keeper.take_notes(),
                },
                Err(e) => answer(&e),
            },
            Request::Undo { app, step } => match self.keeper.undo(&app, step) {
                Ok(outcome) => Reply::Step {
                    outcome,
                    notes: self.keeper.take_notes(),
                },
                Err(e) => answer(&e),
            },
            Request::Redo { app, setup, step } => match self.keeper.redo(&app, setup.as_deref(), step) {
                Ok(outcome) => Reply::Step {
                    outcome,
                    notes: self.keeper.take_notes(),
                },
                Err(e) => answer(&e),
            },
            Request::Adopt { app, setup, owned } => match self.keeper.adopt(&app, setup.as_deref(), owned) {
                Ok(_) => Reply::Ready,
                Err(e) => answer(&e),
            },
            Request::Refresh { app, step } => match self.keeper.refresh(&app, step) {
                Ok(()) => Reply::Ready,
                Err(e) => answer(&e),
            },
        }
    }

    /// Says `ready`, then answers each request line of `input` on `output`, until `input` ends.
    ///
    /// # Errors
    ///
    /// When `input` can't be read or `output` written.
    pub fn serve(&mut self, input: impl BufRead, mut output: impl Write) -> io::Result<()> {
        let say = |reply: &Reply, output: &mut dyn Write| -> io::Result<()> {
            let line = serde_json::to_string(reply).map_err(io::Error::other)?;
            writeln!(output, "{line}")?;
            output.flush()
        };
        say(&Reply::Ready, &mut output)?;
        for line in input.lines() {
            let reply = self.handle(&line?);
            say(&reply, &mut output)?;
        }
        Ok(())
    }

    /// Checks that the release's checkout, its catalog, the app's folder and everything in it,
    /// and the repository setup root runs (`system/apt-repo.sh`) are the owner's and no one else
    /// may write them, so what runs as root is what the release shipped.
    fn root_only(&self, app: &str) -> Result<(), String> {
        let folder = self.catalog.join(app);
        if !folder.join(super::folder::APP_FILE).is_file() {
            return Err(format!("the catalog has no app {app}"));
        }
        self.release_only()?;
        under(&folder, self.owner)
    }

    /// Checks that the release's checkout, its catalog and `system/`, and the repository setup
    /// root runs are the owner's and no one else may write them.
    fn release_only(&self) -> Result<(), String> {
        let checkout = self.catalog.parent().unwrap_or(&self.catalog).to_path_buf();
        let system = checkout.join("system");
        for path in [
            checkout.clone(),
            self.catalog.clone(),
            system.clone(),
            system.join(APT_REPO),
        ] {
            owned_by(&path, self.owner)?;
        }
        Ok(())
    }
}

/// The script root runs to set up a vendor repository, in the checkout's `system/`.
const APT_REPO: &str = "apt-repo.sh";

/// Checks `dir` and everything in it, at any depth, as [`owned_by`] does.
fn under(dir: &Path, owner: u32) -> Result<(), String> {
    owned_by(dir, owner)?;
    let entries = fs::read_dir(dir).map_err(|e| format!("reading {}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| format!("reading {}: {e}", dir.display()))?.path();
        if fs::symlink_metadata(&path).is_ok_and(|m| m.is_dir()) {
            under(&path, owner)?;
        } else {
            owned_by(&path, owner)?;
        }
    }
    Ok(())
}

fn owned_by(path: &Path, owner: u32) -> Result<(), String> {
    let meta = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if meta.file_type().is_symlink() {
        return Err(format!("{} is a link; refusing to run it", path.display()));
    }
    if meta.uid() != owner {
        return Err(format!(
            "{} isn't owned by uid {owner}; refusing to run it",
            path.display()
        ));
    }
    if meta.mode() & 0o022 != 0 {
        return Err(format!("{} is writable by others; refusing to run it", path.display()));
    }
    Ok(())
}

/// The reply to a step that didn't go through: a refusal for a request the helper doesn't take.
fn answer(e: &StepError) -> Reply {
    match e {
        StepError::WrongSide { .. } | StepError::NoStep { .. } | StepError::Folder(_) => {
            Reply::Refused { why: vec![chain(e)] }
        }
        _ => Reply::Failed { why: chain(e) },
    }
}
