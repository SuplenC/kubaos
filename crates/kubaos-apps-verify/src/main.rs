//! `kubaos-apps-verify`: holds the Apps catalog's pins to their upstreams, for a reviewer and CI.
//!
//! ```text
//! kubaos-apps-verify [--root DIR] [--release CODENAME] [ID…]   those apps, or every app
//! kubaos-apps-verify [--root DIR] --changed BASE               the apps a branch changed
//! kubaos-apps-verify [--root DIR] --bump [ID…]                 move those apps' pins on
//! ```
//!
//! For each app, in its own actions and every setup's: a vendor repository's key against its
//! pinned fingerprint, and its next key's where the vendor is moving to one, and its release file
//! (`InRelease`, or `Release` and `Release.gpg`) verified with those keys alone; each apt package's
//! candidate for each architecture the action is for, its per-architecture packages and a
//! `<name>:<architecture>` package's other architecture included, in the vendor's repository and
//! else Ubuntu's archive for the release (their `Packages` indexes, never the machine's apt), and
//! one of the app's `versions` where the action names them; each pinned download's and DKMS
//! source's sha256 as served and as the vendor publishes it; each nixpkgs package at the checkout's
//! `flake.lock`, for each system the action is for, and its `unfree` mark against nixpkgs'; and a
//! web app's URL. It prints a table and exits 1 on a mismatch or a check that couldn't run, 2 on a
//! usage error. An upstream that is down on every try (an HTTP 5xx or 429, a redirect loop, a
//! timeout, a name that doesn't resolve, a refused or broken connection, a failed TLS handshake) is
//! a warning: its row says `WARN`, GitHub Actions shows it as an annotation, and the run passes, as
//! the catalog may well be right. A fetch gets 7 tries within 3 minutes, a timeout 2, and a host
//! out of reach on every try isn't tried again in the run. It installs nothing and reads no apt,
//! dpkg or gpg state of the machine's: keys are checked in a gpg home of their own.
//!
//! `--changed BASE` checks the apps whose folder changed since the merge base with BASE, and the
//! apps whose vendor repository changed (`system/packages.toml`, a sources file). There an outage
//! on a pin the branch changed (an action that differs from the merge base's, or a repository whose
//! pin or sources file changed) is an error: nothing has checked that pin yet.
//!
//! `--bump` checks nothing: it moves each pinned `.deb` and download of the apps named, or of every
//! app, on to its vendor's newest release, writing the new version, URL and sha256 into the app's
//! `app.toml`. It exits 1 when a pin's newest release can't be found or has no file for it, as when
//! a vendor renames its downloads, and 3 when a feed was down and nothing failed, writing nothing
//! for that app. The daily `Apps bump` workflow runs it and opens a pull request for each app it
//! moved, whose checks run this check on the app.
//!
//! # Owns
//!
//! - The command line, picking the apps, running them side by side, the table and the exit code.
//!
//! # Not here
//!
//! - The checks: `check.rs`. Moving pins on: `bump.rs`. Repositories: `apt.rs`. Fetching:
//!   `fetch.rs`.

mod apt;
mod bump;
mod changed;
mod check;
mod fetch;
#[cfg(test)]
mod stub;

use std::collections::BTreeSet;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;

use kubaos_core::apps::Catalog;
use kubaos_core::apps::Folder;
use kubaos_core::apps::action::Kind;
use kubaos_core::apps::catalog::CATALOG;
use kubaos_core::apt::Manifest;

use crate::changed::Changed;
use crate::check::Context;
use crate::check::Outcome;
use crate::check::Row;
use crate::check::chain;

/// The Ubuntu release KubaOS installs, as `install/iso/lib/build-iso.sh`'s `SUITE` names it.
const DEFAULT_RELEASE: &str = "resolute";

const USAGE: &str = "\
usage: kubaos-apps-verify [--root DIR] [--release CODENAME] [ID...]
       kubaos-apps-verify [--root DIR] [--release CODENAME] --changed BASE
       kubaos-apps-verify [--root DIR] --bump [ID...]

Holds the Apps catalog's pins to their upstreams: vendor keys and InRelease signatures, package
candidates, pinned downloads' sha256, nixpkgs versions and web app URLs. Every app with no ID.
--bump moves the apps' pinned downloads on to their newest releases instead, in app.toml.";

struct Args {
    root: PathBuf,
    release: String,
    changed: Option<String>,
    bump: bool,
    ids: Vec<String>,
}

fn parse(mut args: impl Iterator<Item = String>) -> Result<Option<Args>, String> {
    let mut parsed = Args {
        root: PathBuf::from("."),
        release: DEFAULT_RELEASE.to_owned(),
        changed: None,
        bump: false,
        ids: Vec::new(),
    };
    while let Some(arg) = args.next() {
        let mut value = |name: &str| args.next().ok_or_else(|| format!("{name} takes a value"));
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--root" => parsed.root = PathBuf::from(value("--root")?),
            "--release" => parsed.release = value("--release")?,
            "--changed" => parsed.changed = Some(value("--changed")?),
            "--bump" => parsed.bump = true,
            flag if flag.starts_with('-') => return Err(format!("unknown flag {flag}")),
            _ => parsed.ids.push(arg),
        }
    }
    if parsed.changed.is_some() && !parsed.ids.is_empty() {
        return Err("--changed takes no IDs".to_owned());
    }
    if parsed.changed.is_some() && parsed.bump {
        return Err("--bump and --changed don't go together".to_owned());
    }
    Ok(Some(parsed))
}

#[expect(clippy::print_stdout, clippy::print_stderr, reason = "the command's output")]
fn main() -> ExitCode {
    let args = match parse(env::args().skip(1)) {
        Ok(Some(args)) => args,
        Ok(None) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(e) => {
            eprintln!("kubaos-apps-verify: {e}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match run(&args) {
        Ok(rows) => {
            print!("{}", table(&rows));
            if env::var_os("GITHUB_ACTIONS").is_some_and(|on| on == "true") {
                print!("{}", annotations(&rows));
            }
            let warned = rows.iter().filter(|row| row.outcome == Outcome::Warning).count();
            if warned > 0 {
                eprintln!("kubaos-apps-verify: {warned} warned, upstream down");
            }
            let failed = rows.iter().filter(|row| row.outcome.fails()).count();
            if failed > 0 {
                eprintln!("kubaos-apps-verify: {failed} failed");
            }
            ExitCode::from(exit_code(&rows, args.bump))
        }
        Err(Usage(e)) => {
            eprintln!("kubaos-apps-verify: {e}");
            ExitCode::from(2)
        }
    }
}

/// 1 when a row fails; else, for `--bump`, 3 when a row warned, so `test/apps-bump/run` can count
/// an app's outages; else 0: a check's warnings pass.
fn exit_code(rows: &[Row], bump: bool) -> u8 {
    if rows.iter().any(|row| row.outcome.fails()) {
        1
    } else if bump && rows.iter().any(|row| row.outcome == Outcome::Warning) {
        3
    } else {
        0
    }
}

/// A GitHub Actions `::warning::` line for each warning, so an outage shows on the run's summary
/// even though the run passes.
///
/// <https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands#setting-a-warning-message>
fn annotations(rows: &[Row]) -> String {
    let mut out = String::new();
    for row in rows.iter().filter(|row| row.outcome == Outcome::Warning) {
        let message = format!("{} {} {}: {}", row.app, row.check, row.subject, row.detail);
        out.push_str("::warning title=Upstream down::");
        out.push_str(&escape(&message));
        out.push('\n');
    }
    out
}

/// `text` escaped for a workflow command's message: `%`, CR and LF would end or change it.
fn escape(text: &str) -> String {
    text.replace('%', "%25").replace('\r', "%0D").replace('\n', "%0A")
}

/// An app that isn't the catalog's, or a catalog or manifest that can't be read.
struct Usage(String);

#[expect(clippy::print_stdout, reason = "the command's output")]
fn run(args: &Args) -> Result<Vec<Row>, Usage> {
    let catalog = Catalog::load(&args.root.join(CATALOG)).map_err(|e| Usage(chain(&e)))?;
    let manifest = Manifest::read(&args.root.join("system/packages.toml"))
        .map_err(|e| Usage(chain(&e)))?
        .unwrap_or_default();
    let mut fresh = None;
    let apps: Vec<&Folder> = match &args.changed {
        Some(base) => {
            let (apps, merge_base) = changed(&args.root, base, &catalog)?;
            fresh = Some(Changed::read(&args.root, &merge_base, &manifest));
            apps
        }
        None if args.ids.is_empty() => catalog.apps().iter().collect(),
        None => args
            .ids
            .iter()
            .map(|id| {
                catalog
                    .get(id)
                    .ok_or_else(|| Usage(format!("the catalog has no app {id}")))
            })
            .collect::<Result<_, _>>()?,
    };
    if apps.is_empty() {
        println!("kubaos-apps-verify: no app to check");
        return Ok(Vec::new());
    }
    let root = args
        .root
        .canonicalize()
        .map_err(|e| Usage(format!("{}: {e}", args.root.display())))?;
    if args.bump {
        let catalog = root.join(CATALOG);
        return Ok(side_by_side(&apps, |app| bump::app(&catalog, &app.id)));
    }
    let mut context = Context::new(root, manifest, args.release.clone());
    if let Some(fresh) = fresh {
        context = context.with_changed(fresh);
    }
    Ok(side_by_side(&apps, |app| check::app(&context, app)))
}

/// `each` app's rows, in the apps' order. Each app waits on the network, so the apps run side by
/// side.
fn side_by_side(apps: &[&Folder], each: impl Fn(&Folder) -> Vec<Row> + Sync) -> Vec<Row> {
    thread::scope(|scope| {
        let running: Vec<_> = apps.iter().map(|app| scope.spawn(|| each(app))).collect();
        running
            .into_iter()
            .flat_map(|rows| rows.join().expect("an app's checks panicked"))
            .collect()
    })
}

/// The apps whose folder changed since the merge base with `base`, and those taking packages from
/// a vendor repository whose pin or sources file changed; and the merge base.
fn changed<'a>(root: &Path, base: &str, catalog: &'a Catalog) -> Result<(Vec<&'a Folder>, String), Usage> {
    let git = |args: &[&str]| {
        let output = kubaos_git::at(root)
            .args(args)
            .output()
            .map_err(|e| Usage(format!("running git: {e}")))?;
        if !output.status.success() {
            return Err(Usage(format!(
                "git {}: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    };
    let merge_base = git(&["merge-base", base, "HEAD"])?;
    let paths = git(&["diff", "--name-only", &merge_base, "HEAD", "--"])?;
    Ok((touched(&paths, catalog), merge_base))
}

/// The apps the changed `paths`, one a line, reach: an app's own folder, or the pin or sources
/// file of a vendor repository it takes packages from.
fn touched<'a>(paths: &str, catalog: &'a Catalog) -> Vec<&'a Folder> {
    let mut ids = BTreeSet::new();
    let mut repos = BTreeSet::new();
    let mut every_repo = false;
    for path in paths.lines() {
        if let Some(rest) = path.strip_prefix(&format!("{CATALOG}/"))
            && let Some((id, _)) = rest.split_once('/')
        {
            ids.insert(id.to_owned());
        } else if path == "system/packages.toml" {
            every_repo = true;
        } else if let Some(name) = path
            .strip_prefix("system/files/apt/sources.list.d/")
            .and_then(|file| file.strip_suffix(".sources"))
        {
            repos.insert(name.to_owned());
        }
    }
    catalog
        .apps()
        .iter()
        .filter(|app| {
            ids.contains(&app.id)
                || app.actions.iter().chain(app.setups.iter().flat_map(|setup| &setup.actions)).any(
                    |action| matches!(&action.kind, Kind::Apt { repo: Some(repo), .. } if every_repo || repos.contains(repo)),
                )
        })
        .collect()
}

/// The rows as a table, its columns as wide as their widest cell, each cell of the last column
/// left whole.
fn table(rows: &[Row]) -> String {
    let header = ["APP", "CHECK", "WHAT", "RESULT", "DETAIL"];
    let cells: Vec<[&str; 5]> = rows
        .iter()
        .map(|row| {
            [
                row.app.as_str(),
                row.check,
                row.subject.as_str(),
                row.outcome.word(),
                row.detail.as_str(),
            ]
        })
        .collect();
    let mut widths = header.map(str::len);
    for line in &cells {
        for (width, cell) in widths.iter_mut().zip(line) {
            *width = (*width).max(cell.chars().count());
        }
    }
    let mut out = String::new();
    for line in std::iter::once(&header).chain(&cells) {
        for (i, cell) in line.iter().enumerate() {
            if i + 1 == line.len() {
                out.push_str(cell);
            } else {
                out.push_str(cell);
                out.push_str(&" ".repeat(widths[i] - cell.chars().count() + 2));
            }
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests;
