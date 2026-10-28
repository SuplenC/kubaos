//! `kubaos-tokens`: checks `design/tokens.toml` and writes the toolkits' copies.
//!
//! ```text
//! kubaos-tokens check [--root DIR]        exit 1 when a rule is broken, an output can't be
//!                                         rendered, or a checked-in output is stale
//! kubaos-tokens write [--root DIR]        write every output a checkout keeps that differs
//! kubaos-tokens build OUT [--root DIR]    write the ignored outputs under OUT (the Nix build)
//! ```
//!
//! # Owns
//!
//! - The command line, its output and its exit codes: 2 for a usage error, 1 for broken rules, an
//!   output that can't be rendered, a stale output on `check`, or a file that can't be read or
//!   written.
//!
//! # Not here
//!
//! - Everything else: the library.

use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use kubaos_tokens::Error;
use kubaos_tokens::TOKENS_PATH;
use kubaos_tokens::target;

const USAGE: &str = "\
usage: kubaos-tokens check [--root DIR]
       kubaos-tokens write [--root DIR]
       kubaos-tokens build OUT [--root DIR]

check  fail when design/tokens.toml breaks a rule, an output can't be rendered, or a checked-in
       output is stale
write  write every output a checkout keeps, checked in or ignored, that differs from the tokens
build  write the ignored outputs under OUT, at their paths in the repo

DIR is the repo's root; by default the nearest directory up from here holding design/tokens.toml.";

#[expect(
    clippy::print_stderr,
    reason = "A usage error goes to whoever ran the command, on stderr."
)]
fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let (command, out, root) = match args.as_slice() {
        ["build", out] => ("build", Some(*out), find_root()),
        ["build", out, "--root", dir] => ("build", Some(*out), Some(PathBuf::from(dir))),
        [command] => (*command, None, find_root()),
        [command, "--root", dir] => (*command, None, Some(PathBuf::from(dir))),
        _ => return usage(),
    };
    let Some(root) = root else {
        eprintln!("kubaos-tokens: no {TOKENS_PATH} here or in any parent; pass --root");
        return ExitCode::from(2);
    };
    match (command, out) {
        ("check", None) => check(&root),
        ("write", None) => write(&root),
        ("build", Some(out)) => build(&root, Path::new(out)),
        _ => usage(),
    }
}

#[expect(
    clippy::print_stderr,
    reason = "A usage error goes to whoever ran the command, on stderr."
)]
fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

fn find_root() -> Option<PathBuf> {
    let here = env::current_dir().ok()?;
    here.ancestors()
        .find(|dir| dir.join(TOKENS_PATH).is_file())
        .map(Path::to_path_buf)
}

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The verdict and the stale files are the check's output, for the developer or the CI log."
)]
fn check(root: &Path) -> ExitCode {
    let tokens = match kubaos_tokens::load(root) {
        Ok(tokens) => tokens,
        Err(err) => return failed(&err),
    };
    match target::stale(root, &tokens) {
        Ok(stale) if stale.is_empty() => {
            println!("kubaos-tokens: the tokens hold every rule and every checked-in output is current");
            ExitCode::SUCCESS
        }
        Ok(stale) => {
            for target::Stale { path } in stale {
                eprintln!("kubaos-tokens: {} is stale; run `kubaos-tokens write`", path.display());
            }
            ExitCode::FAILURE
        }
        Err(err) => failed(&err),
    }
}

#[expect(
    clippy::print_stdout,
    reason = "The files written are the command's report, for the developer who ran it."
)]
fn write(root: &Path) -> ExitCode {
    let tokens = match kubaos_tokens::load(root) {
        Ok(tokens) => tokens,
        Err(err) => return failed(&err),
    };
    match target::write(root, &tokens) {
        Ok(written) => {
            for path in &written {
                println!("kubaos-tokens: wrote {}", path.display());
            }
            if written.is_empty() {
                println!("kubaos-tokens: every output is current");
            }
            ExitCode::SUCCESS
        }
        Err(err) => failed(&err),
    }
}

#[expect(
    clippy::print_stdout,
    reason = "The files written are the command's report, for the build's log."
)]
fn build(root: &Path, out: &Path) -> ExitCode {
    let tokens = match kubaos_tokens::load(root) {
        Ok(tokens) => tokens,
        Err(err) => return failed(&err),
    };
    match target::build(root, &tokens, out) {
        Ok(written) => {
            for path in &written {
                println!("kubaos-tokens: built {}", path.display());
            }
            ExitCode::SUCCESS
        }
        Err(err) => failed(&err),
    }
}

/// Prints `err`, its causes, and every broken rule, then fails.
#[expect(
    clippy::print_stderr,
    reason = "The error and every broken rule are the command's answer to the developer who ran it."
)]
fn failed(err: &(dyn std::error::Error + 'static)) -> ExitCode {
    eprint!("kubaos-tokens: {err}");
    let mut cause = err.source();
    while let Some(next) = cause {
        eprint!(": {next}");
        cause = next.source();
    }
    eprintln!();
    if let Some(Error::Invalid(violations)) = err.downcast_ref::<Error>() {
        for violation in violations {
            eprintln!("  {violation}");
        }
    }
    ExitCode::FAILURE
}
