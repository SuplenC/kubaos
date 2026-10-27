//! Which file events the Extension's watchers act on: a write, a rename or a removal of the one
//! file each watches, never a read of it.
//!
//! # Owns
//!
//! - Telling a change of a watched file from a read of it.
//!
//! # Not here
//!
//! - The watchers themselves and what each reads once its file settles: `main.rs`.

use std::path::Path;

use kotodama_extension::watch::Event;

/// Whether `event` changes `file`. Each watcher reads its file once it settles, and the kernel
/// reports that open as an event on the file too: counted as a change, it would set the watcher
/// off again for ever.
#[must_use]
pub fn changes(event: &Event, file: &Path) -> bool {
    !event.kind.is_access() && event.paths.iter().any(|path| path == file)
}

#[cfg(test)]
mod tests;
