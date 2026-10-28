//! The command's output: the lines `kubaos` prints for the person who ran it, through [`out!`]
//! to stdout and [`err!`] to stderr, with `println!`'s arguments.
//!
//! # Owns
//!
//! - Writing a line of the command's output or of its errors and progress.
//!
//! # Not here
//!
//! - The `--events` stream: the commands' modules, through `kubaos-core`'s events.
//! - What a line says: each command's module.

use std::fmt;

/// Prints a line of the command's output to stdout, as `println!` does.
macro_rules! out {
    ($($arg:tt)*) => {
        $crate::out::stdout(format_args!($($arg)*))
    };
}

/// Prints a line of the command's errors or progress to stderr, as `eprintln!` does.
macro_rules! err {
    ($($arg:tt)*) => {
        $crate::out::stderr(format_args!($($arg)*))
    };
}

pub(crate) use err;
pub(crate) use out;

/// Writes `line` and a newline to stdout. [`out!`] calls it.
#[expect(
    clippy::print_stdout,
    reason = "The CLI's answers are its output, for the person or the script that ran it."
)]
pub fn stdout(line: fmt::Arguments<'_>) {
    println!("{line}");
}

/// Writes `line` and a newline to stderr. [`err!`] calls it.
#[expect(
    clippy::print_stderr,
    reason = "The CLI's errors and progress are its output, for the person who ran it."
)]
pub fn stderr(line: fmt::Arguments<'_>) {
    eprintln!("{line}");
}
