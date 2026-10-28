//! Partition math. Pure: everything here works on numbers and parsed
//! tables, so it's unit-tested without a disk.

use serde::Deserialize;

use crate::config::DiskTarget;
use crate::probe::Region;

pub const MIB: u64 = 1024 * 1024;
pub const GIB: u64 = 1024 * MIB;
pub const ESP_BYTES: u64 = GIB;
pub const BOOT_BYTES: u64 = 2 * GIB;
/// Smallest root partition we'll create, on top of the payload.
pub const ROOT_HEADROOM: u64 = 4 * GIB;
/// Free regions smaller than this aren't offered.
pub const MIN_REGION: u64 = GIB;

pub const ESP_GUID: &str = "C12A7328-F81F-11D2-BA4B-00A0C93EC93B";

/// `/dev/sda` + 3 -> `/dev/sda3`; `/dev/nvme0n1` + 3 -> `/dev/nvme0n1p3`.
/// The kernel's rule: a `p` when the disk name ends in a digit.
pub fn partition_path(disk: &str, n: u32) -> String {
    if disk.chars().last().is_some_and(|c| c.is_ascii_digit()) {
        format!("{disk}p{n}")
    } else {
        format!("{disk}{n}")
    }
}

/// `sfdisk --json <disk>`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SfdiskDump {
    pub partitiontable: PartTable,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PartTable {
    pub label: String,
    #[serde(default)]
    pub device: String,
    #[serde(default = "default_sector")]
    pub sectorsize: u64,
    pub firstlba: u64,
    pub lastlba: u64,
    #[serde(default)]
    pub partitions: Vec<Part>,
}

fn default_sector() -> u64 {
    512
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Part {
    pub node: String,
    pub start: u64,
    pub size: u64,
    #[serde(rename = "type", default)]
    pub type_: String,
    #[serde(default)]
    pub name: Option<String>,
}

impl Part {
    pub fn end(&self) -> u64 {
        self.start + self.size - 1
    }
    pub fn is_esp(&self) -> bool {
        self.type_.eq_ignore_ascii_case(ESP_GUID)
    }
    /// The partition number from the node name (`/dev/nvme0n1p12` -> 12).
    pub fn number(&self) -> Option<u32> {
        let digits: String = self.node.chars().rev().take_while(|c| c.is_ascii_digit()).collect();
        digits.chars().rev().collect::<String>().parse().ok()
    }
}

pub fn parse_sfdisk(json: &str) -> Result<PartTable, String> {
    serde_json::from_str::<SfdiskDump>(json)
        .map(|d| d.partitiontable)
        .map_err(|e| format!("can't parse sfdisk --json: {e}"))
}

impl PartTable {
    /// A fresh GPT over a disk of `size_bytes` (what `sgdisk -o` gives).
    pub fn empty_gpt(device: &str, size_bytes: u64, sectorsize: u64) -> Self {
        let sectors = size_bytes / sectorsize;
        // 1 MiB in, and room for the 32-sector backup array + header at the end.
        let backup = 16 * 1024 / sectorsize + 1;
        PartTable {
            label: "gpt".into(),
            device: device.into(),
            sectorsize,
            firstlba: (MIB / sectorsize).max(34),
            lastlba: sectors.saturating_sub(backup + 1),
            partitions: vec![],
        }
    }

    fn align(&self) -> u64 {
        (MIB / self.sectorsize).max(1)
    }

    /// Unallocated gaps, aligned to 1 MiB, as inclusive sector ranges.
    pub fn gaps(&self) -> Vec<(u64, u64)> {
        let a = self.align();
        let mut parts: Vec<&Part> = self.partitions.iter().collect();
        parts.sort_by_key(|p| p.start);
        let mut out = Vec::new();
        let mut cursor = self.firstlba;
        let mut push = |from: u64, to: u64| {
            // [from, to] inclusive; align start up, end+1 down.
            let s = from.div_ceil(a) * a;
            let e_excl = (to + 1) / a * a;
            if e_excl > s {
                out.push((s, e_excl - 1));
            }
        };
        for p in parts {
            if p.start > cursor {
                push(cursor, p.start - 1);
            }
            cursor = cursor.max(p.end() + 1);
        }
        if self.lastlba >= cursor {
            push(cursor, self.lastlba);
        }
        out
    }

    /// Gaps as byte regions (end exclusive), the big ones only.
    pub fn free_regions(&self) -> Vec<Region> {
        self.gaps()
            .into_iter()
            .map(|(s, e)| Region {
                start: s * self.sectorsize,
                end: (e + 1) * self.sectorsize,
            })
            .filter(|r| r.end - r.start >= MIN_REGION)
            .collect()
    }

    pub fn esp(&self) -> Option<&Part> {
        self.partitions.iter().find(|p| p.is_esp())
    }

    fn free_numbers(&self, n: usize) -> Vec<u32> {
        let used: Vec<u32> = self.partitions.iter().filter_map(Part::number).collect();
        (1..=128).filter(|i| !used.contains(i)).take(n).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewPart {
    pub number: u32,
    /// Inclusive sector range.
    pub start: u64,
    pub end: u64,
    /// sgdisk type code.
    pub code: &'static str,
    pub name: &'static str,
}

impl NewPart {
    pub fn sgdisk_args(&self) -> Vec<String> {
        let n = self.number;
        vec![
            format!("--new={n}:{}:{}", self.start, self.end),
            format!("--typecode={n}:{}", self.code),
            format!("--change-name={n}:{}", self.name),
        ]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Esp {
    New(NewPart),
    /// Keep the disk's ESP (dual boot): its node.
    Reuse(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub esp: Esp,
    pub boot: NewPart,
    pub root: NewPart,
}

impl Layout {
    pub fn new_parts(&self) -> Vec<&NewPart> {
        let mut v = Vec::new();
        if let Esp::New(p) = &self.esp {
            v.push(p);
        }
        v.push(&self.boot);
        v.push(&self.root);
        v
    }
}

/// Bytes the install needs on the disk: our partitions plus the unpacked
/// system plus headroom (a fifth of it, at least `ROOT_HEADROOM`).
pub fn required_bytes(installed_bytes: u64, reuse_esp: bool) -> u64 {
    let esp = if reuse_esp { 0 } else { ESP_BYTES };
    esp + BOOT_BYTES + installed_bytes + (installed_bytes / 5).max(ROOT_HEADROOM)
}

/// Where the partitions go. `table` is the disk's GPT as it is now (after
/// `sgdisk --zap-all -o` in whole-disk mode).
pub fn plan_layout(
    target: &DiskTarget,
    table: &PartTable,
    encrypt: bool,
    min_root_bytes: u64,
) -> Result<Layout, String> {
    if table.label != "gpt" {
        return Err(format!(
            "{} has a {} partition table; only GPT is supported for installing next to another system",
            table.device, table.label
        ));
    }
    let ss = table.sectorsize;
    let a = table.align();
    let (start, end, reuse) = match target {
        DiskTarget::Whole => {
            let (s, e) = table
                .gaps()
                .into_iter()
                .max_by_key(|(s, e)| e - s)
                .ok_or_else(|| format!("{} has no usable space", table.device))?;
            (s, e, None)
        }
        DiskTarget::FreeSpace { start, end } => {
            if end <= start {
                return Err(format!("free-space region {start}..{end} is empty"));
            }
            let s = (start / ss).div_ceil(a) * a;
            let e = (end / ss) / a * a; // exclusive
            if e <= s {
                return Err("free-space region is smaller than 1 MiB".into());
            }
            let (s, e) = (s, e - 1);
            if !table.gaps().iter().any(|(gs, ge)| *gs <= s && e <= *ge) {
                return Err(format!(
                    "the region {start}..{end} overlaps an existing partition on {}",
                    table.device
                ));
            }
            (s, e, table.esp().map(|p| p.node.clone()))
        }
    };
    let nums = table.free_numbers(3);
    if nums.len() < 3 {
        return Err("no free partition numbers left".into());
    }
    let mut cursor = start;
    let mut nums = nums.into_iter();
    let mut take = |bytes: u64, code, name| {
        let n = nums.next().unwrap_or(0);
        let p = NewPart {
            number: n,
            start: cursor,
            end: cursor + bytes / ss - 1,
            code,
            name,
        };
        cursor = p.end + 1;
        p
    };
    let esp = match reuse {
        Some(node) => Esp::Reuse(node),
        None => Esp::New(take(ESP_BYTES, "ef00", "EFI")),
    };
    let boot = take(BOOT_BYTES, "8300", "boot");
    if cursor > end {
        return Err("the region is too small for /boot".into());
    }
    let root_code = if encrypt { "8309" } else { "8300" };
    let root_sectors = end + 1 - cursor;
    if root_sectors * ss < min_root_bytes {
        return Err(format!(
            "only {:.1} GiB left for the system, it needs {:.1} GiB",
            (root_sectors * ss) as f64 / GIB as f64,
            min_root_bytes as f64 / GIB as f64
        ));
    }
    let n = nums.next().unwrap_or(0);
    let root = NewPart {
        number: n,
        start: cursor,
        end,
        code: root_code,
        name: "root",
    };
    Ok(Layout { esp, boot, root })
}

/// Swap file size for a root filesystem of `root_bytes`: 8 GiB like
/// `system/00-btrfs-subvolumes.md`, but none on small disks (test VMs).
pub fn swap_bytes(root_bytes: u64) -> u64 {
    if root_bytes >= 64 * GIB { 8 * GIB } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_names() {
        assert_eq!(partition_path("/dev/sda", 3), "/dev/sda3");
        assert_eq!(partition_path("/dev/vda", 1), "/dev/vda1");
        assert_eq!(partition_path("/dev/nvme0n1", 2), "/dev/nvme0n1p2");
        assert_eq!(partition_path("/dev/mmcblk0", 1), "/dev/mmcblk0p1");
        assert_eq!(partition_path("/dev/loop7", 3), "/dev/loop7p3");
    }

    const DUALBOOT: &str = include_str!("../../tests/fixtures/sfdisk-dualboot.json");

    #[test]
    fn parses_sfdisk_and_finds_gaps() {
        let t = parse_sfdisk(DUALBOOT).unwrap();
        assert_eq!(t.label, "gpt");
        assert_eq!(t.partitions.len(), 4);
        assert_eq!(t.esp().unwrap().node, "/dev/nvme0n1p1");
        assert_eq!(t.partitions[3].number(), Some(4));
        // One gap: after the Windows partition (C: shrunk), before recovery.
        let regions = t.free_regions();
        assert_eq!(regions.len(), 1, "{regions:?}");
        let r = regions[0];
        assert_eq!(r.start % MIB, 0);
        assert_eq!(r.end % MIB, 0);
        assert_eq!(r.start, (2048 + 204800 + 32768 + 524288000) * 512);
        assert_eq!(r.end, 1_500_000_256 / 2048 * 2048 * 512);
    }

    #[test]
    fn whole_disk_layout() {
        let t = PartTable::empty_gpt("/dev/vda", 64 * GIB, 512);
        let l = plan_layout(&DiskTarget::Whole, &t, true, 10 * GIB).unwrap();
        let Esp::New(esp) = &l.esp else { panic!() };
        assert_eq!((esp.number, esp.start, esp.end), (1, 2048, 2048 + 2097152 - 1));
        assert_eq!(l.boot.number, 2);
        assert_eq!(l.boot.start, esp.end + 1);
        assert_eq!((l.boot.end + 1 - l.boot.start) * 512, BOOT_BYTES);
        assert_eq!(l.root.number, 3);
        assert_eq!(l.root.code, "8309");
        assert_eq!(l.root.start, l.boot.end + 1);
        assert_eq!((l.root.end + 1) % 2048, 0, "root ends on a MiB boundary");
        assert!(l.root.end <= t.lastlba);
        assert_eq!(
            esp.sgdisk_args(),
            vec!["--new=1:2048:2099199", "--typecode=1:ef00", "--change-name=1:EFI"]
        );
    }

    #[test]
    fn whole_disk_too_small() {
        let t = PartTable::empty_gpt("/dev/vda", 8 * GIB, 512);
        let e = plan_layout(&DiskTarget::Whole, &t, true, 10 * GIB).unwrap_err();
        assert!(e.contains("needs 10.0 GiB"), "{e}");
    }

    #[test]
    fn free_space_reuses_esp_and_numbers() {
        let t = parse_sfdisk(DUALBOOT).unwrap();
        let r = t.free_regions()[0];
        let l = plan_layout(
            &DiskTarget::FreeSpace {
                start: r.start + 4096,
                end: r.end,
            },
            &t,
            true,
            10 * GIB,
        )
        .unwrap();
        assert_eq!(l.esp, Esp::Reuse("/dev/nvme0n1p1".into()));
        assert_eq!(l.boot.number, 5);
        assert_eq!(l.root.number, 6);
        // Start rounded up to the next MiB, end stays inside the region.
        assert_eq!(l.boot.start * 512, r.start + MIB);
        assert_eq!((l.root.end + 1) * 512, r.end);
    }

    #[test]
    fn free_space_rejects_overlap() {
        let t = parse_sfdisk(DUALBOOT).unwrap();
        let e = plan_layout(
            &DiskTarget::FreeSpace {
                start: 0,
                end: 100 * GIB,
            },
            &t,
            true,
            GIB,
        )
        .unwrap_err();
        assert!(e.contains("overlaps"), "{e}");
    }

    #[test]
    fn mbr_rejected() {
        let mut t = PartTable::empty_gpt("/dev/sda", 64 * GIB, 512);
        t.label = "dos".into();
        assert!(plan_layout(&DiskTarget::Whole, &t, true, GIB).is_err());
    }

    #[test]
    fn sizes() {
        assert_eq!(required_bytes(10 * GIB, false), GIB + 2 * GIB + 10 * GIB + 4 * GIB);
        assert_eq!(required_bytes(40 * GIB, true), 2 * GIB + 40 * GIB + 8 * GIB);
        assert_eq!(swap_bytes(500 * GIB), 8 * GIB);
        assert_eq!(swap_bytes(20 * GIB), 0);
    }

    #[test]
    fn gaps_on_4k_sectors() {
        let t = PartTable::empty_gpt("/dev/nvme1n1", 100 * GIB, 4096);
        let g = t.gaps();
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].0, 256);
        assert_eq!((g[0].1 + 1) % 256, 0);
    }
}
