//! `kubaos`: the KubaOS CLI, which does the work of changing a KubaOS machine. Kōbō, its
//! graphical front end, runs the same engine in-process. See docs/INSTALL-INTERNALS.md, "CLI".
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
//! - The Bitwarden vault (`bitwarden`): `bitwarden.rs`.
//! - The recoloured pointer (`cursor`): `cursor.rs`.
//! - The keyboard layouts (`keyboard`): `keyboard.rs`.
//! - Packages from nixpkgs (`package`, `try`): `package.rs`.
//! - The power stops (`power`): `power.rs`, which runs `kubaos-power`.
//! - The work network's switch (`proxy`): `proxy.rs`, which runs `kubaos-proxy`.
//! - KubaOS's apt packages (`packages`): `apt.rs`.
//! - Suites (`suite`): `suite.rs`.
//! - Project pins (`pin`, `unpin`, `pins`): `pin.rs`.
//! - Switch jobs (`switch`, `job`): `job.rs`.
//! - Updates (`update`, `upgrade`, `rollback`, `channel`): `update.rs`.
//! - The work itself: the `kubaos-core` crate.

use std::fmt::Display;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;

use crate::out::err;

mod apt;
mod bitwarden;
mod cursor;
mod install;
mod job;
mod keyboard;
mod out;
mod package;
mod pin;
mod power;
mod proxy;
mod setup;
mod suite;
mod update;

#[derive(Parser)]
#[command(name = "kubaos", version, about = "Installs and changes a KubaOS machine")]
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
    /// The Bitwarden vault: SSH keys, commit signing, gh and project env from it.
    Bitwarden {
        #[command(subcommand)]
        cmd: bitwarden::BitwardenCmd,
    },
    /// Packages from your nixpkgs, recorded in kubaos.d/ and installed by a switch.
    Package {
        #[command(subcommand)]
        cmd: package::PackageCmd,
    },
    /// KubaOS's apt packages: how they compare with the release, and the optional groups.
    Packages {
        #[command(subcommand)]
        cmd: apt::PackagesCmd,
    },
    /// Suites: a whole programming setup, recorded in kubaos.d/ and installed by a switch.
    Suite {
        #[command(subcommand)]
        cmd: suite::SuiteCmd,
    },
    /// Pin a Suite version for a folder and everything below it, in its .envrc.
    Pin(pin::PinArgs),
    /// Remove a folder's pin of a Suite.
    Unpin(pin::UnpinArgs),
    /// The Project pins in effect in a folder, or every one under the Projects root.
    Pins(pin::PinsArgs),
    /// Open a shell with a package from your nixpkgs, installing nothing.
    Try { name: String },
    /// Switch your home to the user flake, as a queued job.
    Switch {
        #[command(flatten)]
        mode: job::SwitchMode,
    },
    /// Switch jobs: list them, follow one.
    Job {
        #[command(subcommand)]
        cmd: job::JobCmd,
    },
    /// Update packages and move to the newest KubaOS release of your channel.
    Update(update::UpdateArgs),
    /// Move to the next KubaOS release series, on the stable channel.
    Upgrade(update::UpgradeArgs),
    /// Undo the last update: the KubaOS release and your flake.lock go back.
    Rollback(update::RollbackArgs),
    /// Print the channel updates follow, or set it: stable or edge.
    Channel(update::ChannelArgs),
    /// The pointer in the wallpaper's colours: its style and size.
    Cursor {
        #[command(subcommand)]
        cmd: cursor::CursorCmd,
    },
    /// The keyboard layouts, the first the default: show them, or change them.
    Keyboard {
        #[command(subcommand)]
        cmd: keyboard::KeyboardCmd,
    },
    /// The power stops, Super Saver to Max Performance: set one, see what it does, measure it.
    #[command(disable_help_flag = true)]
    Power {
        /// `set STOP`, `status`, `measure [--seconds N]`, `undo`, `charge-limit 80|90|100`,
        /// `scheduler [on|off|status]`, …
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// The Proxy, one switch for the work network: on, off, status, or import IT's WireGuard
    /// config.
    #[command(disable_help_flag = true)]
    Proxy {
        /// `on`, `off`, `status [--json] [--live]`, `import FILE`, `pull`
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

/// How a command prints its events: JSON lines for a front end to parse, or text for a human.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum Events {
    Json,
    Human,
}

/// Prints `kubaos: <msg>` on stderr and returns exit code 2.
pub(crate) fn usage_error(msg: impl Display) -> ExitCode {
    err!("kubaos: {msg}");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    match Cli::parse().cmd {
        Cmd::Install(cmd) => install::main(cmd),
        Cmd::Setup { cmd } => setup::main(&cmd),
        Cmd::Bitwarden { cmd } => bitwarden::main(cmd),
        Cmd::Package { cmd } => package::main(cmd),
        Cmd::Packages { cmd } => apt::main(cmd),
        Cmd::Suite { cmd } => suite::main(cmd),
        Cmd::Pin(args) => pin::pin(&args),
        Cmd::Unpin(args) => pin::unpin(&args),
        Cmd::Pins(args) => pin::list(&args),
        Cmd::Try { name } => package::try_main(&name),
        Cmd::Switch { mode } => match kubaos_core::state::Paths::from_env() {
            Ok(paths) => job::switch(&paths, mode),
            Err(e) => usage_error(e),
        },
        Cmd::Job { cmd } => job::main(cmd),
        Cmd::Update(args) => update::update(&args),
        Cmd::Upgrade(args) => update::upgrade(&args),
        Cmd::Rollback(args) => update::rollback(&args),
        Cmd::Channel(args) => update::channel(&args),
        Cmd::Cursor { cmd } => cursor::main(cmd),
        Cmd::Keyboard { cmd } => keyboard::main(cmd),
        Cmd::Power { args } => power::main(&args),
        Cmd::Proxy { args } => proxy::main(&args),
    }
}
