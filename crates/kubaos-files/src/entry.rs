//! What yazi opens for an argument or a D-Bus call: a local path, from a path as typed (`~`
//! included) or a `file://` URI, percent-escapes and all.
//!
//! # Owns
//!
//! - Parsing `file://` URIs, and which hosts count as this machine.
//! - Expanding `~`, and resolving a relative path against the working folder.
//! - Falling back to the nearest folder that exists, for a path that doesn't.
//!
//! # Not here
//!
//! - What yazi does with a folder or a file: [`crate::window`].

use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::ffi::OsStringExt;
use std::path::Path;
use std::path::PathBuf;

/// Where Linux keeps the machine's host name, which a `file://` URI may name as its host.
const HOSTNAME: &str = "/proc/sys/kernel/hostname";

/// Why an argument names no local path.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum EntryError {
    /// A URI of another scheme, such as `sftp://` or `smb://`.
    #[error("{0} isn't a path or a file:// URI")]
    Scheme(String),
    /// A `file://` URI on another machine.
    #[error("{uri} is on {host}, not on this machine")]
    Remote { uri: String, host: String },
    /// A `file:` URI with no absolute path, or a broken `%` escape.
    #[error("{0} isn't a file:// URI I can read")]
    Malformed(String),
}

/// What a relative argument and a URI's host are resolved against.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolver {
    home: PathBuf,
    cwd: PathBuf,
    /// This machine's host name, which a `file://` URI may name instead of `localhost`.
    host: Option<String>,
}

impl Resolver {
    /// Resolves `~` to `home`, a relative path against `cwd`, and takes `file://` URIs on `host`
    /// as local.
    #[must_use]
    pub fn new(home: impl Into<PathBuf>, cwd: impl Into<PathBuf>, host: Option<String>) -> Self {
        Resolver {
            home: home.into(),
            cwd: cwd.into(),
            host,
        }
    }

    /// The process's `HOME`, working folder and host name. Without a `HOME` it is `/`, and without
    /// a working folder, the home.
    #[must_use]
    pub fn from_env() -> Self {
        let home = std::env::var_os("HOME")
            .filter(|home| !home.is_empty())
            .map_or_else(|| PathBuf::from("/"), PathBuf::from);
        let cwd = std::env::current_dir().unwrap_or_else(|_| home.clone());
        let host = fs::read_to_string(HOSTNAME)
            .ok()
            .map(|name| name.trim().to_owned())
            .filter(|name| !name.is_empty());
        Resolver::new(home, cwd, host)
    }

    /// The home folder.
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// The local path `arg` names: a `file:` URI, `~` or `~/…`, an absolute path, or a path
    /// relative to the working folder.
    ///
    /// # Errors
    ///
    /// When `arg` is a URI of another scheme, on another machine, or malformed.
    pub fn path(&self, arg: &OsStr) -> Result<PathBuf, EntryError> {
        let bytes = arg.as_bytes();
        if starts_with_ignoring_case(bytes, b"file:") {
            let uri = arg
                .to_str()
                .ok_or_else(|| EntryError::Malformed(arg.to_string_lossy().into_owned()))?;
            return self.uri(uri);
        }
        if has_scheme(bytes) {
            return Err(EntryError::Scheme(arg.to_string_lossy().into_owned()));
        }
        if bytes == b"~" {
            return Ok(self.home.clone());
        }
        if let Some(rest) = bytes.strip_prefix(b"~/") {
            return Ok(self.home.join(OsStr::from_bytes(rest)));
        }
        Ok(self.cwd.join(arg))
    }

    /// The local path of the `file:` URI `uri`: `file:///path`, `file://localhost/path`,
    /// `file://<this host>/path` or `file:/path`, unescaped. A query or a fragment is no part of
    /// the path.
    ///
    /// # Errors
    ///
    /// When `uri` isn't a `file:` URI, is on another machine, or is malformed.
    pub fn uri(&self, uri: &str) -> Result<PathBuf, EntryError> {
        let malformed = || EntryError::Malformed(uri.to_owned());
        if !starts_with_ignoring_case(uri.as_bytes(), b"file:") {
            return Err(EntryError::Scheme(uri.to_owned()));
        }
        let rest = &uri["file:".len()..];
        let rest = rest.split(['?', '#']).next().unwrap_or_default();
        let path = match rest.strip_prefix("//") {
            Some(authority) => {
                let (host, path) = authority.find('/').map_or((authority, ""), |at| authority.split_at(at));
                if !self.is_local(host) {
                    return Err(EntryError::Remote {
                        uri: uri.to_owned(),
                        host: host.to_owned(),
                    });
                }
                path
            }
            None => rest,
        };
        if !path.starts_with('/') {
            return Err(malformed());
        }
        let bytes = unescape(path).ok_or_else(malformed)?;
        Ok(PathBuf::from(OsString::from_vec(bytes)))
    }

    fn is_local(&self, host: &str) -> bool {
        host.is_empty()
            || host.eq_ignore_ascii_case("localhost")
            || self.host.as_deref().is_some_and(|own| own.eq_ignore_ascii_case(host))
    }
}

/// `path` if it exists, else its nearest ancestor that does: a file that is gone still opens yazi
/// where it was. `/` when nothing on the way exists.
pub fn existing(path: &Path) -> PathBuf {
    path.ancestors()
        .find(|ancestor| !ancestor.as_os_str().is_empty() && ancestor.exists())
        .map_or_else(|| PathBuf::from("/"), Path::to_path_buf)
}

fn starts_with_ignoring_case(bytes: &[u8], prefix: &[u8]) -> bool {
    bytes.len() >= prefix.len() && bytes[..prefix.len()].eq_ignore_ascii_case(prefix)
}

/// Whether `bytes` starts with a URI scheme and `://`, as `sftp://host/…` does. A path such as
/// `notes:draft.md` has no `//` after its colon, so it stays a path.
fn has_scheme(bytes: &[u8]) -> bool {
    let Some(colon) = bytes.iter().position(|&byte| byte == b':') else {
        return false;
    };
    let (scheme, rest) = bytes.split_at(colon);
    let starts_alphabetic = scheme.first().is_some_and(u8::is_ascii_alphabetic);
    let scheme_chars = scheme
        .iter()
        .all(|&byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-' | b'.'));
    starts_alphabetic && scheme_chars && rest.starts_with(b"://")
}

/// `text` with each `%XX` escape replaced by its byte; `None` on a `%` without two hex digits.
fn unescape(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'%' {
            let &[high, low] = bytes.get(at + 1..at + 3)? else {
                return None;
            };
            out.push(hex_digit(high)? << 4 | hex_digit(low)?);
            at += 3;
        } else {
            out.push(bytes[at]);
            at += 1;
        }
    }
    Some(out)
}

fn hex_digit(byte: u8) -> Option<u8> {
    (byte as char).to_digit(16).map(|digit| digit as u8)
}

#[cfg(test)]
mod tests;
