//! Disks: space on the mounts that matter (root, home on a disk of its own, removable drives),
//! traffic from `/proc/diskstats`, and what Btrfs snapshots hold.
//!
//! Snapshot space comes from Btrfs quota groups in sysfs (`/sys/fs/btrfs/<uuid>/qgroups`): snapper's
//! own group `1/0` when it set one up, otherwise what the subvolumes no mount uses hold alone. With
//! quotas off the kernel doesn't know, and the space is `None`.
//!
//! # Owns
//!
//! - Parsing `/proc/self/mounts` and `/proc/diskstats`, picking the mounts shown, their space, and
//!   the snapshot figures.
//!
//! # Not here
//!
//! - Mounting and ejecting drives: `crates/kubaos-automount`. Traffic speeds: [`crate::sampler`].

use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use crate::read;

/// Where removable drives get mounted: udisks's folders.
const REMOVABLE_ROOTS: [&str; 2] = ["/media/", "/run/media/"];

/// Block devices that aren't disks of their own: loops, RAM disks, and mapped devices sitting on
/// a disk already counted.
const NOT_DISKS: [&str; 6] = ["loop", "ram", "zram", "dm-", "sr", "md"];

/// A line of the mount table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mount {
    pub device: String,
    pub point: String,
    pub fstype: String,
    pub options: String,
}

impl Mount {
    /// The Btrfs subvolume it mounts, from `subvolid=`.
    pub fn subvolume_id(&self) -> Option<u64> {
        self.options
            .split(',')
            .find_map(|option| option.strip_prefix("subvolid="))
            .and_then(|id| id.parse().ok())
    }
}

/// Parses `/proc/self/mounts`, undoing its octal escapes (`\040` for a space).
pub fn parse_mounts(text: &str) -> Vec<Mount> {
    text.lines()
        .filter_map(|line| {
            let mut fields = line.split_ascii_whitespace();
            Some(Mount {
                device: unescape(fields.next()?),
                point: unescape(fields.next()?),
                fstype: fields.next()?.to_owned(),
                options: fields.next()?.to_owned(),
            })
        })
        .collect()
}

fn unescape(field: &str) -> String {
    let bytes = field.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let code = bytes
            .get(index + 1..index + 4)
            .filter(|_| bytes[index] == b'\\')
            .and_then(|digits| u8::from_str_radix(std::str::from_utf8(digits).ok()?, 8).ok());
        if let Some(code) = code {
            out.push(code);
            index += 4;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// A mount shown in the popup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Watched {
    pub point: String,
    pub label: String,
    pub removable: bool,
}

/// The mounts shown: root, `/home` when it is on another device, and each removable drive.
pub fn watched(mounts: &[Mount]) -> Vec<Watched> {
    let root = mounts.iter().rev().find(|mount| mount.point == "/");
    let mut shown: Vec<Watched> = root
        .map(|_| Watched {
            point: "/".to_owned(),
            label: "/".to_owned(),
            removable: false,
        })
        .into_iter()
        .collect();
    let home = mounts.iter().rev().find(|mount| mount.point == "/home");
    if let Some(home) = home
        && root.is_none_or(|root| root.device != home.device)
    {
        shown.push(Watched {
            point: home.point.clone(),
            label: home.point.clone(),
            removable: false,
        });
    }
    for mount in mounts {
        if REMOVABLE_ROOTS.iter().any(|prefix| mount.point.starts_with(prefix))
            && !shown.iter().any(|disk| disk.point == mount.point)
        {
            let label = mount.point.rsplit('/').next().unwrap_or(&mount.point).to_owned();
            shown.push(Watched {
                point: mount.point.clone(),
                label,
                removable: true,
            });
        }
    }
    shown
}

/// A mount's space.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Disk {
    pub point: String,
    pub label: String,
    pub removable: bool,
    pub total: u64,
    pub used: u64,
    /// What the user can still write.
    pub free: u64,
    pub percent: f32,
}

/// The space on `disk` now; `None` when the mount can't be asked (gone, or not allowed).
pub fn space(disk: &Watched) -> Option<Disk> {
    let stat = rustix::fs::statvfs(disk.point.as_str()).ok()?;
    let block = stat.f_frsize;
    let total = stat.f_blocks * block;
    if total == 0 {
        return None;
    }
    let used = total.saturating_sub(stat.f_bfree * block);
    let free = stat.f_bavail * block;
    // `df`'s percentage: used against what users may fill, the reserved blocks left out.
    let usable = used + free;
    Some(Disk {
        point: disk.point.clone(),
        label: disk.label.clone(),
        removable: disk.removable,
        total,
        used,
        free,
        percent: if usable == 0 {
            0.0
        } else {
            used as f32 * 100.0 / usable as f32
        },
    })
}

/// The whole disks under `/sys/block`, found once.
#[derive(Clone, Debug, Default)]
pub struct DiskSet {
    names: Vec<String>,
}

impl DiskSet {
    /// The disks under `root`.
    pub fn discover(root: &Path) -> Self {
        let names = read::entries(&root.join("sys/block"))
            .into_iter()
            .filter(|name| !NOT_DISKS.iter().any(|prefix| name.starts_with(prefix)))
            .collect();
        DiskSet { names }
    }
}

/// The bytes read and written since boot over the disks in `set`, from `/proc/diskstats`.
pub fn parse_diskstats(text: &str, set: &DiskSet) -> (u64, u64) {
    // Sectors in diskstats are 512 bytes whatever the disk's own sector size.
    const SECTOR: u64 = 512;
    let mut read = 0;
    let mut written = 0;
    for line in text.lines() {
        let fields: Vec<&str> = line.split_ascii_whitespace().collect();
        let Some(name) = fields.get(2) else {
            continue;
        };
        if !set.names.iter().any(|disk| disk == name) {
            continue;
        }
        let sectors = |index: usize| {
            fields
                .get(index)
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(0)
        };
        read += sectors(5) * SECTOR;
        written += sectors(9) * SECTOR;
    }
    (read, written)
}

/// What Btrfs snapshots hold.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshots {
    /// The space deleting them would free; `None` with Btrfs quotas off.
    pub bytes: Option<u64>,
    /// How many snapper keeps in `/.snapshots`, where it can be read.
    pub count: Option<u32>,
}

/// The snapshot figures for a Btrfs root; `None` when root isn't Btrfs.
pub fn snapshots(root: &Path, mounts: &[Mount]) -> Option<Snapshots> {
    let system = mounts.iter().rev().find(|mount| mount.point == "/")?;
    if system.fstype != "btrfs" {
        return None;
    }
    let mounted: Vec<u64> = mounts
        .iter()
        .filter(|mount| mount.fstype == "btrfs" && mount.device == system.device)
        .filter_map(Mount::subvolume_id)
        .collect();
    let btrfs = root.join("sys/fs/btrfs");
    let qgroups = read::entries(&btrfs)
        .into_iter()
        .find(|entry| entry.contains('-'))
        .map(|uuid| btrfs.join(uuid).join("qgroups"));
    let bytes = qgroups.and_then(|dir| snapshot_bytes(&dir, &mounted));
    let entries = read::entries(&root.join(".snapshots"));
    let count = entries
        .iter()
        .filter(|name| name.bytes().all(|b| b.is_ascii_digit()))
        .count();
    Some(Snapshots {
        bytes,
        count: (!entries.is_empty()).then_some(count as u32),
    })
}

fn snapshot_bytes(qgroups: &Path, mounted: &[u64]) -> Option<u64> {
    let exclusive = |id: &str| read::number::<u64>(&qgroups.join(id).join("exclusive"));
    let ids = read::entries(qgroups);
    if ids.is_empty() {
        return None;
    }
    if let Some(bytes) = exclusive("1_0") {
        return Some(bytes);
    }
    // Subvolume 5 is the filesystem's top level, which holds the others.
    let total = ids
        .iter()
        .filter_map(|id| id.strip_prefix("0_")?.parse::<u64>().ok())
        .filter(|id| *id != 5 && !mounted.contains(id))
        .filter_map(|id| exclusive(&format!("0_{id}")))
        .sum();
    Some(total)
}

#[cfg(test)]
mod tests;
