//! Bitwarden desktop's releases on GitHub, as the update's Bitwarden phase reads them: which one
//! to move to, and whether a download is the one GitHub published.
//!
//! `system/70-bitwarden.sh` installs one pinned `.deb`; an update moves it on to the newest
//! release of the same major series (the year, as Bitwarden numbers them: `2026.10.1`). A new
//! series waits for a KubaOS release that pins it.
//!
//! # Owns
//!
//! - Reading GitHub's release list, the choice of release ([`choose`]), the download's URL, and the
//!   digest check ([`check_digest`]).
//!
//! # Not here
//!
//! - Running curl, sha256sum and apt: `updater/bitwarden.rs`.

use serde::Deserialize;

use super::version::Version;

/// GitHub's list of the repository's newest releases. The repository releases the web vault,
/// the browser extension, the desktop app and the CLI from one list, so a page of 100 reaches back
/// past several desktop releases.
pub const RELEASES_API: &str = "https://api.github.com/repos/bitwarden/clients/releases?per_page=100";
/// Where the release assets download from.
const DOWNLOADS: &str = "https://github.com/bitwarden/clients/releases/download";
/// What starts a desktop release's tag, `desktop-v2026.10.1`.
const TAG_PREFIX: &str = "desktop-v";
/// What starts a digest GitHub publishes in SHA-256.
const SHA256_PREFIX: &str = "sha256:";

/// One desktop release on GitHub.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub version: Version,
    /// The digest GitHub publishes for the release's `.deb`, `sha256:<hex>`; none when it has no
    /// such asset or publishes no digest for it.
    pub digest: Option<String>,
}

impl Release {
    /// The `.deb`'s file name, `Bitwarden-2026.10.1-amd64.deb`. Bitwarden publishes none for
    /// another architecture.
    pub fn asset(&self) -> String {
        asset_name(&self.version)
    }

    /// Where the `.deb` downloads from, as `system/70-bitwarden.sh` takes it.
    pub fn url(&self) -> String {
        format!("{DOWNLOADS}/{TAG_PREFIX}{}/{}", self.version, self.asset())
    }
}

fn asset_name(version: &Version) -> String {
    format!("Bitwarden-{version}-amd64.deb")
}

/// What an update does with Bitwarden.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Choice {
    /// The newest release of the installed series, when it is newer than the installed one.
    pub update: Option<Release>,
    /// The newest release of a later series, which waits for a KubaOS release.
    pub waiting: Option<Version>,
}

/// GitHub's release list could not be read.
#[derive(Debug, thiserror::Error)]
#[error("reading GitHub's list of Bitwarden releases")]
pub struct ReleasesError(#[source] serde_json::Error);

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    #[serde(default)]
    digest: Option<String>,
}

/// The published desktop releases in GitHub's release list: no drafts, no pre-releases, and no
/// tag that isn't a version.
///
/// # Errors
///
/// [`ReleasesError`] when the text is not GitHub's list of releases.
pub fn parse_releases(json: &str) -> Result<Vec<Release>, ReleasesError> {
    let releases: Vec<GitHubRelease> = serde_json::from_str(json).map_err(ReleasesError)?;
    Ok(releases
        .into_iter()
        .filter(|release| !release.draft && !release.prerelease)
        .filter_map(|release| {
            let version = Version::parse(release.tag_name.strip_prefix(TAG_PREFIX)?).ok()?;
            if version.is_pre() {
                return None;
            }
            let name = asset_name(&version);
            let digest = release
                .assets
                .into_iter()
                .find(|asset| asset.name == name)
                .and_then(|asset| asset.digest);
            Some(Release { version, digest })
        })
        .collect())
}

/// The release to move to from `installed`: the newest of its major series that is newer, and
/// the newest of a later series, which waits.
pub fn choose(installed: &Version, releases: &[Release]) -> Choice {
    let update = releases
        .iter()
        .filter(|release| release.version.major == installed.major && release.version > *installed)
        .max_by(|a, b| a.version.cmp(&b.version))
        .cloned();
    let waiting = releases
        .iter()
        .map(|release| &release.version)
        .filter(|version| version.major > installed.major)
        .max()
        .cloned();
    Choice { update, waiting }
}

/// The installed version from `dpkg-query -W -f '${Version}' bitwarden`; none when it isn't one.
pub fn installed_version(text: &str) -> Option<Version> {
    Version::parse(text).ok().filter(|version| !version.is_pre())
}

/// A download that is not the asset GitHub published.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum DigestError {
    /// GitHub publishes no SHA-256 digest for the asset, so nothing can vouch for it.
    #[error("GitHub publishes no sha256 digest for it")]
    Missing,
    /// sha256sum printed no digest.
    #[error("sha256sum printed {0:?}, not a digest")]
    Unreadable(String),
    /// The download's digest differs from the published one.
    #[error("its sha256 is {actual}, GitHub publishes {published}")]
    Mismatch { published: String, actual: String },
}

/// Checks the download's digest, `sha256sum`'s output for it, against the one GitHub publishes.
///
/// # Errors
///
/// [`DigestError`] unless GitHub publishes a SHA-256 digest and the download has it.
pub fn check_digest(published: Option<&str>, sha256sum: &str) -> Result<(), DigestError> {
    let published = published
        .and_then(|digest| digest.strip_prefix(SHA256_PREFIX))
        .filter(|hex| is_sha256(hex))
        .ok_or(DigestError::Missing)?
        .to_ascii_lowercase();
    let actual = sha256sum.split_whitespace().next().unwrap_or_default();
    if !is_sha256(actual) {
        return Err(DigestError::Unreadable(sha256sum.trim().to_owned()));
    }
    let actual = actual.to_ascii_lowercase();
    if actual == published {
        Ok(())
    } else {
        Err(DigestError::Mismatch { published, actual })
    }
}

fn is_sha256(hex: &str) -> bool {
    hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests;
