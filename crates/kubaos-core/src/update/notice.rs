//! What the daily check found, for the bar and the Command menu, and which of it was notified.
//!
//! `~/.local/state/kubaos/update-available` holds `key=value` lines, and is empty when there is
//! nothing to install (missing before the first check):
//!
//! ```text
//! update=main at 1a2b3c4 (0.1.0)
//! update-commit=1a2b3c4d…
//! upgrade=0.2.0
//! upgrade-codename=akihabara
//! upgrade-commit=5e6f…
//! ```
//!
//! `update` is what `kubaos update` would install, `upgrade` the next series `kubaos upgrade`
//! would. The Command menu's KubaOS Extension reads the same keys (`crates/kotodama-kubaos`).
//! `~/.local/state/kubaos/notified` lists one `update <commit>` or `upgrade <commit>` per line, so
//! each is notified once.
//!
//! # Owns
//!
//! - The two files' formats, and the text of each notification.
//!
//! # Not here
//!
//! - Finding what is available: `policy.rs`. Sending the notification: `updater.rs`.

use std::collections::BTreeSet;
use std::fmt::Write;

use super::policy::Pick;
use super::policy::Target;
use super::record::capitalise;

/// What is available to install.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Available {
    pub update: Option<Target>,
    pub upgrade: Option<Target>,
}

impl Available {
    /// What a pick found.
    #[must_use]
    pub fn from_pick(pick: &Pick) -> Self {
        Self {
            update: pick.update.clone(),
            upgrade: pick.upgrade.clone(),
        }
    }

    /// Whether there is nothing to install.
    pub fn is_empty(&self) -> bool {
        self.update.is_none() && self.upgrade.is_none()
    }

    /// The file's text.
    pub fn to_text(&self) -> String {
        let mut text = String::new();
        for (key, target) in [("update", &self.update), ("upgrade", &self.upgrade)] {
            let Some(target) = target else {
                continue;
            };
            let _ = write!(text, "{key}={}\n{key}-commit={}\n", target.label, target.commit);
            if let Some(codename) = &target.codename {
                let _ = writeln!(text, "{key}-codename={codename}");
            }
        }
        text
    }

    /// The notifications to send for it, as (key in `notified`, text), skipping those in
    /// `notified`.
    pub fn notifications(&self, notified: &BTreeSet<String>) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if let Some(target) = &self.update {
            let name = target.codename.as_deref().map_or_else(|| "KubaOS".to_owned(), capitalise);
            out.push((
                format!("update {}", target.commit),
                format!(
                    "{name} {} is ready. Update when you feel like it: kubaos update. Or don't, I'm not your mum.",
                    target.label
                ),
            ));
        }
        if let Some(target) = &self.upgrade {
            let name = target.codename.as_deref().map(capitalise);
            let text = match name {
                Some(name) => format!(
                    "KubaOS {name} ({}) is out. kubaos upgrade when you're ready, I'll wait.",
                    target.label
                ),
                None => format!("KubaOS {} is out. kubaos upgrade when you're ready, I'll wait.", target.label),
            };
            out.push((format!("upgrade {}", target.commit), text));
        }
        out.retain(|(key, _)| !notified.contains(key));
        out
    }
}

/// The keys in a `notified` file.
pub fn parse_notified(text: &str) -> BTreeSet<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests;
