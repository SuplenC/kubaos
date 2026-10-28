use super::busctl;
use super::device_call;
use super::first;
use super::parse_addresses;
use super::parse_battery;
use super::parse_device;
use super::parse_ids;
use super::valid_plugin;
use crate::device::Battery;
use crate::link::Action;
use crate::link::LinkError;

const ID: &str = "897224eb_f278_4be9_bd3f_f76230491def";

/// `busctl --json=short` of the device's `GetAll`, as kdeconnectd 26.08 answers for an iPhone
/// paired and out of reach.
const IPHONE: &str = r#"{"type":"a{sv}","data":[{"activeProviderNames":{"type":"as","data":[]},"iconName":{"type":"s","data":"smartphonedisconnected"},"isPairRequested":{"type":"b","data":false},"isPairRequestedByPeer":{"type":"b","data":false},"isPaired":{"type":"b","data":true},"isReachable":{"type":"b","data":false},"name":{"type":"s","data":"iKuba"},"pairState":{"type":"i","data":3},"reachableAddresses":{"type":"as","data":[]},"statusIconName":{"type":"s","data":"smartphonetrusted"},"supportedPlugins":{"type":"as","data":["kdeconnect_clipboard","kdeconnect_findmyphone","kdeconnect_share","kdeconnect_battery"]},"type":{"type":"s","data":"phone"},"verificationKey":{"type":"s","data":"807704F2"}}]}"#;

#[test]
fn every_call_goes_to_the_user_bus_and_never_starts_kdeconnectd() {
    let args = busctl(&["call", "org.kde.kdeconnect", "/modules/kdeconnect"]);

    assert_eq!(args[..3], ["--user", "--auto-start=no", "--json=short"]);
}

#[test]
fn the_device_ids_come_from_the_daemons_answer() {
    let ids = parse_ids(&first(r#"{"type":"as","data":[["897224eb_f278_4be9_bd3f_f76230491def","site24"]]}"#).unwrap());

    assert_eq!(ids, [ID, "site24"]);
}

#[test]
fn a_paired_iphone_out_of_reach_reads_as_such() {
    let device = parse_device(ID, &first(IPHONE).unwrap());

    assert_eq!(device.id, ID);
    assert_eq!(device.name, "iKuba");
    assert_eq!(device.kind, "phone");
    assert!(device.paired && !device.reachable && !device.wants_to_pair);
    assert_eq!(device.verification_key.as_deref(), Some("807704F2"));
}

#[test]
fn a_device_asking_to_pair_says_so() {
    let asking = IPHONE
        .replace(
            r#""isPairRequestedByPeer":{"type":"b","data":false}"#,
            r#""isPairRequestedByPeer":{"type":"b","data":true}"#,
        )
        .replace(
            r#""isPaired":{"type":"b","data":true}"#,
            r#""isPaired":{"type":"b","data":false}"#,
        );

    let device = parse_device(ID, &first(&asking).unwrap());

    assert!(device.wants_to_pair && !device.paired);
}

#[test]
fn the_battery_reads_its_charge_and_whether_it_charges() {
    let answer = r#"{"type":"a{sv}","data":[{"charge":{"type":"i","data":82},"isCharging":{"type":"b","data":true}}]}"#;

    assert_eq!(
        parse_battery(&first(answer).unwrap()),
        Some(Battery {
            charge: 82,
            charging: true
        })
    );
    assert_eq!(
        parse_battery(&first(r#"{"type":"a{sv}","data":[{"charge":{"type":"i","data":-1}}]}"#).unwrap()),
        None
    );
}

#[test]
fn an_answer_that_isnt_busctls_json_is_an_error() {
    assert!(matches!(
        first("Call failed: no such method"),
        Err(LinkError::Answer(_))
    ));
}

#[test]
fn the_laptops_addresses_leave_out_containers_and_bridges() {
    let ip = r#"[{"ifindex":3,"ifname":"wlp1s0","addr_info":[{"family":"inet","local":"192.168.1.23","prefixlen":24,"scope":"global"}]},
        {"ifindex":4,"ifname":"docker0","addr_info":[{"family":"inet","local":"172.17.0.1","prefixlen":16,"scope":"global"}]},
        {"ifindex":5,"ifname":"br-1a2b","addr_info":[{"family":"inet","local":"172.18.0.1","prefixlen":16,"scope":"global"}]},
        {"ifindex":6,"ifname":"enp2s0","addr_info":[{"family":"inet","local":"10.0.0.5","prefixlen":8,"scope":"global"}]}]"#;

    assert_eq!(parse_addresses(ip), ["192.168.1.23", "10.0.0.5"]);
    assert_eq!(parse_addresses("not json"), Vec::<String>::new());
}

#[test]
fn each_action_calls_its_plugin_on_the_devices_object() {
    let path = format!("/modules/kdeconnect/devices/{ID}");
    let call = |action| device_call(ID, &action).unwrap();

    assert_eq!(
        call(Action::Accept),
        [
            "call",
            "org.kde.kdeconnect",
            &path,
            "org.kde.kdeconnect.device",
            "acceptPairing"
        ]
    );
    assert_eq!(call(Action::Unpair)[4], "unpair");
    assert_eq!(
        call(Action::SendClipboard)[2..],
        [
            format!("{path}/clipboard"),
            "org.kde.kdeconnect.device.clipboard".into(),
            "sendClipboard".into()
        ]
    );
    assert_eq!(
        call(Action::Ring)[2..],
        [
            format!("{path}/findmyphone"),
            "org.kde.kdeconnect.device.findmyphone".into(),
            "ring".into()
        ]
    );
    assert_eq!(
        call(Action::Share("file:///home/kuba/a.jpg".into()))[2..],
        [
            format!("{path}/share"),
            "org.kde.kdeconnect.device.share".into(),
            "shareUrl".into(),
            "s".into(),
            "file:///home/kuba/a.jpg".into()
        ]
    );
}

#[test]
fn only_a_kde_connect_plugin_name_goes_to_set_plugin_enabled() {
    assert!(valid_plugin("kdeconnect_mousepad"));
    assert!(valid_plugin("kdeconnect_connectivity_report"));
    for name in ["mousepad", "kdeconnect_Mouse", "kdeconnect_a b", "kdeconnect_x\" true"] {
        assert!(!valid_plugin(name), "{name:?}");
    }
}

#[test]
fn an_id_that_would_leave_the_devices_object_is_refused() {
    for id in ["", "../daemon", "a/b", "a b"] {
        assert!(
            matches!(device_call(id, &Action::Ring), Err(LinkError::BadId(_))),
            "{id:?}"
        );
    }
}
