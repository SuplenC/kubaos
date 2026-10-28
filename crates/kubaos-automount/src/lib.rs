//! KubaOS's automount. udisks2 mounts nothing on its own; a desktop's file manager or shell
//! usually asks it to. This service does: it mounts a USB drive's or a card's filesystems as they
//! appear, offers Open, Import photos and Eject on its notification, and keeps the list of
//! mounted removable drives the Command menu ejects from.
//!
//! ```text
//! udisks2 ──InterfacesAdded, PropertiesChanged──▶ service ──Mount, PowerOff──▶ udisks2
//! shell   ◀──Notify── service ◀──ActionInvoked── shell
//! service ──▶ $XDG_RUNTIME_DIR/kubaos-drives ──▶ Command menu ──eject──▶ kubaos-automount
//! ```
//!
//! # Owns
//!
//! - Which filesystems are mounted and how a drive is ejected ([`block`]), udisks2's side
//!   ([`udisks`]), the loop ([`service`]) and its notifications ([`notify`]).
//! - The drives file ([`drives`]), the automount setting ([`settings`]) and the photo import
//!   ([`import`]).
//!
//! # Not here
//!
//! - The user unit: `home/automount.nix`.
//! - The Eject Items: `crates/kotodama-kubaos/src/drives.rs`.

pub mod block;
pub mod drives;
pub mod import;
pub mod notify;
pub mod service;
pub mod settings;
pub mod udisks;
