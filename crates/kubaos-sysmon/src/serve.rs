//! The shell's stream: `kubaos-sysmon serve` writes a sample a line on stdout at the pace the shell
//! asks for, an alert line when one fires, and answers requests read a line at a time on stdin. It
//! lives as long as the shell, so a sample costs reading files, not starting a process.
//!
//! ```text
//! out: {"type":"sample","t":...,"cpu":{...},...}          every 2 s (pill), 10 s (quiet), 1 s (popup),
//!                                                         none (paused)
//!      {"type":"alert","id":"memory","title":...}         when an alert fires
//! in:  {"id":1,"op":"mode","mode":"popup"}                {"id":1,"mode":"popup"}
//!      {"id":2,"op":"process","pid":4242}                 {"id":2,"process":{...}}
//!      {"id":3,"op":"signal","pid":4242,"signal":"term"}  {"id":3,"ok":true}
//!      {"id":4,"op":"settings"}                           {"id":4,"settings":{...}}
//!      {"id":5,"op":"set","key":"pill","value":"off"}     {"id":5,"settings":{...}}
//! ```
//!
//! A request that fails gets `{"id":n,"error":"..."}`. Each sample is also written to the stream
//! file (`$XDG_RUNTIME_DIR/kubaos-sysmon.json`), where the Command menu reads it, so the pill, the
//! panel and the menu show the same numbers.
//!
//! # Owns
//!
//! - The modes and their pace, the requests and replies, reading the settings again when they
//!   change, and the stream file.
//!
//! # Not here
//!
//! - What a sample holds: [`crate::sampler`]. When alerts fire: [`crate::alerts`].

use std::env;
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use serde_json::Value;
use serde_json::json;

use crate::alerts::Alerts;
use crate::processes;
use crate::processes::Signal;
use crate::sampler::Sample;
use crate::sampler::Sampler;
use crate::settings;
use crate::settings::Settings;

/// The pill's pace: fresh enough for a number in the bar, slow enough to cost nothing.
pub const PILL_INTERVAL: Duration = Duration::from_secs(2);
/// The pill's pace in a power saver stop, whose background work waits: a number in the bar may lag
/// a few seconds, and five times fewer samples wake the CPU five times less.
pub const QUIET_INTERVAL: Duration = Duration::from_secs(10);
/// The panel's pace, for its graphs.
pub const POPUP_INTERVAL: Duration = Duration::from_secs(1);
/// How soon after a restart of the counters the first sample comes: enough ticks for a usage.
pub const AFTER_RESTART: Duration = Duration::from_millis(500);
/// The stream file, in `$XDG_RUNTIME_DIR`.
pub const STREAM_FILE: &str = "kubaos-sysmon.json";

/// How often the shell wants samples.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// Light samples for the bar's pill and the alerts.
    #[default]
    Pill,
    /// The pill's samples at a slower pace, while a power saver stop holds background work back.
    Quiet,
    /// Detailed samples with processes, for the open panel.
    Popup,
    /// None: the screen is locked, or nothing shows them.
    Paused,
}

impl Mode {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "pill" => Some(Mode::Pill),
            "quiet" => Some(Mode::Quiet),
            "popup" => Some(Mode::Popup),
            "paused" => Some(Mode::Paused),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Mode::Pill => "pill",
            Mode::Quiet => "quiet",
            Mode::Popup => "popup",
            Mode::Paused => "paused",
        }
    }

    /// The pace of samples; `None` while paused.
    pub fn interval(self) -> Option<Duration> {
        match self {
            Mode::Pill => Some(PILL_INTERVAL),
            Mode::Quiet => Some(QUIET_INTERVAL),
            Mode::Popup => Some(POPUP_INTERVAL),
            Mode::Paused => None,
        }
    }
}

/// When the next sample is due, after a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Next {
    /// As planned.
    Keep,
    /// At once: the panel opened and wants numbers now.
    Now,
    /// After [`AFTER_RESTART`]: the counters were read afresh.
    Soon,
}

/// The sampler, the alerts and the settings, as `serve` drives them.
#[derive(Debug)]
pub struct Server {
    sampler: Sampler,
    alerts: Alerts,
    settings_path: Option<PathBuf>,
    settings: Settings,
    /// What was wrong with the settings when last read.
    problem: Option<String>,
    modified: Option<SystemTime>,
    stream: Option<PathBuf>,
    mode: Mode,
    /// The last sample's alert conditions want a process scan.
    culprits: bool,
}

impl Server {
    /// A server sampling under `root`, with the settings in `dir` and the stream file at `stream`.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, dir: Option<PathBuf>, stream: Option<PathBuf>) -> Self {
        let mut server = Server {
            sampler: Sampler::new(root),
            alerts: Alerts::default(),
            settings_path: dir.map(|dir| dir.join(settings::FILE)),
            settings: Settings::default(),
            problem: None,
            modified: None,
            stream,
            mode: Mode::default(),
            culprits: false,
        };
        server.reload();
        server
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    fn reload(&mut self) {
        self.modified = self.settings_path.as_deref().and_then(modified);
        let loaded = self
            .settings_path
            .as_deref()
            .map_or_else(|| Ok(Settings::default()), Settings::load);
        self.problem = loaded.as_ref().err().map(|err| chain(err));
        self.settings = loaded.unwrap_or_default();
    }

    fn refresh(&mut self) {
        if self.settings_path.as_deref().and_then(modified) != self.modified {
            self.reload();
        }
    }

    /// The lines a due sample makes at `now` (ms since the epoch): the sample, then any alert.
    pub fn tick(&mut self, now: u64) -> Vec<String> {
        self.refresh();
        let detail = self.mode == Mode::Popup;
        let sample = self.sampler.sample(now, detail, detail || self.culprits);
        let mut lines = vec![sample_line(&sample)];
        if let Some(stream) = &self.stream {
            write_stream(stream, &lines[0]);
        }
        if self.settings.alerts {
            self.culprits = Alerts::wants_processes(&sample, &self.settings.thresholds);
            for alert in self.alerts.check(&sample, &self.settings.thresholds, now) {
                let mut line = serde_json::to_value(&alert).unwrap_or_default();
                line["type"] = json!("alert");
                lines.push(line.to_string());
            }
        } else {
            self.culprits = false;
        }
        lines
    }

    /// Answers one request line at `now`, and says when the next sample is due.
    pub fn handle(&mut self, line: &str, now: u64) -> (String, Next) {
        self.refresh();
        let request: Value = match serde_json::from_str(line) {
            Ok(request) => request,
            Err(err) => {
                return (
                    json!({ "id": Value::Null, "error": format!("not JSON: {err}") }).to_string(),
                    Next::Keep,
                );
            }
        };
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let text = |key: &str| request.get(key).and_then(Value::as_str).unwrap_or_default().to_owned();
        let pid = request.get("pid").and_then(Value::as_i64).map(|pid| pid as i32);
        let mut next = Next::Keep;
        let mut reply = match text("op").as_str() {
            "mode" => match Mode::parse(&text("mode")) {
                Some(mode) => {
                    next = self.switch(mode, now);
                    Ok(json!({ "mode": mode.name() }))
                }
                None => Err(format!("no mode {:?}; pill, popup or paused", text("mode"))),
            },
            "process" => pid
                .ok_or_else(|| "no pid".to_owned())
                .and_then(|pid| {
                    processes::details(self.sampler.root(), pid, self.sampler.boot_time(), self.sampler.page())
                        .ok_or_else(|| format!("process {pid} is gone"))
                })
                .map(|details| json!({ "process": details })),
            "signal" => match (pid, Signal::parse(&text("signal"))) {
                (Some(pid), Some(how)) => processes::signal(pid, how)
                    .map(|()| json!({ "ok": true }))
                    .map_err(|err| chain(&err)),
                _ => Err("a signal needs a pid and `term` or `kill`".to_owned()),
            },
            "settings" => Ok(self.settings_json()),
            "set" => match self.settings_path.clone() {
                Some(path) => settings::set(&path, &text("key"), &text("value"))
                    .map_err(|err| chain(&err))
                    .map(|()| {
                        self.reload();
                        self.settings_json()
                    }),
                None => Err("neither XDG_CONFIG_HOME nor HOME is set, so there is no settings file".to_owned()),
            },
            other => Err(format!("no op {other:?}")),
        }
        .unwrap_or_else(|error| json!({ "error": error }));
        reply["id"] = id;
        (reply.to_string(), next)
    }

    fn settings_json(&self) -> Value {
        json!({ "settings": self.settings, "problem": self.problem })
    }

    /// Moves to `mode`. Out of a pause the counters start afresh; into the panel the processes
    /// are scanned, so its first full sample has their CPU use.
    fn switch(&mut self, mode: Mode, now: u64) -> Next {
        let before = self.mode;
        self.mode = mode;
        match (before, mode) {
            (before, after) if before == after => Next::Keep,
            (_, Mode::Paused) => Next::Keep,
            (Mode::Paused, after) => {
                self.sampler.restart(now, after == Mode::Popup);
                Next::Soon
            }
            (_, Mode::Popup) => {
                self.sampler.baseline(now, true);
                Next::Soon
            }
            (_, Mode::Pill | Mode::Quiet) => Next::Now,
        }
    }
}

/// A sample as a stream line, `"type":"sample"` in it.
pub fn sample_line(sample: &Sample) -> String {
    let mut line = serde_json::to_value(sample).unwrap_or_default();
    line["type"] = json!("sample");
    line.to_string()
}

/// The stream file: `$XDG_RUNTIME_DIR/kubaos-sysmon.json`.
pub fn stream_file() -> Option<PathBuf> {
    env::var_os("XDG_RUNTIME_DIR")
        .filter(|dir| !dir.is_empty())
        .map(|dir| PathBuf::from(dir).join(STREAM_FILE))
}

/// The last sample in the stream file, if it is no older than `fresh` ms at `now`.
pub fn read_stream(path: &Path, now: u64, fresh: u64) -> Option<Sample> {
    let sample: Sample = serde_json::from_str(&fs::read_to_string(path).ok()?).ok()?;
    (now.saturating_sub(sample.t) <= fresh).then_some(sample)
}

/// Writes `line` over the stream file through a temporary file and a rename, so the menu never
/// reads half a sample. A failure costs the menu its fresh numbers only, so it is let go.
fn write_stream(path: &Path, line: &str) {
    let tmp = path.with_extension("json.new");
    if fs::write(&tmp, line).is_ok() {
        let _ = fs::rename(&tmp, path);
    }
}

fn modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).and_then(|meta| meta.modified()).ok()
}

/// Now, in ms since the epoch.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as u64)
}

/// An error and each of its sources, joined by `: `.
pub fn chain(err: &dyn std::error::Error) -> String {
    let mut text = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        text.push_str(": ");
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}

/// Serves until `input` ends: samples at the mode's pace on `output`, a reply for each request.
/// Requests are read on a thread of their own, so a sample is never late waiting for one.
///
/// # Errors
///
/// When writing to `output` fails, as when the shell has gone.
pub fn run(server: &mut Server, input: impl BufRead + Send + 'static, mut output: impl Write) -> io::Result<()> {
    let (requests, received) = mpsc::channel::<String>();
    thread::spawn(move || {
        for line in input.lines() {
            let Ok(line) = line else {
                break;
            };
            if requests.send(line).is_err() {
                break;
            }
        }
    });
    let mut due = Instant::now();
    loop {
        let message = match server.mode().interval() {
            None => received.recv().map_err(|_| RecvTimeoutError::Disconnected),
            Some(_) => received.recv_timeout(due.saturating_duration_since(Instant::now())),
        };
        match message {
            Ok(line) if line.trim().is_empty() => {}
            Ok(line) => {
                let (reply, next) = server.handle(&line, now_ms());
                writeln!(output, "{reply}")?;
                match next {
                    Next::Keep => {}
                    Next::Now => due = Instant::now(),
                    Next::Soon => due = Instant::now() + AFTER_RESTART,
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                for line in server.tick(now_ms()) {
                    writeln!(output, "{line}")?;
                }
                due = Instant::now() + server.mode().interval().unwrap_or(PILL_INTERVAL);
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
        output.flush()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
