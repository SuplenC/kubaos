//! The root helper binary, run as the tests' own user on a checkout of their own: it answers
//! only the requests the action engine takes, and refuses the rest before anything runs.

use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::process::Stdio;

const APP: &str = r#"
name = "Fixture"
summary = "An app for the tests"
category = "system"
tags = ["fixture"]
description = "An app the tests install."
description_from = "https://example.com/fixture"

[[action]]
name = "Install the tool"
kind = "apt"
packages = ["fixture-tool"]

[[action]]
name = "Write its config"
kind = "file"
to = "~/.config/fixture/config.toml"
text = "on\n"
"#;

/// The helper's replies to `requests`, one a line, after its `ready`.
fn serve(requests: &[&str]) -> Vec<String> {
    let dir = tempfile::tempdir().unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755)).unwrap();
    let checkout = dir.path().join("checkout");
    let app = checkout.join("apps/fixture");
    fs::create_dir_all(&app).unwrap();
    fs::create_dir_all(checkout.join("system")).unwrap();
    fs::write(app.join("app.toml"), APP).unwrap();
    fs::write(checkout.join("system/apt-repo.sh"), "#!/bin/sh\n").unwrap();
    for (path, mode) in [
        (checkout.clone(), 0o755),
        (checkout.join("apps"), 0o755),
        (app.clone(), 0o755),
        (app.join("app.toml"), 0o644),
        (checkout.join("system"), 0o755),
        (checkout.join("system/apt-repo.sh"), 0o755),
    ] {
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
    }
    let mut child = Command::new(env!("CARGO_BIN_EXE_kubaos-apps"))
        .arg("serve")
        .env("KUBAOS_APPS_CHECKOUT", dir.path().join("checkout"))
        .env("KUBAOS_APPS_STATE", dir.path().join("state"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    for request in requests {
        writeln!(stdin, "{request}").unwrap();
    }
    drop(stdin);
    let lines: Vec<String> = BufReader::new(child.stdout.take().unwrap())
        .lines()
        .map(Result::unwrap)
        .collect();
    assert!(child.wait().unwrap().success());
    assert!(
        !dir.path().join("state/journals").exists(),
        "a refused request wrote a journal"
    );
    lines
}

#[test]
fn the_helper_says_ready_then_refuses_what_isnt_a_root_step_of_a_catalog_app() {
    let replies = serve(&[
        "rm -rf /",
        r#"{"op":"run","argv":["sh"]}"#,
        r#"{"op":"step","app":"../../etc","step":1,"windows":false}"#,
        r#"{"op":"step","app":"other","step":1,"windows":false}"#,
        r#"{"op":"step","app":"fixture","step":2,"windows":false}"#,
        r#"{"op":"step","app":"fixture","step":1,"windows":false,"packages":["sl"]}"#,
    ]);

    assert_eq!(replies[0], r#"{"reply":"ready"}"#);
    assert_eq!(replies.len(), 7);
    for reply in &replies[1..] {
        assert!(reply.starts_with(r#"{"reply":"refused""#), "{reply}");
    }
}

#[test]
fn the_helper_takes_only_serve() {
    let status = Command::new(env!("CARGO_BIN_EXE_kubaos-apps"))
        .args(["install", "fixture"])
        .stderr(Stdio::null())
        .status()
        .unwrap();

    assert_eq!(status.code(), Some(2));
}
