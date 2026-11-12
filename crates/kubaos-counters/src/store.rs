//! The counters: their names, values, auto resets and order, as `counters.toml` keeps them, and
//! every change to them. A change says what it logs; writing the file and the log is the caller's.
//!
//! ```toml
//! last = 1
//! next_id = 3
//!
//! [[counters]]
//! id = 1
//! name = "Pushups"
//! value = 12
//! rule = "every day at 9:00"
//! next_reset = "2026-11-13T08:00:00Z"
//!
//! [[counters]]
//! id = 2
//! name = "Coffee"
//! value = 3
//! stop = 4
//! ```
//!
//! A value is never below 0. A counter with a stop never counts past it: +1 at the stop is refused,
//! and a value set above it is the stop. A counter with a rule goes back to 0 when the rule's next
//! time comes; [`Counters::catch_up`] does every reset that came due, so one missed while asleep or
//! switched off happens at the next command.
//!
//! # Owns
//!
//! - The counters' fields and their file, the order, the counter used last, and the rules of each
//!   change.
//!
//! # Not here
//!
//! - The log's lines, groups and sorts: [`crate::history`]. Reading rules:
//!   `kubaos_clock::recurrence`. Locking, the live file and the reset timer: `main.rs`.

use std::fs;
use std::io;
use std::path::Path;

use jiff::Timestamp;
use jiff::Zoned;
use kubaos_clock::recurrence::Recurrence;
use serde::Deserialize;
use serde::Serialize;

use crate::history::Entry;
use crate::history::Kind;

/// The name a new counter takes when it is given none.
pub const DEFAULT_NAME: &str = "Counter";

/// Every counter, in the order they are shown.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    list: Vec<Counter>,
    /// The id of the counter changed last, which the key adds to.
    last: Option<u32>,
    /// The id the next new counter takes; ids are never used twice, so a removed counter's log
    /// lines can't be taken for a new one's.
    next_id: u32,
}

/// One counter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Counter {
    pub id: u32,
    pub name: String,
    pub value: u64,
    pub reset: AutoReset,
    /// When the rule next sets it to 0; only with a rule.
    pub next_reset: Option<Timestamp>,
}

/// When a counter goes back to 0 by itself, or where it stops.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AutoReset {
    #[default]
    Never,
    /// Back to 0 each time the rule comes.
    Rule(Recurrence),
    /// Counts up to this and no further; reset by hand.
    Stop(u64),
}

/// Why a change was refused.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ChangeError {
    #[error("There's no counter named `{0}`")]
    NoSuch(String),
    #[error("There's a counter named `{0}` already")]
    Taken(String),
    #[error("A counter's name can't be empty or hold tabs or line breaks")]
    BadName,
    #[error("{0} is at 0 and can't go lower")]
    AtZero(String),
    #[error("{name} stopped at {stop}; reset it to count again")]
    Stopped { name: String, stop: u64 },
    #[error("A counter can only stop at 1 or more")]
    StopAtZero,
    #[error("There are no counters yet; make one with `kubaos-counters new`")]
    NoCounters,
    #[error("No counter has been used yet; name the one to change")]
    NoLast,
}

/// Why `counters.toml` didn't load.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LoadError {
    #[error("reading {0}")]
    Read(String, #[source] io::Error),
    #[error("reading {0}")]
    Parse(String, #[source] toml::de::Error),
    #[error("counter `{name}` in {path}: {why}")]
    Invalid { path: String, name: String, why: String },
}

/// Why `counters.toml` couldn't be written.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SaveError {
    #[error("turning the counters into TOML")]
    Serialize(#[source] toml::ser::Error),
    #[error("writing {0}")]
    Write(String, #[source] io::Error),
}

impl SaveError {
    fn write(path: &Path) -> impl FnOnce(io::Error) -> SaveError + '_ {
        move |err| SaveError::Write(path.display().to_string(), err)
    }
}

/// The file's shape.
#[derive(Debug, Default, Serialize, Deserialize)]
struct File {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last: Option<u32>,
    #[serde(default)]
    next_id: u32,
    #[serde(default)]
    counters: Vec<FileCounter>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileCounter {
    id: u32,
    name: String,
    #[serde(default)]
    value: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rule: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    next_reset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stop: Option<u64>,
}

impl Counters {
    /// The counters in `path`; none when there is no file.
    ///
    /// # Errors
    ///
    /// When the file can't be read, isn't TOML of this shape, or a counter's rule or stop isn't
    /// one.
    pub fn load(path: &Path) -> Result<Counters, LoadError> {
        let shown = || path.display().to_string();
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Counters::default()),
            Err(err) => return Err(LoadError::Read(shown(), err)),
        };
        let file: File = toml::from_str(&text).map_err(|err| LoadError::Parse(shown(), err))?;
        let invalid = |name: &str, why: String| LoadError::Invalid {
            path: shown(),
            name: name.to_owned(),
            why,
        };
        let mut counters = Vec::with_capacity(file.counters.len());
        for counter in file.counters {
            let reset = match (&counter.rule, counter.stop) {
                (Some(_), Some(_)) => return Err(invalid(&counter.name, "it has both a rule and a stop".to_owned())),
                (Some(rule), None) => {
                    AutoReset::Rule(Recurrence::parse(rule).map_err(|err| invalid(&counter.name, err.to_string()))?)
                }
                (None, Some(0)) => return Err(invalid(&counter.name, ChangeError::StopAtZero.to_string())),
                (None, Some(stop)) => AutoReset::Stop(stop),
                (None, None) => AutoReset::Never,
            };
            let next_reset = match &counter.next_reset {
                Some(text) => Some(
                    text.parse()
                        .map_err(|err: jiff::Error| invalid(&counter.name, err.to_string()))?,
                ),
                None => None,
            };
            counters.push(Counter {
                id: counter.id,
                name: counter.name,
                value: counter.value,
                reset,
                next_reset: next_reset.filter(|_| matches!(reset, AutoReset::Rule(_))),
            });
        }
        let next_id = counters.iter().map(|c| c.id + 1).max().unwrap_or(1).max(file.next_id);
        Ok(Counters {
            list: counters,
            last: file.last,
            next_id,
        })
    }

    /// Writes the counters to `path` through a temporary file and a rename, so no reader sees half.
    ///
    /// # Errors
    ///
    /// When the counters don't make TOML, the folder can't be made or the file can't be written.
    pub fn save(&self, path: &Path) -> Result<(), SaveError> {
        let file = File {
            last: self.last,
            next_id: self.next_id,
            counters: self
                .list
                .iter()
                .map(|counter| FileCounter {
                    id: counter.id,
                    name: counter.name.clone(),
                    value: counter.value,
                    rule: match counter.reset {
                        AutoReset::Rule(rule) => Some(rule.to_string()),
                        _ => None,
                    },
                    next_reset: counter.next_reset.map(|at| at.to_string()),
                    stop: match counter.reset {
                        AutoReset::Stop(stop) => Some(stop),
                        _ => None,
                    },
                })
                .collect(),
        };
        let text = toml::to_string(&file).map_err(SaveError::Serialize)?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(SaveError::write(dir))?;
        }
        let tmp = path.with_extension("toml.new");
        fs::write(&tmp, text).map_err(SaveError::write(&tmp))?;
        fs::rename(&tmp, path).map_err(SaveError::write(path))
    }

    /// The counters, in their order.
    pub fn all(&self) -> &[Counter] {
        &self.list
    }

    /// The counter changed last, if it is still there.
    pub fn last(&self) -> Option<&Counter> {
        self.list.iter().find(|counter| Some(counter.id) == self.last)
    }

    /// The counter named `name`: exactly, or else whatever the case.
    ///
    /// # Errors
    ///
    /// When no counter has that name.
    pub fn find(&self, name: &str) -> Result<&Counter, ChangeError> {
        self.position(name).map(|at| &self.list[at])
    }

    fn position(&self, name: &str) -> Result<usize, ChangeError> {
        let name = name.trim();
        self.list
            .iter()
            .position(|counter| counter.name == name)
            .or_else(|| {
                let lower = name.to_lowercase();
                self.list
                    .iter()
                    .position(|counter| counter.name.to_lowercase() == lower)
            })
            .ok_or_else(|| ChangeError::NoSuch(name.to_owned()))
    }

    /// The name of the counter the key adds to: the one changed last, or the only one.
    ///
    /// # Errors
    ///
    /// When there are no counters, or several and none changed yet.
    pub fn last_name(&self) -> Result<&str, ChangeError> {
        if let Some(counter) = self.last() {
            return Ok(&counter.name);
        }
        match self.list.as_slice() {
            [] => Err(ChangeError::NoCounters),
            [only] => Ok(&only.name),
            _ => Err(ChangeError::NoLast),
        }
    }

    /// Does every reset that came due by `now`, and works out the next one of each rule. A counter
    /// already at 0 is reset without a log line, so a rule every minute doesn't fill the log.
    pub fn catch_up(&mut self, now: &Zoned) -> Vec<Entry> {
        let mut entries = Vec::new();
        for counter in &mut self.list {
            let AutoReset::Rule(rule) = counter.reset else {
                continue;
            };
            match counter.next_reset {
                Some(due) if due <= now.timestamp() => {
                    if counter.value > 0 {
                        counter.value = 0;
                        entries.push(Entry::new(due, counter.id, Kind::AutoReset, 0));
                    }
                    counter.next_reset = rule.next_after(now).map(|at| at.timestamp());
                }
                Some(_) => {}
                None => counter.next_reset = rule.next_after(now).map(|at| at.timestamp()),
            }
        }
        entries
    }

    /// The earliest reset any rule has next.
    pub fn next_reset(&self) -> Option<Timestamp> {
        self.list.iter().filter_map(|counter| counter.next_reset).min()
    }

    /// Adds a counter at 0 at the end. An empty `name` is [`DEFAULT_NAME`], or `Counter 2` and on
    /// when that is taken.
    ///
    /// # Errors
    ///
    /// When the name is taken, or holds a tab or a line break.
    pub fn create(&mut self, name: &str, now: &Zoned) -> Result<Vec<Entry>, ChangeError> {
        let name = name.trim();
        let name = if name.is_empty() {
            self.free_default()
        } else {
            self.free(name, None)?;
            name.to_owned()
        };
        check_name(&name)?;
        let id = self.next_id.max(1);
        self.next_id = id + 1;
        self.list.push(Counter {
            id,
            name: name.clone(),
            value: 0,
            reset: AutoReset::Never,
            next_reset: None,
        });
        self.last = Some(id);
        Ok(vec![Entry::new(now.timestamp(), id, Kind::Created, 0).named(name)])
    }

    /// Adds 1 to `name`.
    ///
    /// # Errors
    ///
    /// When there's no such counter, or it is at its stop.
    pub fn add(&mut self, name: &str, now: &Zoned) -> Result<Vec<Entry>, ChangeError> {
        let counter = self.counter_mut(name)?;
        if let AutoReset::Stop(stop) = counter.reset
            && counter.value >= stop
        {
            return Err(ChangeError::Stopped {
                name: counter.name.clone(),
                stop,
            });
        }
        counter.value += 1;
        Ok(self.changed(name, Kind::Add, now))
    }

    /// Takes 1 from `name`.
    ///
    /// # Errors
    ///
    /// When there's no such counter, or it is at 0.
    pub fn sub(&mut self, name: &str, now: &Zoned) -> Result<Vec<Entry>, ChangeError> {
        let counter = self.counter_mut(name)?;
        if counter.value == 0 {
            return Err(ChangeError::AtZero(counter.name.clone()));
        }
        counter.value -= 1;
        Ok(self.changed(name, Kind::Sub, now))
    }

    /// Sets `name` to `value`, or to its stop when `value` is past it. Setting the value it has
    /// logs nothing.
    ///
    /// # Errors
    ///
    /// When there's no such counter.
    pub fn set(&mut self, name: &str, value: u64, now: &Zoned) -> Result<Vec<Entry>, ChangeError> {
        let counter = self.counter_mut(name)?;
        let value = match counter.reset {
            AutoReset::Stop(stop) => value.min(stop),
            _ => value,
        };
        if counter.value == value {
            self.touch(name);
            return Ok(Vec::new());
        }
        counter.value = value;
        Ok(self.changed(name, Kind::Set, now))
    }

    /// Sets `name` back to 0; at 0 already, it logs nothing.
    ///
    /// # Errors
    ///
    /// When there's no such counter.
    pub fn reset(&mut self, name: &str, now: &Zoned) -> Result<Vec<Entry>, ChangeError> {
        let counter = self.counter_mut(name)?;
        if counter.value == 0 {
            self.touch(name);
            return Ok(Vec::new());
        }
        counter.value = 0;
        Ok(self.changed(name, Kind::Reset, now))
    }

    /// Renames `name` to `to`; a change of case alone is a rename too.
    ///
    /// # Errors
    ///
    /// When there's no such counter, or `to` is taken or isn't a name.
    pub fn rename(&mut self, name: &str, to: &str, now: &Zoned) -> Result<Vec<Entry>, ChangeError> {
        let at = self.position(name)?;
        let to = to.trim();
        check_name(to)?;
        self.free(to, Some(at))?;
        let counter = &mut self.list[at];
        if counter.name == to {
            return Ok(Vec::new());
        }
        to.clone_into(&mut counter.name);
        Ok(vec![
            Entry::new(now.timestamp(), counter.id, Kind::Renamed, counter.value).named(to.to_owned()),
        ])
    }

    /// Gives `name` its auto reset: a rule, a stop, or never. A stop below the value brings the
    /// value down to it, logged as a set.
    ///
    /// # Errors
    ///
    /// When there's no such counter, or the stop is 0.
    pub fn set_reset(&mut self, name: &str, reset: AutoReset, now: &Zoned) -> Result<Vec<Entry>, ChangeError> {
        if reset == AutoReset::Stop(0) {
            return Err(ChangeError::StopAtZero);
        }
        let counter = self.counter_mut(name)?;
        counter.reset = reset;
        counter.next_reset = match reset {
            AutoReset::Rule(rule) => rule.next_after(now).map(|at| at.timestamp()),
            _ => None,
        };
        match reset {
            AutoReset::Stop(stop) if counter.value > stop => {
                counter.value = stop;
                Ok(self.changed(name, Kind::Set, now))
            }
            _ => Ok(Vec::new()),
        }
    }

    /// Moves `name` to `position`, counted from 1; past the end is the end.
    ///
    /// # Errors
    ///
    /// When there's no such counter.
    pub fn move_to(&mut self, name: &str, position: usize) -> Result<(), ChangeError> {
        let at = self.position(name)?;
        let counter = self.list.remove(at);
        let to = position.saturating_sub(1).min(self.list.len());
        self.list.insert(to, counter);
        Ok(())
    }

    /// Removes `name`. Its log lines go with it ([`crate::history::without`]); the line this
    /// returns says it was removed.
    ///
    /// # Errors
    ///
    /// When there's no such counter.
    pub fn remove(&mut self, name: &str, now: &Zoned) -> Result<(Counter, Vec<Entry>), ChangeError> {
        let at = self.position(name)?;
        let counter = self.list.remove(at);
        if self.last == Some(counter.id) {
            self.last = None;
        }
        let entry = Entry::new(now.timestamp(), counter.id, Kind::Removed, counter.value).named(counter.name.clone());
        Ok((counter, vec![entry]))
    }

    fn counter_mut(&mut self, name: &str) -> Result<&mut Counter, ChangeError> {
        let at = self.position(name)?;
        Ok(&mut self.list[at])
    }

    /// Marks `name` used last.
    fn touch(&mut self, name: &str) {
        if let Ok(at) = self.position(name) {
            self.last = Some(self.list[at].id);
        }
    }

    /// Marks `name` used last, and the log line of the change it just had.
    fn changed(&mut self, name: &str, kind: Kind, now: &Zoned) -> Vec<Entry> {
        self.touch(name);
        self.find(name)
            .map(|counter| vec![Entry::new(now.timestamp(), counter.id, kind, counter.value)])
            .unwrap_or_default()
    }

    /// Whether `name` is free, whatever the case, but for the counter at `except`.
    fn free(&self, name: &str, except: Option<usize>) -> Result<(), ChangeError> {
        let lower = name.to_lowercase();
        let taken = self
            .list
            .iter()
            .enumerate()
            .any(|(at, counter)| Some(at) != except && counter.name.to_lowercase() == lower);
        if taken {
            return Err(ChangeError::Taken(name.to_owned()));
        }
        Ok(())
    }

    /// `Counter`, or the first of `Counter 2`, `Counter 3`… that is free. Of one more names than
    /// there are counters, one is.
    fn free_default(&self) -> String {
        (1..=self.list.len() + 1)
            .map(|n| match n {
                1 => DEFAULT_NAME.to_owned(),
                n => format!("{DEFAULT_NAME} {n}"),
            })
            .find(|name| self.free(name, None).is_ok())
            .unwrap_or_else(|| DEFAULT_NAME.to_owned())
    }
}

/// Whether `name` can be a counter's: not empty, with no tab or line break, which the log's lines
/// are split by.
fn check_name(name: &str) -> Result<(), ChangeError> {
    if name.trim().is_empty() || name.chars().any(char::is_control) {
        return Err(ChangeError::BadName);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
