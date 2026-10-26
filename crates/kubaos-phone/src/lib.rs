//! KubaOS's phone link: the clipboard and files shared with the user's phone through KDE Connect.
//! It is opt-in. kdeconnectd announces itself on the network and listens for phones, so it runs
//! only once the user sets up a phone, and turning the link off stops it again.
//!
//! ```text
//! Kōbō's "Your phone" card ─┐
//! kubaos-phone (the menu) ──┴─▶ Link ──busctl──▶ kdeconnectd ◀──Wi-Fi──▶ the phone's app
//!                                 └──▶ the setting file, systemctl --user kubaos-phone.service
//! ```
//!
//! # Owns
//!
//! - What the phone link does, for the card and the command alike ([`phone`]).
//! - The machine behind it ([`link`]): KDE Connect on the session bus and the user unit
//!   ([`kdeconnect`]), and the opt-in ([`setting`]).
//! - What it reports ([`device`]).
//!
//! # Not here
//!
//! - The user unit: `home/phone.nix`.
//! - The card: `crates/kubaos-core/src/setup/` and `crates/kubaos-kobo/src/setup/`.
//! - The Command menu's Items: `config/kotodama/kubaos.toml`.

#![forbid(unsafe_code)]

pub mod device;
pub mod kdeconnect;
pub mod link;
pub mod phone;
pub mod setting;

#[cfg(any(test, feature = "fake"))]
pub mod fake;

pub use device::Battery;
pub use device::Device;
pub use device::Status;
pub use kdeconnect::KdeConnect;
pub use link::Action;
pub use link::Link;
pub use link::LinkError;
pub use phone::Error;
pub use phone::TEST_TEXT;
