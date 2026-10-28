//! The KubaOS Extension's protocol loop, and with `layouts` the "Add a keyboard layout" app's: each
//! config the Daemon sends makes the Items over, sending only the ones that changed, and so does
//! each change the update check writes, each change to the keyboard layouts, to the wallpaper sets
//! file, to Hyprland's screens file, to the drives file and to a file a state of the actions is
//! read from; each config also reads the user folders and the states again; each run is answered
//! with its outcome, after the View it pushes or pops. Each Live query is answered at once, with a
//! monitor answer, processes to stop, a date answer, or nothing.
//!
//! # Owns
//!
//! - The loop, logging the config's problems, and turning what a run came to into messages.
//! - Watching the update check's file, the keyboard settings and `/etc/default/keyboard`, the
//!   wallpaper sets file, Hyprland's screens file, the drives file and the states' files.
//!
//! # Not here
//!
//! - The Items and the runs: the library's modules.

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
use kotodama_kubaos::desk;
use kotodama_kubaos::desk::Desk;
use kotodama_kubaos::drives;
use kotodama_kubaos::drives::Drives;
use kotodama_kubaos::extension::Extension;
use kotodama_kubaos::files::UserFolders;
use kotodama_kubaos::keyboard::Keyboards;
use kotodama_kubaos::layouts::LayoutsApp;
use kotodama_kubaos::run::Programs;
use kotodama_kubaos::states::Places;
use kotodama_kubaos::states::States;
use kotodama_kubaos::sysmon;
use kotodama_kubaos::sysmon::Monitor;
use kotodama_kubaos::toggles::Ran;
use kotodama_kubaos::updates;
use kotodama_kubaos::updates::Pending;
use kotodama_kubaos::wallpapers;
use kotodama_kubaos::wallpapers::WallpaperSets;
use kotodama_kubaos::watched;
use kotodama_protocol::Clipboard;
use kotodama_protocol::Error;
use kotodama_protocol::FromExtension;
use kotodama_protocol::LiveResult;
use kotodama_protocol::Outcome;
use kotodama_protocol::ToExtension;
use kubaos_clock::settings as clock_settings;
use kubaos_core::keyboard;
use kubaos_core::keyboard::settings as keyboard_settings;
use kubaos_sysmon::processes;
use kubaos_sysmon::serve;

/// How long the update check's folder stays quiet before its file is read:
/// the check writes it through a temporary file and a rename.
const SETTLE: Duration = Duration::from_millis(200);

/// What the watches send the loop.
enum Own {
    /// The update check's file changed.
    Pending(Pending),
    /// The keyboard settings or `/etc/default/keyboard` changed.
    Keyboards(Keyboards),
    /// The wallpaper sets file changed.
    WallpaperSets(WallpaperSets),
    /// The drives file changed.
    Drives(Drives),
    /// Hyprland's screens file changed.
    Desk(Desk),
    /// A file a state is read from changed.
    States(States),
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
    // The "Add a keyboard layout" app is an Extension of its own, as Kotodama has one app an
    // Extension; its manifest runs this binary with `layouts`.
    if std::env::args().nth(1).as_deref() == Some("layouts") {
        return kotodama_extension::main("kotodama-kubaos-layouts", run_layouts);
    }
    kotodama_extension::main("kotodama-kubaos", run)
}

/// The "Add a keyboard layout" app's loop: its Items follow the user's layouts, and each run is
/// answered with what `kubaos keyboard add` came to.
fn run_layouts() -> Result<(), Error> {
    let Connection {
        mut writer,
        events,
        received,
    } = kotodama_extension::connect::<Own>()?;
    let lst = std::fs::read_to_string(keyboard::RULES_LST).unwrap_or_default();
    let keyboard_files = KeyboardFiles::new();
    let layouts = keyboard_files
        .as_ref()
        .map(|files| files.read().layouts().to_vec())
        .unwrap_or_default();
    let mut app = LayoutsApp::new(layouts, &lst);
    let _keyboard_watcher = keyboard_files.and_then(|files| watch_keyboards(files, events));
    let mut sent = SentItems::default();
    SentStatus::default().usable(&mut writer)?;
    let mut configured = false;

    for event in received {
        match event {
            Event::Message(ToExtension::Config(_)) => {
                configured = true;
                sent.publish(&mut writer, app.items())?;
            }
            Event::Own(Own::Keyboards(keyboards)) => {
                app.set_layouts(keyboards.layouts().to_vec());
                if configured {
                    sent.publish(&mut writer, app.items())?;
                }
            }
            Event::Message(ToExtension::RunAction {
                invocation,
                action,
                item,
                ..
            }) => {
                let outcome = match app.run(&item.id, &action, &Programs) {
                    Ok(_) => Outcome::Close,
                    Err(message) => Outcome::Failed { message },
                };
                writer.send(&FromExtension::ActionDone { invocation, outcome })?;
            }
            Event::Own(_) | Event::Message(_) => {}
            Event::Hangup => break,
        }
    }
    Ok(())
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
    let keyboard_files = KeyboardFiles::new();
    if let Some(files) = &keyboard_files {
        extension.set_keyboards(files.read());
    }
    let _keyboard_watcher = keyboard_files.and_then(|files| watch_keyboards(files, events.clone()));
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
    let _desk_watcher = read_desk(&mut extension, events.clone());
    let (places, _states_watcher) = read_states(&mut extension, events.clone());
    let _watcher = state.and_then(|dir| watch(dir, events));
    let mut configured = false;

    for event in received {
        match event {
            Event::Message(ToExtension::Config(table)) => {
                for problem in extension.configure(&table) {
                    config::warn(problem);
                }
                read_again(&mut extension, &places);
                configured = true;
                sent.publish(&mut writer, extension.items())?;
            }
            Event::Own(own) => {
                match own {
                    Own::Pending(pending) => extension.set_pending(pending),
                    Own::Keyboards(keyboards) => extension.set_keyboards(keyboards),
                    Own::WallpaperSets(sets) => extension.set_wallpaper_sets(sets),
                    Own::Drives(drives) => extension.set_drives(drives),
                    Own::Desk(desk) => extension.set_desk(desk),
                    Own::States(states) => extension.set_states(states),
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
        move |event, _| watched::changes(event, &changed),
        move |_| {
            let _ = events.send(Event::Own(Own::Pending(Pending::read(&file))));
        },
    )
    .map_err(|err| kotodama_extension::log!("not watching for KubaOS updates: {err}"))
    .ok()?;
    watcher.watch(vec![dir]);
    Some(watcher)
}

/// Where the keyboard layouts come from: the user's keyboard settings, then the system's file.
#[derive(Clone)]
struct KeyboardFiles {
    mine: PathBuf,
}

impl KeyboardFiles {
    /// The files under `XDG_CONFIG_HOME`, else `~/.config`; none without either.
    fn new() -> Option<Self> {
        let config_home = kubaos_user_layer::config_home()?;
        Some(KeyboardFiles {
            mine: keyboard_settings::path(&config_home),
        })
    }

    fn system() -> &'static Path {
        Path::new(keyboard::DEFAULT_KEYBOARD)
    }

    fn read(&self) -> Keyboards {
        Keyboards::read(&self.mine, Self::system(), Path::new(keyboard::RULES_LST))
    }
}

/// Watches the folders of the keyboard settings and of `/etc/default/keyboard`, sending the
/// layouts on each change to either. Without a watch the keyboard Items keep the layouts of the
/// start.
fn watch_keyboards(files: KeyboardFiles, events: Sender<Event<Own>>) -> Option<Watcher> {
    let mine_dir = files.mine.parent()?.to_path_buf();
    // `dot-config apply` makes the folder with its seed; one made here waits for the first change.
    let _ = std::fs::create_dir_all(&mine_dir);
    // The events name the real path, behind any symlink on the way.
    let mine_dir = mine_dir.canonicalize().unwrap_or(mine_dir);
    let mine = mine_dir.join(files.mine.file_name()?);
    let system = KeyboardFiles::system().to_path_buf();
    let system_dir = system.parent()?.to_path_buf();
    let watcher = Watcher::start(
        "keyboard",
        RecursiveMode::NonRecursive,
        SETTLE,
        // `kubaos keyboard` writes the settings, and its helper the system's file, through a
        // temporary file and a rename, as the update check does.
        move |event, _| watched::changes(event, &mine) || watched::changes(event, &system),
        move |_| {
            let _ = events.send(Event::Own(Own::Keyboards(files.read())));
        },
    )
    .map_err(|err| kotodama_extension::log!("not watching the keyboard layouts: {err}"))
    .ok()?;
    watcher.watch(vec![mine_dir, system_dir]);
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
        move |event, _| watched::changes(event, &changed),
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
        move |event, _| watched::changes(event, &changed),
        move |_| {
            let _ = events.send(Event::Own(Own::Drives(Drives::read(&file))));
        },
    )
    .map_err(|err| kotodama_extension::log!("not watching the mounted drives: {err}"))
    .ok()?;
    watcher.watch(vec![dir]);
    Some(watcher)
}

/// Gives `extension` the desk positions in Hyprland's screens file and the user's workspace
/// groups, and watches both.
fn read_desk(extension: &mut Extension<SystemBus, Programs>, events: Sender<Event<Own>>) -> Option<Watcher> {
    let files = DeskFiles {
        screens: desk::file()?,
        groups: desk::groups::path(),
    };
    extension.set_desk(files.read());
    watch_desk(files, events)
}

/// Where the desk Items come from: Hyprland's screens file and the user's workspace groups.
#[derive(Clone)]
struct DeskFiles {
    screens: PathBuf,
    groups: Option<PathBuf>,
}

impl DeskFiles {
    fn read(&self) -> Desk {
        let desk = Desk::read(&self.screens);
        match &self.groups {
            Some(groups) => desk.with_groups_file(groups.clone()),
            None => desk,
        }
    }
}

/// Watches the folders of the screens file and of the groups file, sending the desk on each change
/// to either. Without a watch the desk Items keep what the files said at the start.
fn watch_desk(files: DeskFiles, events: Sender<Event<Own>>) -> Option<Watcher> {
    let mut dirs = Vec::new();
    let mut changed = Vec::new();
    for file in std::iter::once(&files.screens).chain(&files.groups) {
        let dir = file.parent()?.to_path_buf();
        // Hyprland writes the screens file on its first monitor change, and `dot-config apply`
        // seeds the groups file; each folder waits for its file till then.
        let _ = std::fs::create_dir_all(&dir);
        // The events name the real path, behind any symlink on the way.
        let dir = dir.canonicalize().unwrap_or(dir);
        changed.push(dir.join(file.file_name()?));
        dirs.push(dir);
    }
    let watcher = Watcher::start(
        "desk",
        RecursiveMode::NonRecursive,
        SETTLE,
        // `config/hypr/workspaces.lua` and the menu's own edits write through a temporary file and
        // a rename onto the file.
        move |event, _| changed.iter().any(|file| watched::changes(event, file)),
        move |_| {
            let _ = events.send(Event::Own(Own::Desk(files.read())));
        },
    )
    .map_err(|err| kotodama_extension::log!("not watching the desk positions: {err}"))
    .ok()?;
    watcher.watch(dirs);
    Some(watcher)
}

/// Reads again what each config does: the user folders, which aren't watched, as they change
/// rarely and a folder made since shows up at the next reload; and the states, so a reload brings
/// them back in step when their watch couldn't start.
fn read_again(extension: &mut Extension<SystemBus, Programs>, places: &Places) {
    extension.set_user_folders(UserFolders::from_env());
    extension.set_states(States::read(places));
}

/// Gives `extension` the states of this machine and session, and watches their files.
fn read_states(
    extension: &mut Extension<SystemBus, Programs>,
    events: Sender<Event<Own>>,
) -> (Places, Option<Watcher>) {
    let places = Places::from_env();
    extension.set_states(States::read(&places));
    let watcher = watch_states(places.clone(), events);
    (places, watcher)
}

/// Watches the folders of every file a state is read from, sending the states on each change to
/// one. A folder that isn't there yet is watched once it is. Without a watch the actions keep the
/// states of the start, until the next config.
fn watch_states(places: Places, events: Sender<Event<Own>>) -> Option<Watcher> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut changed = Vec::new();
    for file in places.files() {
        let (Some(dir), Some(name)) = (file.parent(), file.file_name()) else {
            continue;
        };
        // The events name the real path, behind any symlink on the way.
        let dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        changed.push(dir.join(name));
        if !dirs.contains(&dir) {
            dirs.push(dir);
        }
    }
    let watcher = Watcher::start(
        "states",
        RecursiveMode::NonRecursive,
        SETTLE,
        // Most of the files are written through a temporary one and a rename, and the flag files
        // are made and removed: each is a change of the file itself.
        move |event, _| changed.iter().any(|file| watched::changes(event, file)),
        move |_| {
            let _ = events.send(Event::Own(Own::States(States::read(&places))));
        },
    )
    .map_err(|err| kotodama_extension::log!("not watching the states of the actions: {err}"))
    .ok()?;
    watcher.watch(dirs);
    Some(watcher)
}
