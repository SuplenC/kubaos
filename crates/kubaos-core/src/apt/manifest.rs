//! The package manifest, `system/packages.toml`: KubaOS's apt packages in groups, and the vendor
//! repositories some groups come from.
//!
//! The file keeps to the TOML the system scripts' awk reader understands (tables, strings
//! without escapes, booleans, arrays of strings); [`Manifest::parse`] refuses an escape, so a
//! value never reads differently in the two.
//!
//! # Owns
//!
//! - [`Manifest`], [`Group`] and [`Repo`], and which groups are active on a machine.
//!
//! # Not here
//!
//! - The user's choices: `choices.rs`. Comparing with the machine: `plan.rs`.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;

use super::choices::Choices;
use super::choices::Switch;

/// The manifest, inside the release's checkout.
pub const MANIFEST: &str = "system/packages.toml";

/// The manifest could not be read, or isn't one.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ManifestError {
    #[error("reading {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("the package manifest is not valid TOML of its shape")]
    Parse(#[source] toml::de::Error),
    #[error("group {group} takes its packages from repo {repo}, which the manifest doesn't define")]
    UnknownRepo { group: String, repo: String },
    #[error("group {group} lists {name:?}, which is not an apt package name")]
    InvalidPackage { group: String, name: String },
    #[error("{value:?} has a backslash or a quote, which the scripts' reader can't take")]
    Escaped { value: String },
    #[error("repo {repo} names some of its next key's next_key_url, next_fingerprint and next_keyring, not all three")]
    HalfNextKey { repo: String },
    #[error(
        "repo {repo}: {name:?} is not a keyring's file name in /etc/apt/trusted.gpg.d (a .gpg or .asc of letters, digits, ., _ and -, not starting with .)"
    )]
    InvalidGlobalKeyring { repo: String, name: String },
    #[error("group {group} has a when_on or when_off script but isn't optional")]
    ScriptNotOptional { group: String },
}

/// One group of packages.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    /// The table's name, `[group.<name>]`.
    #[serde(skip)]
    pub name: String,
    pub summary: String,
    pub packages: Vec<String>,
    /// The only architecture it is for, as `dpkg --print-architecture` names it.
    #[serde(default)]
    pub arch: Option<String>,
    /// The vendor repository its packages come from.
    #[serde(default)]
    pub repo: Option<String>,
    /// Whether its packages are held, so apt never moves them.
    #[serde(default)]
    pub hold: bool,
    /// Whether a package with no install candidate is skipped rather than reported.
    #[serde(default)]
    pub if_available: bool,
    /// Set when the user may turn it off or on: what it is until they do.
    #[serde(default)]
    pub optional: Option<Switch>,
    /// The script and its arguments, relative to the checkout, that sets up what the group needs
    /// beyond its packages when it is turned on.
    #[serde(default)]
    pub when_on: Vec<String>,
    /// The same, for turning it off, before its packages go.
    #[serde(default)]
    pub when_off: Vec<String>,
}

/// Why a group is not active on a machine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inactive {
    /// It is for another architecture.
    Arch(String),
    /// It is optional and off.
    Off,
    /// Its packages are the Apps catalog's (`kubaos app`), which the manifest leaves alone.
    Catalog,
}

impl Group {
    /// Whether the group is on: its choice when the user made one, else its default. A group that
    /// isn't optional is always on.
    pub fn enabled(&self, choices: &Choices) -> bool {
        match self.optional {
            None => true,
            Some(default) => choices.get(&self.name).unwrap_or(default) == Switch::On,
        }
    }

    /// Why the group is inactive on a machine of `arch` with `choices`, or `None` when its
    /// packages belong there.
    pub fn inactive(&self, arch: &str, choices: &Choices) -> Option<Inactive> {
        if let Some(only) = &self.arch
            && only != arch
        {
            return Some(Inactive::Arch(only.clone()));
        }
        (!self.enabled(choices)).then_some(Inactive::Off)
    }
}

/// A vendor's apt repository. Its sources file is `system/files/apt/sources.list.d/<name>.sources`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Repo {
    pub key_url: String,
    /// The primary key's fingerprint, the only one the key may hold.
    pub fingerprint: String,
    /// Where the key is stored, as the sources file's `Signed-By` names it.
    pub keyring: String,
    /// The key the vendor signs the repository with once `fingerprint`'s expires, its three keys
    /// set together: the sources file's `Signed-By` names both keyrings, so the repository verifies
    /// before and after the vendor moves to it.
    #[serde(default)]
    pub next_key_url: Option<String>,
    #[serde(default)]
    pub next_fingerprint: Option<String>,
    #[serde(default)]
    pub next_keyring: Option<String>,
    /// Files of `/etc/apt/trusted.gpg.d` the vendor's own setup writes, where apt trusts their
    /// keys for every repository: repo setup removes each whose keys are all `fingerprint` or one
    /// of `global_fingerprints`, as the sources file names the repository's key itself.
    #[serde(default)]
    pub global_keyrings: Vec<String>,
    /// The vendor's older keys, which its setup trusts in `global_keyrings` beside its own.
    #[serde(default)]
    pub global_fingerprints: Vec<String>,
}

impl Repo {
    /// The keyrings its sources file is signed by: its key's, then its next key's.
    pub fn keyrings(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.keyring.as_str()).chain(self.next_keyring.as_deref())
    }

    /// Whether the repository `name` is set up as repo setup leaves it on the machine whose `/` is
    /// `root`: `/etc/apt/sources.list.d/<name>.sources` is there, each of its `Signed-By` fields
    /// names every one of [`Repo::keyrings`], and each keyring is a file. A vendor's own source of
    /// the repository is another file, or signed by another keyring, so it isn't this.
    #[must_use]
    pub fn in_place(&self, name: &str, root: &Path) -> bool {
        let at = |path: &str| root.join(path.trim_start_matches('/'));
        let Ok(text) = fs::read_to_string(at(&format!("{SOURCES_D}/{name}.sources"))) else {
            return false;
        };
        let mut signed = text
            .lines()
            .filter_map(|line| line.strip_prefix("Signed-By:"))
            .peekable();
        signed.peek().is_some()
            && signed.all(|value| {
                let named: Vec<&str> = value.split_whitespace().collect();
                self.keyrings().all(|k| named.contains(&k))
            })
            && self.keyrings().all(|k| at(k).is_file())
    }
}

/// Where apt reads its sources files, and repo setup writes a repository's.
const SOURCES_D: &str = "/etc/apt/sources.list.d";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    #[serde(default)]
    repo: BTreeMap<String, Repo>,
    #[serde(default)]
    group: BTreeMap<String, Group>,
}

/// The package manifest.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Manifest {
    repos: BTreeMap<String, Repo>,
    /// In the file's order.
    groups: Vec<Group>,
}

impl Manifest {
    /// Parses the manifest's text.
    ///
    /// # Errors
    ///
    /// [`ManifestError`] when it isn't TOML of the manifest's shape, a group names a repo that
    /// isn't there or a package that can't be one, or a value has an escape.
    pub fn parse(text: &str) -> Result<Self, ManifestError> {
        let file: File = toml::from_str(text).map_err(ManifestError::Parse)?;
        let mut groups = file.group;
        let mut ordered = Vec::with_capacity(groups.len());
        for name in group_order(text) {
            if let Some(mut group) = groups.remove(&name) {
                group.name = name;
                ordered.push(group);
            }
        }
        // A group the line scan missed, such as an inline table, still counts, after the rest.
        for (name, mut group) in groups {
            group.name = name;
            ordered.push(group);
        }
        let manifest = Self {
            repos: file.repo,
            groups: ordered,
        };
        manifest.check()?;
        Ok(manifest)
    }

    /// Reads the manifest at `path`; `None` when there is none, as in a release from before it.
    ///
    /// # Errors
    ///
    /// [`ManifestError`] when it can't be read or parsed.
    pub fn read(path: &Path) -> Result<Option<Self>, ManifestError> {
        match fs::read_to_string(path) {
            Ok(text) => Self::parse(&text).map(Some),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(ManifestError::Read {
                path: path.to_path_buf(),
                source,
            }),
        }
    }

    fn check(&self) -> Result<(), ManifestError> {
        for (name, repo) in &self.repos {
            let next = [&repo.next_key_url, &repo.next_fingerprint, &repo.next_keyring];
            if next.iter().any(|v| v.is_some()) && !next.iter().all(|v| v.is_some()) {
                return Err(ManifestError::HalfNextKey { repo: name.clone() });
            }
            if let Some(keyring) = repo.global_keyrings.iter().find(|k| !is_keyring_name(k)) {
                return Err(ManifestError::InvalidGlobalKeyring {
                    repo: name.clone(),
                    name: keyring.clone(),
                });
            }
            let values = [&repo.key_url, &repo.fingerprint, &repo.keyring]
                .into_iter()
                .chain(next.into_iter().flatten())
                .chain(&repo.global_keyrings)
                .chain(&repo.global_fingerprints);
            for value in values {
                unescaped(value)?;
            }
        }
        for group in &self.groups {
            if let Some(repo) = &group.repo
                && !self.repos.contains_key(repo)
            {
                return Err(ManifestError::UnknownRepo {
                    group: group.name.clone(),
                    repo: repo.clone(),
                });
            }
            if group.optional.is_none() && !(group.when_on.is_empty() && group.when_off.is_empty()) {
                return Err(ManifestError::ScriptNotOptional {
                    group: group.name.clone(),
                });
            }
            for name in &group.packages {
                if !is_package_name(name) {
                    return Err(ManifestError::InvalidPackage {
                        group: group.name.clone(),
                        name: name.clone(),
                    });
                }
            }
            let words = group.when_on.iter().chain(&group.when_off);
            for value in [&group.summary].into_iter().chain(group.arch.iter()).chain(words) {
                unescaped(value)?;
            }
        }
        Ok(())
    }

    /// The groups, in the file's order.
    pub fn groups(&self) -> &[Group] {
        &self.groups
    }

    /// The group named `name`.
    pub fn group(&self, name: &str) -> Option<&Group> {
        self.groups.iter().find(|g| g.name == name)
    }

    /// The repositories, by name.
    pub fn repos(&self) -> &BTreeMap<String, Repo> {
        &self.repos
    }

    /// Whether any group, active or not, lists `package`.
    pub fn lists(&self, package: &str) -> bool {
        self.groups.iter().any(|g| g.packages.iter().any(|p| p == package))
    }
}

/// The names of the `[group.<name>]` tables, in the order the text has them.
fn group_order(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("[group.")?.split(']').next())
        .map(|name| name.trim().to_owned())
        .collect()
}

/// Whether `name` can be a Debian package name: lowercase letters, digits, `+`, `-` and `.`,
/// at least two, starting with a letter or digit.
/// <https://www.debian.org/doc/debian-policy/ch-controlfields.html#source>
pub fn is_package_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && name.len() >= 2
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '+' | '-' | '.'))
}

/// Whether `name` is a keyring's file name of `/etc/apt/trusted.gpg.d`: letters, digits, `.`,
/// `_` and `-` only, so the shell can't split or glob it, not starting with `.`, and a `.gpg` or
/// `.asc`, as apt reads no other there.
fn is_keyring_name(name: &str) -> bool {
    let stem = name.strip_suffix(".gpg").or_else(|| name.strip_suffix(".asc"));
    stem.is_some_and(|s| !s.is_empty() && !s.starts_with('.'))
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

fn unescaped(value: &str) -> Result<(), ManifestError> {
    if value.contains(['\\', '"']) {
        Err(ManifestError::Escaped {
            value: value.to_owned(),
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
