//! `kubaos autostart`: the startup apps, which the session starts at login.
//!
//!   kubaos autostart list        the apps that start on login, a line each: the desktop id, a
//!                                tab, the name; sorted by name
//!   kubaos autostart on <app>    start it on login
//!   kubaos autostart off <app>   don't start it on login
//!
//! `<app>` is the app's desktop id, `solaar.desktop`, or the id without `.desktop`. A change is
//! an entry in `~/.config/autostart` (`kubaos_core::autostart` says which), then
//! `systemctl --user daemon-reload`, which writes the units of the entries again: a user manager
//! that outlives the session, as one with lingering on does, would start the old set at the next
//! login otherwise. Nothing starts or stops now; the change is for the next login. The Command
//! menu's "Start on login" and "Don't start on login" run `on` and `off`.
//! Exit codes: 0 ok, 1 writing the entry failed, 2 usage or no such app.
//!
//! # Owns
//!
//! - The command line, and telling systemd.
//!
//! # Not here
//!
//! - Which entries start, and the entries written: `kubaos_core::autostart`.
//! - Starting them at login: `config/hypr/autostart.lua`.

use std::process::Command;
use std::process::ExitCode;
use std::process::Stdio;

use clap::Subcommand;
use kubaos_core::autostart::AutostartError;
use kubaos_core::autostart::Dirs;

use crate::job::chain;
use crate::out::err;
use crate::out::out;
use crate::usage_error;

#[derive(Subcommand)]
pub enum AutostartCmd {
    /// Print the apps that start on login: the desktop id, a tab, the name.
    List,
    /// Start an app on login, by its desktop id (`solaar.desktop`, or `solaar`).
    On { app: String },
    /// Don't start an app on login, by its desktop id (`solaar.desktop`, or `solaar`).
    Off { app: String },
}

pub fn main(cmd: AutostartCmd) -> ExitCode {
    let Some(dirs) = Dirs::from_env() else {
        return usage_error("HOME isn't set");
    };
    match cmd {
        AutostartCmd::List => {
            for app in dirs.startup_apps() {
                out!("{}\t{}", app.id, app.name);
            }
            ExitCode::SUCCESS
        }
        AutostartCmd::On { app } => change(dirs.turn_on(&desktop_id(&app))),
        AutostartCmd::Off { app } => change(dirs.turn_off(&desktop_id(&app))),
    }
}

/// `app` as a desktop id: with `.desktop` added when it hasn't one.
fn desktop_id(app: &str) -> String {
    if app.ends_with(".desktop") {
        app.to_owned()
    } else {
        format!("{app}.desktop")
    }
}

fn change(result: Result<(), AutostartError>) -> ExitCode {
    match result {
        Ok(()) => {
            reload();
            ExitCode::SUCCESS
        }
        Err(e @ (AutostartError::InvalidId(_) | AutostartError::NoApp(_))) => usage_error(e),
        Err(e) => {
            err!("kubaos: {}", chain(&e));
            ExitCode::FAILURE
        }
    }
}

/// Has systemd write the startup apps' units again. Without a user manager, as in a container,
/// the next one writes them when it starts, so a failure is only said.
fn reload() {
    let reloaded = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .stdout(Stdio::null())
        .status();
    match reloaded {
        Ok(status) if status.success() => {}
        Ok(status) => err!("kubaos: systemctl --user daemon-reload: {status}; the change shows after a reboot"),
        Err(e) => err!("kubaos: systemctl --user daemon-reload: {e}; the change shows after a reboot"),
    }
}

#[cfg(test)]
mod tests;
