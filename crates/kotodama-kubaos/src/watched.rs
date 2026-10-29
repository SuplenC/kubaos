//! Which file events the Extension's watchers act on: a write, a rename or a removal of the one
//! file each watches, never a read of it.
//!
//! # Owns
//!
//! - Telling a change of a watched file from a read of it.
//! - Saying why a watcher couldn't start, naming the kernel's limit when it is one.
//!
//! # Not here
//!
//! - The watchers themselves and what each reads once its file settles: `main.rs`.

use std::path::Path;

use kotodama_extension::watch::Event;
use kotodama_extension::watch::notify;

/// `EMFILE`, which `inotify_init` returns when the user holds as many inotify instances as
/// `fs.inotify.max_user_instances` allows, or the process as many files as its `RLIMIT_NOFILE`.
const EMFILE: i32 = 24;

/// Whether `event` changes `file`. Each watcher reads its file once it settles, and the kernel
/// reports that open as an event on the file too: counted as a change, it would set the watcher
/// off again for ever.
#[must_use]
pub fn changes(event: &Event, file: &Path) -> bool {
    !event.kind.is_access() && event.paths.iter().any(|path| path == file)
}

/// Why a watcher couldn't start: `err`, and the sysctl to raise when the user's inotify instances
/// or watches are used up. Each watcher holds an instance of its own, and "Too many open files"
/// alone doesn't say which limit ran out.
#[must_use]
pub fn why(err: &notify::Error) -> String {
    match &err.kind {
        notify::ErrorKind::MaxFilesWatch => {
            format!("{err}: the user's inotify watches are used up (fs.inotify.max_user_watches)")
        }
        notify::ErrorKind::Io(io) if io.raw_os_error() == Some(EMFILE) => {
            format!(
                "{err}: the user's inotify instances are used up (fs.inotify.max_user_instances), or the \
                 process's open files (RLIMIT_NOFILE)"
            )
        }
        _ => err.to_string(),
    }
}

#[cfg(test)]
mod tests;
