use std::io;

use super::Outcome;
use super::PackageChange;
use super::Phase;
use super::ProgramResult;
use super::Prompt;
use super::Restart;
use super::UpdateEvent;
use super::chain;
use crate::update::firmware::FirmwareOutcome;

/// A summary with every field set, as a front end reads it.
const SUMMARY: &str = include_str!("summary.json");

#[test]
fn an_event_is_one_json_object_tagged_with_its_type() {
    let finished = UpdateEvent::PhaseFinished {
        phase: Phase::SystemPackages,
        outcome: Outcome::Done,
        elapsed_ms: 1200,
        detail: Some("12 packages".into()),
    };
    let prompt = UpdateEvent::Prompt {
        prompt: Prompt::Confirm {
            question: "Update now?".into(),
        },
    };

    assert_eq!(
        finished.to_json_line(),
        r#"{"type":"phase_finished","phase":"system_packages","outcome":"done","elapsed_ms":1200,"detail":"12 packages"}"#
    );
    assert_eq!(
        prompt.to_json_line(),
        r#"{"type":"prompt","prompt":{"kind":"confirm","question":"Update now?"}}"#
    );
    assert_eq!(UpdateEvent::Declined.to_json_line(), r#"{"type":"declined"}"#);
}

#[test]
fn events_read_back_from_their_json() {
    let events = [
        UpdateEvent::Output {
            line: "Unpacking vim".into(),
        },
        UpdateEvent::Prompt {
            prompt: Prompt::Password,
        },
        UpdateEvent::Failed {
            phase: Some(Phase::Home),
            error: "switching your home".into(),
            output: vec!["error: builder failed".into()],
            log: Some("/tmp/u.log".into()),
        },
    ];

    for event in events {
        assert_eq!(UpdateEvent::from_json_line(&event.to_json_line()).unwrap(), event);
    }
}

#[test]
fn the_chain_names_each_cause_once() {
    #[derive(Debug, thiserror::Error)]
    #[error("upgrading apt packages")]
    struct Outer(#[source] io::Error);

    let e = Outer(io::Error::other("exit 100"));

    assert_eq!(chain(&e), "upgrading apt packages: exit 100");
}

#[test]
fn the_summary_fixture_reads_with_every_list_a_front_end_shows() {
    let line: serde_json::Value = serde_json::from_str(SUMMARY).unwrap();

    let UpdateEvent::Summary(summary) = UpdateEvent::from_json_line(&line.to_string()).unwrap() else {
        panic!("the fixture is a summary");
    };

    assert_eq!(
        (summary.from.version.as_str(), summary.to.version.as_str()),
        ("0.1.0", "0.1.1")
    );
    assert_eq!(
        summary.upgraded[1].describe(),
        "linux-image-6.8.0-60-generic 6.8.0-60.63, new"
    );
    assert_eq!(
        summary.nix_packages.as_deref().unwrap()[2],
        PackageChange {
            name: "old-tool".into(),
            from: Some("2.1".into()),
            to: None
        }
    );
    assert_eq!(
        summary.programs[0].result,
        ProgramResult::Updated {
            from: "2026.10.1".into(),
            to: "2026.12.0".into()
        }
    );
    assert_eq!(summary.programs[0].waiting.as_deref(), Some("2027.1.0"));
    assert_eq!(summary.firmware.with(FirmwareOutcome::Staged).count(), 1);
    assert_eq!(summary.restart, Restart::Reboot);
}

#[test]
fn the_summary_writes_back_the_json_it_was_read_from() {
    let fixture: serde_json::Value = serde_json::from_str(SUMMARY).unwrap();
    let event = UpdateEvent::from_json_line(&fixture.to_string()).unwrap();

    let written: serde_json::Value = serde_json::from_str(&event.to_json_line()).unwrap();

    assert_eq!(written, fixture);
}

#[test]
fn a_summary_without_the_package_lists_still_reads_and_leaves_them_unknown() {
    let line = r#"{"type":"summary","command":"update","from":{"version":"0.1.0","commit":"a"},"to":{"version":"0.1.0","commit":"a"},"channel":"stable","changes":[],"packages":0,"inputs":[],"migrations":[],"reloaded":[],"restart":"none"}"#;

    let UpdateEvent::Summary(summary) = UpdateEvent::from_json_line(line).unwrap() else {
        panic!("the line is a summary");
    };

    assert!(summary.upgraded.is_empty());
    assert_eq!(summary.nix_packages, None);
    assert_eq!(UpdateEvent::Summary(summary).to_json_line(), line);
}
