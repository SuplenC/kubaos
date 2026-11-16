//! The Apps catalog: a few essential apps KubaOS knows how to install well, each from one pinned,
//! verified source, and the user's picks of them.
//!
//! An app is a manifest, `apps/<id>.toml` in the release's checkout (`/usr/share/kubaos/apps/` on
//! a machine): an apt package from Ubuntu's archive or a vendor repository of the package
//! manifest, a package of the user's nixpkgs, or a vendor's `.deb` pinned to its sha256. The
//! user's picks, `~/.config/kubaos/apps.toml`, are what `kubaos update` reconciles the machine to.
//! An app may offer setups, one installed at a time, each adding its own parts in the user's home
//! to the source they share. Whether an app is installed, and in which setup, is read from the
//! machine each time, never recorded.
//!
//! An app may also be a folder, `apps/<id>/app.toml`: a list of typed actions, each with its
//! inverse, recorded step by step in a journal that rolls a failed install back and makes a
//! removal undo exactly what the install did.
//!
//! # Owns
//!
//! - An app's folder ([`folder`]), the action vocabulary ([`action`]), the machine it acts on
//!   ([`host`]), the journal ([`journal`]), installing and removing ([`engine`]) and the root
//!   helper's requests ([`root`]).
//! - The manifest's format ([`App`]), the catalog ([`Catalog`]), the picks ([`Picks`]), the record
//!   of the nixpkgs names each app added ([`Added`]) and how the machine compares with them
//!   ([`plan`]).
//!
//! # Not here
//!
//! - Running the root helper, apt and the switch, and `kubaos update`'s phase: the updater's
//!   `apps.rs`. The root helper itself: `system/files/apps/kubaos-apps`.
//! - Vendor repositories: the package manifest, `apt::manifest`.

pub mod action;
pub mod added;
pub mod catalog;
pub mod engine;
pub mod folder;
pub mod host;
pub mod journal;
pub mod manifest;
pub mod picks;
pub mod plan;
pub mod root;

pub use self::added::Added;
pub use self::added::AddedError;
pub use self::catalog::Catalog;
pub use self::catalog::CatalogError;
pub use self::manifest::App;
pub use self::manifest::ManifestError;
pub use self::manifest::Setup;
pub use self::manifest::Source;
pub use self::picks::Picks;
pub use self::picks::PicksError;
pub use self::plan::AppState;
pub use self::plan::Status;

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
