//! `kubaos pin`, `unpin` and `pins`: Project pins, a Suite version for a folder and everything
//! below it.
//!
//!   kubaos pin SUITE VERSION [FOLDER]
//!   kubaos unpin SUITE [FOLDER]
//!   kubaos pins [FOLDER] [--json]
//!   kubaos pins --all [--json]
//!
//! `pin` writes `use suite SUITE VERSION` into the folder's `.envrc` (the current folder by
//! default), replacing the Suite's line if there is one, and runs `direnv allow` on it. `unpin`
//! removes the line, and the file when nothing else is left in it. `pins` shows the pins in
//! effect in a folder, and `--all` every pin under the Projects root. FOLDER may start with `~`,
//! and a relative one, `..` included, is taken from the shell's logical `$PWD`, as direnv walks
//! it. `pins FOLDER --json` prints one object (`folder`, `envrc`, `pins`, `hidden`), and
//! `pins --all --json` an array of pins; a path that isn't UTF-8 has its bad bytes replaced.
//! Exit codes: 0 ok, 1 the work failed, 2 usage.
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
use kubaos_core::pins::Fetch;
use kubaos_core::pins::Pin;
use kubaos_core::pins::PinError;
use kubaos_core::pins::PinStatus;
use kubaos_core::pins::Pins;
use kubaos_core::pins::Resolution;
use kubaos_core::state::Paths;
use serde_json::Value;
use serde_json::json;

use crate::job::report;
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
    /// The folder whose `.envrc` holds the pin; the current folder by default.
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
            None if pinned.created => format!(", in a new {}", tilde(&paths, &pinned.envrc)),
            None => String::new(),
        };
        println!(
            "pinned the {} Suite to {} for {}{was}",
            suite.title,
            args.version,
            tilde(&paths, &folder)
        );
        match pins.resolve(&folder) {
            Ok(resolution) => warn_overridden(&paths, &resolution, suite.name, &pinned.envrc),
            Err(e) => return report(e),
        }
        let status = match pins.status(Pin {
            suite: suite.name.to_owned(),
            version: args.version.clone(),
            envrc: pinned.envrc,
        }) {
            Ok(status) => status,
            Err(e) => return report(e),
        };
        for note in notes(&status) {
            println!("{note}");
        }
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
    let file = if unpinned.deleted {
        format!(
            ", and deleted {}, which held nothing else",
            tilde(&paths, &unpinned.envrc)
        )
    } else {
        String::new()
    };
    println!(
        "unpinned {} {} from {}{file}",
        args.suite,
        unpinned.version,
        tilde(&paths, &folder)
    );
    match pins.resolve(&folder) {
        Ok(resolution) => match resolution.pins.iter().find(|pin| pin.suite == args.suite) {
            Some(above) => println!(
                "{} {} from {} applies there now",
                above.suite,
                above.version,
                tilde(&paths, &above.envrc)
            ),
            None => println!("no {} pin applies there now", args.suite),
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
    if args.all {
        return all(&paths, &pins, args.json);
    }
    let resolution = match pins.resolve(&folder) {
        Ok(resolution) => resolution,
        Err(e) => return report(e),
    };
    let mut statuses = Vec::new();
    for pin in resolution.pins.clone() {
        match pins.status(pin) {
            Ok(status) => statuses.push(status),
            Err(e) => return report(e),
        }
    }
    if args.json {
        println!("{:#}", resolution_json(&folder, &resolution, &statuses));
        return ExitCode::SUCCESS;
    }
    if statuses.is_empty() {
        println!(
            "no Project pins in {}: the installed Suites' versions apply",
            tilde(&paths, &folder)
        );
    }
    for status in &statuses {
        println!(
            "{} {}  from {}  ({})",
            status.pin.suite,
            status.pin.version,
            tilde(&paths, &status.pin.envrc),
            describe(status)
        );
    }
    for hidden in &resolution.hidden {
        println!(
            "{} {} in {} doesn't reach here: {} has no `source_up`",
            hidden.pin.suite,
            hidden.pin.version,
            tilde(&paths, &hidden.pin.envrc),
            tilde(&paths, &hidden.by)
        );
    }
    ExitCode::SUCCESS
}

fn all(paths: &Paths, pins: &Pins, json: bool) -> ExitCode {
    let found = match pins.all() {
        Ok(found) => found,
        Err(e) => return report(e),
    };
    let mut statuses = Vec::new();
    for pin in found {
        match pins.status(pin) {
            Ok(status) => statuses.push(status),
            Err(e) => return report(e),
        }
    }
    if json {
        let rows: Vec<Value> = statuses.iter().map(status_json).collect();
        println!("{:#}", Value::Array(rows));
        return ExitCode::SUCCESS;
    }
    if statuses.is_empty() {
        println!(
            "no Project pins under {}; add one with `kubaos pin SUITE VERSION FOLDER`",
            tilde(paths, &paths.projects_root())
        );
    }
    for status in &statuses {
        let folder = status.pin.envrc.parent().unwrap_or(&status.pin.envrc);
        println!(
            "{}  {} {}  ({})",
            tilde(paths, folder),
            status.pin.suite,
            status.pin.version,
            describe(status)
        );
    }
    ExitCode::SUCCESS
}

/// The paths, the pins and the folder a command works on: `folder` with `~` expanded and made
/// absolute, or the current folder.
fn setup(folder: Option<&Path>) -> Result<(Paths, Pins, PathBuf), ExitCode> {
    if rustix::process::geteuid().is_root() {
        return Err(usage_error(
            "run kubaos pin as yourself, not root: the .envrc and direnv's allow list are yours",
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
/// folder, so `..` and the `.envrc` files above are the ones direnv sees; the resolved folder
/// otherwise.
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
            applies_from: Some(envrc),
            ..
        } => {
            let folder = envrc.parent().unwrap_or(envrc);
            Some(format!(
                "the pin in effect there is in {}: kubaos unpin {suite} {}",
                tilde(paths, envrc),
                tilde(paths, folder)
            ))
        }
        PinError::Allow { .. } | PinError::AllowFailed { .. } => {
            Some("the pin is written; run `direnv allow` in the folder once direnv works".to_owned())
        }
        _ => None,
    };
    let code = report(e);
    if let Some(hint) = hint {
        eprintln!("kubaos: {hint}");
    }
    code
}

/// Tells when the pin just written doesn't decide the version in its own folder: a later line
/// that sources a pin from above wins over it.
fn warn_overridden(paths: &Paths, resolution: &Resolution, suite: &str, envrc: &Path) {
    let Some(winner) = resolution.pins.iter().find(|pin| pin.suite == suite) else {
        return;
    };
    if winner.envrc != envrc {
        println!(
            "but {} {} from {} wins there: its `source_up` comes after the pin; move the pin below it",
            winner.suite,
            winner.version,
            tilde(paths, &winner.envrc)
        );
    }
}

/// What the user should know about a pin that was just written.
fn notes(status: &PinStatus) -> Vec<String> {
    let mut notes = Vec::new();
    let title = status.suite.map_or(status.pin.suite.as_str(), |suite| suite.title);
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
    match &status.toolchain {
        Fetch::InStore { .. } => {}
        Fetch::OnFirstUse => notes.push(format!(
            "{} {} isn't in the Nix store yet: direnv fetches it the first time you enter the folder",
            status.pin.suite, status.pin.version
        )),
        Fetch::Missing => {
            notes.push("your home has no toolchain for it yet: run `kubaos switch`, then enter the folder".to_owned());
        }
    }
    notes
}

/// "the home has 1; in the store".
fn describe(status: &PinStatus) -> String {
    let global = match (&status.installed, status.global_version()) {
        (None, _) if status.suite.is_none() => "not a KubaOS Suite".to_owned(),
        (None, _) => "Suite not installed".to_owned(),
        (Some(_), Some(global)) if global == status.pin.version => "the installed version".to_owned(),
        (Some(_), Some(global)) => format!("installed: {global}"),
        (Some(_), None) => "installed".to_owned(),
    };
    let toolchain = match &status.toolchain {
        Fetch::InStore { .. } => "in the store",
        Fetch::OnFirstUse => "fetched on first use",
        Fetch::Missing => "no toolchain in your home",
    };
    format!("{global}; {toolchain}")
}

/// `pins FOLDER --json`: the folder, its `.envrc`, the pins in effect and the hidden ones.
fn resolution_json(folder: &Path, resolution: &Resolution, statuses: &[PinStatus]) -> Value {
    json!({
        "folder": path_json(folder),
        "envrc": resolution.nearest.as_deref().map(path_json),
        "pins": statuses.iter().map(status_json).collect::<Vec<_>>(),
        "hidden": resolution.hidden.iter().map(|hidden| json!({
            "suite": hidden.pin.suite,
            "version": hidden.pin.version,
            "envrc": path_json(&hidden.pin.envrc),
            "by": path_json(&hidden.by),
        })).collect::<Vec<_>>(),
    })
}

/// `path` as a JSON string. serde fails on a path that isn't UTF-8; this replaces its bad bytes.
fn path_json(path: &Path) -> Value {
    Value::String(path.to_string_lossy().into_owned())
}

fn status_json(status: &PinStatus) -> Value {
    json!({
        "suite": status.pin.suite,
        "version": status.pin.version,
        "envrc": path_json(&status.pin.envrc),
        "known": status.suite.is_some(),
        "installed": status.installed.is_some(),
        "installed_version": status.global_version(),
        "toolchain": match &status.toolchain {
            Fetch::InStore { .. } => "in-store",
            Fetch::OnFirstUse => "on-first-use",
            Fetch::Missing => "missing",
        },
    })
}

#[cfg(test)]
mod tests;
