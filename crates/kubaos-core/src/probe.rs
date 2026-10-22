//! What the machine offers: `kubaos probe --json` prints one `Probe`.
//! Kōbō fills its pickers from it.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Probe {
    /// `x86_64` or `aarch64`.
    pub arch: String,
    pub uefi: bool,
    pub disks: Vec<DiskInfo>,
    /// Plain XKB layout names (`us`, `it`); variants aren't listed in v1.
    pub keymaps: Vec<String>,
    pub timezones: Vec<String>,
    pub locales: Vec<String>,
    pub wifi: Vec<WifiNetwork>,
    pub online: bool,
    /// `None` when no payload was found (Kōbō refuses to continue).
    pub payload: Option<Manifest>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiskInfo {
    pub path: String,
    pub model: String,
    pub size_bytes: u64,
    pub removable: bool,
    /// The stick we booted from: never offered as a target.
    pub is_boot_medium: bool,
    pub partitions: Vec<PartitionInfo>,
    pub free_regions: Vec<Region>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartitionInfo {
    pub path: String,
    pub size_bytes: u64,
    pub fstype: Option<String>,
    pub label: Option<String>,
    /// e.g. "Windows", "EFI system", from type GUID / fs.
    pub os_hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: u8,
    pub secured: bool,
}

/// `<payload dir>/manifest.json`, written by the image build.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    /// Release name, e.g. `Shinjuku` (docs/BRAND.md).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codename: Option<String>,
    pub arch: String,
    /// RFC 3339.
    pub built_at: String,
    /// Git commit of the dotfiles repo the image was built from.
    pub commit: String,
    /// The user the home-manager generation was built for.
    pub username: String,
    pub hostname: String,
    /// File name of the rootfs image inside the payload dir.
    pub rootfs: String,
    pub rootfs_bytes: u64,
    /// Uncompressed size, for the disk-size check and progress.
    pub installed_bytes: u64,
    pub sha256: String,
    pub kernel: String,
    /// Store path of the home-manager activation package.
    pub home_generation: String,
    /// The wallpaper sets in the rootfs (`/usr/share/kubaos/wallpapers/`),
    /// sorted by id. Empty for older images.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wallpaper_sets: Vec<WallpaperSet>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WallpaperSet {
    /// The folder name in the wallpaper repo, e.g. `shinjuku`.
    pub id: String,
    /// Images and videos in it.
    pub images: u32,
    /// A small JPEG of its first image, relative to the payload dir.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
}
