use std::collections::BTreeMap;

use super::*;
use crate::fake::FakeBus;
use crate::fake::FakeRunner;
use crate::fake::key;

const ADAPTER: Property = Property {
    service: "org.bluez",
    path: "/org/bluez/hci0",
    interface: "org.bluez.Adapter1",
    name: "Powered",
};

fn none() -> BTreeMap<String, String> {
    BTreeMap::new()
}

fn percent_of(value: &str) -> BTreeMap<String, String> {
    BTreeMap::from([(PERCENT.to_owned(), value.to_owned())])
}

fn wifi_bus(on: bool) -> FakeBus {
    let mut bus = FakeBus::default();
    bus.booleans.insert(key(&WIFI), on);
    bus
}

fn bluetooth_bus(on: bool) -> FakeBus {
    let mut bus = FakeBus::default();
    bus.children
        .insert("org.bluez /org/bluez".into(), vec!["hci0".into(), "hci1".into()]);
    bus.booleans.insert(key(&ADAPTER), on);
    bus
}

#[test]
fn every_toggle_is_an_item_of_its_own_id() {
    let ids: std::collections::BTreeSet<String> = Toggle::ALL.into_iter().map(|toggle| toggle.item().id).collect();

    assert_eq!(ids.len(), Toggle::ALL.len(), "each toggle's id is its own");
    for toggle in Toggle::ALL {
        let item = toggle.item();
        assert_eq!(Toggle::from_id(&item.id), Some(toggle));
        assert_eq!(item.kind.name(), KIND);
        assert_ne!(item.actions, [] as [kotodama_protocol::Action; 0]);
    }
}

#[test]
fn toggling_wifi_turns_it_off_when_on() {
    let mut bus = wifi_bus(true);

    let ran = Toggle::Wifi.run(TOGGLE, &none(), &mut bus, &FakeRunner::default());

    assert_eq!(ran, Ok(Ran::Close));
    assert!(!bus.booleans[&key(&WIFI)]);
}

#[test]
fn toggling_wifi_turns_it_on_when_off() {
    let mut bus = wifi_bus(false);

    Toggle::Wifi
        .run(TOGGLE, &none(), &mut bus, &FakeRunner::default())
        .unwrap();

    assert!(bus.booleans[&key(&WIFI)]);
}

#[test]
fn wifi_on_and_off_set_it_whatever_it_was() {
    let mut bus = FakeBus::default();

    Toggle::Wifi.run(ON, &none(), &mut bus, &FakeRunner::default()).unwrap();
    assert!(bus.booleans[&key(&WIFI)]);
    Toggle::Wifi
        .run(OFF, &none(), &mut bus, &FakeRunner::default())
        .unwrap();
    assert!(!bus.booleans[&key(&WIFI)]);
}

#[test]
fn wifi_without_networkmanager_says_so() {
    let mut bus = FakeBus {
        down: true,
        ..FakeBus::default()
    };

    let err = Toggle::Wifi
        .run(TOGGLE, &none(), &mut bus, &FakeRunner::default())
        .unwrap_err();

    assert!(err.starts_with("Couldn't tell whether Wi-Fi is on"), "{err}");
}

#[test]
fn toggling_bluetooth_powers_the_first_adapter() {
    let mut bus = bluetooth_bus(false);

    let ran = Toggle::Bluetooth.run(TOGGLE, &none(), &mut bus, &FakeRunner::default());

    assert_eq!(ran, Ok(Ran::Close));
    assert!(bus.booleans[&key(&ADAPTER)]);
}

#[test]
fn bluetooth_without_an_adapter_says_so() {
    let mut bus = FakeBus::default();

    let err = Toggle::Bluetooth
        .run(ON, &none(), &mut bus, &FakeRunner::default())
        .unwrap_err();

    assert_eq!(err, "There's no Bluetooth adapter, so there's nothing to turn on");
}

#[test]
fn mute_toggles_the_default_sink() {
    let runner = FakeRunner::default();

    Toggle::Mute
        .run(RUN, &none(), &mut FakeBus::default(), &runner)
        .unwrap();

    assert_eq!(runner.ran(), ["wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"]);
}

#[test]
fn volume_steps_keep_the_menu_open_for_another() {
    let runner = FakeRunner::default();

    let ran = Toggle::VolumeUp.run(RUN, &none(), &mut FakeBus::default(), &runner);

    assert_eq!(ran, Ok(Ran::KeepOpen));
    assert_eq!(runner.ran(), ["wpctl set-volume -l 1 @DEFAULT_AUDIO_SINK@ 5%+"]);
}

#[test]
fn volume_takes_a_percentage_with_or_without_its_sign() {
    let runner = FakeRunner::default();

    Toggle::Volume
        .run(RUN, &percent_of("40"), &mut FakeBus::default(), &runner)
        .unwrap();
    Toggle::Volume
        .run(RUN, &percent_of(" 75 % "), &mut FakeBus::default(), &runner)
        .unwrap();

    assert_eq!(
        runner.ran(),
        [
            "wpctl set-volume -l 1 @DEFAULT_AUDIO_SINK@ 40%",
            "wpctl set-volume -l 1 @DEFAULT_AUDIO_SINK@ 75%"
        ]
    );
}

#[test]
fn a_volume_out_of_range_runs_nothing() {
    let runner = FakeRunner::default();

    let err = Toggle::Volume
        .run(RUN, &percent_of("140"), &mut FakeBus::default(), &runner)
        .unwrap_err();

    assert_eq!(err, "Type a percentage from 0 to 100, not `140`");
    assert_eq!(runner.ran(), [] as [std::string::String; 0]);
}

#[test]
fn brightness_never_goes_to_zero() {
    let runner = FakeRunner::default();

    let err = Toggle::Brightness
        .run(RUN, &percent_of("0"), &mut FakeBus::default(), &runner)
        .unwrap_err();
    Toggle::Brightness
        .run(RUN, &percent_of("70"), &mut FakeBus::default(), &runner)
        .unwrap();

    assert!(err.contains("from 1 to 100"), "{err}");
    assert_eq!(runner.ran(), ["brightnessctl -e4 -n2 set 70%"]);
}

#[test]
fn a_failing_program_says_what_it_said() {
    let runner = FakeRunner {
        failing: Some("Could not connect to PipeWire".into()),
        ..FakeRunner::default()
    };

    let err = Toggle::Mute
        .run(RUN, &none(), &mut FakeBus::default(), &runner)
        .unwrap_err();

    assert_eq!(err, "wpctl failed: Could not connect to PipeWire");
}

#[test]
fn audio_output_lists_the_sinks_in_use_first() {
    let runner = FakeRunner {
        outputs: BTreeMap::from([(
            "wpctl status".to_owned(),
            "Audio\n ├─ Sinks:\n │      52. HDMI   [vol: 1.00]\n │  *   53. Speakers   [vol: 0.40]\n".to_owned(),
        )]),
        ..FakeRunner::default()
    };

    let ran = Toggle::AudioOutput
        .run(RUN, &none(), &mut FakeBus::default(), &runner)
        .unwrap();

    let Ran::Push(View {
        id,
        content: ViewContent::List { items, .. },
        ..
    }) = ran
    else {
        panic!("expected a list View, got {ran:?}");
    };
    assert_eq!(id, "audio-output");
    let rows: Vec<(&str, &str, Option<&str>)> = items
        .iter()
        .map(|item| (item.id.as_str(), item.title.as_str(), item.subtitle.as_deref()))
        .collect();
    assert_eq!(
        rows,
        [
            ("audio-output:53", "Speakers", Some("In use")),
            ("audio-output:52", "HDMI", None)
        ]
    );
    assert!(items.iter().all(|item| item.kind.name() == DEVICE_KIND));
}

#[test]
fn no_audio_devices_is_said_rather_than_an_empty_list() {
    let err = Toggle::AudioInput
        .run(RUN, &none(), &mut FakeBus::default(), &FakeRunner::default())
        .unwrap_err();

    assert_eq!(err, "PipeWire lists no audio devices. Hello? Anyone?");
}

#[test]
fn using_a_device_makes_it_the_default_and_closes_its_list() {
    let runner = FakeRunner::default();

    let ran = use_device("audio-input:54", &runner);

    assert_eq!(ran, Ok(Ran::Pop("audio-input".into())));
    assert_eq!(runner.ran(), ["wpctl set-default 54"]);
}

#[test]
fn a_device_id_that_is_no_device_runs_nothing() {
    let runner = FakeRunner::default();

    assert!(use_device("audio-input:@DEFAULT@", &runner).is_err());
    assert!(use_device("wifi:1", &runner).is_err());
    assert!(use_device("54", &runner).is_err());
    assert_eq!(runner.ran(), [] as [std::string::String; 0]);
}
