//! `kubaos app`: the Apps catalog, the few apps KubaOS installs well, and the user's picks of
//! them (`~/.config/kubaos/apps.toml`).
//!
//!   kubaos app list [--json]
//!   kubaos app show ID [--json]
//!   kubaos app add ID... [--setup SETUP] [--json]
//!   kubaos app remove ID... [--json]
//!   kubaos app status [--json]
//!   kubaos app adopt
//!
//! `list` is the catalog and which apps are picked, read from the catalog and the picks and
//! nothing else, for the Command menu. `show` is one app: what it is, and what it owns on this
//! machine, step by step, from its journals. `status` adds where each app stands, from the
//! journals and one dpkg query for the versions. `add` picks apps and installs them, every step
//! checked and those not done done; an app with setups needs `--setup`, as none is the default,
//! and one the machine has in another setup switches. `remove` drops apps and undoes their
//! journals. Both ask for root once, through polkit, before their first step when a step needs
//! it, show each step as `kubaos update` does, and with `--json` write its events; `kubaos update`
//! keeps the picked apps installed. `adopt` takes in the apps the machine had before KubaOS kept
//! journals, running nothing but checks; a user migration runs it. Exit codes: 0 ok, 1 the work
//! failed, 2 usage.
//!
//! # Owns
//!
//! - The command line of `app`, its list and its status table.
//!
//! # Not here
//!
//! - The catalog, the picks, the journals and the status: `kubaos_core::apps`. Running the engine
//!   and the root helper: `kubaos_core::update`'s `updater/apps.rs`.

use std::fmt::Write as _;
use std::process::ExitCode;
use std::time::SystemTime;

use clap::Subcommand;
use kubaos_core::apps::AppState;
use kubaos_core::apps::Catalog;
use kubaos_core::apps::Folder;
use kubaos_core::apps::Picks;
use kubaos_core::apps::Status;
use kubaos_core::apps::folder::FolderError;
use kubaos_core::apps::journal::Journal;
use kubaos_core::apps::journal::JournalError;
use kubaos_core::apps::status::listing;
use kubaos_core::state::Paths;
use kubaos_core::update::Updater;
use kubaos_core::update::record::SystemPaths;
use serde::Serialize;

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
    /// One app: what it is, and what it owns on this machine.
    Show {
        id: String,
        /// Write it as JSON.
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
    /// Take in the apps this machine had before KubaOS kept their journals; a migration runs it.
    #[command(hide = true)]
    Adopt,
}

pub fn main(cmd: AppCmd) -> ExitCode {
    match cmd {
        AppCmd::List { json } => list(json),
        AppCmd::Add { ids, setup, json } => change(&ids, Change::Add(setup.as_deref()), json),
        AppCmd::Remove { ids, json } => change(&ids, Change::Remove, json),
        AppCmd::Status { json } => status(json),
        AppCmd::Show { id, json } => show(&id, json),
        AppCmd::Adopt => adopt(),
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
        for setup in &app.setups {
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
            .and_then(|id| app.setup_info(id))
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
        .filter(|a| a.picked && a.state == AppState::Missing)
        .count();
    if missing == 1 {
        out.push_str("\n1 picked app missing: kubaos update installs it.\n");
    } else if missing > 1 {
        let _ = writeln!(out, "\n{missing} picked apps missing: kubaos update installs them.");
    }
    out
}

/// One app as `kubaos app show` has it: what it is, and each step its journals record.
#[derive(Serialize)]
struct Showing<'a> {
    #[serde(flatten)]
    about: About<'a>,
    /// The setup it is installed in.
    #[serde(skip_serializing_if = "Option::is_none")]
    setup: Option<String>,
    complete: bool,
    /// Every step of both journals, by step.
    steps: Vec<Owned>,
}

#[derive(Serialize)]
struct About<'a> {
    id: &'a str,
    name: &'a str,
    category: &'static str,
    tags: &'a [String],
    summary: &'a str,
    description: &'a str,
    description_from: &'a str,
}

/// One step of an app's journals.
#[derive(Serialize)]
struct Owned {
    step: usize,
    name: String,
    /// `user` or `root`.
    side: &'static str,
    /// `done`, `found`, `skipped`, or `doing`/`undoing` for a run stopped during it.
    state: String,
    /// What it changed, which a removal undoes.
    owns: Vec<String>,
    /// What it found there, which a removal leaves.
    found: Vec<String>,
}

/// Reads `id`'s folder and both its journals.
fn shown<'a>(folder: &'a Folder, paths: &Paths) -> Result<Showing<'a>, JournalError> {
    let user = Journal::read(&paths.app_journals(), &folder.id)?;
    let root = Journal::read(&SystemPaths::machine().app_journals(), &folder.id)?;
    let mut steps: Vec<Owned> = [("user", &user), ("root", &root)]
        .into_iter()
        .flat_map(|(side, journal)| {
            journal.iter().flat_map(move |j| {
                j.entries.iter().map(move |e| Owned {
                    step: e.step,
                    name: e.action.name.clone(),
                    side,
                    state: serde_json::to_value(e.state)
                        .ok()
                        .and_then(|v| v.as_str().map(str::to_owned))
                        .unwrap_or_default(),
                    owns: e.claims.iter().filter(|c| c.owned).map(|c| c.claim.clone()).collect(),
                    found: e.claims.iter().filter(|c| !c.owned).map(|c| c.claim.clone()).collect(),
                })
            })
        })
        .collect();
    steps.sort_by_key(|s| s.step);
    Ok(Showing {
        about: About {
            id: &folder.id,
            name: &folder.name,
            category: folder.category.word(),
            tags: &folder.tags,
            summary: &folder.summary,
            description: &folder.description,
            description_from: &folder.description_from,
        },
        setup: user.as_ref().and_then(|j| j.setup.clone()),
        complete: user.is_some_and(|j| j.complete),
        steps,
    })
}

#[expect(
    clippy::print_stdout,
    reason = "What the app owns is the command's output, written whole as it ends its own lines."
)]
fn show(id: &str, json: bool) -> ExitCode {
    let paths = match Paths::from_env() {
        Ok(paths) => paths,
        Err(e) => return usage_error(e),
    };
    let folder = match Folder::load(&SystemPaths::machine().catalog(), id) {
        Ok(folder) => folder,
        Err(FolderError::Read { .. }) => {
            return usage_error(format!("the Apps catalog has no app {id} (kubaos app list)"));
        }
        Err(e) => return job::report(e),
    };
    let shown = match shown(&folder, &paths) {
        Ok(shown) => shown,
        Err(e) => return job::report(e),
    };
    if json {
        out!("{}", serde_json::to_string(&shown).expect("an app always serializes"));
        return ExitCode::SUCCESS;
    }
    print!("{}", show_text(&shown));
    ExitCode::SUCCESS
}

/// An app as `kubaos app show` prints it.
fn show_text(shown: &Showing) -> String {
    let about = &shown.about;
    let mut out = format!(
        "{} ({}): {}
{}
  from {}

",
        about.name, about.category, about.summary, about.description, about.description_from
    );
    if shown.steps.is_empty() {
        out.push_str(
            "Not installed: it owns nothing on this machine.
",
        );
        return out;
    }
    let state = if shown.complete {
        "Installed"
    } else {
        "Partly installed"
    };
    match &shown.setup {
        Some(setup) => {
            let _ = writeln!(out, "{state} ({setup}). What each step owns, which a removal undoes:");
        }
        None => {
            let _ = writeln!(out, "{state}. What each step owns, which a removal undoes:");
        }
    }
    for step in &shown.steps {
        let _ = writeln!(out, "  {}. {} ({}, {})", step.step, step.name, step.side, step.state);
        for claim in &step.owns {
            let _ = writeln!(out, "       owns   {claim}");
        }
        for claim in &step.found {
            let _ = writeln!(out, "       found  {claim}");
        }
    }
    out
}

#[expect(clippy::print_stdout, reason = "The apps taken in are the command's output.")]
fn adopt() -> ExitCode {
    let (paths, mut system) = match update::machine(SwitchIn::Unit) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let mut updater = Updater::new(&mut system, SystemPaths::machine(), paths, SystemTime::now());
    match updater.adopt_apps() {
        Ok(taken) => {
            for name in taken {
                println!("took in {name}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => job::report(e),
    }
}

#[cfg(test)]
mod tests;
