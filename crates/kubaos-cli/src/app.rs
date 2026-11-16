//! `kubaos app`: the Apps catalog, the few apps KubaOS installs well, and the user's picks of
//! them (`~/.config/kubaos/apps.toml`).
//!
//!   kubaos app list [--json]
//!   kubaos app add ID... [--setup SETUP] [--json]
//!   kubaos app remove ID... [--json]
//!   kubaos app status [--json]
//!
//! `list` is the catalog and which apps are picked, read from two files and nothing else, for the
//! Command menu. `status` adds where each app stands on the machine, from one dpkg query. `add`
//! picks apps and installs those the machine lacks; an app with setups needs `--setup`, as none
//! is the default, and one the machine has in another setup switches. `remove` drops apps and
//! takes them away.
//! Both ask for root once, through polkit, when an app has a root half (apt, a pinned `.deb`) and
//! for nothing otherwise, show their progress as `kubaos update` does, and with `--json` write its
//! events; `kubaos update` installs a picked app the machine lacks. Exit
//! codes: 0 ok, 1 the work failed, 2 usage.
//!
//! # Owns
//!
//! - The command line of `app`, its list and its status table.
//!
//! # Not here
//!
//! - The catalog, the picks and the comparison: `kubaos_core::apps`. Running the root helper and
//!   the switch: `kubaos_core::update`'s `updater/apps.rs`.

use std::fmt::Write as _;
use std::process::ExitCode;
use std::time::SystemTime;

use clap::Subcommand;
use kubaos_core::apps::AppState;
use kubaos_core::apps::Catalog;
use kubaos_core::apps::Picks;
use kubaos_core::apps::Status;
use kubaos_core::apps::plan::listing;
use kubaos_core::state::Paths;
use kubaos_core::update::Updater;
use kubaos_core::update::record::SystemPaths;

use crate::job;
use crate::job::SwitchIn;
use crate::out::out;
use crate::update;
use crate::update::Shown;
use crate::usage_error;

#[derive(Subcommand)]
pub enum AppCmd {
    /// The apps of the catalog, and which you picked.
    List {
        /// Write it as JSON, for the Command menu.
        #[arg(long)]
        json: bool,
    },
    /// Pick apps and install them: kubaos update keeps them installed.
    Add {
        #[arg(required = true)]
        ids: Vec<String>,
        /// The setup, for an app that can be set up more than one way (kubaos app list).
        #[arg(long, value_name = "SETUP")]
        setup: Option<String>,
        /// Write one JSON event per line on stdout, for a graphical front end.
        #[arg(long)]
        json: bool,
    },
    /// Drop apps from your picks and remove them.
    Remove {
        #[arg(required = true)]
        ids: Vec<String>,
        /// Write one JSON event per line on stdout, for a graphical front end.
        #[arg(long)]
        json: bool,
    },
    /// Where each app of the catalog stands on this machine.
    Status {
        /// Write it as JSON, for the Command menu.
        #[arg(long)]
        json: bool,
    },
}

pub fn main(cmd: AppCmd) -> ExitCode {
    match cmd {
        AppCmd::List { json } => list(json),
        AppCmd::Add { ids, setup, json } => change(&ids, Change::Add(setup.as_deref()), json),
        AppCmd::Remove { ids, json } => change(&ids, Change::Remove, json),
        AppCmd::Status { json } => status(json),
    }
}

#[expect(
    clippy::print_stdout,
    reason = "The list is the command's output, written whole as it ends its own lines."
)]
fn list(json: bool) -> ExitCode {
    let paths = match Paths::from_env() {
        Ok(paths) => paths,
        Err(e) => return usage_error(e),
    };
    let catalog = match Catalog::load(&SystemPaths::machine().catalog()) {
        Ok(catalog) => catalog,
        Err(e) => return job::report(e),
    };
    let picks = match Picks::read(&paths.app_picks()) {
        Ok(picks) => picks,
        Err(e) => return job::report(e),
    };
    let listed = listing(&catalog, &picks);
    if json {
        out!(
            "{}",
            serde_json::to_string(&listed).expect("the list always serializes")
        );
        return ExitCode::SUCCESS;
    }
    let width = listed.iter().map(|a| a.id.len()).max().unwrap_or(0);
    let mut text = String::new();
    for app in &listed {
        let mark = if app.picked { "picked" } else { "" };
        let _ = writeln!(text, "{:<6}  {:<width$}  {}: {}", mark, app.id, app.name, app.summary);
        for setup in app.setups {
            let mark = if app.setup == Some(setup.id.as_str()) {
                "picked"
            } else {
                ""
            };
            let _ = writeln!(
                text,
                "{:<6}  {:<width$}    --setup {}: {}, {}",
                mark, "", setup.id, setup.name, setup.summary
            );
        }
    }
    print!("{text}");
    ExitCode::SUCCESS
}

#[derive(Clone, Copy)]
enum Change<'a> {
    /// In the setup, for the apps that have setups.
    Add(Option<&'a str>),
    Remove,
}

fn change(ids: &[String], change: Change, json: bool) -> ExitCode {
    let (paths, mut system) = match update::machine(SwitchIn::Unit) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let mut shown = Shown::new(json);
    let mut updater = Updater::new(&mut system, SystemPaths::machine(), paths, SystemTime::now()).reporting(&mut shown);
    let done = match change {
        Change::Add(setup) => updater.add_apps(ids, setup),
        Change::Remove => updater.remove_apps(ids),
    };
    match done {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

#[expect(
    clippy::print_stdout,
    reason = "The table is the command's output, written whole as it ends its own lines."
)]
fn status(json: bool) -> ExitCode {
    let (paths, mut system) = match update::machine(SwitchIn::Unit) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let mut updater = Updater::new(&mut system, SystemPaths::machine(), paths, SystemTime::now());
    let status = match updater.app_status() {
        Ok(status) => status,
        Err(e) => return job::report(e),
    };
    if json {
        out!(
            "{}",
            serde_json::to_string(&status).expect("the status always serializes")
        );
    } else {
        print!("{}", table(&status));
    }
    ExitCode::SUCCESS
}

/// The status as `kubaos app status` prints it: one line an app, lined up, then what the next
/// update does about it.
fn table(status: &Status) -> String {
    let width = status.apps.iter().map(|a| a.id.len()).max().unwrap_or(0);
    let mut out = format!("KubaOS's apps ({})\n\n", status.arch);
    for app in &status.apps {
        let picked = if app.picked { "picked" } else { "" };
        let name = app
            .setup
            .as_ref()
            .and_then(|id| app.setups.iter().find(|setup| setup.id == *id))
            .map_or(&app.name, |setup| &setup.name);
        let mut line = format!("  {:<11} {:<6}  {:<width$}  {}", app.state.word(), picked, app.id, name);
        if let Some(version) = &app.version {
            let _ = write!(line, "  {version}");
        }
        let _ = writeln!(out, "{}", line.trim_end());
    }
    for id in &status.unknown {
        let _ = writeln!(out, "  {id}: picked, but this release's catalog has no such app");
    }
    let missing = status
        .apps
        .iter()
        .filter(|a| a.picked && matches!(a.state, AppState::Missing | AppState::Outdated))
        .count();
    if missing == 1 {
        out.push_str("\n1 picked app missing: kubaos update installs it.\n");
    } else if missing > 1 {
        let _ = writeln!(out, "\n{missing} picked apps missing: kubaos update installs them.");
    }
    out
}

#[cfg(test)]
mod tests;
