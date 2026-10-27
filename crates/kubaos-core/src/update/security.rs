//! The apt security updates waiting on the machine, for the daily check.
//!
//! The check counts them without root and without touching apt's own state: `apt-get update`, the
//! simulated upgrade and `apt-cache policy` run as the user against package lists kept under
//! `~/.local/state/kubaos/apt`. Ubuntu's own refresh (`apt-daily.timer`) is retired, so the
//! system's lists only change when `kubaos update` runs; these private ones are what make a
//! pending security update visible in between.
//!
//! # Owns
//!
//! - The apt options that point apt at the private lists, and reading apt's output.
//!
//! # Not here
//!
//! - Running apt, and notifying: `updater.rs`. Installing the updates: `kubaos update`'s
//!   `apt-get upgrade`.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use super::machine::MachineError;

/// Counting the pending security updates failed at one step.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CountError {
    /// The folder for the check's lists can't be made.
    #[error("creating {}", .path.display())]
    Dir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// `apt-get update` failed, or fetched only some of the lists.
    #[error("refreshing the package lists")]
    Refresh(#[source] MachineError),
    /// `apt-get -s upgrade` failed.
    #[error("simulating the upgrade")]
    Simulate(#[source] MachineError),
    /// `apt-cache policy` failed.
    #[error("reading the packages' versions")]
    Policy(#[source] MachineError),
}

impl CountError {
    /// A folder that couldn't be made.
    pub fn dir(path: &Path) -> impl FnOnce(io::Error) -> Self {
        let path = path.to_owned();
        move |source| Self::Dir { path, source }
    }
}

/// What `apt-get -s upgrade` would install, for one package.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub name: String,
    pub version: String,
    /// Whether the candidate version comes from a security pocket.
    pub security: bool,
}

impl Candidate {
    /// `name=version`.
    fn label(&self) -> String {
        format!("{}={}", self.name, self.version)
    }
}

/// The `-o` options that move apt's lists to `dir` and leave everything else (sources, dpkg's
/// status, the auto-installed marks) as the system's. The package caches are off and only the
/// English descriptions are fetched: the lists alone take about 100 MiB, and the caches added
/// another 100 MiB that two short-lived commands never reuse.
pub fn apt_options(dir: &Path) -> Vec<OsString> {
    let mut options = Vec::new();
    for (key, value) in [
        ("Dir::State", dir.join("state").into_os_string()),
        ("Dir::Cache", dir.join("cache").into_os_string()),
        ("Dir::State::status", "/var/lib/dpkg/status".into()),
        ("Dir::State::extended_states", "/var/lib/apt/extended_states".into()),
        ("Dir::Cache::srcpkgcache", OsString::new()),
        ("Dir::Cache::pkgcache", OsString::new()),
        ("Acquire::Languages", "none".into()),
    ] {
        let mut option = OsString::from(format!("{key}="));
        option.push(value);
        options.push("-o".into());
        options.push(option);
    }
    options
}

/// The folders apt needs under `dir` before it runs as a user.
pub fn apt_dirs(dir: &Path) -> [PathBuf; 2] {
    [dir.join("state/lists/partial"), dir.join("cache")]
}

/// The packages in `apt-get -s upgrade`'s output, in order.
pub fn candidates(simulated: &str) -> Vec<Candidate> {
    simulated.lines().filter_map(candidate_line).collect()
}

/// A candidate from an `Inst name [old] (new origin, origin [arch])` line; its origins are
/// every pocket that carries the version (`noble-security`, `noble-updates`).
fn candidate_line(line: &str) -> Option<Candidate> {
    let rest = line.strip_prefix("Inst ")?;
    let (name, rest) = rest.split_once(' ')?;
    let (_, candidate) = rest.split_once('(')?;
    let (version, origins) = candidate.split_once(' ')?;
    Some(Candidate {
        name: name.to_owned(),
        version: version.to_owned(),
        security: origins.split(", ").any(|origin| origin.contains("-security")),
    })
}

/// The packages `apt-cache policy` described that have a security version newer than the
/// installed one. A candidate from `-updates` is often newer than the security fix it carries,
/// and then lists only its own pocket.
///
/// The version table is newest first and marks the installed version with `***`, so a security
/// pocket above that mark is a fix not installed yet.
pub fn with_newer_security(policy: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut package = None;
    let mut newer = true;
    for line in policy.lines() {
        if !line.starts_with(' ') {
            package = line.strip_suffix(':');
            newer = true;
        } else if line.starts_with(" *** ") {
            newer = false;
        } else if newer
            && line.starts_with("        ")
            && line.contains("-security")
            && let Some(name) = package
        {
            found.insert(name.to_owned());
        }
    }
    found
}

/// The packages to report, as `name=version`, sorted: every candidate from a security pocket,
/// and those named in `with_newer`.
pub fn pending(candidates: &[Candidate], with_newer: &BTreeSet<String>) -> Vec<String> {
    let mut found: Vec<String> = candidates
        .iter()
        .filter(|c| c.security || with_newer.contains(&c.name))
        .map(Candidate::label)
        .collect();
    found.sort();
    found.dedup();
    found
}

#[cfg(test)]
mod tests;
