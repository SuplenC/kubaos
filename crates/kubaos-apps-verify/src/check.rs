//! What is checked for one app, and what each check found.
//!
//! # Owns
//!
//! - [`Context`]: the checkout's package manifest and sources files, the Ubuntu release the apps
//!   are for, and the archive's indexes, fetched once a run.
//! - [`app`]: every check of one app's actions, in every setup, as [`Row`]s.
//!
//! # Not here
//!
//! - Reading repositories: `apt.rs`. Fetching: `fetch.rs`. The table: `main.rs`.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

use kubaos_core::apps::Folder;
use kubaos_core::apps::action::Download;
use kubaos_core::apps::action::Kind;
use kubaos_core::apps::version;
use kubaos_core::apt::Manifest;

use crate::apt;
use crate::apt::AptError;
use crate::apt::Sources;
use crate::fetch;

/// The architectures KubaOS is built for, as dpkg names them, with Nix's system for each.
const ARCHES: [(&str, &str); 2] = [("amd64", "x86_64-linux"), ("arm64", "aarch64-linux")];

/// The components of Ubuntu's archive.
const UBUNTU_COMPONENTS: [&str; 4] = ["main", "restricted", "universe", "multiverse"];

/// The Ubuntu archive's keys, as `ubuntu-keyring` installs them on every Ubuntu machine and
/// runner. Only read.
const UBUNTU_KEYRING: &str = "/usr/share/keyrings/ubuntu-archive-keyring.gpg";

/// What a check found.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Outcome {
    Ok,
    /// Nothing to hold the pin to, said in the detail; not a failure.
    Note,
    /// The pin and upstream disagree.
    Mismatch,
    /// The check couldn't run: upstream unreachable, a signature that doesn't verify.
    Error,
}

impl Outcome {
    pub fn word(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Note => "note",
            Self::Mismatch => "MISMATCH",
            Self::Error => "ERROR",
        }
    }

    pub fn fails(self) -> bool {
        matches!(self, Self::Mismatch | Self::Error)
    }
}

/// One line of the table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub app: String,
    pub check: &'static str,
    pub subject: String,
    pub outcome: Outcome,
    pub detail: String,
}

/// What every app's checks share.
#[derive(Debug)]
pub struct Context {
    root: PathBuf,
    manifest: Manifest,
    /// The Ubuntu release the apps are for, by its codename.
    release: String,
    /// Ubuntu's newest version of each package, by architecture, fetched on first use.
    ubuntu: Mutex<BTreeMap<String, Result<Versions, String>>>,
}

/// The newest version of each package, by name.
type Versions = BTreeMap<String, String>;

impl Context {
    pub fn new(root: PathBuf, manifest: Manifest, release: String) -> Self {
        Self {
            root,
            manifest,
            release,
            ubuntu: Mutex::new(BTreeMap::new()),
        }
    }

    /// Ubuntu's newest version of each package for `arch`, from the release, `-updates` and
    /// `-security`, each `InRelease` verified with the archive's keys.
    fn ubuntu(&self, arch: &str) -> Result<Versions, String> {
        let mut cache = self.ubuntu.lock().expect("Ubuntu index cache mutex poisoned");
        if let Some(found) = cache.get(arch) {
            return found.clone();
        }
        let found = self.fetch_ubuntu(arch).map_err(|e| chain(&e));
        cache.insert(arch.to_owned(), found.clone());
        found
    }

    fn fetch_ubuntu(&self, arch: &str) -> Result<Versions, AptError> {
        let keys = fs::read(UBUNTU_KEYRING).map_err(|e| AptError::Invalid(format!("reading {UBUNTU_KEYRING}: {e}")))?;
        let mirror = if arch == "amd64" {
            "http://archive.ubuntu.com/ubuntu"
        } else {
            "http://ports.ubuntu.com/ubuntu-ports"
        };
        let mut newest: BTreeMap<String, String> = BTreeMap::new();
        for pocket in ["", "-updates", "-security"] {
            let dists = format!("{mirror}/dists/{}{pocket}", self.release);
            let (_, release) = apt::release(&dists, &keys)?;
            for component in UBUNTU_COMPONENTS {
                for (package, found) in apt::index(&dists, &release, component, arch)? {
                    keep_newest(&mut newest, package, found);
                }
            }
        }
        Ok(newest)
    }
}

/// Every check of `app`'s actions, its own and every setup's.
pub fn app(context: &Context, app: &Folder) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut row = |check, subject: String, outcome, detail: String| {
        rows.push(Row {
            app: app.id.clone(),
            check,
            subject,
            outcome,
            detail,
        });
    };
    let actions = app
        .actions
        .iter()
        .chain(app.setups.iter().flat_map(|setup| &setup.actions));
    let mut seen_repos = BTreeSet::new();
    for action in actions {
        match &action.kind {
            Kind::Apt { packages, repo, arch } => {
                let arches: Vec<&str> = ARCHES
                    .iter()
                    .map(|(arch, _)| *arch)
                    .filter(|each| arch.as_deref().is_none_or(|only| only == *each))
                    .collect();
                match repo {
                    Some(repo) if seen_repos.insert(repo.clone()) => vendor(context, repo, packages, &arches, &mut row),
                    Some(_) => {}
                    None => ubuntu(context, packages, &arches, &mut row),
                }
            }
            Kind::Deb { amd64, arm64, .. } | Kind::Download { amd64, arm64, .. } => {
                for (arch, download) in [("amd64", amd64), ("arm64", arm64)] {
                    if let Some(download) = download {
                        pinned(arch, download, &mut row);
                    }
                }
            }
            Kind::Nix { packages } => nix(&context.root, packages, &mut row),
            Kind::WebApp { url, .. } => web_app(url, &mut row),
            _ => {}
        }
    }
    rows
}

type Report<'a> = dyn FnMut(&'static str, String, Outcome, String) + 'a;

/// A vendor repository's key against its pin, its `InRelease` against the key, and each package's
/// candidate for each architecture.
fn vendor(context: &Context, name: &str, packages: &[String], arches: &[&str], row: &mut Report<'_>) {
    let Some(pin) = context.manifest.repos().get(name) else {
        row(
            "key",
            name.to_owned(),
            Outcome::Error,
            "the package manifest has no such repo".to_owned(),
        );
        return;
    };
    let key = match apt::key(&pin.key_url) {
        Ok(key) => key,
        Err(e) => return row("key", name.to_owned(), Outcome::Error, chain(&e)),
    };
    match apt::fingerprints(&key) {
        Ok(found) if found == [pin.fingerprint.clone()] => {
            row("key", name.to_owned(), Outcome::Ok, pin.fingerprint.clone());
        }
        Ok(found) => row(
            "key",
            name.to_owned(),
            Outcome::Mismatch,
            format!("pinned {}, the key holds {}", pin.fingerprint, found.join(", ")),
        ),
        Err(e) => return row("key", name.to_owned(), Outcome::Error, chain(&e)),
    }
    let path = context
        .root
        .join(format!("system/files/apt/sources.list.d/{name}.sources"));
    let sources =
        match fs::read_to_string(&path).map_err(|e| AptError::Invalid(format!("reading {}: {e}", path.display()))) {
            Ok(text) => Sources::parse(&text),
            Err(e) => Err(e),
        };
    let sources = match sources {
        Ok(sources) => sources,
        Err(e) => return row("InRelease", name.to_owned(), Outcome::Error, chain(&e)),
    };
    let dists = sources.dists();
    let release = match apt::release(&dists, &key) {
        Ok((verified, release)) if verified.primary == pin.fingerprint => {
            row(
                "InRelease",
                format!("{name} {}", sources.suite),
                Outcome::Ok,
                format!("signed by {}", verified.primary),
            );
            release
        }
        Ok((verified, _)) => {
            return row(
                "InRelease",
                format!("{name} {}", sources.suite),
                Outcome::Mismatch,
                format!("signed by {}, not the pin", verified.primary),
            );
        }
        Err(e) => {
            return row(
                "InRelease",
                format!("{name} {}", sources.suite),
                Outcome::Error,
                chain(&e),
            );
        }
    };
    for arch in arches {
        if !sources.serves(arch) {
            row(
                "candidate",
                format!("{name} ({arch})"),
                Outcome::Mismatch,
                "the sources file doesn't serve it".to_owned(),
            );
            continue;
        }
        let mut newest = BTreeMap::new();
        for component in &sources.components {
            match apt::index(&dists, &release, component, arch) {
                Ok(found) => found
                    .into_iter()
                    .for_each(|(package, found)| keep_newest(&mut newest, package, found)),
                Err(e) => {
                    return row(
                        "candidate",
                        format!("{name} {component} ({arch})"),
                        Outcome::Error,
                        chain(&e),
                    );
                }
            }
        }
        candidates(&format!("{name} {}", sources.suite), packages, arch, &newest, row);
    }
}

/// Each of Ubuntu's packages' candidate in the archive, for each architecture.
fn ubuntu(context: &Context, packages: &[String], arches: &[&str], row: &mut Report<'_>) {
    for arch in arches {
        match context.ubuntu(arch) {
            Ok(newest) => candidates(&format!("ubuntu {}", context.release), packages, arch, &newest, row),
            Err(e) => row("candidate", format!("ubuntu ({arch})"), Outcome::Error, e),
        }
    }
}

fn candidates(source: &str, packages: &[String], arch: &str, newest: &BTreeMap<String, String>, row: &mut Report<'_>) {
    for package in packages {
        let subject = format!("{package} ({arch})");
        match newest.get(package) {
            Some(found) => row("candidate", subject, Outcome::Ok, format!("{found} in {source}")),
            None => row("candidate", subject, Outcome::Mismatch, format!("not in {source}")),
        }
    }
}

/// A pinned download's sha256 against the pin, and against the digest its vendor publishes.
fn pinned(arch: &str, download: &Download, row: &mut Report<'_>) {
    let file = download.url.rsplit('/').next().unwrap_or(&download.url).to_owned();
    let subject = format!("{file} ({arch})");
    match fetch::sha256(&download.url) {
        Ok(got) if got == download.sha256 => row("sha256", subject.clone(), Outcome::Ok, got),
        Ok(got) => row(
            "sha256",
            subject.clone(),
            Outcome::Mismatch,
            format!("pinned {}, served {got}", download.sha256),
        ),
        Err(e) => row("sha256", subject.clone(), Outcome::Error, chain(&e)),
    }
    match published(&download.url) {
        Ok(Some((source, digest))) if digest == download.sha256 => row("published", subject, Outcome::Ok, source),
        Ok(Some((source, digest))) => row(
            "published",
            subject,
            Outcome::Mismatch,
            format!("pinned {}, {source} says {digest}", download.sha256),
        ),
        Ok(None) => row(
            "published",
            subject,
            Outcome::Note,
            "the vendor publishes no digest beside it".to_owned(),
        ),
        Err(e) => row("published", subject, Outcome::Error, e),
    }
}

/// The sha256 a vendor publishes for `url`, and where it was found: a GitHub release asset's
/// digest, or a checksum file beside the download.
fn published(url: &str) -> Result<Option<(String, String)>, String> {
    if let Some(rest) = url.strip_prefix("https://github.com/") {
        let parts: Vec<&str> = rest.splitn(6, '/').collect();
        if let [owner, repo, "releases", "download", tag, name] = parts[..] {
            let release = fetch::github(&format!("repos/{owner}/{repo}/releases/tags/{tag}")).map_err(|e| chain(&e))?;
            let digest = release["assets"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|asset| asset["name"] == name)
                .and_then(|asset| asset["digest"].as_str())
                .and_then(|digest| digest.strip_prefix("sha256:"));
            return Ok(digest.map(|digest| ("GitHub's asset digest".to_owned(), digest.to_ascii_lowercase())));
        }
    }
    let (dir, file) = url.rsplit_once('/').unwrap_or(("", url));
    for candidate in [
        format!("{url}.sha256"),
        format!("{url}.sha256sum"),
        format!("{dir}/SHA256SUMS"),
    ] {
        if let Ok(body) = fetch::get(&candidate)
            && let Some(digest) = digest_for(&String::from_utf8_lossy(&body), file)
        {
            return Ok(Some((candidate, digest)));
        }
    }
    Ok(None)
}

/// The sha256 a checksum file gives `file`: on the line naming it, or its only line.
fn digest_for(text: &str, file: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().filter(|line| !line.trim().is_empty()).collect();
    let line = lines
        .iter()
        .find(|line| line.split_whitespace().any(|word| word.trim_start_matches('*') == file))
        .or(if lines.len() == 1 { lines.first() } else { None })?;
    line.split_whitespace()
        .find(|word| word.len() == 64 && word.chars().all(|c| c.is_ascii_hexdigit()))
        .map(str::to_ascii_lowercase)
}

/// Each nixpkgs package's version at the checkout's `flake.lock`, for each system, and whether it
/// is available there (its platforms, not broken; unfree allowed, as the user flake decides it).
fn nix(root: &Path, packages: &[String], row: &mut Report<'_>) {
    for package in packages {
        for (_, system) in ARCHES {
            let subject = format!("{package} ({system})");
            let output = Command::new("nix")
                .args(["eval", "--json", "--impure", "--inputs-from"])
                .arg(root)
                .arg(format!("nixpkgs#legacyPackages.{system}.{package}"))
                .args([
                    "--apply",
                    "p: { version = p.version or null; available = p.meta.available or false; }",
                ])
                .env("NIXPKGS_ALLOW_UNFREE", "1")
                .output();
            let output = match output {
                Ok(output) if output.status.success() => output,
                Ok(output) => {
                    let said = String::from_utf8_lossy(&output.stderr);
                    let last = said
                        .lines()
                        .rfind(|line| !line.trim().is_empty())
                        .unwrap_or("nix eval failed");
                    row("nixpkgs", subject, Outcome::Error, last.trim().to_owned());
                    continue;
                }
                Err(e) => {
                    row("nixpkgs", subject, Outcome::Error, format!("running nix: {e}"));
                    continue;
                }
            };
            let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_default();
            let version = value["version"].as_str().unwrap_or("no version").to_owned();
            if value["available"] == true {
                row("nixpkgs", subject, Outcome::Ok, version);
            } else {
                row(
                    "nixpkgs",
                    subject,
                    Outcome::Mismatch,
                    format!("{version}, not available on {system}"),
                );
            }
        }
    }
}

/// A web app's URL answers with a success after its redirects. One of this machine's, plain HTTP
/// on a loopback host, isn't reachable from here; any other URL that isn't HTTPS is a mismatch.
fn web_app(url: &str, row: &mut Report<'_>) {
    if let Some(rest) = url.strip_prefix("http://") {
        let (outcome, detail) = if loopback(rest) {
            (Outcome::Note, "this machine's own, not checked")
        } else {
            (Outcome::Mismatch, "plain HTTP off this machine")
        };
        return row("web_app", url.to_owned(), outcome, detail.to_owned());
    }
    if !url.starts_with("https://") {
        return row(
            "web_app",
            url.to_owned(),
            Outcome::Mismatch,
            "not an HTTPS URL".to_owned(),
        );
    }
    match fetch::status(url) {
        Ok(status) if (200..300).contains(&status) => {
            row("web_app", url.to_owned(), Outcome::Ok, format!("HTTP {status}"));
        }
        Ok(status) => row("web_app", url.to_owned(), Outcome::Mismatch, format!("HTTP {status}")),
        Err(e) => row("web_app", url.to_owned(), Outcome::Error, chain(&e)),
    }
}

fn keep_newest(newest: &mut BTreeMap<String, String>, package: String, found: String) {
    if newest
        .get(&package)
        .is_none_or(|known| version::compare(&found, known).is_gt())
    {
        newest.insert(package, found);
    }
}

/// An error and its causes, on one line.
pub fn chain(error: &dyn std::error::Error) -> String {
    let mut text = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        text.push_str(": ");
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}

/// Whether the URL after its scheme names a loopback host: `localhost`, 127.0.0.0/8 or `[::1]`.
fn loopback(rest: &str) -> bool {
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
    let host = if host.starts_with('[') {
        host.split_once(']').map_or(host, |(ip, _)| &host[..=ip.len()])
    } else {
        host.split(':').next().unwrap_or_default()
    };
    host.eq_ignore_ascii_case("localhost")
        || host == "[::1]"
        || host.parse::<std::net::Ipv4Addr>().is_ok_and(|ip| ip.is_loopback())
}

#[cfg(test)]
mod tests;
