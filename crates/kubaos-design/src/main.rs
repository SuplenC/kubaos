//! `kubaos-design`: the design lint and the geometry check of frames from any toolkit.
//!
//! ```text
//! kubaos-design lint [--root DIR]            exit 1 when a file is over or under its baseline
//! kubaos-design lint --all [--root DIR]      every violation, the baselined ones included
//! kubaos-design baseline [--root DIR]        shrink the baseline to what the lint finds
//! kubaos-design baseline --init [--root DIR] write the first baseline, when git never had one
//! kubaos-design geometry FILE… [--root DIR]  exit 1 when a frame breaks a geometry rule
//! ```
//!
//! # Owns
//!
//! - The command line, its output and its exit codes: 2 for a usage error, 1 for a failed check
//!   or a file that can't be read or written.
//!
//! # Not here
//!
//! - Everything else: the library.

#![forbid(unsafe_code)]

use std::env;
use std::fmt::Write as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use kubaos_design::Error;
use kubaos_design::baseline;
use kubaos_design::baseline::Baseline;
use kubaos_design::geometry;
use kubaos_design::lint;

const USAGE: &str = "\
usage: kubaos-design lint [--all] [--root DIR]
       kubaos-design baseline [--init] [--root DIR]
       kubaos-design geometry FILE... [--root DIR]

lint      literal colours, sizes, spacing, radii, durations and fonts, and the toolkits' own
          controls, in QML and Rust UI code; fail when a file has more or fewer than
          design/lint-baseline.toml allows. --all lists every one, the baselined included
baseline  lower design/lint-baseline.toml to what the lint finds; it never raises a count.
          --init writes it from what the lint finds, only when git has never had one
geometry  check frames (JSON, one or an array per file) against the geometry rules

DIR is the repo's root; by default the nearest directory up from here holding design/tokens.toml.";

fn main() -> ExitCode {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let root = match args.iter().position(|a| a == "--root") {
        Some(at) if at + 1 < args.len() => {
            let dir = args.remove(at + 1);
            args.remove(at);
            Some(PathBuf::from(dir))
        }
        Some(_) => return usage(),
        None => find_root(),
    };
    let Some(root) = root else {
        eprintln!(
            "kubaos-design: no {} here or in any parent; pass --root",
            kubaos_tokens::TOKENS_PATH
        );
        return ExitCode::from(2);
    };
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match args.as_slice() {
        ["lint"] => lint(&root, false),
        ["lint", "--all"] => lint(&root, true),
        ["baseline"] => shrink(&root),
        ["baseline", "--init"] => init(&root),
        ["geometry", files @ ..] if !files.is_empty() => geometry(&root, files),
        _ => return usage(),
    };
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(err) => {
            report(&err);
            ExitCode::FAILURE
        }
    }
}

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

fn report(err: &Error) {
    let mut message = format!("kubaos-design: {err}");
    let mut source = std::error::Error::source(err);
    while let Some(cause) = source {
        let _ = write!(message, ": {cause}");
        source = cause.source();
    }
    eprintln!("{message}");
}

/// The nearest directory up from the current one that holds the tokens.
fn find_root() -> Option<PathBuf> {
    let here = env::current_dir().ok()?;
    here.ancestors()
        .find(|dir| dir.join(kubaos_tokens::TOKENS_PATH).is_file())
        .map(Path::to_path_buf)
}

fn lint(root: &Path, all: bool) -> Result<bool, Error> {
    let scan = lint::scan(root)?;
    if all {
        for violation in &scan.violations {
            println!("{violation}");
        }
    }
    let baseline = Baseline::load(root)?;
    let problems = baseline::compare(&scan, &baseline);
    for problem in &problems {
        eprintln!("{problem}");
    }
    for marker in &scan.bad_markers {
        eprintln!("{marker}");
    }
    let passed = problems.is_empty() && scan.bad_markers.is_empty();
    let baselined: usize = baseline.files.values().flat_map(|rules| rules.values()).sum();
    eprintln!(
        "kubaos-design lint: {} file(s), {} violation(s), {baselined} in the baseline: {}",
        scan.files.len(),
        scan.violations.len(),
        if passed { "ok" } else { "FAILED" }
    );
    Ok(passed)
}

fn shrink(root: &Path) -> Result<bool, Error> {
    let scan = lint::scan(root)?;
    let before = Baseline::load(root)?;
    let after = baseline::shrink(&scan, &before)?;
    after.write(root)?;
    let count = |b: &Baseline| b.files.values().flat_map(|rules| rules.values()).sum::<usize>();
    eprintln!(
        "kubaos-design baseline: {} -> {} violation(s) in {}",
        count(&before),
        count(&after),
        baseline::BASELINE_PATH
    );
    Ok(true)
}

fn init(root: &Path) -> Result<bool, Error> {
    if root.join(baseline::BASELINE_PATH).exists() {
        eprintln!(
            "kubaos-design baseline: {} exists, and only shrinks; `kubaos-design baseline` shrinks it",
            baseline::BASELINE_PATH
        );
        return Ok(false);
    }
    if baseline::in_history(root)? {
        eprintln!(
            "kubaos-design baseline: git has had {}, so it only shrinks; restore it with `git checkout`",
            baseline::BASELINE_PATH
        );
        return Ok(false);
    }
    let scan = lint::scan(root)?;
    let first = Baseline::of(&scan);
    first.write(root)?;
    eprintln!(
        "kubaos-design baseline: {} violation(s) in {} file(s) written to {}",
        scan.violations.len(),
        first.files.len(),
        baseline::BASELINE_PATH
    );
    Ok(true)
}

fn geometry(root: &Path, files: &[&str]) -> Result<bool, Error> {
    let rules = geometry::rules(root)?;
    let paths: Vec<&Path> = files.iter().map(Path::new).collect();
    let findings = geometry::check_files(&paths, &rules)?;
    for finding in &findings {
        println!("{finding}");
    }
    eprintln!("kubaos-design geometry: {} finding(s)", findings.len());
    Ok(findings.is_empty())
}
