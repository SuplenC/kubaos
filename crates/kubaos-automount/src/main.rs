//! `kubaos-automount`: the automount service, and the commands the menu and its notifications
//! run.
//!
//! ```text
//! kubaos-automount watch          the service, until a bus goes (the user unit runs it)
//! kubaos-automount eject TARGET   unmount and power off the drive TARGET names: its udisks2 object
//!                                 path, a device file or a mount point
//! kubaos-automount import FOLDER  copy the photos and videos of FOLDER (or of its DCIM folder)
//!                                 into ~/Pictures/<year>/<month>/
//! kubaos-automount on|off         mount drives as they're plugged in, or don't
//! ```
//!
//! # Owns
//!
//! - The command line and its exit codes: 2 for a usage error, 1 when the command failed, with the
//!   reason on stderr. `eject` and `import` also say how it went in a notification, since the menu
//!   and the notifications start them with no terminal.
//!
//! # Not here
//!
//! - Everything else: the library's modules.

use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use kotodama_extension::Chain;
use kubaos_automount::import;
use kubaos_automount::import::Report;
use kubaos_automount::notify::Note;
use kubaos_automount::notify::Notifier;
use kubaos_automount::service;
use kubaos_automount::settings::Settings;
use kubaos_automount::udisks::UDisks;

const USAGE: &str = "\
usage: kubaos-automount watch
       kubaos-automount eject TARGET
       kubaos-automount import FOLDER
       kubaos-automount on|off

watch   mount removable drives as they're plugged in (the kubaos-automount user unit runs it)
eject   unmount and power off the drive TARGET names: a udisks2 drive, a device or a mount point
import  copy the photos and videos of FOLDER, or of its DCIM folder, into ~/Pictures by month
on|off  turn mounting drives as they're plugged in on or off";

const PHOTOS_ICON: &str = "camera-photo";

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
        ["eject", target] => async_io::block_on(eject(target)),
        ["import", folder] => async_io::block_on(import(Path::new(folder))),
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
    reason = "The error the watch exits with goes to stderr, which the user unit's journal keeps."
)]
fn watch() -> ExitCode {
    match async_io::block_on(service::run()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("kubaos-automount: {}", Chain(&err));
            ExitCode::FAILURE
        }
    }
}

#[expect(
    clippy::print_stderr,
    reason = "The refusal is the command's answer to whoever ran it, beside its notification."
)]
async fn eject(target: &str) -> ExitCode {
    let system = match zbus::Connection::system().await {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("kubaos-automount: connecting to the system bus: {err}");
            return ExitCode::FAILURE;
        }
    };
    let result = service::eject(&UDisks::new(system), target).await;
    let failed = result.as_ref().err().map(|err| Chain(err).to_string());
    tell(&service::eject_note(result)).await;
    match failed {
        None => ExitCode::SUCCESS,
        Some(reason) => {
            eprintln!("kubaos-automount: {reason}");
            ExitCode::FAILURE
        }
    }
}

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The counts or the refusal are the command's answer, beside its notification."
)]
async fn import(folder: &Path) -> ExitCode {
    let source = import::dcim(folder).unwrap_or_else(|| folder.to_path_buf());
    let pictures = pictures();
    match import::import(&source, &pictures) {
        Ok(report) => {
            println!(
                "{} copied, {} already there, {} failed",
                report.copied, report.already, report.failed
            );
            tell(&import_note(report, &pictures)).await;
            if report.failed == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(err) => {
            let reason = Chain(&err).to_string();
            eprintln!("kubaos-automount: {reason}");
            tell(&Note::new(PHOTOS_ICON, "Couldn't import the photos", reason)).await;
            ExitCode::FAILURE
        }
    }
}

fn import_note(report: Report, pictures: &Path) -> Note {
    let plural = |count: usize, one: &str, many: &str| {
        if count == 1 {
            format!("1 {one}")
        } else {
            format!("{count} {many}")
        }
    };
    let summary = if report.copied == 0 {
        "No new photos".to_owned()
    } else {
        format!("Imported {}", plural(report.copied, "photo", "photos"))
    };
    let mut body = vec![format!("Into {}, by month.", pictures.display())];
    if report.already > 0 {
        body.push(format!("{} already there.", plural(report.already, "was", "were")));
    }
    if report.failed > 0 {
        body.push(format!("{} could not be copied.", report.failed));
    }
    let body = body.join(" ");
    Note::new(PHOTOS_ICON, summary, body)
}

/// The pictures folder from `HOME` and `XDG_CONFIG_HOME`.
fn pictures() -> PathBuf {
    let var = |name| {
        std::env::var_os(name)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    };
    let home = var("HOME").unwrap_or_else(|| PathBuf::from("/"));
    let config = var("XDG_CONFIG_HOME").unwrap_or_else(|| home.join(".config"));
    import::pictures_dir(&home, &config)
}

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The setting's new state or the refusal is the command's answer to whoever ran it."
)]
fn set(on: bool) -> ExitCode {
    let Some(settings) = Settings::from_env() else {
        eprintln!("kubaos-automount: neither XDG_CONFIG_HOME nor HOME is set");
        return ExitCode::FAILURE;
    };
    match settings.set_automount(on) {
        Ok(()) => {
            println!("Automount is {}", if on { "on" } else { "off" });
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("kubaos-automount: changing the setting: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Shows `note`; with no session bus or notification server it only goes unseen.
async fn tell(note: &Note) {
    let Ok(session) = zbus::Connection::session().await else {
        return;
    };
    let _ = Notifier::new(session).send(note).await;
}
