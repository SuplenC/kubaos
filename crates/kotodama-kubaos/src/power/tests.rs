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

#[test]
fn the_scheduler_item_is_found_by_its_names() {
    let item = scheduler_item();

    assert_eq!(item.id, SCHEDULER_ID);
    assert_eq!(item.title, "CPU scheduler: status");
    assert!(item.keywords.contains(&"lavd".to_owned()));
    assert!(item.keywords.contains(&"eevdf".to_owned()));
    assert_eq!(ids(), [ID, SCHEDULER_ID, DRAIN_ID]);
}

#[test]
fn the_scheduler_view_says_which_runs_and_why_then_the_details() {
    let report = Report {
        running: "Linux's own (EEVDF)".to_owned(),
        why: "LAVD crashed and the kernel fell back to its own scheduler; it starts again in 2 s.".to_owned(),
        agrees: false,
        details: vec![
            (
                "Setting".to_owned(),
                "LAVD on (scheduler = \"lavd\" in power.toml)".to_owned(),
            ),
            (
                "Service".to_owned(),
                "kubaos-scx.service: activating (auto-restart)".to_owned(),
            ),
        ],
    };

    let view = scheduler_view(&report);

    assert_eq!(view.title, "CPU scheduler");
    let ViewContent::List { items, .. } = view.content else {
        panic!("a list View");
    };
    let rows: Vec<(&str, Option<&str>)> = items
        .iter()
        .map(|item| (item.title.as_str(), item.subtitle.as_deref()))
        .collect();
    assert_eq!(
        rows,
        [
            (
                "Running: Linux's own (EEVDF)",
                Some("LAVD crashed and the kernel fell back to its own scheduler; it starts again in 2 s.")
            ),
            ("Setting: LAVD on (scheduler = \"lavd\" in power.toml)", None),
            ("Service: kubaos-scx.service: activating (auto-restart)", None),
        ]
    );
}

#[test]
fn the_drain_item_is_found_by_what_people_type() {
    let item = drain_item();

    assert_eq!(item.id, DRAIN_ID);
    assert_eq!(item.title, "What's draining the battery");
    assert!(item.keywords.contains(&"battery drain".to_owned()));
}

#[test]
fn the_drain_view_has_a_row_a_line_of_the_answer() {
    let drain = Drain {
        on_battery: true,
        watts: Some(7.1),
        percent: Some(64.0),
        stop: "Power Saver".to_owned(),
        wakers: Some(kubaos_power::history::Wakers {
            secs: 600,
            irqs: vec![("24 amdgpu".to_owned(), 120.0)],
            procs: vec![("qs".to_owned(), 30.0)],
        }),
    };

    let view = drain_view(&drain);

    let ViewContent::List { items, .. } = view.content else {
        panic!("a list");
    };
    let rows: Vec<(&str, Option<&str>)> = items
        .iter()
        .map(|item| (item.title.as_str(), item.subtitle.as_deref()))
        .collect();
    assert_eq!(
        rows,
        [
            ("Drawing 7.1 W now", Some("Power Saver, 64 % left")),
            ("qs: 30 a second", Some("A process waking")),
            ("24 amdgpu: 120 a second", Some("An interrupt line")),
        ]
    );
}
