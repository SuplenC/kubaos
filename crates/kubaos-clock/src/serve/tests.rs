use std::fs;
use std::io::Cursor;

use jiff::Zoned;
use serde_json::Value;

use super::Server;
use super::run;
use crate::settings;

fn now() -> Zoned {
    "2026-10-24T10:00:00[Europe/Warsaw]".parse().expect("zoned")
}

fn server(settings: &str) -> (tempfile::TempDir, Server) {
    let dir = tempfile::tempdir().expect("a temp dir");
    if !settings.is_empty() {
        fs::write(dir.path().join(settings::FILE), settings).expect("write the settings");
    }
    let server = Server::new(Some(dir.path().to_path_buf()));
    (dir, server)
}

fn ask(server: &mut Server, request: &str) -> Value {
    serde_json::from_str(&server.handle(request, &now())).expect("a JSON reply")
}

#[test]
fn a_query_is_answered_with_its_id() {
    let (_dir, mut server) = server("holidays = \"pl\"\n");

    let reply = ask(&mut server, r#"{"id":7,"op":"query","text":"days until christmas"}"#);

    assert_eq!(reply["id"], 7);
    assert_eq!(reply["answer"]["title"], "62 days");
    assert_eq!(reply["answer"]["iso"], "P62D");
    assert_eq!(reply["answer"]["date"], "2026-12-25");
}

#[test]
fn a_query_that_is_not_a_date_is_answered_with_null() {
    let (_dir, mut server) = server("");

    let reply = ask(&mut server, r#"{"id":1,"op":"query","text":"firefox"}"#);

    assert!(reply["answer"].is_null());
}

#[test]
fn the_state_has_the_settings_clocks_today_and_countdowns() {
    let (_dir, mut server) = server(
        "zones = [\"tokyo\"]\nholidays = \"pl\"\nbar_countdown = \"Trip\"\n[[countdowns]]\nname = \"Trip\"\nat = \"2026-12-19\"\n",
    );

    let state = ask(&mut server, r#"{"id":1,"op":"state"}"#);

    assert_eq!(state["settings"]["firstWeekday"], 1);
    assert_eq!(state["settings"]["barCountdown"], "Trip");
    assert_eq!(state["settings"]["timeFormat"], "");
    assert_eq!(state["settings"]["dateOrder"], "");
    assert_eq!(state["country"], "Poland");
    assert_eq!(state["here"]["label"], "Warsaw");
    assert_eq!(state["today"]["dayOfYear"], 297);
    assert_eq!(state["clocks"][0]["label"], "Tokyo");
    assert_eq!(state["clocks"][0]["relative"], "+7 h");
    assert_eq!(state["countdowns"][0]["name"], "Trip");
    assert!(state["countdowns"][0]["end"].as_i64().is_some());
    assert!(state["problem"].is_null());
}

#[test]
fn a_broken_settings_file_keeps_the_defaults_and_says_why() {
    let (_dir, mut server) = server("zones = [");

    let state = ask(&mut server, r#"{"id":1,"op":"state"}"#);

    assert!(
        state["problem"]
            .as_str()
            .is_some_and(|problem| problem.contains("clock settings"))
    );
    assert_eq!(state["settings"]["firstWeekday"], 1);
}

#[test]
fn a_month_and_a_day_come_back_for_the_calendar() {
    let (_dir, mut server) = server("holidays = \"pl\"\n");

    let month = ask(&mut server, r#"{"id":1,"op":"month","year":2026,"month":11}"#);
    let day = ask(&mut server, r#"{"id":2,"op":"day","date":"2026-11-11"}"#);

    assert_eq!(month["month"]["title"], "November 2026");
    assert_eq!(month["month"]["weeks"].as_array().map(Vec::len), Some(6));
    assert_eq!(day["day"]["holiday"], "Independence Day");
    assert_eq!(day["day"]["detail"], "in 18 days, a Wednesday · week 46 · day 315");
}

#[test]
fn timers_and_alarms_are_read() {
    let (_dir, mut server) = server("");

    let timer = ask(&mut server, r#"{"id":1,"op":"duration","text":"25m"}"#);
    let alarm = ask(&mut server, r#"{"id":2,"op":"alarm","text":"7:30"}"#);
    let nothing = ask(&mut server, r#"{"id":3,"op":"duration","text":"soon"}"#);

    assert_eq!(timer["ms"], 25 * 60 * 1000);
    assert_eq!(alarm["label"], "07:30 tomorrow");
    assert!(nothing["ms"].is_null());
}

#[test]
fn changes_land_in_the_file_and_the_state_comes_back() {
    let (dir, mut server) = server("");

    let state = ask(&mut server, r#"{"id":1,"op":"zone-add","name":"new york"}"#);
    assert_eq!(state["clocks"][0]["name"], "America/New_York");

    let state = ask(
        &mut server,
        r#"{"id":2,"op":"countdown-add","name":"Trip","when":"19 dec"}"#,
    );
    assert_eq!(state["countdowns"][0]["at"], "2026-12-19");

    let state = ask(&mut server, r#"{"id":3,"op":"set","key":"analog","value":"on"}"#);
    assert_eq!(state["settings"]["analog"], true);

    let state = ask(&mut server, r#"{"id":5,"op":"set","key":"time-format","value":"24h"}"#);
    assert_eq!(state["settings"]["timeFormat"], "24h");

    let state = ask(&mut server, r#"{"id":6,"op":"set","key":"date-order","value":"day-month"}"#);
    assert_eq!(state["settings"]["dateOrder"], "day-month");

    let text = fs::read_to_string(dir.path().join(settings::FILE)).expect("the file");
    assert!(
        text.contains("America/New_York") && text.contains("2026-12-19"),
        "{text}"
    );

    let state = ask(&mut server, r#"{"id":4,"op":"zone-remove","name":"new york"}"#);
    assert_eq!(state["clocks"].as_array().map(Vec::len), Some(0));
}

#[test]
fn a_refused_change_is_an_error_with_its_reason() {
    let (_dir, mut server) = server("");

    let zone = ask(&mut server, r#"{"id":1,"op":"zone-add","name":"atlantis"}"#);
    let countdown = ask(
        &mut server,
        r#"{"id":2,"op":"countdown-add","name":"x","when":"days until christmas"}"#,
    );
    let op = ask(&mut server, r#"{"id":3,"op":"dance"}"#);

    assert_eq!(zone["error"], "no time zone called \"atlantis\"");
    assert_eq!(countdown["id"], 2);
    assert!(countdown["error"].as_str().is_some());
    assert!(op["error"].as_str().is_some());
}

#[test]
fn an_edit_by_hand_is_read_at_the_next_request() {
    let (dir, mut server) = server("analog = false\n");

    fs::write(
        dir.path().join(settings::FILE),
        "analog = true\nfirst_weekday = \"sunday\"\n",
    )
    .expect("edit");
    // The file's time may not move within the filesystem's resolution; a change of size is
    // enough for the test, and a real edit moves the time.
    let filetime = std::time::SystemTime::now() + std::time::Duration::from_secs(5);
    let file = fs::File::options()
        .write(true)
        .open(dir.path().join(settings::FILE))
        .expect("open");
    file.set_modified(filetime).expect("touch");

    let state = ask(&mut server, r#"{"id":1,"op":"state"}"#);

    assert_eq!(state["settings"]["analog"], true);
    assert_eq!(state["settings"]["firstWeekday"], 7);
}

#[test]
fn serve_answers_each_line_and_skips_blank_ones() {
    let (_dir, mut server) = server("");
    let input = Cursor::new("{\"id\":1,\"op\":\"duration\",\"text\":\"5m\"}\n\nnot json\n");
    let mut output = Vec::new();

    run(&mut server, input, &mut output).expect("served");

    let lines: Vec<Value> = String::from_utf8(output)
        .expect("utf-8")
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON"))
        .collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["ms"], 300_000);
    assert!(lines[1]["error"].as_str().is_some());
}
