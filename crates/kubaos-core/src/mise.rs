//! mise, the Suites' toolchain manager: every call KubaOS makes to it goes through [`Mise`].
//!
//! KubaOS runs mise from the filesystem's root, outside any project, and never with `--global`,
//! which reads only the user's own `config.toml`: a Suite's tools live in its fragment in
//! `conf.d/`, which mise loads with every other config (`Paths::mise_config`).
//!
//! # Owns
//!
//! - The [`Mise`] seam and its real implementation, [`Cli`], which runs the `mise` on `PATH`.
//! - Reading mise's JSON: the versions `mise ls --json` lists and the updates `mise outdated
//!   --json` finds.
//!
//! # Not here
//!
//! - Which tools a Suite asks for, and its fragment: `suites.rs`.
//! - Where mise's config lives: `state.rs`, [`crate::state::Paths::mise_config`].

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;

use serde::Deserialize;

/// One tool at a requested version: what `mise install <tool>@<request>` takes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ToolRequest {
    /// mise's name for the tool: a registry name (`node`) or a backend's (`aqua:astral-sh/uv`).
    pub tool: String,
    /// The version asked for, a prefix such as `24.22` or an exact version.
    pub request: String,
}

impl fmt::Display for ToolRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.tool, self.request)
    }
}

/// One version of a tool, as `mise ls --json` lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListedVersion {
    pub tool: String,
    /// The exact version.
    pub version: String,
    /// What the config that selects it asks for, when one does.
    pub requested: Option<String>,
    /// The config file that selects it, when one does.
    pub source: Option<PathBuf>,
    /// Whether it is installed; a version a config asks for may not be yet.
    pub installed: bool,
}

/// One tool that has a newer release, as `mise outdated --json` reports it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutdatedTool {
    pub tool: String,
    /// What its config asks for.
    pub requested: String,
    /// The version installed, or `None` when none is.
    pub current: Option<String>,
    /// The newest version the request allows (`latest`), or, with `--bump`, the newest release.
    pub latest: String,
    /// With `--bump`, the request that reaches `latest`, at the request's precision.
    pub bump: Option<String>,
    /// The config file that asks for it.
    pub source: Option<PathBuf>,
}

/// What KubaOS asks of mise. Every call runs from outside any project, so only the user's and
/// KubaOS's configs are read.
pub trait Mise: fmt::Debug + Send + Sync {
    /// Installs each tool at its request, with nothing written to any config.
    ///
    /// # Errors
    ///
    /// [`MiseError`] when mise can't start or fails; the tools that did install stay.
    fn install(&self, tools: &[ToolRequest]) -> Result<(), MiseError>;

    /// Uninstalls these exact versions.
    ///
    /// # Errors
    ///
    /// [`MiseError`] when mise can't start or fails.
    fn uninstall(&self, versions: &[ToolRequest]) -> Result<(), MiseError>;

    /// Every version mise knows of: installed, or asked for by a config.
    ///
    /// # Errors
    ///
    /// [`MiseError`] when mise can't start, fails or prints something else.
    fn list(&self) -> Result<Vec<ListedVersion>, MiseError>;

    /// The tools with a newer release: within their request, or past it with `bump`.
    ///
    /// # Errors
    ///
    /// [`MiseError`] when mise can't start, fails or prints something else.
    fn outdated(&self, bump: bool) -> Result<Vec<OutdatedTool>, MiseError>;

    /// The newest release of `tool` that `prefix` selects (`24` selects `24.22.1`), or `None`
    /// when none does.
    ///
    /// # Errors
    ///
    /// [`MiseError`] when mise can't start or fails.
    fn latest(&self, tool: &str, prefix: &str) -> Result<Option<String>, MiseError>;

    /// Moves each of `tools` to the newest release its request allows, leaving every config as
    /// it is.
    ///
    /// # Errors
    ///
    /// [`MiseError`] when mise can't start or fails; the tools moved before the failure stay.
    fn upgrade(&self, tools: &[String]) -> Result<(), MiseError>;

    /// Moves each tool past its request to the release `request` selects, and rewrites the
    /// config that asks for the tool to ask for `request`.
    ///
    /// # Errors
    ///
    /// [`MiseError`] when mise can't start or fails; the tools moved before the failure stay.
    fn bump(&self, targets: &[ToolRequest]) -> Result<(), MiseError>;

    /// Installs every tool the configs ask for at the version the lockfile names.
    ///
    /// # Errors
    ///
    /// [`MiseError`] when mise can't start or fails, as it does with no lockfile.
    fn install_locked(&self) -> Result<(), MiseError>;

    /// Deletes every installed version no tracked config selects, at once.
    ///
    /// # Errors
    ///
    /// [`MiseError`] when mise can't start or fails.
    fn prune(&self) -> Result<(), MiseError>;

    /// Locks the tools of the global configs, KubaOS's fragments included, in mise's global
    /// lockfile ([`crate::state::Paths::mise_lock`]).
    ///
    /// # Errors
    ///
    /// [`MiseError`] when mise can't start or fails.
    fn lock(&self) -> Result<(), MiseError>;
}

/// A call to mise failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MiseError {
    /// mise could not be started.
    #[error("running {program}")]
    Spawn {
        program: String,
        #[source]
        source: io::Error,
    },
    /// mise exited with an error; `message` is the last line it printed on stderr, when it
    /// printed there and not to the terminal.
    #[error("`mise {command}` failed ({status}){}", if message.is_empty() { String::new() } else { format!(": {message}") })]
    Failed {
        command: String,
        status: String,
        message: String,
    },
    /// mise printed something other than the JSON asked for.
    #[error("reading what `mise {command}` printed")]
    Output {
        command: String,
        #[source]
        source: serde_json::Error,
    },
}

/// The real mise: the `mise` on `PATH`, run from `/`.
#[derive(Clone, Debug)]
pub struct Cli {
    program: OsString,
    cwd: PathBuf,
    /// Whether an install's progress goes to this process's stderr, where a person sees it.
    shown: bool,
}

impl Default for Cli {
    fn default() -> Self {
        Self::new()
    }
}

impl Cli {
    /// The `mise` on `PATH`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            program: "mise".into(),
            cwd: PathBuf::from("/"),
            shown: true,
        }
    }

    /// Keeps what mise prints to itself, for a caller that reports in its own stream; a failure
    /// still carries mise's last line.
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.shown = false;
        self
    }

    /// The mise at `program`.
    #[must_use]
    pub fn with_program(mut self, program: impl Into<OsString>) -> Self {
        self.program = program.into();
        self
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(&self.program);
        // `MISE_YES` answers mise's own prompts (prune's, a plugin's install) as a script must.
        command
            .args(args)
            .current_dir(&self.cwd)
            .env("MISE_YES", "1")
            .stdin(Stdio::null());
        command
    }

    fn program_name(&self) -> String {
        self.program.to_string_lossy().into_owned()
    }

    /// Runs mise with its output on this process's stderr, where a person sees an install's
    /// progress, unless it is [`Cli::quiet`].
    fn run_shown(&self, args: &[&str]) -> Result<(), MiseError> {
        if !self.shown {
            return self.run_captured(args).map(drop);
        }
        let status = self
            .command(args)
            .stdout(Stdio::from(io::stderr()))
            .status()
            .map_err(|source| MiseError::Spawn {
                program: self.program_name(),
                source,
            })?;
        if status.success() {
            return Ok(());
        }
        Err(MiseError::Failed {
            command: args.join(" "),
            status: status.to_string(),
            message: String::new(),
        })
    }

    /// Runs mise and keeps what it prints.
    fn run_captured(&self, args: &[&str]) -> Result<Output, MiseError> {
        let out = self.command(args).output().map_err(|source| MiseError::Spawn {
            program: self.program_name(),
            source,
        })?;
        if out.status.success() {
            return Ok(out);
        }
        Err(MiseError::Failed {
            command: args.join(" "),
            status: out.status.to_string(),
            message: last_line(&String::from_utf8_lossy(&out.stderr)),
        })
    }
}

impl Mise for Cli {
    fn install(&self, tools: &[ToolRequest]) -> Result<(), MiseError> {
        if tools.is_empty() {
            return Ok(());
        }
        let specs: Vec<String> = tools.iter().map(ToString::to_string).collect();
        let mut args = vec!["install"];
        args.extend(specs.iter().map(String::as_str));
        self.run_shown(&args)
    }

    fn uninstall(&self, versions: &[ToolRequest]) -> Result<(), MiseError> {
        if versions.is_empty() {
            return Ok(());
        }
        let specs: Vec<String> = versions.iter().map(ToString::to_string).collect();
        let mut args = vec!["uninstall"];
        args.extend(specs.iter().map(String::as_str));
        self.run_shown(&args)
    }

    fn list(&self) -> Result<Vec<ListedVersion>, MiseError> {
        let args = ["ls", "--json"];
        let out = self.run_captured(&args)?;
        parse_list(&out.stdout).map_err(|source| MiseError::Output {
            command: args.join(" "),
            source,
        })
    }

    fn outdated(&self, bump: bool) -> Result<Vec<OutdatedTool>, MiseError> {
        let args: &[&str] = if bump {
            &["outdated", "--bump", "--json"]
        } else {
            &["outdated", "--json"]
        };
        let out = self.run_captured(args)?;
        parse_outdated(&out.stdout).map_err(|source| MiseError::Output {
            command: args.join(" "),
            source,
        })
    }

    fn latest(&self, tool: &str, prefix: &str) -> Result<Option<String>, MiseError> {
        let spec = format!("{tool}@{prefix}");
        let out = self.run_captured(&["latest", &spec])?;
        let latest = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        Ok((!latest.is_empty()).then_some(latest))
    }

    fn upgrade(&self, tools: &[String]) -> Result<(), MiseError> {
        if tools.is_empty() {
            return Ok(());
        }
        let mut args = vec!["upgrade"];
        args.extend(tools.iter().map(String::as_str));
        self.run_shown(&args)
    }

    fn bump(&self, targets: &[ToolRequest]) -> Result<(), MiseError> {
        if targets.is_empty() {
            return Ok(());
        }
        let specs: Vec<String> = targets.iter().map(ToString::to_string).collect();
        let mut args = vec!["upgrade", "--bump"];
        args.extend(specs.iter().map(String::as_str));
        self.run_shown(&args)
    }

    fn install_locked(&self) -> Result<(), MiseError> {
        self.run_shown(&["install", "--locked"])
    }

    fn prune(&self) -> Result<(), MiseError> {
        self.run_shown(&["prune", "--tools"])
    }

    fn lock(&self) -> Result<(), MiseError> {
        self.run_captured(&["lock", "--global"]).map(drop)
    }
}

/// One entry of `mise ls --json`, which maps each tool to its versions.
#[derive(Deserialize)]
struct LsEntry {
    version: String,
    #[serde(default)]
    requested_version: Option<String>,
    #[serde(default)]
    source: Option<LsSource>,
    #[serde(default)]
    installed: bool,
}

#[derive(Deserialize)]
struct LsSource {
    path: PathBuf,
}

/// The versions in what `mise ls --json` printed, by tool and in mise's order.
///
/// # Errors
///
/// [`serde_json::Error`] when it is not a map of tools to their versions.
pub fn parse_list(json: &[u8]) -> Result<Vec<ListedVersion>, serde_json::Error> {
    let by_tool: BTreeMap<String, Vec<LsEntry>> = serde_json::from_slice(json)?;
    Ok(by_tool
        .into_iter()
        .flat_map(|(tool, entries)| {
            entries.into_iter().map(move |entry| ListedVersion {
                tool: tool.clone(),
                version: entry.version,
                requested: entry.requested_version,
                source: entry.source.map(|s| s.path),
                installed: entry.installed,
            })
        })
        .collect())
}

/// One entry of `mise outdated --json`, which maps each tool to it.
#[derive(Deserialize)]
struct OutdatedEntry {
    requested: String,
    #[serde(default)]
    current: Option<String>,
    latest: String,
    #[serde(default)]
    bump: Option<String>,
    #[serde(default)]
    source: Option<LsSource>,
}

/// The tools in what `mise outdated --json` printed, by tool.
///
/// # Errors
///
/// [`serde_json::Error`] when it is not a map of tools to their updates.
pub fn parse_outdated(json: &[u8]) -> Result<Vec<OutdatedTool>, serde_json::Error> {
    let by_tool: BTreeMap<String, OutdatedEntry> = serde_json::from_slice(json)?;
    Ok(by_tool
        .into_iter()
        .map(|(tool, entry)| OutdatedTool {
            tool,
            requested: entry.requested,
            current: entry.current,
            latest: entry.latest,
            bump: entry.bump,
            source: entry.source.map(|s| s.path),
        })
        .collect())
}

/// The last line `stderr` holds, which is where mise says what failed.
fn last_line(stderr: &str) -> String {
    stderr
        .lines()
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .unwrap_or_default()
        .to_owned()
}

#[cfg(test)]
pub(crate) mod fake;
#[cfg(test)]
mod tests;
