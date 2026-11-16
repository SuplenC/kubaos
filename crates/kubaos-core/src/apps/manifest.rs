//! One app's manifest, `apps/<id>.toml`: its name, a line on what it is, and the one source it
//! comes from.
//!
//! The file keeps to the TOML the system scripts' awk reader understands (tables, one-line
//! strings without escapes, booleans, arrays of strings), as the package manifest does: the root
//! helper reads it with `manifest_dump`. [`App::parse`] refuses an escape or a line break, so a
//! value never reads differently in the two.
//!
//! # Owns
//!
//! - [`App`], [`Source`] and what a valid manifest is.
//!
//! # Not here
//!
//! - The catalog of every manifest: `catalog.rs`. A vendor repository: `apt::manifest`'s
//!   [`Repo`](crate::apt::manifest::Repo).

use serde::Deserialize;
use serde::Serialize;

use crate::apt::manifest::is_package_name;
use crate::packages::validate_name;

/// The architectures a pinned download may be for, as `dpkg --print-architecture` names them.
pub const ARCHES: [&str; 2] = ["amd64", "arm64"];

/// A manifest that isn't one.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ManifestError {
    #[error("the manifest is not valid TOML of its shape")]
    Parse(#[source] toml::de::Error),
    #[error("{0:?} is not an app id: lowercase letters, digits and dashes, starting with a letter or digit")]
    InvalidId(String),
    #[error("an app comes from exactly one source, [apt], [nix] or [deb]; this has {0}")]
    Sources(usize),
    #[error("{key} is empty")]
    Empty { key: &'static str },
    #[error("{name:?} is not an apt package name")]
    InvalidPackage { name: String },
    #[error("{name:?} is not a nixpkgs attribute name")]
    InvalidAttribute { name: String },
    #[error("{value:?} is not a Debian version")]
    InvalidVersion { value: String },
    #[error("{value:?} is not an https:// URL")]
    InvalidUrl { value: String },
    #[error("{value:?} is not a sha256: 64 lowercase hex digits")]
    InvalidSha256 { value: String },
    #[error("{value:?} has a quote or a control character, which the scripts' reader can't take")]
    Unreadable { value: String },
    #[error("the manifest has an escape or a multi-line string, which the scripts' reader can't take")]
    Escaped,
}

/// One app of the catalog.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct App {
    /// The file's name without `.toml`, which `kubaos app` and the root helper take.
    pub id: String,
    /// The app's name, as people read it.
    pub name: String,
    /// One line on what it is.
    pub summary: String,
    pub source: Source,
}

/// Where an app comes from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Source {
    /// apt packages, from Ubuntu's archive or a vendor repository of the package manifest.
    Apt {
        /// The `[repo.<name>]` of `system/packages.toml` they come from.
        #[serde(skip_serializing_if = "Option::is_none")]
        repo: Option<String>,
        packages: Vec<String>,
    },
    /// Packages of the user's nixpkgs, in the User flake's `packages.toml`, as `kubaos package
    /// add` puts them there.
    Nix { packages: Vec<String> },
    /// A vendor's `.deb`, pinned to its sha256 for each architecture.
    Deb(Deb),
}

/// A `.deb` from a release URL.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Deb {
    /// The package the `.deb` installs, as dpkg lists it.
    pub package: String,
    /// The version the downloads are, as dpkg reports it once installed.
    pub version: String,
    /// Whether the app keeps itself current, so an installed one is never put back to the pin.
    pub self_updates: bool,
    /// The download for each architecture it has one for, by [`ARCHES`]' names.
    pub downloads: Vec<(String, Download)>,
}

/// One pinned download.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Download {
    pub url: String,
    pub sha256: String,
}

impl Deb {
    /// The download for `arch`, when there is one.
    pub fn download(&self, arch: &str) -> Option<&Download> {
        self.downloads.iter().find(|(a, _)| a == arch).map(|(_, d)| d)
    }
}

impl Source {
    /// The source's word: `apt`, `nix` or `deb`.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Apt { .. } => "apt",
            Self::Nix { .. } => "nix",
            Self::Deb(_) => "deb",
        }
    }

    /// Whether installing or removing it needs root, through the helper.
    pub fn needs_root(&self) -> bool {
        !matches!(self, Self::Nix { .. })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    name: String,
    summary: String,
    apt: Option<AptTable>,
    nix: Option<NixTable>,
    deb: Option<DebTable>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AptTable {
    repo: Option<String>,
    packages: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NixTable {
    packages: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DebTable {
    package: String,
    version: String,
    #[serde(default)]
    self_updates: bool,
    amd64: Option<Download>,
    arm64: Option<Download>,
}

impl App {
    /// Parses the manifest of the app `id`.
    ///
    /// # Errors
    ///
    /// [`ManifestError`] when `id` can't be an app's, the text isn't TOML of the manifest's shape,
    /// it has no source or more than one, or a value can't be what its key says.
    pub fn parse(id: &str, text: &str) -> Result<Self, ManifestError> {
        if !is_app_id(id) {
            return Err(ManifestError::InvalidId(id.to_owned()));
        }
        if text.contains('\\') || text.contains("\"\"\"") || text.contains("'''") {
            return Err(ManifestError::Escaped);
        }
        let file: File = toml::from_str(text).map_err(ManifestError::Parse)?;
        let sources = usize::from(file.apt.is_some()) + usize::from(file.nix.is_some()) + usize::from(file.deb.is_some());
        let source = match (file.apt, file.nix, file.deb) {
            (Some(apt), None, None) => Source::Apt {
                repo: apt.repo,
                packages: apt.packages,
            },
            (None, Some(nix), None) => Source::Nix { packages: nix.packages },
            (None, None, Some(deb)) => Source::Deb(Deb {
                package: deb.package,
                version: deb.version,
                self_updates: deb.self_updates,
                downloads: ARCHES
                    .into_iter()
                    .zip([deb.amd64, deb.arm64])
                    .filter_map(|(arch, d)| Some((arch.to_owned(), d?)))
                    .collect(),
            }),
            _ => return Err(ManifestError::Sources(sources)),
        };
        let app = Self {
            id: id.to_owned(),
            name: file.name,
            summary: file.summary,
            source,
        };
        app.check()?;
        Ok(app)
    }

    fn check(&self) -> Result<(), ManifestError> {
        non_empty("name", &self.name)?;
        non_empty("summary", &self.summary)?;
        let mut values = vec![&self.name, &self.summary];
        match &self.source {
            Source::Apt { repo, packages } => {
                non_empty_list("apt.packages", packages)?;
                for name in packages {
                    if !is_package_name(name) {
                        return Err(ManifestError::InvalidPackage { name: name.clone() });
                    }
                }
                values.extend(repo);
            }
            Source::Nix { packages } => {
                non_empty_list("nix.packages", packages)?;
                for name in packages {
                    validate_name(name).map_err(|_| ManifestError::InvalidAttribute { name: name.clone() })?;
                }
            }
            Source::Deb(deb) => {
                if !is_package_name(&deb.package) {
                    return Err(ManifestError::InvalidPackage {
                        name: deb.package.clone(),
                    });
                }
                if !is_version(&deb.version) {
                    return Err(ManifestError::InvalidVersion {
                        value: deb.version.clone(),
                    });
                }
                if deb.downloads.is_empty() {
                    return Err(ManifestError::Empty {
                        key: "deb.amd64 or deb.arm64",
                    });
                }
                for (_, download) in &deb.downloads {
                    if !is_url(&download.url) {
                        return Err(ManifestError::InvalidUrl {
                            value: download.url.clone(),
                        });
                    }
                    if !is_sha256(&download.sha256) {
                        return Err(ManifestError::InvalidSha256 {
                            value: download.sha256.clone(),
                        });
                    }
                    values.push(&download.url);
                }
            }
        }
        for value in values {
            if value.chars().any(|c| c == '"' || c.is_control()) {
                return Err(ManifestError::Unreadable { value: value.clone() });
            }
        }
        Ok(())
    }
}

/// Whether `id` can be an app's: lowercase ASCII letters, digits and `-`, starting with a letter
/// or a digit, at most 64. The root helper holds ids to the same pattern, so an id is never a
/// path.
pub fn is_app_id(id: &str) -> bool {
    id.len() <= 64
        && id.chars().next().is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Whether `value` can be a Debian version: letters, digits and `.+~:-`, starting with a digit.
/// <https://www.debian.org/doc/debian-policy/ch-controlfields.html#version>
fn is_version(value: &str) -> bool {
    value.chars().next().is_some_and(|c| c.is_ascii_digit())
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '~' | ':' | '-'))
}

fn is_url(value: &str) -> bool {
    value.strip_prefix("https://").is_some_and(|rest| {
        !rest.is_empty() && !rest.starts_with('/') && !rest.chars().any(|c| c.is_whitespace() || c.is_control())
    })
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|c| c.is_ascii_digit() || matches!(c, 'a'..='f'))
}

fn non_empty(key: &'static str, value: &str) -> Result<(), ManifestError> {
    if value.trim().is_empty() {
        Err(ManifestError::Empty { key })
    } else {
        Ok(())
    }
}

fn non_empty_list(key: &'static str, values: &[String]) -> Result<(), ManifestError> {
    if values.is_empty() {
        Err(ManifestError::Empty { key })
    } else {
        Ok(())
    }
}
