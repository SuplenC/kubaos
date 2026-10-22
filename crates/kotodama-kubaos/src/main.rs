//! The KubaOS Extension's protocol loop: each config the Daemon sends makes
//! the Items over, sending only the ones that changed, and so does each
//! change the update check writes; each run is answered with its outcome,
//! after the View it pushes or pops.
//!
//! # Owns
//!
//! - The loop, logging the config's problems, and turning what a run came to into messages.
//! - Watching the update check's file.
//!
//! # Not here
//!
//! - The Items and the runs: the library's modules.

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::mpsc::Sender;
use std::time::Duration;

use kotodama_extension::Connection;
use kotodama_extension::Event;
use kotodama_extension::SentItems;
use kotodama_extension::SentStatus;
use kotodama_extension::config;
use kotodama_extension::watch::RecursiveMode;
use kotodama_extension::watch::Watcher;
use kotodama_kubaos::bus::SystemBus;
use kotodama_kubaos::extension::Extension;
use kotodama_kubaos::run::Programs;
use kotodama_kubaos::toggles::Ran;
use kotodama_kubaos::updates;
use kotodama_kubaos::updates::Pending;
use kotodama_protocol::Error;
use kotodama_protocol::FromExtension;
use kotodama_protocol::Outcome;
use kotodama_protocol::ToExtension;

/// How long the update check's folder stays quiet before its file is read:
/// the check writes it through a temporary file and a rename.
const SETTLE: Duration = Duration::from_millis(200);

fn main() -> ExitCode {
    kotodama_extension::main("kotodama-kubaos", run)
}

fn run() -> Result<(), Error> {
    let Connection {
        mut writer,
        events,
        received,
    } = kotodama_extension::connect::<Pending>()?;
    let mut extension = Extension::new(SystemBus, Programs);
    let mut sent = SentItems::default();
    // The toggles work on any machine, and the actions need only their programs.
    SentStatus::default().usable(&mut writer)?;
    let state = updates::state_dir();
    if let Some(dir) = &state {
        extension.set_pending(Pending::read(&dir.join(updates::FILE)));
    }
    let _watcher = state.and_then(|dir| watch(dir, events));
    let mut configured = false;

    for event in received {
        match event {
            Event::Message(ToExtension::Config(table)) => {
                for problem in extension.configure(&table) {
                    config::warn(problem);
                }
                configured = true;
                sent.publish(&mut writer, extension.items())?;
            }
            Event::Own(pending) => {
                extension.set_pending(pending);
                if configured {
                    sent.publish(&mut writer, extension.items())?;
                }
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
fn watch(dir: PathBuf, events: Sender<Event<Pending>>) -> Option<Watcher> {
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
            let _ = events.send(Event::Own(Pending::read(&file)));
        },
    )
    .map_err(|err| kotodama_extension::log!("not watching for KubaOS updates: {err}"))
    .ok()?;
    watcher.watch(vec![dir]);
    Some(watcher)
}
