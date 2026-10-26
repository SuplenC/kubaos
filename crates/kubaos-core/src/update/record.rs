//! What a machine records about its KubaOS release: the channel it follows, the release it runs,
//! and the history of updates a rollback goes back through.
//!
//! - `/etc/kubaos/channel`: `stable` or `edge`, one word.
//! - `/var/lib/kubaos/release`: `key=value` lines, `version`, `codename`, `commit`, `channel`,
//!   `date`.
//! - `/var/lib/kubaos/update-history`: one line per update, oldest first, tab-separated: the date,
//!   the commit and version before, the commit and version after, and the user's saved
//!   `flake.lock`.
//!
//! The install writes the first two; every update and rollback rewrites them through sudo. All
//! three are world-readable, so the notifier and the Command menu read them as the user.
//!
//! # Owns
//!
//! - Where the machine-wide files live ([`SystemPaths`]), and their formats.
//!
//! # Not here
//!
//! - The user's files (saved locks, migration markers, the notice): `state::Paths`.
//! - Writing them as root: `machine.rs`.

use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;

use super::version::InvalidVersion;
use super::version::Version;

/// KubaOS's root-owned checkout, which the user flake takes KubaOS from.
pub const CHECKOUT: &str = "usr/share/kubaos";
/// The local branch of the checkout the user flake reads, which only an update moves.
pub const RELEASE_BRANCH: &str = "release";
/// The keys that sign KubaOS's commits and tags, in git's `allowed_signers` format. Root-owned, so
/// nothing running as the user can add a key.
pub const ALLOWED_SIGNERS: &str = "usr/share/kubaos-keys/allowed_signers";
/// The channel the machine follows.
pub const CHANNEL: &str = "etc/kubaos/channel";
/// The release the machine runs.
pub const RELEASE: &str = "var/lib/kubaos/release";
/// The updates a rollback goes back through.
pub const HISTORY: &str = "var/lib/kubaos/update-history";
/// A marker per system migration that ran.
pub const MARKERS: &str = "var/lib/kubaos/migrations";
/// The Hyprland pin apt reads, installed from the release's copy.
pub const APT_PIN: &str = "etc/apt/preferences.d/kubaos-hyprland";
/// The release's Hyprland pin, inside the checkout.
pub const RELEASE_PIN: &str = "system/files/apt/preferences.d/kubaos-hyprland";

/// The machine-wide paths of updates, under a root: `/` on the machine, the target's mount point
/// during an install, a temporary folder in tests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemPaths {
    root: PathBuf,
}

impl SystemPaths {
    /// The paths of the running machine.
    #[must_use]
    pub fn machine() -> Self {
        Self::under("/")
    }

    /// The paths under `root`.
    #[must_use]
    pub fn under(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn join(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    /// The root these paths are under.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// KubaOS's checkout, `/usr/share/kubaos`.
    pub fn checkout(&self) -> PathBuf {
        self.join(CHECKOUT)
    }

    /// `/usr/share/kubaos-keys/allowed_signers`.
    pub fn allowed_signers(&self) -> PathBuf {
        self.join(ALLOWED_SIGNERS)
    }

    /// `/etc/kubaos/channel`.
    pub fn channel(&self) -> PathBuf {
        self.join(CHANNEL)
    }

    /// `/var/lib/kubaos/release`.
    pub fn release(&self) -> PathBuf {
        self.join(RELEASE)
    }

    /// `/var/lib/kubaos/update-history`.
    pub fn history(&self) -> PathBuf {
        self.join(HISTORY)
    }

    /// `/var/lib/kubaos/migrations/`.
    pub fn markers(&self) -> PathBuf {
        self.join(MARKERS)
    }

    /// The system migrations of the checked-out release.
    pub fn system_migrations(&self) -> PathBuf {
        self.checkout().join("migrations/system")
    }

    /// The user migrations of the checked-out release.
    pub fn user_migrations(&self) -> PathBuf {
        self.checkout().join("migrations/user")
    }

    /// `/etc/apt/preferences.d/kubaos-hyprland`.
    pub fn apt_pin(&self) -> PathBuf {
        self.join(APT_PIN)
    }

    /// The checked-out release's package manifest.
    pub fn manifest(&self) -> PathBuf {
        self.checkout().join(crate::apt::manifest::MANIFEST)
    }

    /// The checked-out release's script that sets up a vendor apt repository.
    pub fn apt_repo_script(&self) -> PathBuf {
        self.checkout().join("system/apt-repo.sh")
    }

    /// `/etc/kubaos/apt-groups`, the user's choices of optional package groups.
    pub fn apt_choices(&self) -> PathBuf {
        self.join(crate::apt::choices::CHOICES)
    }

    /// `/var/lib/kubaos/apt-installed`, the packages KubaOS installed.
    pub fn apt_record(&self) -> PathBuf {
        self.join(crate::apt::record::RECORD)
    }
}

/// Which releases a machine follows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    /// Verified release tags of the installed series.
    Stable,
    /// `origin/main`, its head commit checked against the allowed signers.
    Edge,
}

/// A channel name that is no channel.
#[derive(Debug, thiserror::Error)]
#[error("{0:?} is no channel (stable or edge)")]
pub struct InvalidChannel(String);

impl Channel {
    /// The channel's name, as `/etc/kubaos/channel` holds it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Edge => "edge",
        }
    }
}

impl FromStr for Channel {
    type Err = InvalidChannel;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text.trim() {
            "stable" => Ok(Self::Stable),
            "edge" => Ok(Self::Edge),
            other => Err(InvalidChannel(other.to_owned())),
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The release a machine runs, as `/var/lib/kubaos/release` holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Installed {
    /// `VERSION` at the installed commit.
    pub version: Version,
    /// `CODENAME` at the installed commit, when it has one.
    pub codename: Option<String>,
    /// The commit `release` points at.
    pub commit: String,
    pub channel: Channel,
    /// When it was installed, `YYYY-MM-DD HH:MM:SS` in UTC; empty when no record said.
    pub date: String,
}

/// `/var/lib/kubaos/release` is not a release record.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RecordError {
    /// A key it needs is missing.
    #[error("no `{0}` in the release record")]
    Missing(&'static str),
    /// Its version is not one.
    #[error("the release record's version")]
    Version(#[source] InvalidVersion),
    /// Its channel is not one.
    #[error("the release record's channel")]
    Channel(#[source] InvalidChannel),
}

impl Installed {
    /// Parses the record's `key=value` lines. Unknown keys are ignored, so a later KubaOS can add
    /// some.
    ///
    /// # Errors
    ///
    /// [`RecordError`] when `version`, `commit` or `channel` is missing or wrong.
    pub fn parse(text: &str) -> Result<Self, RecordError> {
        let get = |key: &str| {
            text.lines()
                .filter_map(|line| line.split_once('='))
                .find(|(k, _)| k.trim() == key)
                .map(|(_, v)| v.trim().to_owned())
                .filter(|v| !v.is_empty())
        };
        let version = get("version").ok_or(RecordError::Missing("version"))?;
        let channel = get("channel").ok_or(RecordError::Missing("channel"))?;
        Ok(Self {
            version: Version::parse(&version).map_err(RecordError::Version)?,
            codename: get("codename"),
            commit: get("commit").ok_or(RecordError::Missing("commit"))?,
            channel: channel.parse().map_err(RecordError::Channel)?,
            date: get("date").unwrap_or_default(),
        })
    }

    /// The record as `/var/lib/kubaos/release` holds it.
    pub fn to_text(&self) -> String {
        format!(
            "version={}\ncodename={}\ncommit={}\nchannel={}\ndate={}\n",
            self.version,
            self.codename.as_deref().unwrap_or_default(),
            self.commit,
            self.channel,
            self.date,
        )
    }

    /// "Shinjuku 0.1.0 (edge, 1a2b3c4)", for people.
    pub fn describe(&self) -> String {
        let name = self.codename.as_deref().map(capitalise).unwrap_or_default();
        let sep = if name.is_empty() { "" } else { " " };
        format!(
            "{name}{sep}{} ({}, {})",
            self.version,
            self.channel,
            short(&self.commit)
        )
    }
}

/// One update in `/var/lib/kubaos/update-history`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryEntry {
    pub date: String,
    pub from_commit: String,
    pub from_version: String,
    pub to_commit: String,
    pub to_version: String,
    /// The user's `flake.lock` as it was before the update, when there was one.
    pub lock: Option<PathBuf>,
}

impl HistoryEntry {
    /// Parses one line of the history; `None` for a line that is not one.
    pub fn parse(line: &str) -> Option<Self> {
        let mut fields = line.split('\t');
        let mut next = || fields.next().map(str::to_owned);
        let entry = Self {
            date: next()?,
            from_commit: next()?,
            from_version: next()?,
            to_commit: next()?,
            to_version: next()?,
            lock: next().filter(|lock| !lock.is_empty()).map(PathBuf::from),
        };
        (!entry.from_commit.is_empty()).then_some(entry)
    }

    /// The entry as a line of the history, with its newline.
    pub fn line(&self) -> String {
        let lock = self.lock.as_deref().map(Path::to_string_lossy).unwrap_or_default();
        format!(
            "{}\t{}\t{}\t{}\t{}\t{lock}\n",
            self.date, self.from_commit, self.from_version, self.to_commit, self.to_version
        )
    }
}

/// The entries of a history file, oldest first, skipping lines that are not entries.
pub fn parse_history(text: &str) -> Vec<HistoryEntry> {
    text.lines().filter_map(HistoryEntry::parse).collect()
}

/// The first 7 characters of a commit, as git shows it.
pub fn short(commit: &str) -> &str {
    commit.get(..7).unwrap_or(commit)
}

/// `shinjuku` as people read it, `Shinjuku`.
pub fn capitalise(word: &str) -> String {
    let mut chars = word.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
