use std::path::Path;

use super::Error;
use super::TEST_TEXT;
use super::accept;
use super::received;
use super::restrict;
use super::ring;
use super::send_clipboard;
use super::send_file;
use super::send_test;
use super::status;
use super::turn_off;
use super::turn_on;
use super::typed_path;
use super::unpair;
use crate::device::Device;
use crate::fake::Fake;
use crate::fake::iphone;
use crate::fake::iphone_asking;
use crate::fake::stranger;

fn ipad_away() -> Device {
    Device {
        id: "ipad".into(),
        name: "iPad".into(),
        kind: "tablet".into(),
        reachable: false,
        battery: None,
        ..iphone()
    }
}

#[test]
fn a_link_that_is_off_never_asks_kdeconnect_and_names_the_laptop_by_its_host_name() {
    let mut fake = Fake {
        running: true,
        devices: vec![iphone()],
        hostname: Some("alias2k".into()),
        ..Fake::new()
    };

    let st = status(&mut fake);

    assert!(!st.on && !st.running, "{st:?}");
    assert_eq!(st.devices, Vec::<Device>::new());
    assert_eq!(st.name.as_deref(), Some("alias2k"));
}

#[test]
fn a_running_link_lists_its_devices_its_name_and_the_laptops_addresses() {
    let mut fake = Fake::running(vec![iphone_asking(), stranger()]);

    let st = status(&mut fake);

    assert!(st.on && st.running);
    assert_eq!(st.name.as_deref(), Some("kubaos"));
    assert_eq!(st.addresses, ["192.168.1.23"]);
    assert_eq!(st.request().map(|d| d.name.as_str()), Some("iPhone"));
    assert_eq!(st.others().map(|d| d.name.as_str()).collect::<Vec<_>>(), ["site24"]);
}

#[test]
fn turning_on_writes_the_setting_before_it_starts_kdeconnect() {
    let mut fake = Fake::new();

    let st = turn_on(&mut fake).expect("turn the link on");

    assert_eq!(fake.calls, ["on", "start"]);
    assert!(st.on && st.running);
}

#[test]
fn turning_on_says_so_when_kdeconnect_never_comes_up() {
    let mut fake = Fake {
        starts: false,
        ..Fake::new()
    };

    let err = turn_on(&mut fake).expect_err("kdeconnectd never answers");

    assert!(matches!(err, Error::NotStarted), "{err}");
    assert!(fake.on, "the setting stays on, so the next login tries again");
}

#[test]
fn turning_off_takes_the_setting_away_before_it_stops_kdeconnect() {
    let mut fake = Fake::running(vec![iphone()]);

    turn_off(&mut fake).expect("turn the link off");

    assert_eq!(fake.calls, ["off", "stop"]);
    assert!(!fake.on && !fake.running);
}

#[test]
fn accepting_pairs_the_device_that_asked() {
    let mut fake = Fake::running(vec![stranger(), iphone_asking()]);
    let id = iphone().id;

    let phone = accept(&mut fake, &id).expect("accept the iPhone");

    assert!(phone.paired && !phone.wants_to_pair);
    assert_eq!(fake.calls, [format!("Accept {id}")]);
}

#[test]
fn a_device_that_didnt_ask_to_pair_is_never_accepted() {
    let mut fake = Fake::running(vec![stranger(), iphone_asking()]);

    let err = accept(&mut fake, "site24").expect_err("site24 didn't ask");

    assert!(matches!(err, Error::NoRequest(ref name) if name == "site24"), "{err}");
    assert_eq!(fake.calls, Vec::<String>::new());
}

#[test]
fn nothing_is_sent_while_the_link_is_off() {
    let mut fake = Fake {
        running: true,
        devices: vec![iphone()],
        ..Fake::new()
    };

    let err = send_clipboard(&mut fake, None).expect_err("the link is off");

    assert!(matches!(err, Error::Off), "{err}");
    assert_eq!(fake.calls, Vec::<String>::new());
}

#[test]
fn a_link_that_is_on_with_kdeconnect_down_says_it_isnt_running() {
    let mut fake = Fake {
        on: true,
        ..Fake::new()
    };

    let err = ring(&mut fake, None).expect_err("kdeconnectd is down");

    assert!(matches!(err, Error::NotRunning), "{err}");
}

#[test]
fn the_clipboard_goes_to_the_paired_phone_in_reach() {
    let mut fake = Fake::running(vec![ipad_away(), stranger(), iphone()]);

    let phone = send_clipboard(&mut fake, None).expect("send the clipboard");

    assert_eq!(phone.name, "iPhone");
    assert_eq!(fake.calls, [format!("SendClipboard {}", iphone().id)]);
}

#[test]
fn with_no_paired_phone_or_none_in_reach_it_says_which() {
    let mut fake = Fake::running(vec![stranger()]);
    let err = ring(&mut fake, None).expect_err("nothing is paired");
    assert!(matches!(err, Error::NoPhone), "{err}");

    let mut fake = Fake::running(vec![ipad_away()]);
    let err = ring(&mut fake, None).expect_err("the iPad is away");
    assert!(matches!(err, Error::Unreachable(ref name) if name == "iPad"), "{err}");
}

#[test]
fn a_phone_is_picked_by_its_name_in_any_case_or_by_its_id() {
    let mut fake = Fake::running(vec![
        iphone(),
        Device {
            reachable: true,
            ..ipad_away()
        },
    ]);

    assert_eq!(ring(&mut fake, Some("IPAD")).expect("ring the iPad").name, "iPad");
    assert_eq!(ring(&mut fake, Some(&iphone().id)).expect("ring the iPhone").name, "iPhone");
    let err = ring(&mut fake, Some("pixel")).expect_err("no Pixel");
    assert!(matches!(err, Error::Unknown(ref name) if name == "pixel"), "{err}");
}

#[test]
fn a_file_goes_by_its_url_with_the_odd_characters_escaped() {
    let dir = tempfile::tempdir().expect("a folder");
    let file = dir.path().join("a photo #1.jpg");
    std::fs::write(&file, "jpeg").expect("write the photo");
    let mut fake = Fake::running(vec![iphone()]);

    send_file(&mut fake, None, &file).expect("send the photo");

    let url = format!("file://{}/a%20photo%20%231.jpg", dir.path().display());
    assert_eq!(fake.calls, [format!("Share({url:?}) {}", iphone().id)]);
}

#[test]
fn a_file_that_isnt_there_is_never_sent() {
    let mut fake = Fake::running(vec![iphone()]);

    let err = send_file(&mut fake, None, Path::new("/nowhere/photo.jpg")).expect_err("no such file");

    assert!(matches!(err, Error::NoFile(_)), "{err}");
    assert_eq!(fake.calls, Vec::<String>::new());
}

#[test]
fn unpairing_needs_the_phone_paired_but_not_in_reach() {
    let mut fake = Fake::running(vec![ipad_away()]);

    let phone = unpair(&mut fake, None).expect("unpair the iPad");

    assert_eq!(phone.name, "iPad");
    assert_eq!(fake.calls, ["Unpair ipad"]);
}

#[test]
fn the_test_copies_its_text_here_then_sends_the_clipboard() {
    let mut fake = Fake::running(vec![iphone()]);

    send_test(&mut fake).expect("send the test");

    assert_eq!(
        fake.calls,
        [format!("copy {TEST_TEXT}"), format!("SendClipboard {}", iphone().id)]
    );
}

#[test]
fn what_the_phone_sent_is_the_clipboard_once_it_is_no_longer_the_test() {
    let mut fake = Fake::running(vec![iphone()]);
    fake.clipboard = TEST_TEXT.into();
    assert!(matches!(received(&mut fake), Err(Error::NothingNew)));

    fake.clipboard = "  ".into();
    assert!(matches!(received(&mut fake), Err(Error::NothingNew)));

    fake.clipboard = "from the phone\n".into();
    assert_eq!(received(&mut fake).expect("the phone's text"), "from the phone");
}

#[test]
fn a_refusal_from_kdeconnect_keeps_its_reason() {
    let mut fake = Fake::running(vec![iphone()]);
    fake.refuse = Some("Unknown method ring".into());

    let err = ring(&mut fake, None).expect_err("kdeconnectd refuses");

    assert!(matches!(err, Error::Link(_)), "{err}");
    let cause = std::error::Error::source(&err).map(ToString::to_string).unwrap_or_default();
    assert!(cause.contains("Unknown method ring"), "{cause}");
}

/// The plugins kdeconnectd 26.08 loads for an iPhone, all on, as a new pairing leaves them.
fn iphone_plugins(fake: &mut Fake, id: &str) {
    for plugin in [
        "kdeconnect_clipboard",
        "kdeconnect_share",
        "kdeconnect_ping",
        "kdeconnect_battery",
        "kdeconnect_findmyphone",
        "kdeconnect_findthisdevice",
        "kdeconnect_mousepad",
        "kdeconnect_runcommand",
        "kdeconnect_presenter",
        "kdeconnect_screensaver_inhibit",
        "kdeconnect_shareinputdevicesremote",
    ] {
        fake.plugins.push((id.to_string(), plugin.to_string(), true));
    }
}

fn on_plugins(fake: &Fake, id: &str) -> Vec<String> {
    fake.plugins
        .iter()
        .filter(|(d, _, on)| d == id && *on)
        .map(|(_, p, _)| p.clone())
        .collect()
}

const LINK_PLUGINS: [&str; 6] = [
    "kdeconnect_clipboard",
    "kdeconnect_share",
    "kdeconnect_ping",
    "kdeconnect_battery",
    "kdeconnect_findmyphone",
    "kdeconnect_findthisdevice",
];

#[test]
fn restricting_leaves_a_phone_only_the_plugins_the_link_is_for() {
    let id = iphone().id;
    let mut fake = Fake::running(vec![iphone()]);
    iphone_plugins(&mut fake, &id);

    let changed = restrict(&mut fake).expect("restrict the plugins");

    assert_eq!(on_plugins(&fake, &id), LINK_PLUGINS);
    assert_eq!(changed.len(), 5, "{changed:?}");
    assert!(changed.iter().any(|c| c == "iPhone: kdeconnect_mousepad off"), "{changed:?}");
}

#[test]
fn a_plugin_kubaos_doesnt_know_stays_off_and_one_the_user_allowed_stays_on() {
    let id = iphone().id;
    let mut fake = Fake::running(vec![iphone()]);
    iphone_plugins(&mut fake, &id);
    fake.plugins.push((id.clone(), "kdeconnect_from_the_future".into(), true));
    fake.extra = vec!["kdeconnect_mousepad".into()];

    restrict(&mut fake).expect("restrict the plugins");

    let on = on_plugins(&fake, &id);
    assert!(on.contains(&"kdeconnect_mousepad".to_string()), "{on:?}");
    assert!(!on.contains(&"kdeconnect_from_the_future".to_string()), "{on:?}");
}

#[test]
fn a_plugin_the_user_turned_off_is_left_off() {
    let id = iphone().id;
    let mut fake = Fake::running(vec![iphone()]);
    fake.plugins.push((id.clone(), "kdeconnect_clipboard".into(), false));

    let changed = restrict(&mut fake).expect("restrict the plugins");

    assert_eq!(changed, Vec::<String>::new());
    assert_eq!(on_plugins(&fake, &id), Vec::<String>::new());
}

#[test]
fn turning_on_restricts_every_device_kde_connect_knows() {
    let id = iphone().id;
    let mut fake = Fake {
        devices: vec![iphone(), stranger()],
        ..Fake::new()
    };
    iphone_plugins(&mut fake, &id);
    iphone_plugins(&mut fake, "site24");

    turn_on(&mut fake).expect("turn the link on");

    assert_eq!(on_plugins(&fake, &id), LINK_PLUGINS);
    assert_eq!(on_plugins(&fake, "site24"), LINK_PLUGINS);
}

#[test]
fn accepting_a_pairing_restricts_the_new_phone() {
    let id = iphone().id;
    let mut fake = Fake::running(vec![iphone_asking()]);
    iphone_plugins(&mut fake, &id);

    accept(&mut fake, &id).expect("accept the iPhone");

    assert_eq!(on_plugins(&fake, &id), LINK_PLUGINS);
}

#[test]
fn a_typed_path_starting_with_a_tilde_is_under_home() {
    let home = Some(Path::new("/home/kuba"));

    assert_eq!(typed_path("~/Downloads/a.jpg", home), Path::new("/home/kuba/Downloads/a.jpg"));
    assert_eq!(typed_path("~", home), Path::new("/home/kuba"));
    assert_eq!(typed_path("/tmp/~a", home), Path::new("/tmp/~a"));
    assert_eq!(typed_path("~/a.jpg", None), Path::new("~/a.jpg"));
}
