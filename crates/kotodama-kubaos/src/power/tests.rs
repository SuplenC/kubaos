use kubaos_power::stop::Stop;

use super::*;

fn now() -> Now {
    Now {
        stop: Stop::SuperSaver,
        header: "Power: Super Saver, on battery".to_owned(),
        rows: vec![
            Row {
                name: "PCIe ASPM",
                wanted: "powersupersave".to_owned(),
                now: Some("default".to_owned()),
                agrees: Some(false),
            },
            Row {
                name: "CPU boost",
                wanted: "off".to_owned(),
                now: Some("off".to_owned()),
                agrees: Some(true),
            },
            Row {
                name: "Charge limit",
                wanted: "100 %".to_owned(),
                now: None,
                agrees: None,
            },
        ],
    }
}

#[test]
fn the_item_is_found_by_what_people_type() {
    let item = item();

    assert_eq!(item.id, ID);
    assert_eq!(item.title, "Power: what this stop does");
    assert!(item.keywords.contains(&"super saver".to_owned()));
}

#[test]
fn each_row_says_what_the_stop_wants_and_what_the_machine_has() {
    let view = view(&now());

    assert_eq!(view.title, "Power: Super Saver, on battery");
    let ViewContent::List { items, .. } = view.content else {
        panic!("a list View");
    };
    let rows: Vec<(&str, &str)> = items
        .iter()
        .map(|item| (item.title.as_str(), item.subtitle.as_deref().unwrap_or("")))
        .collect();
    assert_eq!(
        rows,
        [
            ("PCIe ASPM: powersupersave", "Now default, not what the stop sets"),
            ("CPU boost: off", "Now off"),
            ("Charge limit: 100 %", "Not on this machine"),
        ]
    );
}
