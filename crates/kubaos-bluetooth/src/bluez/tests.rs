use super::*;

#[test]
fn the_address_comes_from_the_object_path() {
    assert_eq!(address("/org/bluez/hci0/dev_AA_BB_CC_DD_EE_0F"), "AA:BB:CC:DD:EE:0F");
}

#[test]
fn an_error_is_matched_on_its_names_last_part() {
    let err = BluezError::new("org.bluez.Error.AuthenticationFailed", "Authentication Failed");
    assert_eq!(err.short_name(), "AuthenticationFailed");
    assert!(err.is("AuthenticationFailed"));
    assert!(!err.is("Failed"));
}

#[test]
fn refusals_reach_bluetoothd_under_its_error_names() {
    use zbus::DBusError as _;

    assert_eq!(
        AgentError::from(Refusal::Rejected).name().as_str(),
        "org.bluez.Error.Rejected"
    );
    assert_eq!(
        AgentError::from(Refusal::Canceled).name().as_str(),
        "org.bluez.Error.Canceled"
    );
}
