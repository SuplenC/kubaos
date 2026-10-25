//! yazi, KubaOS's terminal file manager, in a window of its own, and the Downloads tidy. This
//! crate opens yazi on a path or a `file://` URI, for its desktop entry (Open With in Nautilus,
//! the default file manager), its key and its Command menu Items; it can also answer apps' "Show
//! in folder" (`org.freedesktop.FileManager1`) for a user who registers it there instead.
//!
//! # Owns
//!
//! - Turning paths and URIs into what yazi opens: [`entry`].
//! - The command that opens the window: [`window`].
//! - The `org.freedesktop.FileManager1` service: [`service`].
//! - Sorting Downloads into folders by kind: [`tidy`].
//!
//! # Not here
//!
//! - The desktop entry, the folder defaults and the daily tidy's timer: `home/files.nix`.
//! - yazi's settings and keys: the user layer, `~/.config/kubaos/tools/yazi/`.
//! - The Command menu's Files Items: `crates/kotodama-kubaos/src/files.rs`.

#![forbid(unsafe_code)]

pub mod entry;
pub mod service;
pub mod tidy;
pub mod window;
