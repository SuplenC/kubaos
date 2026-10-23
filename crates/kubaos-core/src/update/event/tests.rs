use std::io;

use super::Outcome;
use super::Phase;
use super::Prompt;
use super::UpdateEvent;
use super::chain;

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
