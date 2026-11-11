//! `kubaos update`, `upgrade`, `rollback` and `channel`: updating the installed KubaOS.
//!
//!   kubaos update [-y] [--packages-only] [--foreground] [--json]
//!   kubaos update --check [--json]
//!   kubaos update --turn-on | --turn-off bitwarden | nix | firmware
//!   kubaos upgrade [--foreground] [--json]
//!   kubaos rollback [-y] [--foreground] [--json]
//!   kubaos channel [stable | edge]
//!
//! They run as the desktop user and ask for sudo for the steps that need root. Once confirmed, an
//! update, upgrade or rollback switches the home back to the release first when it is a live
//! preview (`kubaos switch --from`). `--foreground`
//! runs the home-manager switch in this process, for a session without a systemd user manager.
//! `--check` is the daily notifier's: it only looks at origin. `--turn-on` and `--turn-off` set
//! whether an update moves Bitwarden desktop or Nix on within its series, or updates firmware, and
//! update nothing. An
//! update offers to stage the firmware that installs at the next reboot only when it asks, so never
//! with `-y` or `--json`. On a terminal they show a live view of each phase; elsewhere plain lines;
//! with `--json` one event per line (`kubaos_core::update::event`), reading a prompt's answer from
//! stdin. Programs' output goes to the run's log, `~/.local/state/kubaos/update/`. An update,
//! upgrade or rollback that nobody reads in a terminal and that isn't `--json` also sends its
//! summary as a notification. Exit codes: 0 ok, 1 the update failed or was declined, 2 usage.
//!
//! # Owns
//!
//! - The command line of these commands, and the real machine the updater runs on.
//! - Showing the updater's events: `update/render.rs`; when the summary is also a notification.
//!
//! # Not here
//!
//! - The steps and the events: `kubaos_core::update`.

use std::io;
use std::io::IsTerminal;
use std::process::Command;
use std::process::ExitCode;
use std::process::Stdio;
use std::time::SystemTime;

use clap::Args;
use clap::ValueEnum;
use kubaos_core::state::Paths;
use kubaos_core::update::Confirm;
use kubaos_core::update::UpdateError;
use kubaos_core::update::UpdateOptions;
use kubaos_core::update::Updater;
use kubaos_core::update::event::Sink;
use kubaos_core::update::event::UpdateEvent;
use kubaos_core::update::programs::Part;
use kubaos_core::update::programs::Switches;
use kubaos_core::update::record::Channel;
use kubaos_core::update::record::SystemPaths;
use kubaos_core::update::summary::Notification;

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
    /// Turn this part of every update back on; updates nothing now.
    #[arg(long, value_enum, value_name = "PART", conflicts_with_all = ["check", "packages_only", "turn_off"])]
    turn_on: Option<ProgramArg>,
    /// Leave this part out of every update; updates nothing now.
    #[arg(long, value_enum, value_name = "PART", conflicts_with_all = ["check", "packages_only"])]
    turn_off: Option<ProgramArg>,
}

/// A part of the update the user can turn off.
#[derive(Clone, Copy, ValueEnum)]
enum ProgramArg {
    /// Bitwarden desktop, to the newest release of its year.
    Bitwarden,
    /// The Nix daemon, to the newest patch release of its minor.
    Nix,
    /// Firmware, through fwupd and the LVFS.
    Firmware,
}

impl ProgramArg {
    fn part(self) -> Part {
        match self {
            Self::Bitwarden => Part::Bitwarden,
            Self::Nix => Part::Nix,
            Self::Firmware => Part::Firmware,
        }
    }
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
    if let Some(program) = args.turn_on {
        return turn(program.part(), true);
    }
    if let Some(program) = args.turn_off {
        return turn(program.part(), false);
    }
    let (paths, mut system) = match machine(place(args.foreground)) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let mut ui = Shown::new(args.json);
    let mut updater = Updater::new(&mut system, SystemPaths::machine(), paths, SystemTime::now()).reporting(&mut ui);
    if args.check {
        return done(&updater.check());
    }
    let mut opts = UpdateOptions::new().confirm(confirm(args.yes));
    if !args.json {
        opts = opts.stage_firmware();
    }
    done(&updater.update(if args.packages_only { opts.packages_only() } else { opts }))
}

pub fn upgrade(args: &UpgradeArgs) -> ExitCode {
    let (paths, mut system) = match machine(place(args.foreground)) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let mut ui = Shown::new(args.json);
    let mut updater = Updater::new(&mut system, SystemPaths::machine(), paths, SystemTime::now()).reporting(&mut ui);
    done(&updater.upgrade())
}

pub fn rollback(args: &RollbackArgs) -> ExitCode {
    let (paths, mut system) = match machine(place(args.foreground)) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let mut ui = Shown::new(args.json);
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

/// The events shown in the [`Mode`] the run picks, and the summary sent as a notification when
/// [`notifies`] says nobody else shows it.
struct Shown {
    ui: Ui,
    notify: bool,
}

impl Shown {
    fn new(json: bool) -> Self {
        Self {
            ui: Ui::new(Mode::detect(json)),
            notify: notifies(json, io::stdout().is_terminal(), io::stderr().is_terminal()),
        }
    }
}

impl Sink for Shown {
    fn event(&mut self, event: &UpdateEvent) {
        self.ui.event(event);
        if self.notify
            && let UpdateEvent::Summary(summary) = event
        {
            send(&summary.notification());
        }
    }
}

/// Whether a run's summary goes to a notification: only when no one reads it where it ran. A
/// terminal on stdout or stderr shows it (the Command menu's and the bar's terminals stay open
/// after the run), and with `--json` the front end shows the `summary` event itself. What is
/// left is a run in a systemd unit, a timer or a script with its output in a file.
fn notifies(json: bool, stdout_terminal: bool, stderr_terminal: bool) -> bool {
    !json && !stdout_terminal && !stderr_terminal
}

/// Sends `notification` through `notify-send`. A desktop without a notification daemon loses it,
/// never the update: the summary is in the log too.
fn send(notification: &Notification) {
    let urgency = if notification.urgent {
        "--urgency=normal"
    } else {
        "--urgency=low"
    };
    let mut cmd = Command::new("notify-send");
    cmd.args(["--app-name=KubaOS", urgency, &notification.title]);
    if !notification.body.is_empty() {
        cmd.arg(&notification.body);
    }
    let _ = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

/// Turns `part` of the update on or off in the user layer.
fn turn(part: Part, on: bool) -> ExitCode {
    let paths = match Paths::from_env() {
        Ok(paths) => paths,
        Err(e) => return usage_error(e),
    };
    let switches = Switches::new(paths.config_home());
    let turned = if on {
        switches.turn_on(part)
    } else {
        switches.turn_off(part)
    };
    if let Err(e) = turned {
        return job::report(e);
    }
    let text = match (part, on) {
        (Part::Bitwarden, true) => {
            "Bitwarden updates on: kubaos update moves Bitwarden desktop to the newest release of its series"
        }
        (Part::Bitwarden, false) => "Bitwarden updates off: kubaos update leaves Bitwarden desktop at its version",
        (Part::Nix, true) => "Nix updates on: kubaos update moves Nix to the newest patch release of its minor",
        (Part::Nix, false) => "Nix updates off: kubaos update leaves Nix at its version",
        (Part::Firmware, true) => "Firmware updates on: kubaos update installs firmware from the LVFS through fwupd",
        (Part::Firmware, false) => "Firmware updates off: kubaos update leaves firmware alone",
    };
    out!("{text}");
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests;
