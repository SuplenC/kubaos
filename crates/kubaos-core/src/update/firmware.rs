//! Firmware through fwupd and the LVFS: what `fwupdmgr get-updates --json` says is waiting, which
//! of it can install now and which waits for a reboot, and what the firmware phase of an update
//! did.
//!
//! # Owns
//!
//! - Reading fwupd's list ([`parse_updates`]) and the phase's result ([`Firmware`]), one
//!   [`FirmwareChange`] a device.
//!
//! # Not here
//!
//! - Running fwupdmgr, asking and reporting: the updater's `firmware.rs`. The switch that turns the
//!   phase off: `update::programs`.

use serde::Deserialize;
use serde::Serialize;

/// The device flags with which fwupd says an update applies only after a reboot or a shutdown.
const REBOOT_FLAGS: [&str; 2] = ["needs-reboot", "needs-shutdown"];

/// What fwupdmgr's exit code 2 means: nothing to do, as "no updates available" or "metadata is
/// up to date".
pub const NOTHING_TO_DO: i32 = 2;

/// One device's firmware update.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirmwareUpdate {
    /// The device as fwupd names it, "UEFI dbx".
    pub device: String,
    /// fwupd's id of the device, which `fwupdmgr update` takes.
    pub id: String,
    /// The version installed; empty when fwupd doesn't know it.
    pub from: String,
    /// The version the update installs.
    pub to: String,
    /// Whether it applies only at the next reboot or shutdown.
    pub reboot: bool,
}

impl FirmwareUpdate {
    /// "UEFI dbx 20250507 → 20260707", or "UEFI dbx → 20260707" without an installed version.
    pub fn describe(&self) -> String {
        if self.from.is_empty() {
            format!("{} → {}", self.device, self.to)
        } else {
            format!("{} {} → {}", self.device, self.from, self.to)
        }
    }
}

/// What became of one firmware update.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FirmwareOutcome {
    /// Installed now: it needed no reboot.
    Applied,
    /// Handed to fwupd, which installs it at the next reboot.
    Staged,
    /// It waits for a reboot and isn't staged: the user said no, or nobody could be asked.
    Waiting,
    /// fwupd refused or failed to install it; the run's warnings say why.
    Failed,
}

/// One firmware update and what became of it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirmwareChange {
    #[serde(flatten)]
    pub update: FirmwareUpdate,
    pub outcome: FirmwareOutcome,
}

/// What the firmware phase did, one change a device, in fwupd's order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Firmware(Vec<FirmwareChange>);

impl Firmware {
    /// Records that `update` ended as `outcome`.
    pub fn push(&mut self, update: FirmwareUpdate, outcome: FirmwareOutcome) {
        self.0.push(FirmwareChange { update, outcome });
    }

    /// Every change.
    pub fn changes(&self) -> &[FirmwareChange] {
        &self.0
    }

    /// The updates that ended as `outcome`.
    pub fn with(&self, outcome: FirmwareOutcome) -> impl Iterator<Item = &FirmwareUpdate> {
        self.0.iter().filter(move |c| c.outcome == outcome).map(|c| &c.update)
    }

    /// Whether the phase changed nothing and found nothing waiting.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The phase's result in a few words.
    pub fn detail(&self) -> String {
        let words = [
            (FirmwareOutcome::Applied, "applied"),
            (FirmwareOutcome::Staged, "staged for the reboot"),
            (FirmwareOutcome::Waiting, "waiting for a reboot"),
            (FirmwareOutcome::Failed, "failed"),
        ];
        let parts: Vec<String> = words
            .into_iter()
            .map(|(outcome, word)| (self.with(outcome).count(), word))
            .filter(|(n, _)| *n > 0)
            .map(|(n, word)| format!("{n} {word}"))
            .collect();
        if parts.is_empty() {
            "nothing waiting".to_owned()
        } else {
            parts.join(", ")
        }
    }
}

/// `fwupdmgr get-updates --json` printed something that isn't its list.
#[derive(Debug, thiserror::Error)]
#[error("reading fwupdmgr's list of updates")]
pub struct ParseError(#[source] serde_json::Error);

/// The updates `fwupdmgr get-updates --json` lists, one per device, each to the device's newest
/// release, which is the one `fwupdmgr update` installs. Empty output lists none.
///
/// # Errors
///
/// [`ParseError`] when the output is not fwupd's JSON.
pub fn parse_updates(json: &str) -> Result<Vec<FirmwareUpdate>, ParseError> {
    if json.trim().is_empty() {
        return Ok(Vec::new());
    }
    let list: List = serde_json::from_str(json).map_err(ParseError)?;
    Ok(list
        .devices
        .into_iter()
        .filter_map(|device| {
            // fwupd lists a device's releases newest first.
            let release = device.releases.into_iter().next()?;
            Some(FirmwareUpdate {
                reboot: device.flags.iter().any(|f| REBOOT_FLAGS.contains(&f.as_str())),
                device: device.name,
                id: device.id,
                from: device.version.unwrap_or_default(),
                to: release.version,
            })
        })
        .collect())
}

/// The part of fwupd's JSON the phase reads.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct List {
    #[serde(default)]
    devices: Vec<Device>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Device {
    name: String,
    #[serde(rename = "DeviceId")]
    id: String,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    flags: Vec<String>,
    #[serde(default)]
    releases: Vec<Release>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Release {
    version: String,
}

#[cfg(test)]
mod tests;
