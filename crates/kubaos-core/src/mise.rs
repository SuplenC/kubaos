//! mise, the Suites' toolchain manager: every call KubaOS makes to it goes through [`Mise`].
//!
//! KubaOS runs mise from the filesystem's root, outside any project, and never with `--global`,
//! which reads only the user's own `config.toml`: a Suite's tools live in its fragment in
//! `conf.d/`, which mise loads with every other config (`Paths::mise_config`). The one exception
//! is [`Mise::current`], which runs in a project's folder to read its Project pins.
//!
//! # Owns
//!
//! - The [`Mise`] seam and its real implementation, [`Cli`], which runs the `mise` on `PATH`.
//! - Reading mise's JSON: the versions `mise ls --json` lists, the ones `mise ls --current --json`
//!   selects in a folder, and the updates `mise outdated --json` finds.
//!
//! # Not here
//!
//! - Which tools a Suite asks for, and its fragment: `suites.rs`.
//! - Where mise's config lives: `state.rs`, [`crate::state::Paths::mise_config`].

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
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

    /// The version of each tool mise selects in `folder`, from the folder's configs, its parents'
    /// and the global ones, with the config each comes from. mise runs in the folder, which
    /// records the folder's configs as tracked, so [`Mise::prune`] keeps the versions they select
    /// while they exist.
    ///
    /// # Errors
    ///
    /// [`MiseError`] when mise can't start, fails or prints something else.
    fn current(&self, folder: &Path) -> Result<Vec<ListedVersion>, MiseError>;

    /// Trusts `config`, so mise loads the parts of it that run code or set the environment.
    ///
    /// # Errors
    ///
    /// [`MiseError`] when mise can't start or fails.
    fn trust(&self, config: &Path) -> Result<(), MiseError>;

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

    /// Runs `argv` with the tools the global configs select, as `mise exec -- <argv>`, keeping
    /// what it prints.
    ///
    /// # Errors
    ///
    /// [`MiseError`] when mise can't start, or it or the command fails.
    fn exec(&self, argv: &[&str]) -> Result<(), MiseError>;

    /// Deletes every installed version no tracked config selects, at once.
    ///
    /// # Errors
    ///
    /// [`MiseError`] when mise can't start or fails.
    fn prune(&self) -> Result<(), MiseError>;

    /// Locks `tools`, which the global configs ask for, in mise's global lockfile
    /// ([`crate::state::Paths::mise_lock`]), for this machine's platform only, and leaves every
    /// other entry as it is.
    ///
    /// # Errors
    ///
    /// [`MiseError`] when mise can't start or fails; the lockfile is then as it was.
    fn lock(&self, tools: &[String]) -> Result<(), MiseError>;
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
    /// mise needed GitHub's API, and the calls GitHub allows without a token (60 an hour, for
    /// everything on the machine's address) were used up. `resets` is when they come back, as mise
    /// printed it, when it did.
    #[error(
        "`mise {command}` hit GitHub's limit of 60 API calls an hour without a token, which {}: run it again then, \
         or now with a token, as `GITHUB_TOKEN=$(gh auth token) kubaos …`",
        resets.as_deref().map_or_else(|| "resets within the hour".to_owned(), |at| format!("resets at {at}"))
    )]
    RateLimited { command: String, resets: Option<String> },
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
        self.command_in(&self.cwd, args)
    }

    fn command_in(&self, cwd: &Path, args: &[&str]) -> Command {
        let mut command = Command::new(&self.program);
        // `MISE_YES` answers mise's own prompts (prune's, a plugin's install) as a script must.
        command
            .args(args)
            .current_dir(cwd)
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
        let mut command = self.command(args);
        command.stdout(Stdio::from(io::stderr()));
        self.run(command, args, Shown::Yes).map(drop)
    }

    /// Runs mise and keeps what it prints.
    fn run_captured(&self, args: &[&str]) -> Result<Output, MiseError> {
        self.capture(self.command(args), args)
    }

    /// Runs `command`, mise with `args`, and keeps what it prints.
    fn capture(&self, command: Command, args: &[&str]) -> Result<Output, MiseError> {
        self.run(command, args, Shown::No)
    }

    /// Runs `command`, mise with `args`, with its warnings also written to a log file of its own,
    /// so that a failure can say why even when a person saw the output and this process didn't.
    fn run(&self, mut command: Command, args: &[&str], shown: Shown) -> Result<Output, MiseError> {
        let log = tempfile::NamedTempFile::new().ok();
        if let Some(log) = &log {
            command
                .env("MISE_LOG_FILE", log.path())
                .env("MISE_LOG_FILE_LEVEL", "warn");
        }
        let spawned = match shown {
            Shown::Yes => command.status().map(|status| Output {
                status,
                stdout: Vec::new(),
                stderr: Vec::new(),
            }),
            Shown::No => command.output(),
        };
        let out = spawned.map_err(|source| MiseError::Spawn {
            program: self.program_name(),
            source,
        })?;
        if out.status.success() {
            return Ok(out);
        }
        let stderr = String::from_utf8_lossy(&out.stderr);
        let logged = log
            .as_ref()
            .and_then(|log| fs::read_to_string(log.path()).ok())
            .unwrap_or_default();
        let command = args.join(" ");
        if is_rate_limited(&logged) || is_rate_limited(&stderr) {
            return Err(MiseError::RateLimited {
                command,
                resets: resets_at(&logged).or_else(|| resets_at(&stderr)),
            });
        }
        Err(MiseError::Failed {
            command,
            status: out.status.to_string(),
            message: last_line(&stderr),
        })
    }
}

/// Whether a run shows mise's output to a person or keeps it.
#[derive(Clone, Copy)]
enum Shown {
    Yes,
    No,
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

    fn current(&self, folder: &Path) -> Result<Vec<ListedVersion>, MiseError> {
        let args = ["ls", "--current", "--json"];
        // HAZARD: no `MISE_YES` here. mise answers its trust prompt with it, so reading a cloned
        // project's folder would trust the hooks and environment of its config unasked.
        let mut command = self.command_in(folder, &args);
        command.env_remove("MISE_YES");
        let out = self.capture(command, &args)?;
        parse_list(&out.stdout).map_err(|source| MiseError::Output {
            command: args.join(" "),
            source,
        })
    }

    fn trust(&self, config: &Path) -> Result<(), MiseError> {
        let mut command = self.command(&["trust"]);
        command.arg(config);
        self.capture(command, &["trust", &config.to_string_lossy()]).map(drop)
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

    fn exec(&self, argv: &[&str]) -> Result<(), MiseError> {
        let mut call = vec!["exec", "--"];
        call.extend(argv);
        self.run_captured(&call).map(drop)
    }

    fn prune(&self) -> Result<(), MiseError> {
        self.run_shown(&["prune", "--tools"])
    }

    fn lock(&self, tools: &[String]) -> Result<(), MiseError> {
        if tools.is_empty() {
            return Ok(());
        }
        // Every platform a new lockfile would hold costs a lock-time provenance check, some of
        // them a GitHub API call each; this machine only installs its own.
        let mut args = vec!["lock", "--global"];
        if let Some(platform) = PLATFORM {
            args.extend(["--platform", platform]);
        }
        args.extend(tools.iter().map(String::as_str));
        self.run_captured(&args).map(drop)
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

/// This machine's platform, as mise names it in a lockfile, or `None` on one KubaOS doesn't run
/// on, where mise picks the platforms itself.
const PLATFORM: Option<&str> = if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
    Some("linux-x64")
} else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
    Some("linux-arm64")
} else {
    None
};

/// Whether mise's output says GitHub refused a call for its rate limit.
fn is_rate_limited(text: &str) -> bool {
    text.lines()
        .any(|line| line.to_ascii_lowercase().contains("rate limit exceeded"))
}

/// When GitHub's rate limit resets, from mise's `GitHub rate limit exceeded. Resets at <time>`.
fn resets_at(text: &str) -> Option<String> {
    text.lines()
        .rev()
        .find_map(|line| line.split_once("Resets at ").map(|(_, at)| at.trim().to_owned()))
        .filter(|at| !at.is_empty())
}

/// The last line `stderr` holds, which is where mise says what failed, before the two lines its
/// errors end with: its version, and how to see more.
fn last_line(stderr: &str) -> String {
    stderr
        .lines()
        .map(str::trim)
        .rfind(|line| !line.is_empty() && !line.contains("ERROR Version:") && !line.contains("MISE_VERBOSE"))
        .unwrap_or_default()
        .to_owned()
}

#[cfg(test)]
pub(crate) mod fake;
#[cfg(test)]
mod tests;
