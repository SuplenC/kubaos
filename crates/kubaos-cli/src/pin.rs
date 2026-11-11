//! `kubaos pin`, `unpin` and `pins`: Project pins, a Suite version for a folder and everything
//! below it.
//!
//!   kubaos pin SUITE VERSION [FOLDER]
//!   kubaos unpin SUITE [FOLDER]
//!   kubaos pins [FOLDER] [--json]
//!   kubaos pins --all [--json]
//!
//! `pin` installs the version with mise, then sets the Suite's main tool in the folder's
//! `mise.toml` (the current folder by default), keeping what else the file holds. `unpin` removes
//! it, and the file when nothing else is left in it, then prunes. `pins` shows the pins mise
//! applies in a folder, and `--all` every pin under the Projects root; both name the `use suite`
//! and `use bwenv` lines left in an `.envrc`, which nothing reads. FOLDER may start with `~`, and
//! a relative one, `..` included, is taken from the shell's logical `$PWD`. `pins FOLDER --json`
//! prints one object (`folder`, `pins`, `leftovers`), and `pins --all --json` one of `pins` and
//! `leftovers`; a path that isn't UTF-8 has its bad bytes replaced. `pins --migrate`, which a
//! migration runs, moves the `.envrc` lines under the Projects root to `mise.toml`. Exit codes:
//! 0 ok, 1 the work failed, 2 usage.
//!
//! # Owns
//!
//! - The command line of `pin`, `unpin` and `pins`, and what they print.
//!
//! # Not here
//!
//! - Reading and writing pins: `kubaos_core::pins`.

use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;
use kubaos_core::pins::Leftover;
use kubaos_core::pins::LeftoverKind;
use kubaos_core::pins::Migrated;
use kubaos_core::pins::PinError;
use kubaos_core::pins::PinStatus;
use kubaos_core::pins::Pins;
use kubaos_core::pins::main_tool;
use kubaos_core::state::Paths;
use kubaos_core::suites::catalog;
use serde_json::Value;
use serde_json::json;

use crate::job::report;
use crate::out::err;
use crate::out::out;
use crate::suite::tilde;
use crate::suite::with_suite;
use crate::usage_error;

#[derive(Args)]
pub struct PinArgs {
    /// The Suite, as `kubaos suite list` names it.
    suite: String,
    /// One of the Suite's versions (`kubaos suite show SUITE`).
    version: String,
    /// The folder the pin covers, with everything below it; the current folder by default.
    folder: Option<PathBuf>,
}

#[derive(Args)]
pub struct UnpinArgs {
    suite: String,
    /// The folder whose `mise.toml` holds the pin; the current folder by default.
    folder: Option<PathBuf>,
}

#[derive(Args)]
pub struct PinsArgs {
    /// The folder to look from; the current folder by default.
    #[arg(conflicts_with = "all")]
    folder: Option<PathBuf>,
    /// Every pin under the Projects root instead.
    #[arg(long)]
    all: bool,
    #[arg(long)]
    json: bool,
    /// Moves the `use suite` and `use bwenv` lines of every `.envrc` under the Projects root to
    /// the folder's `mise.toml`; a migration runs it.
    #[arg(long, hide = true, conflicts_with_all = ["folder", "all", "json"])]
    migrate: bool,
}

pub fn pin(args: &PinArgs) -> ExitCode {
    let (paths, pins, folder) = match setup(args.folder.as_deref()) {
        Ok(found) => found,
        Err(code) => return code,
    };
    with_suite(&args.suite, |suite| {
        let pinned = match pins.pin(suite, &args.version, &folder) {
            Ok(pinned) => pinned,
            Err(e) => return report_pin(&paths, e),
        };
        let was = match &pinned.previous {
            Some(previous) if *previous == args.version => " (already)".to_owned(),
            Some(previous) => format!(" (was {previous})"),
            None if pinned.created => format!(", in a new {}", tilde(&paths, &pinned.config)),
            None => format!(", in {}", tilde(&paths, &pinned.config)),
        };
        out!(
            "pinned the {} Suite to {} for {}{was}",
            suite.title,
            args.version,
            tilde(&paths, &folder)
        );
        if let Some(applies) = &pinned.applies
            && applies.config != pinned.config
        {
            out!(
                "but {} {} from {} wins there: mise reads it after {}; change it there",
                applies.suite,
                applies.version,
                tilde(&paths, &applies.config),
                tilde(&paths, &pinned.config)
            );
        }
        let resolution = match pins.resolve(&folder) {
            Ok(resolution) => resolution,
            Err(e) => return report(e),
        };
        if let Some(status) = resolution.pins.iter().find(|status| status.pin.suite == suite.name) {
            for note in notes(status) {
                out!("{note}");
            }
        }
        warn_leftovers(&paths, &resolution.leftovers);
        ExitCode::SUCCESS
    })
}

pub fn unpin(args: &UnpinArgs) -> ExitCode {
    let (paths, pins, folder) = match setup(args.folder.as_deref()) {
        Ok(found) => found,
        Err(code) => return code,
    };
    let unpinned = match pins.unpin(&args.suite, &folder) {
        Ok(unpinned) => unpinned,
        Err(e) => return report_pin(&paths, e),
    };
    if unpinned.deleted {
        out!(
            "unpinned {} {} from {}, and deleted {}, which held nothing else",
            args.suite,
            unpinned.version,
            tilde(&paths, &folder),
            tilde(&paths, &unpinned.config)
        );
    } else {
        out!(
            "unpinned {} {} from {}",
            args.suite,
            unpinned.version,
            tilde(&paths, &folder)
        );
    }
    match pins.resolve(&folder) {
        Ok(resolution) => match resolution.pins.iter().find(|status| status.pin.suite == args.suite) {
            Some(above) => out!(
                "{} {} from {} applies there now",
                above.pin.suite,
                above.pin.version,
                tilde(&paths, &above.pin.config)
            ),
            None => out!("no {} pin applies there now", args.suite),
        },
        Err(e) => return report(e),
    }
    ExitCode::SUCCESS
}

pub fn list(args: &PinsArgs) -> ExitCode {
    let (paths, pins, folder) = match setup(args.folder.as_deref()) {
        Ok(found) => found,
        Err(code) => return code,
    };
    if args.migrate {
        return migrate(&paths, &pins);
    }
    if args.all {
        return all(&paths, &pins, args.json);
    }
    let resolution = match pins.resolve(&folder) {
        Ok(resolution) => resolution,
        Err(e) => return report(e),
    };
    if args.json {
        out!(
            "{:#}",
            json!({
                "folder": path_json(&folder),
                "pins": resolution.pins.iter().map(status_json).collect::<Vec<_>>(),
                "leftovers": resolution.leftovers.iter().map(leftover_json).collect::<Vec<_>>(),
            })
        );
        return ExitCode::SUCCESS;
    }
    if resolution.pins.is_empty() {
        out!(
            "no Project pins in {}: the installed Suites' versions apply",
            tilde(&paths, &folder)
        );
    }
    for status in &resolution.pins {
        out!(
            "{} {}  from {}  ({})",
            status.pin.suite,
            status.pin.version,
            tilde(&paths, &status.pin.config),
            describe(status)
        );
    }
    warn_leftovers(&paths, &resolution.leftovers);
    ExitCode::SUCCESS
}

fn all(paths: &Paths, pins: &Pins, json: bool) -> ExitCode {
    let found = match pins.all() {
        Ok(found) => found,
        Err(e) => return report(e),
    };
    let statuses = match pins.statuses(found.pins) {
        Ok(statuses) => statuses,
        Err(e) => return report(e),
    };
    if json {
        out!(
            "{:#}",
            json!({
                "pins": statuses.iter().map(status_json).collect::<Vec<_>>(),
                "leftovers": found.leftovers.iter().map(leftover_json).collect::<Vec<_>>(),
            })
        );
        return ExitCode::SUCCESS;
    }
    if statuses.is_empty() {
        out!(
            "no Project pins under {}; add one with `kubaos pin SUITE VERSION FOLDER`",
            tilde(paths, &paths.projects_root())
        );
    }
    for status in &statuses {
        let folder = status.pin.config.parent().unwrap_or(&status.pin.config);
        out!(
            "{}  {} {}  ({})",
            tilde(paths, folder),
            status.pin.suite,
            status.pin.version,
            describe(status)
        );
    }
    warn_leftovers(paths, &found.leftovers);
    ExitCode::SUCCESS
}

/// `pins --migrate`: moves the `.envrc` lines under the Projects root, then names the ones left.
fn migrate(paths: &Paths, pins: &Pins) -> ExitCode {
    let migrated = match pins.migrate() {
        Ok(migrated) => migrated,
        Err(e) => return report(e),
    };
    for done in &migrated {
        print_migrated(paths, done);
    }
    match pins.all() {
        Ok(found) => warn_leftovers(paths, &found.leftovers),
        Err(e) => return report(e),
    }
    ExitCode::SUCCESS
}

fn print_migrated(paths: &Paths, done: &Migrated) {
    let envrc = tilde(paths, &done.envrc);
    let config = tilde(paths, &done.config);
    for (suite, version) in &done.pins {
        out!("{envrc}: {suite} {version} is pinned in {config} now");
    }
    for (suite, version) in &done.kept {
        out!("{envrc}: dropped {suite} {version}, since {config} pins {suite} and its pin applied already");
    }
    if !done.bwenv.is_empty() {
        out!(
            "{envrc}: the Bitwarden projects {} load from {config} now",
            done.bwenv.join(" ")
        );
    }
    if done.envrc_deleted {
        out!("deleted {envrc}, which held nothing else");
    }
}

/// Names each `.envrc` line nothing reads, with what to do instead.
fn warn_leftovers(paths: &Paths, leftovers: &[Leftover]) {
    for left in leftovers {
        let at = format!("{}:{}", tilde(paths, &left.envrc), left.line);
        let folder = tilde(paths, left.envrc.parent().unwrap_or(&left.envrc));
        match &left.kind {
            LeftoverKind::Suite { suite, version } => {
                let pinnable = catalog::find(suite).is_some_and(|known| main_tool(known).is_ok());
                if pinnable {
                    out!(
                        "{at}: nothing reads `use suite {suite} {version}`; run `kubaos pin {suite} {version} {folder}` and delete the line"
                    );
                } else {
                    out!(
                        "{at}: nothing reads `use suite {suite} {version}`, and the {suite} Suite can't be pinned; delete the line"
                    );
                }
            }
            LeftoverKind::Bwenv { projects } => out!(
                "{at}: nothing reads `use bwenv {}`; load it from {folder}/mise.toml instead (docs/SECRETS.md)",
                projects.join(" ")
            ),
        }
    }
}

/// The paths, the pins and the folder a command works on: `folder` with `~` expanded and made
/// absolute, or the current folder.
fn setup(folder: Option<&Path>) -> Result<(Paths, Pins, PathBuf), ExitCode> {
    if rustix::process::geteuid().is_root() {
        return Err(usage_error(
            "run kubaos pin as yourself, not root: the mise.toml and mise's installs are yours",
        ));
    }
    let paths = Paths::from_env().map_err(usage_error)?;
    let cwd = logical_cwd().map_err(|e| usage_error(format!("finding the current folder: {e}")))?;
    let folder = match folder {
        Some(folder) => normalize(&cwd.join(expand_tilde(&paths, folder))),
        None => cwd,
    };
    let pins = Pins::new(&paths);
    Ok((paths, pins, folder))
}

/// The current folder as the shell names it: `$PWD`, links and all, when it is this process's
/// folder, so `..` is the one the user sees; the resolved folder otherwise.
fn logical_cwd() -> io::Result<PathBuf> {
    let physical = std::env::current_dir()?;
    let Some(pwd) = std::env::var_os("PWD").map(PathBuf::from) else {
        return Ok(physical);
    };
    let same = |a: &Path, b: &Path| match (fs::metadata(a), fs::metadata(b)) {
        (Ok(a), Ok(b)) => a.dev() == b.dev() && a.ino() == b.ino(),
        _ => false,
    };
    if pwd.is_absolute() && same(&pwd, &physical) {
        Ok(normalize(&pwd))
    } else {
        Ok(physical)
    }
}

/// `path` with its `.` and `..` parts worked out from the text alone, as the shell's `cd` does,
/// without resolving links.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                if out.file_name().is_some() {
                    out.pop();
                } else if !out.has_root() {
                    out.push(part);
                }
            }
            _ => out.push(part),
        }
    }
    out
}

/// `path` with a leading `~` or `~/` as the home. The Command menu passes a typed path on as it
/// is, with no shell to expand it.
fn expand_tilde(paths: &Paths, path: &Path) -> PathBuf {
    match path.strip_prefix("~") {
        Ok(rest) => paths.home().join(rest),
        Err(_) => path.to_path_buf(),
    }
}

/// Reports a failed pin or unpin, with the way on where there is one.
fn report_pin(paths: &Paths, e: PinError) -> ExitCode {
    let hint = match &e {
        PinError::NotPinned {
            suite,
            applies_from: Some(config),
            ..
        } => {
            let folder = config.parent().unwrap_or(config);
            Some(format!(
                "the pin in effect there is in {}: kubaos unpin {suite} {}",
                tilde(paths, config),
                tilde(paths, folder)
            ))
        }
        PinError::Resolve { folder, .. } => Some(format!(
            "the pin is written; `mise ls --current` in {} says what mise makes of the folder",
            tilde(paths, folder)
        )),
        PinError::Prune(_) => Some("the pin is removed; `mise prune` deletes the version it leaves".to_owned()),
        _ => None,
    };
    let code = report(e);
    if let Some(hint) = hint {
        err!("kubaos: {hint}");
    }
    code
}

/// What the user should know about a pin that was just written.
fn notes(status: &PinStatus) -> Vec<String> {
    let mut notes = Vec::new();
    let title = status.suite.title;
    match (&status.installed, status.global_version()) {
        (None, _) => notes.push(format!(
            "the {title} Suite isn't installed: the pin puts its toolchain on PATH there, without \
             its editor setup (`kubaos suite add {}` for that)",
            status.pin.suite
        )),
        (Some(_), Some(global)) if global != status.pin.version => {
            notes.push(format!("everywhere else, the installed {global} stays"));
        }
        _ => {}
    }
    notes
}

/// "installed: 1; 3.0.0".
fn describe(status: &PinStatus) -> String {
    let global = match (&status.installed, status.global_version()) {
        (None, _) => "Suite not installed".to_owned(),
        (Some(_), Some(global)) if global == status.pin.version => "the installed version".to_owned(),
        (Some(_), Some(global)) => format!("installed: {global}"),
        (Some(_), None) => "installed".to_owned(),
    };
    if let Some(reason) = &status.unresolved {
        return format!("{global}; unresolved: {reason}");
    }
    match (&status.selected, status.fetched) {
        (Some(version), true) => format!("{global}; {version}"),
        (Some(version), false) => format!("{global}; {version} not installed: `mise install` in the folder"),
        (None, _) => format!("{global}; another config in the folder wins"),
    }
}

/// `path` as a JSON string. serde fails on a path that isn't UTF-8; this replaces its bad bytes.
fn path_json(path: &Path) -> Value {
    Value::String(path.to_string_lossy().into_owned())
}

fn status_json(status: &PinStatus) -> Value {
    json!({
        "suite": status.pin.suite,
        "tool": status.pin.tool,
        "version": status.pin.version,
        "config": path_json(&status.pin.config),
        "installed": status.installed.is_some(),
        "installed_version": status.global_version(),
        "selected": status.selected,
        "fetched": status.fetched,
        "unresolved": status.unresolved,
    })
}

fn leftover_json(left: &Leftover) -> Value {
    let mut row = json!({
        "envrc": path_json(&left.envrc),
        "line": left.line,
    });
    match &left.kind {
        LeftoverKind::Suite { suite, version } => {
            row["kind"] = json!("suite");
            row["suite"] = json!(suite);
            row["version"] = json!(version);
        }
        LeftoverKind::Bwenv { projects } => {
            row["kind"] = json!("bwenv");
            row["projects"] = json!(projects);
        }
    }
    row
}

#[cfg(test)]
mod tests;
