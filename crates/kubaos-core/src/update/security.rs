//! The apt security updates waiting on the machine, for the daily check.
//!
//! The check counts them without root and without touching apt's own state: `apt-get update` and
//! the simulated upgrade run as the user against package lists kept under
//! `~/.local/state/kubaos/apt`. Ubuntu's own refresh (`apt-daily.timer`) is retired, so the
//! system's lists only change when `kubaos update` runs; these private ones are what make a
//! pending security update visible in between.
//!
//! # Owns
//!
//! - The apt options that point apt at the private lists, and reading the simulation's output.
//!
//! # Not here
//!
//! - Running apt, and notifying: `updater.rs`. Installing the updates: `kubaos update`'s
//!   `apt-get upgrade`.

use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

/// The `-o` options that move apt's lists and package cache to `dir`, and leave everything else
/// (sources, dpkg's status, the auto-installed marks) as the system's.
pub fn apt_options(dir: &Path) -> Vec<OsString> {
    let mut options = Vec::new();
    for (key, value) in [
        ("Dir::State", dir.join("state")),
        ("Dir::Cache", dir.join("cache")),
        ("Dir::State::status", PathBuf::from("/var/lib/dpkg/status")),
        (
            "Dir::State::extended_states",
            PathBuf::from("/var/lib/apt/extended_states"),
        ),
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

/// The packages `apt-get -s upgrade` would install from a security pocket, as `name=version`,
/// sorted. A package in `-updates` and `-security` at once counts: apt lists every pocket that
/// carries the candidate version.
pub fn pending(simulated: &str) -> Vec<String> {
    let mut found: Vec<String> = simulated.lines().filter_map(security_line).collect();
    found.sort();
    found.dedup();
    found
}

/// `name=version` for an `Inst name [old] (new origin, origin [arch])` line from a security
/// pocket (`noble-security`, and ESM's `noble-apps-security`).
fn security_line(line: &str) -> Option<String> {
    let rest = line.strip_prefix("Inst ")?;
    let (name, rest) = rest.split_once(' ')?;
    let (_, candidate) = rest.split_once('(')?;
    let (version, origins) = candidate.split_once(' ')?;
    origins
        .split(", ")
        .any(|origin| origin.contains("-security"))
        .then(|| format!("{name}={version}"))
}

#[cfg(test)]
mod tests;
