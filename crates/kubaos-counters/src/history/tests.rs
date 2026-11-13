use jiff::Zoned;
use jiff::tz::TimeZone;

use super::*;

fn ts(text: &str) -> Timestamp {
    let zoned: Zoned = format!("{text}[Europe/Warsaw]").parse().expect("a zoned time");
    zoned.timestamp()
}

fn warsaw() -> TimeZone {
    TimeZone::get("Europe/Warsaw").expect("the zone")
}

fn change(at: &str, kind: Kind, value: u64) -> Entry {
    Entry::new(ts(at), 1, kind, value)
}

/// A week of Pushups: two days in one week, one in the next, and a change of another counter.
fn week() -> Vec<Entry> {
    vec![
        change("2026-11-12T08:00:00", Kind::Created, 0).named("Pushups".to_owned()),
        change("2026-11-12T08:05:00", Kind::Add, 1),
        change("2026-11-12T08:06:00", Kind::Add, 2),
        change("2026-11-12T20:00:00", Kind::Sub, 1),
        Entry::new(ts("2026-11-12T21:00:00"), 2, Kind::Add, 1),
        change("2026-11-13T09:00:00", Kind::AutoReset, 0),
        change("2026-11-13T10:00:00", Kind::Set, 10),
        change("2026-11-13T10:01:00", Kind::Add, 11),
        change("2026-11-16T07:00:00", Kind::Add, 12),
        change("2026-11-16T07:30:00", Kind::Renamed, 12).named("Push-ups".to_owned()),
    ]
}

fn summary(rows: &[Row]) -> Vec<(String, u64, i64, usize)> {
    rows.iter()
        .map(|row| {
            let at = row.at.to_zoned(warsaw()).strftime("%m-%d %H:%M").to_string();
            (at, row.value, row.added, row.changes)
        })
        .collect()
}

#[test]
fn a_line_reads_back_to_the_same_entry() {
    let entries = [
        change("2026-11-12T08:05:00", Kind::Add, 1),
        change("2026-11-12T08:00:00", Kind::Renamed, 3).named("Push\u{2011}ups · daily".to_owned()),
        change("2026-11-13T09:00:00", Kind::AutoReset, 0),
    ];

    for entry in entries {
        assert_eq!(Entry::parse(&entry.line()), Some(entry.clone()), "{}", entry.line());
    }
    assert_eq!(
        change("2026-11-12T08:05:00", Kind::Add, 1).line(),
        "2026-11-12T07:05:00Z\t1\tadd\t1"
    );
}

#[test]
fn lines_that_do_not_read_are_skipped() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("counters.log");
    std::fs::write(
        &path,
        "garbage\n2026-11-12T07:05:00Z\t1\tadd\t1\n2026-11-12T07:05:00Z\t1\tjump\t1\n",
    )
    .unwrap();

    let entries = read(&path).unwrap();

    assert_eq!(
        entries,
        [Entry::new("2026-11-12T07:05:00Z".parse().unwrap(), 1, Kind::Add, 1)]
    );
}

#[test]
fn appending_keeps_what_is_there_and_removing_takes_one_counter_out() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state/counters.log");
    let entries = week();

    append(&path, &entries[..5]).unwrap();
    append(&path, &entries[5..]).unwrap();
    assert_eq!(read(&path).unwrap(), entries);

    without(&path, 1).unwrap();
    let left = read(&path).unwrap();
    assert_eq!(left, [entries[4].clone()]);
}

#[test]
fn each_change_lists_the_value_changes_newest_first() {
    let rows = history(&week(), 1, Group::Change, Sort::Newest, &warsaw());

    let kinds: Vec<Option<Kind>> = rows.iter().map(|row| row.kind).collect();
    assert_eq!(
        kinds,
        [
            Some(Kind::Add),
            Some(Kind::Add),
            Some(Kind::Set),
            Some(Kind::AutoReset),
            Some(Kind::Sub),
            Some(Kind::Add),
            Some(Kind::Add)
        ]
    );
    assert_eq!(rows[0].value, 12);
}

#[test]
fn oldest_first_is_the_log_order() {
    let rows = history(&week(), 1, Group::Change, Sort::Oldest, &warsaw());

    assert_eq!(rows[0].kind, Some(Kind::Add));
    assert_eq!(rows[0].value, 1);
    assert_eq!(rows.last().unwrap().value, 12);
}

#[test]
fn highest_puts_the_highest_values_first_and_the_newest_of_a_tie() {
    let mut entries = week();
    entries.push(change("2026-11-17T07:00:00", Kind::Set, 12));

    let rows = history(&entries, 1, Group::Change, Sort::Highest, &warsaw());

    assert_eq!(rows[0].at, ts("2026-11-17T07:00:00"));
    assert_eq!(rows[1].at, ts("2026-11-16T07:00:00"));
    assert_eq!(rows.last().unwrap().value, 0);
}

#[test]
fn days_sum_what_was_added_in_them() {
    let rows = history(&week(), 1, Group::Day, Sort::Oldest, &warsaw());

    assert_eq!(
        summary(&rows),
        [
            ("11-12 00:00".to_owned(), 1, 1, 3),
            ("11-13 00:00".to_owned(), 11, 1, 3),
            ("11-16 00:00".to_owned(), 12, 1, 1),
        ]
    );
}

#[test]
fn weeks_start_on_monday_and_months_on_the_first() {
    let weeks = history(&week(), 1, Group::Week, Sort::Newest, &warsaw());
    assert_eq!(
        summary(&weeks),
        [
            ("11-16 00:00".to_owned(), 12, 1, 1),
            ("11-09 00:00".to_owned(), 11, 2, 6)
        ]
    );

    let months = history(&week(), 1, Group::Month, Sort::Newest, &warsaw());
    assert_eq!(summary(&months), [("11-01 00:00".to_owned(), 12, 3, 7)]);
}

#[test]
fn grouped_highest_puts_the_most_added_first() {
    let rows = history(&week(), 1, Group::Week, Sort::Highest, &warsaw());

    assert_eq!(rows[0].added, 2);
}

#[test]
fn days_follow_the_zone_not_utc() {
    let late = vec![change("2026-11-12T23:30:00", Kind::Add, 1)];

    let rows = history(&late, 1, Group::Day, Sort::Newest, &warsaw());

    assert_eq!(rows[0].at, ts("2026-11-12T00:00:00"));
}
