//! The progress stream of `kubaos update`, `upgrade`, `rollback` and `update --check`: what the
//! updater reports as it runs. The terminal renders it, and `--json` writes one [`UpdateEvent`]
//! per line for a graphical front end, the same events either way.
//!
//! A run, in order:
//!
//! ```text
//! started
//!   phase_started ─► step | progress | output | note | warning ... ─► phase_finished
//!   ... one pair per phase that runs ...
//!   preview, prompt, answered      (before anything changes; a prompt only when it asks)
//! summary | available | declined | failed      (exactly one, last)
//! ```
//!
//! `step`, `progress` and `output` belong to the phase started last. A `prompt` of a password
//! can come inside any phase that runs a root command.
//!
//! # Owns
//!
//! - [`UpdateEvent`] and its parts, their JSON form, and [`Sink`], where the updater sends them.
//!
//! # Not here
//!
//! - Sending them and the log file: `journal.rs`. Rendering them: `kubaos-cli`'s
//!   `update/render.rs`. The install engine's own stream: `crate::event`.

use std::error::Error;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use super::record::Channel;
use super::record::capitalise;
use super::record::short;

/// Which command runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Update,
    Upgrade,
    Rollback,
    Check,
    /// `kubaos packages enable|disable`: an optional group of the package manifest turned on or
    /// off.
    Packages,
}

impl Command {
    /// The command's word, as `kubaos <word>` takes it.
    pub fn word(self) -> &'static str {
        match self {
            Self::Update => "update",
            Self::Upgrade => "upgrade",
            Self::Rollback => "rollback",
            Self::Check => "check",
            Self::Packages => "packages",
        }
    }
}

/// One stage of a run. Only the phases that have work to do are started.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// The lock, the free space and the installed release.
    Checking,
    /// `git fetch` of origin, or `git ls-remote` for a check.
    Fetching,
    /// The release policy's pick, with the signatures it checks.
    Verifying,
    /// apt's package lists, the release's commits and what will run.
    Reviewing,
    /// The snapper snapshot, the saved `flake.lock` and the history line.
    Snapshot,
    /// `apt-get upgrade`.
    SystemPackages,
    /// Moving the checkout's `release` branch.
    Release,
    /// Installing the release's Hyprland pin.
    Hyprland,
    /// Bringing the apt packages in line with the release's package manifest.
    Manifest,
    /// The release's new system migrations, as root.
    SystemMigrations,
    /// `nix flake update` of the user flake.
    Inputs,
    /// The relink and the home-manager switch, and the running session following it.
    Home,
    /// The release's new user migrations.
    UserMigrations,
}

impl Phase {
    /// The phase's name, as the terminal shows it.
    pub fn title(self) -> &'static str {
        match self {
            Self::Checking => "Checking",
            Self::Fetching => "Fetching",
            Self::Verifying => "Verifying",
            Self::Reviewing => "Reviewing changes",
            Self::Snapshot => "Snapshot",
            Self::SystemPackages => "System packages",
            Self::Release => "Release",
            Self::Hyprland => "Hyprland pin",
            Self::Manifest => "Package manifest",
            Self::SystemMigrations => "System migrations",
            Self::Inputs => "Nix inputs",
            Self::Home => "Home",
            Self::UserMigrations => "Your migrations",
        }
    }
}

/// How a phase ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Done,
    /// It found nothing it could do; the detail says why.
    Skipped,
    Failed,
}

/// A KubaOS release, as the stream names it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Release {
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codename: Option<String>,
    pub commit: String,
}

impl Release {
    /// "Shinjuku 0.1.0 (1a2b3c4)", or "0.1.0 (1a2b3c4)" without a codename.
    pub fn describe(&self) -> String {
        match &self.codename {
            Some(codename) => format!("{} {} ({})", capitalise(codename), self.version, short(&self.commit)),
            None => format!("{} ({})", self.version, short(&self.commit)),
        }
    }
}

/// What the run asks.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Prompt {
    /// A yes-or-no question, yes by default. The `kubaos` command reads the answer from stdin.
    Confirm { question: String },
    /// sudo's password, for the root commands. sudo asks for it on the terminal.
    Password,
}

/// What the user has to do for the update to apply in full.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Restart {
    #[default]
    None,
    /// Log out and back in: the running Hyprland is the old one.
    Relogin,
    /// An updated package asks for a reboot.
    Reboot,
}

/// What an update or upgrade brings, shown before it asks.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preview {
    pub from: Release,
    /// The release it moves to; none when only packages update.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<Release>,
    pub channel: Channel,
    /// The release's commits, `<short hash> <subject>`, newest first.
    pub changes: Vec<String>,
    /// The apt packages to upgrade.
    pub packages: usize,
    /// What the package manifest's phase will install, by group; nothing is removed here, since
    /// a removal is only known once apt has simulated it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub apt: Vec<GroupChange>,
    /// Whether the Hyprland pin changes.
    pub hyprland_pin: bool,
    /// The system migrations it will run.
    pub migrations: Vec<String>,
}

/// What the package manifest's phase changed in one group.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupChange {
    pub group: String,
    /// The packages it installed, which KubaOS now records as its own.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub installed: Vec<String>,
    /// The packages it removed: KubaOS installed them, and no group wants them now.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed: Vec<String>,
}

/// What a finished update, upgrade, rollback or `kubaos packages enable|disable` did.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summary {
    pub command: Command,
    pub from: Release,
    pub to: Release,
    pub channel: Channel,
    /// The release's commits, `<short hash> <subject>`, newest first.
    pub changes: Vec<String>,
    /// The apt packages upgraded.
    pub packages: usize,
    /// What the package manifest's phase installed and removed, by group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub apt: Vec<GroupChange>,
    /// The user flake's inputs `nix flake update` moved.
    pub inputs: Vec<String>,
    /// The migrations that ran, system ones first.
    pub migrations: Vec<String>,
    /// What the running session reloaded or restarted after the switch (`scripts/dot-session`).
    pub reloaded: Vec<String>,
    pub restart: Restart,
    /// Why a restart is needed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restart_reason: Option<String>,
    /// The command that undoes it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub undo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log: Option<PathBuf>,
}

/// One thing the updater reports.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UpdateEvent {
    /// First event. `log` is where the run's full output goes; a check keeps none.
    Started {
        command: Command,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        log: Option<PathBuf>,
    },
    PhaseStarted {
        phase: Phase,
    },
    /// What the phase does now, in plain words.
    Step {
        text: String,
    },
    /// `done` of `total` things in the phase (packages set up, migrations run).
    Progress {
        done: usize,
        total: usize,
    },
    /// One line a program printed, on stdout or stderr, without terminal escapes.
    Output {
        line: String,
    },
    /// A line for the user to read.
    Note {
        text: String,
    },
    Warning {
        text: String,
    },
    Preview(Preview),
    Prompt {
        prompt: Prompt,
    },
    /// The answer to the last prompt: yes, or the password was taken.
    Answered {
        yes: bool,
    },
    PhaseFinished {
        phase: Phase,
        outcome: Outcome,
        elapsed_ms: u64,
        /// The result in a few words, or why it skipped.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
    /// Last event of a check: what `kubaos update` and `kubaos upgrade` would install.
    Available {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        update: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        upgrade: Option<String>,
        /// How many apt packages from a security pocket `kubaos update` would upgrade.
        #[serde(default, skip_serializing_if = "count_is_zero")]
        apt_security: usize,
    },
    /// Last event of an update, upgrade or rollback that went through.
    Summary(Summary),
    /// Last event when the user said no: nothing changed.
    Declined,
    /// Last event of a run that stopped. `output` is the end of what the failing program printed.
    Failed {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        phase: Option<Phase>,
        error: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        output: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        log: Option<PathBuf>,
    },
}

impl UpdateEvent {
    /// The event as one line of JSON, without the newline.
    ///
    /// # Panics
    ///
    /// If `serde_json` refuses the event; its fields are plain data with string keys, which it
    /// always takes.
    pub fn to_json_line(&self) -> String {
        serde_json::to_string(self).expect("update events always serialize")
    }

    /// An event from one line of JSON.
    ///
    /// # Errors
    ///
    /// When the line is not an event.
    pub fn from_json_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }
}

/// Where the updater sends its events.
pub trait Sink {
    fn event(&mut self, event: &UpdateEvent);
}

impl Sink for Vec<UpdateEvent> {
    fn event(&mut self, event: &UpdateEvent) {
        self.push(event.clone());
    }
}

/// `e` and its causes, on one line.
pub fn chain(e: &dyn Error) -> String {
    let mut text = e.to_string();
    let mut source = e.source();
    while let Some(cause) = source {
        text.push_str(": ");
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}

/// For `skip_serializing_if`: a count that is zero leaves its key out of the JSON.
#[expect(clippy::trivially_copy_pass_by_ref, reason = "Serde passes the field by reference.")]
fn count_is_zero(n: &usize) -> bool {
    *n == 0
}

#[cfg(test)]
mod tests;
