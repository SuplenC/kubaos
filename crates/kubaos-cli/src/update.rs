//! `kubaos update`, `upgrade`, `rollback` and `channel`: updating the installed KubaOS.
//!
//!   kubaos update [-y] [--packages-only] [--foreground]
//!   kubaos update --check
//!   kubaos upgrade [--foreground]
//!   kubaos rollback [-y] [--foreground]
//!   kubaos channel [stable | edge]
//!
//! They run as the desktop user and ask for sudo for the steps that need root. `--foreground`
//! runs the home-manager switch in this process, for a session without a systemd user manager.
//! `--check` is the daily notifier's: it only looks at origin. Exit codes: 0 ok, 1 the update
//! failed or was declined, 2 usage.
//!
//! # Owns
//!
//! - The command line of these commands, and the real machine the updater runs on.
//!
//! # Not here
//!
//! - The steps: `kubaos_core::update`.

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

mod system;

use system::System;

#[derive(Args)]
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
}

#[derive(Args)]
pub struct UpgradeArgs {
    /// Run the home-manager switch in this process rather than in a systemd user unit.
    #[arg(long)]
    foreground: bool,
}

#[derive(Args)]
pub struct RollbackArgs {
    /// Don't ask before rolling back.
    #[arg(short, long)]
    yes: bool,
    /// Run the home-manager switch in this process rather than in a systemd user unit.
    #[arg(long)]
    foreground: bool,
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
fn machine(place: SwitchIn) -> Result<(Paths, System), ExitCode> {
    if rustix::process::geteuid().is_root() {
        return Err(usage_error(UpdateError::Root));
    }
    let paths = Paths::from_env().map_err(usage_error)?;
    let for_switch = paths.clone();
    let system = System::new(move || job::switch_step(&for_switch, place));
    Ok((paths, system))
}

fn done(result: Result<(), UpdateError>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => job::report(e),
    }
}

pub fn update(args: &UpdateArgs) -> ExitCode {
    let (paths, mut system) = match machine(place(args.foreground)) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let mut updater = Updater::new(&mut system, SystemPaths::machine(), paths, SystemTime::now());
    if args.check {
        return done(updater.check().map(drop));
    }
    let opts = UpdateOptions::new().confirm(confirm(args.yes));
    done(updater.update(if args.packages_only { opts.packages_only() } else { opts }))
}

pub fn upgrade(args: &UpgradeArgs) -> ExitCode {
    let (paths, mut system) = match machine(place(args.foreground)) {
        Ok(m) => m,
        Err(code) => return code,
    };
    done(Updater::new(&mut system, SystemPaths::machine(), paths, SystemTime::now()).upgrade())
}

pub fn rollback(args: &RollbackArgs) -> ExitCode {
    let (paths, mut system) = match machine(place(args.foreground)) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let mut updater = Updater::new(&mut system, SystemPaths::machine(), paths, SystemTime::now());
    done(updater.rollback(confirm(args.yes)))
}

pub fn channel(args: &ChannelArgs) -> ExitCode {
    let (paths, mut system) = match machine(SwitchIn::Unit) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let mut updater = Updater::new(&mut system, SystemPaths::machine(), paths, SystemTime::now());
    let Some(name) = &args.channel else {
        return match updater.installed() {
            Ok(installed) => {
                println!("{}", updater.channel(&installed));
                ExitCode::SUCCESS
            }
            Err(e) => job::report(e),
        };
    };
    match name.parse::<Channel>() {
        Ok(channel) => done(updater.set_channel(channel)),
        Err(e) => usage_error(e),
    }
}
