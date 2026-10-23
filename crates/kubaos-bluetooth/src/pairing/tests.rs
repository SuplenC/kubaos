use futures_lite::future;

use super::*;
use crate::fake::FakeBluez;
use crate::fake::PairReply;
use crate::fake::Recorder;
use crate::fake::keyboard;
use crate::state::Kind;

const SHORT: Duration = Duration::from_millis(20);

fn pairing(
    bluez: FakeBluez,
) -> (
    Pairing<Recorder, FakeBluez>,
    Arc<Hub<Recorder>>,
    Arc<FakeBluez>,
) {
    let hub = Arc::new(Hub::new(Recorder::default()));
    let bluez = Arc::new(bluez);
    let pairing = Pairing::new(Arc::clone(&hub), Arc::clone(&bluez))
        .timeout(SHORT)
        .linger(SHORT);
    (pairing, hub, bluez)
}

fn failing(name: &str, message: &str) -> FakeBluez {
    FakeBluez {
        pair: PairReply::Err(BluezError::new(name, message)),
        ..FakeBluez::default()
    }
}

#[test]
fn a_paired_device_is_trusted_then_connected() {
    let (pairing, hub, bluez) = pairing(FakeBluez::default());
    let device = keyboard().path;

    future::block_on(pairing.pair(&device));

    assert_eq!(
        bluez.calls(),
        [
            format!("Pair {device}"),
            format!("Trust {device}"),
            format!("Connect {device}")
        ]
    );
    let state = hub.state();
    assert!(state.pairing.is_empty());
    assert!(state.errors.is_empty());
}

#[test]
fn the_device_shows_as_pairing_while_pair_runs() {
    let (pairing, hub, _) = pairing(FakeBluez::default());
    let device = keyboard().path;
    future::block_on(pairing.pair(&device));
    let published = hub.sink().states();
    assert!(
        published.first().unwrap().pairing.contains(&device),
        "{published:#?}"
    );
}

#[test]
fn a_hung_pairing_is_cancelled_after_the_timeout() {
    let bluez = FakeBluez {
        pair: PairReply::Never,
        ..FakeBluez::default()
    };
    let (pairing, hub, bluez) = pairing(bluez);
    let device = keyboard().path;

    future::block_on(pairing.pair(&device));

    let calls = bluez.calls();
    assert_eq!(
        calls[..2],
        [format!("Pair {device}"), format!("CancelPairing {device}")]
    );
    let failed = hub
        .sink()
        .states()
        .into_iter()
        .find(|state| !state.errors.is_empty());
    assert_eq!(
        failed
            .expect("the timeout was shown")
            .errors
            .get(&device)
            .map(String::as_str),
        Some(TIMED_OUT)
    );
    assert!(hub.state().pairing.is_empty());
}

#[test]
fn a_failed_device_keeps_its_reason_then_is_forgotten() {
    let (pairing, hub, bluez) = pairing(failing(
        "org.bluez.Error.AuthenticationFailed",
        "Authentication Failed",
    ));
    let device = keyboard().path;
    hub.show(Kind::DisplayPasskey, &keyboard(), "123456".into(), None);

    future::block_on(pairing.pair(&device));

    let shown = hub
        .sink()
        .states()
        .into_iter()
        .find(|state| !state.errors.is_empty())
        .unwrap();
    assert_eq!(
        shown.errors.get(&device).map(String::as_str),
        Some("Wrong code, or the device said no")
    );
    assert_eq!(shown.prompt, None, "the code to type goes with the pairing");
    assert_eq!(
        bluez.calls(),
        [
            format!("Pair {device}"),
            format!("Paired? {device}"),
            format!("RemoveDevice {device}")
        ]
    );
    assert!(
        hub.state().errors.is_empty(),
        "the reason goes with the device"
    );
}

#[test]
fn a_device_paired_meanwhile_isnt_forgotten() {
    let bluez = FakeBluez {
        paired: true,
        ..failing("org.bluez.Error.AuthenticationCanceled", "")
    };
    let (pairing, _, bluez) = pairing(bluez);
    let device = keyboard().path;
    future::block_on(pairing.pair(&device));
    assert!(!bluez.calls().contains(&format!("RemoveDevice {device}")));
}

#[test]
fn a_device_already_paired_counts_as_paired() {
    let (pairing, hub, bluez) = pairing(failing("org.bluez.Error.AlreadyExists", "Already Exists"));
    let device = keyboard().path;
    future::block_on(pairing.pair(&device));
    assert!(bluez.calls().contains(&format!("Connect {device}")));
    assert!(hub.state().errors.is_empty());
}

#[test]
fn a_second_pairing_of_the_same_device_waits_for_the_first() {
    let (pairing, hub, bluez) = pairing(FakeBluez::default());
    let device = keyboard().path;
    hub.begin_pairing(&device);
    future::block_on(pairing.pair(&device));
    assert!(bluez.calls().is_empty());
}

#[test]
fn cancelling_a_pairing_bluetoothd_doesnt_have_ends_it_here() {
    let bluez = FakeBluez {
        cancel: Err(BluezError::new(
            "org.bluez.Error.DoesNotExist",
            "No such pairing",
        )),
        ..FakeBluez::default()
    };
    let (pairing, hub, _) = pairing(bluez);
    let device = keyboard().path;
    hub.begin_pairing(&device);

    future::block_on(pairing.cancel(&device));

    let state = hub.state();
    assert!(state.pairing.is_empty());
    assert_eq!(
        state.errors.get(&device).map(String::as_str),
        Some(CANCELLED)
    );
}

#[test]
fn a_cancel_bluetoothd_takes_leaves_the_ending_to_pair() {
    let (pairing, hub, bluez) = pairing(FakeBluez::default());
    let device = keyboard().path;
    hub.begin_pairing(&device);
    future::block_on(pairing.cancel(&device));
    assert_eq!(bluez.calls(), [format!("CancelPairing {device}")]);
    assert!(hub.is_pairing(&device));
}

#[test]
fn errors_read_as_a_few_words() {
    let cases = [
        ("org.bluez.Error.AuthenticationCanceled", "", CANCELLED),
        (
            "org.bluez.Error.AuthenticationRejected",
            "",
            "The device said no",
        ),
        ("org.bluez.Error.AuthenticationTimeout", "", TIMED_OUT),
        ("Timeout", "timed out", TIMED_OUT),
        (
            "org.bluez.Error.ConnectionAttemptFailed",
            "Page Timeout",
            "Couldn't reach it. Is it in pairing mode?",
        ),
        ("org.bluez.Error.NotReady", "Resource Not Ready", NOT_READY),
        (
            "org.freedesktop.DBus.Error.UnknownObject",
            "",
            "It's gone. Scan again",
        ),
        ("org.bluez.Error.Failed", "", "Pairing failed"),
        (
            "org.bluez.Error.Failed",
            "Input/output error",
            "Pairing failed: Input/output error",
        ),
    ];
    for (name, message, shown) in cases {
        assert_eq!(describe(&BluezError::new(name, message)), shown, "{name}");
    }
}
