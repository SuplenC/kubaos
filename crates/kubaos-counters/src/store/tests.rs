use jiff::Zoned;

use super::*;

fn at(text: &str) -> Zoned {
    format!("{text}[Europe/Warsaw]").parse().expect("a zoned time")
}

fn now() -> Zoned {
    at("2026-11-12T10:00:00")
}

fn kinds(entries: &[Entry]) -> Vec<(Kind, u64)> {
    entries.iter().map(|entry| (entry.kind, entry.value)).collect()
}

fn with(names: &[&str]) -> Counters {
    let mut counters = Counters::default();
    for name in names {
        counters.create(name, &now()).expect("create");
    }
    counters
}

fn value(counters: &Counters, name: &str) -> u64 {
    counters.find(name).expect("the counter").value
}

fn rule(text: &str) -> AutoReset {
    AutoReset::Rule(Recurrence::parse(text).expect("a rule"))
}

#[test]
fn a_new_counter_starts_at_zero_at_the_end_and_is_logged() {
    let mut counters = with(&["Pushups"]);

    let entries = counters.create("Coffee", &now()).unwrap();

    let names: Vec<&str> = counters.all().iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Pushups", "Coffee"]);
    assert_eq!(value(&counters, "Coffee"), 0);
    assert_eq!(kinds(&entries), [(Kind::Created, 0)]);
    assert_eq!(entries[0].name.as_deref(), Some("Coffee"));
    assert_eq!(counters.last().unwrap().name, "Coffee");
}

#[test]
fn an_empty_name_is_counter_then_numbered() {
    let mut counters = with(&["", "  "]);
    counters.create("", &now()).unwrap();

    let names: Vec<&str> = counters.all().iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Counter", "Counter 2", "Counter 3"]);
}

#[test]
fn names_are_unique_whatever_the_case_and_hold_no_tabs() {
    let mut counters = with(&["Pushups"]);

    assert!(matches!(counters.create("pushups", &now()), Err(ChangeError::Taken(_))));
    assert!(matches!(counters.create("a\tb", &now()), Err(ChangeError::BadName)));
    assert!(matches!(counters.rename("Pushups", "", &now()), Err(ChangeError::BadName)));
}

#[test]
fn counters_are_found_exactly_or_whatever_the_case() {
    let counters = with(&["Pushups"]);

    assert_eq!(counters.find("PUSHUPS").unwrap().name, "Pushups");
    assert!(matches!(counters.find("Squats"), Err(ChangeError::NoSuch(_))));
}

#[test]
fn ids_are_never_used_twice() {
    let mut counters = with(&["A", "B"]);
    counters.remove("B", &now()).unwrap();

    counters.create("C", &now()).unwrap();

    assert_eq!(counters.find("C").unwrap().id, 3);
}

#[test]
fn values_never_go_below_zero() {
    let mut counters = with(&["Pushups"]);

    assert!(matches!(counters.sub("Pushups", &now()), Err(ChangeError::AtZero(_))));

    counters.add("Pushups", &now()).unwrap();
    let entries = counters.sub("Pushups", &now()).unwrap();
    assert_eq!(kinds(&entries), [(Kind::Sub, 0)]);
}

#[test]
fn a_counter_stops_at_its_stop() {
    let mut counters = with(&["Coffee"]);
    counters.set_reset("Coffee", AutoReset::Stop(2), &now()).unwrap();
    counters.add("Coffee", &now()).unwrap();
    counters.add("Coffee", &now()).unwrap();

    let refused = counters.add("Coffee", &now());

    assert!(matches!(refused, Err(ChangeError::Stopped { stop: 2, .. })));
    assert_eq!(value(&counters, "Coffee"), 2);
}

#[test]
fn a_value_set_past_the_stop_is_the_stop() {
    let mut counters = with(&["Coffee"]);
    counters.set_reset("Coffee", AutoReset::Stop(4), &now()).unwrap();

    let entries = counters.set("Coffee", 10, &now()).unwrap();

    assert_eq!(kinds(&entries), [(Kind::Set, 4)]);
}

#[test]
fn a_stop_below_the_value_brings_it_down() {
    let mut counters = with(&["Coffee"]);
    counters.set("Coffee", 10, &now()).unwrap();

    let entries = counters.set_reset("Coffee", AutoReset::Stop(4), &now()).unwrap();

    assert_eq!(kinds(&entries), [(Kind::Set, 4)]);
    assert!(matches!(
        counters.set_reset("Coffee", AutoReset::Stop(0), &now()),
        Err(ChangeError::StopAtZero)
    ));
}

#[test]
fn changes_that_change_nothing_log_nothing() {
    let mut counters = with(&["Pushups"]);

    assert!(counters.reset("Pushups", &now()).unwrap().is_empty());
    assert!(counters.set("Pushups", 0, &now()).unwrap().is_empty());
    assert!(counters.rename("Pushups", "Pushups", &now()).unwrap().is_empty());
}

#[test]
fn each_change_logs_its_kind_and_the_value_after_it() {
    let mut counters = with(&["Pushups"]);

    let mut entries = counters.add("Pushups", &now()).unwrap();
    entries.extend(counters.add("Pushups", &now()).unwrap());
    entries.extend(counters.set("Pushups", 7, &now()).unwrap());
    entries.extend(counters.sub("Pushups", &now()).unwrap());
    entries.extend(counters.reset("Pushups", &now()).unwrap());
    entries.extend(counters.rename("Pushups", "Push-ups", &now()).unwrap());

    assert_eq!(
        kinds(&entries),
        [
            (Kind::Add, 1),
            (Kind::Add, 2),
            (Kind::Set, 7),
            (Kind::Sub, 6),
            (Kind::Reset, 0),
            (Kind::Renamed, 0)
        ]
    );
    assert_eq!(entries[5].name.as_deref(), Some("Push-ups"));
}

#[test]
fn the_key_adds_to_the_counter_changed_last_or_the_only_one() {
    let mut counters = Counters::default();
    assert!(matches!(counters.last_name(), Err(ChangeError::NoCounters)));

    counters.create("Pushups", &now()).unwrap();
    assert_eq!(counters.last_name().unwrap(), "Pushups");

    counters.create("Coffee", &now()).unwrap();
    counters.add("Pushups", &now()).unwrap();
    assert_eq!(counters.last_name().unwrap(), "Pushups");

    counters.remove("Pushups", &now()).unwrap();
    assert_eq!(counters.last_name().unwrap(), "Coffee");

    counters.create("Soda", &now()).unwrap();
    counters.create("Beer", &now()).unwrap();
    counters.remove("Beer", &now()).unwrap();
    assert!(matches!(counters.last_name(), Err(ChangeError::NoLast)));
}

#[test]
fn a_counter_moves_to_a_place_counted_from_one() {
    let mut counters = with(&["A", "B", "C"]);

    counters.move_to("C", 1).unwrap();
    counters.move_to("A", 99).unwrap();

    let names: Vec<&str> = counters.all().iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["C", "B", "A"]);
}

#[test]
fn a_rule_resets_on_time_and_logs_it_at_the_time_it_came_due() {
    let mut counters = with(&["Pushups"]);
    counters
        .set_reset("Pushups", rule("every day at 9"), &at("2026-11-12T10:00:00"))
        .unwrap();
    counters.set("Pushups", 5, &now()).unwrap();
    assert_eq!(
        counters.next_reset().unwrap(),
        at("2026-11-13T09:00:00").timestamp()
    );

    assert!(counters.catch_up(&at("2026-11-13T08:59:59")).is_empty());
    let entries = counters.catch_up(&at("2026-11-13T09:00:00"));

    assert_eq!(kinds(&entries), [(Kind::AutoReset, 0)]);
    assert_eq!(entries[0].at, at("2026-11-13T09:00:00").timestamp());
    assert_eq!(
        counters.next_reset().unwrap(),
        at("2026-11-14T09:00:00").timestamp()
    );
}

#[test]
fn resets_missed_while_asleep_or_off_happen_once_at_the_next_command() {
    let mut counters = with(&["Pushups"]);
    counters
        .set_reset("Pushups", rule("every 2 hours"), &at("2026-11-12T10:30:00"))
        .unwrap();
    counters.set("Pushups", 5, &now()).unwrap();

    let entries = counters.catch_up(&at("2026-11-15T07:15:00"));

    assert_eq!(kinds(&entries), [(Kind::AutoReset, 0)]);
    assert_eq!(entries[0].at, at("2026-11-12T12:00:00").timestamp());
    assert_eq!(value(&counters, "Pushups"), 0);
    assert_eq!(
        counters.next_reset().unwrap(),
        at("2026-11-15T08:00:00").timestamp()
    );
}

#[test]
fn a_counter_at_zero_resets_without_a_log_line() {
    let mut counters = with(&["Pushups"]);
    counters
        .set_reset("Pushups", rule("every minute"), &at("2026-11-12T10:00:00"))
        .unwrap();

    assert!(counters.catch_up(&at("2026-11-12T10:05:00")).is_empty());
    assert_eq!(
        counters.next_reset().unwrap(),
        at("2026-11-12T10:06:00").timestamp()
    );
}

#[test]
fn never_takes_the_rule_and_its_next_reset_away() {
    let mut counters = with(&["Pushups"]);
    counters.set_reset("Pushups", rule("daily"), &now()).unwrap();

    counters.set_reset("Pushups", AutoReset::Never, &now()).unwrap();

    assert_eq!(counters.next_reset(), None);
}

#[test]
fn the_file_keeps_everything_and_reads_back_the_same() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state/counters.toml");
    let mut counters = with(&["Pushups", "Coffee", "Steps"]);
    counters.set_reset("Pushups", rule("mondays 8am"), &now()).unwrap();
    counters.set_reset("Coffee", AutoReset::Stop(4), &now()).unwrap();
    counters.set("Steps", 9000, &now()).unwrap();

    counters.save(&path).unwrap();
    let read = Counters::load(&path).unwrap();

    assert_eq!(read, counters);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("rule = \"every Monday at 8:00\""), "{text}");
    assert!(text.contains("stop = 4"), "{text}");
}

#[test]
fn no_file_is_no_counters() {
    let dir = tempfile::tempdir().unwrap();

    let counters = Counters::load(&dir.path().join("counters.toml")).unwrap();

    assert!(counters.all().is_empty());
}

#[test]
fn a_file_with_a_rule_it_cannot_read_says_which_counter() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("counters.toml");
    std::fs::write(&path, "[[counters]]\nid = 1\nname = \"Pushups\"\nrule = \"sometimes\"\n").unwrap();

    let err = Counters::load(&path).unwrap_err();

    assert!(matches!(err, LoadError::Invalid { ref name, .. } if name == "Pushups"));
}

#[test]
fn a_hand_added_rule_gets_its_next_reset_at_the_next_command() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("counters.toml");
    std::fs::write(&path, "[[counters]]\nid = 4\nname = \"Pushups\"\nvalue = 3\nrule = \"daily\"\n").unwrap();
    let mut counters = Counters::load(&path).unwrap();

    assert!(counters.catch_up(&now()).is_empty());

    assert_eq!(value(&counters, "Pushups"), 3);
    assert_eq!(
        counters.next_reset().unwrap(),
        at("2026-11-13T00:00:00").timestamp()
    );
    counters.create("Coffee", &now()).unwrap();
    assert_eq!(counters.find("Coffee").unwrap().id, 5);
}
