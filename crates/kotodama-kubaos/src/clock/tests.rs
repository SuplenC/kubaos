use std::fs;

use jiff::Zoned;

use super::COPY;
use super::COPY_ISO;
use super::Dates;
use super::KIND;
use super::SHOW;
use crate::fake::FakeRunner;

fn now() -> Zoned {
    "2026-10-24T10:00:00[Europe/Warsaw]".parse().expect("zoned")
}

fn dates(settings: &str) -> (tempfile::TempDir, Dates) {
    let dir = tempfile::tempdir().expect("a temp dir");
    fs::write(dir.path().join("clock.toml"), settings).expect("write the settings");
    let dates = Dates::new(Some(dir.path().to_path_buf()));
    (dir, dates)
}

#[test]
fn a_date_query_is_answered_in_the_top_row() {
    let (_dir, mut dates) = dates("");

    let result = dates.live("days until christmas", &now()).expect("an answer");

    assert!(result.answer);
    assert_eq!(result.item.title, "62 days");
    assert_eq!(result.item.id, "days until christmas");
    assert_eq!(result.item.kind.name(), KIND);
    assert!(
        result
            .item
            .subtitle
            .as_deref()
            .is_some_and(|line| line.contains("25 December 2026"))
    );
}

#[test]
fn the_answer_is_found_by_its_question_so_it_ranks_above_the_calculators() {
    let (_dir, mut dates) = dates("");

    let result = dates.live("  days until christmas ", &now()).expect("an answer");

    assert_eq!(result.item.keywords, ["days until christmas"]);
}

#[test]
fn other_queries_and_lone_words_are_left_to_the_search() {
    let (_dir, mut dates) = dates("");

    assert!(dates.live("firefox", &now()).is_none());
    assert!(dates.live("next wallpaper", &now()).is_none());
    // `mon` and `may` start app names as often as they mean a day.
    assert!(dates.live("mon", &now()).is_none());
    assert!(dates.live("may", &now()).is_none());
    assert!(dates.live("tomorrow", &now()).is_some());
    assert!(dates.live("2027-03-01", &now()).is_some());
}

#[test]
fn the_holidays_are_the_popups() {
    let (_dir, mut dates) = dates("holidays = \"pl\"\n");

    let result = dates.live("11 nov", &now()).expect("an answer");

    assert!(
        result
            .item
            .subtitle
            .as_deref()
            .is_some_and(|line| line.contains("Independence Day"))
    );
}

#[test]
fn the_actions_copy_the_answer_or_its_iso_form() {
    let (_dir, mut dates) = dates("");
    let runner = FakeRunner::default();
    dates.live("days until christmas", &now()).expect("an answer");

    let answer = dates.run(COPY, "days until christmas", &now(), &runner);
    let iso = dates.run(COPY_ISO, "days until christmas", &now(), &runner);

    assert_eq!(answer, Ok(Some("62 days".to_owned())));
    assert_eq!(iso, Ok(Some("P62D".to_owned())));
}

#[test]
fn an_older_answer_is_worked_out_again() {
    let (_dir, mut dates) = dates("");
    let runner = FakeRunner::default();
    dates.live("next friday", &now()).expect("an answer");

    let iso = dates.run(COPY_ISO, "days until christmas", &now(), &runner);
    let gone = dates.run(COPY, "firefox", &now(), &runner);

    assert_eq!(iso, Ok(Some("P62D".to_owned())));
    assert!(gone.is_err());
}

#[test]
fn show_opens_the_popup_on_the_query() {
    let (_dir, mut dates) = dates("");
    let runner = FakeRunner::default();

    let shown = dates.run(SHOW, "next friday", &now(), &runner);

    assert_eq!(shown, Ok(None));
    assert_eq!(
        runner.launched.borrow().first().map(|(argv, _)| argv.join(" ")),
        Some("qs ipc call clock search next friday".to_owned())
    );
}
