//! What a release brings, as people read it: one [`Change`] per commit that reached `main`, read
//! from its Conventional Commit subject, with the pull request that merged it.
//!
//! `main` takes pull requests as merge commits and the odd direct commit. Its first-parent
//! history is one entry per merge or direct commit; a merge stands for the commits it brought in,
//! and its own subject ("Merge pull request #12 from …") is never a change.
//!
//! # Owns
//!
//! - [`Change`], [`ChangeKind`], [`Section`] and [`Changelog`], their JSON form, and reading them
//!   from `git log` ([`collapse`]).
//! - The web address of a GitHub `origin` ([`github_repository`]).
//!
//! # Not here
//!
//! - Running git: `git.rs`. Showing the changes: `kubaos-cli`'s `update/render.rs`.

use std::collections::HashMap;
use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

/// The Conventional Commit type of a change; `other` for a subject that isn't one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Feat,
    Fix,
    Perf,
    Build,
    Refactor,
    Revert,
    Style,
    Docs,
    Test,
    Ci,
    Chore,
    Other,
}

impl ChangeKind {
    /// The kind a Conventional Commit type names; `None` for a word that names none.
    fn of(word: &str) -> Option<Self> {
        Some(match word {
            "feat" => Self::Feat,
            "fix" => Self::Fix,
            "perf" => Self::Perf,
            "build" => Self::Build,
            "refactor" => Self::Refactor,
            "revert" => Self::Revert,
            "style" => Self::Style,
            "docs" => Self::Docs,
            "test" => Self::Test,
            "ci" => Self::Ci,
            "chore" => Self::Chore,
            _ => return None,
        })
    }

    /// The section of the changelog the kind is listed under.
    pub fn section(self) -> Section {
        match self {
            Self::Feat => Section::New,
            Self::Fix => Section::Fixed,
            Self::Perf => Section::Faster,
            Self::Build => Section::Build,
            Self::Refactor | Self::Revert | Self::Style | Self::Other => Section::Other,
            Self::Docs | Self::Test | Self::Ci | Self::Chore => Section::Internal,
        }
    }
}

/// Where a change is listed, in the order the changelog lists them. Breaking changes come before
/// every section, whatever their kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Section {
    New,
    Fixed,
    Faster,
    Build,
    Other,
    /// Docs, tests, CI and chores: counted, not listed.
    Internal,
}

impl Section {
    /// The sections that list their changes, in order.
    pub const LISTED: [Self; 5] = [Self::New, Self::Fixed, Self::Faster, Self::Build, Self::Other];

    /// The section's label, as the changelog shows it.
    pub fn title(self) -> &'static str {
        match self {
            Self::New => "New",
            Self::Fixed => "Fixed",
            Self::Faster => "Faster",
            Self::Build => "Build",
            Self::Other => "Other",
            Self::Internal => "Internal",
        }
    }
}

/// One commit a release brings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Change {
    pub kind: ChangeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// The subject after `type(scope):`; the whole subject for kind `other`.
    pub subject: String,
    /// A `!` after the type or scope, or a `BREAKING CHANGE:` footer.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub breaking: bool,
    /// The pull request that merged it; none for a commit made on `main`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pr: Option<u32>,
    /// The commit's full hash.
    pub commit: String,
}

impl Change {
    /// The change `message`, the commit's whole message, describes. A subject ending in `(#12)`,
    /// as GitHub's squash merges write it, names its pull request when `pr` doesn't.
    #[must_use]
    pub fn parse(commit: impl Into<String>, message: &str, pr: Option<u32>) -> Self {
        let mut lines = message.lines();
        let subject = lines.next().unwrap_or("").trim();
        let (subject, squashed) = strip_pr_suffix(subject);
        let footer = lines.any(|line| line.starts_with("BREAKING CHANGE:") || line.starts_with("BREAKING-CHANGE:"));
        let commit = commit.into();
        let pr = pr.or(squashed);
        match conventional(subject) {
            Some((kind, scope, bang, description)) => Self {
                kind,
                scope: scope.map(str::to_owned),
                subject: description.to_owned(),
                breaking: bang || footer,
                pr,
                commit,
            },
            None => Self {
                kind: ChangeKind::Other,
                scope: None,
                subject: subject.to_owned(),
                breaking: footer,
                pr,
                commit,
            },
        }
    }

    /// The section the change is listed under; `None` for a breaking one, which comes first.
    pub fn section(&self) -> Option<Section> {
        (!self.breaking).then(|| self.kind.section())
    }
}

/// `type(scope)!: description` split up; `None` when `subject` isn't a Conventional Commit of a
/// known type.
fn conventional(subject: &str) -> Option<(ChangeKind, Option<&str>, bool, &str)> {
    let (head, description) = subject.split_once(':')?;
    let description = description.trim();
    let (head, bang) = match head.strip_suffix('!') {
        Some(head) => (head, true),
        None => (head, false),
    };
    let (word, scope) = match head.split_once('(') {
        Some((word, scope)) => (word, Some(scope.strip_suffix(')')?)),
        None => (head, None),
    };
    let kind = ChangeKind::of(word)?;
    if description.is_empty() || scope.is_some_and(|s| s.is_empty() || s.contains(['(', ')'])) {
        return None;
    }
    Some((kind, scope, bang, description))
}

/// `subject` without a trailing ` (#12)`, and the number.
fn strip_pr_suffix(subject: &str) -> (&str, Option<u32>) {
    let parsed = subject
        .strip_suffix(')')
        .and_then(|rest| rest.rsplit_once(" (#"))
        .and_then(|(rest, number)| Some((rest.trim_end(), number.parse().ok()?)));
    match parsed {
        Some((rest, number)) => (rest, Some(number)),
        None => (subject, None),
    }
}

/// The pull request a merge commit's subject names: "Merge pull request #12 from …".
fn merged_pr(subject: &str) -> Option<u32> {
    let rest = subject.strip_prefix("Merge pull request #")?;
    let digits = rest.split(|c: char| !c.is_ascii_digit()).next()?;
    digits.parse().ok()
}

/// A release's changes, newest first, as far as the limit lists them.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Changelog {
    /// The breaking changes and those of the listed sections, newest first: every breaking one,
    /// then the others up to the limit.
    pub changes: Vec<Change>,
    /// How many of the listed sections' changes the limit left out.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub more_changes: usize,
    /// How many internal changes (docs, tests, CI, chores) there are; they aren't listed.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub internal_changes: usize,
    /// The web address of the GitHub repository the pull requests are on; none when `origin`
    /// isn't on GitHub.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
}

impl Changelog {
    /// The changelog of `changes`, newest first, listing at most `limit` that aren't breaking.
    #[must_use]
    pub fn new(changes: Vec<Change>, limit: usize, repository: Option<String>) -> Self {
        let mut log = Self {
            repository,
            ..Self::default()
        };
        let mut listed = 0;
        for change in changes {
            match change.section() {
                None => log.changes.push(change),
                Some(Section::Internal) => log.internal_changes += 1,
                Some(_) if listed < limit => {
                    listed += 1;
                    log.changes.push(change);
                }
                Some(_) => log.more_changes += 1,
            }
        }
        log
    }

    /// How many changes people would read about: the listed ones and those the limit left out.
    pub fn count(&self) -> usize {
        self.changes.len() + self.more_changes
    }

    /// Whether the release brings no change at all, internal ones included.
    pub fn is_empty(&self) -> bool {
        self.count() == 0 && self.internal_changes == 0
    }

    /// The web address of pull request `pr`, when the repository is known.
    pub fn pr_url(&self, pr: u32) -> Option<String> {
        self.repository.as_ref().map(|repo| format!("{repo}/pull/{pr}"))
    }
}

/// One commit of `git log`: its hash, its parents' hashes and its whole message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogEntry {
    pub commit: String,
    pub parents: Vec<String>,
    pub message: String,
}

/// The changes of `entries`, every commit of a range newest first, one per commit that isn't a
/// merge, newest first. Walks the first-parent history from the newest: a merge's changes are the
/// commits it brought in, each carrying the pull request its subject names.
pub fn collapse(entries: &[LogEntry]) -> Vec<Change> {
    let index: HashMap<&str, usize> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| (e.commit.as_str(), i))
        .collect();
    let mut spine = Vec::new();
    let mut at = (!entries.is_empty()).then_some(0);
    while let Some(i) = at {
        spine.push(i);
        at = entries[i].parents.first().and_then(|p| index.get(p.as_str()).copied());
    }
    // Oldest first, so whatever an older merge brought in is claimed before a newer merge's
    // second parent reaches it again.
    let mut seen: HashSet<usize> = spine.iter().copied().collect();
    let mut changes = Vec::new();
    for &i in spine.iter().rev() {
        let entry = &entries[i];
        let Some(branch) = entry.parents.get(1) else {
            changes.push(vec![Change::parse(&entry.commit, &entry.message, None)]);
            continue;
        };
        let pr = merged_pr(entry.message.lines().next().unwrap_or(""));
        let merged = brought_in(branch, entries, &index, &mut seen)
            .into_iter()
            .map(|e| Change::parse(&e.commit, &e.message, pr))
            .collect();
        changes.push(merged);
    }
    changes.into_iter().rev().flatten().collect()
}

/// The commits that aren't merges reachable from `tip` within `entries` and not `seen`, in the
/// order of `entries`; marks each one seen.
fn brought_in<'e>(
    tip: &str,
    entries: &'e [LogEntry],
    index: &HashMap<&str, usize>,
    seen: &mut HashSet<usize>,
) -> Vec<&'e LogEntry> {
    let mut found = Vec::new();
    let mut stack = vec![tip];
    while let Some(hash) = stack.pop() {
        let Some(&i) = index.get(hash) else {
            continue;
        };
        if !seen.insert(i) {
            continue;
        }
        if entries[i].parents.len() < 2 {
            found.push(i);
        }
        stack.extend(entries[i].parents.iter().map(String::as_str));
    }
    found.sort_unstable();
    found.into_iter().map(|i| &entries[i]).collect()
}

/// The web address of the GitHub repository `url` names, `https://github.com/<owner>/<repo>`,
/// from an HTTPS or SSH remote; none for a remote elsewhere.
pub fn github_repository(url: &str) -> Option<String> {
    let url = url.trim();
    let path = [
        "https://github.com/",
        "http://github.com/",
        "ssh://git@github.com/",
        "git@github.com:",
    ]
    .iter()
    .find_map(|prefix| url.strip_prefix(prefix))?;
    let path = path.trim_end_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let (owner, repo) = path.split_once('/')?;
    let valid = |part: &str| {
        !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    };
    (valid(owner) && valid(repo)).then(|| format!("https://github.com/{owner}/{repo}"))
}

/// For `skip_serializing_if`: a count that is zero leaves its key out of the JSON.
#[expect(clippy::trivially_copy_pass_by_ref, reason = "Serde passes the field by reference.")]
fn is_zero(n: &usize) -> bool {
    *n == 0
}

#[cfg(test)]
mod tests;
