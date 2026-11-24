//! What is checked for one app, and what each check found.
//!
//! # Owns
//!
//! - [`Context`]: the checkout's package manifest and sources files, the Ubuntu release the apps
//!   are for, and the archive's indexes, fetched once a run.
//! - [`app`]: every check of one app's actions, in every setup, as [`Row`]s, an outage a warning
//!   ([`Problem`]), or an error on a pin the branch changed.
//!
//! # Not here
//!
//! - Reading repositories: `apt.rs`. Fetching: `fetch.rs`. What a branch changed: `changed.rs`. The
//!   table: `main.rs`.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;

use kubaos_core::apps::Folder;
use kubaos_core::apps::action::Kind;
use kubaos_core::apps::action::version_matches;
use kubaos_core::apps::version;
use kubaos_core::apt::Manifest;

use crate::apt;
use crate::apt::AptError;
use crate::apt::Sources;
use crate::changed::Changed;
use crate::fetch;
use crate::fetch::Class;
use crate::fetch::FetchError;

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
    /// `--bump` moved the pin on to a newer release.
    Moved,
    /// Upstream was down or out of reach on every try, said in the detail; not a failure, since
    /// the pin may well be right.
    Warning,
    /// The pin and upstream disagree.
    Mismatch,
    /// The check couldn't run, or upstream's answer is wrong: a missing file, a signature that
    /// doesn't verify, a feed that doesn't read.
    Error,
}

impl Outcome {
    pub fn word(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Note => "note",
            Self::Moved => "moved",
            Self::Warning => "WARN",
            Self::Mismatch => "MISMATCH",
            Self::Error => "ERROR",
        }
    }

    pub fn fails(self) -> bool {
        matches!(self, Self::Mismatch | Self::Error)
    }
}

/// Why a check reached no verdict, and how its row reports it: a warning for an outage, else an
/// error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    pub outcome: Outcome,
    pub detail: String,
}

impl Problem {
    /// A problem that fails the check.
    pub fn error(detail: impl Into<String>) -> Self {
        Self {
            outcome: Outcome::Error,
            detail: detail.into(),
        }
    }

    pub fn fetch(error: &FetchError) -> Self {
        Self::classed(error, error.class())
    }

    pub fn apt(error: &AptError) -> Self {
        Self::classed(error, error.class())
    }

    fn classed(error: &dyn std::error::Error, class: Class) -> Self {
        Self {
            outcome: match class {
                Class::Outage => Outcome::Warning,
                Class::Failure => Outcome::Error,
            },
            detail: chain(error),
        }
    }

    /// The problem with `context` before its detail.
    #[must_use]
    pub fn context(self, context: impl std::fmt::Display) -> Self {
        Self {
            detail: format!("{context}: {}", self.detail),
            ..self
        }
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
    ubuntu: Once<Result<Versions, Problem>>,
    /// What the branch changed, under `--changed`.
    changed: Option<Changed>,
}

/// The newest version of each package, by name.
type Versions = BTreeMap<String, String>;

/// A value per key, such as an architecture, worked out once a run. Each key has a lock of its
/// own, so two keys are fetched side by side and only a second asker of the same key waits.
#[derive(Debug)]
struct Once<T> {
    cells: Mutex<BTreeMap<String, Arc<OnceLock<T>>>>,
}

impl<T> Default for Once<T> {
    fn default() -> Self {
        Self {
            cells: Mutex::new(BTreeMap::new()),
        }
    }
}

impl<T: Clone> Once<T> {
    fn get_or_init(&self, key: &str, init: impl FnOnce() -> T) -> T {
        let cell = {
            let mut cells = self.cells.lock().expect("cache mutex poisoned");
            Arc::clone(cells.entry(key.to_owned()).or_default())
        };
        cell.get_or_init(init).clone()
    }
}

impl Context {
    pub fn new(root: PathBuf, manifest: Manifest, release: String) -> Self {
        Self {
            root,
            manifest,
            release,
            ubuntu: Once::default(),
            changed: None,
        }
    }

    /// The context with what the branch changed, so an outage on a changed pin fails.
    #[must_use]
    pub fn with_changed(self, changed: Changed) -> Self {
        Self {
            changed: Some(changed),
            ..self
        }
    }

    /// Ubuntu's newest version of each package for `arch`, from the release, `-updates` and
    /// `-security`, each `InRelease` verified with the archive's keys.
    fn ubuntu(&self, arch: &str) -> Result<Versions, Problem> {
        self.ubuntu
            .get_or_init(arch, || self.fetch_ubuntu(arch).map_err(|e| Problem::apt(&e)))
    }

    fn fetch_ubuntu(&self, arch: &str) -> Result<Versions, AptError> {
        let keys = fs::read(UBUNTU_KEYRING).map_err(|e| AptError::Invalid(format!("reading {UBUNTU_KEYRING}: {e}")))?;
        // i386 is served beside amd64, on the archive's main mirror.
        let mirror = if matches!(arch, "amd64" | "i386") {
            "http://archive.ubuntu.com/ubuntu"
        } else {
            "http://ports.ubuntu.com/ubuntu-ports"
        };
        let mut newest: BTreeMap<String, String> = BTreeMap::new();
        for pocket in ["", "-updates", "-security"] {
            let dists = format!("{mirror}/dists/{}{pocket}", self.release);
            let (_, release) = apt::release(&dists, &keys)?;
            for (package, found) in apt::index(&dists, &release, &UBUNTU_COMPONENTS, arch)? {
                keep_newest(&mut newest, package, found);
            }
        }
        Ok(newest)
    }
}

/// Every check of `app`'s actions, its own and every setup's.
pub fn app(context: &Context, app: &Folder) -> Vec<Row> {
    let mut rows = Vec::new();
    // Whether the branch changed the pin the next rows are about, under `--changed`.
    let fresh = Cell::new(false);
    let mut row = |check, subject: String, outcome, detail: String| {
        let (outcome, detail) = escalated(outcome, detail, fresh.get());
        rows.push(Row {
            app: app.id.clone(),
            check,
            subject,
            outcome,
            detail,
        });
    };
    let changed = context.changed.as_ref();
    let fresh_repo = |name: &str| changed.is_some_and(|changed| changed.repo(name));
    let actions = app
        .actions
        .iter()
        .chain(app.setups.iter().flat_map(|setup| &setup.actions));
    let mut repos: BTreeMap<String, Option<Repository>> = BTreeMap::new();
    for action in actions {
        let fresh_action = changed.is_some_and(|changed| changed.action(&app.id, &action.kind));
        fresh.set(fresh_action);
        match &action.kind {
            Kind::Apt {
                packages,
                repo,
                arch,
                versions,
                amd64,
                arm64,
                ..
            } => {
                if let Some(name) = repo
                    && !repos.contains_key(name)
                {
                    fresh.set(fresh_repo(name));
                    let checked = vendor(context, name, &mut row);
                    repos.insert(name.clone(), checked);
                }
                fresh.set(fresh_action || repo.as_deref().is_some_and(fresh_repo));
                // A repository whose key or InRelease failed was reported; its packages aren't
                // looked up.
                let repository = match repo {
                    Some(name) => match repos.get(name).and_then(Option::as_ref) {
                        Some(repository) => Some(repository),
                        None => continue,
                    },
                    None => None,
                };
                for (machine, _) in ARCHES {
                    if arch.as_deref().is_some_and(|only| only != machine) {
                        continue;
                    }
                    let own = match machine {
                        "amd64" => amd64.as_ref(),
                        _ => arm64.as_ref(),
                    };
                    let wanted = packages.iter().chain(own.into_iter().flat_map(|own| &own.packages));
                    for package in wanted {
                        candidate(context, repository, package, machine, versions.as_deref(), &mut row);
                    }
                }
            }
            Kind::Deb { amd64, arm64, .. } | Kind::Download { amd64, arm64, .. } => {
                for (arch, download) in [("amd64", amd64), ("arm64", arm64)] {
                    if let Some(download) = download {
                        pinned(&format!("({arch})"), &download.url, &download.sha256, &mut row);
                    }
                }
            }
            Kind::Dkms {
                module, url, sha256, ..
            } => pinned(&format!("({module})"), url, sha256, &mut row),
            Kind::Nix { packages, arch, unfree } => nix(&context.root, packages, arch.as_deref(), *unfree, &mut row),
            Kind::WebApp { url, .. } => web_app(url, &mut row),
            _ => {}
        }
    }
    rows
}

type Report<'a> = dyn FnMut(&'static str, String, Outcome, String) + 'a;

/// An outage's row as it is, or an error when the branch changed the pin it is about: that pin has
/// never been checked, and a run that passes would let it in unchecked.
fn escalated(outcome: Outcome, detail: String, fresh: bool) -> (Outcome, String) {
    if outcome == Outcome::Warning && fresh {
        (
            Outcome::Error,
            format!("{detail}; this branch changes the pin, so it can't pass unchecked"),
        )
    } else {
        (outcome, detail)
    }
}

/// A vendor repository whose key and `InRelease` checked out: what its packages are looked up in.
#[derive(Debug)]
struct Repository {
    name: String,
    sources: Sources,
    release: apt::Release,
    /// Its newest version of each package, by architecture, fetched on first use.
    indexes: Once<Result<Versions, Problem>>,
}

impl Repository {
    /// The repository's newest version of each package for `arch`, over its indexes, or `None`
    /// when its sources file doesn't serve `arch`.
    fn index(&self, arch: &str) -> Option<Result<Versions, Problem>> {
        if !self.sources.serves(arch) {
            return None;
        }
        Some(self.indexes.get_or_init(arch, || {
            let components: Vec<&str> = self.sources.components.iter().map(String::as_str).collect();
            apt::index(&self.sources.dists(), &self.release, &components, arch).map_err(|e| Problem::apt(&e))
        }))
    }

    fn label(&self) -> String {
        format!("{} {}", self.name, self.sources.suite)
    }
}

/// A vendor repository's key against its pin, and its next key's when the vendor is moving to
/// one, and its `InRelease` against them: signed by either, as its sources file trusts both.
fn vendor(context: &Context, name: &str, row: &mut Report<'_>) -> Option<Repository> {
    let Some(pin) = context.manifest.repos().get(name) else {
        row(
            "key",
            name.to_owned(),
            Outcome::Error,
            "the package manifest has no such repo".to_owned(),
        );
        return None;
    };
    let mut keys = Vec::new();
    let next = pin.next_key_url.as_deref().zip(pin.next_fingerprint.as_deref());
    let pinned_keys = std::iter::once((pin.key_url.as_str(), pin.fingerprint.as_str())).chain(next);
    let mut good = true;
    for (url, fingerprint) in pinned_keys {
        let subject = if fingerprint == pin.fingerprint {
            name.to_owned()
        } else {
            format!("{name} (next)")
        };
        let key = match apt::key(url) {
            Ok(key) => key,
            Err(e) => {
                let problem = Problem::apt(&e);
                row("key", subject, problem.outcome, problem.detail);
                good = false;
                continue;
            }
        };
        match apt::fingerprints(&key) {
            Ok(found) if found == [fingerprint] => row("key", subject, Outcome::Ok, fingerprint.to_owned()),
            Ok(found) => {
                row(
                    "key",
                    subject,
                    Outcome::Mismatch,
                    format!("pinned {fingerprint}, the key holds {}", found.join(", ")),
                );
                good = false;
            }
            Err(e) => {
                row("key", subject, Outcome::Error, chain(&e));
                good = false;
            }
        }
        keys.extend(key);
    }
    if !good {
        return None;
    }
    let path = context
        .root
        .join(format!("system/files/apt/sources.list.d/{name}.sources"));
    let sources = fs::read_to_string(&path)
        .map_err(|e| AptError::Invalid(format!("reading {}: {e}", path.display())))
        .and_then(|text| Sources::parse(&text));
    let sources = match sources {
        Ok(sources) => sources,
        Err(e) => {
            row("InRelease", name.to_owned(), Outcome::Error, chain(&e));
            return None;
        }
    };
    let subject = format!("{name} {}", sources.suite);
    let trusted = [Some(pin.fingerprint.as_str()), pin.next_fingerprint.as_deref()];
    match apt::release(&sources.dists(), &keys) {
        Ok((verified, release)) if trusted.contains(&Some(verified.primary.as_str())) => {
            row(
                "InRelease",
                subject,
                Outcome::Ok,
                format!("signed by {}", verified.primary),
            );
            Some(Repository {
                name: name.to_owned(),
                sources,
                release,
                indexes: Once::default(),
            })
        }
        Ok((verified, _)) => {
            row(
                "InRelease",
                subject,
                Outcome::Mismatch,
                format!("signed by {}, not a pin", verified.primary),
            );
            None
        }
        Err(e) => {
            let problem = Problem::apt(&e);
            row("InRelease", subject, problem.outcome, problem.detail);
            None
        }
    }
}

/// `package`'s candidate on a `machine`: in the vendor `repository` where it serves it, else in
/// Ubuntu's archive, as apt with both finds it. A `package` named `<name>:<architecture>` is that
/// architecture's, as dpkg takes it. With `versions`, the candidate must be one of the app's.
fn candidate(
    context: &Context,
    repository: Option<&Repository>,
    package: &str,
    machine: &str,
    versions: Option<&str>,
    row: &mut Report<'_>,
) {
    let (name, arch) = package.split_once(':').unwrap_or((package, machine));
    let subject = format!("{package} ({machine})");
    let mut found = None;
    if let Some(repository) = repository {
        match repository.index(arch) {
            Some(Ok(newest)) => found = newest.get(name).map(|version| (repository.label(), version.clone())),
            Some(Err(e)) => {
                let problem = e.context(repository.label());
                return row("candidate", subject, problem.outcome, problem.detail);
            }
            None => {}
        }
    }
    let found = match found {
        Some(found) => Some(found),
        None => match context.ubuntu(arch) {
            Ok(newest) => newest
                .get(name)
                .map(|version| (format!("ubuntu {}", context.release), version.clone())),
            Err(e) => {
                let problem = e.context(format!("ubuntu ({arch})"));
                return row("candidate", subject, problem.outcome, problem.detail);
            }
        },
    };
    let searched = repository.map_or_else(
        || format!("ubuntu {}", context.release),
        |repository| format!("{} or ubuntu {}", repository.label(), context.release),
    );
    match (found, versions) {
        (None, _) => row("candidate", subject, Outcome::Mismatch, format!("not in {searched}")),
        (Some((source, version)), Some(pattern)) if !version_matches(pattern, &version) => row(
            "candidate",
            subject,
            Outcome::Mismatch,
            format!("{version} in {source}, not one of the app's versions ({pattern})"),
        ),
        (Some((source, version)), _) => row("candidate", subject, Outcome::Ok, format!("{version} in {source}")),
    }
}

/// A pinned download's sha256 against the pin, and against the digest its vendor publishes;
/// `what` follows the file's name in the table.
fn pinned(what: &str, url: &str, sha256: &str, row: &mut Report<'_>) {
    let file = url.rsplit('/').next().unwrap_or(url).to_owned();
    let subject = format!("{file} {what}");
    match fetch::sha256(url) {
        Ok(got) if got == sha256 => row("sha256", subject.clone(), Outcome::Ok, got),
        Ok(got) => row(
            "sha256",
            subject.clone(),
            Outcome::Mismatch,
            format!("pinned {sha256}, served {got}"),
        ),
        Err(e) => {
            let problem = Problem::fetch(&e);
            row("sha256", subject.clone(), problem.outcome, problem.detail);
            // The host is down, so the vendor's digests beside the file would only spend their
            // tries on it too.
            if e.class() == Class::Outage {
                return;
            }
        }
    }
    match published(url) {
        Ok(Some((source, digest))) if digest == sha256 => row("published", subject, Outcome::Ok, source),
        Ok(Some((source, digest))) => row(
            "published",
            subject,
            Outcome::Mismatch,
            format!("pinned {sha256}, {source} says {digest}"),
        ),
        Ok(None) => row(
            "published",
            subject,
            Outcome::Note,
            "the vendor publishes no digest beside it".to_owned(),
        ),
        Err(e) => row("published", subject, e.outcome, e.detail),
    }
}

/// The sha256 a vendor publishes for `url`, and where it was found: a GitHub release asset's
/// digest, or a checksum file beside the download. A checksum file that was down, with none found
/// beside it, is a warning.
fn published(url: &str) -> Result<Option<(String, String)>, Problem> {
    if let Some(rest) = url.strip_prefix("https://github.com/") {
        let parts: Vec<&str> = rest.splitn(6, '/').collect();
        if let [owner, repo, "releases", "download", tag, name] = parts[..] {
            let release =
                fetch::github(&format!("repos/{owner}/{repo}/releases/tags/{tag}")).map_err(|e| Problem::fetch(&e))?;
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
    let mut down = None;
    for candidate in [
        format!("{url}.sha256"),
        format!("{url}.sha256sum"),
        format!("{dir}/SHA256SUMS"),
    ] {
        match fetch::get(&candidate) {
            Ok(body) => {
                if let Some(digest) = digest_for(&String::from_utf8_lossy(&body), file) {
                    return Ok(Some((candidate, digest)));
                }
            }
            Err(e) if e.class() == Class::Outage => down = down.or(Some(Problem::fetch(&e))),
            Err(_) => {}
        }
    }
    // A checksum file that was down may be the digest the vendor publishes; "none" would hide it.
    down.map_or(Ok(None), Err)
}

/// The sha256 a checksum file gives `file`: on the line naming it, or its only line.
pub fn digest_for(text: &str, file: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().filter(|line| !line.trim().is_empty()).collect();
    let line = lines
        .iter()
        .find(|line| line.split_whitespace().any(|word| word.trim_start_matches('*') == file))
        .or(if lines.len() == 1 { lines.first() } else { None })?;
    line.split_whitespace()
        .find(|word| word.len() == 64 && word.chars().all(|c| c.is_ascii_hexdigit()))
        .map(str::to_ascii_lowercase)
}

/// Each nixpkgs package's version at the checkout's `flake.lock`, for each system the action is
/// for, and whether it is available there (its platforms, not broken; unfree allowed, as the user
/// flake decides it). An action is `unfree` exactly when nixpkgs marks one of its packages unfree.
fn nix(root: &Path, packages: &[String], arch: Option<&str>, unfree: bool, row: &mut Report<'_>) {
    for (machine, system) in ARCHES {
        if arch.is_some_and(|only| only != machine) {
            continue;
        }
        let mut found_unfree = Vec::new();
        let mut evaluated = true;
        for package in packages {
            let subject = format!("{package} ({system})");
            match evaluate(root, system, package) {
                Ok(value) => {
                    let version = value["version"].as_str().unwrap_or("no version").to_owned();
                    if value["unfree"] == true {
                        found_unfree.push(package.as_str());
                    }
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
                Err(e) => {
                    evaluated = false;
                    row("nixpkgs", subject, Outcome::Error, e);
                }
            }
        }
        if !evaluated {
            continue;
        }
        let subject = format!("{} ({system})", packages.join(" "));
        match (unfree, found_unfree.is_empty()) {
            (true, true) => row(
                "unfree",
                subject,
                Outcome::Mismatch,
                "marked unfree, but nixpkgs marks none of them unfree".to_owned(),
            ),
            (false, false) => row(
                "unfree",
                subject,
                Outcome::Mismatch,
                format!("nixpkgs marks {} unfree; the action says not", found_unfree.join(", ")),
            ),
            _ => {}
        }
    }
}

/// `package`'s version, availability and licence in nixpkgs at the checkout's `flake.lock`, for
/// `system`, as `nix eval` gives them; the last line nix said when it fails.
fn evaluate(root: &Path, system: &str, package: &str) -> Result<serde_json::Value, String> {
    // Nix reads the checkout's flake from git.
    let output = kubaos_git::clear(&mut Command::new("nix"))
        .args(["eval", "--json", "--impure", "--inputs-from"])
        .arg(root)
        .arg(format!("nixpkgs#legacyPackages.{system}.{package}"))
        .args([
            "--apply",
            "p: { version = p.version or null; available = p.meta.available or false; unfree = p.meta.unfree or false; }",
        ])
        .env("NIXPKGS_ALLOW_UNFREE", "1")
        .output()
        .map_err(|e| format!("running nix: {e}"))?;
    if !output.status.success() {
        let said = String::from_utf8_lossy(&output.stderr);
        let last = said
            .lines()
            .rfind(|line| !line.trim().is_empty())
            .unwrap_or("nix eval failed");
        return Err(last.trim().to_owned());
    }
    Ok(serde_json::from_slice(&output.stdout).unwrap_or_default())
}

/// A web app's URL answers with a success after its redirects, or Cloudflare's bot challenge; one
/// that is down on every try is a warning. One of this machine's, plain HTTP on a loopback host,
/// isn't reachable from here; any other URL that isn't HTTPS is a mismatch.
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
        Ok(answer) if (200..300).contains(&answer.status) => {
            row(
                "web_app",
                url.to_owned(),
                Outcome::Ok,
                format!("HTTP {}", answer.status),
            );
        }
        Ok(answer) if answer.challenged => row(
            "web_app",
            url.to_owned(),
            Outcome::Note,
            format!(
                "HTTP {}, Cloudflare's bot challenge, which a browser passes",
                answer.status
            ),
        ),
        Ok(answer) => row(
            "web_app",
            url.to_owned(),
            Outcome::Mismatch,
            format!("HTTP {}", answer.status),
        ),
        Err(e) => {
            let problem = Problem::fetch(&e);
            row("web_app", url.to_owned(), problem.outcome, problem.detail);
        }
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
