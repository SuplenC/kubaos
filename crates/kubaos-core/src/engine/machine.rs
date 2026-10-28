//! `probe`: what the machine offers. Reads lsblk, sysfs, xkb data,
//! zoneinfo, nmcli and the payload manifest. Works unprivileged.

use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;

use super::disk::GIB;
use super::disk::Part;
use super::disk::PartTable;
use crate::probe::DiskInfo;
use crate::probe::Manifest;
use crate::probe::PartitionInfo;
use crate::probe::Probe;
use crate::probe::WifiNetwork;

pub const DEFAULT_PAYLOAD_DIR: &str = "/run/kubaos/payload";

/// Test-only switch (`KUBAOS_TEST=1`): offer loop devices as disks and
/// pretend to be booted with UEFI, so the e2e test can run in a container.
pub fn test_mode() -> bool {
    std::env::var("KUBAOS_TEST").is_ok_and(|v| v == "1")
}

// ---------------------------------------------------------------------------
// lsblk

#[derive(Debug, Clone, Deserialize)]
pub struct Lsblk {
    pub blockdevices: Vec<BlockDev>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockDev {
    pub name: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default, deserialize_with = "num")]
    pub size: u64,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default, deserialize_with = "flag")]
    pub rm: bool,
    #[serde(default, deserialize_with = "flag")]
    pub ro: bool,
    #[serde(default)]
    pub tran: Option<String>,
    #[serde(default)]
    pub fstype: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub parttype: Option<String>,
    #[serde(default)]
    pub partlabel: Option<String>,
    #[serde(default)]
    pub pttype: Option<String>,
    #[serde(default, deserialize_with = "opt_num")]
    pub start: Option<u64>,
    /// util-linux >= 2.37; older ones have `mountpoint`.
    #[serde(default)]
    pub mountpoints: Vec<Option<String>>,
    #[serde(default)]
    pub mountpoint: Option<String>,
    #[serde(default)]
    pub children: Vec<BlockDev>,
}

/// lsblk prints numbers and booleans as JSON numbers/bools, or as strings
/// in older versions.
#[derive(Deserialize)]
#[serde(untagged)]
enum NumOrStr {
    N(u64),
    B(bool),
    S(String),
}

fn num<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    Ok(opt_num(d)?.unwrap_or(0))
}

fn opt_num<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<u64>, D::Error> {
    Ok(match Option::<NumOrStr>::deserialize(d)? {
        Some(NumOrStr::N(n)) => Some(n),
        Some(NumOrStr::S(s)) => s.trim().parse().ok(),
        Some(NumOrStr::B(_)) | None => None,
    })
}

fn flag<'de, D: serde::Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    Ok(match Option::<NumOrStr>::deserialize(d)? {
        Some(NumOrStr::B(b)) => b,
        Some(NumOrStr::N(n)) => n != 0,
        Some(NumOrStr::S(s)) => s.trim() == "1" || s.trim() == "true",
        None => false,
    })
}

impl BlockDev {
    pub fn dev_path(&self) -> String {
        self.path.clone().unwrap_or_else(|| format!("/dev/{}", self.name))
    }

    pub fn mounts(&self) -> Vec<String> {
        self.mountpoints
            .iter()
            .flatten()
            .cloned()
            .chain(self.mountpoint.clone())
            .collect()
    }

    /// This device and everything below it (partitions, LUKS, LVM).
    pub fn walk(&self) -> Vec<&BlockDev> {
        let mut v = vec![self];
        for c in &self.children {
            v.extend(c.walk());
        }
        v
    }

    fn os_hint(&self) -> Option<String> {
        let pt = self.parttype.as_deref().unwrap_or("").to_ascii_lowercase();
        let fs = self.fstype.as_deref().unwrap_or("");
        let hint = match (pt.as_str(), fs) {
            ("c12a7328-f81f-11d2-ba4b-00a0c93ec93b", _) | ("0xef", _) => "EFI system",
            ("e3c9e316-0b5c-4db8-817d-f92df00215ae", _) => "Windows (reserved)",
            ("de94bba4-06d1-4d40-a16a-bfd50179d6ac", _) => "Windows recovery",
            (_, "ntfs") | (_, "BitLocker") => "Windows",
            (_, "crypto_LUKS") => "Encrypted (LUKS)",
            (_, "btrfs" | "ext4" | "xfs" | "LVM2_member") => "Linux",
            (_, "swap") => "Linux swap",
            (_, "apfs" | "hfsplus") => "macOS",
            _ => return None,
        };
        Some(hint.into())
    }

    /// The partition table as lsblk sees it, in 512-byte sectors (sysfs
    /// units). `None` for a disk with no table (or one lsblk can't name).
    pub fn table(&self) -> Option<PartTable> {
        let parts: Vec<Part> = self
            .children
            .iter()
            .filter(|c| c.type_ == "part")
            .filter_map(|c| {
                Some(Part {
                    node: c.dev_path(),
                    start: c.start?,
                    size: c.size / 512,
                    type_: c.parttype.clone().unwrap_or_default(),
                    name: c.partlabel.clone(),
                })
            })
            .collect();
        // Without udev lsblk leaves pttype empty; partitions mean a table.
        let label = match self.pttype.as_deref() {
            Some(l) => l.to_string(),
            None if !parts.is_empty() => "gpt".into(),
            None => return None,
        };
        let mut t = PartTable::empty_gpt(&self.dev_path(), self.size, 512);
        t.label = label;
        t.partitions = parts;
        Some(t)
    }
}

/// What on one disk is in use, from its lsblk tree.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct InUse {
    /// Active swap: the live system turns on any swap partition it finds,
    /// so a disk with an old Linux on it usually has one.
    pub swaps: Vec<String>,
    /// Mount points of everything else that is mounted.
    pub mounts: Vec<String>,
}

pub fn in_use(l: &Lsblk, disk: &str) -> InUse {
    let mut u = InUse::default();
    let devs = l.blockdevices.iter().filter(|b| b.dev_path() == disk);
    for dev in devs.flat_map(BlockDev::walk) {
        for m in dev.mounts() {
            if m == "[SWAP]" {
                u.swaps.push(dev.dev_path());
            } else {
                u.mounts.push(m);
            }
        }
    }
    u
}

pub fn parse_lsblk(json: &str) -> Result<Lsblk, String> {
    serde_json::from_str(json).map_err(|e| format!("can't parse lsblk --json: {e}"))
}

pub const LSBLK_COLUMNS: &str =
    "NAME,PATH,TYPE,SIZE,MODEL,RM,RO,TRAN,FSTYPE,LABEL,PARTTYPE,PARTLABEL,PTTYPE,START,MOUNTPOINTS,PKNAME";

/// Mount points that mean "this is the stick we booted from".
const LIVE_MOUNTS: [&str; 5] = [
    "/cdrom",
    "/run/live/medium",
    "/run/initramfs/live",
    "/isodevice",
    "/run/archiso/bootmnt",
];

/// Disks for `Probe`. `boot_devs` are device names (any level) that hold
/// the live system or the payload; their top-level disk is the boot medium.
pub fn disks_from_lsblk(l: &Lsblk, boot_devs: &BTreeSet<String>, include_loop: bool) -> Vec<DiskInfo> {
    l.blockdevices
        .iter()
        .filter(|d| d.type_ == "disk" || (include_loop && d.type_ == "loop"))
        .filter(|d| !d.name.starts_with("zram") && !d.ro && d.size > 0)
        .map(|d| {
            let all = d.walk();
            let is_boot = all.iter().any(|x| {
                boot_devs.contains(&x.name) || x.mounts().iter().any(|m| LIVE_MOUNTS.contains(&m.as_str()) || m == "/")
            });
            let partitions = d
                .children
                .iter()
                .filter(|c| c.type_ == "part")
                .map(|c| PartitionInfo {
                    path: c.dev_path(),
                    size_bytes: c.size,
                    fstype: c.fstype.clone(),
                    label: c.label.clone().or_else(|| c.partlabel.clone()),
                    os_hint: c.os_hint(),
                })
                .collect();
            let free_regions = match d.table() {
                Some(t) if t.label == "gpt" => t.free_regions(),
                _ => vec![],
            };
            DiskInfo {
                path: d.dev_path(),
                model: d.model.clone().unwrap_or_default().trim().to_string(),
                size_bytes: d.size,
                removable: d.rm || d.tran.as_deref() == Some("usb"),
                is_boot_medium: is_boot,
                partitions,
                free_regions,
            }
        })
        .collect()
}

fn command_stdout(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Device names behind the live medium and the payload dir, following loop
/// devices to their backing file (a squashfs on the stick, say).
fn boot_device_names(payload_dir: &Path) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut targets: Vec<PathBuf> = LIVE_MOUNTS.iter().map(PathBuf::from).collect();
    targets.push(payload_dir.to_path_buf());
    let mut seen = 0;
    while let Some(t) = targets.pop() {
        seen += 1;
        if seen > 32 || !t.exists() {
            continue;
        }
        let Some(src) = command_stdout("findmnt", &["-no", "SOURCE", "--target", &t.to_string_lossy()]) else {
            continue;
        };
        // "/dev/sda1[/subdir]" for bind mounts; keep the device.
        let src = src.trim().split('[').next().unwrap_or("").to_string();
        let Some(name) = src.strip_prefix("/dev/") else {
            continue;
        };
        let name = name.rsplit('/').next().unwrap_or(name).to_string();
        if name.starts_with("loop")
            && let Some(back) = command_stdout("losetup", &["-nO", "BACK-FILE", &src])
        {
            let back = back.trim();
            if !back.is_empty() {
                targets.push(PathBuf::from(back));
            }
        }
        if name.starts_with("dm-") || src.starts_with("/dev/mapper/") {
            // Resolve to the kernel name lsblk uses.
            if let Ok(p) = std::fs::canonicalize(&src)
                && let Some(n) = p.file_name()
            {
                names.insert(n.to_string_lossy().into_owned());
            }
        }
        names.insert(name);
    }
    names
}

// ---------------------------------------------------------------------------
// keymaps, timezones, locales

/// Layout names from an xkb rules `.lst` file (`! layout` section).
pub fn parse_xkb_layouts(lst: &str) -> Vec<String> {
    let mut in_layout = false;
    let mut out = Vec::new();
    for line in lst.lines() {
        let t = line.trim();
        if let Some(section) = t.strip_prefix('!') {
            in_layout = section.trim() == "layout";
            continue;
        }
        if in_layout && let Some(name) = t.split_whitespace().next() {
            out.push(name.to_string());
        }
    }
    out.sort();
    out.dedup();
    out
}

fn keymaps() -> Vec<String> {
    for f in [
        "/usr/share/X11/xkb/rules/base.lst",
        "/usr/share/X11/xkb/rules/evdev.lst",
    ] {
        if let Ok(s) = std::fs::read_to_string(f) {
            let v = parse_xkb_layouts(&s);
            if !v.is_empty() {
                return v;
            }
        }
    }
    command_stdout("localectl", &["list-x11-keymap-layouts"])
        .map(|s| s.lines().map(str::to_string).filter(|l| !l.is_empty()).collect())
        .unwrap_or_default()
}

/// Zone names under a zoneinfo tree: TZif files, minus the `posix/` and
/// `right/` copies.
pub fn timezones_in(root: &Path) -> Vec<String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            let rel = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().into_owned();
            if rel == "posix" || rel == "right" || rel.starts_with('.') {
                continue;
            }
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_dir() {
                walk(root, &p, out);
            } else if is_tzif(&p) && rel != "localtime" && rel != "posixrules" && rel != "Factory" {
                out.push(rel);
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

fn is_tzif(p: &Path) -> bool {
    let mut head = [0u8; 4];
    std::fs::File::open(p)
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut head))
        .is_ok()
        && &head == b"TZif"
}

/// UTF-8 locales from `/usr/share/i18n/SUPPORTED` (`en_US.UTF-8 UTF-8`).
pub fn parse_supported_locales(s: &str) -> Vec<String> {
    let mut v: Vec<String> = s
        .lines()
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            let name = it.next()?;
            (it.next() == Some("UTF-8") && !name.starts_with('#')).then(|| name.to_string())
        })
        .collect();
    v.sort();
    v.dedup();
    v
}

fn locales() -> Vec<String> {
    std::fs::read_to_string("/usr/share/i18n/SUPPORTED")
        .map(|s| parse_supported_locales(&s))
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// wifi

/// `nmcli -t -f SSID,SIGNAL,SECURITY dev wifi list`: colon-separated, with
/// `\:` and `\\` escapes. Strongest entry per SSID, strongest first.
pub fn parse_nmcli_wifi(s: &str) -> Vec<WifiNetwork> {
    let mut nets: Vec<WifiNetwork> = Vec::new();
    for line in s.lines() {
        let f = split_nmcli(line);
        if f.len() < 3 || f[0].is_empty() {
            continue;
        }
        let signal = f[1].parse().unwrap_or(0);
        let secured = !f[2].is_empty() && f[2] != "--";
        match nets.iter_mut().find(|n| n.ssid == f[0]) {
            Some(n) if n.signal < signal => {
                n.signal = signal;
                n.secured = secured;
            }
            Some(_) => {}
            None => nets.push(WifiNetwork {
                ssid: f[0].clone(),
                signal,
                secured,
            }),
        }
    }
    nets.sort_by(|a, b| b.signal.cmp(&a.signal).then(a.ssid.cmp(&b.ssid)));
    nets
}

fn split_nmcli(line: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    let mut esc = false;
    for c in line.chars() {
        if esc {
            out.last_mut().unwrap().push(c);
            esc = false;
        } else if c == '\\' {
            esc = true;
        } else if c == ':' {
            out.push(String::new());
        } else {
            out.last_mut().unwrap().push(c);
        }
    }
    out
}

fn wifi() -> Vec<WifiNetwork> {
    command_stdout("nmcli", &["-t", "-f", "SSID,SIGNAL,SECURITY", "dev", "wifi", "list"])
        .map(|s| parse_nmcli_wifi(&s))
        .unwrap_or_default()
}

fn online() -> bool {
    command_stdout("nmcli", &["networking", "connectivity"]).is_some_and(|s| s.trim() == "full")
}

// ---------------------------------------------------------------------------

pub fn load_manifest(dir: &Path) -> Result<Manifest, String> {
    let p = dir.join("manifest.json");
    let s = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    serde_json::from_str(&s).map_err(|e| format!("{}: {e}", p.display()))
}

pub fn arch() -> String {
    std::env::consts::ARCH.to_string()
}

pub fn uefi() -> bool {
    Path::new("/sys/firmware/efi").exists() || test_mode()
}

pub fn read_lsblk() -> Result<Lsblk, String> {
    let out = Command::new("lsblk")
        .args(["-J", "-b", "-o", LSBLK_COLUMNS])
        .output()
        .map_err(|e| format!("lsblk: {e}"))?;
    if !out.status.success() {
        return Err(format!("lsblk: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    parse_lsblk(&String::from_utf8_lossy(&out.stdout))
}

pub fn probe(payload_dir: &Path) -> Result<Probe, String> {
    let lsblk = read_lsblk()?;
    let boot = boot_device_names(payload_dir);
    Ok(Probe {
        arch: arch(),
        uefi: uefi(),
        disks: disks_from_lsblk(&lsblk, &boot, test_mode()),
        keymaps: keymaps(),
        timezones: timezones_in(Path::new("/usr/share/zoneinfo")),
        locales: locales(),
        wifi: wifi(),
        online: online(),
        payload: load_manifest(payload_dir).ok(),
    })
}

/// Rough size label for logs: `465.8 GiB`.
pub fn gib(bytes: u64) -> String {
    format!("{:.1} GiB", bytes as f64 / GIB as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAPTOP: &str = include_str!("../../tests/fixtures/lsblk-laptop.json");
    const LOOP: &str = include_str!("../../tests/fixtures/lsblk-loop-noudev.json");

    #[test]
    fn a_disk_in_use_has_its_swap_apart_from_its_mounts() {
        let l = parse_lsblk(
            r#"{"blockdevices": [
              {"name": "sda", "path": "/dev/sda", "type": "disk", "mountpoints": [null], "children": [
                {"name": "sda1", "path": "/dev/sda1", "type": "part", "mountpoints": [null]},
                {"name": "sda2", "path": "/dev/sda2", "type": "part", "fstype": "swap", "mountpoints": ["[SWAP]"]},
                {"name": "sda3", "path": "/dev/sda3", "type": "part", "fstype": "crypto_LUKS", "mountpoints": [null], "children": [
                  {"name": "luks-old", "path": "/dev/mapper/luks-old", "type": "crypt", "mountpoints": ["/media/old", "/media/old2"]}
                ]}
              ]},
              {"name": "sdb", "path": "/dev/sdb", "type": "disk", "mountpoints": [null], "children": [
                {"name": "sdb1", "path": "/dev/sdb1", "type": "part", "fstype": "swap", "mountpoints": ["[SWAP]"]}
              ]}
            ]}"#,
        )
        .unwrap();
        assert_eq!(
            in_use(&l, "/dev/sda"),
            InUse {
                swaps: vec!["/dev/sda2".into()],
                mounts: vec!["/media/old".into(), "/media/old2".into()],
            }
        );
        assert_eq!(in_use(&l, "/dev/sdc"), InUse::default());
    }

    #[test]
    fn laptop_disks() {
        let l = parse_lsblk(LAPTOP).unwrap();
        let disks = disks_from_lsblk(&l, &BTreeSet::new(), false);
        let names: Vec<&str> = disks.iter().map(|d| d.path.as_str()).collect();
        // No loop, no rom, no zram.
        assert_eq!(names, vec!["/dev/sda", "/dev/nvme0n1"]);
        let stick = &disks[0];
        assert!(stick.is_boot_medium, "/cdrom is mounted from it");
        assert!(stick.removable);
        assert!(stick.free_regions.is_empty(), "dos table: no free-space install");
        let nvme = &disks[1];
        assert!(!nvme.is_boot_medium);
        assert_eq!(nvme.model, "Samsung SSD 990 PRO 1TB");
        assert_eq!(nvme.partitions.len(), 4);
        assert_eq!(nvme.partitions[0].os_hint.as_deref(), Some("EFI system"));
        assert_eq!(nvme.partitions[2].os_hint.as_deref(), Some("Windows"));
        assert_eq!(nvme.partitions[3].os_hint.as_deref(), Some("Windows recovery"));
        assert_eq!(nvme.free_regions.len(), 1);
        assert_eq!(nvme.free_regions[0].start, 524527616 * 512);
    }

    #[test]
    fn boot_medium_by_device_name() {
        let l = parse_lsblk(LAPTOP).unwrap();
        let boot: BTreeSet<String> = ["nvme0n1p3".to_string()].into();
        let disks = disks_from_lsblk(&l, &boot, false);
        assert!(disks[1].is_boot_medium);
    }

    #[test]
    fn loop_without_udev() {
        let l = parse_lsblk(LOOP).unwrap();
        assert!(disks_from_lsblk(&l, &BTreeSet::new(), false).is_empty());
        let disks = disks_from_lsblk(&l, &BTreeSet::new(), true);
        assert_eq!(disks.len(), 1);
        let d = &disks[0];
        assert_eq!(d.path, "/dev/loop0");
        assert_eq!(d.partitions.len(), 2);
        assert_eq!(d.partitions[0].fstype, None);
        // pttype is null without udev, but partitions imply a table.
        assert_eq!(d.free_regions.len(), 1);
        let r = d.free_regions[0];
        assert_eq!(r.start, (206848 + 8 * 1024 * 2048) * 512);
        assert!(r.end <= 20 * GIB);
    }

    #[test]
    fn old_lsblk_strings() {
        let j = r#"{"blockdevices":[{"name":"sda","type":"disk","size":"1000000000","rm":"1","ro":"0","mountpoint":null}]}"#;
        let l = parse_lsblk(j).unwrap();
        let d = &l.blockdevices[0];
        assert_eq!(d.size, 1_000_000_000);
        assert!(d.rm);
        assert!(!d.ro);
        assert_eq!(d.dev_path(), "/dev/sda");
    }

    #[test]
    fn xkb_layouts() {
        let lst = "! model\n  pc105  Generic 105-key PC\n\n! layout\n  us  English (US)\n  it  Italian\n  de  German\n\n! variant\n  intl  us: English (US, intl.)\n";
        assert_eq!(parse_xkb_layouts(lst), vec!["de", "it", "us"]);
    }

    #[test]
    fn locales_list() {
        let s = "en_US.UTF-8 UTF-8\nen_US ISO-8859-1\nit_IT.UTF-8 UTF-8\nde_DE@euro ISO-8859-15\n";
        assert_eq!(parse_supported_locales(s), vec!["en_US.UTF-8", "it_IT.UTF-8"]);
    }

    #[test]
    fn nmcli() {
        let s = "Home\\:5G:82:WPA2\nCafe:40:\nHome\\:5G:60:WPA2\n:30:WPA2\nOffice:82:WPA2 WPA3\n";
        let n = parse_nmcli_wifi(s);
        assert_eq!(
            n,
            vec![
                WifiNetwork {
                    ssid: "Home:5G".into(),
                    signal: 82,
                    secured: true
                },
                WifiNetwork {
                    ssid: "Office".into(),
                    signal: 82,
                    secured: true
                },
                WifiNetwork {
                    ssid: "Cafe".into(),
                    signal: 40,
                    secured: false
                },
            ]
        );
    }

    #[test]
    fn zoneinfo_walk() {
        let root = std::env::temp_dir().join(format!("kubaos-tz-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (p, c) in [
            ("Europe/Rome", &b"TZif2..."[..]),
            ("UTC", b"TZif2..."),
            ("posix/Europe/Rome", b"TZif2..."),
            ("zone1970.tab", b"# tab"),
            ("localtime", b"TZif2..."),
        ] {
            let f = root.join(p);
            std::fs::create_dir_all(f.parent().unwrap()).unwrap();
            std::fs::write(f, c).unwrap();
        }
        assert_eq!(timezones_in(&root), vec!["Europe/Rome", "UTC"]);
        std::fs::remove_dir_all(&root).unwrap();
    }
}
