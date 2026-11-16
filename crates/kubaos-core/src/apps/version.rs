//! Package versions as dpkg reads and orders them, for a requirement's `min`.
//!
//! # Owns
//!
//! - What a version is ([`is_version`]) and dpkg's order of two ([`compare`]).
//!
//! # Not here
//!
//! - Which version an app's source offers: `engine.rs`. KubaOS's own release versions:
//!   `update::version`.

use std::cmp::Ordering;

/// Whether `version` is one dpkg takes: `[epoch:]upstream[-revision]`, starting with a digit, of
/// ASCII letters, digits and `.+~-:`.
pub fn is_version(version: &str) -> bool {
    let upstream = version.split_once(':').map_or(version, |(epoch, rest)| {
        if epoch.is_empty() || !epoch.chars().all(|c| c.is_ascii_digit()) {
            ""
        } else {
            rest
        }
    });
    version.len() <= 128
        && upstream.chars().next().is_some_and(|c| c.is_ascii_digit())
        && upstream
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '~' | '-' | ':'))
}

/// `a` against `b`, as `dpkg --compare-versions` orders them: the epoch, then the upstream
/// version, then the revision, each compared in runs of non-digits (a `~` before anything, even
/// the end) and of digits as numbers.
/// <https://www.debian.org/doc/debian-policy/ch-controlfields.html#version>
pub fn compare(a: &str, b: &str) -> Ordering {
    let (ae, au, ar) = parts(a);
    let (be, bu, br) = parts(b);
    ae.cmp(&be).then_with(|| part(au, bu)).then_with(|| part(ar, br))
}

/// The epoch, the upstream version and the revision of `version`.
fn parts(version: &str) -> (u64, &str, &str) {
    let (epoch, rest) = match version.split_once(':') {
        Some((epoch, rest)) => (epoch.parse().unwrap_or(0), rest),
        None => (0, version),
    };
    match rest.rsplit_once('-') {
        Some((upstream, revision)) => (epoch, upstream, revision),
        None => (epoch, rest, ""),
    }
}

/// One part of two versions, dpkg's `verrevcmp`.
fn part(a: &str, b: &str) -> Ordering {
    let (mut a, mut b) = (a.as_bytes(), b.as_bytes());
    while !a.is_empty() || !b.is_empty() {
        let (a_text, a_rest) = split(a, |c| !c.is_ascii_digit());
        let (b_text, b_rest) = split(b, |c| !c.is_ascii_digit());
        let longest = a_text.len().max(b_text.len());
        for i in 0..=longest {
            let order = weight(a_text.get(i).copied()).cmp(&weight(b_text.get(i).copied()));
            if order != Ordering::Equal {
                return order;
            }
        }
        let (a_digits, a_next) = split(a_rest, |c| c.is_ascii_digit());
        let (b_digits, b_next) = split(b_rest, |c| c.is_ascii_digit());
        let order = number(a_digits).cmp(&number(b_digits));
        if order != Ordering::Equal {
            return order;
        }
        (a, b) = (a_next, b_next);
    }
    Ordering::Equal
}

/// The longest start of `text` whose bytes all are `run`, and the rest.
fn split(text: &[u8], run: impl Fn(u8) -> bool) -> (&[u8], &[u8]) {
    let end = text.iter().position(|&c| !run(c)).unwrap_or(text.len());
    text.split_at(end)
}

/// A run of digits without its leading zeros, which then orders as a number does: by length,
/// then digit by digit.
fn number(digits: &[u8]) -> (usize, &[u8]) {
    let start = digits.iter().position(|&c| c != b'0').unwrap_or(digits.len());
    let digits = &digits[start..];
    (digits.len(), digits)
}

/// Where a character of a non-digit run sorts: `~` before the end, the end before a letter, a
/// letter before anything else.
fn weight(c: Option<u8>) -> i32 {
    match c {
        Some(b'~') => -1,
        None => 0,
        Some(c) if c.is_ascii_alphabetic() => i32::from(c),
        Some(c) => i32::from(c) + 256,
    }
}

#[cfg(test)]
mod tests;
