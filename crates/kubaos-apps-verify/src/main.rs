//! `kubaos-apps-verify`: holds the Apps catalog's pins to their upstreams, for a reviewer and CI.
//!
//! ```text
//! kubaos-apps-verify [--root DIR] [--release CODENAME] [ID…]   those apps, or every app
//! kubaos-apps-verify [--root DIR] --changed BASE               the apps a branch changed
//! ```
//!
//! For each app: a vendor repository's key against its pinned fingerprint and its `InRelease`
//! verified with that key alone; each package's candidate in that repository, or in Ubuntu's
//! archive for the release (its `Packages` indexes, never the machine's apt), for each
//! architecture; each pinned download's sha256 as served and as the vendor publishes it; each
//! nixpkgs package at the checkout's `flake.lock`, for each system; and a web app's URL. It prints
//! a table and exits 1 on a mismatch or a check that couldn't run, 2 on a usage error. It installs
//! nothing and reads no apt, dpkg or gpg state of the machine's: keys are checked in a gpg home of
//! their own.
//!
//! `--changed BASE` checks the apps whose folder changed since the merge base with BASE, and the
//! apps whose vendor repository changed (`system/packages.toml`, a sources file).
//!
//! # Owns
//!
//! - The command line, picking the apps, running them side by side, the table and the exit code.
//!
//! # Not here
//!
//! - The checks: `check.rs`. Repositories: `apt.rs`. Fetching: `fetch.rs`.

mod apt;
mod check;
mod fetch;

use std::collections::BTreeSet;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;
use std::thread;

use kubaos_core::apps::Catalog;
use kubaos_core::apps::Folder;
use kubaos_core::apps::action::Kind;
use kubaos_core::apps::catalog::CATALOG;
use kubaos_core::apt::Manifest;

use crate::check::Context;
use crate::check::Row;
use crate::check::chain;

/// The Ubuntu release KubaOS installs, as `install/iso/lib/build-iso.sh`'s `SUITE` names it.
const DEFAULT_RELEASE: &str = "resolute";

const USAGE: &str = "\
usage: kubaos-apps-verify [--root DIR] [--release CODENAME] [ID...]
       kubaos-apps-verify [--root DIR] [--release CODENAME] --changed BASE

Holds the Apps catalog's pins to their upstreams: vendor keys and InRelease signatures, package
candidates, pinned downloads' sha256, nixpkgs versions and web app URLs. Every app with no ID.";

struct Args {
    root: PathBuf,
    release: String,
    changed: Option<String>,
    ids: Vec<String>,
}

fn parse(mut args: impl Iterator<Item = String>) -> Result<Option<Args>, String> {
    let mut parsed = Args {
        root: PathBuf::from("."),
        release: DEFAULT_RELEASE.to_owned(),
        changed: None,
        ids: Vec::new(),
    };
    while let Some(arg) = args.next() {
        let mut value = |name: &str| args.next().ok_or_else(|| format!("{name} takes a value"));
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--root" => parsed.root = PathBuf::from(value("--root")?),
            "--release" => parsed.release = value("--release")?,
            "--changed" => parsed.changed = Some(value("--changed")?),
            flag if flag.starts_with('-') => return Err(format!("unknown flag {flag}")),
            _ => parsed.ids.push(arg),
        }
    }
    if parsed.changed.is_some() && !parsed.ids.is_empty() {
        return Err("--changed takes no IDs".to_owned());
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
            let failed = rows.iter().filter(|row| row.outcome.fails()).count();
            if failed > 0 {
                eprintln!("kubaos-apps-verify: {failed} failed");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(Usage(e)) => {
            eprintln!("kubaos-apps-verify: {e}");
            ExitCode::from(2)
        }
    }
}

/// An app that isn't the catalog's, or a catalog or manifest that can't be read.
struct Usage(String);

#[expect(clippy::print_stdout, reason = "the command's output")]
fn run(args: &Args) -> Result<Vec<Row>, Usage> {
    let catalog = Catalog::load(&args.root.join(CATALOG)).map_err(|e| Usage(chain(&e)))?;
    let manifest = Manifest::read(&args.root.join("system/packages.toml"))
        .map_err(|e| Usage(chain(&e)))?
        .unwrap_or_default();
    let apps: Vec<&Folder> = match &args.changed {
        Some(base) => changed(&args.root, base, &catalog)?,
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
    let context = Context::new(root, manifest, args.release.clone());
    // Each app's checks wait on the network, so the apps run side by side.
    let rows = thread::scope(|scope| {
        let running: Vec<_> = apps
            .iter()
            .map(|app| scope.spawn(|| check::app(&context, app)))
            .collect();
        running
            .into_iter()
            .flat_map(|each| each.join().expect("an app's checks panicked"))
            .collect()
    });
    Ok(rows)
}

/// The apps whose folder changed since the merge base with `base`, and those taking packages from
/// a vendor repository whose pin or sources file changed.
fn changed<'a>(root: &Path, base: &str, catalog: &'a Catalog) -> Result<Vec<&'a Folder>, Usage> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["diff", "--name-only", &format!("{base}...HEAD"), "--"])
        .output()
        .map_err(|e| Usage(format!("running git: {e}")))?;
    if !output.status.success() {
        return Err(Usage(format!(
            "git diff against {base}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(touched(&String::from_utf8_lossy(&output.stdout), catalog))
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
