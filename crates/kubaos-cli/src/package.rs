//! `kubaos package` and `kubaos try`: packages from the user's nixpkgs, recorded in the state
//! directory (`kubaos.d/packages.toml`) and installed by a switch.
//!
//!   kubaos package add NAME... [--no-switch] [--detach | --foreground]
//!   kubaos package remove NAME... [--no-switch] [--detach | --foreground]
//!   kubaos package list [--json]
//!   kubaos package unfree [on|off] [--no-switch] [--detach | --foreground]
//!   kubaos package restore GENERATION
//!   kubaos try NAME
//!
//! `add` and `remove` first wire `home.nix` to the state directory if it isn't yet, backing it up
//! to `~/.local/state/kubaos/backups/`.
//! `add` and `list` mark unfree packages, from one `nix eval` per command, and `add` refuses one
//! while `unfree` is off. `unfree off` refuses while a package kubaos.d/ installs is unfree, which
//! would fail the switch.
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
//! - The unfree setting and the license check: `kubaos_core::unfree`.
//! - The `home.nix` migration and the restore: `kubaos_core::state`.
//! - The switch: `job.rs`.

use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;

use clap::Subcommand;
use clap::ValueEnum;
use kubaos_core::packages::Packages;
use kubaos_core::packages::try_command;
use kubaos_core::packages::validate_name;
use kubaos_core::state::Declared;
use kubaos_core::state::Origin;
use kubaos_core::state::Paths;
use kubaos_core::state::migrate::Migration;
use kubaos_core::state::migrate::wire_home_nix;
use kubaos_core::unfree::License;
use kubaos_core::unfree::LicenseCheck;
use kubaos_core::unfree::Licenses;
use kubaos_core::unfree::Setting;
use kubaos_core::unfree::Unfree;

use crate::job;
use crate::job::SwitchMode;
use crate::job::chain;
use crate::job::report;
use crate::out::err;
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
    /// The packages kubaos.d/ installs, which file lists each, and which are unfree.
    List {
        #[arg(long)]
        json: bool,
    },
    /// Whether your nixpkgs takes unfree packages: print it, or turn it on or off, then switch.
    Unfree {
        to: Option<OnOff>,
        /// Only record it; the next switch applies it.
        #[arg(long)]
        no_switch: bool,
        #[command(flatten)]
        mode: SwitchMode,
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
        PackageCmd::Unfree { to, no_switch, mode } => unfree(&paths, to, no_switch, mode),
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

/// What `kubaos package unfree` turns unfree packages to.
#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OnOff {
    /// Unfree packages install like any other (the default).
    On,
    /// Only free packages install.
    Off,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Edit {
    Add,
    Remove,
}

/// Wires `home.nix` to the state directory unless it is already, as every command that edits the
/// directory does first. Returns whether it changed the file, or the exit code of the failure.
pub fn wire(paths: &Paths) -> Result<bool, ExitCode> {
    let home_nix = need_user_flake(paths)?;
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

/// The user flake's `home.nix`, or the exit code of a usage error when it is missing.
fn need_user_flake(paths: &Paths) -> Result<PathBuf, ExitCode> {
    let home_nix = paths.home_nix();
    if home_nix.is_file() {
        Ok(home_nix)
    } else {
        Err(usage_error(format!("no user flake: {} is missing", home_nix.display())))
    }
}

fn edit(paths: &Paths, names: &[String], how: Edit, no_switch: bool, mode: SwitchMode) -> ExitCode {
    let licenses = if how == Edit::Add {
        match admit(paths, names) {
            Ok(licenses) => licenses,
            Err(code) => return code,
        }
    } else {
        Licenses::default()
    };
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
        if licenses.is_unfree(name) {
            out!("{did} {name} (unfree)");
        } else {
            out!("{did} {name}");
        }
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

/// Checks, before anything is written, that the setting lets `names` in: one `nix eval` of their
/// licenses, which `add` then marks. While unfree packages are allowed, a check that can't run
/// only warns; while they are refused, it stops the add, which would otherwise fail the switch.
fn admit(paths: &Paths, names: &[String]) -> Result<Licenses, ExitCode> {
    need_user_flake(paths)?;
    for name in names {
        validate_name(name).map_err(usage_error)?;
    }
    let setting = Setting::new(&paths.state_dir(), paths.edit_lock())
        .get()
        .map_err(report)?;
    match LicenseCheck::new(paths.flake()).run(names) {
        Ok(licenses) => {
            setting.admit(&licenses).map_err(report)?;
            Ok(licenses)
        }
        Err(e) if setting == Unfree::Allowed => {
            err!("kubaos: not telling unfree packages apart: {}", chain(&e));
            Ok(Licenses::default())
        }
        Err(e) => {
            err!(
                "kubaos: unfree packages are turned off, and checking that these are free failed: {}",
                chain(&e)
            );
            Err(ExitCode::FAILURE)
        }
    }
}

/// `kubaos package unfree [on|off]`.
fn unfree(paths: &Paths, to: Option<OnOff>, no_switch: bool, mode: SwitchMode) -> ExitCode {
    let setting = Setting::new(&paths.state_dir(), paths.edit_lock());
    let Some(to) = to else {
        return match setting.get() {
            Ok(Unfree::Allowed) => {
                out!("on");
                ExitCode::SUCCESS
            }
            Ok(Unfree::Refused) => {
                out!("off");
                ExitCode::SUCCESS
            }
            Err(e) => report(e),
        };
    };
    let wanted = match to {
        OnOff::On => Unfree::Allowed,
        OnOff::Off => Unfree::Refused,
    };
    if let Err(code) = need_user_flake(paths) {
        return code;
    }
    if wanted == Unfree::Refused
        && let Err(code) = refuse_installed_unfree(paths)
    {
        return code;
    }
    let wired = match wire(paths) {
        Ok(wired) => wired,
        Err(code) => return code,
    };
    let changed = match setting.set(wanted) {
        Ok(changed) => changed,
        Err(e) => return report(e),
    };
    match (wanted, changed) {
        (Unfree::Allowed, true) => out!("unfree packages: on"),
        (Unfree::Refused, true) => out!("unfree packages: off, free ones only"),
        (Unfree::Allowed, false) => out!("unfree packages: already on"),
        (Unfree::Refused, false) => out!("unfree packages: already off"),
    }
    if no_switch || (!changed && !wired) {
        return ExitCode::SUCCESS;
    }
    job::switch(paths, mode)
}

/// Fails, naming them and their files, while a package kubaos.d/ installs is unfree: with the
/// setting off, the switch would fail on it. A check that can't run fails too.
fn refuse_installed_unfree(paths: &Paths) -> Result<(), ExitCode> {
    let declared = paths.state_dir().declared().map_err(report)?;
    let names = declared.names();
    let licenses = LicenseCheck::new(paths.flake()).run(&names).map_err(|e| {
        err!("kubaos: checking the installed packages for unfree ones: {}", chain(&e));
        ExitCode::FAILURE
    })?;
    let unfree = licenses.unfree();
    if unfree.is_empty() {
        return Ok(());
    }
    for name in &unfree {
        err!("kubaos: {name} is unfree ({})", origins(&declared, name).join(", "));
    }
    let removable: Vec<&str> = unfree
        .iter()
        .copied()
        .filter(|name| declared.origins(name).contains(&Origin::Packages))
        .collect();
    let them = if unfree.len() == 1 { "it" } else { "them" };
    if removable.is_empty() {
        err!("kubaos: remove {them} before turning unfree packages off");
    } else {
        err!(
            "kubaos: remove {them} before turning unfree packages off: kubaos package remove {}",
            removable.join(" ")
        );
    }
    Err(ExitCode::FAILURE)
}

/// The files that list `name`, as `list` prints them.
fn origins(declared: &Declared, name: &str) -> Vec<String> {
    declared
        .origins(name)
        .into_iter()
        .map(|o| match o {
            Origin::Packages => "packages.toml".to_owned(),
            Origin::Suite(suite) => format!("suites/{suite}.toml"),
        })
        .collect()
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
    let names = declared.names();
    let licenses = match LicenseCheck::new(paths.flake()).run(&names) {
        Ok(licenses) => licenses,
        Err(e) => {
            err!("kubaos: not telling unfree packages apart: {}", chain(&e));
            Licenses::default()
        }
    };
    if json {
        let value: serde_json::Map<String, serde_json::Value> = names
            .iter()
            .map(|name| {
                let unfree = match licenses.get(name) {
                    License::Unfree => serde_json::Value::Bool(true),
                    License::Free => serde_json::Value::Bool(false),
                    License::Unknown => serde_json::Value::Null,
                };
                let row = serde_json::json!({ "from": origins(&declared, name), "unfree": unfree });
                ((*name).to_owned(), row)
            })
            .collect();
        out!("{}", serde_json::to_string_pretty(&value).expect("the list serializes"));
        return ExitCode::SUCCESS;
    }
    for name in names {
        let from = origins(&declared, name).join(", ");
        if licenses.is_unfree(name) {
            out!("{name}  ({from})  unfree");
        } else {
            out!("{name}  ({from})");
        }
    }
    ExitCode::SUCCESS
}
