//! `kubaos package` and `kubaos try`: packages from the user's nixpkgs, recorded in the state
//! directory (`kubaos.d/packages.toml`) and installed by a switch.
//!
//!   kubaos package add NAME... [--no-switch] [--detach | --foreground]
//!   kubaos package remove NAME... [--no-switch] [--detach | --foreground]
//!   kubaos package list [--json]
//!   kubaos package restore GENERATION
//!   kubaos try NAME
//!
//! `add` and `remove` first wire `home.nix` to the state directory if it isn't yet, backing it up
//! to `~/.local/state/kubaos/backups/`.
//! `try` opens `nix shell` with the package, in this terminal or a new one, and writes nothing.
//! Exit codes: 0 ok, 1 the work failed, 2 usage.
//!
//! # Owns
//!
//! - The command line of `package` and `try`, and what they print.
//!
//! # Not here
//!
//! - Editing `packages.toml` and the `nix shell` command: `kubaos_core::packages`.
//! - The `home.nix` migration and the restore: `kubaos_core::state`.
//! - The switch: `job.rs`.

use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;

use clap::Subcommand;
use kubaos_core::packages::Packages;
use kubaos_core::packages::try_command;
use kubaos_core::state::Origin;
use kubaos_core::state::Paths;
use kubaos_core::state::migrate::Migration;
use kubaos_core::state::migrate::wire_home_nix;

use crate::job;
use crate::job::SwitchMode;
use crate::job::report;
use crate::out::out;
use crate::usage_error;

/// The terminal `try` opens when it is not run from one.
const TERMINAL: &str = "ghostty";

#[derive(Subcommand)]
pub enum PackageCmd {
    /// Install packages from your nixpkgs, then switch.
    Add {
        #[arg(required = true)]
        names: Vec<String>,
        /// Only record them; the next switch installs them.
        #[arg(long)]
        no_switch: bool,
        #[command(flatten)]
        mode: SwitchMode,
    },
    /// Uninstall packages added with `add`, then switch.
    Remove {
        #[arg(required = true)]
        names: Vec<String>,
        /// Only record it; the next switch uninstalls them.
        #[arg(long)]
        no_switch: bool,
        #[command(flatten)]
        mode: SwitchMode,
    },
    /// The packages kubaos.d/ installs, and which file lists each.
    List {
        #[arg(long)]
        json: bool,
    },
    /// Put back the kubaos.d/ a home-manager generation was built from (what a rollback does).
    Restore { generation: PathBuf },
}

pub fn main(cmd: PackageCmd) -> ExitCode {
    if rustix::process::geteuid().is_root() {
        return usage_error("run kubaos package as yourself, not root: it changes your home");
    }
    let paths = match Paths::from_env() {
        Ok(paths) => paths,
        Err(e) => return usage_error(e),
    };
    match cmd {
        PackageCmd::Add { names, no_switch, mode } => edit(&paths, &names, Edit::Add, no_switch, mode),
        PackageCmd::Remove { names, no_switch, mode } => edit(&paths, &names, Edit::Remove, no_switch, mode),
        PackageCmd::List { json } => list(&paths, json),
        PackageCmd::Restore { generation } => match paths.state_dir().restore_from_generation(&generation) {
            Ok(()) => {
                out!(
                    "restored {} from {}",
                    paths.state_dir().root().display(),
                    generation.display()
                );
                ExitCode::SUCCESS
            }
            Err(e) => report(e),
        },
    }
}

/// `kubaos try NAME`.
pub fn try_main(name: &str) -> ExitCode {
    let paths = match Paths::from_env() {
        Ok(paths) => paths,
        Err(e) => return usage_error(e),
    };
    let argv = match try_command(paths.flake(), name) {
        Ok(argv) => argv,
        Err(e) => return usage_error(e),
    };
    let Some((nix, nix_args)) = argv.split_first() else {
        return usage_error("nothing to run");
    };
    if std::io::stdin().is_terminal() {
        return match Command::new(nix).args(nix_args).status() {
            Ok(status) if status.success() => ExitCode::SUCCESS,
            Ok(_) => ExitCode::FAILURE,
            Err(e) => report(e),
        };
    }
    match Command::new(TERMINAL).arg("-e").args(&argv).spawn() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => report(e),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Edit {
    Add,
    Remove,
}

/// Wires `home.nix` to the state directory unless it is already, as every command that edits the
/// directory does first. Returns whether it changed the file, or the exit code of the failure.
pub fn wire(paths: &Paths) -> Result<bool, ExitCode> {
    let home_nix = paths.home_nix();
    if !home_nix.is_file() {
        return Err(usage_error(format!("no user flake: {} is missing", home_nix.display())));
    }
    match wire_home_nix(&home_nix, &paths.backups()) {
        Ok(Migration::AlreadyWired) => Ok(false),
        Ok(Migration::Wired { backup }) => {
            out!(
                "wired {} to kubaos.d/ (backup: {})",
                home_nix.display(),
                backup.display()
            );
            Ok(true)
        }
        Err(e) => Err(report(e)),
    }
}

fn edit(paths: &Paths, names: &[String], how: Edit, no_switch: bool, mode: SwitchMode) -> ExitCode {
    let wired = match wire(paths) {
        Ok(wired) => wired,
        Err(code) => return code,
    };
    let packages = Packages::new(&paths.state_dir(), paths.edit_lock());
    let change = match how {
        Edit::Add => packages.add(names),
        Edit::Remove => packages.remove(names),
    };
    let change = match change {
        Ok(change) => change,
        Err(e) => return report(e),
    };
    let (did, didnt) = match how {
        Edit::Add => ("added", "already there"),
        Edit::Remove => ("removed", "not added with kubaos package add"),
    };
    for name in &change.changed {
        out!("{did} {name}");
    }
    for name in &change.unchanged {
        out!("{name}: {didnt}");
    }
    if how == Edit::Remove {
        warn_still_in_suites(paths, names);
    }
    if no_switch || (change.is_empty() && !wired) {
        return ExitCode::SUCCESS;
    }
    job::switch(paths, mode)
}

/// A package the user asked to remove that a Suite lists stays installed; say so.
fn warn_still_in_suites(paths: &Paths, removed: &[String]) {
    let Ok(declared) = paths.state_dir().declared() else {
        return;
    };
    for name in removed {
        let suites: Vec<String> = declared
            .origins(name)
            .into_iter()
            .filter_map(|o| match o {
                Origin::Suite(suite) => Some(suite),
                Origin::Packages => None,
            })
            .collect();
        if !suites.is_empty() {
            out!("{name} stays installed: the {} Suite lists it", suites.join(", "));
        }
    }
}

fn list(paths: &Paths, json: bool) -> ExitCode {
    let declared = match paths.state_dir().declared() {
        Ok(declared) => declared,
        Err(e) => return report(e),
    };
    let rows: Vec<(String, Vec<String>)> = declared
        .names()
        .into_iter()
        .map(|name| {
            let from = declared
                .origins(name)
                .into_iter()
                .map(|o| match o {
                    Origin::Packages => "packages.toml".to_owned(),
                    Origin::Suite(suite) => format!("suites/{suite}.toml"),
                })
                .collect();
            (name.to_owned(), from)
        })
        .collect();
    if json {
        let value: serde_json::Map<String, serde_json::Value> =
            rows.into_iter().map(|(name, from)| (name, from.into())).collect();
        out!("{}", serde_json::to_string_pretty(&value).expect("the list serializes"));
        return ExitCode::SUCCESS;
    }
    for (name, from) in rows {
        out!("{name}  ({})", from.join(", "));
    }
    ExitCode::SUCCESS
}
