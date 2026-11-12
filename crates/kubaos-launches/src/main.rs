//! `kubaos-launches`: the launch watcher, and the commands that turn it on and off.
//!
//! ```text
//! kubaos-launches watch    the service, until the bus goes (the user unit runs it)
//! kubaos-launches on|off   open apps on the workspace they were launched from, or where focus is
//! ```
//!
//! # Owns
//!
//! - The command line and its exit codes: 2 for a usage error, 1 when the command failed, with the
//!   reason on stderr.
//!
//! # Not here
//!
//! - Everything else: the library's modules.

use std::error::Error;
use std::process::ExitCode;

use kubaos_launches::hyprland::Hyprland;
use kubaos_launches::service;
use kubaos_launches::settings::Settings;

const USAGE: &str = "\
usage: kubaos-launches watch
       kubaos-launches on|off

watch   tell Hyprland of each app's launch (the kubaos-launches user unit runs it)
on|off  open apps on the workspace they were launched from, or on the one focused when they show";

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The help and the usage are the command's output, for whoever ran it."
)]
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["watch"] => watch(),
        [setting @ ("on" | "off")] => set(*setting == "on"),
        ["-h" | "--help"] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

#[expect(
    clippy::print_stderr,
    reason = "The service's errors go to stderr, which the user unit's journal keeps."
)]
fn watch() -> ExitCode {
    let Some(settings) = Settings::from_env() else {
        eprintln!("kubaos-launches: neither XDG_CONFIG_HOME nor HOME is set");
        return ExitCode::FAILURE;
    };
    let Some(hyprland) = Hyprland::from_env() else {
        eprintln!("kubaos-launches: XDG_RUNTIME_DIR or HYPRLAND_INSTANCE_SIGNATURE isn't set");
        return ExitCode::FAILURE;
    };
    let failed = |err: &service::TellError| eprintln!("kubaos-launches: {}", chain(err));
    match service::run(&settings, &hyprland, failed) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("kubaos-launches: {}", chain(&err));
            ExitCode::FAILURE
        }
    }
}

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The setting's new state or the refusal is the command's answer to whoever ran it."
)]
fn set(on: bool) -> ExitCode {
    let Some(settings) = Settings::from_env() else {
        eprintln!("kubaos-launches: neither XDG_CONFIG_HOME nor HOME is set");
        return ExitCode::FAILURE;
    };
    match settings.set_open_where_launched(on) {
        Ok(()) if on => {
            println!("Apps open on the workspace they were launched from");
            ExitCode::SUCCESS
        }
        Ok(()) => {
            println!("Apps open on the workspace focused when they show");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("kubaos-launches: changing the setting: {err}");
            ExitCode::FAILURE
        }
    }
}

/// `err` and each of its causes, joined by `: `.
fn chain(err: &dyn Error) -> String {
    let mut text = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        text.push_str(": ");
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}
