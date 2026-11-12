//! `kubaos-counters` run as the menu, the key and the shell run it: the files it writes, in what
//! order, and the timer it arms, with `systemctl`, `systemd-run` and `notify-send` stood in for.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::process::Output;

use tempfile::TempDir;

struct Session {
    dir: TempDir,
}

impl Session {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let bin = dir.path().join("bin");
        fs::create_dir_all(&bin).expect("make bin");
        fs::create_dir_all(dir.path().join("run")).expect("make run");
        for program in ["systemctl", "systemd-run", "notify-send"] {
            let path = bin.join(program);
            let script = format!(
                "#!/bin/sh\necho \"{program} $*\" >> \"{}\"\n",
                dir.path().join("calls").display()
            );
            fs::write(&path, script).expect("write a stand-in");
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("make it runnable");
        }
        Session { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        let path = format!(
            "{}:{}",
            self.dir.path().join("bin").display(),
            std::env::var("PATH").unwrap_or_default()
        );
        Command::new(env!("CARGO_BIN_EXE_kubaos-counters"))
            .args(args)
            .env("PATH", path)
            .env("XDG_STATE_HOME", self.dir.path().join("state"))
            .env("XDG_RUNTIME_DIR", self.dir.path().join("run"))
            .env("XDG_CONFIG_HOME", self.dir.path().join("config"))
            .env("TZ", "Europe/Warsaw")
            .output()
            .expect("run kubaos-counters")
    }

    fn ok(&self, args: &[&str]) -> String {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim_end().to_owned()
    }

    fn read(&self, path: &str) -> String {
        fs::read_to_string(self.dir.path().join(path)).unwrap_or_default()
    }

    fn calls(&self) -> Vec<String> {
        self.read("calls").lines().map(str::to_owned).collect()
    }
}

fn kinds(log: &str) -> Vec<String> {
    log.lines()
        .map(|line| line.split('\t').skip(1).take(3).collect::<Vec<_>>().join(" "))
        .collect()
}

#[test]
fn each_change_is_logged_saved_and_shown_to_the_shell() {
    let session = Session::new();

    assert_eq!(session.ok(&["new", "Pushups"]), "New counter: Pushups");
    assert_eq!(session.ok(&["add", "pushups"]), "Pushups: 1");
    assert_eq!(session.ok(&["add", "--last"]), "Pushups: 2");
    assert_eq!(session.ok(&["sub", "Pushups"]), "Pushups: 1");

    assert_eq!(
        kinds(&session.read("state/kubaos/counters.log")),
        ["1 created 0", "1 add 1", "1 add 2", "1 sub 1"]
    );
    assert!(session.read("state/kubaos/counters.toml").contains("value = 1"));
    let live: serde_json::Value = serde_json::from_str(&session.read("run/kubaos-counters.json")).unwrap();
    assert_eq!(live["last"], "Pushups");
    assert_eq!(live["counters"][0]["value"], 1);
}

#[test]
fn a_refusal_exits_one_and_says_why_in_a_toast_when_asked() {
    let session = Session::new();
    session.ok(&["new", "Coffee"]);

    let output = session.run(&["sub", "Coffee", "--notify"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Coffee is at 0"));
    assert!(
        session
            .calls()
            .iter()
            .any(|call| call.starts_with("notify-send") && call.contains("Coffee is at 0"))
    );
}

#[test]
fn usage_errors_exit_two() {
    let session = Session::new();

    assert_eq!(session.run(&["jump"]).status.code(), Some(2));
    assert_eq!(session.run(&["set", "Coffee"]).status.code(), Some(2));
}

#[test]
fn a_rule_arms_the_timer_and_never_stops_it() {
    let session = Session::new();
    session.ok(&["new", "Pushups"]);
    assert!(session.calls().is_empty(), "no rule, no timer: {:?}", session.calls());

    let said = session.ok(&["rule", "Pushups", "every", "day", "at", "9"]);

    assert!(said.starts_with("Pushups: Resets every day at 9:00; next "), "{said}");
    let calls = session.calls();
    assert_eq!(calls[0], "systemctl --user stop kubaos-counters-reset.timer");
    assert!(calls[1].starts_with("systemd-run --user --quiet --collect --unit=kubaos-counters-reset --on-calendar="));
    let program = env!("CARGO_BIN_EXE_kubaos-counters");
    assert!(
        calls[1].ends_with(&format!(" UTC --timer-property=AccuracySec=1s -- {program} tick")),
        "{}",
        calls[1]
    );

    session.ok(&["add", "Pushups"]);
    assert_eq!(
        session.calls().len(),
        2,
        "a change that keeps the next reset leaves the timer"
    );

    session.ok(&["rule", "Pushups", "never"]);
    assert_eq!(session.calls().len(), 3);
    assert_eq!(session.calls()[2], "systemctl --user stop kubaos-counters-reset.timer");
}

#[test]
fn removing_a_counter_takes_its_history_and_logs_that_it_went() {
    let session = Session::new();
    session.ok(&["new", "Pushups"]);
    session.ok(&["new", "Coffee"]);
    session.ok(&["add", "Pushups"]);
    session.ok(&["add", "Coffee"]);

    assert_eq!(session.ok(&["remove", "Pushups"]), "Removed Pushups");

    assert_eq!(
        kinds(&session.read("state/kubaos/counters.log")),
        ["2 created 0", "2 add 1", "1 removed 1"]
    );
    assert!(session.read("state/kubaos/counters.log").ends_with("\tPushups\n"));
    assert_eq!(session.ok(&["list"]), "Coffee  1");
}

#[test]
fn the_log_groups_and_sorts_as_asked() {
    let session = Session::new();
    session.ok(&["new", "Pushups"]);
    session.ok(&["add", "Pushups"]);
    session.ok(&["add", "Pushups"]);
    session.ok(&["set", "Pushups", "9"]);

    let changes: serde_json::Value =
        serde_json::from_str(&session.ok(&["log", "Pushups", "--sort", "highest", "--json"])).unwrap();
    let days: serde_json::Value =
        serde_json::from_str(&session.ok(&["log", "Pushups", "--group", "day", "--json"])).unwrap();

    assert_eq!(changes[0]["kind"], "set");
    assert_eq!(changes[0]["value"], 9);
    assert_eq!(days.as_array().unwrap().len(), 1);
    assert_eq!(days[0]["added"], 2);
    assert_eq!(days[0]["changes"], 3);
    assert_eq!(
        session.run(&["log", "Pushups", "--group", "year"]).status.code(),
        Some(1)
    );
}

#[test]
fn tick_writes_the_shell_state_even_with_no_counters() {
    let session = Session::new();

    session.ok(&["tick"]);

    assert_eq!(
        session.read("run/kubaos-counters.json"),
        "{\"counters\":[],\"last\":\"\"}\n"
    );
    assert!(!Path::new(&session.dir.path().join("state/kubaos/counters.toml")).exists());
}
