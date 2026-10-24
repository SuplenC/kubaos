//! The KubaOS Extension's protocol loop: each config the Daemon sends makes
//! the Items over, sending only the ones that changed, and so does each
//! change the update check writes, each change to the keyboard layouts, to
//! the wallpaper sets file and to the drives file; each config also reads the
//! user folders again; each run is answered with its outcome, after the View it
//! pushes or pops. Each Live query is answered at once, with a monitor answer,
//! processes to stop, a date answer, or nothing.
//!
//! # Owns
//!
//! - The loop, logging the config's problems, and turning what a run came to into messages.
//! - Watching the update check's file, `/etc/default/keyboard`, the wallpaper sets file and the
//!   drives file.
//!
//! # Not here
//!
//! - The Items and the runs: the library's modules.

#![forbid(unsafe_code)]

use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::mpsc::Sender;
use std::time::Duration;

use jiff::Zoned;
use kotodama_extension::Connection;
use kotodama_extension::Event;
use kotodama_extension::SentItems;
use kotodama_extension::SentStatus;
use kotodama_extension::config;
use kotodama_extension::watch::RecursiveMode;
use kotodama_extension::watch::Watcher;
use kotodama_kubaos::bus::SystemBus;
use kotodama_kubaos::clock;
use kotodama_kubaos::clock::Dates;
use kotodama_kubaos::drives;
use kotodama_kubaos::drives::Drives;
use kotodama_kubaos::extension::Extension;
use kotodama_kubaos::files::UserFolders;
use kotodama_kubaos::keyboard::Keyboards;
use kotodama_kubaos::run::Programs;
use kotodama_kubaos::sysmon;
use kotodama_kubaos::sysmon::Monitor;
use kotodama_kubaos::toggles::Ran;
use kotodama_kubaos::updates;
use kotodama_kubaos::updates::Pending;
use kotodama_kubaos::wallpapers;
use kotodama_kubaos::wallpapers::WallpaperSets;
use kotodama_protocol::Clipboard;
use kotodama_protocol::Error;
use kotodama_protocol::FromExtension;
use kotodama_protocol::LiveResult;
use kotodama_protocol::Outcome;
use kotodama_protocol::ToExtension;
use kubaos_clock::settings as clock_settings;
use kubaos_core::keyboard;
use kubaos_sysmon::processes;
use kubaos_sysmon::serve;

/// How long the update check's folder stays quiet before its file is read:
/// the check writes it through a temporary file and a rename.
const SETTLE: Duration = Duration::from_millis(200);

/// What the watches send the loop.
enum Own {
    /// The update check's file changed.
    Pending(Pending),
    /// `/etc/default/keyboard` changed.
    Keyboards(Keyboards),
    /// The wallpaper sets file changed.
    WallpaperSets(WallpaperSets),
    /// The drives file changed.
    Drives(Drives),
}

/// What answers Live queries: the system monitor, then the dates.
struct Answers {
    dates: Dates,
    monitor: Monitor,
}

impl Answers {
    /// The kinds of the Items they answer with, whose Actions they run.
    const KINDS: [&str; 3] = [clock::KIND, sysmon::KIND, sysmon::PROCESS_KIND];

    fn live(&mut self, query: &str) -> Vec<LiveResult> {
        let mut results = self.monitor.live(query, serve::now_ms());
        if results.is_empty() {
            results.extend(self.dates.live(query, &Zoned::now()));
        }
        results
    }

    fn run(&mut self, kind: &str, action: &str, id: &str) -> Outcome {
        let ran = if kind == clock::KIND {
            self.dates.run(action, id, &Zoned::now(), &Programs)
        } else {
            self.monitor.run(kind, action, id, &Programs, processes::signal)
        };
        match ran {
            Ok(Some(text)) => Outcome::Copy(Clipboard::Text(text)),
            Ok(None) => Outcome::Close,
            Err(message) => Outcome::Failed { message },
        }
    }
}

fn main() -> ExitCode {
    kotodama_extension::main("kotodama-kubaos", run)
}

fn run() -> Result<(), Error> {
    let Connection {
        mut writer,
        events,
        received,
    } = kotodama_extension::connect::<Own>()?;
    let mut extension = Extension::new(SystemBus, Programs);
    let mut answers = Answers {
        dates: Dates::new(clock_settings::dir()),
        monitor: Monitor::new("/", serve::stream_file()),
    };
    let mut sent = SentItems::default();
    // The toggles work on any machine, and the actions need only their programs.
    SentStatus::default().usable(&mut writer)?;
    let state = updates::state_dir();
    if let Some(dir) = &state {
        extension.set_pending(Pending::read(&dir.join(updates::FILE)));
    }
    let keyboard_file = Path::new(keyboard::DEFAULT_KEYBOARD);
    extension.set_keyboards(read_keyboards(keyboard_file));
    let _keyboard_watcher = watch_keyboards(keyboard_file, events.clone());
    let sets_file = wallpapers::sets_file();
    if let Some(file) = &sets_file {
        extension.set_wallpaper_sets(WallpaperSets::read(&wallpapers::sets_dir(), file));
    }
    let _sets_watcher = sets_file.and_then(|file| watch_wallpaper_sets(file, events.clone()));
    let drives_file = drives::file();
    if let Some(file) = &drives_file {
        extension.set_drives(Drives::read(file));
    }
    let _drives_watcher = drives_file.and_then(|file| watch_drives(file, events.clone()));
    let _watcher = state.and_then(|dir| watch(dir, events));
    let mut configured = false;

    for event in received {
        match event {
            Event::Message(ToExtension::Config(table)) => {
                for problem in extension.configure(&table) {
                    config::warn(problem);
                }
                // The user folders are read again with each config, not watched: they change
                // rarely, and a folder made since shows up at the next reload.
                extension.set_user_folders(UserFolders::from_env());
                configured = true;
                sent.publish(&mut writer, extension.items())?;
            }
            Event::Own(own) => {
                match own {
                    Own::Pending(pending) => extension.set_pending(pending),
                    Own::Keyboards(keyboards) => extension.set_keyboards(keyboards),
                    Own::WallpaperSets(sets) => extension.set_wallpaper_sets(sets),
                    Own::Drives(drives) => extension.set_drives(drives),
                }
                if configured {
                    sent.publish(&mut writer, extension.items())?;
                }
            }
            Event::Message(ToExtension::LiveQuery { id, query, .. }) => {
                writer.send(&FromExtension::LiveResults {
                    live_query: id,
                    results: answers.live(&query),
                    done: true,
                })?;
            }
            Event::Message(ToExtension::RunAction {
                invocation,
                action,
                item,
                ..
            }) if Answers::KINDS.contains(&item.kind.name()) => {
                let outcome = answers.run(item.kind.name(), &action, &item.id);
                writer.send(&FromExtension::ActionDone { invocation, outcome })?;
            }
            Event::Message(ToExtension::RunAction {
                invocation,
                action,
                item,
                arguments,
            }) => {
                let outcome = match extension.run(&action, &item, &arguments) {
                    Ok(Ran::Close) => Outcome::Close,
                    Ok(Ran::KeepOpen) => Outcome::KeepOpen,
                    Ok(Ran::Push(view)) => {
                        writer.send(&FromExtension::PushView(view))?;
                        Outcome::KeepOpen
                    }
                    Ok(Ran::Pop(view)) => {
                        writer.send(&FromExtension::PopView { view })?;
                        Outcome::Close
                    }
                    Err(message) => Outcome::Failed { message },
                };
                writer.send(&FromExtension::ActionDone { invocation, outcome })?;
            }
            Event::Message(_) => {}
            Event::Hangup => break,
        }
    }
    Ok(())
}

/// Watches `dir` for the update check's file, sending what it says on each change. Without a
/// watch the update actions keep what the file said at start.
fn watch(dir: PathBuf, events: Sender<Event<Own>>) -> Option<Watcher> {
    // The events name the real path, behind any symlink on the way.
    let dir = dir.canonicalize().unwrap_or(dir);
    let file = dir.join(updates::FILE);
    let changed = file.clone();
    let watcher = Watcher::start(
        "watch",
        RecursiveMode::NonRecursive,
        SETTLE,
        // The check writes through a temporary file and a rename onto `file`.
        move |event, _| event.paths.contains(&changed),
        move |_| {
            let _ = events.send(Event::Own(Own::Pending(Pending::read(&file))));
        },
    )
    .map_err(|err| kotodama_extension::log!("not watching for KubaOS updates: {err}"))
    .ok()?;
    watcher.watch(vec![dir]);
    Some(watcher)
}

fn read_keyboards(file: &Path) -> Keyboards {
    Keyboards::read(file, Path::new(keyboard::RULES_LST))
}

/// Watches the folder of `file`, `/etc/default/keyboard`, sending the layouts on each change to
/// it. Without a watch the keyboard Items keep the layouts of the start.
fn watch_keyboards(file: &Path, events: Sender<Event<Own>>) -> Option<Watcher> {
    let dir = file.parent()?.to_path_buf();
    let file = file.to_path_buf();
    let changed = file.clone();
    let watcher = Watcher::start(
        "keyboard",
        RecursiveMode::NonRecursive,
        SETTLE,
        // `kubaos keyboard`'s helper writes the file through a temporary one and a rename, as the
        // update check does.
        move |event, _| event.paths.contains(&changed),
        move |_| {
            let _ = events.send(Event::Own(Own::Keyboards(read_keyboards(&file))));
        },
    )
    .map_err(|err| kotodama_extension::log!("not watching the keyboard layouts: {err}"))
    .ok()?;
    watcher.watch(vec![dir]);
    Some(watcher)
}

/// Watches the folder of the wallpaper sets `file`, sending the sets on each change to it. The
/// folder may not be there yet; it is watched once it is. Without a watch the set Items keep the
/// sets of the start.
fn watch_wallpaper_sets(file: PathBuf, events: Sender<Event<Own>>) -> Option<Watcher> {
    let dir = file.parent()?.to_path_buf();
    // The events name the real path, behind any symlink on the way.
    let dir = dir.canonicalize().unwrap_or(dir);
    let changed = dir.join(file.file_name()?);
    let watcher = Watcher::start(
        "wallpaper-sets",
        RecursiveMode::NonRecursive,
        SETTLE,
        // `dot-theme` writes the file through a temporary one and a rename.
        move |event, _| event.paths.contains(&changed),
        move |_| {
            let sets = WallpaperSets::read(&wallpapers::sets_dir(), &file);
            let _ = events.send(Event::Own(Own::WallpaperSets(sets)));
        },
    )
    .map_err(|err| kotodama_extension::log!("not watching the wallpaper sets: {err}"))
    .ok()?;
    watcher.watch(vec![dir]);
    Some(watcher)
}

/// Watches the drives `file` in `$XDG_RUNTIME_DIR`, sending the drives on each change to it.
/// Without a watch the Eject Items keep the drives of the start.
fn watch_drives(file: PathBuf, events: Sender<Event<Own>>) -> Option<Watcher> {
    let dir = file.parent()?.to_path_buf();
    let changed = file.clone();
    let watcher = Watcher::start(
        "drives",
        RecursiveMode::NonRecursive,
        SETTLE,
        // The automount service writes the file through a temporary one and a rename.
        move |event, _| event.paths.contains(&changed),
        move |_| {
            let _ = events.send(Event::Own(Own::Drives(Drives::read(&file))));
        },
    )
    .map_err(|err| kotodama_extension::log!("not watching the mounted drives: {err}"))
    .ok()?;
    watcher.watch(vec![dir]);
    Some(watcher)
}
