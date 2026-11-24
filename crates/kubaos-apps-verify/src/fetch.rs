//! Fetching over HTTP with `curl`, which every machine and runner the check runs on has, each fetch
//! tried again while the service looks down.
//!
//! # Owns
//!
//! - Reading a URL whole ([`get`]), hashing one as it streams ([`sha256`]), a URL's final status
//!   ([`status`]) and GitHub's API, with `GITHUB_TOKEN` when it is set ([`github`]).
//! - Telling an outage from a wrong answer ([`Class`]), the tries and time a fetch gets before an
//!   outage is reported ([`Retry`]), and the hosts found out of reach this run.
//!
//! # Not here
//!
//! - What is checked against what, and how a class is reported: the checks' modules.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fmt::Write as _;
use std::io;
use std::io::Read as _;
use std::io::Write as _;
use std::process::Child;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Output;
use std::process::Stdio;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use sha2::Digest as _;
use sha2::Sha256;

/// How long one try of a transfer may take, in seconds: a vendor's `.deb` runs to a few hundred MB.
/// A try after the first gets what is left of [`Retry::budget`] instead.
const MAX_TIME: u64 = if cfg!(test) { 1 } else { 900 };

/// How long a web app's answer may take, in seconds; only its status is read.
const STATUS_MAX_TIME: u64 = if cfg!(test) { 1 } else { 60 };

/// How long reaching a server may take, in seconds, so a host that drops packets fails in half a
/// minute instead of curl's five.
const CONNECT_TIMEOUT: &str = "30";

/// A transfer slower than [`SPEED_LIMIT`] bytes a second for this many seconds is a timeout, so a
/// host that accepts and then stalls costs a minute, not [`MAX_TIME`].
const SPEED_TIME: &str = if cfg!(test) { "1" } else { "60" };
const SPEED_LIMIT: &str = "1024";

/// How a fetch that looks like an outage is tried again.
#[derive(Clone, Copy, Debug)]
struct Retry {
    /// The most tries, the first included.
    tries: u32,
    /// The first wait between tries, doubled after each.
    backoff: Duration,
    /// No try starts once this long has passed since the first began.
    budget: Duration,
}

impl Retry {
    /// 7 tries, waiting 1, 2, 4, 8, 16 and 32 seconds between, within 3 minutes: a mirror such as
    /// Launchpad's answers 503 for a minute or so at a time, and a check that gives up sooner
    /// reports an outage that has passed, while the CI job's 30 minutes hold many hosts' worst
    /// cases.
    const RUNS: Self = Self {
        tries: 7,
        backoff: Duration::from_secs(1),
        budget: Duration::from_mins(3),
    };

    /// The tests' tries wait none, so they stay fast.
    const TESTS: Self = Self {
        tries: 3,
        backoff: Duration::ZERO,
        budget: Duration::from_secs(10),
    };

    /// The wait before try `next` (2 for the first try again), none when no try is due: the tries
    /// or the budget are spent, or a timeout has been tried again once already, since a host that
    /// times out twice costs minutes a try.
    fn wait(&self, next: u32, elapsed: Duration, timeouts: u32) -> Option<Duration> {
        if next > self.tries || timeouts > 1 {
            return None;
        }
        let wait = self.backoff * 2u32.pow(next.saturating_sub(2));
        (elapsed + wait < self.budget).then_some(wait)
    }
}

const RETRY: Retry = if cfg!(test) { Retry::TESTS } else { Retry::RUNS };

/// How many requests a fetch that stays down makes, for the other modules' tests.
#[cfg(test)]
pub fn tries() -> usize {
    RETRY.tries as usize
}

/// How long a run may fetch: no fetch starts once this long has passed since the run's first, and
/// one in flight is cut at it, an outage either way. The `apps verify` job has 30 minutes, of which
/// its checkout, nix and the build take up to about 8 with cold caches; whatever the outages, the
/// run's fetching ends by this, with a margin. A healthy run of the whole catalog takes 2 to 4.
const DEADLINE: Duration = if cfg!(test) {
    Duration::from_mins(10)
} else {
    Duration::from_mins(18)
};

/// When the run's first fetch started, which [`DEADLINE`] counts from.
static RUN_START: OnceLock<Instant> = OnceLock::new();

/// How long the run may still fetch.
fn run_left() -> Duration {
    DEADLINE.saturating_sub(RUN_START.get_or_init(Instant::now).elapsed())
}

/// The seconds a try may take of `max_time` with `left` of the run, none when the run's time is
/// spent; at least a second, as curl takes whole seconds.
fn capped(max_time: u64, left: Duration) -> Option<u64> {
    (!left.is_zero()).then(|| max_time.min(left.as_secs().max(1)))
}

/// The hosts that were out of reach on every try this run: a later fetch from one is an outage at
/// once, so a host that drops packets costs its tries once, not once a URL.
static DOWN: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

/// `url`'s host and port, as the scheme leaves them.
fn host(url: &str) -> &str {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    rest.split(['/', '?', '#']).next().unwrap_or(rest)
}

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
    Curl {
        exit: Option<i32>,
        said: String,
        /// Where the transfer got to, as `--write-out` said.
        written: Written,
    },
    #[error("the answer isn't JSON")]
    Json(#[source] serde_json::Error),
    /// The host was out of reach on every try of an earlier fetch this run.
    #[error("{0} was out of reach earlier in this run; not tried again")]
    HostDown(String),
    /// The run's time for fetching was spent before this fetch could start.
    #[error("the run's time for fetching was spent; not tried")]
    Late,
}

impl Cause {
    fn class(&self) -> Class {
        match self {
            Self::Status(status) if *status == 429 || (500..600).contains(status) => Class::Outage,
            Self::Curl { exit: Some(exit), .. } if curl_outage(*exit) => Class::Outage,
            Self::HostDown(_) | Self::Late => Class::Outage,
            _ => Class::Failure,
        }
    }

    /// The host that was out of reach, when the failure was clearly at connect time: no name, no
    /// connection, no TLS, or a timeout before any connection. A transfer that timed out or
    /// slowed after connecting may be one big file on a slow path, and says nothing of the host.
    /// It is the host the transfer ended at, after its redirects, else `url`'s.
    fn down_host<'a>(&'a self, url: &'a str) -> Option<&'a str> {
        let Self::Curl {
            exit: Some(exit),
            written,
            ..
        } = self
        else {
            return None;
        };
        let at_connect = matches!(exit, 5 | 6 | 7 | 35) || (*exit == 28 && !written.connected);
        let effective = if written.effective.is_empty() {
            url
        } else {
            &written.effective
        };
        at_connect.then(|| host(effective))
    }

    fn timed_out(&self) -> bool {
        matches!(self, Self::Curl { exit: Some(28), .. })
    }

    /// The cause of a curl run that failed: the HTTP error `--fail` stopped on, else curl's exit.
    /// `--write-out` put its [`WRITTEN`] line last on stderr, after curl's own message.
    fn of(status: ExitStatus, stderr: &[u8]) -> Self {
        let text = String::from_utf8_lossy(stderr);
        let text = text.trim_end();
        let (said, written) = text.rsplit_once('\n').unwrap_or(("", text));
        let written = Written::parse(written);
        // curl's exit 22 is `--fail` stopping on an HTTP status of 400 or more.
        if status.code() == Some(22) && written.http >= 400 {
            return Self::Status(written.http);
        }
        Self::Curl {
            exit: status.code(),
            said: said.trim().to_owned(),
            written,
        }
    }
}

/// What curl's `--write-out` writes of a transfer, for [`Written::parse`].
const WRITTEN: &str = "%{http_code} %{time_connect} %{url_effective}";

/// Where a transfer got to: its last HTTP status, whether it connected, and the URL it ended at
/// after its redirects.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Written {
    pub http: u16,
    pub connected: bool,
    pub effective: String,
}

impl Written {
    /// Reads a [`WRITTEN`] line; a field that is missing or doesn't read is its default.
    fn parse(line: &str) -> Self {
        let mut fields = line.split_whitespace();
        let http = fields.next().and_then(|code| code.parse().ok()).unwrap_or(0);
        let connected = fields
            .next()
            .and_then(|time| time.parse::<f64>().ok())
            .is_some_and(|time| time > 0.0);
        let effective = fields.next().unwrap_or_default().to_owned();
        Self {
            http,
            connected,
            effective,
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

fn curl(max_time: u64) -> Command {
    let mut command = Command::new("curl");
    command.args([
        "--silent",
        "--show-error",
        "--location",
        "--connect-timeout",
        CONNECT_TIMEOUT,
    ]);
    command.args(["--speed-limit", SPEED_LIMIT, "--speed-time", SPEED_TIME]);
    command.arg("--max-time").arg(max_time.to_string());
    // The tests' stub server is on loopback, which a proxy of the machine's must not stand in for.
    if cfg!(test) {
        command.args(["--noproxy", "*"]);
    }
    command
}

/// curl stopping on an HTTP error and writing the final status last on stderr, for [`Cause::of`].
fn failing_curl(max_time: u64) -> Command {
    let mut command = curl(max_time);
    command
        .args(["--fail", "--write-out"])
        .arg(format!("%{{stderr}}{WRITTEN}\n"));
    command
}

/// `once` tried, given how many seconds the try may take of `max_time`, until it succeeds, fails
/// with what isn't an outage, or [`RETRY`] allows no further try. A host out of reach at connect
/// time on the last try is remembered in [`DOWN`], and a later fetch from it isn't tried.
fn tried<T>(url: &str, max_time: u64, mut once: impl FnMut(u64) -> Result<T, Cause>) -> Result<T, FetchError> {
    let host = host(url);
    if DOWN.lock().expect("down hosts mutex poisoned").contains(host) {
        return Err(FetchError::new(url, Cause::HostDown(host.to_owned())));
    }
    let Some(mut this_try) = capped(max_time, run_left()) else {
        return Err(FetchError::new(url, Cause::Late));
    };
    let start = Instant::now();
    let mut timeouts = 0;
    let mut next = 2;
    loop {
        let cause = match once(this_try) {
            Err(cause) if cause.class() == Class::Outage => cause,
            done => return done.map_err(|cause| FetchError::new(url, cause)),
        };
        timeouts += u32::from(cause.timed_out());
        let elapsed = start.elapsed();
        let Some(wait) = RETRY.wait(next, elapsed, timeouts) else {
            if let Some(down) = cause.down_host(url) {
                DOWN.lock().expect("down hosts mutex poisoned").insert(down.to_owned());
            }
            return Err(FetchError::new(url, cause));
        };
        let Some(in_run) = capped(max_time, run_left().saturating_sub(wait)) else {
            return Err(FetchError::new(url, cause));
        };
        thread::sleep(wait);
        let left = RETRY.budget.saturating_sub(elapsed + wait).as_secs().max(1);
        this_try = in_run.min(left);
        next += 1;
    }
}

/// `child`'s output once it has ended; killed first when `failed`, so no curl is left running.
fn reaped(mut child: Child, failed: Option<io::Error>) -> Result<Output, Cause> {
    if let Some(e) = failed {
        let _ = child.kill();
        let _ = child.wait();
        return Err(Cause::Spawn(e));
    }
    child.wait_with_output().map_err(Cause::Spawn)
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
    tried(url, MAX_TIME, |max_time| {
        let output = failing_curl(max_time)
            .arg("--")
            .arg(url)
            .output()
            .map_err(Cause::Spawn)?;
        succeeded(output)
    })
}

/// The sha256 of what `url` serves, as lowercase hex, hashed as it arrives.
///
/// # Errors
///
/// [`FetchError`] when curl can't run or the server answers with an error on the last try.
pub fn sha256(url: &str) -> Result<String, FetchError> {
    tried(url, MAX_TIME, |max_time| {
        let mut child = failing_curl(max_time)
            .arg("--")
            .arg(url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(Cause::Spawn)?;
        let mut hasher = Sha256::new();
        let mut stdout = child.stdout.take().expect("curl's stdout is piped");
        let mut buffer = vec![0; 1 << 16];
        let failed = loop {
            match stdout.read(&mut buffer) {
                Ok(0) => break None,
                Ok(read) => hasher.update(&buffer[..read]),
                Err(e) => break Some(e),
            }
        };
        drop(stdout);
        succeeded(reaped(child, failed)?)?;
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
    tried(url, STATUS_MAX_TIME, |max_time| {
        let output = curl(max_time)
            .args(["--output", "/dev/null"])
            .arg("--write-out")
            .arg(format!("{WRITTEN} %header{{cf-mitigated}}"))
            .arg("--")
            .arg(url)
            .output()
            .map_err(Cause::Spawn)?;
        let text = String::from_utf8_lossy(&output.stdout);
        // The challenge header comes last, after the three fields of [`WRITTEN`], none with a space.
        let mut fields = text.trim().splitn(4, ' ');
        let written = Written::parse(&fields.by_ref().take(3).collect::<Vec<_>>().join(" "));
        let mitigated = fields.next().unwrap_or_default();
        if !output.status.success() {
            return Err(Cause::Curl {
                exit: output.status.code(),
                said: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
                written,
            });
        }
        let answer = Answer {
            status: written.http,
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
    // The bump asks for one repository's releases once for each architecture of a pin.
    static ANSWERS: Mutex<BTreeMap<String, serde_json::Value>> = Mutex::new(BTreeMap::new());
    if let Some(known) = ANSWERS.lock().expect("GitHub answers mutex poisoned").get(path) {
        return Ok(known.clone());
    }
    let answer = github_uncached(path)?;
    ANSWERS
        .lock()
        .expect("GitHub answers mutex poisoned")
        .insert(path.to_owned(), answer.clone());
    Ok(answer)
}

fn github_uncached(path: &str) -> Result<serde_json::Value, FetchError> {
    let url = format!("https://api.github.com/{path}");
    let token = env::var("GITHUB_TOKEN").ok().filter(|token| !token.is_empty());
    tried(&url, MAX_TIME, |max_time| {
        let mut command = failing_curl(max_time);
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
        let failed = token
            .as_ref()
            .and_then(|token| writeln!(stdin, "Authorization: Bearer {token}").err());
        drop(stdin);
        let body = succeeded(reaped(child, failed)?)?;
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
