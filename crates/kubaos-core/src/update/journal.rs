//! The updater's side of the progress stream: each event to the sink and, as text, to the run's
//! log; the phase in progress and its clock; the end of the last program's output, for a failure;
//! and what the summary learns from the programs' output.
//!
//! # Owns
//!
//! - The update logs, `~/.local/state/kubaos/update/<date>-<command>.log`: their text, and how
//!   many are kept.
//!
//! # Not here
//!
//! - The events: `event.rs`. When each is sent: `updater.rs`.

use std::collections::VecDeque;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

use super::UpdateError;
use super::event::Command;
use super::event::Outcome;
use super::event::Phase;
use super::event::Prompt;
use super::event::Sink;
use super::event::UpdateEvent;
use super::event::chain;
use super::machine::Cmd;

/// How many update logs are kept: the newest, the oldest going when a run starts.
const KEEP_LOGS: usize = 10;

/// How many lines of a failed program's output its failure shows.
pub const TAIL_LINES: usize = 20;

/// What `scripts/dot-session` prints for each part of the session it reloads.
const SESSION_PREFIX: &str = "dot-session: ";

/// How `nix flake update` names an input it moved: `• Updated input 'nixpkgs':`.
const INPUT_MARK: &str = "Updated input '";

/// The phase that runs, since when, and how it will end.
struct Running {
    phase: Phase,
    started: Instant,
    detail: Option<String>,
    skipped: bool,
}

/// Output lines starting with `prefix` count towards `total` (apt's `Setting up`).
struct Count {
    prefix: &'static str,
    done: usize,
    total: usize,
}

/// Sends the updater's events, and writes them to the run's log.
pub struct Journal<'a> {
    sink: Option<&'a mut dyn Sink>,
    log: Option<(PathBuf, File)>,
    started: Instant,
    phase: Option<Running>,
    /// The phase that failed, for the `failed` event.
    failed: Option<Phase>,
    /// The end of the running program's output; emptied when it succeeds.
    tail: VecDeque<String>,
    count: Option<Count>,
    /// What the session reloaded, from `scripts/dot-session`'s lines in the switch's output.
    pub reloaded: Vec<String>,
    /// The user flake's inputs `nix flake update` moved.
    pub inputs: Vec<String>,
}

impl fmt::Debug for Journal<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Journal")
            .field("log", &self.log.as_ref().map(|(path, _)| path))
            .field("phase", &self.phase.as_ref().map(|running| running.phase))
            .finish_non_exhaustive()
    }
}

impl<'a> Journal<'a> {
    /// A journal that sends nowhere and writes no log.
    pub fn new() -> Self {
        Self {
            sink: None,
            log: None,
            started: Instant::now(),
            phase: None,
            failed: None,
            tail: VecDeque::new(),
            count: None,
            reloaded: Vec::new(),
            inputs: Vec::new(),
        }
    }

    pub fn send_to(&mut self, sink: &'a mut dyn Sink) {
        self.sink = Some(sink);
    }

    /// Starts the log `<dir>/<stamp>-<command>.log`, appending when it exists, and drops the oldest
    /// logs beyond [`KEEP_LOGS`].
    pub fn open_log(&mut self, dir: &Path, stamp: &str, command: Command) -> io::Result<()> {
        fs::create_dir_all(dir)?;
        prune_logs(dir, KEEP_LOGS - 1)?;
        let path = dir.join(format!("{stamp}-{}.log", command.word()));
        let file = File::options().create(true).append(true).open(&path)?;
        self.log = Some((path, file));
        Ok(())
    }

    pub fn log_path(&self) -> Option<PathBuf> {
        self.log.as_ref().map(|(path, _)| path.clone())
    }

    /// Sends `event` and writes it to the log.
    pub fn emit(&mut self, event: &UpdateEvent) {
        if let Some(text) = log_text(event) {
            self.log_line(&text);
        }
        if let Some(sink) = self.sink.as_deref_mut() {
            sink.event(event);
        }
    }

    /// Writes `text` to the log only.
    pub fn log_line(&mut self, text: &str) {
        let secs = self.started.elapsed().as_secs_f64();
        if let Some((_, file)) = &mut self.log {
            for line in text.lines() {
                // A log that can't be written loses lines, never the update.
                let _ = writeln!(file, "[{secs:8.2}] {line}");
            }
        }
    }

    pub fn phase_start(&mut self, phase: Phase) {
        self.phase = Some(Running {
            phase,
            started: Instant::now(),
            detail: None,
            skipped: false,
        });
        self.emit(&UpdateEvent::PhaseStarted { phase });
    }

    /// Sets what the running phase reports when it ends.
    pub fn detail(&mut self, text: impl Into<String>) {
        if let Some(running) = &mut self.phase {
            running.detail = Some(text.into());
        }
    }

    /// Ends the running phase as skipped, for `reason`, when it ends without an error.
    pub fn skip(&mut self, reason: impl Into<String>) {
        if let Some(running) = &mut self.phase {
            running.detail = Some(reason.into());
            running.skipped = true;
        }
    }

    pub fn phase_end(&mut self, ok: bool) {
        let Some(running) = self.phase.take() else {
            return;
        };
        self.count = None;
        let outcome = match (ok, running.skipped) {
            (false, _) => Outcome::Failed,
            (true, true) => Outcome::Skipped,
            (true, false) => Outcome::Done,
        };
        if !ok {
            self.failed = Some(running.phase);
        }
        self.emit(&UpdateEvent::PhaseFinished {
            phase: running.phase,
            outcome,
            elapsed_ms: running.started.elapsed().as_millis() as u64,
            detail: running.detail,
        });
    }

    pub fn step(&mut self, text: impl Into<String>) {
        self.emit(&UpdateEvent::Step { text: text.into() });
    }

    pub fn note(&mut self, text: impl Into<String>) {
        self.emit(&UpdateEvent::Note { text: text.into() });
    }

    pub fn warn(&mut self, text: impl Into<String>) {
        self.emit(&UpdateEvent::Warning { text: text.into() });
    }

    pub fn progress(&mut self, done: usize, total: usize) {
        self.emit(&UpdateEvent::Progress { done, total });
    }

    pub fn prompt(&mut self, prompt: Prompt) {
        self.emit(&UpdateEvent::Prompt { prompt });
    }

    /// Counts the output lines starting with `prefix` as progress towards `total`, until the
    /// phase ends.
    pub fn count(&mut self, prefix: &'static str, total: usize) {
        self.count = (total > 0).then_some(Count { prefix, done: 0, total });
    }

    /// A program starts: its command line goes to the log, and the tail starts over.
    pub fn command(&mut self, cmd: &Cmd) {
        self.command_line(&cmd.display());
    }

    pub fn command_line(&mut self, line: &str) {
        self.tail.clear();
        self.log_line(&format!("$ {line}"));
    }

    /// The running program succeeded: its output is no failure's.
    pub fn succeeded(&mut self) {
        self.tail.clear();
    }

    /// One line the running program printed.
    pub fn output(&mut self, raw: &str) {
        let line = strip_escapes(raw);
        let line = line.trim_end();
        if line.trim().is_empty() {
            return;
        }
        if self.tail.len() == TAIL_LINES {
            self.tail.pop_front();
        }
        self.tail.push_back(line.to_owned());
        self.learn(line);
        let mut progress = None;
        if let Some(count) = &mut self.count
            && line.starts_with(count.prefix)
            && count.done < count.total
        {
            count.done += 1;
            progress = Some((count.done, count.total));
        }
        self.emit(&UpdateEvent::Output { line: line.to_owned() });
        if let Some((done, total)) = progress {
            self.progress(done, total);
        }
    }

    /// Picks up what the summary reports from a line of output.
    fn learn(&mut self, line: &str) {
        if let Some(part) = line.trim_start().strip_prefix(SESSION_PREFIX) {
            self.reloaded.push(part.trim().to_owned());
        }
        if let Some(start) = line.find(INPUT_MARK) {
            let rest = &line[start + INPUT_MARK.len()..];
            if let Some(name) = rest.split('\'').next()
                && !name.is_empty()
                && !self.inputs.iter().any(|known| known == name)
            {
                self.inputs.push(name.to_owned());
            }
        }
    }

    /// Ends the run on `e`: `declined` when the user said no, else `failed`, with the phase that
    /// failed and the end of the failing program's output.
    pub fn failed(&mut self, e: &UpdateError) {
        if matches!(e, UpdateError::Declined) {
            self.emit(&UpdateEvent::Declined);
            return;
        }
        let event = UpdateEvent::Failed {
            phase: self.failed.take(),
            error: chain(e),
            output: self.tail.drain(..).collect(),
            log: self.log_path(),
        };
        self.emit(&event);
    }
}

/// Removes all but the newest `keep` logs in `dir`. Their names start with the date, so the
/// newest sort last.
fn prune_logs(dir: &Path, keep: usize) -> io::Result<()> {
    let mut logs: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "log"))
        .collect();
    logs.sort();
    let extra = logs.len().saturating_sub(keep);
    for old in &logs[..extra] {
        fs::remove_file(old)?;
    }
    Ok(())
}

/// The event as the log's text; none for one the log needn't keep.
fn log_text(event: &UpdateEvent) -> Option<String> {
    Some(match event {
        UpdateEvent::Started { command, .. } => format!("kubaos {}", command.word()),
        UpdateEvent::PhaseStarted { phase } => format!("== {}", phase.title()),
        UpdateEvent::Step { text } => format!("-- {text}"),
        UpdateEvent::Progress { .. } => return None,
        UpdateEvent::Output { line } => format!("   {line}"),
        UpdateEvent::Note { text } => text.clone(),
        UpdateEvent::Warning { text } => format!("warning: {text}"),
        UpdateEvent::Preview(preview) => format!("preview: {}", serde_json::to_string(preview).ok()?),
        UpdateEvent::Prompt {
            prompt: Prompt::Confirm { question },
        } => format!("? {question}"),
        UpdateEvent::Prompt {
            prompt: Prompt::Password,
        } => "? sudo's password".to_owned(),
        UpdateEvent::Answered { yes } => format!("answer: {}", if *yes { "yes" } else { "no" }),
        UpdateEvent::PhaseFinished {
            phase,
            outcome,
            elapsed_ms,
            detail,
        } => {
            let word = match outcome {
                Outcome::Done => "done",
                Outcome::Skipped => "skipped",
                Outcome::Failed => "failed",
            };
            let detail = detail.as_ref().map(|d| format!(": {d}")).unwrap_or_default();
            format!("== {} {word} in {:.1}s{detail}", phase.title(), *elapsed_ms as f64 / 1000.0)
        }
        UpdateEvent::Available {
            update,
            upgrade,
            apt_security,
        } => format!(
            "available: update {}, upgrade {}, {apt_security} apt security",
            update.as_deref().unwrap_or("none"),
            upgrade.as_deref().unwrap_or("none")
        ),
        UpdateEvent::Summary(summary) => format!("summary: {}", serde_json::to_string(summary).ok()?),
        UpdateEvent::Declined => "declined: nothing changed".to_owned(),
        UpdateEvent::Failed { error, .. } => format!("failed: {error}"),
    })
}

/// `line` without terminal escapes (colours, cursor moves, titles) and control characters.
pub fn strip_escapes(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\x1b' => match chars.next() {
                // CSI: parameters, then one final byte in @..~.
                Some('[') => {
                    for c in chars.by_ref() {
                        if ('@'..='~').contains(&c) {
                            break;
                        }
                    }
                }
                // OSC: up to BEL or ST (ESC \).
                Some(']') => {
                    while let Some(c) = chars.next() {
                        if c == '\x07' {
                            break;
                        }
                        if c == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                _ => {}
            },
            '\t' => out.push(' '),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests;
