//! `kubaos-files`: yazi, KubaOS's terminal file manager, in a window of its own, and the Downloads
//! tidy.
//!
//! ```text
//! kubaos-files open [PATH|URI]...   yazi on each path or file:// URI: a folder opens inside it,
//!                                   a file in its folder with the file hovered, several as tabs.
//!                                   The home folder without one. The Yazi desktop entry runs it.
//! kubaos-files serve                org.freedesktop.FileManager1 on the session bus, until the
//!                                   calls stop for a while, for an activation file that names it.
//! kubaos-files tidy [--scheduled]   sort the files in Downloads older than a day into folders by
//!                                   kind; with --scheduled, only when the daily tidy is on
//! kubaos-files tidy on|off          turn the daily tidy on or off
//! kubaos-files pick [--folder] [--title TITLE] [KIND]...
//!                                   the system's file picker, through xdg-desktop-portal: prints
//!                                   the path chosen, or nothing and exits 1 on a cancel. KIND is
//!                                   an extension (`conf`) or MIME type (`image/*`) a file may have.
//! ```
//!
//! # Owns
//!
//! - The command line and its exit codes: 2 for a usage error, 1 when nothing could be opened or
//!   picked.
//!
//! # Not here
//!
//! - Everything else: the library's modules.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;
use std::time::SystemTime;

use kotodama_extension::Chain;
use kotodama_extension::log;
use kubaos_files::entry;
use kubaos_files::entry::Resolver;
use kubaos_files::pick;
use kubaos_files::pick::Pick;
use kubaos_files::service;
use kubaos_files::service::Entries;
use kubaos_files::tidy;
use kubaos_files::tidy::Setting;
use kubaos_files::window;

const USAGE: &str = "\
usage: kubaos-files open [PATH|URI]...
       kubaos-files serve
       kubaos-files tidy [--scheduled|on|off]
       kubaos-files pick [--folder] [--title TITLE] [KIND]...

open   yazi on each path or file:// URI (the home folder without one)
serve  org.freedesktop.FileManager1 on the session bus, for D-Bus to start
tidy   sort Downloads into folders by kind, leaving files younger than a day;
       --scheduled only when the daily tidy is on; on|off turns the daily tidy on or off
pick   the system's file picker: prints the path chosen, or exits 1 on a cancel;
       --folder for a folder, KIND an extension (conf) or MIME type (image/*) a file may have";

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The help and the usage are the command's output, for whoever ran it."
)]
fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let command = args.next();
    match command.as_ref().and_then(|command| command.to_str()) {
        Some("open") => open(&args.collect::<Vec<_>>()),
        Some("serve") if args.next().is_none() => serve(),
        Some("pick") => {
            if let Some(request) = pick_request(args) {
                pick(&request)
            } else {
                eprintln!("{USAGE}");
                ExitCode::from(2)
            }
        }
        Some("tidy") => {
            let rest: Vec<OsString> = args.collect();
            match rest.iter().map(|arg| arg.to_str()).collect::<Vec<_>>().as_slice() {
                [] => tidy_downloads(false),
                [Some("--scheduled")] => tidy_downloads(true),
                [Some(setting @ ("on" | "off"))] => set_daily(*setting == "on"),
                _ => {
                    eprintln!("{USAGE}");
                    ExitCode::from(2)
                }
            }
        }
        Some("-h" | "--help") => {
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
    reason = "A path that can't be opened is the command's answer to whoever ran it."
)]
fn open(args: &[OsString]) -> ExitCode {
    let resolver = Resolver::from_env();
    if args.is_empty() {
        return show(&[resolver.home().to_path_buf()]);
    }
    let mut paths: Vec<PathBuf> = Vec::new();
    for arg in args {
        match resolver.path(arg) {
            Ok(path) => paths.push(entry::existing(&path)),
            Err(err) => eprintln!("kubaos-files: {err}"),
        }
    }
    if paths.is_empty() {
        return ExitCode::FAILURE;
    }
    show(&paths)
}

#[expect(
    clippy::print_stderr,
    reason = "The refusal is the command's answer to whoever ran it."
)]
fn show(paths: &[PathBuf]) -> ExitCode {
    match window::open(paths) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("kubaos-files: {err}");
            ExitCode::FAILURE
        }
    }
}

/// `pick`'s arguments: `--folder`, `--title TITLE` and the kinds of file. `None` for one that
/// isn't text, a title missing, or kinds given to a folder.
fn pick_request(args: impl Iterator<Item = OsString>) -> Option<Pick> {
    let mut request = Pick::default();
    let mut args = args.map(OsString::into_string);
    while let Some(arg) = args.next() {
        match arg.ok()?.as_str() {
            "--folder" => request.folder = true,
            "--title" => request.title = args.next()?.ok()?,
            kind if !kind.starts_with("--") => request.accept.push(kind.to_owned()),
            _ => return None,
        }
    }
    (!request.folder || request.accept.is_empty()).then_some(request)
}

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The path chosen, or why the picker couldn't open, is the command's answer to whoever ran it."
)]
fn pick(request: &Pick) -> ExitCode {
    match pick::pick(request, &Resolver::from_env()) {
        Ok(Some(path)) => {
            println!("{}", path.display());
            ExitCode::SUCCESS
        }
        Ok(None) => ExitCode::FAILURE,
        Err(err) => {
            eprintln!("kubaos-files: {}", Chain(&err));
            ExitCode::FAILURE
        }
    }
}

fn serve() -> ExitCode {
    let resolver = Resolver::from_env();
    let opened = service::serve(&resolver, service::DEFAULT_IDLE, |entries: Entries| {
        for refused in &entries.refused {
            log!("not shown: {refused}");
        }
        if entries.paths.is_empty() {
            return;
        }
        if let Err(err) = window::open(&entries.paths) {
            log!("{err}");
        }
    });
    match opened {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            log!("{}", Chain(&err));
            ExitCode::FAILURE
        }
    }
}

/// `HOME`, and `XDG_CONFIG_HOME` or `~/.config`.
fn home_and_config() -> (PathBuf, PathBuf) {
    let var = |name| {
        std::env::var_os(name)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    };
    let home = var("HOME").unwrap_or_else(|| PathBuf::from("/"));
    let config = var("XDG_CONFIG_HOME").unwrap_or_else(|| home.join(".config"));
    (home, config)
}

/// Tidies Downloads, and says how it went in a notification, as the menu and the timer start it
/// with no terminal. A scheduled run with nothing to move stays quiet.
#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The count moved or the refusal is the command's answer, beside its notification."
)]
fn tidy_downloads(scheduled: bool) -> ExitCode {
    let (home, config) = home_and_config();
    if scheduled && !Setting::new(&config).daily() {
        return ExitCode::SUCCESS;
    }
    let downloads = tidy::downloads_dir(&home, &config);
    match tidy::tidy(&downloads, SystemTime::now(), tidy::DEFAULT_MIN_AGE) {
        Ok(moved) => {
            println!("{moved} moved");
            let summary = match moved {
                0 if scheduled => return ExitCode::SUCCESS,
                0 => "Downloads is already tidy".to_owned(),
                1 => "Tidied 1 file in Downloads".to_owned(),
                _ => format!("Tidied {moved} files in Downloads"),
            };
            notify(
                &summary,
                "Sorted into Images, Documents, Archives, Installers, Video and Audio.",
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("kubaos-files: tidying {}: {err}", downloads.display());
            notify("Couldn't tidy Downloads", &err.to_string());
            ExitCode::FAILURE
        }
    }
}

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The setting's new state or the refusal is the command's answer to whoever ran it."
)]
fn set_daily(on: bool) -> ExitCode {
    let (_, config) = home_and_config();
    match Setting::new(config).set_daily(on) {
        Ok(()) => {
            println!("The daily Downloads tidy is {}", if on { "on" } else { "off" });
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("kubaos-files: changing the setting: {err}");
            ExitCode::FAILURE
        }
    }
}

/// A notification through `notify-send` (libnotify-bin, `system/20-base.sh`): this crate speaks
/// D-Bus through a few hand-made calls only, and a notification's hints dictionary isn't one of
/// them. Without it, the notification only goes unseen.
fn notify(summary: &str, body: &str) {
    let _ = Command::new("notify-send")
        .args(["--app-name=Files", "--icon=folder-download", summary, body])
        .status();
}
