//! `kubaos package`, through the binary in a sandbox with a fake `nix` and a user flake in a
//! temporary home: unfree packages, and the flake's git index. Nothing here switches: every
//! change runs with `--no-switch`.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

use kubaos_core::sandbox;
use kubaos_core::sandbox::Sandbox;

/// The fake `nix` every test runs. It answers with what the home it runs in (`HOME`) wrote for
/// it, and records there that it ran.
const FAKE_NIX: &str =
    "#!/bin/sh\ntouch \"$HOME/nix-ran\"\ncat \"$HOME/nix-stdout\"\nexit \"$(cat \"$HOME/nix-code\")\"\n";

/// A home with a user flake wired to its state directory, and a sandbox to run `kubaos` in, with
/// the fake `nix`.
struct Home {
    dir: sandbox::TempDir,
    sandbox: Sandbox,
}

impl Home {
    fn new() -> Self {
        let dir = sandbox::tempdir().expect("a temporary home");
        let home = dir.path().join("home");
        fs::create_dir_all(&home).expect("make the home");
        let sandbox = Sandbox::new(&dir.path().join("sandbox"), &home, "tester");
        sandbox.without("nix");
        sandbox::program(&sandbox.bin().join("nix"), FAKE_NIX);
        let home = Home { dir, sandbox };
        home.write(
            ".config/kubaos/home/home.nix",
            "{ pkgs, ... }:\n{\n  kubaos.stateDir = ./kubaos.d;\n}\n",
        );
        home
    }

    fn path(&self, path: &str) -> PathBuf {
        self.dir.path().join("home").join(path)
    }

    /// The user flake.
    fn flake(&self) -> PathBuf {
        self.path(".config/kubaos/home")
    }

    fn state(&self, file: &str) -> PathBuf {
        self.flake().join("kubaos.d").join(file)
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

    /// `kubaos` with `args`, in the sandbox, to be given more of its environment.
    fn command(&self, args: &[&str]) -> Command {
        let mut command = self.sandbox.command(env!("CARGO_BIN_EXE_kubaos"));
        command.args(args);
        command
    }

    fn kubaos(&self, args: &[&str]) -> Output {
        self.command(args).output().expect("run kubaos")
    }

    /// `git -C dir` in the sandbox, with a test identity.
    fn git(&self, dir: &Path) -> Command {
        self.sandbox.git(dir)
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

/// Every file under `dir` with its content, to tell whether anything in it changed.
fn snapshot(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files = Vec::new();
    let mut todo = vec![dir.to_path_buf()];
    while let Some(next) = todo.pop() {
        for entry in fs::read_dir(&next).expect("read a folder") {
            let path = entry.expect("a folder entry").path();
            if path.is_dir() {
                todo.push(path);
            } else {
                let bytes = fs::read(&path).expect("read a file");
                files.push((path, bytes));
            }
        }
    }
    files.sort();
    files
}

#[test]
fn adding_a_package_stages_it_in_the_flakes_repository_and_not_in_one_the_environment_names() {
    let home = Home::new();
    home.nix("[false]", 0);
    let git = |dir: &Path, args: &[&str]| {
        let out = home.git(dir).args(args).output().expect("run git");
        assert!(out.status.success(), "git {args:?}: {}", stderr(&out));
        stdout(&out)
    };
    fs::create_dir_all(home.flake()).expect("make the flake");
    git(&home.flake(), &["init", "-q"]);
    let decoy = home.path("decoy");
    fs::create_dir_all(&decoy).expect("make the decoy");
    git(&decoy, &["init", "-q"]);
    fs::write(decoy.join("README"), "decoy\n").expect("write a decoy file");
    git(&decoy, &["add", "README"]);
    git(&decoy, &["commit", "-q", "-m", "decoy"]);
    let before = snapshot(&decoy);

    // As a git hook or `git rebase --exec` runs it: inside another repository's command.
    let out = home
        .command(&["package", "add", "hello", "--no-switch"])
        .env("GIT_DIR", decoy.join(".git"))
        .env("GIT_WORK_TREE", &decoy)
        .env("GIT_INDEX_FILE", decoy.join(".git/index"))
        .output()
        .expect("run kubaos");

    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(git(&home.flake(), &["ls-files"]), "kubaos.d/packages.toml\n");
    let after = snapshot(&decoy);
    let changed: Vec<_> = after
        .iter()
        .filter(|file| !before.contains(file))
        .chain(before.iter().filter(|file| !after.contains(file)))
        .map(|(path, _)| path.display().to_string())
        .collect();
    assert!(changed.is_empty(), "kubaos changed the decoy repository: {changed:?}");
}
