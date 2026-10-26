//! What apt and dpkg say about the machine: which packages are installed and held, which have an
//! install candidate, and what removing some would take with them.
//!
//! # Owns
//!
//! - The commands' arguments ([`QUERY`]) and reading their output.
//!
//! # Not here
//!
//! - Running them: the updater, through its `Machine`.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// `dpkg-query`'s arguments: one `<status> <want> <package> <version>` line per package dpkg
/// knows. The want is `hold` for a held package.
pub const QUERY: [&str; 3] = [
    "-W",
    "-f",
    "${db:Status-Status} ${db:Status-Want} ${Package} ${Version}\n",
];

/// One installed package.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Installed {
    pub version: String,
    pub held: bool,
}

/// The installed packages, from [`QUERY`]'s output.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Dpkg {
    installed: BTreeMap<String, Installed>,
    /// Packages on hold that aren't installed, which apt won't install until they're unheld.
    held_missing: BTreeSet<String>,
}

impl Dpkg {
    /// Reads [`QUERY`]'s output. A package of another architecture shows once per architecture;
    /// it counts as installed when one of them is.
    pub fn parse(text: &str) -> Self {
        let mut dpkg = Self::default();
        for line in text.lines() {
            let mut words = line.split_whitespace();
            let (Some(status), Some(want), Some(name)) = (words.next(), words.next(), words.next()) else {
                continue;
            };
            let held = want == "hold";
            if status == "installed" {
                dpkg.held_missing.remove(name);
                dpkg.installed.insert(
                    name.to_owned(),
                    Installed {
                        version: words.next().unwrap_or_default().to_owned(),
                        held,
                    },
                );
            } else if held && !dpkg.installed.contains_key(name) {
                dpkg.held_missing.insert(name.to_owned());
            }
        }
        dpkg
    }

    /// `package`, when it is installed.
    pub fn get(&self, package: &str) -> Option<&Installed> {
        self.installed.get(package)
    }

    /// Whether `package` is installed.
    pub fn has(&self, package: &str) -> bool {
        self.installed.contains_key(package)
    }

    /// Whether `package` is on hold without being installed.
    pub fn held_missing(&self, package: &str) -> bool {
        self.held_missing.contains(package)
    }
}

/// The packages `apt-cache policy` names with an install candidate.
pub fn candidates(policy: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut current: Option<&str> = None;
    for line in policy.lines() {
        if !line.starts_with(char::is_whitespace) {
            current = line.trim_end().strip_suffix(':');
            continue;
        }
        if let (Some(name), Some(candidate)) = (current, line.trim().strip_prefix("Candidate:"))
            && !matches!(candidate.trim(), "" | "(none)")
        {
            found.insert(name.to_owned());
        }
    }
    found
}

/// The packages `apt-get -s remove` would remove: its `Remv <package> [<version>]` lines.
pub fn removals(simulated: &str) -> BTreeSet<String> {
    simulated
        .lines()
        .filter_map(|line| line.strip_prefix("Remv "))
        .filter_map(|rest| rest.split_whitespace().next())
        .map(|name| name.split(':').next().unwrap_or(name).to_owned())
        .collect()
}

#[cfg(test)]
mod tests;
