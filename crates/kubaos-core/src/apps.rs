//! The Apps catalog: a few essential apps KubaOS knows how to install well, each from one pinned,
//! verified source, and the user's picks of them.
//!
//! An app is a manifest, `apps/<id>.toml` in the release's checkout (`/usr/share/kubaos/apps/` on
//! a machine): an apt package from Ubuntu's archive or a vendor repository of the package
//! manifest, a package of the user's nixpkgs, or a vendor's `.deb` pinned to its sha256. The
//! user's picks, `~/.config/kubaos/apps.toml`, are what `kubaos update` reconciles the machine to.
//! Whether an app is installed is read from the machine each time, never recorded.
//!
//! # Owns
//!
//! - The manifest's format ([`App`]), the catalog ([`Catalog`]), the picks ([`Picks`]) and how the
//!   machine compares with them ([`plan`]).
//!
//! # Not here
//!
//! - Running the root helper, apt and the switch, and `kubaos update`'s phase: the updater's
//!   `apps.rs`. The root helper itself: `system/files/apps/kubaos-apps`.
//! - Vendor repositories: the package manifest, `apt::manifest`.

pub mod catalog;
pub mod manifest;
pub mod picks;
pub mod plan;

pub use self::catalog::Catalog;
pub use self::catalog::CatalogError;
pub use self::manifest::App;
pub use self::manifest::ManifestError;
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
