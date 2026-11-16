//! The Apps catalog: a few essential apps KubaOS knows how to install well, and the user's picks
//! of them.
//!
//! An app is a folder, `apps/<id>/app.toml` in the release's checkout (`/usr/share/kubaos/apps/`
//! on a machine): what the app is, the ways it can be set up, and the typed actions its install
//! does in order, each with its inverse, recorded step by step in a journal that rolls a failed
//! install back and makes a removal undo exactly what the install did. The user's picks,
//! `~/.config/kubaos/apps.toml`, are what `kubaos update` reconciles the machine to. Whether an app
//! is installed, and in which setup, is its journal's.
//!
//! # Owns
//!
//! - An app's folder ([`folder`]), the catalog of them ([`Catalog`]), the action vocabulary
//!   ([`action`]), the machine it acts on ([`host`]), the journal ([`journal`]), installing,
//!   removing and taking in what the machine had ([`engine`]), the root helper's requests
//!   ([`root`]), the picks ([`Picks`]) and where each app stands ([`status`]).
//!
//! # Not here
//!
//! - `kubaos app` and `kubaos update`'s Apps phase: the updater's `apps.rs`. The root helper's
//!   binary: `kubaos-cli`'s `apps_root.rs`.
//! - Vendor repositories: the package manifest, `apt::manifest`.

pub mod action;
pub mod catalog;
pub mod engine;
pub mod folder;
pub mod host;
pub mod journal;
pub mod picks;
pub mod root;
pub mod status;

pub use self::catalog::Catalog;
pub use self::catalog::CatalogError;
pub use self::folder::Folder;
pub use self::folder::FolderSetup;
pub use self::picks::Picks;
pub use self::picks::PicksError;
pub use self::status::AppState;
pub use self::status::Status;

/// The machine's architecture, as dpkg names it. The `kubaos` binary is built for the machine's
/// own architecture, so this needs no `dpkg --print-architecture`: the Command menu asks for the
/// status as it opens.
pub fn arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        other => other,
    }
}


#[cfg(test)]
mod tests;
