//! Nix's releases, as the update's Nix phase reads them: the installed version, the newest patch
//! release of its minor, and the store path that release is.
//!
//! `system/60-nix.sh` installs one pinned Nix with the official multi-user installer. An update
//! moves the daemon on to the newest patch release of the installed minor (`2.35.3` to `2.35.4`)
//! with `nix upgrade-nix`, pointed at that release's `fallback-paths.nix`; a new minor waits for a
//! KubaOS release that pins it. The store path comes from Nix's binary cache, whose signature Nix
//! checks against its trusted keys before it installs anything.
//!
//! # Owns
//!
//! - The release list's URL and reading it, the choice of release ([`newest_patch`]), and reading a
//!   release's `fallback-paths.nix` ([`store_path`]).
//!
//! # Not here
//!
//! - Running Nix and systemd: `updater/nix.rs`.

use super::version::Version;

/// The multi-user installer's profile, which holds the daemon's Nix.
pub const PROFILE: &str = "nix/var/nix/profiles/default";
/// The bucket behind `releases.nixos.org`, which lists the releases as S3 XML; the site's own
/// page builds the same listing with JavaScript.
const LISTING: &str = "https://nix-releases.s3.amazonaws.com/?delimiter=/&prefix=nix/nix-";
/// Where each release's files are, `nix-2.35.4/fallback-paths.nix` among them.
const RELEASES: &str = "https://releases.nixos.org/nix";
/// The daemon's systemd units, which the installer links into `/etc/systemd/system`.
pub const UNITS: [&str; 2] = ["nix-daemon.service", "nix-daemon.socket"];

/// The listing of the releases of the minor `major.minor`.
pub fn listing_url(major: u64, minor: u64) -> String {
    format!("{LISTING}{major}.{minor}.")
}

/// The file `nix upgrade-nix` reads the release's store paths from.
pub fn fallback_paths_url(version: &Version) -> String {
    format!("{RELEASES}/nix-{version}/fallback-paths.nix")
}

/// The version `nix --version` prints, `nix (Nix) 2.35.3`; none when it prints no release.
pub fn installed_version(text: &str) -> Option<Version> {
    let word = text.split_whitespace().last()?;
    Version::parse(word).ok().filter(|version| !version.is_pre())
}

/// The releases the S3 listing names, as `<Prefix>nix/nix-2.35.4/</Prefix>`. A pre-release, as
/// `nix-2.36.0pre20260101_0123abc`, isn't a version and is left out.
pub fn parse_listing(xml: &str) -> Vec<Version> {
    xml.split("<Prefix>")
        .skip(1)
        .filter_map(|rest| {
            let prefix = rest.split("</Prefix>").next()?;
            let name = prefix.strip_prefix("nix/nix-")?.strip_suffix('/')?;
            Version::parse(name).ok().filter(|version| !version.is_pre())
        })
        .collect()
}

/// The newest patch release of `installed`'s minor that is newer than it.
pub fn newest_patch(installed: &Version, releases: &[Version]) -> Option<Version> {
    releases
        .iter()
        .filter(|version| version.same_series(installed) && *version > installed)
        .max()
        .cloned()
}

/// The Nix system of a Debian architecture, `amd64` → `x86_64-linux`.
pub fn system_of(arch: &str) -> Option<&'static str> {
    match arch.trim() {
        "amd64" => Some("x86_64-linux"),
        "arm64" => Some("aarch64-linux"),
        _ => None,
    }
}

/// The store path `fallback-paths.nix` gives `system`, when it is `version`'s Nix:
/// `x86_64-linux = "/nix/store/<hash>-nix-2.35.4";`.
pub fn store_path(fallback_paths: &str, system: &str, version: &Version) -> Option<String> {
    let path = fallback_paths.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        (key.trim() == system).then(|| value.trim().trim_end_matches(';').trim().trim_matches('"'))
    })?;
    let name = path.strip_prefix("/nix/store/")?;
    let (hash, rest) = name.split_once('-')?;
    let is_hash = hash.len() == 32 && hash.bytes().all(|b| b.is_ascii_digit() || b.is_ascii_lowercase());
    (is_hash && rest == format!("nix-{version}")).then(|| path.to_owned())
}

#[cfg(test)]
mod tests;
