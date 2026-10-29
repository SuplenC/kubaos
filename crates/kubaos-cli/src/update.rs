//! `kubaos update`, `upgrade`, `rollback` and `channel`: updating the installed KubaOS.
//!
//!   kubaos update [-y] [--packages-only] [--foreground] [--json]
//!   kubaos update --check [--json]
//!   kubaos upgrade [--foreground] [--json]
//!   kubaos rollback [-y] [--foreground] [--json]
//!   kubaos channel [stable | edge]
//!
//! They run as the desktop user and ask for sudo for the steps that need root. Once confirmed, an
//! update, upgrade or rollback switches the home back to the release first when it is a live
//! preview (`kubaos switch --from`). `--foreground`
//! runs the home-manager switch in this process, for a session without a systemd user manager.
//! `--check` is the daily notifier's: it only looks at origin. On a terminal they show a live
//! view of each phase; elsewhere plain lines; with `--json` one event per line
//! (`kubaos_core::update::event`), reading a prompt's answer from stdin. Programs' output goes to
//! the run's log, `~/.local/state/kubaos/update/`. Exit codes: 0 ok, 1 the update failed or was
//! declined, 2 usage.
//!
//! # Owns
//!
//! - The command line of these commands, and the real machine the updater runs on.
//! - Showing the updater's events: `update/render.rs`.
//!
//! # Not here
//!
//! - The steps and the events: `kubaos_core::update`.

use std::process::ExitCode;
use std::time::SystemTime;

use clap::Args;
use kubaos_core::state::Paths;
use kubaos_core::update::Confirm;
use kubaos_core::update::UpdateError;
use kubaos_core::update::UpdateOptions;
use kubaos_core::update::Updater;
use kubaos_core::update::record::Channel;
use kubaos_core::update::record::SystemPaths;

use crate::job;
use crate::job::SwitchIn;
use crate::usage_error;

pub(crate) mod render;
mod system;

use self::render::Mode;
use self::render::Ui;
use self::system::System;
use crate::out::out;

#[derive(Args)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "The bools are the command's flags, as clap parses them."
)]
pub struct UpdateArgs {
    /// Don't ask before updating.
    #[arg(short, long)]
    yes: bool,
    /// Update apt and Nix packages only: leave the KubaOS release where it is.
    #[arg(long, conflicts_with = "check")]
    packages_only: bool,
    /// Run the home-manager switch in this process rather than in a systemd user unit.
    #[arg(long)]
    foreground: bool,
    /// Only look for a new release: write ~/.local/state/kubaos/update-available and notify.
    #[arg(long)]
    check: bool,
    /// Write one JSON event per line on stdout, for a graphical front end.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
pub struct UpgradeArgs {
    /// Run the home-manager switch in this process rather than in a systemd user unit.
    #[arg(long)]
    foreground: bool,
    /// Write one JSON event per line on stdout, for a graphical front end.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
pub struct RollbackArgs {
    /// Don't ask before rolling back.
    #[arg(short, long)]
    yes: bool,
    /// Run the home-manager switch in this process rather than in a systemd user unit.
    #[arg(long)]
    foreground: bool,
    /// Write one JSON event per line on stdout, for a graphical front end.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
pub struct ChannelArgs {
    /// `stable` or `edge`; without it, prints the channel.
    channel: Option<String>,
}

fn place(foreground: bool) -> SwitchIn {
    if foreground { SwitchIn::Process } else { SwitchIn::Unit }
}

fn confirm(yes: bool) -> Confirm {
    if yes { Confirm::AssumeYes } else { Confirm::Ask }
}

/// The user's paths and the machine, or the exit code of why there are none.
pub(crate) fn machine(place: SwitchIn) -> Result<(Paths, System), ExitCode> {
    if rustix::process::geteuid().is_root() {
        return Err(usage_error(UpdateError::Root));
    }
    let paths = Paths::from_env().map_err(usage_error)?;
    let (for_switch, for_preview, for_leave) = (paths.clone(), paths.clone(), paths.clone());
    let system = System::new(move |on_output| job::switch_step(&for_switch, place, on_output)).previewing(
        move || job::active_preview(&for_preview),
        move |on_output| job::leave_preview(&for_leave, place, on_output),
    );
    Ok((paths, system))
}

/// The exit code of a run whose events have shown how it ended.
fn done<T>(result: &Result<T, UpdateError>) -> ExitCode {
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

pub fn update(args: &UpdateArgs) -> ExitCode {
    let (paths, mut system) = match machine(place(args.foreground)) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let mut ui = Ui::new(Mode::detect(args.json));
    let mut updater = Updater::new(&mut system, SystemPaths::machine(), paths, SystemTime::now()).reporting(&mut ui);
    if args.check {
        return done(&updater.check());
    }
    let opts = UpdateOptions::new().confirm(confirm(args.yes));
    done(&updater.update(if args.packages_only { opts.packages_only() } else { opts }))
}

pub fn upgrade(args: &UpgradeArgs) -> ExitCode {
    let (paths, mut system) = match machine(place(args.foreground)) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let mut ui = Ui::new(Mode::detect(args.json));
    let mut updater = Updater::new(&mut system, SystemPaths::machine(), paths, SystemTime::now()).reporting(&mut ui);
    done(&updater.upgrade())
}

pub fn rollback(args: &RollbackArgs) -> ExitCode {
    let (paths, mut system) = match machine(place(args.foreground)) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let mut ui = Ui::new(Mode::detect(args.json));
    let mut updater = Updater::new(&mut system, SystemPaths::machine(), paths, SystemTime::now()).reporting(&mut ui);
    done(&updater.rollback(confirm(args.yes)))
}

pub fn channel(args: &ChannelArgs) -> ExitCode {
    let (paths, mut system) = match machine(SwitchIn::Unit) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let mut ui = Ui::new(Mode::Plain);
    let mut updater = Updater::new(&mut system, SystemPaths::machine(), paths, SystemTime::now()).reporting(&mut ui);
    let Some(name) = &args.channel else {
        return match updater.installed() {
            Ok(installed) => {
                out!("{}", updater.channel(&installed));
                ExitCode::SUCCESS
            }
            Err(e) => job::report(e),
        };
    };
    match name.parse::<Channel>() {
        Ok(channel) => match updater.set_channel(channel) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => job::report(e),
        },
        Err(e) => usage_error(e),
    }
}
