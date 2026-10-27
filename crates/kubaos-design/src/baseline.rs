//! The lint's baseline: how many violations of each rule each file had when the lint arrived, and
//! may keep until its front end is aligned. It only shrinks.
//!
//! The counts are per file and rule, not per line, so moving code around a file never trips it. A
//! file over its count fails with every violation of that rule listed; a file under it fails too,
//! until the baseline is shrunk to match ([`shrink`]), so a fix can't be undone unnoticed. No
//! command grows it: a file new to it, or a count going up, is a hand edit a reviewer sees.
//!
//! # Owns
//!
//! - Reading and writing `design/lint-baseline.toml`, comparing a scan with it, and shrinking it.
//!
//! # Not here
//!
//! - Finding violations: [`crate::lint`].

use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use crate::Error;
use crate::lint::Rule;
use crate::lint::Scan;
use crate::lint::Violation;

/// Where the baseline lives, relative to the repo's root.
pub const BASELINE_PATH: &str = "design/lint-baseline.toml";

const HEADER: &str = "\
# The design lint's baseline (`kubaos-design lint`, docs/DESIGN.md, \"Checks\"): how many
# literal design values and stray components each file may still have, by rule, until its
# front end is aligned onto the tokens and the components.
#
# It only shrinks. A file over its count fails the lint; a file under it fails too until
# `just design-baseline` writes the lower count here. Nothing writes a higher one: a new
# violation is fixed, or allowed in place with a `design-allow(rule): reason` marker.
";

/// Allowed violations, by file and rule.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Baseline {
    pub files: BTreeMap<String, BTreeMap<Rule, usize>>,
}

/// How a file and rule stand against the baseline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem<'a> {
    /// More violations than allowed: each one of the rule in the file, since any may be the new one.
    Over {
        path: &'a str,
        rule: Rule,
        allowed: usize,
        found: Vec<&'a Violation>,
    },
    /// Fewer than allowed: the baseline must shrink.
    Under {
        path: String,
        rule: Rule,
        allowed: usize,
        found: usize,
    },
}

impl fmt::Display for Problem<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Problem::Over {
                path,
                rule,
                allowed,
                found,
            } => {
                write!(
                    f,
                    "{path}: {} {rule} violation(s), the baseline allows {allowed}; fix the new one, or mark it \
                     `// design-allow({rule}): why`",
                    found.len()
                )?;
                for violation in found {
                    write!(f, "\n  {violation}")?;
                }
                Ok(())
            }
            Problem::Under {
                path,
                rule,
                allowed,
                found,
            } => write!(
                f,
                "{path}: {found} {rule} violation(s), the baseline still allows {allowed}; run `just design-baseline` \
                 to shrink it"
            ),
        }
    }
}

impl Baseline {
    /// Reads the baseline under `root`; none there is an empty one.
    ///
    /// # Errors
    ///
    /// When the file can't be read, or isn't counts by file and rule.
    pub fn load(root: &Path) -> Result<Baseline, Error> {
        let path = root.join(BASELINE_PATH);
        match fs::read_to_string(&path) {
            Ok(text) => Baseline::parse(&text),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Baseline::default()),
            Err(source) => Err(Error::read(&path, source)),
        }
    }

    /// Reads a baseline from its text.
    ///
    /// # Errors
    ///
    /// [`Error::Baseline`] when it isn't counts by file and known rule.
    pub fn parse(text: &str) -> Result<Baseline, Error> {
        let raw: BTreeMap<String, BTreeMap<String, usize>> = toml::from_str(text).map_err(Error::Baseline)?;
        let mut files = BTreeMap::new();
        for (path, counts) in raw {
            let mut rules = BTreeMap::new();
            for (name, count) in counts {
                let rule = Rule::from_name(&name).ok_or_else(|| Error::UnknownRule {
                    path: path.clone(),
                    rule: name,
                })?;
                rules.insert(rule, count);
            }
            files.insert(path, rules);
        }
        Ok(Baseline { files })
    }

    /// The counts of every violation in `scan`.
    pub fn of(scan: &Scan) -> Baseline {
        let mut files: BTreeMap<String, BTreeMap<Rule, usize>> = BTreeMap::new();
        for violation in &scan.violations {
            *files
                .entry(violation.path.clone())
                .or_default()
                .entry(violation.rule)
                .or_default() += 1;
        }
        Baseline { files }
    }

    /// The baseline as its file's text.
    pub fn render(&self) -> String {
        let mut text = HEADER.to_owned();
        for (path, rules) in &self.files {
            let counts: Vec<_> = rules.iter().filter(|&(_, &count)| count > 0).collect();
            if counts.is_empty() {
                continue;
            }
            let _ = write!(text, "\n[{path:?}]\n");
            for (rule, count) in counts {
                let _ = writeln!(text, "{rule} = {count}");
            }
        }
        text
    }

    /// Writes the baseline under `root`.
    ///
    /// # Errors
    ///
    /// When the file can't be written.
    pub fn write(&self, root: &Path) -> Result<(), Error> {
        let path = root.join(BASELINE_PATH);
        fs::write(&path, self.render()).map_err(|source| Error::Write { path, source })
    }

    fn allowed(&self, path: &str, rule: Rule) -> usize {
        self.files
            .get(path)
            .and_then(|rules| rules.get(&rule))
            .copied()
            .unwrap_or(0)
    }
}

/// Every file and rule of `scan` over or under `baseline`.
pub fn compare<'a>(scan: &'a Scan, baseline: &Baseline) -> Vec<Problem<'a>> {
    let mut by_file: BTreeMap<(&str, Rule), Vec<&Violation>> = BTreeMap::new();
    for violation in &scan.violations {
        by_file
            .entry((violation.path.as_str(), violation.rule))
            .or_default()
            .push(violation);
    }
    let mut problems = Vec::new();
    for (&(path, rule), found) in &by_file {
        let allowed = baseline.allowed(path, rule);
        if found.len() > allowed {
            problems.push(Problem::Over {
                path,
                rule,
                allowed,
                found: found.clone(),
            });
        }
    }
    for (path, rules) in &baseline.files {
        for (&rule, &allowed) in rules {
            let found = by_file.get(&(path.as_str(), rule)).map_or(0, Vec::len);
            if found < allowed {
                problems.push(Problem::Under {
                    path: path.clone(),
                    rule,
                    allowed,
                    found,
                });
            }
        }
    }
    problems
}

/// `baseline` with each count lowered to what `scan` found, and files with none dropped.
///
/// # Errors
///
/// [`Error::Grows`] when a file has more than its count: the baseline never grows.
pub fn shrink(scan: &Scan, baseline: &Baseline) -> Result<Baseline, Error> {
    let found = Baseline::of(scan);
    let grown: Vec<String> = compare(scan, baseline)
        .iter()
        .filter_map(|problem| match problem {
            Problem::Over { path, rule, .. } => Some(format!("{path} ({rule})")),
            Problem::Under { .. } => None,
        })
        .collect();
    if !grown.is_empty() {
        return Err(Error::Grows(grown));
    }
    let mut files = BTreeMap::new();
    for (path, rules) in &baseline.files {
        let kept: BTreeMap<Rule, usize> = rules
            .keys()
            .map(|&rule| (rule, found.allowed(path, rule)))
            .filter(|&(_, count)| count > 0)
            .collect();
        if !kept.is_empty() {
            files.insert(path.clone(), kept);
        }
    }
    Ok(Baseline { files })
}

/// Whether git has ever had the baseline in the history of the repo at `root`. The first baseline
/// may be written only when it hasn't: a deleted one would otherwise come back at today's counts.
///
/// # Errors
///
/// [`Error::History`] when git can't be run or `root` isn't a git checkout, so the history is
/// unknown.
pub fn in_history(root: &Path) -> Result<bool, Error> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "--all", "--format=%H", "-1", "--", BASELINE_PATH])
        .output()
        .map_err(|source| Error::History(Some(source)))?;
    if !output.status.success() {
        return Err(Error::History(None));
    }
    Ok(!output.stdout.trim_ascii().is_empty())
}

#[cfg(test)]
mod tests;
