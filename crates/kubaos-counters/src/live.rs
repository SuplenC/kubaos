//! What the shell reads: the counters as JSON in `$XDG_RUNTIME_DIR/kubaos-counters.json`, written
//! whole after every change and watched by the bar, and the JSON answers of `check` and `log`.
//!
//! ```json
//! {"last": "Pushups",
//!  "counters": [{"id": 1, "name": "Pushups", "value": 12, "stop": 0,
//!                "rule": "every day at 9:00", "next": 1794560400,
//!                "detail": "Resets every day at 9:00"}]}
//! ```
//!
//! `last` is the counter the key adds to, empty when there is none. `stop` is 0 without one, `rule`
//! empty and `next` 0 without a rule, and `detail` the line under the name, empty without either.
//! `next` is in seconds since the epoch.
//!
//! # Owns
//!
//! - The JSON shapes and their words, and writing the state file.
//!
//! # Not here
//!
//! - The counters and their changes: [`crate::store`]. The history's rows: [`crate::history`].

use std::fs;
use std::io;
use std::path::Path;

use jiff::Timestamp;
use jiff::Zoned;
use kubaos_clock::recurrence;
use kubaos_clock::recurrence::Recurrence;
use kubaos_clock::settings::TimeFormat;
use serde_json::Value;
use serde_json::json;

use crate::history::Row;
use crate::store::AutoReset;
use crate::store::Counter;
use crate::store::Counters;

/// The state file's name in `$XDG_RUNTIME_DIR`.
pub const FILE: &str = "kubaos-counters.json";

/// The counters as the shell reads them.
pub fn state(counters: &Counters, format: Option<TimeFormat>) -> Value {
    json!({
        "last": counters.last_name().unwrap_or(""),
        "counters": counters.all().iter().map(|counter| counter_json(counter, format)).collect::<Vec<_>>(),
    })
}

fn counter_json(counter: &Counter, format: Option<TimeFormat>) -> Value {
    let (rule, stop) = match counter.reset {
        AutoReset::Rule(rule) => (rule.to_string(), 0),
        AutoReset::Stop(stop) => (String::new(), stop),
        AutoReset::Never => (String::new(), 0),
    };
    json!({
        "id": counter.id,
        "name": counter.name,
        "value": counter.value,
        "stop": stop,
        "rule": rule,
        "next": counter.next_reset.map_or(0, Timestamp::as_second),
        "detail": detail(counter.reset, format),
    })
}

/// The line under a counter's name: `Resets every day at 9:00`, `Stops at 100`, or nothing.
pub fn detail(reset: AutoReset, format: Option<TimeFormat>) -> String {
    match reset {
        AutoReset::Rule(rule) => format!("Resets {}", rule.phrase(format)),
        AutoReset::Stop(stop) => format!("Stops at {stop}"),
        AutoReset::Never => String::new(),
    }
}

/// Writes the shell's state file at `path` through a temporary file and a rename, so the bar never
/// reads half.
///
/// # Errors
///
/// When the file can't be written.
pub fn write(path: &Path, state: &Value) -> io::Result<()> {
    let tmp = path.with_extension("json.new");
    fs::write(&tmp, format!("{state}\n"))?;
    fs::rename(&tmp, path)
}

/// What `typed` reads as for an auto reset, as `now`, for the box it is typed in: the rule's words
/// and its next time, or why it doesn't read and phrases that do.
///
/// ```json
/// {"ok": true, "rule": "every day at 9:00", "words": "Every day at 9:00",
///  "next": 1794560400, "when": "Next tomorrow at 9:00"}
/// {"ok": false, "error": "`soon` isn't a time…", "examples": ["every day at 9", …]}
/// ```
pub fn check(typed: &str, now: &Zoned, format: Option<TimeFormat>) -> Value {
    match Recurrence::parse(typed) {
        Ok(rule) => {
            let next = rule.next_after(now);
            json!({
                "ok": true,
                "rule": rule.to_string(),
                "words": rule.words(format),
                "next": next.as_ref().map_or(0, |at| at.timestamp().as_second()),
                "when": next.map_or_else(String::new, |at| format!("Next {}", when(&at, now, format))),
            })
        }
        Err(err) => json!({
            "ok": false,
            "error": err.to_string(),
            "examples": recurrence::EXAMPLES,
        }),
    }
}

/// A history's rows as the shell reads them: `at` in seconds since the epoch, `kind` the change's
/// (`add`, `sub`, `set`, `reset`, `auto-reset`) or empty for a group, `label` how it reads (`+1`).
pub fn history(rows: &[Row]) -> Value {
    rows.iter()
        .map(|row| {
            json!({
                "at": row.at.as_second(),
                "kind": row.kind.map_or("", |kind| kind.name()),
                "label": row.kind.map_or("", |kind| kind.label()),
                "value": row.value,
                "added": row.added,
                "changes": row.changes,
            })
        })
        .collect()
}

/// When `at` is, from `now`: `today at 9:00`, `tomorrow at 9:00`, `Monday at 9:00` within the week,
/// or `16 November at 9:00`, the time as `format` writes it.
pub fn when(at: &Zoned, now: &Zoned, format: Option<TimeFormat>) -> String {
    let time = match format {
        Some(TimeFormat::TwelveHour) => at.strftime("%-I:%M %p").to_string(),
        _ => at.strftime("%-H:%M").to_string(),
    };
    match (at.date() - now.date()).get_days() {
        0 => format!("today at {time}"),
        1 => format!("tomorrow at {time}"),
        2..=6 => format!("{} at {time}", at.strftime("%A")),
        _ => format!("{} at {time}", at.strftime("%-d %B")),
    }
}

#[cfg(test)]
mod tests;
