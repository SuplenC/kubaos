use futures_lite::future;

use super::*;
use crate::bluez::BluezError;
use crate::fake::FakeBluez;

const ADAPTER: &str = "/org/bluez/hci0";

#[test]
fn starting_stops_the_old_session_first() {
    let bluez = FakeBluez::default();
    assert_eq!(future::block_on(scan(&bluez, ADAPTER, true)), Ok(()));
    assert_eq!(
        bluez.calls(),
        [
            "StopDiscovery /org/bluez/hci0",
            "StartDiscovery /org/bluez/hci0"
        ]
    );
}

#[test]
fn stopping_without_a_session_is_fine() {
    let bluez = FakeBluez::default();
    assert_eq!(future::block_on(scan(&bluez, ADAPTER, false)), Ok(()));
    assert_eq!(bluez.calls(), ["StopDiscovery /org/bluez/hci0"]);
}

#[test]
fn not_ready_says_how_to_get_bluetooth_back() {
    let bluez = FakeBluez::default();
    bluez.start.lock().unwrap().push_back(Err(BluezError::new(
        "org.bluez.Error.NotReady",
        "Resource Not Ready",
    )));
    let err = future::block_on(scan(&bluez, ADAPTER, true)).unwrap_err();
    assert_eq!(err, NOT_READY);
    assert!(err.contains("sudo systemctl restart bluetooth"));
}

#[test]
fn another_refusal_carries_bluetoothds_message() {
    let bluez = FakeBluez::default();
    bluez.start.lock().unwrap().push_back(Err(BluezError::new(
        "org.bluez.Error.Failed",
        "Operation failed",
    )));
    assert_eq!(
        future::block_on(scan(&bluez, ADAPTER, true)),
        Err("Couldn't scan: Operation failed".to_owned())
    );
}
