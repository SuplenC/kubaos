//! What udisks2 reports of the block devices and drives, and the decisions made from it: which
//! filesystem to mount when it appears, which drives the menu can eject, and what ejecting one
//! takes. Plain data, so every decision is tested without a bus.
//!
//! # Owns
//!
//! - [`Block`] and [`Drive`], the parts of udisks2's objects the decisions read.
//! - [`verdict`], [`mounted_drives`], [`find_drive`] and [`eject_plan`].
//! - What the user calls a filesystem or a drive: [`block_name`], [`drive_name`].
//!
//! # Not here
//!
//! - Reading the objects off the system bus, and making the calls: [`crate::udisks`].

use std::path::Path;
use std::path::PathBuf;

/// A block device: a partition, a whole disk, or the cleartext side of an unlocked LUKS volume.
#[expect(clippy::struct_excessive_bools, reason = "each flag is one of udisks2's properties, read as it is")]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Block {
    /// Its udisks2 object path, `/org/freedesktop/UDisks2/block_devices/sdb1`.
    pub path: String,
    /// Its device file, `/dev/sdb1`.
    pub device: String,
    /// The object path of the drive it is on; none for a loop device or a cleartext volume.
    pub drive: Option<String>,
    /// `IdUsage`: `filesystem`, `crypto`, `raid`, `other`, or empty.
    pub usage: String,
    /// `IdLabel`, the filesystem's own name.
    pub label: String,
    /// Whether udisks2 offers the `Filesystem` interface on it, so it can be mounted.
    pub filesystem: bool,
    pub mount_points: Vec<PathBuf>,
    /// `HintIgnore`: udev rules asked for it to be left out of user interfaces.
    pub hint_ignore: bool,
    /// `HintSystem`: udisks2's own call that it is part of the machine, not removable.
    pub hint_system: bool,
    /// The object path of the LUKS block this one is the unlocked side of.
    pub crypto_backing: Option<String>,
    /// Whether `/etc/fstab` or `/etc/crypttab` names it, which leaves it to systemd.
    pub configured: bool,
}

/// A drive: a USB stick, a card reader, a disk.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Drive {
    /// Its udisks2 object path, `/org/freedesktop/UDisks2/drives/SanDisk_Ultra_1234`.
    pub path: String,
    pub vendor: String,
    pub model: String,
    /// `Removable` or `MediaRemovable`: plugged in from outside, or taking cards or discs.
    pub removable: bool,
    /// `Ejectable`: the medium comes out on a command (a disc, some card readers).
    pub ejectable: bool,
    /// `CanPowerOff`: the drive can be powered down, which is a USB stick's "safe to remove".
    pub can_power_off: bool,
}

/// What to do with a block device that appeared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Mount,
    /// A locked LUKS volume on a removable drive. Unlocking takes its passphrase, so it is left
    /// to the user, who unlocks it from the Files sidebar.
    Encrypted,
    Skip(Skip),
}

/// Why a block device isn't mounted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skip {
    /// Nothing to mount on it: a partition table, swap, an unknown filesystem.
    NoFilesystem,
    AlreadyMounted,
    Ignored,
    /// In fstab or crypttab, so systemd and the user decide when it mounts.
    Configured,
    /// The cleartext side of a LUKS volume: whoever unlocked it mounts it.
    Unlocked,
    /// On no drive (a loop device), or on a drive that is part of the machine.
    Internal,
}

/// What to do with `block`, which appeared on `drive`.
pub fn verdict(block: &Block, drive: Option<&Drive>) -> Verdict {
    if block.hint_ignore {
        return Verdict::Skip(Skip::Ignored);
    }
    if block.configured {
        return Verdict::Skip(Skip::Configured);
    }
    if block.crypto_backing.is_some() {
        return Verdict::Skip(Skip::Unlocked);
    }
    if block.hint_system || !drive.is_some_and(|drive| drive.removable) {
        return Verdict::Skip(Skip::Internal);
    }
    if block.usage == "crypto" {
        return Verdict::Encrypted;
    }
    if !block.filesystem || block.usage != "filesystem" {
        return Verdict::Skip(Skip::NoFilesystem);
    }
    if !block.mount_points.is_empty() {
        return Verdict::Skip(Skip::AlreadyMounted);
    }
    Verdict::Mount
}

/// A removable drive with at least one filesystem mounted: what the menu offers to eject.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MountedDrive {
    pub drive: String,
    /// What the user calls it: the first filesystem's label, else the vendor and model, else the
    /// device file.
    pub name: String,
    pub mount_points: Vec<PathBuf>,
}

/// The removable drives with a filesystem mounted, in `drives` order. A LUKS volume's cleartext
/// side counts for the drive its locked side is on.
pub fn mounted_drives(blocks: &[Block], drives: &[Drive]) -> Vec<MountedDrive> {
    drives
        .iter()
        .filter(|drive| drive.removable)
        .filter_map(|drive| {
            let mounted: Vec<&Block> = blocks
                .iter()
                .filter(|block| !block.mount_points.is_empty() && drive_of(block, blocks) == Some(&drive.path))
                .collect();
            let first = mounted.first()?;
            Some(MountedDrive {
                drive: drive.path.clone(),
                name: block_name(first, Some(drive)),
                mount_points: mounted.iter().flat_map(|block| block.mount_points.clone()).collect(),
            })
        })
        .collect()
}

/// The drive `target` names: a drive's object path, a block device's object path or device
/// file, or a mount point.
pub fn find_drive<'a>(target: &str, blocks: &'a [Block], drives: &'a [Drive]) -> Option<&'a Drive> {
    if let Some(drive) = drives.iter().find(|drive| drive.path == target) {
        return Some(drive);
    }
    let target_path = Path::new(target);
    let block = blocks.iter().find(|block| {
        block.path == target || block.device == target || block.mount_points.iter().any(|mount| mount == target_path)
    })?;
    let drive = drive_of(block, blocks)?;
    drives.iter().find(|candidate| candidate.path == drive)
}

/// One step of ejecting a drive, in the order [`eject_plan`] lists them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Unmount the filesystem on this block.
    Unmount(String),
    /// Lock the LUKS volume on this block, once its cleartext side is unmounted.
    Lock(String),
    /// Power the drive down, which a USB stick needs to be safe to pull.
    PowerOff(String),
    /// Eject the drive's medium.
    Eject(String),
}

/// What ejecting `drive` takes: unmount every filesystem on it, lock every LUKS volume on it that
/// is open, then power it down if it can be, else eject its medium if it can be.
pub fn eject_plan(drive: &Drive, blocks: &[Block]) -> Vec<Step> {
    let on_drive = |block: &&Block| drive_of(block, blocks) == Some(drive.path.as_str());
    let mut steps: Vec<Step> = blocks
        .iter()
        .filter(on_drive)
        .filter(|block| !block.mount_points.is_empty())
        .map(|block| Step::Unmount(block.path.clone()))
        .collect();
    steps.extend(
        blocks
            .iter()
            .filter(on_drive)
            .filter_map(|block| block.crypto_backing.clone())
            .map(Step::Lock),
    );
    if drive.can_power_off {
        steps.push(Step::PowerOff(drive.path.clone()));
    } else if drive.ejectable {
        steps.push(Step::Eject(drive.path.clone()));
    }
    steps
}

/// The drive `block` is on, through its locked side for a LUKS volume's cleartext side.
fn drive_of<'a>(block: &'a Block, blocks: &'a [Block]) -> Option<&'a str> {
    if let Some(drive) = &block.drive {
        return Some(drive);
    }
    let backing = block.crypto_backing.as_deref()?;
    blocks
        .iter()
        .find(|candidate| candidate.path == backing)
        .and_then(|candidate| candidate.drive.as_deref())
}

/// What the user calls the filesystem on `block`: its label, else its drive's name, else its
/// device file.
pub fn block_name(block: &Block, drive: Option<&Drive>) -> String {
    let label = block.label.trim();
    if !label.is_empty() {
        return label.to_owned();
    }
    drive
        .and_then(drive_name)
        .unwrap_or_else(|| block.device.clone())
}

/// A drive's vendor and model, when it reports either.
pub fn drive_name(drive: &Drive) -> Option<String> {
    let product = format!("{} {}", drive.vendor.trim(), drive.model.trim());
    let product = product.trim();
    (!product.is_empty()).then(|| product.to_owned())
}

#[cfg(test)]
mod tests;
