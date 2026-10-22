//! KubaOS release versions, `X.Y.Z` with an optional `-pre` part, as `VERSION` and the `v*` tags
//! write them.
//!
//! # Owns
//!
//! - Parsing a version or a tag name, ordering versions, and what a series (`X.Y`) is.
//!
//! # Not here
//!
//! - Which version an update goes to: `policy.rs`.

use std::cmp::Ordering;
use std::fmt;

/// One KubaOS version. Versions order by `X.Y.Z`, and a pre-release comes before its release.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    /// What follows the `-` of a pre-release, `rc.1` in `0.2.0-rc.1`.
    pub pre: Option<String>,
}

/// Text that is not a version.
#[derive(Debug, thiserror::Error)]
#[error("{0:?} is not a KubaOS version (X.Y.Z)")]
pub struct InvalidVersion(String);

impl Version {
    /// Parses `0.1.2`, `0.2.0-rc.1`, or a tag name with its `v`, `v0.1.2`. Surrounding
    /// whitespace, as a `VERSION` file's newline, is ignored.
    ///
    /// # Errors
    ///
    /// [`InvalidVersion`] unless the text is three numbers joined by dots, optionally followed by
    /// `-` and a non-empty pre-release part.
    pub fn parse(text: &str) -> Result<Self, InvalidVersion> {
        let invalid = || InvalidVersion(text.to_owned());
        let trimmed = text.trim();
        let bare = trimmed.strip_prefix('v').unwrap_or(trimmed);
        let (numbers, pre) = match bare.split_once('-') {
            Some((numbers, pre)) if !pre.is_empty() => (numbers, Some(pre.to_owned())),
            Some(_) => return Err(invalid()),
            None => (bare, None),
        };
        let mut parts = numbers.split('.');
        let mut number = || {
            parts
                .next()
                .filter(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
                .and_then(|part| part.parse::<u64>().ok())
                .ok_or_else(invalid)
        };
        let (major, minor, patch) = (number()?, number()?, number()?);
        if parts.next().is_some() {
            return Err(invalid());
        }
        Ok(Self {
            major,
            minor,
            patch,
            pre,
        })
    }

    /// The release tag's name, `v0.1.2`.
    pub fn tag(&self) -> String {
        format!("v{self}")
    }

    /// Whether it is a pre-release, which the stable channel skips.
    pub fn is_pre(&self) -> bool {
        self.pre.is_some()
    }

    /// Whether `other` is in the same series, `X.Y`: a patch of it, or it.
    pub fn same_series(&self, other: &Self) -> bool {
        (self.major, self.minor) == (other.major, other.minor)
    }

    /// The series, `0.1`.
    pub fn series(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.pre {
            write!(f, "-{pre}")?;
        }
        Ok(())
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch)
            .cmp(&(other.major, other.minor, other.patch))
            .then_with(|| match (&self.pre, &other.pre) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(a), Some(b)) => a.cmp(b),
            })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests;
