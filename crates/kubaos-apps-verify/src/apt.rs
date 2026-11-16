//! apt repositories read from outside apt: a sources file, a signed `InRelease` checked with `gpg`
//! against one key, and the `Packages` indexes it lists, each held to its sha256 there.
//!
//! # Owns
//!
//! - Parsing a deb822 sources file ([`Sources`]), a release file's checksums ([`Release`]) and a
//!   `Packages` index's newest version of each package ([`newest`]).
//! - A key's primary fingerprints ([`fingerprints`]) and an `InRelease` verified with it
//!   ([`verify`]), in a `gpg` home of their own, so no keyring of the machine's is read or written.
//! - A repository's index for one component and architecture ([`index`]).
//!
//! # Not here
//!
//! - Which repositories an app uses and what is reported: `check.rs`.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::io::Read as _;
use std::path::Path;
use std::process::Command;

use flate2::read::GzDecoder;
use kubaos_core::apps::version;
use sha2::Digest as _;
use sha2::Sha256;

use crate::fetch;
use crate::fetch::FetchError;

/// A repository that couldn't be read or doesn't check out.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AptError {
    #[error("fetching {url}")]
    Fetch {
        url: String,
        #[source]
        source: FetchError,
    },
    #[error("running gpg")]
    Gpg(#[source] io::Error),
    #[error("gpg: {0}")]
    GpgFailed(String),
    #[error("{0}")]
    Invalid(String),
}

impl AptError {
    fn fetch(url: &str) -> impl FnOnce(FetchError) -> Self {
        let url = url.to_owned();
        move |source| Self::Fetch { url, source }
    }
}

/// A one-stanza deb822 sources file, as `system/files/apt/sources.list.d/` holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sources {
    pub uri: String,
    pub suite: String,
    pub components: Vec<String>,
    /// Empty when the file names none, or `@ARCH@`, which the install fills with the machine's.
    pub architectures: Vec<String>,
}

impl Sources {
    /// Parses the first stanza of a sources file.
    ///
    /// # Errors
    ///
    /// [`AptError::Invalid`] when it has no `URIs`, `Suites` or `Components`.
    pub fn parse(text: &str) -> Result<Self, AptError> {
        let mut sources = Self::default();
        for line in text.lines().filter(|line| !line.starts_with('#')) {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let words = || value.split_whitespace().map(str::to_owned);
            match key.trim() {
                "URIs" => sources.uri = words().next().unwrap_or_default(),
                "Suites" => sources.suite = words().next().unwrap_or_default(),
                "Components" => sources.components = words().collect(),
                "Architectures" => sources.architectures = words().filter(|arch| arch != "@ARCH@").collect(),
                _ => {}
            }
        }
        if sources.uri.is_empty() || sources.suite.is_empty() || sources.components.is_empty() {
            return Err(AptError::Invalid(
                "the sources file lacks URIs, Suites or Components".to_owned(),
            ));
        }
        Ok(sources)
    }

    /// Whether the file allows `arch`.
    pub fn serves(&self, arch: &str) -> bool {
        self.architectures.is_empty() || self.architectures.iter().any(|each| each == arch)
    }

    /// The URL of the suite's folder, `…/dists/<suite>`.
    pub fn dists(&self) -> String {
        format!("{}/dists/{}", self.uri.trim_end_matches('/'), self.suite)
    }
}

/// What a release file lists: each index's path in the suite and its sha256.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Release {
    pub sha256: BTreeMap<String, String>,
}

impl Release {
    /// Parses a release file's `SHA256` section; the other fields are passed over.
    pub fn parse(text: &str) -> Self {
        let mut sha256 = BTreeMap::new();
        let mut inside = false;
        for line in text.lines() {
            if !line.starts_with(' ') {
                inside = line.trim_end() == "SHA256:";
                continue;
            }
            if !inside {
                continue;
            }
            let mut fields = line.split_whitespace();
            if let (Some(hash), Some(_size), Some(path)) = (fields.next(), fields.next(), fields.next()) {
                sha256.insert(path.to_owned(), hash.to_ascii_lowercase());
            }
        }
        Self { sha256 }
    }
}

/// The newest version of each package a `Packages` index lists, in dpkg's order.
pub fn newest(packages: &str) -> BTreeMap<String, String> {
    let mut found: BTreeMap<String, String> = BTreeMap::new();
    for stanza in packages.split("\n\n") {
        let field = |name: &str| {
            stanza
                .lines()
                .find_map(|line| line.strip_prefix(name)?.strip_prefix(':').map(str::trim))
        };
        let (Some(package), Some(version)) = (field("Package"), field("Version")) else {
            continue;
        };
        let newer = found
            .get(package)
            .is_none_or(|known| version::compare(version, known).is_gt());
        if newer {
            found.insert(package.to_owned(), version.to_owned());
        }
    }
    found
}

/// The primary fingerprints a key file holds, uppercase.
///
/// # Errors
///
/// [`AptError`] when gpg can't run or can't read the key.
pub fn fingerprints(key: &[u8]) -> Result<Vec<String>, AptError> {
    let home = tempfile::tempdir().map_err(AptError::Gpg)?;
    let file = home.path().join("key");
    fs::write(&file, key).map_err(AptError::Gpg)?;
    let output = gpg(home.path())
        .args(["--with-colons", "--show-keys"])
        .arg(&file)
        .output()
        .map_err(AptError::Gpg)?;
    if !output.status.success() {
        return Err(AptError::GpgFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(primary_fingerprints(&String::from_utf8_lossy(&output.stdout)))
}

/// The fingerprint of each `pub` record in gpg's `--with-colons` listing, from the `fpr` line
/// that follows it; a subkey's `fpr` follows its `sub`.
fn primary_fingerprints(listing: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut primary = false;
    for line in listing.lines() {
        let mut fields = line.split(':');
        match fields.next() {
            Some("pub") => primary = true,
            Some("sub") => primary = false,
            Some("fpr") if primary => {
                if let Some(fpr) = fields.nth(8) {
                    found.push(fpr.to_ascii_uppercase());
                }
                primary = false;
            }
            _ => {}
        }
    }
    found
}

/// An `InRelease` whose signature checked out: the primary key that made it and the text signed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verified {
    pub primary: String,
    pub text: String,
}

/// Checks the signature of `inrelease` against the keys in `keys` (an armoured or binary key file)
/// and nothing else, and returns the signed text.
///
/// # Errors
///
/// [`AptError`] when gpg can't run, or the signature isn't good and made by a key of `keys`.
pub fn verify(keys: &[u8], inrelease: &[u8]) -> Result<Verified, AptError> {
    let home = tempfile::tempdir().map_err(AptError::Gpg)?;
    let key = home.path().join("key");
    let signed = home.path().join("InRelease");
    let text = home.path().join("Release");
    fs::write(&key, keys).map_err(AptError::Gpg)?;
    fs::write(&signed, inrelease).map_err(AptError::Gpg)?;
    let imported = gpg(home.path())
        .arg("--import")
        .arg(&key)
        .output()
        .map_err(AptError::Gpg)?;
    if !imported.status.success() {
        return Err(AptError::GpgFailed(
            String::from_utf8_lossy(&imported.stderr).trim().to_owned(),
        ));
    }
    let output = gpg(home.path())
        .args(["--status-fd", "1", "--output"])
        .arg(&text)
        .arg("--decrypt")
        .arg(&signed)
        .output()
        .map_err(AptError::Gpg)?;
    let status = String::from_utf8_lossy(&output.stdout);
    let primary = status.lines().find_map(|line| {
        let fields: Vec<&str> = line.strip_prefix("[GNUPG:] VALIDSIG ")?.split(' ').collect();
        fields.last().map(|fpr| fpr.to_ascii_uppercase())
    });
    // An expired or revoked key's signature has a VALIDSIG too, but an EXPKEYSIG or REVKEYSIG
    // where a good one has GOODSIG.
    let good = status.lines().any(|line| line.starts_with("[GNUPG:] GOODSIG "));
    let (true, true, Some(primary)) = (output.status.success(), good, primary) else {
        let said = status
            .lines()
            .filter_map(|line| line.strip_prefix("[GNUPG:] "))
            .filter(|line| !line.starts_with("NEWSIG") && !line.starts_with("PLAINTEXT"))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(AptError::GpgFailed(format!("the signature didn't verify: {said}")));
    };
    let text = fs::read_to_string(&text).map_err(AptError::Gpg)?;
    Ok(Verified { primary, text })
}

/// gpg in its own home, with no agent, no keyserver and no trust database to update.
fn gpg(home: &Path) -> Command {
    let mut command = Command::new("gpg");
    command.arg("--homedir").arg(home).args([
        "--batch",
        "--no-tty",
        "--quiet",
        "--no-autostart",
        "--trust-model",
        "always",
    ]);
    command
}

/// The newest version of each package in `component` for `arch`, from the index `release` lists
/// under `dists`, held to its sha256 there: `Packages.gz` where the suite has it, else `Packages`.
///
/// # Errors
///
/// [`AptError`] when the suite lists no index for it, or the index can't be fetched, doesn't
/// match its sha256 or can't be read.
pub fn index(
    dists: &str,
    release: &Release,
    component: &str,
    arch: &str,
) -> Result<BTreeMap<String, String>, AptError> {
    let base = format!("{component}/binary-{arch}/Packages");
    let gz = format!("{base}.gz");
    let (path, gzipped) = if release.sha256.contains_key(&gz) {
        (gz, true)
    } else if release.sha256.contains_key(&base) {
        (base, false)
    } else {
        return Err(AptError::Invalid(format!("the suite lists no {base} index")));
    };
    let url = format!("{dists}/{path}");
    let body = fetch::get(&url).map_err(AptError::fetch(&url))?;
    let got = fetch::hex(&Sha256::digest(&body));
    if release.sha256.get(&path) != Some(&got) {
        return Err(AptError::Invalid(format!(
            "{path}'s sha256 {got} isn't the one InRelease lists"
        )));
    }
    let text = if gzipped {
        let mut text = String::new();
        GzDecoder::new(body.as_slice())
            .read_to_string(&mut text)
            .map_err(|e| AptError::Invalid(format!("{path}: {e}")))?;
        text
    } else {
        String::from_utf8(body).map_err(|e| AptError::Invalid(format!("{path}: {e}")))?
    };
    Ok(newest(&text))
}

/// Fetches `InRelease` from `dists` and verifies it with `keys`.
///
/// # Errors
///
/// [`AptError`] when it can't be fetched or its signature doesn't verify with `keys`.
pub fn release(dists: &str, keys: &[u8]) -> Result<(Verified, Release), AptError> {
    let url = format!("{dists}/InRelease");
    let inrelease = fetch::get(&url).map_err(AptError::fetch(&url))?;
    let verified = verify(keys, &inrelease)?;
    let release = Release::parse(&verified.text);
    Ok((verified, release))
}

/// Fetches a key file.
///
/// # Errors
///
/// [`AptError::Fetch`] when it can't be fetched.
pub fn key(url: &str) -> Result<Vec<u8>, AptError> {
    fetch::get(url).map_err(AptError::fetch(url))
}

#[cfg(test)]
mod tests;
