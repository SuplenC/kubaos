//! `kubaos packages`: KubaOS's apt packages, as the release's package manifest
//! (`system/packages.toml`) lists them.
//!
//!   kubaos packages status [--json]
//!   kubaos packages enable GROUP [--json]
//!   kubaos packages disable GROUP [--json]
//!
//! `status` lists each group with its packages, installed, held, missing or skipped, and the
//! extra ones KubaOS installed that no group lists now; `--json` is the same for the Command
//! menu and the updater's front end. `enable` and `disable` turn an optional group (Docker,
//! Chrome, the security key, PIV, TUXEDO's drivers) on or off for good: the choice is kept in
//! `/etc/kubaos/apt-groups`, so `kubaos update` follows it. They show their progress as
//! `kubaos update` does, ask for sudo, and with `--json` write its events. Exit codes: 0 ok,
//! 1 the work failed, 2 usage.
//!
//! # Owns
//!
//! - The command line of `packages`, and the status table.
//!
//! # Not here
//!
//! - The manifest, the choices and the comparison: `kubaos_core::apt`. Running apt:
//!   `kubaos_core::update`. Nix packages (`kubaos package`): `package.rs`.

use std::fmt::Write as _;
use std::process::ExitCode;
use std::time::SystemTime;

use clap::Subcommand;
use kubaos_core::apt::Status;
use kubaos_core::apt::Switch;
use kubaos_core::apt::plan::GroupStatus;
use kubaos_core::apt::plan::PackageState;
use kubaos_core::apt::plan::PackageStatus;
use kubaos_core::update::Updater;
use kubaos_core::update::record::SystemPaths;

use crate::job;
use crate::job::SwitchIn;
use crate::update;
use crate::update::render::Mode;
use crate::update::render::Ui;

#[derive(Subcommand)]
pub enum PackagesCmd {
    /// How the apt packages compare with the release's package manifest.
    Status {
        /// Write it as JSON, for the Command menu.
        #[arg(long)]
        json: bool,
    },
    /// Turn an optional group on: install it now and keep it in every update.
    Enable {
        group: String,
        /// Write one JSON event per line on stdout, for a graphical front end.
        #[arg(long)]
        json: bool,
    },
    /// Turn an optional group off: remove what KubaOS installed for it and leave it out of updates.
    Disable {
        group: String,
        /// Write one JSON event per line on stdout, for a graphical front end.
        #[arg(long)]
        json: bool,
    },
}

pub fn main(cmd: PackagesCmd) -> ExitCode {
    match cmd {
        PackagesCmd::Status { json } => status(json),
        PackagesCmd::Enable { group, json } => set(&group, Switch::On, json),
        PackagesCmd::Disable { group, json } => set(&group, Switch::Off, json),
    }
}

fn status(json: bool) -> ExitCode {
    let (paths, mut system) = match update::machine(SwitchIn::Unit) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let mut updater = Updater::new(&mut system, SystemPaths::machine(), paths, SystemTime::now());
    let status = match updater.package_status() {
        Ok(status) => status,
        Err(e) => return job::report(e),
    };
    if json {
        println!(
            "{}",
            serde_json::to_string(&status).expect("the status always serializes")
        );
    } else {
        print!("{}", table(&status));
    }
    ExitCode::SUCCESS
}

fn set(group: &str, switch: Switch, json: bool) -> ExitCode {
    let (paths, mut system) = match update::machine(SwitchIn::Unit) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let mut ui = Ui::new(Mode::detect(json));
    let mut updater = Updater::new(&mut system, SystemPaths::machine(), paths, SystemTime::now()).reporting(&mut ui);
    match updater.set_group(group, switch) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

/// The status as `kubaos packages status` prints it: a group a block, its packages under it,
/// lined up; a group that is off or for another architecture in one line, with the packages of
/// it that are installed all the same.
fn table(status: &Status) -> String {
    let all = status.groups.iter().flat_map(|g| &g.packages).chain(&status.extra);
    let width = all.map(|p| p.name.len()).max().unwrap_or(0);
    let mut out = format!("KubaOS's apt packages ({})\n", status.arch);
    for group in &status.groups {
        out.push('\n');
        out.push_str(&heading(group));
        for package in &group.packages {
            if group.active || package.version.is_some() {
                out.push_str(&line(package, width));
            }
        }
    }
    if !status.extra.is_empty() {
        out.push_str("\nextra: KubaOS installed them, no group lists them; kubaos update removes them\n");
        for package in &status.extra {
            out.push_str(&line(package, width));
        }
    }
    let missing = status
        .groups
        .iter()
        .flat_map(|g| &g.packages)
        .filter(|p| p.state == PackageState::Missing)
        .count();
    if missing > 0 {
        let them = if missing == 1 { "it" } else { "them" };
        let _ = writeln!(out, "\n{} missing: kubaos update installs {them}.", count(missing));
    }
    out
}

fn heading(group: &GroupStatus) -> String {
    let mut text = format!("{}: {}", group.name, group.summary);
    match (group.inactive.as_deref(), group.optional) {
        (Some("off"), _) => {
            let _ = write!(text, " [off: kubaos packages enable {}]", group.name);
        }
        (Some(why), _) => {
            let _ = write!(text, " [{why}]");
        }
        (None, Some(_)) => {
            let _ = write!(text, " [on: kubaos packages disable {}]", group.name);
        }
        (None, None) => {}
    }
    text.push('\n');
    text
}

fn line(package: &PackageStatus, width: usize) -> String {
    let mut text = format!("  {:<11} {:<width$}", package.state.word(), package.name);
    if let Some(version) = &package.version {
        let _ = write!(text, "  {version}");
    }
    if package.state == PackageState::Extra && !package.group.is_empty() {
        let _ = write!(text, "  (was {})", package.group);
    }
    format!("{}\n", text.trim_end())
}

/// "1 package", "3 packages".
fn count(n: usize) -> String {
    if n == 1 {
        "1 package".to_owned()
    } else {
        format!("{n} packages")
    }
}

#[cfg(test)]
mod tests;
