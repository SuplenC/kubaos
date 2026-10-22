//! `kubaos`: the KubaOS CLI, which does the work of changing a KubaOS machine. The GUIs are
//! front ends to it. See docs/INSTALLER.md, "CLI".
//!
//! Exit codes: 0 ok, 1 the work failed (the last event is `failed`), 2 a usage or config error.
//!
//! # Owns
//!
//! - The command line: parsing it and handing each command to its module.
//! - What every command shares: the `--events` format and how a usage error is reported.
//!
//! # Not here
//!
//! - The install commands (`probe`, `validate`, `plan`, `run`, `demo`): `install.rs`.
//! - First-boot setup (`setup`): `setup.rs`.
//! - The work itself: the `kubaos-core` crate.

#![forbid(unsafe_code)]

use std::fmt::Display;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;

mod install;
mod setup;

#[derive(Parser)]
#[command(
    name = "kubaos",
    version,
    about = "Installs and changes a KubaOS machine"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    #[command(flatten)]
    Install(install::InstallCmd),
    /// First-boot setup on the installed system, as the desktop user.
    Setup {
        #[command(subcommand)]
        cmd: setup::SetupCmd,
    },
}

/// How a command prints its events: JSON lines for a front end to parse, or text for a human.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum Events {
    Json,
    Human,
}

/// Prints `kubaos: <msg>` on stderr and returns exit code 2.
///
/// The GUI shows the last stderr line of an exit 2 as the reason, without the `kubaos: ` prefix.
pub(crate) fn usage_error(msg: impl Display) -> ExitCode {
    eprintln!("kubaos: {msg}");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    match Cli::parse().cmd {
        Cmd::Install(cmd) => install::main(cmd),
        Cmd::Setup { cmd } => setup::main(cmd),
    }
}
