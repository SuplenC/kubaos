//! Moving the installed Suites' tools on, tool by tool: patch releases within each request
//! (`kubaos suite update`) and minor and major releases past it (`kubaos suite upgrade`).
//!
//! Both read what mise offers for the tools the Suites' fragments ask for, and leave out any tool
//! another config asks for, so a project's `mise.toml` is never read, rewritten or bumped. An
//! update leaves every fragment as it is; an upgrade has mise rewrite the fragment to the new
//! request, and a Suite whose main toolchain moves to another of its lines takes that line in
//! its record. Both prune at once.
//!
//! # Owns
//!
//! - [`Suites::available`], [`Suites::update`] and [`Suites::upgrade`], and what they report.
//!
//! # Not here
//!
//! - The lockfile entries an update saves and a rollback puts back: `lockfile.rs`.
//! - The pick list and the switch after an upgrade: `kubaos-cli`'s `suite.rs`.

use std::collections::BTreeSet;

use super::Before;
use super::Request;
use super::Suite;
use super::SuiteConfig;
use super::SuiteError;
use super::Suites;
use super::catalog;
use super::major_minor;
use super::on_line;
use super::same_file;
use crate::mise::OutdatedTool;
use crate::mise::ToolRequest;

/// How far a tool moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reach {
    /// To the newest patch release its request allows.
    Update,
    /// Past its request, to the newest minor or major release.
    Upgrade,
}

/// A tool of an installed Suite that has a newer release.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Available {
    pub tool: String,
    /// What the Suite's fragment asks for.
    pub request: String,
    /// The version installed, or `None` when none is.
    pub current: Option<String>,
    /// The release it moves to.
    pub latest: String,
    /// For an upgrade, the request the fragment takes, at the request's precision (`0.13`).
    pub target: Option<String>,
}

/// The tools of one Suite that have newer releases.
#[derive(Clone, Debug)]
pub struct SuiteUpdates {
    pub suite: &'static Suite,
    pub tools: Vec<Available>,
}

/// What [`Suites::upgrade`] did.
#[derive(Clone, Debug)]
pub struct Upgraded {
    /// The tools it moved, by Suite.
    pub moved: Vec<SuiteUpdates>,
    /// The Suites whose record took a new line with their main toolchain, which a switch
    /// applies, each with its record and fragment as they were, for [`Suites::restore`] when
    /// that switch fails.
    pub lines: Vec<(&'static Suite, Before)>,
}

impl Suites {
    /// The tools of the installed Suites that have a newer release within `reach`, by Suite in
    /// the catalog's order. A tool another config asks for is left out, and so is a main toolchain
    /// whose newer release is on no line the Suite offers, which instead offers the newest
    /// release on one that is. mise runs only when a Suite has a fragment.
    ///
    /// # Errors
    ///
    /// [`SuiteError::Read`] when a record or fragment can't be read, or
    /// [`SuiteError::Outdated`] when mise can't say what is newer.
    pub fn available(&self, reach: Reach) -> Result<Vec<SuiteUpdates>, SuiteError> {
        let with_tools = self.with_tools()?;
        self.available_for(&with_tools, reach)
    }

    /// [`Suites::available`] for `suites` only, in their order.
    fn available_for(&self, suites: &[&'static Suite], reach: Reach) -> Result<Vec<SuiteUpdates>, SuiteError> {
        let mut fragments = Vec::new();
        for &suite in suites {
            let tools = self.fragment(suite)?;
            if !tools.is_empty() {
                fragments.push((suite, tools));
            }
        }
        if fragments.is_empty() {
            return Ok(Vec::new());
        }
        let outdated = self
            .mise
            .outdated(reach == Reach::Upgrade)
            .map_err(SuiteError::Outdated)?;
        let mut found = Vec::new();
        for (suite, tools) in fragments {
            let fragment = self.paths.mise_fragment(suite.name);
            let mut available = Vec::new();
            for entry in &outdated {
                if !entry
                    .source
                    .as_deref()
                    .is_some_and(|source| same_file(source, &fragment))
                {
                    continue;
                }
                let Some(asked) = tools.iter().find(|t| t.tool == entry.tool) else {
                    continue;
                };
                if let Some(offer) = self.offer(suite, &asked.request, entry, reach)? {
                    available.push(offer);
                }
            }
            if !available.is_empty() {
                found.push(SuiteUpdates {
                    suite,
                    tools: available,
                });
            }
        }
        Ok(found)
    }

    /// The installed Suites that have mise tools in a fragment, in the catalog's order.
    ///
    /// # Errors
    ///
    /// [`SuiteError::Read`] when a record or fragment can't be read.
    pub fn with_tools(&self) -> Result<Vec<&'static Suite>, SuiteError> {
        let installed = self.installed()?;
        let mut with_tools = Vec::new();
        for suite in catalog::all().filter(|suite| installed.contains_key(suite.name)) {
            if !self.fragment(suite)?.is_empty() {
                with_tools.push(suite);
            }
        }
        Ok(with_tools)
    }

    /// Moves each tool of `suites` that has a newer patch release to it, then prunes. Every
    /// fragment and record stays as it is. A Suite without a fragment has nothing to move.
    ///
    /// # Errors
    ///
    /// A failed lock, read or mise call; the tools mise moved before failing stay moved.
    pub fn update(&self, suites: &[&'static Suite]) -> Result<Vec<SuiteUpdates>, SuiteError> {
        let _lock = self.lock()?;
        let moved = self.available_for(suites, Reach::Update)?;
        let tools: BTreeSet<String> = moved
            .iter()
            .flat_map(|s| s.tools.iter().map(|t| t.tool.clone()))
            .collect();
        if tools.is_empty() {
            return Ok(moved);
        }
        self.mise
            .upgrade(&tools.into_iter().collect::<Vec<_>>())
            .map_err(SuiteError::Upgrade)?;
        self.mise.prune().map_err(SuiteError::Prune)?;
        Ok(moved)
    }

    /// Moves each tool of `suites` that has a newer minor or major release to it, naming the
    /// request it moves to; mise rewrites the fragment. A Suite whose main toolchain lands on
    /// another of its lines takes that line in its record, for the switch that is the caller's.
    /// Then prunes.
    ///
    /// # Errors
    ///
    /// A failed lock, read, write, git sync or mise call. The tools mise moved before failing
    /// stay moved, and their Suites' records follow them.
    pub fn upgrade(&self, suites: &[&'static Suite]) -> Result<Upgraded, SuiteError> {
        let _lock = self.lock()?;
        let moved = self.available_for(suites, Reach::Upgrade)?;
        if moved.is_empty() {
            return Ok(Upgraded {
                moved,
                lines: Vec::new(),
            });
        }
        let mut before = Vec::new();
        for s in &moved {
            let state = self.before(s.suite, self.config(s.suite)?)?;
            before.push((s.suite, state));
        }
        let targets: Vec<ToolRequest> = moved
            .iter()
            .flat_map(|s| &s.tools)
            .filter_map(|t| {
                Some(ToolRequest {
                    tool: t.tool.clone(),
                    request: t.target.clone()?,
                })
            })
            .collect();
        let bumped = self.mise.bump(&targets);
        let lines = self.follow_lines(before)?;
        bumped.map_err(SuiteError::Upgrade)?;
        self.mise.prune().map_err(SuiteError::Prune)?;
        Ok(Upgraded { moved, lines })
    }

    /// What `entry` offers the tool `suite` asks for at `request`, or `None` when nothing within
    /// `reach` is newer.
    fn offer(
        &self,
        suite: &Suite,
        request: &str,
        entry: &OutdatedTool,
        reach: Reach,
    ) -> Result<Option<Available>, SuiteError> {
        if entry.current.as_deref() == Some(entry.latest.as_str()) {
            return Ok(None);
        }
        let available = |latest: String, target: Option<String>| Available {
            tool: entry.tool.clone(),
            request: request.to_owned(),
            current: entry.current.clone(),
            latest,
            target,
        };
        if reach == Reach::Update {
            return Ok(Some(available(entry.latest.clone(), None)));
        }
        let Some(bump) = entry.bump.clone().filter(|bump| bump != request) else {
            return Ok(None);
        };
        if !is_main(suite, &entry.tool) || line_of(suite, &bump).is_some() {
            return Ok(Some(available(entry.latest.clone(), Some(bump))));
        }
        Ok(self
            .newest_on_a_line(suite, entry, request)?
            .map(|(latest, target)| available(latest, Some(target))))
    }

    /// The newest release of `suite`'s main toolchain on one of the Suite's lines, with the
    /// request that reaches it, when it is newer than what is installed and past `request`.
    fn newest_on_a_line(
        &self,
        suite: &Suite,
        entry: &OutdatedTool,
        request: &str,
    ) -> Result<Option<(String, String)>, SuiteError> {
        let mut newest: Option<(String, &str)> = None;
        for line in suite.versions {
            let Some(latest) = self.mise.latest(&entry.tool, line).map_err(SuiteError::Outdated)? else {
                continue;
            };
            if newest.as_ref().is_none_or(|(best, _)| newer(&latest, best)) {
                newest = Some((latest, line));
            }
        }
        let Some((latest, line)) = newest else {
            return Ok(None);
        };
        let target = major_minor(&latest)
            .filter(|target| on_line(target, line))
            .unwrap_or_else(|| (*line).to_owned());
        let past = entry.current.as_deref().is_none_or(|current| newer(&latest, current));
        Ok((past && target != request && !on_line(request, &target)).then_some((latest, target)))
    }

    /// Gives each Suite in `before` whose main toolchain the fragment now asks for on another of
    /// its lines that line in its record, and returns them.
    fn follow_lines(&self, before: Vec<(&'static Suite, Before)>) -> Result<Vec<(&'static Suite, Before)>, SuiteError> {
        let mut changed = Vec::new();
        for (suite, was) in before {
            let Some(config) = &was.config else {
                continue;
            };
            let Some(main) = suite.mise.iter().find(|t| t.request == Request::Suite) else {
                continue;
            };
            let Some(asked) = self.fragment(suite)?.into_iter().find(|t| t.tool == main.name) else {
                continue;
            };
            let Some(line) = line_of(suite, &asked.request) else {
                continue;
            };
            if config.version.as_deref() == Some(line) {
                continue;
            }
            let config = SuiteConfig {
                version: Some(line.to_owned()),
                ..config.clone()
            };
            self.write(suite, &config)?;
            changed.push((suite, was));
        }
        Ok(changed)
    }
}

/// Whether `tool` is `suite`'s main toolchain, which asks for the Suite's version.
fn is_main(suite: &Suite, tool: &str) -> bool {
    suite.mise.iter().any(|t| t.name == tool && t.request == Request::Suite)
}

/// The line of `suite` that `request` is on, the most precise when several are.
fn line_of(suite: &Suite, request: &str) -> Option<&'static str> {
    suite
        .versions
        .iter()
        .copied()
        .filter(|line| on_line(request, line))
        .max_by_key(|line| line.len())
}

/// Whether version `a` is newer than `b`, comparing their dot-separated parts as numbers where
/// both are numbers and as text where not.
fn newer(a: &str, b: &str) -> bool {
    let parts = |v: &str| -> Vec<(u64, String)> {
        v.split('.')
            .map(|part| (part.parse().unwrap_or(0), part.to_owned()))
            .collect()
    };
    parts(a) > parts(b)
}

#[cfg(test)]
pub(super) mod tests;
