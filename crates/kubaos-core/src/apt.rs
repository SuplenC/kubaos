//! KubaOS's apt packages: the package manifest every install and update reads, the user's
//! choices of optional groups, the record of what KubaOS installed, and how the machine compares.
//!
//! The manifest is `system/packages.toml` in the release's checkout. Its groups list packages,
//! each group active or not by architecture and, when it is optional, by the user's choice in
//! `/etc/kubaos/apt-groups`. `/var/lib/kubaos/apt-installed` names each package KubaOS installed,
//! with its group, so an update removes a package dropped from the manifest only when KubaOS put
//! it there. The system scripts read the same three files through `system/manifest.sh`.
//!
//! # Owns
//!
//! - The formats of the manifest, the choices and the record.
//! - Reading `dpkg-query`, `apt-cache policy` and `apt-get -s` output.
//! - What a machine has of the manifest ([`plan::Status`]) and what a sync changes (`plan.rs`).
//!
//! # Not here
//!
//! - Running apt, and the phase of `kubaos update` that does: `update::updater`'s `manifest.rs`.
//! - Installing groups from the system scripts, and setting up a vendor repository:
//!   `system/manifest.sh`, `system/apt-repo.sh`.

pub mod choices;
pub mod dpkg;
pub mod manifest;
pub mod plan;
pub mod record;

pub use self::choices::Choices;
pub use self::choices::Switch;
pub use self::dpkg::Dpkg;
pub use self::manifest::Group;
pub use self::manifest::Manifest;
pub use self::manifest::ManifestError;
pub use self::plan::Status;
pub use self::record::Record;

#[cfg(test)]
mod tests;
