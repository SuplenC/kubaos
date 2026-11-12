//! The change log beside `counters.toml`, `counters.log`: a line for every change to every
//! counter, appended and kept forever, until the counter is removed. A line is the time in UTC,
//! the counter's id, the kind of change and the value after it, split by tabs, and the name when
//! the change gives one (`→` is a tab here):
//!
//! ```text
//! 2026-11-12T08:00:00Z → 1 → created → 0 → Pushups
//! 2026-11-12T08:05:13Z → 1 → add → 1
//! 2026-11-13T08:00:00Z → 1 → auto-reset → 0
//! 2026-11-14T10:00:00Z → 1 → renamed → 4 → Push-ups
//! ```
//!
//! Removing a counter takes its lines out ([`without`]) and appends one `removed` line, the only
//! time a line goes. A line that doesn't read is skipped, so a hand edit gone wrong loses that line
//! alone.
//!
//! # Owns
//!
//! - The lines, the kinds, and a counter's history grouped by day, week or month and sorted.
//!
//! # Not here
//!
//! - Which changes there are: [`crate::store`]. Locking: `main.rs`.

use std::cmp::Reverse;
use std::fs;
use std::io;
use std::io::Write as _;
use std::path::Path;

use jiff::Timestamp;
use jiff::ToSpan;
use jiff::Zoned;
use jiff::civil::Date;
use jiff::tz::TimeZone;

/// What a change was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// +1.
    Add,
    /// −1.
    Sub,
    Set,
    /// Set back to 0 by hand.
    Reset,
    /// Set back to 0 by its rule.
    AutoReset,
    Created,
    Renamed,
    Removed,
}

impl Kind {
    const ALL: [Kind; 8] = [
        Kind::Add,
        Kind::Sub,
        Kind::Set,
        Kind::Reset,
        Kind::AutoReset,
        Kind::Created,
        Kind::Renamed,
        Kind::Removed,
    ];

    /// Its name in the log and to the shell.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Add => "add",
            Kind::Sub => "sub",
            Kind::Set => "set",
            Kind::Reset => "reset",
            Kind::AutoReset => "auto-reset",
            Kind::Created => "created",
            Kind::Renamed => "renamed",
            Kind::Removed => "removed",
        }
    }

    fn from_name(name: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|kind| kind.name() == name)
    }

    /// How a person reads it in a list of changes: `+1`, `−1` (U+2212, as wide as the plus),
    /// `Set`, `Reset`, `Auto reset`.
    pub fn label(self) -> &'static str {
        match self {
            Kind::Add => "+1",
            Kind::Sub => "\u{2212}1",
            Kind::Set => "Set",
            Kind::Reset => "Reset",
            Kind::AutoReset => "Auto reset",
            Kind::Created => "Created",
            Kind::Renamed => "Renamed",
            Kind::Removed => "Removed",
        }
    }

    /// Whether the history shows it: the changes to the value, not the counter's own.
    pub fn is_value_change(self) -> bool {
        matches!(self, Kind::Add | Kind::Sub | Kind::Set | Kind::Reset | Kind::AutoReset)
    }

    /// What it adds to a group's total: +1 and −1 count, sets and resets don't.
    fn added(self) -> i64 {
        match self {
            Kind::Add => 1,
            Kind::Sub => -1,
            _ => 0,
        }
    }
}

/// One line of the log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub at: Timestamp,
    /// The counter's id, which a rename keeps.
    pub id: u32,
    pub kind: Kind,
    /// The value after the change.
    pub value: u64,
    /// The name it was given, when created or renamed, or had, when removed.
    pub name: Option<String>,
}

impl Entry {
    #[must_use]
    pub fn new(at: Timestamp, id: u32, kind: Kind, value: u64) -> Self {
        Entry {
            at,
            id,
            kind,
            value,
            name: None,
        }
    }

    #[must_use]
    pub fn named(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Its line, without the line break.
    pub fn line(&self) -> String {
        let at = self.at.strftime("%Y-%m-%dT%H:%M:%SZ");
        let mut line = format!("{at}\t{}\t{}\t{}", self.id, self.kind.name(), self.value);
        if let Some(name) = &self.name {
            line.push('\t');
            line.push_str(name);
        }
        line
    }

    /// The entry a line holds; `None` if it doesn't read.
    pub fn parse(line: &str) -> Option<Entry> {
        let mut fields = line.splitn(5, '\t');
        let at = fields.next()?.parse().ok()?;
        let id = fields.next()?.parse().ok()?;
        let kind = Kind::from_name(fields.next()?)?;
        let value = fields.next()?.parse().ok()?;
        let name = fields.next().map(str::to_owned);
        Some(Entry {
            at,
            id,
            kind,
            value,
            name,
        })
    }
}

/// Every line of the log at `path` that reads, in order; none when there is no log.
///
/// # Errors
///
/// When the log is there but can't be read.
pub fn read(path: &Path) -> io::Result<Vec<Entry>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(text.lines().filter_map(Entry::parse).collect()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(err) => Err(err),
    }
}

/// Appends `entries` to the log at `path`, making it and its folder if they aren't there.
///
/// # Errors
///
/// When the log can't be opened or written.
pub fn append(path: &Path, entries: &[Entry]) -> io::Result<()> {
    if entries.is_empty() {
        return Ok(());
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let text: String = entries.iter().map(|entry| entry.line() + "\n").collect();
    let mut file = fs::OpenOptions::new().create(true).append(true).open(path)?;
    // One write, so a line from another process never lands inside one of these.
    file.write_all(text.as_bytes())
}

/// Takes every line of the counter `id` out of the log at `path`, through a temporary file and a
/// rename. Lines that don't read stay as they are.
///
/// # Errors
///
/// When the log can't be read or written.
pub fn without(path: &Path, id: u32) -> io::Result<()> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };
    let mut kept = String::with_capacity(text.len());
    for line in text.lines() {
        if Entry::parse(line).is_none_or(|entry| entry.id != id) {
            kept.push_str(line);
            kept.push('\n');
        }
    }
    let tmp = path.with_extension("log.new");
    fs::write(&tmp, kept)?;
    fs::rename(&tmp, path)
}

/// How a history is grouped.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Group {
    /// Each change on its own.
    #[default]
    Change,
    Day,
    /// Monday to Sunday.
    Week,
    Month,
}

impl Group {
    /// The group named `name` on the command line: `change`, `day`, `week` or `month`.
    pub fn from_name(name: &str) -> Option<Group> {
        match name {
            "change" | "each" => Some(Group::Change),
            "day" => Some(Group::Day),
            "week" => Some(Group::Week),
            "month" => Some(Group::Month),
            _ => None,
        }
    }

    /// The first day of the group `date` is in; `None` for [`Group::Change`].
    fn start(self, date: Date) -> Option<Date> {
        match self {
            Group::Change => None,
            Group::Day => Some(date),
            Group::Week => date
                .checked_sub(i64::from(date.weekday().to_monday_zero_offset()).days())
                .ok(),
            Group::Month => Some(date.first_of_month()),
        }
    }
}

/// How a history is sorted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sort {
    #[default]
    Newest,
    Oldest,
    /// The highest value first, or grouped, the most added.
    Highest,
}

impl Sort {
    /// The sort named `name` on the command line: `newest`, `oldest` or `highest`.
    pub fn from_name(name: &str) -> Option<Sort> {
        match name {
            "newest" => Some(Sort::Newest),
            "oldest" => Some(Sort::Oldest),
            "highest" | "most" => Some(Sort::Highest),
            _ => None,
        }
    }
}

/// One row of a history: a change, or a day, week or month of them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// When the change was, or the start of its group (midnight, here).
    pub at: Timestamp,
    /// The change's kind; `None` for a group.
    pub kind: Option<Kind>,
    /// The value after the change, or after the group's last.
    pub value: u64,
    /// What was added: 1 or −1 for a change by one, 0 for a set or a reset; a group's sum of them.
    pub added: i64,
    /// How many changes it holds.
    pub changes: usize,
}

/// The value changes of the counter `id` in `entries`, grouped by `group` in `tz`'s days and
/// sorted by `sort`; ties keep the newest first.
pub fn history(entries: &[Entry], id: u32, group: Group, sort: Sort, tz: &TimeZone) -> Vec<Row> {
    let changes = entries
        .iter()
        .filter(|entry| entry.id == id && entry.kind.is_value_change());
    let mut rows: Vec<Row> = Vec::new();
    for entry in changes {
        let row = Row {
            at: entry.at,
            kind: Some(entry.kind),
            value: entry.value,
            added: entry.kind.added(),
            changes: 1,
        };
        let Some(start) = group.start(entry.at.to_zoned(tz.clone()).date()) else {
            rows.push(row);
            continue;
        };
        let start = start
            .to_zoned(tz.clone())
            .map_or(entry.at, |start: Zoned| start.timestamp());
        match rows.last_mut() {
            Some(last) if last.at == start => {
                last.value = entry.value;
                last.added += row.added;
                last.changes += 1;
            }
            _ => rows.push(Row {
                at: start,
                kind: None,
                ..row
            }),
        }
    }
    match sort {
        Sort::Newest => rows.reverse(),
        Sort::Oldest => {}
        Sort::Highest => {
            rows.reverse();
            if group == Group::Change {
                rows.sort_by_key(|row| Reverse(row.value));
            } else {
                rows.sort_by_key(|row| Reverse(row.added));
            }
        }
    }
    rows
}

#[cfg(test)]
mod tests;
