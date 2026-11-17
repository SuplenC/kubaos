//! The workspace's test code starts no process outside the sandbox (`kubaos_core::sandbox`): a
//! `Command::new` (under any name it is imported as) or a `kubaos_git::at` or `anywhere` in a
//! test, a fake, a fixture or any other module compiled only for tests fails this, unless a
//! comment just above it says why it may (`Outside the sandbox: …`). One such comment covers the
//! one start after it. Strings and comments are not code, so neither a message nor a doc that
//! names `Command::new` counts.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// What a reviewed exception says, in a comment at most this many lines above its start.
const EXCEPTION: &str = "Outside the sandbox:";
const EXCEPTION_REACH: usize = 6;
/// The sandbox itself, whose `Command::new` is the wall.
const SANDBOX: &str = "kubaos-core/src/sandbox.rs";

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

/// `text` with every comment and the inside of every string and character literal blanked,
/// line for line, so a pattern found in it is code.
fn code(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let blank = |c: char| if c == '\n' { '\n' } else { ' ' };
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c == '/' && next == Some('/') {
            while i < chars.len() && chars[i] != '\n' {
                out.push(' ');
                i += 1;
            }
        } else if c == '/' && next == Some('*') {
            let mut depth = 0;
            while i < chars.len() {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    out.push_str("  ");
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    out.push_str("  ");
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    out.push(blank(chars[i]));
                    i += 1;
                }
            }
        } else if c == 'r' && matches!(next, Some('"' | '#')) && !is_ident(chars.get(i.wrapping_sub(1)).copied()) {
            let hashes = chars[i + 1..].iter().take_while(|&&h| h == '#').count();
            if chars.get(i + 1 + hashes) != Some(&'"') {
                out.push(c);
                i += 1;
                continue;
            }
            let end: Vec<char> = std::iter::once('"').chain(std::iter::repeat_n('#', hashes)).collect();
            out.push('"');
            i += 2 + hashes;
            while i < chars.len() && !chars[i..].starts_with(&end) {
                out.push(blank(chars[i]));
                i += 1;
            }
            out.push('"');
            i += end.len();
        } else if c == '"' {
            out.push('"');
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' {
                    out.push(' ');
                    i += 1;
                }
                if let Some(&escaped) = chars.get(i) {
                    out.push(blank(escaped));
                }
                i += 1;
            }
            out.push('"');
            i += 1;
        } else if c == '\'' && next == Some('\\') {
            // An escaped character literal: `'\''`, `'\n'`, `'\u{2026}'`.
            out.push_str("''");
            i = chars
                .get(i + 3..)
                .and_then(|rest| rest.iter().position(|&q| q == '\''))
                .map_or(chars.len(), |at| i + 4 + at);
        } else if c == '\'' && chars.get(i + 2) == Some(&'\'') {
            // A character literal, not a lifetime: `'"'`.
            out.push_str("''");
            i += 3;
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

fn is_ident(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_alphanumeric() || c == '_')
}

/// The names `std::process::Command` goes by in `code`: its own and any it is imported as.
fn command_names(code: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::from(["Command".to_owned()]);
    for (at, _) in code.match_indices("Command as ") {
        let alias: String = code[at + "Command as ".len()..]
            .chars()
            .take_while(|&c| is_ident(Some(c)))
            .collect();
        if !alias.is_empty() {
            names.insert(alias);
        }
    }
    names
}

/// The patterns that start a process, or make a git command, in `code`.
fn starts(code: &str) -> Vec<String> {
    let mut patterns: Vec<String> = command_names(code)
        .into_iter()
        .map(|name| format!("{name}::new("))
        .collect();
    patterns.extend(["kubaos_git::at", "kubaos_git::anywhere"].map(str::to_owned));
    patterns
}

/// How many of `patterns` start on `line`, each at a word's start.
fn count(line: &str, patterns: &[String]) -> usize {
    patterns
        .iter()
        .map(|pattern| {
            line.match_indices(pattern.as_str())
                .filter(|(at, _)| !is_ident(line[..*at].chars().next_back()))
                .count()
        })
        .sum()
}

/// The 1-based lines of `text` whose starts have no exception comment of their own above them.
fn unexplained(text: &str) -> Vec<usize> {
    let code = code(text);
    let patterns = starts(&code);
    let mut marker: Option<usize> = None;
    let mut found = Vec::new();
    for (at, (raw, line)) in text.lines().zip(code.lines()).enumerate() {
        if raw.contains(&format!("// {EXCEPTION}")) {
            marker = Some(at);
        }
        for _ in 0..count(line, &patterns) {
            match marker.take() {
                Some(above) if at - above <= EXCEPTION_REACH => {}
                _ => found.push(at + 1),
            }
        }
    }
    found
}

/// The files of the modules `file` declares under a `cfg` that names `test` (`#[cfg(test)]`,
/// `#[cfg(any(test, feature = "…"))]`), with the folders their own modules live in.
fn test_modules(file: &Path, text: &str) -> Vec<PathBuf> {
    let dir = file.parent().expect("a source's folder");
    let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
    let children = if ["lib", "main", "mod"].contains(&stem) {
        dir.to_path_buf()
    } else {
        dir.join(stem)
    };
    let lines: Vec<&str> = text.lines().collect();
    let mut found = Vec::new();
    for (at, line) in lines.iter().enumerate() {
        let line = line.trim();
        let Some(name) = line
            .trim_start_matches("pub(crate) ")
            .trim_start_matches("pub(super) ")
            .trim_start_matches("pub ")
            .strip_prefix("mod ")
            .and_then(|rest| rest.strip_suffix(';'))
        else {
            continue;
        };
        let attributes = lines[..at]
            .iter()
            .rev()
            .take_while(|l| l.trim_start().starts_with("#["));
        if attributes.into_iter().any(|a| a.contains("cfg(") && names_test(a)) {
            found.push(children.join(format!("{name}.rs")));
            found.push(children.join(name));
        }
    }
    found
}

fn names_test(attribute: &str) -> bool {
    attribute
        .match_indices("test")
        .any(|(at, _)| !is_ident(attribute[..at].chars().next_back()) && !is_ident(attribute[at + 4..].chars().next()))
}

/// Whether `path`, within a crate, is test code by its name: under a `tests` folder, or a
/// `tests.rs`, a fake or a fixture.
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
    let texts: Vec<(PathBuf, String)> = files
        .into_iter()
        .map(|file| {
            let text = fs::read_to_string(&file).expect("read a source");
            (file, text)
        })
        .collect();
    let gated: Vec<PathBuf> = texts.iter().flat_map(|(file, text)| test_modules(file, text)).collect();
    let mut outside = Vec::new();
    for (file, text) in &texts {
        let rel = file.strip_prefix(&root).unwrap_or(file);
        let test_code = is_test_code(rel) || gated.iter().any(|gate| file.starts_with(gate));
        if !test_code || rel == Path::new(SANDBOX) {
            continue;
        }
        for line in unexplained(text) {
            outside.push(format!("{}:{line}", rel.display()));
        }
    }

    assert!(
        outside.is_empty(),
        "a test starts a process outside the sandbox: run it with kubaos_core::sandbox (its \
         `command`, `git` or `output`), or say why it may not above it (`// {EXCEPTION} …`), \
         one comment for each:\n{}",
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

#[test]
fn a_module_compiled_only_for_tests_is_test_code() {
    let file = Path::new("/c/src/engine.rs");
    let text = "#[cfg(test)]\nmod helpers;\n#[cfg(any(test, feature = \"fake\"))]\npub mod stub;\n\
                #[cfg(feature = \"latest\")]\nmod latest;\nmod real;\n";

    let found = test_modules(file, text);

    assert_eq!(
        found,
        [
            PathBuf::from("/c/src/engine/helpers.rs"),
            PathBuf::from("/c/src/engine/helpers"),
            PathBuf::from("/c/src/engine/stub.rs"),
            PathBuf::from("/c/src/engine/stub"),
        ]
    );
}

#[test]
fn a_start_under_another_name_is_found() {
    let text = format!(
        "use std::process::{} as Process;\nfn f() {{ Process{}(\"sh\"); }}\n",
        "Command", "::new"
    );

    assert_eq!(unexplained(&text), [2]);
}

#[test]
fn a_git_command_made_with_kubaos_git_is_found() {
    let text = format!("fn f() {{ kubaos_git{}(dir).output(); }}\n", "::at");

    assert_eq!(unexplained(&text), [1]);
}

#[test]
fn a_start_named_in_a_string_or_a_comment_is_not_code() {
    let start = concat!("Command", "::new(");
    let text = format!(
        "// {start}\n/* {start} */\nlet s = \"{start}\";\nlet r = r#\"{start}\"#;\nlet c = '\"'; let t = \"{start}\";\n"
    );

    assert_eq!(unexplained(&text), Vec::<usize>::new());
}

#[test]
fn one_exception_covers_one_start() {
    let start = concat!("Command", "::new(\"sh\")");
    let text = format!("// {EXCEPTION} it reads only its environment.\nlet a = {start};\nlet b = {start};\n");

    assert_eq!(unexplained(&text), [3]);
}
