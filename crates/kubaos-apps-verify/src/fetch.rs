//! Fetching over HTTP with `curl`, which every machine and runner the check runs on has, each fetch
//! tried again while the service looks down.
//!
//! # Owns
//!
//! - Reading a URL whole ([`get`]), hashing one as it streams ([`sha256`]), a URL's final status
//!   ([`status`]) and GitHub's API, with `GITHUB_TOKEN` when it is set ([`github`]).
//! - Telling an outage from a wrong answer ([`Class`]), and the tries before an outage is reported.
//!
//! # Not here
//!
//! - What is checked against what, and how a class is reported: the checks' modules.

use std::env;
use std::fmt::Write as _;
use std::io;
use std::io::Read as _;
use std::io::Write as _;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Output;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

use sha2::Digest as _;
use sha2::Sha256;

/// How long one transfer may take, in seconds: a vendor's `.deb` runs to a few hundred MB. The
/// tests stall a stub server for longer than theirs.
const MAX_TIME: &str = if cfg!(test) { "1" } else { "900" };

/// How long a web app's answer may take, in seconds; only its status is read.
const STATUS_MAX_TIME: &str = if cfg!(test) { "1" } else { "60" };

/// How long reaching a server may take, in seconds, so a host that is gone fails fast instead of
/// waiting out curl's five minutes.
const CONNECT_TIMEOUT: &str = "30";

/// How many times a fetch is tried while it looks like an outage, waiting [`BACKOFF`], then twice
/// as long each time: 1, 2, 4, 8, 16 and 32 seconds. A mirror such as Launchpad's answers 503 for
/// a minute or so at a time, and a check that gives up sooner reports an outage that has passed.
const TRIES: u32 = if cfg!(test) { 3 } else { 7 };

/// The first wait between tries, doubled after each. The tests wait none, so they stay fast.
const BACKOFF: Duration = if cfg!(test) {
    Duration::ZERO
} else {
    Duration::from_secs(1)
};

/// Whether a fetch that failed says anything about the catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// The service was down or out of reach on every try: a warning, since the catalog may be
    /// right.
    Outage,
    /// The service answered, and the answer is wrong for the pin, or the fetch couldn't run.
    Failure,
}

/// A URL that couldn't be fetched.
#[derive(Debug, thiserror::Error)]
#[error("fetching {url}")]
pub struct FetchError {
    url: String,
    #[source]
    cause: Cause,
}

impl FetchError {
    fn new(url: &str, cause: Cause) -> Self {
        Self {
            url: url.to_owned(),
            cause,
        }
    }

    pub fn class(&self) -> Class {
        self.cause.class()
    }
}

/// Why one try of a fetch failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Cause {
    #[error("running curl")]
    Spawn(#[source] io::Error),
    /// The server answered with an HTTP error.
    #[error("HTTP {0}")]
    Status(u16),
    /// curl gave up for another reason, by its exit code; none when a signal stopped it.
    #[error("curl: {said}")]
    Curl { exit: Option<i32>, said: String },
    #[error("the answer isn't JSON")]
    Json(#[source] serde_json::Error),
}

impl Cause {
    fn class(&self) -> Class {
        match self {
            Self::Status(status) if *status == 429 || (500..600).contains(status) => Class::Outage,
            Self::Curl { exit: Some(exit), .. } if curl_outage(*exit) => Class::Outage,
            _ => Class::Failure,
        }
    }

    /// The cause of a curl run that failed: the HTTP error `--fail` stopped on, else curl's exit.
    /// `written` is what `--write-out` put last on stderr, the final HTTP status, after curl's own
    /// message.
    fn of(status: ExitStatus, stderr: &[u8]) -> Self {
        let text = String::from_utf8_lossy(stderr);
        let text = text.trim_end();
        let (said, written) = text.rsplit_once('\n').unwrap_or(("", text));
        let http: u16 = written.trim().parse().unwrap_or(0);
        // curl's exit 22 is `--fail` stopping on an HTTP status of 400 or more.
        if status.code() == Some(22) && http >= 400 {
            return Self::Status(http);
        }
        Self::Curl {
            exit: status.code(),
            said: said.trim().to_owned(),
        }
    }
}

/// Whether curl's `exit` code means the service was down or out of reach, as curl's manual lists
/// its exit codes: a name that doesn't resolve (5, 6), a refused connection (7), an HTTP/2 error
/// (16, 92), a transfer cut short (18), a timeout (28), a failed TLS handshake (35), too many
/// redirects, as a loop gives (47), no answer (52), or a connection broken off (55, 56).
///
/// <https://curl.se/docs/manpage.html#EXIT>
pub fn curl_outage(exit: i32) -> bool {
    matches!(exit, 5 | 6 | 7 | 16 | 18 | 28 | 35 | 47 | 52 | 55 | 56 | 92)
}

fn curl() -> Command {
    let mut command = Command::new("curl");
    command.args([
        "--silent",
        "--show-error",
        "--location",
        "--connect-timeout",
        CONNECT_TIMEOUT,
    ]);
    // The tests' stub server is on loopback, which a proxy of the machine's must not stand in for.
    if cfg!(test) {
        command.args(["--noproxy", "*"]);
    }
    command
}

/// curl stopping on an HTTP error and writing the final status last on stderr, for [`Cause::of`].
fn failing_curl() -> Command {
    let mut command = curl();
    command.args([
        "--fail",
        "--max-time",
        MAX_TIME,
        "--write-out",
        "%{stderr}%{http_code}\n",
    ]);
    command
}

/// `once` tried until it succeeds, fails with what isn't an outage, or has been tried [`TRIES`]
/// times, waiting longer before each try.
fn tried<T>(url: &str, mut once: impl FnMut() -> Result<T, Cause>) -> Result<T, FetchError> {
    let mut wait = BACKOFF;
    for _ in 1..TRIES {
        match once() {
            Err(cause) if cause.class() == Class::Outage => {
                thread::sleep(wait);
                wait *= 2;
            }
            done => return done.map_err(|cause| FetchError::new(url, cause)),
        }
    }
    once().map_err(|cause| FetchError::new(url, cause))
}

fn succeeded(output: Output) -> Result<Vec<u8>, Cause> {
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(Cause::of(output.status, &output.stderr))
    }
}

/// The body `url` serves.
///
/// # Errors
///
/// [`FetchError`] when curl can't run or the server answers with an error on the last try.
pub fn get(url: &str) -> Result<Vec<u8>, FetchError> {
    tried(url, || {
        let output = failing_curl().arg("--").arg(url).output().map_err(Cause::Spawn)?;
        succeeded(output)
    })
}

/// The sha256 of what `url` serves, as lowercase hex, hashed as it arrives.
///
/// # Errors
///
/// [`FetchError`] when curl can't run or the server answers with an error on the last try.
pub fn sha256(url: &str) -> Result<String, FetchError> {
    tried(url, || {
        let mut child = failing_curl()
            .arg("--")
            .arg(url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(Cause::Spawn)?;
        let mut hasher = Sha256::new();
        let mut stdout = child.stdout.take().expect("curl's stdout is piped");
        let mut buffer = vec![0; 1 << 16];
        loop {
            let read = stdout.read(&mut buffer).map_err(Cause::Spawn)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        let output = child.wait_with_output().map_err(Cause::Spawn)?;
        succeeded(output)?;
        Ok(hex(&hasher.finalize()))
    })
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
/// An outage's status, a 5xx or 429 that isn't the challenge, is tried again and then an error.
///
/// # Errors
///
/// [`FetchError`] when curl can't run, can't reach the server, or the server is down on the last
/// try.
pub fn status(url: &str) -> Result<Answer, FetchError> {
    tried(url, || {
        let output = curl()
            .args(["--max-time", STATUS_MAX_TIME, "--output", "/dev/null"])
            .args(["--write-out", "%{http_code} %header{cf-mitigated}", "--"])
            .arg(url)
            .output()
            .map_err(Cause::Spawn)?;
        if !output.status.success() {
            return Err(Cause::Curl {
                exit: output.status.code(),
                said: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        let written = String::from_utf8_lossy(&output.stdout);
        let (code, mitigated) = written.trim().split_once(' ').unwrap_or((written.trim(), ""));
        let answer = Answer {
            status: code.parse().unwrap_or(0),
            challenged: mitigated.trim().eq_ignore_ascii_case("challenge"),
        };
        let cause = Cause::Status(answer.status);
        if !answer.challenged && cause.class() == Class::Outage {
            return Err(cause);
        }
        Ok(answer)
    })
}

/// GitHub's API at `path` (`repos/OWNER/REPO/…`), as JSON. `GITHUB_TOKEN`, when set, goes to curl
/// on its stdin, never in its arguments: without one GitHub allows 60 calls an hour.
///
/// # Errors
///
/// [`FetchError`] when curl can't run, GitHub answers with an error, or the answer isn't JSON.
pub fn github(path: &str) -> Result<serde_json::Value, FetchError> {
    let url = format!("https://api.github.com/{path}");
    let token = env::var("GITHUB_TOKEN").ok().filter(|token| !token.is_empty());
    tried(&url, || {
        let mut command = failing_curl();
        command.args(["--header", "Accept: application/vnd.github+json"]);
        if token.is_some() {
            command.args(["--header", "@-"]);
        }
        let mut child = command
            .arg("--")
            .arg(&url)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(Cause::Spawn)?;
        let mut stdin = child.stdin.take().expect("curl's stdin is piped");
        if let Some(token) = &token {
            writeln!(stdin, "Authorization: Bearer {token}").map_err(Cause::Spawn)?;
        }
        drop(stdin);
        let body = succeeded(child.wait_with_output().map_err(Cause::Spawn)?)?;
        serde_json::from_slice(&body).map_err(Cause::Json)
    })
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

#[cfg(test)]
mod tests;
