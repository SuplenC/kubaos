//! The Hyprland pin: the apt preferences file a release ships
//! (`system/files/apt/preferences.d/kubaos-hyprland`), one record per package, each pinning the
//! exact version the release was tested with at priority 1001, which apt installs even as a
//! downgrade (apt_preferences(5)).
//!
//! ```text
//! Package: hyprland
//! Pin: version 0.56.1-1~resolute1
//! Pin-Priority: 1001
//! ```
//!
//! # Owns
//!
//! - Reading the pin file, and which packages belong to the held Hyprland stack.
//!
//! # Not here
//!
//! - Installing the pinned versions and holding them: `updater.rs`.

/// One pinned package.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pin {
    pub package: String,
    pub version: String,
}

impl Pin {
    /// `package=version`, as `apt-get install` takes an exact version.
    pub fn install_arg(&self) -> String {
        format!("{}={}", self.package, self.version)
    }
}

/// The pins in an apt preferences file. A record without both a single `Package` and a
/// `Pin: version` is left out.
pub fn parse(text: &str) -> Vec<Pin> {
    text.split("\n\n")
        .filter_map(|record| {
            let field = |name: &str| {
                record
                    .lines()
                    .filter_map(|line| line.split_once(':'))
                    .find(|(key, _)| key.trim().eq_ignore_ascii_case(name))
                    .map(|(_, value)| value.trim())
            };
            let package = field("Package").filter(|p| !p.is_empty() && !p.contains([' ', '*']))?;
            let version = field("Pin")?.strip_prefix("version ")?.trim();
            (!version.is_empty()).then(|| Pin {
                package: package.to_owned(),
                version: version.to_owned(),
            })
        })
        .collect()
}

/// Whether `package` is part of the Hyprland stack KubaOS holds, as `HYPR_HOLD_RE` in
/// `system/lib.sh` says.
pub fn is_held_stack(package: &str) -> bool {
    ["hypr", "libhypr", "libaquamarine", "xdg-desktop-portal-hyprland"]
        .iter()
        .any(|prefix| package.starts_with(prefix))
}

#[cfg(test)]
mod tests;
