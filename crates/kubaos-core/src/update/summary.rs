//! What a [`Summary`] condenses to: whether the run changed anything at all, and the notification
//! sent for it when nobody watched the run in a terminal.
//!
//! # Owns
//!
//! - [`Summary::changed_nothing`] and [`Summary::notification`], with the notification's text.
//!
//! # Not here
//!
//! - The summary's fields and JSON: `event.rs`. Its rows in a terminal: `kubaos-cli`'s
//!   `update/render.rs`. Deciding to notify and sending it: `kubaos-cli`'s `update.rs`.

use super::event::Command;
use super::event::ProgramResult;
use super::event::Restart;
use super::event::Summary;
use super::firmware::FirmwareOutcome;
use super::record::capitalise;

/// The notification at the end of a run nobody watched in a terminal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notification {
    pub title: String,
    /// One line a fact, the restart first when one is needed; empty when nothing changed.
    pub body: String,
    /// Whether it asks for the user's attention: a restart is needed, or a part failed.
    pub urgent: bool,
}

impl Summary {
    /// Whether the run changed nothing and has nothing to report: the release stayed, no package,
    /// migration, program or firmware moved, and nothing waits or failed. What the session
    /// reloaded doesn't count: a switch reloads it even when the home stays the same.
    pub fn changed_nothing(&self) -> bool {
        self.from.commit == self.to.commit
            && self.packages == 0
            && self.upgraded.is_empty()
            && self.apt.is_empty()
            && self.inputs.is_empty()
            && self.nix_packages.as_ref().is_none_or(Vec::is_empty)
            && self.migrations.is_empty()
            && self.programs.iter().all(|p| {
                p.waiting.is_none() && !matches!(p.result, ProgramResult::Updated { .. } | ProgramResult::Failed { .. })
            })
            && self.suites.is_empty()
            && self.firmware.is_empty()
            && self.restart == Restart::None
    }

    /// The notification for this summary: a title naming the release, then the restart, then one
    /// short line a part that changed.
    pub fn notification(&self) -> Notification {
        let mut lines = Vec::new();
        let reason = self.restart_reason.as_deref().unwrap_or_default();
        match self.restart {
            Restart::None => {}
            Restart::Relogin => lines.push(format!("Log out and back in: {reason}")),
            Restart::Reboot => lines.push(format!("Reboot: {reason}")),
        }
        if self.packages > 0 {
            lines.push(counted(
                self.packages,
                "1 apt package upgraded",
                "{n} apt packages upgraded",
            ));
        }
        let installed: usize = self.apt.iter().map(|g| g.installed.len()).sum();
        let removed: usize = self.apt.iter().map(|g| g.removed.len()).sum();
        if installed > 0 {
            lines.push(counted(
                installed,
                "1 apt package installed",
                "{n} apt packages installed",
            ));
        }
        if removed > 0 {
            lines.push(counted(removed, "1 apt package removed", "{n} apt packages removed"));
        }
        if let Some(nix) = self.nix_packages.as_ref().filter(|nix| !nix.is_empty()) {
            lines.push(counted(nix.len(), "1 Nix package changed", "{n} Nix packages changed"));
        }
        for program in &self.programs {
            let name = program.program.title();
            match &program.result {
                ProgramResult::Updated { from, to } => lines.push(format!("{name} {from} → {to}")),
                ProgramResult::Failed { .. } => lines.push(format!("{name} failed to update: the log says why")),
                ProgramResult::Current { .. } | ProgramResult::Off => {}
            }
            if let Some(waiting) = &program.waiting {
                lines.push(format!("{name} {waiting} waits for a KubaOS release"));
            }
        }
        if !self.suites.updated.is_empty() {
            lines.push(counted(
                self.suites.updated.len(),
                "1 Suite tool updated",
                "{n} Suite tools updated",
            ));
        }
        if !self.suites.restored.is_empty() {
            lines.push(counted(
                self.suites.restored.len(),
                "1 Suite tool put back",
                "{n} Suite tools put back",
            ));
        }
        if !self.suites.upgrades.is_empty() {
            lines.push(counted(
                self.suites.upgrades.len(),
                "1 Suite tool upgrade waits: kubaos suite upgrade",
                "{n} Suite tool upgrades wait: kubaos suite upgrade",
            ));
        }
        if self.suites.error.is_some() {
            lines.push("The Suites' tools failed to update: the log says why".to_owned());
        }
        if !self.firmware.is_empty() {
            lines.push(format!("Firmware: {}", self.firmware.detail()));
        }
        if !self.migrations.is_empty() {
            lines.push(counted(self.migrations.len(), "1 migration ran", "{n} migrations ran"));
        }
        let failed = self.firmware.with(FirmwareOutcome::Failed).next().is_some()
            || self.suites.error.is_some()
            || self
                .programs
                .iter()
                .any(|p| matches!(p.result, ProgramResult::Failed { .. }));
        Notification {
            title: self.title(),
            body: lines.join("\n"),
            urgent: self.restart != Restart::None || failed,
        }
    }

    fn title(&self) -> String {
        if self.changed_nothing() {
            return "KubaOS is up to date: nothing changed".to_owned();
        }
        let to = self.to.codename.as_deref().map_or_else(
            || self.to.version.clone(),
            |codename| format!("{} {}", capitalise(codename), self.to.version),
        );
        if self.from.commit == self.to.commit {
            return match self.command {
                Command::Rollback => format!("KubaOS rolled back your packages, at {to}"),
                _ => format!("KubaOS updated, still at {to}"),
            };
        }
        match self.command {
            Command::Upgrade => format!("KubaOS upgraded to {to}"),
            Command::Rollback => format!("KubaOS rolled back to {to}"),
            Command::Update | Command::Check | Command::Packages => format!("KubaOS updated to {to}"),
        }
    }
}

/// `one` for a count of one, else `many` with `{n}` as the count: the count's word belongs to the
/// message.
fn counted(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        one.to_owned()
    } else {
        many.replace("{n}", &n.to_string())
    }
}

#[cfg(test)]
mod tests;
