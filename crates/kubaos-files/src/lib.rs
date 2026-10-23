//! KubaOS's file manager is yazi in a terminal window. This crate opens that window on a path or a
//! `file://` URI, for `xdg-open` (the Files desktop entry) and for apps' "Show in folder"
//! (`org.freedesktop.FileManager1`).
//!
//! # Owns
//!
//! - Turning paths and URIs into what yazi opens: [`entry`].
//! - The command that opens the window: [`window`].
//! - The `org.freedesktop.FileManager1` service: [`service`].
//!
//! # Not here
//!
//! - Registering it as the folder handler and the D-Bus service: `home/files.nix`.
//! - yazi's settings and keys: the user layer, `~/.config/kubaos/yazi/`.
//! - The Command menu's Files Items: `crates/kotodama-kubaos/src/files.rs`.

#![forbid(unsafe_code)]

pub mod entry;
pub mod service;
pub mod window;
