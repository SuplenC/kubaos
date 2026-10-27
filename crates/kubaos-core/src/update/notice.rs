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
//! apt-security=2
//! apt-security-packages=openssl=3.0.13-0ubuntu3.5,linux-image-6.8.0-60-generic=6.8.0-60.63
//! ```
//!
//! `update` is what `kubaos update` would install, `upgrade` the next series `kubaos upgrade`
//! would, and `apt-security` how many apt packages from a security pocket `kubaos update` would
//! upgrade (`security.rs`). The Command menu's KubaOS Extension reads the same keys (`crates/kotodama-kubaos`).
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

/// What each `notified` key for a pending apt security package starts with.
pub const APT_SECURITY_KEY: &str = "apt-security ";

/// What is available to install.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Available {
    pub update: Option<Target>,
    pub upgrade: Option<Target>,
    /// The apt packages from a security pocket waiting to be upgraded, as `name=version`.
    pub apt_security: Vec<String>,
}

impl Available {
    /// What a pick found.
    #[must_use]
    pub fn from_pick(pick: &Pick) -> Self {
        Self {
            update: pick.update.clone(),
            upgrade: pick.upgrade.clone(),
            apt_security: Vec::new(),
        }
    }

    /// Whether there is nothing to install.
    pub fn is_empty(&self) -> bool {
        self.update.is_none() && self.upgrade.is_none() && self.apt_security.is_empty()
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
        if !self.apt_security.is_empty() {
            let _ = write!(
                text,
                "apt-security={}\napt-security-packages={}\n",
                self.apt_security.len(),
                self.apt_security.join(",")
            );
        }
        text
    }

    /// The `notified` key of each pending apt security package: `apt-security name=version`.
    pub fn apt_security_keys(&self) -> BTreeSet<String> {
        self.apt_security
            .iter()
            .map(|package| format!("{APT_SECURITY_KEY}{package}"))
            .collect()
    }

    /// The packages on the `apt-security-packages` line of the file's text.
    pub fn apt_security_of(text: &str) -> Vec<String> {
        text.lines()
            .find_map(|line| line.strip_prefix("apt-security-packages="))
            .map(|packages| packages.split(',').map(str::to_owned).collect())
            .unwrap_or_default()
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
        // One notification when the set gains a package not notified yet, not whenever it changes:
        // installing some of the packages must not announce the rest again.
        if let Some(new) = self.apt_security_keys().into_iter().find(|key| !notified.contains(key)) {
            let text = match self.apt_security.len() {
                1 => "1 apt security update is waiting. kubaos update brings it.".to_owned(),
                n => format!("{n} apt security updates are waiting. kubaos update brings them."),
            };
            out.push((new, text));
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
