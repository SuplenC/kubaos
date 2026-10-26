//! How a machine compares with the manifest: what `kubaos packages status` shows, and what a
//! sync installs, holds and removes. Pure: the facts come in, read by the caller.
//!
//! A package is *wanted* when an active group lists it. A sync installs the wanted packages the
//! machine lacks, holds those of a held group, and removes a package only when KubaOS installed
//! it, no active group wants it, and either no group lists it any more or its group is the one
//! being turned off. Turning a group off is the only way an update-free removal happens, so a
//! group the install left off never loses a package the user installed by hand.
//!
//! # Owns
//!
//! - [`Facts`], [`Status`] and the sync's choices: [`missing`], [`repos_for`], [`to_hold`],
//!   [`removable`].
//!
//! # Not here
//!
//! - Reading the facts and running apt: the updater's `manifest.rs`.

use std::collections::BTreeSet;

use serde::Serialize;

use super::choices::Choices;
use super::choices::Switch;
use super::dpkg::Dpkg;
use super::manifest::Group;
use super::manifest::Inactive;
use super::manifest::Manifest;
use super::record::Record;

/// What is known about one machine.
#[derive(Clone, Copy, Debug)]
pub struct Facts<'a> {
    pub manifest: &'a Manifest,
    pub choices: &'a Choices,
    pub record: &'a Record,
    /// As `dpkg --print-architecture` names it.
    pub arch: &'a str,
    pub dpkg: &'a Dpkg,
}

impl Facts<'_> {
    /// The groups whose packages belong on the machine, in the manifest's order.
    pub fn active(&self) -> impl Iterator<Item = &Group> {
        self.manifest
            .groups()
            .iter()
            .filter(|g| g.inactive(self.arch, self.choices).is_none())
    }

    /// Whether an active group lists `package`.
    pub fn wanted(&self, package: &str) -> bool {
        self.active().any(|g| g.packages.iter().any(|p| p == package))
    }
}

/// A package a sync installs, and the first active group that lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Want {
    pub package: String,
    pub group: String,
}

/// The wanted packages the machine lacks, each once, in the manifest's order.
pub fn missing(facts: &Facts) -> Vec<Want> {
    let mut seen = BTreeSet::new();
    let mut wants = Vec::new();
    for group in facts.active() {
        for package in &group.packages {
            if !facts.dpkg.has(package) && seen.insert(package.as_str()) {
                wants.push(Want {
                    package: package.clone(),
                    group: group.name.clone(),
                });
            }
        }
    }
    wants
}

/// The vendor repositories `wants` come from, each once, sorted.
pub fn repos_for(manifest: &Manifest, wants: &[Want]) -> Vec<String> {
    let repos: BTreeSet<&String> = wants
        .iter()
        .filter_map(|w| manifest.group(&w.group)?.repo.as_ref())
        .collect();
    repos.into_iter().cloned().collect()
}

/// Whether a missing package with no install candidate is passed over without a word, as a
/// group that takes only what the architecture has wants.
pub fn quietly_unavailable(manifest: &Manifest, want: &Want) -> bool {
    manifest.group(&want.group).is_some_and(|g| g.if_available)
}

/// The installed packages of active held groups that aren't held yet, sorted.
pub fn to_hold(facts: &Facts) -> Vec<String> {
    let held: BTreeSet<&String> = facts
        .active()
        .filter(|g| g.hold)
        .flat_map(|g| &g.packages)
        .filter(|p| facts.dpkg.get(p).is_some_and(|i| !i.held))
        .collect();
    held.into_iter().cloned().collect()
}

/// The packages a sync may remove, sorted: installed by KubaOS, still there, wanted by no
/// active group, and either listed by no group or in `turning_off`.
pub fn removable(facts: &Facts, turning_off: Option<&str>) -> Vec<String> {
    let off = turning_off.and_then(|name| facts.manifest.group(name));
    facts
        .record
        .iter()
        .map(|(package, _)| package)
        .filter(|p| facts.dpkg.has(p) && !facts.wanted(p))
        .filter(|p| !facts.manifest.lists(p) || off.is_some_and(|g| g.packages.iter().any(|q| q == p)))
        .map(str::to_owned)
        .collect()
}

/// Where one package stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageState {
    Installed,
    /// Installed and on hold.
    Held,
    /// Wanted and not installed: the next update installs it.
    Missing,
    /// Wanted by a group that takes only what the architecture has, and apt has none.
    Unavailable,
    /// Installed by KubaOS, and no group lists it now: the next update removes it.
    Extra,
    /// Not installed, and its group is off or for another architecture.
    Skipped,
}

impl PackageState {
    /// The state's word, as the table shows it.
    pub fn word(self) -> &'static str {
        match self {
            Self::Installed => "installed",
            Self::Held => "held",
            Self::Missing => "missing",
            Self::Unavailable => "unavailable",
            Self::Extra => "extra",
            Self::Skipped => "skipped",
        }
    }
}

/// One package of the status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PackageStatus {
    pub name: String,
    /// The group that lists it, or for an extra one, the group KubaOS installed it for.
    pub group: String,
    pub state: PackageState,
    /// The installed version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Whether KubaOS installed it.
    pub kubaos: bool,
}

/// One group of the status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GroupStatus {
    pub name: String,
    pub summary: String,
    /// Whether its packages belong on the machine.
    pub active: bool,
    /// Why not: `off`, or the architecture it is for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inactive: Option<String>,
    /// Its default, when the user may turn it on or off.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<Switch>,
    pub packages: Vec<PackageStatus>,
}

/// How the machine compares with the manifest: `kubaos packages status`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Status {
    pub arch: String,
    pub groups: Vec<GroupStatus>,
    /// The packages KubaOS installed that no group lists now.
    pub extra: Vec<PackageStatus>,
}

/// The packages whose install candidate [`Status::new`] needs: the missing ones of active groups
/// that take only what apt has.
pub fn status_candidates(facts: &Facts) -> Vec<String> {
    missing(facts)
        .into_iter()
        .filter(|w| quietly_unavailable(facts.manifest, w))
        .map(|w| w.package)
        .collect()
}

impl Status {
    /// The status of the machine `facts` describes. `candidates` holds which of
    /// [`status_candidates`] apt can install.
    pub fn new(facts: &Facts, candidates: &BTreeSet<String>) -> Self {
        let groups = facts
            .manifest
            .groups()
            .iter()
            .map(|group| {
                let inactive = group.inactive(facts.arch, facts.choices);
                let packages = group
                    .packages
                    .iter()
                    .map(|name| {
                        let installed = facts.dpkg.get(name);
                        let state = match (installed, &inactive) {
                            (Some(i), _) if i.held => PackageState::Held,
                            (Some(_), _) => PackageState::Installed,
                            (None, Some(_)) => PackageState::Skipped,
                            (None, None) if group.if_available && !candidates.contains(name) => {
                                PackageState::Unavailable
                            }
                            (None, None) => PackageState::Missing,
                        };
                        PackageStatus {
                            name: name.clone(),
                            group: group.name.clone(),
                            state,
                            version: installed.map(|i| i.version.clone()),
                            kubaos: facts.record.has(name),
                        }
                    })
                    .collect();
                GroupStatus {
                    name: group.name.clone(),
                    summary: group.summary.clone(),
                    active: inactive.is_none(),
                    inactive: inactive.map(|why| match why {
                        Inactive::Off => "off".to_owned(),
                        Inactive::Arch(arch) => format!("{arch} only"),
                    }),
                    optional: group.optional,
                    packages,
                }
            })
            .collect();
        let extra = facts
            .record
            .iter()
            .filter(|(package, _)| !facts.manifest.lists(package))
            .filter_map(|(package, group)| {
                let installed = facts.dpkg.get(package)?;
                Some(PackageStatus {
                    name: package.to_owned(),
                    group: group.to_owned(),
                    state: PackageState::Extra,
                    version: Some(installed.version.clone()),
                    kubaos: true,
                })
            })
            .collect();
        Self {
            arch: facts.arch.to_owned(),
            groups,
            extra,
        }
    }
}

#[cfg(test)]
mod tests;
