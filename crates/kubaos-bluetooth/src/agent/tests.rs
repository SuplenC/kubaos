use std::time::Instant;

use futures_lite::future;

use super::*;
use crate::fake::Recorder;
use crate::fake::keyboard;
use crate::hub::Answer;

fn fresh() -> (Agent<Recorder>, Arc<Hub<Recorder>>) {
    let hub = Arc::new(Hub::new(Recorder::default()));
    (Agent::new(Arc::clone(&hub)), hub)
}

/// Runs `request` while the user answers its prompt with `answer`, after checking the prompt.
fn answered<T>(
    hub: &Hub<Recorder>,
    request: impl Future<Output = T>,
    answer: Answer,
    check: impl FnOnce(&crate::state::Prompt),
) -> T {
    let user = async {
        let prompt = hub.state().prompt.expect("the request opened a prompt");
        check(&prompt);
        hub.answer(prompt.id, answer).expect("answer the prompt");
    };
    future::block_on(future::zip(request, user)).0
}

#[test]
fn a_passkey_typed_in_the_shell_goes_to_bluetoothd() {
    let (agent, hub) = fresh();
    let passkey = answered(
        &hub,
        agent.request_passkey(&keyboard()),
        Answer::accept(" 012345 "),
        |prompt| {
            assert_eq!(prompt.kind, Kind::Passkey);
            assert_eq!(prompt.name, "K380");
        },
    );
    assert_eq!(passkey, Ok(12_345));
    assert_eq!(hub.state().prompt, None);
}

#[test]
fn a_passkey_that_isnt_six_digits_is_rejected() {
    for typed in ["", "1234567", "12a456", "-1"] {
        let (agent, hub) = fresh();
        let passkey = answered(
            &hub,
            agent.request_passkey(&keyboard()),
            Answer::accept(typed),
            |_| {},
        );
        assert_eq!(passkey, Err(Refusal::Rejected), "{typed:?}");
    }
}

#[test]
fn a_pin_is_one_to_sixteen_letters_or_digits() {
    let (agent, hub) = fresh();
    let pin = answered(
        &hub,
        agent.request_pin_code(&keyboard()),
        Answer::accept("0000"),
        |prompt| {
            assert_eq!(prompt.kind, Kind::PinCode);
        },
    );
    assert_eq!(pin.as_deref(), Ok("0000"));

    for typed in ["", "12345678901234567", "12 34"] {
        let (agent, hub) = fresh();
        let pin = answered(
            &hub,
            agent.request_pin_code(&keyboard()),
            Answer::accept(typed),
            |_| {},
        );
        assert_eq!(pin, Err(Refusal::Rejected), "{typed:?}");
    }
}

#[test]
fn confirmation_shows_the_padded_passkey_and_takes_pair_or_cancel() {
    let (agent, hub) = fresh();
    let confirmed = answered(
        &hub,
        agent.request_confirmation(&keyboard(), 42),
        Answer::accept(""),
        |prompt| {
            assert_eq!(prompt.kind, Kind::Confirm);
            assert_eq!(prompt.code.as_deref(), Some("000042"));
        },
    );
    assert_eq!(confirmed, Ok(()));

    let declined = answered(
        &hub,
        agent.request_confirmation(&keyboard(), 42),
        Answer::reject(),
        |_| {},
    );
    assert_eq!(declined, Err(Refusal::Rejected));
}

#[test]
fn an_unanswered_prompt_times_out_as_cancelled_and_goes() {
    let (agent, hub) = fresh();
    let agent = agent.timeout(Duration::from_millis(30));
    let started = Instant::now();

    let result = future::block_on(agent.request_confirmation(&keyboard(), 1));

    assert_eq!(result, Err(Refusal::Canceled));
    assert!(started.elapsed() >= Duration::from_millis(30));
    assert_eq!(hub.state().prompt, None);
}

#[test]
fn bluetoothds_cancel_ends_the_wait() {
    let (agent, hub) = fresh();
    let cancel = async { agent.cancel() };
    let (result, ()) = future::block_on(future::zip(agent.request_passkey(&keyboard()), cancel));
    assert_eq!(result, Err(Refusal::Canceled));
    assert_eq!(hub.state().prompt, None);
}

#[test]
fn the_code_to_type_stays_shown_until_cancel() {
    let (agent, hub) = fresh();
    agent.display_passkey(&keyboard(), 7, 0);
    agent.display_passkey(&keyboard(), 7, 4);

    let prompt = hub.state().prompt.expect("the passkey is shown");
    assert_eq!(prompt.kind, Kind::DisplayPasskey);
    assert_eq!(prompt.code.as_deref(), Some("000007"));
    assert_eq!(prompt.entered, Some(4));

    agent.cancel();
    assert_eq!(hub.state().prompt, None);

    agent.display_pin_code(&keyboard(), "482913");
    assert_eq!(hub.state().prompt.unwrap().kind, Kind::DisplayPinCode);
}

#[test]
fn a_device_being_paired_needs_no_second_yes() {
    let (agent, hub) = fresh();
    let device = keyboard();
    hub.begin_pairing(&device.path);
    assert_eq!(
        future::block_on(agent.request_authorization(&device)),
        Ok(())
    );
    assert_eq!(hub.state().prompt, None, "nothing was asked");
}

#[test]
fn a_device_asking_to_pair_on_its_own_is_asked_about() {
    let (agent, hub) = fresh();
    let allowed = answered(
        &hub,
        agent.request_authorization(&keyboard()),
        Answer::accept(""),
        |prompt| {
            assert_eq!(prompt.kind, Kind::Authorize);
        },
    );
    assert_eq!(allowed, Ok(()));
}

#[test]
fn services_are_allowed_for_trusted_devices_and_the_one_pairing() {
    let (agent, hub) = fresh();
    let device = keyboard();
    assert_eq!(agent.authorize_service(&device.path, true), Ok(()));
    assert_eq!(
        agent.authorize_service(&device.path, false),
        Err(Refusal::Rejected)
    );
    hub.begin_pairing(&device.path);
    assert_eq!(agent.authorize_service(&device.path, false), Ok(()));
}

#[test]
fn release_marks_the_agent_gone() {
    let (agent, hub) = fresh();
    hub.set_agent(true);
    agent.release();
    assert!(!hub.state().agent);
}
