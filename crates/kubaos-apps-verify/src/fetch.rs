//! Fetching over HTTP with `curl`, which every machine and runner the check runs on has.
//!
//! # Owns
//!
//! - Reading a URL whole ([`get`]), hashing one as it streams ([`sha256`]), a URL's final status
//!   ([`status`]) and GitHub's API, with `GITHUB_TOKEN` when it is set ([`github`]).
//!
//! # Not here
//!
//! - What is checked against what: the checks' modules.

use std::env;
use std::fmt::Write as _;
use std::io;
use std::io::Read as _;
use std::io::Write as _;
use std::process::Command;
use std::process::Stdio;

use sha2::Digest as _;
use sha2::Sha256;

/// How long one transfer may take, in seconds: a vendor's `.deb` runs to a few hundred MB.
const MAX_TIME: &str = "900";

/// A URL that couldn't be fetched.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FetchError {
    #[error("running curl")]
    Spawn(#[source] io::Error),
    #[error("curl: {0}")]
    Failed(String),
}

impl FetchError {
    fn failed(stderr: &[u8]) -> Self {
        Self::Failed(String::from_utf8_lossy(stderr).trim().to_owned())
    }
}

fn curl() -> Command {
    let mut command = Command::new("curl");
    command.args([
        "--silent",
        "--show-error",
        "--fail",
        "--location",
        "--retry",
        "3",
        "--max-time",
        MAX_TIME,
    ]);
    command
}

/// The body `url` serves.
///
/// # Errors
///
/// [`FetchError`] when curl can't run or the server answers with an error.
pub fn get(url: &str) -> Result<Vec<u8>, FetchError> {
    let output = curl().arg("--").arg(url).output().map_err(FetchError::Spawn)?;
    if !output.status.success() {
        return Err(FetchError::failed(&output.stderr));
    }
    Ok(output.stdout)
}

/// The sha256 of what `url` serves, as lowercase hex, hashed as it arrives.
///
/// # Errors
///
/// [`FetchError`] when curl can't run or the server answers with an error.
pub fn sha256(url: &str) -> Result<String, FetchError> {
    let mut child = curl()
        .arg("--")
        .arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(FetchError::Spawn)?;
    let mut hasher = Sha256::new();
    let mut stdout = child.stdout.take().expect("curl's stdout is piped");
    let mut buffer = vec![0; 1 << 16];
    loop {
        let read = stdout.read(&mut buffer).map_err(FetchError::Spawn)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let output = child.wait_with_output().map_err(FetchError::Spawn)?;
    if !output.status.success() {
        return Err(FetchError::failed(&output.stderr));
    }
    Ok(hex(&hasher.finalize()))
}

/// Where a URL ends, after its redirects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Answer {
    pub status: u16,
    /// Cloudflare answered with its bot challenge (`cf-mitigated: challenge`), which a browser
    /// passes and curl can't.
    pub challenged: bool,
}

/// The HTTP status `url` ends on, after its redirects, and whether it is Cloudflare's challenge.
///
/// # Errors
///
/// [`FetchError`] when curl can't run or can't reach the server.
pub fn status(url: &str) -> Result<Answer, FetchError> {
    let output = Command::new("curl")
        .args([
            "--silent",
            "--show-error",
            "--location",
            "--retry",
            "3",
            "--max-time",
            "60",
        ])
        .args([
            "--output",
            "/dev/null",
            "--write-out",
            "%{http_code} %header{cf-mitigated}",
            "--",
        ])
        .arg(url)
        .output()
        .map_err(FetchError::Spawn)?;
    if !output.status.success() {
        return Err(FetchError::failed(&output.stderr));
    }
    let written = String::from_utf8_lossy(&output.stdout);
    let (code, mitigated) = written.trim().split_once(' ').unwrap_or((written.trim(), ""));
    let status = code
        .parse()
        .map_err(|_| FetchError::Failed(format!("no status from {url}")))?;
    Ok(Answer {
        status,
        challenged: mitigated.trim().eq_ignore_ascii_case("challenge"),
    })
}

/// GitHub's API at `path` (`repos/OWNER/REPO/…`), as JSON. `GITHUB_TOKEN`, when set, goes to curl
/// on its stdin, never in its arguments: without one GitHub allows 60 calls an hour.
///
/// # Errors
///
/// [`FetchError`] when curl can't run, GitHub answers with an error, or the answer isn't JSON.
pub fn github(path: &str) -> Result<serde_json::Value, FetchError> {
    let mut command = curl();
    command.args(["--header", "Accept: application/vnd.github+json"]);
    let token = env::var("GITHUB_TOKEN").ok().filter(|token| !token.is_empty());
    if token.is_some() {
        command.args(["--header", "@-"]);
    }
    let mut child = command
        .arg("--")
        .arg(format!("https://api.github.com/{path}"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(FetchError::Spawn)?;
    let mut stdin = child.stdin.take().expect("curl's stdin is piped");
    if let Some(token) = token {
        writeln!(stdin, "Authorization: Bearer {token}").map_err(FetchError::Spawn)?;
    }
    drop(stdin);
    let output = child.wait_with_output().map_err(FetchError::Spawn)?;
    if !output.status.success() {
        return Err(FetchError::failed(&output.stderr));
    }
    serde_json::from_slice(&output.stdout).map_err(|e| FetchError::Failed(format!("GitHub's answer: {e}")))
}

/// `bytes` as lowercase hex.
pub fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        })
}
