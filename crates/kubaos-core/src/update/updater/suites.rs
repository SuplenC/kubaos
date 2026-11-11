//! The Suite step of an update and of a rollback: the Suites' tools moved to their newest patch
//! releases, as `kubaos suite update --all` moves them, and the upgrades left for
//! `kubaos suite upgrade` listed; or, on a rollback, put back at the versions their lockfile
//! entries named before the update. Its failure is a warning: the run goes on.
//!
//! # Owns
//!
//! - Saving the Suites' lockfile entries beside the saved `flake.lock`, and the Suite phase.
//!
//! # Not here
//!
//! - What moves and how, and the lockfile entries: `crate::suites`.

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use super::Updater;
use crate::suites::Reach;
use crate::suites::SuiteError;
use crate::suites::SuiteUpdates;
use crate::suites::Suites;
use crate::update::event::Phase;
use crate::update::event::SuiteChanges;
use crate::update::event::SuiteTool;
use crate::update::event::chain;
use crate::update::machine::Machine;

/// The command that installs the upgrades an update lists.
const UPGRADE: &str = "kubaos suite upgrade";

impl<M: Machine> Updater<'_, M> {
    fn suites(&self) -> Suites {
        Suites::with_mise(&self.user, Arc::clone(&self.mise))
    }

    /// Copies the Suites' entries in mise's lockfile to the saved locks, beside `flake.lock`'s
    /// copy, for a rollback to put back. `None` when no Suite has a locked tool.
    pub(super) fn save_suite_locks(&mut self) -> Result<Option<PathBuf>, SuiteError> {
        let saved = self.saved_lock_path("mise.lock");
        let wrote = self.suites().save_locked(&saved)?;
        Ok(wrote.then_some(saved))
    }

    /// Moves the installed Suites' tools to their newest patch releases, then lists the minor and
    /// major releases waiting for `kubaos suite upgrade`. Runs no phase when no Suite has mise
    /// tools; `saved` is whether their lockfile entries are saved, without which nothing moves.
    pub(super) fn update_suites(&mut self, saved: bool) -> SuiteChanges {
        let suites = self.suites();
        let with_tools = match suites.with_tools() {
            Ok(with_tools) if with_tools.is_empty() => return SuiteChanges::default(),
            Ok(with_tools) => with_tools,
            Err(e) => {
                self.j.phase_start(Phase::Suites);
                return self.suites_failed(&e);
            }
        };
        self.j.phase_start(Phase::Suites);
        let mut changes = SuiteChanges::default();
        if saved {
            self.j.step("mise upgrade");
            match suites.update(&with_tools) {
                Ok(moved) => changes.updated = tools(&moved, |t| (t.current.clone(), t.latest.clone())),
                Err(e) => return self.suites_failed(&e),
            }
        } else {
            self.j
                .warn("the Suites' lockfile entries aren't saved, so their tools stay where they are this time");
        }
        self.j.step("mise outdated --bump");
        match suites.available(Reach::Upgrade) {
            Ok(waiting) => {
                changes.upgrades = tools(&waiting, |t| {
                    (
                        Some(t.request.clone()),
                        t.target.clone().unwrap_or_else(|| t.latest.clone()),
                    )
                });
            }
            Err(e) => return self.suites_failed(&e),
        }
        for tool in &changes.upgrades {
            self.j.note(tool.describe());
        }
        if !changes.upgrades.is_empty() {
            self.j.note(format!("Upgrade them with {UPGRADE}"));
        }
        self.j.detail(match changes.updated.len() {
            0 => "the Suites' tools are at their newest patch releases".to_owned(),
            1 => "1 tool updated".to_owned(),
            n => format!("{n} tools updated"),
        });
        self.j.phase_end_optional(true);
        changes
    }

    /// Puts the Suites' lockfile entries saved at `saved` back, installs those versions and
    /// prunes the newer ones.
    pub(super) fn restore_suites(&mut self, saved: &Path) -> SuiteChanges {
        self.j.phase_start(Phase::Suites);
        self.j.step("mise install --locked");
        match self.suites().restore_locked(saved) {
            Ok(restored) => {
                let restored: Vec<SuiteTool> = restored
                    .into_iter()
                    .map(|(suite, t)| SuiteTool {
                        suite: suite.to_owned(),
                        tool: t.tool,
                        from: None,
                        to: t.request,
                    })
                    .collect();
                if restored.is_empty() {
                    self.j.skip("no Suite still asks for a tool that update moved");
                } else {
                    self.j.detail(match restored.len() {
                        1 => "1 tool put back".to_owned(),
                        n => format!("{n} tools put back"),
                    });
                }
                self.j.phase_end_optional(true);
                SuiteChanges {
                    restored,
                    ..SuiteChanges::default()
                }
            }
            Err(e) => self.suites_failed(&e),
        }
    }

    fn suites_failed(&mut self, e: &SuiteError) -> SuiteChanges {
        let error = chain(e);
        self.j.warn(format!("Suites: {error}. The run goes on without them."));
        self.j.phase_end_optional(false);
        SuiteChanges {
            error: Some(error),
            ..SuiteChanges::default()
        }
    }

    /// A new file in the saved locks, named for now with `extension`.
    pub(super) fn saved_lock_path(&self, extension: &str) -> PathBuf {
        let dir = self.user.saved_locks();
        let mut saved = dir.join(format!("{}.{extension}", self.unix()));
        let mut n = 1;
        while saved.exists() {
            saved = dir.join(format!("{}-{n}.{extension}", self.unix()));
            n += 1;
        }
        saved
    }
}

/// Each tool of `found` as the summary lists it, with the versions `ends` gives it.
fn tools(
    found: &[SuiteUpdates],
    ends: impl Fn(&crate::suites::Available) -> (Option<String>, String),
) -> Vec<SuiteTool> {
    found
        .iter()
        .flat_map(|s| {
            s.tools.iter().map(|t| {
                let (from, to) = ends(t);
                SuiteTool {
                    suite: s.suite.name.to_owned(),
                    tool: t.tool.clone(),
                    from,
                    to,
                }
            })
        })
        .collect()
}
