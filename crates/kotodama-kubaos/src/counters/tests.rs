use std::collections::BTreeMap;

use jiff::Timestamp;
use kubaos_counters::history::Entry;
use kubaos_counters::history::Kind;

use super::*;
use crate::fake::FakeRunner;

fn now() -> Zoned {
    Zoned::now()
}

/// Pushups at 3, resetting every day at 9; Coffee at 2, stopping at 2; Steps at 0.
fn list() -> CounterList {
    let mut counters = Counters::default();
    for name in ["Pushups", "Coffee", "Steps"] {
        counters.create(name, &now()).unwrap();
    }
    let daily = AutoReset::Rule(Recurrence::parse("every day at 9").unwrap());
    counters.set_reset("Pushups", daily, &now()).unwrap();
    counters.set("Pushups", 3, &now()).unwrap();
    counters.set_reset("Coffee", AutoReset::Stop(2), &now()).unwrap();
    counters.set("Coffee", 2, &now()).unwrap();
    CounterList::new(counters, None, None)
}

fn typed(name: &str, value: &str) -> BTreeMap<String, String> {
    BTreeMap::from([(name.to_owned(), value.to_owned())])
}

fn none() -> BTreeMap<String, String> {
    BTreeMap::new()
}

fn launched(runner: &FakeRunner) -> Vec<String> {
    runner
        .launched
        .borrow()
        .iter()
        .map(|(argv, _)| argv.join(" "))
        .collect()
}

fn view_of(ran: Ran) -> View {
    match ran {
        Ran::Push(view) => view,
        other => panic!("expected a View, got {other:?}"),
    }
}

fn rows(view: &View) -> Vec<(String, String, Option<String>)> {
    let ViewContent::List { items, filters_itself } = &view.content else {
        panic!("not a list");
    };
    assert!(filters_itself);
    items
        .iter()
        .map(|item| (item.id.clone(), item.title.clone(), item.subtitle.clone()))
        .collect()
}

#[test]
fn new_counter_comes_first_then_a_counter_an_item_in_order() {
    let items = list().items();

    let titles: Vec<&str> = items.iter().map(|item| item.title.as_str()).collect();
    assert_eq!(titles, ["New counter", "Pushups", "Coffee", "Steps"]);
    assert_eq!(items[0].keyword.as_deref(), Some("new-counter"));
    assert!(items[0].actions[0].arguments[0].optional);
    assert_eq!(items[1].id, "counter:1");
    assert_eq!(items[1].subtitle.as_deref(), Some("3 · Resets every day at 9:00"));
    assert_eq!(items[2].subtitle.as_deref(), Some("2 · Stops at 2"));
    assert_eq!(items[3].subtitle.as_deref(), Some("0"));
    assert_eq!(ids().collect::<Vec<_>>(), [NEW_ID]);
}

#[test]
fn a_counter_offers_every_change_with_plus_one_first() {
    let item = &list().items()[1];

    let actions: Vec<(&str, &str)> = item
        .actions
        .iter()
        .map(|action| (action.id.as_str(), action.title.as_str()))
        .collect();
    assert_eq!(
        actions,
        [
            (ADD, "+1"),
            (SUB, "\u{2212}1"),
            (SET, "Set to…"),
            (RESET, "Reset"),
            (RENAME, "Rename…"),
            (AUTO_RESET, "Auto reset…"),
            (HISTORY, "History"),
            (REMOVE, "Remove"),
        ]
    );
    assert!(item.actions[7].confirm);
    assert_eq!(
        item.actions[2].arguments[0].kind,
        kotodama_protocol::ArgumentKind::Number
    );
}

#[test]
fn new_counter_takes_the_query_as_its_name_or_counter() {
    let runner = FakeRunner::default();

    list().create(&typed(NAME, " Squats "), &runner).unwrap();
    list().create(&none(), &runner).unwrap();

    assert_eq!(
        launched(&runner),
        [
            "kubaos-counters new Squats --notify",
            "kubaos-counters new Counter --notify"
        ]
    );
}

#[test]
fn a_taken_name_is_refused_in_the_menu() {
    let runner = FakeRunner::default();

    let err = list().create(&typed(NAME, "pushups"), &runner).unwrap_err();

    assert_eq!(err, "There's a counter named `pushups` already");
    assert!(launched(&runner).is_empty());
}

#[test]
fn each_change_launches_the_command_by_the_counters_name() {
    let runner = FakeRunner::default();
    let list = list();

    list.run("counter:1", ADD, &none(), &runner).unwrap();
    list.run("counter:1", SUB, &none(), &runner).unwrap();
    list.run("counter:1", SET, &typed(VALUE, "12"), &runner).unwrap();
    list.run("counter:1", RESET, &none(), &runner).unwrap();
    list.run("counter:1", RENAME, &typed(NAME, "Push-ups"), &runner)
        .unwrap();
    list.run("counter:1", REMOVE, &none(), &runner).unwrap();

    assert_eq!(
        launched(&runner),
        [
            "kubaos-counters add Pushups --notify",
            "kubaos-counters sub Pushups --notify",
            "kubaos-counters set Pushups 12 --notify",
            "kubaos-counters reset Pushups --notify",
            "kubaos-counters rename Pushups Push-ups --notify",
            "kubaos-counters remove Pushups --notify",
        ]
    );
}

#[test]
fn refusals_read_in_the_menu_and_launch_nothing() {
    let runner = FakeRunner::default();
    let list = list();

    assert_eq!(
        list.run("counter:3", SUB, &none(), &runner).unwrap_err(),
        "Steps is at 0 and can't go lower"
    );
    assert_eq!(
        list.run("counter:2", ADD, &none(), &runner).unwrap_err(),
        "Coffee stopped at 2; reset it to count again"
    );
    assert_eq!(
        list.run("counter:1", SET, &typed(VALUE, "-1"), &runner).unwrap_err(),
        "`-1` isn't a whole number of 0 or more"
    );
    assert!(list.run("counter:1", RENAME, &typed(NAME, "Coffee"), &runner).is_err());
    assert_eq!(
        list.run("counter:9", ADD, &none(), &runner).unwrap_err(),
        "That counter isn't there any more"
    );
    assert!(launched(&runner).is_empty());
}

#[test]
fn auto_reset_opens_with_the_current_rule_the_examples_never_and_a_stop() {
    let view = view_of(
        list()
            .run("counter:1", AUTO_RESET, &none(), &FakeRunner::default())
            .unwrap(),
    );

    let rows = rows(&view);
    assert_eq!(view.id, "counter-reset:1");
    assert_eq!(view.title, "Auto reset for Pushups");
    assert_eq!(rows[0].0, "counter-reset:1:rule:every day at 9:00");
    assert!(rows[0].2.as_deref().unwrap().starts_with("Now · Next "));
    let titles: Vec<&str> = rows[1..].iter().map(|(_, title, _)| title.as_str()).collect();
    assert_eq!(
        titles,
        [
            "Every day at 9:00",
            "Every Monday at 8:00",
            "Every 2 hours",
            "Every month on the 1st at 0:00",
            "Never",
            "When it reaches N…"
        ]
    );
}

#[test]
fn typing_shows_what_it_reads_and_the_next_reset() {
    let view = list().filter("counter-reset:1", "every 30 minutes").unwrap();

    let rows = rows(&view);
    assert_eq!(rows[0].0, "counter-reset:1:rule:every 30 minutes");
    assert_eq!(rows[0].1, "Every 30 minutes");
    assert!(rows[0].2.as_deref().unwrap().starts_with("Next "));
    assert_eq!(rows[1].1, "Never");
}

#[test]
fn a_number_typed_is_a_stop() {
    let view = list().filter("counter-reset:1", "100").unwrap();

    let rows = rows(&view);
    assert_eq!(
        rows[0],
        (
            "counter-reset:1:stop:100".to_owned(),
            "When it reaches 100".to_owned(),
            None
        )
    );
    assert_eq!(rows.len(), 2);
}

#[test]
fn what_does_not_read_says_so_and_offers_the_examples() {
    let view = list().filter("counter-reset:1", "sometimes").unwrap();

    assert_eq!(
        view.title,
        "Auto reset for Pushups: “sometimes” doesn't read; try one of these"
    );
    assert_eq!(rows(&view)[0].1, "Every day at 9:00");
    assert!(list().filter("idle-lock", "x").is_none());
}

#[test]
fn a_choice_sets_the_auto_reset_and_closes_its_list() {
    let runner = FakeRunner::default();
    let list = list();

    let ran = list
        .choose("counter-reset:1:rule:every 30 minutes", CHOOSE, &none(), &runner)
        .unwrap();
    list.choose("counter-reset:1:never", CHOOSE, &none(), &runner).unwrap();
    list.choose("counter-reset:1:stop:100", CHOOSE, &none(), &runner)
        .unwrap();
    list.choose("counter-reset:1:stop", CHOOSE, &typed(VALUE, "5"), &runner)
        .unwrap();

    assert!(matches!(ran, Ran::Pop(view) if view == "counter-reset:1"));
    assert_eq!(
        launched(&runner),
        [
            "kubaos-counters rule Pushups every 30 minutes --notify",
            "kubaos-counters rule Pushups never --notify",
            "kubaos-counters stop Pushups 100 --notify",
            "kubaos-counters stop Pushups 5 --notify",
        ]
    );
    assert!(
        list.choose("counter-reset:1:stop", CHOOSE, &typed(VALUE, "0"), &runner)
            .is_err()
    );
}

#[test]
fn history_lists_the_changes_newest_first_under_their_day() {
    let dir = tempfile::tempdir().unwrap();
    let log = dir.path().join("counters.log");
    let at = |text: &str| -> Timestamp { text.parse().unwrap() };
    history::append(
        &log,
        &[
            Entry::new(at("2026-11-12T08:05:00Z"), 1, Kind::Add, 1),
            Entry::new(at("2026-11-12T08:06:00Z"), 2, Kind::Add, 1),
            Entry::new(at("2026-11-13T08:00:00Z"), 1, Kind::AutoReset, 0),
        ],
    )
    .unwrap();
    let list = CounterList::new(list().counters, None, Some(log));

    let view = view_of(list.run("counter:1", HISTORY, &none(), &FakeRunner::default()).unwrap());

    let ViewContent::Detail { markdown, metadata } = &view.content else {
        panic!("not a Detail View");
    };
    assert_eq!(view.title, "History of Pushups");
    let lines: Vec<&str> = markdown.lines().filter(|line| line.starts_with("- ")).collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].ends_with(" · Auto reset → 0"), "{markdown}");
    assert!(lines[1].ends_with(" · +1 → 1"), "{markdown}");
    assert_eq!(markdown.matches("**").count(), 4, "a heading a day: {markdown}");
    assert_eq!(metadata[0].value, "3");
    assert_eq!(metadata[1].value, "Resets every day at 9:00");
    assert_eq!(metadata[2].value, "2");
}
