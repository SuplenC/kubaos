//! `kubaos package` and unfree packages, through the binary with a fake `nix` on `PATH` and a
//! user flake in a temporary home. Nothing here switches: every change runs with `--no-switch`.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::OnceLock;

use tempfile::TempDir;

/// The one fake `nix` every test runs, made before the first `kubaos` starts. It answers with
/// what the home it runs in (`HOME`) wrote for it, and records there that it ran.
///
/// HAZARD: one script for every test, written once. A script a test writes while another test
/// forks is held open for writing by that child until it execs, and running it then fails with
/// "Text file busy".
fn fake_nix() -> &'static Path {
    static BIN: OnceLock<TempDir> = OnceLock::new();
    BIN.get_or_init(|| {
        let bin = tempfile::tempdir().expect("a folder for the fake nix");
        let nix = bin.path().join("nix");
        fs::write(
            &nix,
            "#!/bin/sh\ntouch \"$HOME/nix-ran\"\ncat \"$HOME/nix-stdout\"\nexit \"$(cat \"$HOME/nix-code\")\"\n",
        )
        .expect("write the fake nix");
        fs::set_permissions(&nix, fs::Permissions::from_mode(0o755)).expect("make nix executable");
        bin
    })
    .path()
}

/// A home with a user flake wired to its state directory, and a fake `nix` that answers every
/// `nix eval` with one fixed answer.
struct Home {
    dir: TempDir,
}

impl Home {
    fn new() -> Self {
        let home = Home {
            dir: tempfile::tempdir().expect("a temporary home"),
        };
        home.write(
            ".config/kubaos/home/home.nix",
            "{ pkgs, ... }:\n{\n  kubaos.stateDir = ./kubaos.d;\n}\n",
        );
        home
    }

    fn path(&self, path: &str) -> PathBuf {
        self.dir.path().join(path)
    }

    fn state(&self, file: &str) -> PathBuf {
        self.path(".config/kubaos/home/kubaos.d").join(file)
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.path(path);
        fs::create_dir_all(path.parent().expect("a parent")).expect("make the folder");
        fs::write(path, text).expect("write the file");
    }

    /// What the fake `nix` answers in this home: `stdout`, and exit `code`.
    fn nix(&self, stdout: &str, code: i32) {
        self.write("nix-stdout", stdout);
        self.write("nix-code", &code.to_string());
    }

    fn nix_ran(&self) -> bool {
        self.path("nix-ran").exists()
    }

    fn kubaos(&self, args: &[&str]) -> Output {
        let path = format!("{}:{}", fake_nix().display(), std::env::var("PATH").unwrap_or_default());
        Command::new(env!("CARGO_BIN_EXE_kubaos"))
            .args(args)
            .env("HOME", self.dir.path())
            .env("XDG_CONFIG_HOME", self.path(".config"))
            .env("XDG_STATE_HOME", self.path(".local/state"))
            .env("PATH", path)
            .output()
            .expect("run kubaos")
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

#[test]
fn adding_an_unfree_package_says_so() {
    let home = Home::new();
    home.nix("[true]", 0);

    let out = home.kubaos(&["package", "add", "discord", "--no-switch"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "added discord (unfree)\n");
    assert!(read(&home.state("packages.toml")).contains("\"discord\""));
}

#[test]
fn with_unfree_packages_off_adding_one_refuses_before_writing_anything() {
    let home = Home::new();
    home.write(".config/kubaos/home/kubaos.d/nixpkgs.toml", "allow_unfree = false\n");
    home.nix("[true]", 0);

    let out = home.kubaos(&["package", "add", "discord", "--no-switch"]);

    assert_eq!(out.status.code(), Some(1));
    let message = stderr(&out);
    assert!(message.contains("discord is unfree"), "{message}");
    assert!(message.contains("allow_unfree in kubaos.d/nixpkgs.toml"), "{message}");
    assert!(message.contains("kubaos package unfree on"), "{message}");
    assert!(!home.state("packages.toml").exists());
}

#[test]
fn with_unfree_packages_off_a_failed_check_refuses_the_add() {
    let home = Home::new();
    home.write(".config/kubaos/home/kubaos.d/nixpkgs.toml", "allow_unfree = false\n");
    home.nix("", 1);

    let out = home.kubaos(&["package", "add", "hello", "--no-switch"]);

    assert_eq!(out.status.code(), Some(1));
    assert!(!home.state("packages.toml").exists());
}

#[test]
fn with_unfree_packages_on_a_failed_check_only_warns() {
    let home = Home::new();
    home.nix("", 1);

    let out = home.kubaos(&["package", "add", "hello", "--no-switch"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("not telling unfree packages apart"),
        "{}",
        stderr(&out)
    );
    assert!(read(&home.state("packages.toml")).contains("\"hello\""));
}

#[test]
fn the_list_marks_unfree_packages() {
    let home = Home::new();
    home.write(
        ".config/kubaos/home/kubaos.d/packages.toml",
        "packages = [\"discord\", \"hello\"]\n",
    );
    home.nix("[true,false]", 0);

    let text = home.kubaos(&["package", "list"]);
    let json = home.kubaos(&["package", "list", "--json"]);

    assert_eq!(
        stdout(&text),
        "discord  (packages.toml)  unfree\nhello  (packages.toml)\n"
    );
    let value: serde_json::Value = serde_json::from_slice(&json.stdout).expect("list --json is JSON");
    assert_eq!(
        value,
        serde_json::json!({
            "discord": { "from": ["packages.toml"], "unfree": true },
            "hello": { "from": ["packages.toml"], "unfree": false },
        })
    );
}

#[test]
fn turning_unfree_packages_off_refuses_while_a_suite_lists_one_and_says_how_to_remove_it() {
    let home = Home::new();
    home.write(
        ".config/kubaos/home/kubaos.d/suites/chat.toml",
        "packages = [\"discord\"]\n",
    );
    home.nix("[true]", 0);

    let out = home.kubaos(&["package", "unfree", "off", "--no-switch"]);

    assert_eq!(out.status.code(), Some(1));
    let message = stderr(&out);
    assert!(message.contains("discord is unfree (suites/chat.toml)"), "{message}");
    assert!(message.contains("kubaos suite remove chat"), "{message}");
    assert!(!home.state("nixpkgs.toml").exists());
}

#[test]
fn turning_unfree_packages_off_when_they_are_off_asks_nix_nothing() {
    let home = Home::new();
    home.write(".config/kubaos/home/kubaos.d/nixpkgs.toml", "allow_unfree = false\n");
    home.write(".config/kubaos/home/kubaos.d/packages.toml", "packages = [\"hello\"]\n");
    home.nix("", 1);

    let out = home.kubaos(&["package", "unfree", "off", "--no-switch"]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "unfree packages: already off\n");
    assert!(!home.nix_ran());
}

#[test]
fn turning_unfree_packages_off_and_on_prints_and_writes_the_setting() {
    let home = Home::new();
    home.nix("[]", 0);

    let off = home.kubaos(&["package", "unfree", "off", "--no-switch"]);
    let status = home.kubaos(&["package", "unfree"]);
    let on = home.kubaos(&["package", "unfree", "on", "--no-switch"]);

    assert_eq!(stdout(&off), "unfree packages: off, free ones only\n");
    assert_eq!(stdout(&status), "off\n");
    assert_eq!(stdout(&on), "unfree packages: on\n");
    assert!(read(&home.state("nixpkgs.toml")).contains("allow_unfree = true"));
}
