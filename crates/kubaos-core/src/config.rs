//! What the user chose: `install.toml`. Secrets never live in this file.

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

use crate::keyboard::Layout;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallConfig {
    pub locale: Locale,
    pub host: Host,
    pub user: User,
    pub disk: Disk,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<Git>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wifi: Option<Wifi>,
    /// `None`: every set in the payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallpapers: Option<Wallpapers>,
    pub payload: Payload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Locale {
    /// glibc locale, e.g. `en_US.UTF-8`.
    pub lang: String,
    /// XKB layouts, the first the default: `["us", "it"]`. One string, as older configs have
    /// it, reads as XKB's comma-separated field: `"us"`, `"us,it"`.
    #[serde(deserialize_with = "one_or_many")]
    pub keyboard_layout: Vec<String>,
    /// The layouts' variants, by position; a missing or empty one is the layout's own.
    #[serde(default, deserialize_with = "one_or_many")]
    pub keyboard_variant: Vec<String>,
    /// IANA zone, e.g. `Europe/Rome`.
    pub timezone: String,
}

impl Locale {
    /// The keyboard layouts with their variants, the first the default.
    pub fn keyboards(&self) -> Vec<Layout> {
        let mut variants = self.keyboard_variant.iter();
        self.keyboard_layout
            .iter()
            .map(|layout| Layout::new(layout.as_str(), variants.next().map_or("", String::as_str)))
            .collect()
    }

    /// Sets both keyboard fields to `layouts`.
    pub fn set_keyboards(&mut self, layouts: &[Layout]) {
        self.keyboard_layout = layouts.iter().map(|l| l.layout.clone()).collect();
        self.keyboard_variant = layouts.iter().map(|l| l.variant.clone()).collect();
    }
}

/// A list, or one string holding XKB's comma-separated field.
fn one_or_many<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(String),
        Many(Vec<String>),
    }
    Ok(match OneOrMany::deserialize(d)? {
        OneOrMany::One(s) => s.split(',').map(|part| part.trim().to_string()).collect(),
        OneOrMany::Many(v) => v,
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Host {
    /// Any RFC 1123 label. The user flake's output is `<user>@<hostname>`, so plain
    /// `home-manager switch --flake ~/.config/kubaos/home` finds it; the home config itself
    /// never reads the hostname.
    pub hostname: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct User {
    /// v1: must equal the payload manifest's `username` (the home-manager
    /// generation in the image is built for it).
    pub username: String,
    pub full_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Disk {
    /// Whole-disk device node, e.g. `/dev/nvme0n1`, `/dev/vda`.
    pub device: String,
    pub target: DiskTarget,
    /// LUKS2 on the root partition. The passphrase comes from the secrets.
    #[serde(default = "yes")]
    pub encrypt: bool,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum DiskTarget {
    /// Wipe the device.
    Whole,
    /// Install into an unpartitioned region (dual boot). Byte offsets as
    /// reported by `probe`'s `free_regions`.
    FreeSpace { start: u64, end: u64 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Git {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wifi {
    pub ssid: String,
}

/// The wallpapers the picker shows: whole sets, and single wallpapers of other sets. Every set
/// ships in the payload either way (`/usr/share/kubaos/wallpapers/<id>/`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wallpapers {
    /// Ids from the manifest's `wallpaper_sets`, shown whole, so a wallpaper a release adds to
    /// one shows up too. Both lists empty: only the user's own folder.
    #[serde(default)]
    pub sets: Vec<String>,
    /// Single wallpapers, as `<set id>/<file name>`: `shinjuku/kabukicho-red-gate.jpg`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Payload {
    /// Directory holding `manifest.json` and the rootfs image.
    pub dir: String,
}

/// Read from `--secrets <file>` (TOML, mode 0600) or `--secrets-stdin`
/// (one JSON object). Never logged, never written to the target.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Secrets {
    #[serde(default)]
    pub luks_passphrase: Option<String>,
    pub user_password: String,
    #[serde(default)]
    pub wifi_psk: Option<String>,
}

impl std::fmt::Debug for Secrets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secrets { .. }")
    }
}

impl InstallConfig {
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}

#[cfg(test)]
mod tests;
