use futures_lite::future;

use super::*;
use crate::fake::Recorder;
use crate::fake::keyboard;

#[test]
fn an_answer_reaches_the_waiter_and_closes_the_prompt() {
    let hub = Hub::new(Recorder::default());
    let (id, reply) = hub.ask(Kind::PinCode, &keyboard(), None);
    assert_eq!(hub.state().prompt.unwrap().id, id);

    let closed = hub.answer(id, Answer::accept("0000")).expect("answer the open prompt");
    assert_eq!(closed.kind, Kind::PinCode);
    assert_eq!(future::block_on(reply.recv()).unwrap(), Answer::accept("0000"));
    assert_eq!(hub.state().prompt, None);
}

#[test]
fn a_late_answer_to_an_old_prompt_is_refused() {
    let hub = Hub::new(Recorder::default());
    let (old, old_reply) = hub.ask(Kind::Confirm, &keyboard(), Some("123456".into()));
    let (new, _new_reply) = hub.ask(Kind::Confirm, &keyboard(), Some("654321".into()));

    assert_ne!(old, new);
    assert!(hub.answer(old, Answer::accept("")).is_err());
    assert!(
        future::block_on(old_reply.recv()).is_err(),
        "the replaced prompt's waiter hears it was cancelled"
    );
    assert_eq!(hub.state().prompt.unwrap().id, new);
}

#[test]
fn dismissing_tells_the_waiter() {
    let hub = Hub::new(Recorder::default());
    let (_, reply) = hub.ask(Kind::Passkey, &keyboard(), None);
    hub.dismiss();
    assert!(future::block_on(reply.recv()).is_err());
    assert_eq!(hub.state().prompt, None);
}

#[test]
fn showing_the_same_passkey_again_only_updates_the_digits_typed() {
    let hub = Hub::new(Recorder::default());
    let first = hub.show(Kind::DisplayPasskey, &keyboard(), "012345".into(), Some(0));
    let again = hub.show(Kind::DisplayPasskey, &keyboard(), "012345".into(), Some(3));

    assert_eq!(first, again);
    assert_eq!(hub.state().prompt.unwrap().entered, Some(3));
}

#[test]
fn ending_a_pairing_keeps_its_failure_and_closes_its_prompt() {
    let hub = Hub::new(Recorder::default());
    let device = keyboard();
    assert!(hub.begin_pairing(&device.path));
    assert!(
        !hub.begin_pairing(&device.path),
        "a second pairing of the same device is refused"
    );
    hub.show(Kind::DisplayPasskey, &device, "012345".into(), None);

    hub.end_pairing(&device.path, Err("Timed out".into()));

    let state = hub.state();
    assert!(state.pairing.is_empty());
    assert_eq!(state.prompt, None);
    assert_eq!(state.errors.get(&device.path).map(String::as_str), Some("Timed out"));
    assert!(hub.begin_pairing(&device.path));
    assert!(hub.state().errors.is_empty(), "a new attempt forgets the last failure");
}

#[test]
fn every_change_is_published_once_and_nothing_else_is() {
    let hub = Hub::new(Recorder::default());
    hub.set_agent(true);
    hub.set_agent(true);
    hub.clear_error("/org/bluez/hci0/dev_none");
    let (id, _) = hub.ask(Kind::Authorize, &keyboard(), None);
    hub.close(id);
    hub.close(id);

    let published = hub.sink().states();
    assert_eq!(published.len(), 3, "{published:#?}");
    assert!(published.iter().all(|state| state.agent));
    assert_eq!(published.last().unwrap().prompt, None);
}

#[test]
fn losing_the_agent_closes_the_prompt() {
    let hub = Hub::new(Recorder::default());
    hub.set_agent(true);
    let (_, reply) = hub.ask(Kind::Confirm, &keyboard(), Some("123456".into()));
    hub.set_agent(false);
    assert!(future::block_on(reply.recv()).is_err());
    assert_eq!(hub.state().prompt, None);
}

#[test]
fn a_signal_is_published_when_it_moves_past_the_jitter() {
    let hub = Hub::new(Recorder::default());
    let path = keyboard().path;

    hub.heard(&path, -60);
    hub.heard(&path, -60);
    hub.heard(&path, -63);
    hub.heard(&path, -72);

    let heard: Vec<_> = hub
        .sink()
        .states()
        .iter()
        .map(|state| state.rssi.get(&path).copied())
        .collect();
    assert_eq!(heard, [Some(-60), Some(-72)]);
}

#[test]
fn a_device_that_goes_takes_its_signal_with_it() {
    let hub = Hub::new(Recorder::default());
    let path = keyboard().path;
    hub.heard(&path, -60);

    hub.gone(&path);

    assert!(hub.state().rssi.is_empty());
}
