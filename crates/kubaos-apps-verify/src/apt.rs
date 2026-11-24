//! apt repositories read from outside apt: a sources file, a signed release file checked with
//! `gpg` against the repository's pinned keys, and the `Packages` indexes it lists, each held to
//! its sha256 there.
//!
//! # Owns
//!
//! - Parsing a deb822 sources file ([`Sources`]), a release file's checksums ([`Release`]) and a
//!   `Packages` index's newest version of each package ([`newest`]).
//! - A key's primary fingerprints ([`fingerprints`]) and a release file verified with the keys, an
//!   `InRelease` or a `Release` and its `Release.gpg` ([`verify`]), in a `gpg` home of their own,
//!   so no keyring of the machine's is read or written.
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
use crate::fetch::Class;
use crate::fetch::FetchError;

/// A repository that couldn't be read or doesn't check out.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AptError {
    #[error(transparent)]
    Fetch(FetchError),
    #[error("running gpg")]
    Gpg(#[source] io::Error),
    #[error("gpg: {0}")]
    GpgFailed(String),
    #[error("{0}")]
    Invalid(String),
}

impl AptError {
    /// Whether the repository was out of reach, or its answer or the check is at fault.
    pub fn class(&self) -> Class {
        match self {
            Self::Fetch(e) => e.class(),
            _ => Class::Failure,
        }
    }
}

/// A one-stanza deb822 sources file, as `system/files/apt/sources.list.d/` holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sources {
    pub uri: String,
    /// A path ending in `/` for a flat repository, whose release file and one `Packages` index
    /// sit in that folder of the URI.
    pub suite: String,
    /// None for a flat repository.
    pub components: Vec<String>,
    /// Empty when the file names none, or `@ARCH@`, which the install fills with the machine's.
    pub architectures: Vec<String>,
}

impl Sources {
    /// Parses the first stanza of a sources file.
    ///
    /// # Errors
    ///
    /// [`AptError::Invalid`] when it has no `URIs` or `Suites`, or no `Components` for a suite
    /// that isn't flat.
    pub fn parse(text: &str) -> Result<Self, AptError> {
        let mut sources = Self::default();
        for line in text.lines().filter(|line| !line.starts_with('#')) {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            // apt takes a list's words apart at commas as well as spaces.
            let words = || {
                value
                    .split(|c: char| c.is_whitespace() || c == ',')
                    .filter(|word| !word.is_empty())
                    .map(str::to_owned)
            };
            match key.trim() {
                "URIs" => sources.uri = words().next().unwrap_or_default(),
                "Suites" => sources.suite = words().next().unwrap_or_default(),
                "Components" => sources.components = words().collect(),
                "Architectures" => sources.architectures = words().filter(|arch| arch != "@ARCH@").collect(),
                _ => {}
            }
        }
        if sources.uri.is_empty() || sources.suite.is_empty() || (!sources.flat() && sources.components.is_empty()) {
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

    /// Whether it names a flat repository: a suite that is a path, ending in `/`.
    pub fn flat(&self) -> bool {
        self.suite.ends_with('/')
    }

    /// The URL of the folder that holds the suite's release file: `…/dists/<suite>`, or a flat
    /// repository's path under the URI.
    pub fn dists(&self) -> String {
        let uri = self.uri.trim_end_matches('/');
        if !self.flat() {
            return format!("{uri}/dists/{}", self.suite);
        }
        match self.suite.trim_matches('/') {
            "" => uri.to_owned(),
            path => format!("{uri}/{path}"),
        }
    }
}

/// What a release file lists: each index's path in the suite and its sha256.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Release {
    pub sha256: BTreeMap<String, String>,
    /// `Acquire-By-Hash: yes`: each index is also served under `by-hash/SHA256/<its sha256>` in
    /// its folder, an address a mirror never rewrites, so a mirror caught between syncs serves
    /// the index this release lists or nothing, never a newer one.
    pub by_hash: bool,
}

impl Release {
    /// Whether it lists the index `base`, plain or gzipped.
    pub fn lists(&self, base: &str) -> bool {
        self.sha256.contains_key(base) || self.sha256.contains_key(&format!("{base}.gz"))
    }

    /// Parses a release file's `SHA256` section; the other fields are passed over.
    pub fn parse(text: &str) -> Self {
        let mut sha256 = BTreeMap::new();
        let mut by_hash = false;
        let mut inside = false;
        for line in text.lines() {
            if !line.starts_with(' ') {
                inside = line.trim_end() == "SHA256:";
                if let Some(value) = line.strip_prefix("Acquire-By-Hash:") {
                    by_hash = value.trim().eq_ignore_ascii_case("yes");
                }
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
        Self { sha256, by_hash }
    }

    /// Where the index `path` is fetched from under `dists`: by its sha256 where the suite serves
    /// indexes by hash, else by its name.
    pub fn url(&self, dists: &str, path: &str) -> String {
        match self.sha256.get(path).filter(|_| self.by_hash) {
            Some(hash) => {
                let folder = path.rsplit_once('/').map_or("", |(folder, _)| folder);
                let folder = if folder.is_empty() {
                    String::new()
                } else {
                    format!("{folder}/")
                };
                format!("{dists}/{folder}by-hash/SHA256/{hash}")
            }
            None => format!("{dists}/{path}"),
        }
    }
}

/// The newest version of each package a `Packages` index lists for `arch` (its own, or `all`), in
/// dpkg's order.
pub fn newest(packages: &str, arch: &str) -> BTreeMap<String, String> {
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
        if field("Architecture").is_some_and(|of| of != arch && of != "all") {
            continue;
        }
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

/// A release file whose signature checked out: the primary key that made it and the text signed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verified {
    pub primary: String,
    pub text: String,
}

/// How a suite signs its release file: an `InRelease` signed inline, or a `Release` with its
/// signature beside it in `Release.gpg`, which apt reads when the suite has no `InRelease`.
#[derive(Clone, Copy, Debug)]
pub enum Signed<'a> {
    Inline(&'a [u8]),
    Detached { release: &'a [u8], signature: &'a [u8] },
}

/// Checks the signature of a release file against the keys in `keys` (an armoured or binary key
/// file, or several one after another) and nothing else, and returns the signed text.
///
/// # Errors
///
/// [`AptError`] when gpg can't run, or the signature isn't good and made by a key of `keys`.
pub fn verify(keys: &[u8], signed: Signed<'_>) -> Result<Verified, AptError> {
    let home = tempfile::tempdir().map_err(AptError::Gpg)?;
    let key = home.path().join("key");
    let text = home.path().join("Release");
    fs::write(&key, keys).map_err(AptError::Gpg)?;
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
    let mut command = gpg(home.path());
    command.args(["--status-fd", "1"]);
    match signed {
        Signed::Inline(inrelease) => {
            let file = home.path().join("InRelease");
            fs::write(&file, inrelease).map_err(AptError::Gpg)?;
            command.arg("--output").arg(&text).arg("--decrypt").arg(file);
        }
        Signed::Detached { release, signature } => {
            let file = home.path().join("Release.gpg");
            fs::write(&text, release).map_err(AptError::Gpg)?;
            fs::write(&file, signature).map_err(AptError::Gpg)?;
            command.arg("--verify").arg(file).arg(&text);
        }
    }
    let output = command.output().map_err(AptError::Gpg)?;
    let status = String::from_utf8_lossy(&output.stdout);
    let primary = status.lines().find_map(|line| {
        let fields: Vec<&str> = line.strip_prefix("[GNUPG:] VALIDSIG ")?.split(' ').collect();
        fields.last().map(|fpr| fpr.to_ascii_uppercase())
    });
    // An expired or revoked key's signature has a VALIDSIG too, but an EXPKEYSIG or REVKEYSIG
    // where a good one has GOODSIG. A file signed by more keys than are pinned has an ERRSIG for
    // each other one, and gpg fails; apt takes it, as one good signature by a key it trusts is
    // what it asks for. A bad signature by any key is refused.
    let good = status.lines().any(|line| line.starts_with("[GNUPG:] GOODSIG "));
    let bad = status.lines().any(|line| line.starts_with("[GNUPG:] BADSIG "));
    let (true, false, Some(primary)) = (good, bad, primary) else {
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

/// The newest version of each package for `arch` in the suite under `dists`, over every index
/// `release` lists of its `components` for `arch` and for `all`, or a flat repository's one index
/// when there are none, each held to its sha256 there.
///
/// # Errors
///
/// [`AptError`] when the suite lists no index of a component for `arch`, or an index can't be
/// fetched, doesn't match its sha256 or can't be read.
pub fn index(
    dists: &str,
    release: &Release,
    components: &[&str],
    arch: &str,
) -> Result<BTreeMap<String, String>, AptError> {
    let mut indexes = Vec::new();
    if components.is_empty() {
        indexes.push("Packages".to_owned());
    }
    for component in components {
        let listed: Vec<String> = [arch, "all"]
            .iter()
            .map(|of| format!("{component}/binary-{of}/Packages"))
            .filter(|base| release.lists(base))
            .collect();
        if listed.is_empty() {
            return Err(AptError::Invalid(format!(
                "the suite lists no {component}/binary-{arch}/Packages index"
            )));
        }
        indexes.extend(listed);
    }
    let mut found: BTreeMap<String, String> = BTreeMap::new();
    for base in indexes {
        for (package, version) in packages(dists, release, &base, arch)? {
            let newer = found
                .get(&package)
                .is_none_or(|known| version::compare(&version, known).is_gt());
            if newer {
                found.insert(package, version);
            }
        }
    }
    Ok(found)
}

/// One index of the suite, `base` or `base.gz` where the suite has it, held to its sha256 there:
/// its newest version of each package for `arch`.
fn packages(dists: &str, release: &Release, base: &str, arch: &str) -> Result<BTreeMap<String, String>, AptError> {
    let gz = format!("{base}.gz");
    let (path, gzipped) = if release.sha256.contains_key(&gz) {
        (gz, true)
    } else if release.sha256.contains_key(base) {
        (base.to_owned(), false)
    } else {
        return Err(AptError::Invalid(format!("the suite lists no {base} index")));
    };
    let url = release.url(dists, &path);
    let body = fetch::get(&url).map_err(AptError::Fetch)?;
    let got = fetch::hex(&Sha256::digest(&body));
    if release.sha256.get(&path) != Some(&got) {
        return Err(AptError::Invalid(format!(
            "{path}'s sha256 {got} isn't the one the release file lists"
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
    Ok(newest(&text, arch))
}

/// Fetches `InRelease` from `dists` and verifies it with `keys`; for a suite with none, `Release`
/// and then its `Release.gpg`, as apt does. An `InRelease` that is down is reported as it is,
/// without looking for the others.
///
/// # Errors
///
/// [`AptError`] when neither can be fetched, or the signature doesn't verify with `keys`.
pub fn release(dists: &str, keys: &[u8]) -> Result<(Verified, Release), AptError> {
    let url = format!("{dists}/InRelease");
    let verified = match fetch::get(&url) {
        Ok(inrelease) => verify(keys, Signed::Inline(&inrelease))?,
        // A suite that is down serves no `Release` either; asking costs another round of tries.
        Err(down) if down.class() == Class::Outage => return Err(AptError::Fetch(down)),
        Err(missing) => {
            // The InRelease's error is the one to report when the suite has neither, unless the
            // `Release` was down; a missing `Release.gpg` beside a `Release` is its own error.
            let release = match fetch::get(&format!("{dists}/Release")) {
                Ok(release) => release,
                Err(down) if down.class() == Class::Outage => return Err(AptError::Fetch(down)),
                Err(_) => return Err(AptError::Fetch(missing)),
            };
            let signature = fetch::get(&format!("{dists}/Release.gpg")).map_err(AptError::Fetch)?;
            verify(
                keys,
                Signed::Detached {
                    release: &release,
                    signature: &signature,
                },
            )?
        }
    };
    let release = Release::parse(&verified.text);
    Ok((verified, release))
}

/// Fetches a key file.
///
/// # Errors
///
/// [`AptError::Fetch`] when it can't be fetched.
pub fn key(url: &str) -> Result<Vec<u8>, AptError> {
    fetch::get(url).map_err(AptError::Fetch)
}

#[cfg(test)]
mod tests;
