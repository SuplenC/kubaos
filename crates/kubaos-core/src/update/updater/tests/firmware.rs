use std::fs;

use super::Fake;
use super::World;
use super::notes;
use super::phases;
use super::summary;
use super::warnings;
use super::yes;
use crate::update::Confirm;
use crate::update::UpdateOptions;
use crate::update::event::Outcome;
use crate::update::event::Phase;
use crate::update::event::Restart;
use crate::update::event::UpdateEvent;
use crate::update::firmware::Firmware;
use crate::update::firmware::FirmwareOutcome;
use crate::update::programs::Part;
use crate::update::programs::Switches;
use crate::update::record::Channel;

/// A dock and a receiver that update at once, and system firmware that needs a reboot.
const MIXED: &str = include_str!("../../firmware/mixed.json");

const DOCK: &str = "4d3b0f2c9a1e7b6a5c8d9e0f1a2b3c4d5e6f7a8b";
const RECEIVER: &str = "9f8e7d6c5b4a39281706f5e4d3c2b1a098765432";
const SYSTEM: &str = "a45df35ac0e948ee180fe216a5f703f32dda163f";

/// A world whose machine has fwupd.
fn with_fwupd() -> World {
    let w = World::new(Channel::Edge, true);
    let fwupdmgr = w.sys.root().join("usr/bin/fwupdmgr");
    fs::create_dir_all(fwupdmgr.parent().unwrap()).unwrap();
    fs::write(&fwupdmgr, "").unwrap();
    w
}

/// A fake whose fwupd lists `json`.
fn listing(json: &'static str) -> Fake {
    let mut m = Fake::new();
    m.answers.push(("fwupdmgr get-updates", 0, json));
    m
}

/// An update asked at the terminal, which may stage firmware.
fn at_the_terminal() -> UpdateOptions {
    UpdateOptions::new().confirm(Confirm::Ask).stage_firmware()
}

fn finished(events: &[UpdateEvent], phase: Phase) -> Option<(Outcome, Option<String>)> {
    events.iter().find_map(|e| match e {
        UpdateEvent::PhaseFinished {
            phase: p,
            outcome,
            detail,
            ..
        } if *p == phase => Some((*outcome, detail.clone())),
        _ => None,
    })
}

/// The devices' ids whose update ended as `outcome`.
fn ids(firmware: &Firmware, outcome: FirmwareOutcome) -> Vec<&str> {
    firmware.with(outcome).map(|u| u.id.as_str()).collect()
}

fn updated(m: &Fake, id: &str) -> bool {
    m.ran.iter().any(|line| {
        *line == format!("sudo fwupdmgr update {id} --json --no-unreported-check --no-metadata-check --no-reboot-check")
    })
}

#[test]
fn without_fwupd_the_update_warns_and_runs_no_firmware_phase() {
    let w = World::new(Channel::Edge, true);
    let mut m = Fake::new();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert!(!phases(&events).contains(&Phase::Firmware));
    assert!(warnings(&events).iter().any(|w| w.contains("fwupd isn't installed")));
    assert_eq!(m.ran("sudo fwupdmgr") + m.ran("fwupdmgr"), 0);
}

#[test]
fn turned_off_the_update_leaves_firmware_alone() {
    let w = with_fwupd();
    Switches::new(w.user.config_home()).turn_off(Part::Firmware).unwrap();
    let mut m = listing(MIXED);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(at_the_terminal()).unwrap();

    assert!(!phases(&events).contains(&Phase::Firmware));
    assert_eq!(m.ran("sudo fwupdmgr") + m.ran("fwupdmgr"), 0);
    assert!(summary(&events).firmware.is_empty());
}

#[test]
fn nothing_waiting_is_a_done_phase_whatever_exit_code_fwupd_says_it_with() {
    let w = with_fwupd();
    let mut m = Fake::new();
    m.answers.push(("sudo fwupdmgr refresh", 2, ""));
    m.answers.push(("fwupdmgr get-updates", 2, "No updates available\n"));
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert_eq!(
        finished(&events, Phase::Firmware),
        Some((Outcome::Done, Some("nothing waiting".to_owned())))
    );
    let warned = warnings(&events);
    assert!(!warned.iter().any(|w| w.contains("firmware")), "{warned:?}");
    assert_eq!(summary(&events).restart, Restart::None);
}

#[test]
fn at_the_terminal_it_applies_what_needs_no_reboot_and_stages_the_rest_on_a_yes() {
    let w = with_fwupd();
    let mut m = listing(MIXED);
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(at_the_terminal()).unwrap();

    assert!(updated(&m, DOCK) && updated(&m, RECEIVER) && updated(&m, SYSTEM));
    assert_eq!(m.asked, 2, "the update, then the staging");
    let refresh = m
        .ran
        .iter()
        .position(|l| l.starts_with("sudo fwupdmgr refresh"))
        .unwrap();
    let list = m
        .ran
        .iter()
        .position(|l| l.starts_with("fwupdmgr get-updates"))
        .unwrap();
    assert!(refresh < list, "the metadata is refreshed before the list");
    assert!(
        !m.ran
            .iter()
            .any(|l| l.contains("reboot") && !l.contains("--no-reboot-check"))
    );
    let summary = summary(&events);
    assert_eq!(ids(&summary.firmware, FirmwareOutcome::Applied), [DOCK, RECEIVER]);
    assert_eq!(ids(&summary.firmware, FirmwareOutcome::Staged), [SYSTEM]);
    assert_eq!(summary.firmware.changes().len(), 3);
    assert_eq!(
        (summary.restart, summary.restart_reason.as_deref()),
        (Restart::Reboot, Some("firmware is staged to install at it"))
    );
    assert_eq!(
        finished(&events, Phase::Firmware),
        Some((Outcome::Done, Some("2 applied, 1 staged for the reboot".to_owned())))
    );
}

#[test]
fn a_no_leaves_the_reboot_firmware_waiting() {
    let w = with_fwupd();
    let mut m = listing(MIXED);
    m.no_to = Some("Stage it now?");
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(at_the_terminal()).unwrap();

    assert!(updated(&m, DOCK) && !updated(&m, SYSTEM));
    let summary = summary(&events);
    assert_eq!(ids(&summary.firmware, FirmwareOutcome::Waiting), [SYSTEM]);
    assert!(ids(&summary.firmware, FirmwareOutcome::Staged).is_empty());
    assert_eq!(summary.restart, Restart::None);
    assert!(
        notes(&events)
            .iter()
            .any(|n| n.contains("System Firmware 0.1.2 → 0.1.4"))
    );
}

#[test]
fn with_nobody_to_ask_the_reboot_firmware_is_only_reported() {
    for opts in [yes().stage_firmware(), UpdateOptions::new().confirm(Confirm::Ask)] {
        let w = with_fwupd();
        let mut m = listing(MIXED);
        let mut events = Vec::new();

        w.reporting(&mut m, &mut events).update(opts).unwrap();

        assert!(updated(&m, DOCK) && updated(&m, RECEIVER));
        assert!(!updated(&m, SYSTEM), "{opts:?} staged it");
        assert!(events.iter().all(|e| !matches!(
            e,
            UpdateEvent::Prompt {
                prompt: crate::update::event::Prompt::Confirm { question }
            } if question.starts_with("Stage")
        )));
        let firmware = &summary(&events).firmware;
        assert_eq!(ids(firmware, FirmwareOutcome::Waiting), [SYSTEM]);
        assert_eq!(ids(firmware, FirmwareOutcome::Applied), [DOCK, RECEIVER]);
    }
}

#[test]
fn a_failing_device_warns_and_the_update_goes_on() {
    let w = with_fwupd();
    let mut m = listing(MIXED);
    m.answers.insert(0, ("sudo fwupdmgr update 4d3b0f2c", 1, ""));
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    let firmware = &summary(&events).firmware;
    assert_eq!(ids(firmware, FirmwareOutcome::Failed), [DOCK]);
    assert_eq!(ids(firmware, FirmwareOutcome::Applied), [RECEIVER]);
    assert!(
        warnings(&events)
            .iter()
            .any(|w| w.starts_with("updating the firmware of USB-C Dock"))
    );
    assert_eq!(
        finished(&events, Phase::Firmware).map(|(outcome, _)| outcome),
        Some(Outcome::Done)
    );
}

#[test]
fn fwupd_failing_to_list_skips_the_phase_with_a_warning() {
    let w = with_fwupd();
    let mut m = Fake::new();
    m.answers.push(("sudo fwupdmgr refresh", 1, ""));
    m.answers.push(("fwupdmgr get-updates", 1, ""));
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert_eq!(
        finished(&events, Phase::Firmware),
        Some((Outcome::Skipped, Some("fwupd couldn't list the updates".to_owned())))
    );
    let warned = warnings(&events);
    assert!(warned.iter().any(|w| w.starts_with("refreshing the firmware metadata")));
    assert!(warned.iter().any(|w| w.starts_with("listing the firmware updates")));
    assert!(summary(&events).firmware.is_empty());
}

#[test]
fn a_list_that_doesnt_read_skips_the_phase_with_a_warning() {
    let w = with_fwupd();
    let mut m = listing("Devices: none");
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes()).unwrap();

    assert_eq!(
        finished(&events, Phase::Firmware).map(|(outcome, _)| outcome),
        Some(Outcome::Skipped)
    );
    assert!(
        warnings(&events)
            .iter()
            .any(|w| w.starts_with("reading fwupdmgr's list"))
    );
}

#[test]
fn firmware_comes_last_and_runs_on_a_packages_only_update_too() {
    let w = with_fwupd();
    let mut m = Fake::new();
    let mut events = Vec::new();

    w.reporting(&mut m, &mut events).update(yes().packages_only()).unwrap();

    assert_eq!(phases(&events).last(), Some(&Phase::Firmware));
}
