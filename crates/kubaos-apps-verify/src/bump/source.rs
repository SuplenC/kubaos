//! Where a pinned download's newer releases are found: GitHub releases, a few vendors' own release
//! feeds, and a check that an unversioned URL still serves the pinned file.
//!
//! # Owns
//!
//! - [`newest`]: for one architecture's pin, the newest release its vendor publishes, with its
//!   download and sha256.
//! - Each vendor's feed: which URLs it covers and how it is read, and Helm's pinned release keys,
//!   which its checksums are held to.
//!
//! # Not here
//!
//! - Reading and writing `app.toml`, and agreeing the architectures: `bump.rs`. Fetching:
//!   `fetch.rs`.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::sync::Mutex;

use kubaos_core::apps::version;

use crate::apt;
use crate::apt::Signed;
use crate::check::Problem;
use crate::check::chain;
use crate::check::digest_for;
use crate::fetch;

/// What GitHub's release downloads start with.
const GITHUB: &str = "https://github.com/";
/// `JetBrains` Toolbox's downloads, and the release API that names the newest.
const JETBRAINS_TOOLBOX: &str = "https://download.jetbrains.com/toolbox/";
const JETBRAINS_TOOLBOX_API: &str =
    "https://data.services.jetbrains.com/products/releases?code=TBA&latest=true&type=release";
/// Android Studio's downloads, and the list of its releases `JetBrains` keeps, which Google's
/// release notes link to.
const ANDROID_STUDIO: &str = "https://dl.google.com/dl/android/studio/ide-zips/";
const ANDROID_STUDIO_RELEASES: &str = "https://jb.gg/android-studio-releases-list.json";
/// The channels of Android Studio's list that are stable: a release and its patches.
const ANDROID_STUDIO_CHANNELS: [&str; 2] = ["Release", "Patch"];
/// Proton Pass's downloads, and the feed its own updater reads.
const PROTON_PASS: &str = "https://proton.me/download/pass/linux/";
const PROTON_PASS_RELEASES: &str = "https://proton.me/download/PassDesktop/linux/x64/version.json";
/// Blender's release folders, one a series (`Blender5.3/`), each listing its files.
const BLENDER: &str = "https://download.blender.org/release/";
/// Helm's downloads, which its GitHub releases name but don't carry: the releases say which is
/// the newest, and each tarball has its sha256 beside it, `<file>.sha256sum`. The release carries
/// that file's signature, `<file>.sha256sum.asc`, so the sha256 doesn't rest on `get.helm.sh`
/// alone: a host serving a tampered tarball could serve a checksum to match it, not a signature.
const HELM: &str = "https://get.helm.sh/";
const HELM_RELEASES: &str = "repos/helm/helm/releases?per_page=100";
/// The keys Helm's releases are signed with: its `KEYS` file as helm/helm keeps it, a key for
/// each maintainer who cuts releases, since any of them may sign the next one.
const HELM_KEYS: &[u8] = include_bytes!("../../keys/helm.asc");
/// The primary keys [`HELM_KEYS`] holds, which a test holds the file to, so a new key comes in
/// only by changing both: gpg verifies against that keyring alone, so a signature by any other key
/// is refused.
#[cfg(test)]
const HELM_FINGERPRINTS: [&str; 10] = [
    "672C657BE06B4B30969C4A57461449C25E36B98E",
    "F1261BDE929012C8FF2E501D6EA5D7598529A53E",
    "967F8AC5E2216F9F4FD270AD92AA783CBAAE8E3B",
    "76939899B137D575D3274E756DCCB9D752D35BA8",
    "49D09C86C3DC8DA3F0A076221EF612347F8A9958",
    "ABA2529598F6626C420D335B62F49E747D911B60",
    "4AB45F1CB0D292975C6371436E2A23D806B6E6DD",
    "208DD36ED5BB3745A16743A4C7C6FBB5B91C1155",
    "7FEC81FACC7FFB2A010ADD13C2D40F4D8196E874",
    "BF888333D96A1C18E2682AAED79D67C9EC016739",
];

/// One architecture's pin, as an action holds it.
#[derive(Clone, Copy, Debug)]
pub struct Pin<'a> {
    pub version: &'a str,
    /// `amd64` or `arm64`.
    pub arch: &'a str,
    pub url: &'a str,
    pub sha256: &'a str,
}

/// What a pin's vendor publishes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Found {
    /// The pin is the newest release.
    Current,
    /// A newer release, its download and that download's sha256.
    Newer {
        version: String,
        url: String,
        sha256: String,
    },
    /// Nothing says which release is the newest, for the reason given; not a failure.
    NoSource(String),
}

/// The newest release `pin`'s vendor publishes.
///
/// # Errors
///
/// A [`Problem`] when the vendor can't be read, which is a warning when it is down, or an error
/// when its newest release has no file for the pin or an unversioned URL serves another file than
/// the pinned one.
pub fn newest(pin: &Pin<'_>) -> Result<Found, Problem> {
    if let Some(release) = GitHubPin::parse(pin.url) {
        return release.newest(pin);
    }
    if pin.url.starts_with(JETBRAINS_TOOLBOX) {
        return jetbrains_toolbox(pin);
    }
    if pin.url.starts_with(ANDROID_STUDIO) {
        return android_studio(pin);
    }
    if pin.url.starts_with(PROTON_PASS) {
        return proton_pass(pin);
    }
    if pin.url.starts_with(BLENDER) {
        return blender(pin);
    }
    if pin.url.starts_with(HELM) {
        return helm(pin);
    }
    if !pin.url.contains(pin.version) {
        return unversioned(pin);
    }
    Ok(Found::NoSource(
        "no release feed is known for its host; its pin is moved by hand".to_owned(),
    ))
}

/// A pin of a GitHub release asset: `https://github.com/OWNER/REPO/releases/download/TAG/NAME`.
#[derive(Debug, PartialEq, Eq)]
pub struct GitHubPin<'a> {
    owner: &'a str,
    repo: &'a str,
    /// The tag as the URL spells it, `core%4013.4.0`.
    tag: &'a str,
    name: &'a str,
}

impl<'a> GitHubPin<'a> {
    pub fn parse(url: &'a str) -> Option<Self> {
        let parts: Vec<&str> = url.strip_prefix(GITHUB)?.splitn(6, '/').collect();
        let [owner, repo, "releases", "download", tag, name] = parts[..] else {
            return None;
        };
        Some(Self { owner, repo, tag, name })
    }

    fn newest(&self, pin: &Pin<'_>) -> Result<Found, Problem> {
        let tag = Tag::parse(&percent_decoded(self.tag)).map_err(Problem::error)?;
        let releases = fetch::github(&format!("repos/{}/{}/releases?per_page=100", self.owner, self.repo))
            .map_err(|e| Problem::fetch(&e))?;
        let Some(newest) = newest_tag(&releases, &tag.prefix) else {
            return Err(Problem::error(format!(
                "{}/{} has no release tagged {}<version>",
                self.owner, self.repo, tag.prefix
            )));
        };
        let Some(moved) = Moved::from(pin.version, &tag.version, &newest) else {
            return Ok(Found::Current);
        };
        let moved = moved.map_err(Problem::error)?;
        let name = moved.rename(self.name);
        let tag_name = format!("{}{newest}", tag.prefix);
        let assets = releases
            .as_array()
            .into_iter()
            .flatten()
            .find(|release| release["tag_name"].as_str() == Some(&tag_name))
            .map(|release| &release["assets"]);
        let Some(asset) = assets
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .find(|asset| asset["name"].as_str() == Some(&name))
        else {
            return Err(Problem::error(format!(
                "{tag_name} has no {name}: the release renamed or dropped the file the pin follows"
            )));
        };
        let spelled_prefix = self.tag.strip_suffix(&tag.version).unwrap_or(&tag.prefix);
        let url = format!(
            "{GITHUB}{}/{}/releases/download/{spelled_prefix}{newest}/{name}",
            self.owner, self.repo
        );
        let digest = asset["digest"]
            .as_str()
            .and_then(|digest| digest.strip_prefix("sha256:"))
            .map(str::to_ascii_lowercase);
        let sha256 = match digest {
            Some(digest) => digest,
            None => fetch::sha256(&url).map_err(|e| Problem::fetch(&e))?,
        };
        Ok(Found::Newer {
            version: moved.to,
            url,
            sha256,
        })
    }
}

/// A release tag: what comes before its version, `desktop-v`, `core@` or none, and the version.
#[derive(Debug, PartialEq, Eq)]
pub struct Tag {
    pub prefix: String,
    pub version: String,
}

impl Tag {
    /// # Errors
    ///
    /// A message when the tag has no digit to start a version.
    pub fn parse(tag: &str) -> Result<Self, String> {
        let start = tag
            .find(|c: char| c.is_ascii_digit())
            .ok_or_else(|| format!("tag {tag} has no version"))?;
        Ok(Self {
            prefix: tag[..start].to_owned(),
            version: tag[start..].to_owned(),
        })
    }
}

/// The newest version tagged `prefix<version>` in GitHub's release list, drafts and pre-releases
/// left out.
pub fn newest_tag(releases: &serde_json::Value, prefix: &str) -> Option<String> {
    releases
        .as_array()?
        .iter()
        .filter(|release| release["draft"] != true && release["prerelease"] != true)
        .filter_map(|release| release["tag_name"].as_str()?.strip_prefix(prefix))
        .filter(|version| version.starts_with(|c: char| c.is_ascii_digit()) && version::is_version(version))
        .max_by(|a, b| version::compare(a, b))
        .map(str::to_owned)
}

/// A pin's move from one release to a newer one.
#[derive(Debug, PartialEq, Eq)]
pub struct Moved {
    from: String,
    to: String,
    from_release: String,
    to_release: String,
}

impl Moved {
    /// The move from the pin's `version`, which is the release's `from_release` with a suffix of
    /// its own, as Sunshine's `2026.914.233613-1+ubuntu26.04` is release `2026.914.233613`'s; none
    /// when `to_release` is no newer.
    ///
    /// # Errors
    ///
    /// A message when the pin's version doesn't start with its release's.
    pub fn from(version: &str, from_release: &str, to_release: &str) -> Option<Result<Self, String>> {
        if version::compare(to_release, from_release) != Ordering::Greater {
            return None;
        }
        let Some(suffix) = version.strip_prefix(from_release) else {
            return Some(Err(format!(
                "the pin's version {version} doesn't start with its release's, {from_release}"
            )));
        };
        Some(Ok(Self {
            from: version.to_owned(),
            to: format!("{to_release}{suffix}"),
            from_release: from_release.to_owned(),
            to_release: to_release.to_owned(),
        }))
    }

    /// The file's name in the newer release: the pin's version, or else its release's, swapped
    /// for the newer one; a name with neither stays.
    pub fn rename(&self, name: &str) -> String {
        if name.contains(&self.from) {
            name.replace(&self.from, &self.to)
        } else {
            name.replace(&self.from_release, &self.to_release)
        }
    }
}

/// `text` with its `%XX` escapes decoded, as a URL's path spells a tag.
pub fn percent_decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let escaped = (bytes[i] == b'%')
            .then(|| text.get(i + 1..i + 3))
            .flatten()
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        if let Some(byte) = escaped {
            out.push(byte);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// A vendor's JSON feed, read once a run: the bump asks for it once for each architecture of a
/// pin. One that doesn't read as JSON is an error, not an outage: the vendor answered.
fn json(url: &str) -> Result<serde_json::Value, Problem> {
    static FEEDS: Mutex<BTreeMap<String, serde_json::Value>> = Mutex::new(BTreeMap::new());
    if let Some(known) = FEEDS.lock().expect("feeds mutex poisoned").get(url) {
        return Ok(known.clone());
    }
    let body = fetch::get(url).map_err(|e| Problem::fetch(&e))?;
    let feed: serde_json::Value = serde_json::from_slice(&body).map_err(|e| Problem::error(format!("{url}: {e}")))?;
    FEEDS
        .lock()
        .expect("feeds mutex poisoned")
        .insert(url.to_owned(), feed.clone());
    Ok(feed)
}

fn newer(pin: &Pin<'_>, version: &str) -> bool {
    version::compare(version, pin.version) == Ordering::Greater
}

/// The sha256 the vendor gives, else the download's own.
fn sha256_of(url: &str, given: Option<String>) -> Result<String, Problem> {
    match given {
        Some(sha256) => Ok(sha256),
        None => fetch::sha256(url).map_err(|e| Problem::fetch(&e)),
    }
}

fn jetbrains_toolbox(pin: &Pin<'_>) -> Result<Found, Problem> {
    let releases = json(JETBRAINS_TOOLBOX_API)?;
    let release = &releases["TBA"][0];
    let version = release["build"]
        .as_str()
        .ok_or_else(|| Problem::error("JetBrains' release API names no Toolbox build"))?;
    if !newer(pin, version) {
        return Ok(Found::Current);
    }
    let key = if pin.arch == "amd64" { "linux" } else { "linuxARM64" };
    let download = &release["downloads"][key];
    let url = download["link"]
        .as_str()
        .ok_or_else(|| Problem::error(format!("Toolbox {version} has no {key} download")))?;
    let file = url.rsplit('/').next().unwrap_or(url);
    let given = download["checksumLink"]
        .as_str()
        .and_then(|link| fetch::get(link).ok())
        .and_then(|body| digest_for(&String::from_utf8_lossy(&body), file));
    Ok(Found::Newer {
        version: version.to_owned(),
        url: url.to_owned(),
        sha256: sha256_of(url, given)?,
    })
}

fn android_studio(pin: &Pin<'_>) -> Result<Found, Problem> {
    let list = json(ANDROID_STUDIO_RELEASES)?;
    let Some(release) = newest_android_studio(&list) else {
        return Err(Problem::error("Android Studio's release list has no stable release"));
    };
    let version = release["version"].as_str().unwrap_or_default();
    if !newer(pin, version) {
        return Ok(Found::Current);
    }
    let download = release["download"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|download| {
            download["link"]
                .as_str()
                .is_some_and(|link| link.ends_with("-linux.tar.gz"))
        })
        .ok_or_else(|| Problem::error(format!("Android Studio {version} has no Linux download")))?;
    let link = download["link"].as_str().unwrap_or_default();
    let (_, path) = link
        .split_once("/android/studio/ide-zips/")
        .ok_or_else(|| Problem::error(format!("Android Studio's download {link} isn't under ide-zips/")))?;
    let url = format!("{ANDROID_STUDIO}{path}");
    let given = download["checksum"]
        .as_str()
        .filter(|sum| sum.len() == 64 && sum.chars().all(|c| c.is_ascii_hexdigit()))
        .map(str::to_ascii_lowercase);
    Ok(Found::Newer {
        version: version.to_owned(),
        sha256: sha256_of(&url, given)?,
        url,
    })
}

/// The newest stable release in Android Studio's release list.
pub fn newest_android_studio(list: &serde_json::Value) -> Option<&serde_json::Value> {
    list["content"]["item"]
        .as_array()?
        .iter()
        .filter(|item| {
            item["channel"]
                .as_str()
                .is_some_and(|channel| ANDROID_STUDIO_CHANNELS.contains(&channel))
        })
        .filter(|item| item["version"].as_str().is_some_and(version::is_version))
        .max_by(|a, b| {
            version::compare(
                a["version"].as_str().unwrap_or_default(),
                b["version"].as_str().unwrap_or_default(),
            )
        })
}

fn proton_pass(pin: &Pin<'_>) -> Result<Found, Problem> {
    let feed = json(PROTON_PASS_RELEASES)?;
    let Some((version, url)) = newest_proton_pass(&feed) else {
        return Err(Problem::error(
            "Proton Pass's feed has no stable release out to everyone with a .deb",
        ));
    };
    if !newer(pin, &version) {
        return Ok(Found::Current);
    }
    Ok(Found::Newer {
        version,
        sha256: sha256_of(&url, None)?,
        url,
    })
}

/// The newest stable Proton Pass that is out to everyone, and its `.deb`. A release Proton is still
/// rolling out to a share of its users waits.
pub fn newest_proton_pass(feed: &serde_json::Value) -> Option<(String, String)> {
    feed["Releases"]
        .as_array()?
        .iter()
        .filter(|release| release["CategoryName"] == "Stable")
        .filter(|release| release["RolloutPercentage"].as_f64().is_none_or(|share| share >= 1.0))
        .filter_map(|release| {
            let version = release["Version"].as_str()?;
            let url = release["File"]
                .as_array()?
                .iter()
                .filter_map(|file| file["Url"].as_str())
                .find(|url| url.ends_with("_amd64.deb"))?;
            Some((version.to_owned(), url.to_owned()))
        })
        .filter(|(version, _)| version::is_version(version))
        .max_by(|(a, _), (b, _)| version::compare(a, b))
}

fn blender(pin: &Pin<'_>) -> Result<Found, Problem> {
    let index = fetch::get(BLENDER).map_err(|e| Problem::fetch(&e))?;
    let series = links(&String::from_utf8_lossy(&index))
        .filter_map(|link| link.strip_prefix("Blender")?.strip_suffix('/'))
        .filter(|series| version::is_version(series))
        .max_by(|a, b| version::compare(a, b))
        .map(str::to_owned)
        .ok_or_else(|| Problem::error(format!("{BLENDER} lists no Blender series")))?;
    let folder = format!("{BLENDER}Blender{series}/");
    let listing = fetch::get(&folder).map_err(|e| Problem::fetch(&e))?;
    let name = pin.url.rsplit('/').next().unwrap_or(pin.url);
    let (before, after) = name
        .split_once(pin.version)
        .ok_or_else(|| Problem::error(format!("{name} doesn't hold its version, {}", pin.version)))?;
    let version = newest_file(&String::from_utf8_lossy(&listing), before, after)
        .ok_or_else(|| Problem::error(format!("{folder} has no {before}<version>{after}")))?;
    if !newer(pin, &version) {
        return Ok(Found::Current);
    }
    let url = format!("{folder}{before}{version}{after}");
    Ok(Found::Newer {
        sha256: sha256_of(&url, None)?,
        url,
        version,
    })
}

fn helm(pin: &Pin<'_>) -> Result<Found, Problem> {
    let releases = fetch::github(HELM_RELEASES).map_err(|e| Problem::fetch(&e))?;
    let newest =
        newest_tag(&releases, "v").ok_or_else(|| Problem::error("helm/helm has no release tagged v<version>"))?;
    let Some(url) = helm_download(pin, &newest) else {
        return Ok(Found::Current);
    };
    let url = url.map_err(Problem::error)?;
    let name = url.rsplit('/').next().unwrap_or(&url);
    let sums = fetch::get(&format!("{url}.sha256sum")).map_err(|e| Problem::fetch(&e))?;
    let signature = helm_signature(&releases, &newest, name)
        .map(|asc| fetch::get(asc).map_err(|e| Problem::fetch(&e)))
        .transpose()?;
    let sha256 = helm_sha256(&sums, signature.as_deref(), name).map_err(Problem::error)?;
    Ok(Found::Newer {
        version: newest,
        url,
        sha256,
    })
}

/// The download of Helm `newest` for `pin`'s platform, its file named as the pin's with the
/// version swapped; none when `newest` is no newer.
///
/// # Errors
///
/// A message when the pin's file doesn't name its version.
pub fn helm_download(pin: &Pin<'_>, newest: &str) -> Option<Result<String, String>> {
    if !newer(pin, newest) {
        return None;
    }
    let name = pin.url.strip_prefix(HELM).unwrap_or(pin.url);
    if !name.contains(pin.version) {
        return Some(Err(format!("{name} doesn't hold its version, {}", pin.version)));
    }
    Some(Ok(format!("{HELM}{}", name.replace(pin.version, newest))))
}

/// Where Helm's release `version` carries the signature of `name`'s `.sha256sum`; none when the
/// release has no such file.
pub fn helm_signature<'a>(releases: &'a serde_json::Value, version: &str, name: &str) -> Option<&'a str> {
    let tag = format!("v{version}");
    let asc = format!("{name}.sha256sum.asc");
    releases
        .as_array()?
        .iter()
        .find(|release| release["tag_name"].as_str() == Some(&tag))?["assets"]
        .as_array()?
        .iter()
        .find(|asset| asset["name"].as_str() == Some(&asc))?["browser_download_url"]
        .as_str()
}

/// The sha256 of Helm's `name` from its `.sha256sum`, `sums`, once `signature` shows one of
/// Helm's pinned keys signed it: gpg checks it against a keyring of `HELM_KEYS` alone, which a
/// test holds to `HELM_FINGERPRINTS`.
///
/// # Errors
///
/// A message when there is no signature, it doesn't verify with Helm's keys, or the signed file
/// names no sha256 for `name`.
pub fn helm_sha256(sums: &[u8], signature: Option<&[u8]>, name: &str) -> Result<String, String> {
    let Some(signature) = signature else {
        return Err(format!(
            "Helm's release has no {name}.sha256sum.asc, so its sha256 can't be held to Helm's keys"
        ));
    };
    let verified = apt::verify(
        HELM_KEYS,
        Signed::Detached {
            release: sums,
            signature,
        },
    )
    .map_err(|e| format!("{name}.sha256sum: {}", chain(&e)))?;
    // Only a line that names the file counts: the signed text is Helm's word for that file alone.
    verified
        .text
        .lines()
        .filter_map(|line| line.split_once(char::is_whitespace))
        .find(|(_, file)| file.trim().trim_start_matches('*') == name)
        .map(|(sum, _)| sum)
        .filter(|sum| sum.len() == 64 && sum.chars().all(|c| c.is_ascii_hexdigit()))
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| format!("{name}.sha256sum names no sha256 for {name}"))
}

/// The newest version among a folder listing's files named `<before><version><after>`.
pub fn newest_file(listing: &str, before: &str, after: &str) -> Option<String> {
    links(listing)
        .filter_map(|link| link.strip_prefix(before)?.strip_suffix(after))
        .filter(|version| version::is_version(version))
        .max_by(|a, b| version::compare(a, b))
        .map(str::to_owned)
}

/// The `href="…"` targets of an HTML folder listing.
fn links(html: &str) -> impl Iterator<Item = &str> {
    html.split("href=\"")
        .skip(1)
        .filter_map(|rest| rest.split_once('"').map(|(link, _)| link))
}

/// A URL with no version in it serves whatever is newest, so all a check can say is whether that
/// is still the pinned file.
fn unversioned(pin: &Pin<'_>) -> Result<Found, Problem> {
    let served = fetch::sha256(pin.url).map_err(|e| Problem::fetch(&e))?;
    if served == pin.sha256 {
        return Ok(Found::Current);
    }
    Err(Problem::error(format!(
        "it serves another file now, sha256 {served}: its URL names no version, so the new version and \
         sha256 are pinned by hand"
    )))
}

#[cfg(test)]
mod tests;
