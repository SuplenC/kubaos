//! `kubaos-files`: yazi, KubaOS's file manager, in a terminal window.
//!
//! ```text
//! kubaos-files open [PATH|URI]...   yazi on each path or file:// URI: a folder opens inside it,
//!                                   a file in its folder with the file hovered, several as tabs.
//!                                   The home folder without one. The Files desktop entry runs it.
//! kubaos-files serve                org.freedesktop.FileManager1 on the session bus, until the
//!                                   calls stop for a while. D-Bus starts it.
//! ```
//!
//! # Owns
//!
//! - The command line and its exit codes: 2 for a usage error, 1 when nothing could be opened.
//!
//! # Not here
//!
//! - Everything else: the library's modules.

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use kotodama_extension::Chain;
use kotodama_extension::log;
use kubaos_files::entry;
use kubaos_files::entry::Resolver;
use kubaos_files::service;
use kubaos_files::service::Entries;
use kubaos_files::window;

const USAGE: &str = "\
usage: kubaos-files open [PATH|URI]...
       kubaos-files serve

open   yazi on each path or file:// URI (the home folder without one)
serve  org.freedesktop.FileManager1 on the session bus; D-Bus starts it";

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let command = args.next();
    match command.as_ref().and_then(|command| command.to_str()) {
        Some("open") => open(&args.collect::<Vec<_>>()),
        Some("serve") if args.next().is_none() => serve(),
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

fn show(paths: &[PathBuf]) -> ExitCode {
    match window::open(paths) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("kubaos-files: {err}");
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
