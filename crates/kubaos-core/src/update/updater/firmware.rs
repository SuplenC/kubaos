//! The firmware phase of an update: fwupd's metadata refreshed from the LVFS, the updates that
//! need no reboot installed, and those that do staged for the next boot when the user says yes.
//!
//! The phase never stops the update and never reboots. A failure is a warning, and the phase ends
//! skipped. It doesn't start when the user turned it off, nor on a machine without fwupd, which
//! is a warning.
//!
//! # Owns
//!
//! - Running fwupdmgr for an update, and asking before staging.
//!
//! # Not here
//!
//! - fwupd's JSON and the phase's result: `update/firmware.rs`. The switch: `update/programs.rs`.

use super::Updater;
use crate::update::event::Phase;
use crate::update::event::chain;
use crate::update::firmware;
use crate::update::firmware::Firmware;
use crate::update::firmware::FirmwareOutcome;
use crate::update::firmware::FirmwareUpdate;
use crate::update::firmware::NOTHING_TO_DO;
use crate::update::machine::Cmd;
use crate::update::machine::Machine;
use crate::update::machine::MachineError;
use crate::update::programs::Part;
use crate::update::programs::Switches;

/// fwupdmgr on the machine, which the fwupd package installs.
const FWUPDMGR: &str = "usr/bin/fwupdmgr";

/// The options of every fwupdmgr call. With `--json` fwupdmgr asks nothing (its `--help`:
/// "disables all interactive prompts"), and `--no-reboot-check` keeps it from checking for or
/// offering a reboot.
///
/// HAZARD: never add `-y`. It answers yes to every question fwupdmgr has, a reboot and the upload
/// of a report to the LVFS among them.
const QUIET: [&str; 4] = [
    "--json",
    "--no-unreported-check",
    "--no-metadata-check",
    "--no-reboot-check",
];

/// Whether the phase may stage the updates that wait for a reboot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Staging {
    /// Ask, and stage on a yes.
    Ask,
    /// Only report them: nobody is there to answer, as with `--json` or `-y`.
    Report,
}

impl<M: Machine> Updater<'_, M> {
    /// Runs the firmware phase, unless the user turned it off or the machine has no fwupd; what it
    /// did.
    pub(super) fn update_firmware(&mut self, staging: Staging) -> Firmware {
        if !Switches::new(self.user.config_home()).is_on(Part::Firmware) {
            return Firmware::default();
        }
        if !self.sys.root().join(FWUPDMGR).is_file() {
            self.j
                .warn("fwupd isn't installed, so no firmware updates: sudo apt install fwupd");
            return Firmware::default();
        }
        self.phase(Phase::Firmware, |u| Ok(u.try_firmware(staging)))
            .unwrap_or_default()
    }

    fn try_firmware(&mut self, staging: Staging) -> Firmware {
        self.refresh_firmware_metadata();
        let Some(pending) = self.pending_firmware() else {
            return Firmware::default();
        };
        let mut done = Firmware::default();
        let (later, now): (Vec<FirmwareUpdate>, Vec<FirmwareUpdate>) = pending.into_iter().partition(|u| u.reboot);
        for update in now {
            let outcome = self.install_firmware(&update, FirmwareOutcome::Applied);
            done.push(update, outcome);
        }
        if !later.is_empty() {
            self.stage_or_wait(later, staging, &mut done);
        }
        self.j.detail(done.detail());
        done
    }

    /// `fwupdmgr refresh`. A failure leaves the metadata fwupd has, which the daily
    /// `fwupd-refresh.timer` keeps recent.
    fn refresh_firmware_metadata(&mut self) {
        self.j.step("Refreshing the firmware metadata from the LVFS");
        let refresh = Cmd::root("fwupdmgr").arg("refresh").args(QUIET);
        match self.quiet(&refresh) {
            Ok(_) => {}
            Err(e) if exited_with(&e, NOTHING_TO_DO) => {}
            Err(e) => self.j.warn(format!(
                "refreshing the firmware metadata: {}; listing from what fwupd has",
                chain(&e)
            )),
        }
    }

    /// The updates fwupd lists; `None`, with a warning and the phase skipped, when it can't say.
    fn pending_firmware(&mut self) -> Option<Vec<FirmwareUpdate>> {
        self.j.step("Listing the firmware updates");
        let list = Cmd::user("fwupdmgr").arg("get-updates").args(QUIET);
        let json = match self.quiet(&list) {
            Ok(json) => json,
            Err(e) if exited_with(&e, NOTHING_TO_DO) => String::new(),
            Err(e) => {
                self.j.warn(format!("listing the firmware updates: {}", chain(&e)));
                self.j.skip("fwupd couldn't list the updates");
                return None;
            }
        };
        match firmware::parse_updates(&json) {
            Ok(pending) => Some(pending),
            Err(e) => {
                self.j.warn(chain(&e));
                self.j.skip("fwupd's list didn't read");
                None
            }
        }
    }

    /// Asks before staging the updates that wait for a reboot, when someone can answer; the rest
    /// wait, named in a note.
    fn stage_or_wait(&mut self, later: Vec<FirmwareUpdate>, staging: Staging, done: &mut Firmware) {
        let names: Vec<String> = later.iter().map(FirmwareUpdate::describe).collect();
        self.j.note(format!(
            "Firmware that installs at the next reboot: {}",
            names.join(", ")
        ));
        let stage = staging == Staging::Ask
            && self.confirm("Stage it now? fwupd installs it at the next reboot; nothing reboots now");
        if !stage {
            self.j
                .note("It waits: kubaos update in a terminal offers it again, or sudo fwupdmgr update");
            for update in later {
                done.push(update, FirmwareOutcome::Waiting);
            }
            return;
        }
        for update in later {
            let outcome = self.install_firmware(&update, FirmwareOutcome::Staged);
            done.push(update, outcome);
        }
    }

    /// `fwupdmgr update` of one device, as root: `done` when it went through, else failed. For a
    /// device that needs a reboot fwupd only schedules the update, since the reboot check is off.
    fn install_firmware(&mut self, update: &FirmwareUpdate, done: FirmwareOutcome) -> FirmwareOutcome {
        self.j.step(format!("Updating the firmware of {}", update.describe()));
        let cmd = Cmd::root("fwupdmgr").arg("update").arg(&update.id).args(QUIET);
        match self.run(&cmd) {
            Ok(()) => done,
            Err(e) => {
                self.j
                    .warn(format!("updating the firmware of {}: {}", update.device, chain(&e)));
                FirmwareOutcome::Failed
            }
        }
    }
}

/// Whether `e` is a program that exited with `code`.
fn exited_with(e: &MachineError, code: i32) -> bool {
    matches!(e, MachineError::Exited { code: Some(c), .. } if *c == code)
}
