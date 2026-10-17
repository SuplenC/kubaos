//! What the user chose: `install.toml`. Secrets never live in this file.

use serde::{Deserialize, Serialize};

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
    pub payload: Payload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Locale {
    /// glibc locale, e.g. `en_US.UTF-8`.
    pub lang: String,
    /// XKB layout, e.g. `us`, `it`.
    pub keyboard_layout: String,
    #[serde(default)]
    pub keyboard_variant: String,
    /// IANA zone, e.g. `Europe/Rome`.
    pub timezone: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Host {
    /// `tuxedo` or `vm`; also picks the flake output `<user>@<hostname>`.
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
