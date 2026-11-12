use jiff::Zoned;
use serde_json::json;

use super::*;
use crate::history::Kind;

fn now() -> Zoned {
    "2026-11-12T10:00:00[Europe/Warsaw]".parse().expect("a zoned time")
}

#[test]
fn the_state_lists_each_counter_with_its_reset_and_the_last_used() {
    let mut counters = Counters::default();
    for name in ["Pushups", "Coffee", "Steps"] {
        counters.create(name, &now()).unwrap();
    }
    let rule = AutoReset::Rule(Recurrence::parse("every day at 9").unwrap());
    counters.set_reset("Pushups", rule, &now()).unwrap();
    counters.set_reset("Coffee", AutoReset::Stop(4), &now()).unwrap();
    counters.add("Pushups", &now()).unwrap();

    let state = state(&counters, None);

    assert_eq!(state["last"], "Pushups");
    assert_eq!(
        state["counters"][0],
        json!({
            "id": 1, "name": "Pushups", "value": 1, "stop": 0,
            "rule": "every day at 9:00", "next": 1_794_556_800,
            "detail": "Resets every day at 9:00",
        })
    );
    assert_eq!(state["counters"][1]["detail"], "Stops at 4");
    assert_eq!(state["counters"][1]["stop"], 4);
    assert_eq!(state["counters"][2]["detail"], "");
    assert_eq!(state["counters"][2]["next"], 0);
}

#[test]
fn the_last_is_the_only_counter_when_none_was_used() {
    let mut counters = Counters::default();
    counters.create("Pushups", &now()).unwrap();
    counters.create("Coffee", &now()).unwrap();
    counters.remove("Coffee", &now()).unwrap();

    assert_eq!(state(&counters, None)["last"], "Pushups");
}

#[test]
fn no_counters_is_an_empty_list() {
    assert_eq!(state(&Counters::default(), None), json!({"last": "", "counters": []}));
}

#[test]
fn the_details_follow_the_clock_format() {
    let rule = AutoReset::Rule(Recurrence::parse("every day at 21:00").unwrap());

    assert_eq!(
        detail(rule, Some(TimeFormat::TwelveHour)),
        "Resets every day at 9:00 PM"
    );
}

#[test]
fn the_state_file_is_written_whole() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(FILE);

    write(&path, &json!({"last": "", "counters": []})).unwrap();

    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "{\"counters\":[],\"last\":\"\"}\n"
    );
    assert!(!dir.path().join("kubaos-counters.json.new").exists());
}

#[test]
fn a_rule_that_reads_says_its_words_and_next_time() {
    let answer = check("mondays 8am", &now(), None);

    assert_eq!(
        answer,
        json!({
            "ok": true, "rule": "every Monday at 8:00", "words": "Every Monday at 8:00",
            "next": 1_794_812_400, "when": "Next Monday at 8:00",
        })
    );
    assert_eq!(check("every day at 11", &now(), None)["when"], "Next today at 11:00");
    assert_eq!(check("every day at 9", &now(), None)["when"], "Next tomorrow at 9:00");
    assert_eq!(
        check("1st of the month", &now(), None)["when"],
        "Next 1 December at 0:00"
    );
}

#[test]
fn a_rule_that_does_not_read_says_why_with_examples() {
    let answer = check("sometimes", &now(), None);

    assert_eq!(answer["ok"], false);
    assert!(answer["error"].as_str().unwrap().starts_with("`sometimes` isn't"));
    assert_eq!(answer["examples"][0], "every day at 9");
}

#[test]
fn history_rows_carry_their_kind_and_label() {
    let rows = [
        Row {
            at: now().timestamp(),
            kind: Some(Kind::Sub),
            value: 3,
            added: -1,
            changes: 1,
        },
        Row {
            at: now().timestamp(),
            kind: None,
            value: 3,
            added: 5,
            changes: 6,
        },
    ];

    let json = history(&rows);

    assert_eq!(
        json,
        json!([
            {"at": 1_794_474_000, "kind": "sub", "label": "\u{2212}1", "value": 3, "added": -1, "changes": 1},
            {"at": 1_794_474_000, "kind": "", "label": "", "value": 3, "added": 5, "changes": 6},
        ])
    );
}
