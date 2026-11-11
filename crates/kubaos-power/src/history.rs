//! The battery's history, in `~/.local/state/kubaos/power/history.jsonl`: one JSON object a line,
//! oldest first, so a question about last night needs no journal archaeology.
//!
//! ```json
//! {"type":"sample","at":1793000000,"percent":64.0,"wh":51.2,"watts":7.1,"stop":"power-saver","lid_closed":true,"panel_on":false,"brightness":40,"keep_awake":true,"docked":false,"cpu":3.2,"wakers":{"secs":600,"irqs":[["LOC (Local timer interrupts)",412.5]],"procs":[["qs",12.3]]}}
//! {"type":"sleep","at":1793000000,"end":1793005100,"trigger":"lid","percent_before":100.0,"percent_after":96.0,"wh_before":80.0,"wh_after":76.8,"charger":false,"watts":2.26,"percent_per_hour":2.82,"hw_sleep_us":0,"last_hw_sleep_us":0,"hw_share":0.0}
//! {"type":"event","at":1793000000,"what":"stop","detail":"power-saver","percent":64.0}
//! ```
//!
//! Appending a line costs one `open` and one `write`; the page cache batches them, and nothing here
//! calls `fsync`. A line a crash cut short is skipped when read, and the next service start ends
//! it, so the next append has a line of its own.
//!
//! # Owns
//!
//! - [`Entry`] and the file: appending, reading, and dropping what is past [`RETENTION_DAYS`].
//!
//! # Not here
//!
//! - When an entry is written: [`crate::recorder`]. What it reads from the machine:
//!   [`crate::reading`]. Reading it for a person: [`crate::report`].

use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use serde::Deserialize;
use serde::Serialize;

use crate::memory;

/// The file in the state folder.
pub const FILE: &str = "history.jsonl";
/// How long an entry is kept: a season's worth, enough to compare before and after a change.
pub const RETENTION_DAYS: i64 = 90;
/// How far past the retention the oldest entry may be before the file is rewritten, so a rewrite
/// comes about once a week instead of every day.
pub const PRUNE_SLACK_DAYS: i64 = 7;
/// A day, in seconds.
pub const DAY: i64 = 86_400;
/// The file names the machine's processes, so it is the user's alone.
const MODE: u32 = 0o600;

/// A line of the file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Entry {
    Sample(Sample),
    Sleep(Sleep),
    Event(Event),
}

impl Entry {
    /// When it happened, or began, in seconds since the epoch.
    pub fn at(&self) -> i64 {
        match self {
            Entry::Sample(sample) => sample.at,
            Entry::Sleep(sleep) => sleep.at,
            Entry::Event(event) => event.at,
        }
    }

    /// Its line, newline included.
    ///
    /// # Panics
    ///
    /// Never: the entries are strings, numbers and booleans, which always serialise.
    pub fn line(&self) -> String {
        serde_json::to_string(self).expect("the entries are plain JSON") + "\n"
    }
}

/// A minute on battery.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    pub at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percent: Option<f64>,
    /// The energy left, in watt-hours, when the battery reports one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wh: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub watts: Option<f64>,
    /// The stop's id.
    pub stop: String,
    /// `None` on a machine with no lid to read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lid_closed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub panel_on: Option<bool>,
    /// The backlight, in percent of its range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brightness: Option<u8>,
    #[serde(default)]
    pub keep_awake: bool,
    #[serde(default)]
    pub docked: bool,
    /// How busy the CPU was since the sample before, in percent of all its threads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu: Option<f64>,
    /// On the samples that counted them, what woke the machine since the last count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wakers: Option<Wakers>,
}

/// What woke the machine most over `secs`, each as wakeups per second, highest first.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Wakers {
    pub secs: u32,
    pub irqs: Vec<(String, f64)>,
    pub procs: Vec<(String, f64)>,
}

/// A suspend.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Sleep {
    /// When logind said the machine was going to sleep.
    pub at: i64,
    /// When logind said it woke.
    pub end: i64,
    /// What sent it to sleep, when that is known: `lid`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percent_before: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percent_after: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wh_before: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wh_after: Option<f64>,
    /// A charger was in at either end, or the charge rose: the drain rates mean nothing then, and
    /// are left out.
    #[serde(default)]
    pub charger: bool,
    /// The drain, from the energy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub watts: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percent_per_hour: Option<f64>,
    /// The time in hardware sleep, the difference of `total_hw_sleep` across the suspend, in
    /// microseconds; `None` where the machine doesn't report it, which is not 0 %.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hw_sleep_us: Option<u64>,
    /// `last_hw_sleep` after the suspend, in microseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_hw_sleep_us: Option<u64>,
    /// The share of the sleep spent in hardware sleep, from 0 to 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hw_share: Option<f64>,
}

/// What happened, at a moment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum What {
    /// The service started, which marks a boot or a restart.
    ServiceStart,
    Plug,
    Unplug,
    /// The stop changed; the detail is its id.
    Stop,
    /// The low-battery drop came; the detail is the stop it left.
    LowBattery,
    LidOpen,
    LidClosed,
    KeepAwakeOn,
    KeepAwakeOff,
}

/// A line that says something happened.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub at: i64,
    pub what: What,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percent: Option<f64>,
}

/// `value` to `places` decimals: a few bytes less on every line, and no false precision.
pub fn round(value: f64, places: i32) -> f64 {
    let scale = 10_f64.powi(places);
    (value * scale).round() / scale
}

/// The wall clock, in seconds since the epoch. The monotonic clock doesn't count a suspend.
pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| i64::try_from(since.as_secs()).unwrap_or(i64::MAX))
}

/// The file under the state folder.
pub fn path() -> Option<PathBuf> {
    memory::dir().map(|dir| dir.join(FILE))
}

/// Appends `entry` to the file at `path`, as one write, so a reader never sees half a line.
///
/// # Errors
///
/// When the folder or the file can't be written.
pub fn append(path: &Path, entry: &Entry) -> io::Result<()> {
    let line = entry.line();
    let mut file = match open_for_append(path) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir)?;
            }
            open_for_append(path)?
        }
        opened => opened?,
    };
    file.write_all(line.as_bytes())
}

fn open_for_append(path: &Path) -> io::Result<File> {
    OpenOptions::new().create(true).append(true).mode(MODE).open(path)
}

/// Ends a last line a crash cut short, so the next append starts on a line of its own.
///
/// # Errors
///
/// When the file can't be read or written; a missing one needs nothing.
pub fn end_last_line(path: &Path) -> io::Result<()> {
    let mut file = match OpenOptions::new().read(true).append(true).open(path) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        opened => opened?,
    };
    if file.seek(SeekFrom::End(0))? == 0 {
        return Ok(());
    }
    file.seek(SeekFrom::End(-1))?;
    let mut last = [0_u8];
    file.read_exact(&mut last)?;
    if last[0] != b'\n' {
        file.write_all(b"\n")?;
    }
    Ok(())
}

/// The entries of the file at `path` from `since` on, oldest first. A line that doesn't read, a cut
/// one or one of a later release, is skipped; a missing file has none.
pub fn read(path: &Path, since: i64) -> Vec<Entry> {
    let Ok(file) = File::open(path) else {
        return Vec::new();
    };
    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| serde_json::from_str::<Entry>(&line).ok())
        .filter(|entry| entry.at() >= since)
        .collect()
}

/// The entries in the last `bytes` of the file at `path`, oldest first: what a question about now
/// needs, without reading months of history. The first line of a tail is usually cut, and is
/// skipped as any line that doesn't read.
pub fn read_tail(path: &Path, bytes: u64) -> Vec<Entry> {
    let Ok(mut file) = File::open(path) else {
        return Vec::new();
    };
    let Ok(length) = file.seek(SeekFrom::End(0)) else {
        return Vec::new();
    };
    if file.seek(SeekFrom::Start(length.saturating_sub(bytes))).is_err() {
        return Vec::new();
    }
    let mut tail = Vec::new();
    if file.read_to_end(&mut tail).is_err() {
        return Vec::new();
    }
    String::from_utf8_lossy(&tail)
        .lines()
        .filter_map(|line| serde_json::from_str::<Entry>(line).ok())
        .collect()
}

/// When a line happened, without reading the rest of it into an [`Entry`].
#[derive(Deserialize)]
struct When {
    at: i64,
}

/// Drops what is past [`RETENTION_DAYS`] at `now`, and says whether it rewrote the file. Only the
/// first line is read until the oldest entry is [`PRUNE_SLACK_DAYS`] past the retention, so a
/// rewrite comes about once a week and streams the file, however long it is.
///
/// # Errors
///
/// When the file can't be read or written; a missing one needs nothing.
pub fn prune(path: &Path, now: i64) -> io::Result<bool> {
    let file = match File::open(path) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(false),
        opened => opened?,
    };
    let cutoff = now - RETENTION_DAYS * DAY;
    let mut reader = BufReader::new(file);
    let mut first = String::new();
    reader.read_line(&mut first)?;
    let oldest = serde_json::from_str::<When>(&first).map_or(i64::MIN, |when| when.at);
    if first.is_empty() || oldest >= cutoff - PRUNE_SLACK_DAYS * DAY {
        return Ok(false);
    }
    let tmp = path.with_extension("new");
    let written = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(MODE)
        .open(&tmp)?;
    let mut writer = BufWriter::new(written);
    let keep = |line: &str| serde_json::from_str::<When>(line).is_ok_and(|when| when.at >= cutoff);
    if keep(&first) {
        writer.write_all(first.as_bytes())?;
    }
    for line in reader.lines() {
        let line = line?;
        if keep(&line) {
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
        }
    }
    writer.flush()?;
    drop(writer);
    fs::rename(&tmp, path)?;
    Ok(true)
}

#[cfg(test)]
mod tests;
