use std::fs;
use std::io::Cursor;

use serde_json::Value;

use super::Mode;
use super::Next;
use super::Server;
use super::read_stream;
use super::run;
use crate::fixture::Machine;

fn server(machine: &Machine) -> (tempfile::TempDir, Server) {
    let dir = tempfile::tempdir().expect("a temp dir");
    let server = Server::new(
        machine.root(),
        Some(dir.path().join("sysmon")),
        Some(dir.path().join("kubaos-sysmon.json")),
    );
    (dir, server)
}

fn ask(server: &mut Server, request: &str) -> (Value, Next) {
    let (reply, next) = server.handle(request, 5000);
    (serde_json::from_str(&reply).expect("a JSON reply"), next)
}

fn json(line: &str) -> Value {
    serde_json::from_str(line).expect("a JSON line")
}

#[test]
fn a_tick_writes_a_sample_line_and_the_stream_file() {
    let machine = Machine::laptop();
    let (dir, mut server) = server(&machine);

    let lines = server.tick(1000);

    assert_eq!(lines.len(), 1);
    let sample = json(&lines[0]);
    assert_eq!(sample["type"], "sample");
    assert_eq!(sample["cpu"]["count"], 2);
    assert_eq!(sample["detail"], false);
    let stream = read_stream(&dir.path().join("kubaos-sysmon.json"), 2000, 10_000).expect("the stream file");
    assert_eq!(stream.t, 1000);
    assert!(read_stream(&dir.path().join("kubaos-sysmon.json"), 60_000, 10_000).is_none());
}

#[test]
fn the_popup_mode_samples_in_detail_with_processes() {
    let machine = Machine::laptop();
    machine.process(300, "firefox", 10, 100);
    let (_dir, mut server) = server(&machine);

    let (reply, next) = ask(&mut server, r#"{"id":1,"op":"mode","mode":"popup"}"#);
    let sample = json(&server.tick(5500)[0]);

    assert_eq!(reply["id"], 1);
    assert_eq!(reply["mode"], "popup");
    assert_eq!(next, Next::Soon);
    assert_eq!(server.mode(), Mode::Popup);
    assert_eq!(sample["detail"], true);
    assert_eq!(sample["top"]["memory"][0]["name"], "firefox");
}

#[test]
fn the_quiet_mode_samples_the_pill_every_10_seconds() {
    let machine = Machine::laptop();
    let (_dir, mut server) = server(&machine);

    let (reply, next) = ask(&mut server, r#"{"id":1,"op":"mode","mode":"quiet"}"#);

    assert_eq!(reply["mode"], "quiet");
    assert_eq!(next, Next::Now);
    assert_eq!(server.mode().interval(), Some(std::time::Duration::from_secs(10)));
}

#[test]
fn pausing_stops_the_pace_and_resuming_starts_the_counters_afresh() {
    let machine = Machine::laptop();
    let (_dir, mut server) = server(&machine);

    let (_, paused) = ask(&mut server, r#"{"id":1,"op":"mode","mode":"paused"}"#);
    let interval = server.mode().interval();
    let (_, resumed) = ask(&mut server, r#"{"id":2,"op":"mode","mode":"pill"}"#);

    assert_eq!(paused, Next::Keep);
    assert_eq!(interval, None);
    assert_eq!(resumed, Next::Soon);
}

#[test]
fn an_unknown_mode_or_op_is_an_error_with_the_id() {
    let machine = Machine::laptop();
    let (_dir, mut server) = server(&machine);

    let (mode, _) = ask(&mut server, r#"{"id":3,"op":"mode","mode":"fast"}"#);
    let (op, _) = ask(&mut server, r#"{"id":4,"op":"dance"}"#);
    let (reply, _) = server.handle("not json", 0);

    assert_eq!(mode["id"], 3);
    assert!(
        mode["error"]
            .as_str()
            .is_some_and(|error| error.contains("pill, popup or paused"))
    );
    assert_eq!(op["id"], 4);
    assert!(json(&reply)["error"].as_str().is_some());
}

#[test]
fn a_process_is_described_and_a_gone_one_says_so() {
    let machine = Machine::laptop();
    machine.process(300, "firefox", 10, 100);
    let (_dir, mut server) = server(&machine);
    server.tick(1000);

    let (found, _) = ask(&mut server, r#"{"id":5,"op":"process","pid":300}"#);
    let (gone, _) = ask(&mut server, r#"{"id":6,"op":"process","pid":301}"#);

    assert_eq!(found["process"]["name"], "firefox");
    assert_eq!(found["process"]["started"], 1_700_000_001);
    assert_eq!(gone["error"], "process 301 is gone");
}

#[test]
fn init_is_refused_a_signal() {
    let machine = Machine::laptop();
    let (_dir, mut server) = server(&machine);

    let (reply, _) = ask(&mut server, r#"{"id":7,"op":"signal","pid":1,"signal":"kill"}"#);

    assert_eq!(reply["error"], "process 1 can't be ended from here");
}

#[test]
fn settings_are_read_and_changed() {
    let machine = Machine::laptop();
    let (dir, mut server) = server(&machine);

    let (before, _) = ask(&mut server, r#"{"id":8,"op":"settings"}"#);
    let (after, _) = ask(&mut server, r#"{"id":9,"op":"set","key":"metrics","value":"temp gpu"}"#);
    let (refused, _) = ask(&mut server, r#"{"id":10,"op":"set","key":"style","value":"pie"}"#);

    assert_eq!(before["settings"]["pill"], true);
    assert_eq!(before["settings"]["metrics"], serde_json::json!(["cpu", "ram"]));
    assert_eq!(after["settings"]["metrics"], serde_json::json!(["temp", "gpu"]));
    assert!(
        refused["error"]
            .as_str()
            .is_some_and(|error| error.contains("numbers, bars, sparklines"))
    );
    assert!(dir.path().join("sysmon/sysmon.toml").exists());
}

#[test]
fn a_hand_edit_of_the_settings_is_read_at_the_next_tick() {
    let machine = Machine::laptop();
    let (dir, mut server) = server(&machine);
    fs::create_dir_all(dir.path().join("sysmon")).expect("make the folder");
    fs::write(dir.path().join("sysmon/sysmon.toml"), "alerts = false\n").expect("write the settings");

    server.tick(1000);
    let (reply, _) = ask(&mut server, r#"{"id":11,"op":"settings"}"#);

    assert_eq!(reply["settings"]["alerts"], false);
}

#[test]
fn a_broken_settings_file_keeps_the_defaults_and_says_why() {
    let machine = Machine::laptop();
    let (dir, mut server) = server(&machine);
    fs::create_dir_all(dir.path().join("sysmon")).expect("make the folder");
    fs::write(dir.path().join("sysmon/sysmon.toml"), "pill = maybe\n").expect("write the settings");

    let (reply, _) = ask(&mut server, r#"{"id":12,"op":"settings"}"#);

    assert_eq!(reply["settings"]["pill"], true);
    assert!(
        reply["problem"]
            .as_str()
            .is_some_and(|problem| problem.contains("parsing"))
    );
}

#[test]
fn an_alert_is_a_line_after_the_sample() {
    let machine = Machine::laptop();
    machine.write("proc/self/mounts", "");
    machine.meminfo_percent_used(97);
    machine.process(300, "chrome", 10, 1_000_000);
    let (_dir, mut server) = server(&machine);

    server.tick(0);
    server.tick(2000);
    let lines = server.tick(12_000);

    let alert = json(lines.last().expect("lines"));
    assert_eq!(alert["type"], "alert");
    assert_eq!(alert["id"], "memory");
    assert!(
        alert["title"]
            .as_str()
            .is_some_and(|title| title.contains("chrome uses"))
    );
}

#[test]
fn alerts_off_fire_nothing() {
    let machine = Machine::laptop();
    machine.write("proc/self/mounts", "");
    machine.meminfo_percent_used(97);
    let (dir, mut server) = server(&machine);
    fs::create_dir_all(dir.path().join("sysmon")).expect("make the folder");
    fs::write(dir.path().join("sysmon/sysmon.toml"), "alerts = false\n").expect("write the settings");

    server.tick(0);
    let lines = server.tick(60_000);

    assert_eq!(lines.len(), 1);
}

#[test]
fn a_hot_cpu_has_the_pill_scan_processes_to_name_the_culprit() {
    let machine = Machine::laptop();
    machine.write("proc/self/mounts", "");
    machine.meminfo_percent_used(40);
    machine.process(300, "cc1plus", 10, 100);
    let (_dir, mut server) = server(&machine);

    let cool = json(&server.tick(0)[0]);
    machine.cpu_temp(96);
    let first_hot = json(&server.tick(2000)[0]);
    let second_hot = json(&server.tick(4000)[0]);

    assert!(cool["top"].is_null());
    assert!(first_hot["top"].is_null());
    assert_eq!(second_hot["top"]["cpu"][0]["name"], "cc1plus");
}

#[test]
fn run_samples_at_once_answers_requests_and_stops_when_input_ends() {
    let machine = Machine::laptop();
    let (_dir, mut server) = server(&machine);
    let input = Cursor::new(b"{\"id\":1,\"op\":\"settings\"}\n".to_vec());
    let mut output = Vec::new();

    run(&mut server, input, &mut output).expect("serve");

    let text = String::from_utf8(output).expect("UTF-8");
    assert!(text.lines().any(|line| json(line)["id"] == 1), "{text}");
}
