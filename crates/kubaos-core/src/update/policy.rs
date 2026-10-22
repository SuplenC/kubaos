//! The release policy: given the refs `origin` offers, which commit an update moves `release` to,
//! and whether an upgrade to the next series is out. Everything that knows about tags, release
//! branches and signatures is here, behind [`Policy`] and [`Refs`], so a different way of cutting
//! releases replaces this module and leaves the rest of the updater alone.
//!
//! - **Edge** follows `origin/main`. Its head commit must verify against the allowed signers when
//!   the machine has them; without them it is taken with a warning.
//! - **Stable** takes the newest `vX.Y.Z` tag of the installed series that verifies, and points at
//!   the lowest newer series as an upgrade. Without allowed signers it takes nothing.
//!
//! # Owns
//!
//! - [`Policy`], its two channels, and [`Refs`], what a policy may ask of a repository.
//!
//! # Not here
//!
//! - Reading refs from the checkout or from `git ls-remote`: `git.rs`.
//! - Moving `release` and everything an update does after: `updater.rs`.

use std::error::Error;

use super::record::Channel;
use super::record::Installed;
use super::record::short;
use super::version::Version;

/// Whether a commit or tag is signed by a key in the allowed signers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verified {
    /// Signed by an allowed key.
    Good,
    /// Unsigned, or signed by another key.
    Bad,
    /// The machine has no allowed signers to check against.
    NoKeys,
    /// Not checked: the refs came from `git ls-remote`, which has no objects to check. An update
    /// checks again before it moves `release`.
    Unchecked,
}

/// A release tag `origin` offers, and the commit it points at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    pub name: String,
    pub commit: String,
}

/// Reading the refs failed.
#[derive(Debug, thiserror::Error)]
#[error("{what}")]
pub struct RefsError {
    what: String,
    #[source]
    source: Box<dyn Error + Send + Sync>,
}

impl RefsError {
    /// Reading `what` failed with `source`.
    #[must_use]
    pub fn new(what: impl Into<String>, source: impl Into<Box<dyn Error + Send + Sync>>) -> Self {
        Self {
            what: what.into(),
            source: source.into(),
        }
    }
}

/// What a policy may ask of the repository it picks from.
pub trait Refs {
    /// The commit `origin`'s `main` points at, or `None` when it has no `main`.
    ///
    /// # Errors
    ///
    /// [`RefsError`] when the refs can't be read.
    fn main_head(&self) -> Result<Option<String>, RefsError>;

    /// Every tag named `v*`, with the commit it points at.
    ///
    /// # Errors
    ///
    /// [`RefsError`] when the refs can't be read.
    fn tags(&self) -> Result<Vec<Tag>, RefsError>;

    /// Whether `commit` is signed by an allowed key.
    ///
    /// # Errors
    ///
    /// [`RefsError`] when git can't be asked.
    fn verify_commit(&self, commit: &str) -> Result<Verified, RefsError>;

    /// Whether the tag `name` is signed by an allowed key.
    ///
    /// # Errors
    ///
    /// [`RefsError`] when git can't be asked.
    fn verify_tag(&self, name: &str) -> Result<Verified, RefsError>;

    /// The text of `path` at `commit`, when the repository has that commit.
    fn file_at(&self, commit: &str, path: &str) -> Option<String>;
}

/// Where an update or an upgrade goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub commit: String,
    /// `VERSION` at the commit, or the tag's version.
    pub version: Option<Version>,
    /// `CODENAME` at the commit, when the repository has it.
    pub codename: Option<String>,
    /// How people read it: "0.1.2", "main at 1a2b3c4 (0.1.0)".
    pub label: String,
}

/// What a policy found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pick {
    /// The release an update moves to, or `None` when the installed one is the newest.
    pub update: Option<Target>,
    /// The next series, which only `kubaos upgrade` moves to.
    pub upgrade: Option<Target>,
    /// What the user should know about the pick, one line each.
    pub notes: Vec<String>,
}

/// A policy refused to pick.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PolicyError {
    #[error("reading the release refs")]
    Refs(#[source] RefsError),
    /// Edge follows a `main` that `origin` doesn't have.
    #[error("origin has no main branch to follow")]
    NoMain,
    /// Edge's head commit is not signed by an allowed key.
    #[error("origin/main's head {commit} is not signed by a KubaOS key")]
    Unsigned { commit: String },
}

/// How a channel picks its releases.
pub trait Policy {
    /// What an update and an upgrade would move `installed` to.
    ///
    /// # Errors
    ///
    /// [`PolicyError`] when the refs can't be read or the candidate fails its check.
    fn pick(&self, installed: &Installed, refs: &dyn Refs) -> Result<Pick, PolicyError>;
}

/// The policy of `channel`.
pub fn for_channel(channel: Channel) -> &'static dyn Policy {
    match channel {
        Channel::Edge => &Edge,
        Channel::Stable => &Stable,
    }
}

/// Follows `origin/main`.
#[derive(Clone, Copy, Debug)]
pub struct Edge;

/// The note edge prints when the machine has no allowed signers.
pub const NO_KEYS_EDGE: &str = "no /usr/share/kubaos-keys/allowed_signers on this machine: \
     origin/main's head is taken without checking its signature";

impl Policy for Edge {
    fn pick(&self, installed: &Installed, refs: &dyn Refs) -> Result<Pick, PolicyError> {
        let head = refs
            .main_head()
            .map_err(PolicyError::Refs)?
            .ok_or(PolicyError::NoMain)?;
        let mut pick = Pick::default();
        if head == installed.commit {
            return Ok(pick);
        }
        match refs.verify_commit(&head).map_err(PolicyError::Refs)? {
            Verified::Good | Verified::Unchecked => {}
            Verified::NoKeys => pick.notes.push(NO_KEYS_EDGE.to_owned()),
            Verified::Bad => return Err(PolicyError::Unsigned { commit: head }),
        }
        let version = refs.file_at(&head, "VERSION").and_then(|v| Version::parse(&v).ok());
        let label = match &version {
            Some(version) => format!("main at {} ({version})", short(&head)),
            None => format!("main at {}", short(&head)),
        };
        pick.update = Some(Target {
            codename: codename(refs, &head),
            commit: head,
            version,
            label,
        });
        Ok(pick)
    }
}

/// Follows verified release tags.
#[derive(Clone, Copy, Debug)]
pub struct Stable;

impl Policy for Stable {
    fn pick(&self, installed: &Installed, refs: &dyn Refs) -> Result<Pick, PolicyError> {
        let mut pick = Pick::default();
        let mut releases: Vec<(Version, Tag)> = refs
            .tags()
            .map_err(PolicyError::Refs)?
            .into_iter()
            .filter_map(|tag| Some((Version::parse(&tag.name).ok()?, tag)))
            .filter(|(version, _)| !version.is_pre() && *version > installed.version)
            .collect();
        releases.sort_by(|a, b| b.0.cmp(&a.0));
        if releases.is_empty() {
            return Ok(pick);
        }
        let (patches, newer): (Vec<_>, Vec<_>) = releases
            .into_iter()
            .partition(|(version, _)| version.same_series(&installed.version));
        pick.update = newest_verified(refs, patches, &mut pick.notes)?;
        if pick.notes.iter().any(|note| note == NO_KEYS_STABLE) {
            return Ok(pick);
        }
        let next_series = newer.iter().map(|(version, _)| (version.major, version.minor)).min();
        let next: Vec<_> = newer
            .into_iter()
            .filter(|(version, _)| Some((version.major, version.minor)) == next_series)
            .collect();
        pick.upgrade = newest_verified(refs, next, &mut pick.notes)?;
        Ok(pick)
    }
}

/// The note stable prints when the machine has no allowed signers.
pub const NO_KEYS_STABLE: &str = "no /usr/share/kubaos-keys/allowed_signers on this machine: \
     stable can't verify a release, so only packages update";

/// The first of `releases` (newest first) whose tag verifies, noting the ones that don't.
fn newest_verified(
    refs: &dyn Refs,
    releases: Vec<(Version, Tag)>,
    notes: &mut Vec<String>,
) -> Result<Option<Target>, PolicyError> {
    for (version, tag) in releases {
        match refs.verify_tag(&tag.name).map_err(PolicyError::Refs)? {
            Verified::Good | Verified::Unchecked => {
                return Ok(Some(Target {
                    codename: codename(refs, &tag.commit),
                    label: version.to_string(),
                    commit: tag.commit,
                    version: Some(version),
                }));
            }
            Verified::Bad => notes.push(format!("{} is not signed by a KubaOS key: skipped", tag.name)),
            Verified::NoKeys => {
                notes.push(NO_KEYS_STABLE.to_owned());
                return Ok(None);
            }
        }
    }
    Ok(None)
}

fn codename(refs: &dyn Refs, commit: &str) -> Option<String> {
    refs.file_at(commit, "CODENAME")
        .map(|c| c.trim().to_owned())
        .filter(|c| !c.is_empty())
}

#[cfg(test)]
mod tests;
