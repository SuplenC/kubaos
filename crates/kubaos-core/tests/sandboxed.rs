//! The workspace's test code starts no process outside the sandbox (`kubaos_core::sandbox`): a
//! `Command::new` in a test, a fake or a fixture fails this, unless a comment just above it says
//! why it may (`Outside the sandbox: …`).

use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// What a reviewed exception says, in a comment at most this many lines above its `Command::new`.
const EXCEPTION: &str = "Outside the sandbox:";
const EXCEPTION_REACH: usize = 6;
/// A process started, spelled in two so this file doesn't find itself.
const START: &str = concat!("Command", "::new(");

fn crates() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Every `.rs` file under `dir`.
fn sources(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read a folder") {
        let path = entry.expect("a folder entry").path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name != "target") {
                sources(&path, found);
            }
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push(path);
        }
    }
}

/// Whether `path`, within a crate, is test code: under a `tests` folder, or a `tests.rs`, a
/// fake or a fixture.
fn is_test_code(path: &Path) -> bool {
    path.components().any(|part| part.as_os_str() == "tests")
        || path
            .file_stem()
            .is_some_and(|stem| ["tests", "fake", "fixture"].iter().any(|name| stem == *name))
}

#[test]
fn test_code_starts_its_processes_in_the_sandbox() {
    let root = crates();
    let mut files = Vec::new();
    sources(&root, &mut files);
    let mut outside = Vec::new();
    for file in files
        .iter()
        .filter(|file| is_test_code(file.strip_prefix(&root).unwrap_or(file)))
    {
        let text = fs::read_to_string(file).expect("read a source");
        let lines: Vec<&str> = text.lines().collect();
        for (at, line) in lines.iter().enumerate() {
            if !line.contains(START) {
                continue;
            }
            let above = &lines[at.saturating_sub(EXCEPTION_REACH)..at];
            if !above.iter().any(|line| line.contains(EXCEPTION)) {
                outside.push(format!(
                    "{}:{}",
                    file.strip_prefix(&root).unwrap_or(file).display(),
                    at + 1
                ));
            }
        }
    }

    assert!(
        outside.is_empty(),
        "a test starts a process outside the sandbox: run it with kubaos_core::sandbox (its \
         `command`, `git` or `output`), or say why it may not above it (`// {EXCEPTION} …`):\n{}",
        outside.join("\n")
    );
}

#[test]
fn a_test_files_command_outside_the_sandbox_is_found() {
    assert!(is_test_code(Path::new("kubaos-core/src/packages/tests.rs")));
    assert!(is_test_code(Path::new("kubaos-cli/tests/package.rs")));
    assert!(is_test_code(Path::new("kubaos-core/src/update/updater/tests/apps.rs")));
    assert!(is_test_code(Path::new("kubaos-power/src/fake.rs")));
    assert!(is_test_code(Path::new("kubaos-core/src/update/fixture.rs")));
    assert!(!is_test_code(Path::new("kubaos-core/src/state/git.rs")));
}
