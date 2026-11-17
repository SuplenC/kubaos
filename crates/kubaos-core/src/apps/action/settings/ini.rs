//! An INI settings file with sections edited in place, as Zoom's `zoomus.conf` is, which Qt's
//! `QSettings` writes: `[section]` header lines, each followed by its `key=value` lines. The lines
//! are read as Qt reads them where it matters for one key:
//!
//! - a line starting with `;` is a comment, and a `;` later in a line, outside quotes and not after
//!   a `\`, ends its value, the rest a comment kept as it is; `#` is no comment;
//! - the keys before the first header and those under a `[general]` header, in any case, are one
//!   section, General;
//! - a key there twice has its last line's value;
//! - a byte order mark at the start is left out of the first line.
//!
//! A value is the text from the key's first `=` to its end, trimmed. A key's value is replaced
//! where it stands, a line is added after its section's last line or a section at the end of the
//! file, or a line, or the section a step appended, is taken out, and every other line, blank and
//! comment lines included, stays as it was. A value continued on the next line, after a `\` or
//! inside quotes, is refused with its file.
//!
//! # Owns
//!
//! - [`IniText`]: one such file's text, the keys of its sections found and changed in it.
//!
//! # Not here
//!
//! - The format-independent setting: `settings.rs`.

#[cfg(test)]
mod tests;

use std::ops::Range;

/// What goes between a key and its value when the file has no setting to copy it from: Qt's.
const SEPARATOR: &str = "=";

const BOM: char = '\u{feff}';

/// One INI settings file's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IniText {
    text: String,
}

/// One line of the file: where it starts (past a byte order mark), where its text ends before its
/// line break, where the next line starts, and what it is.
struct Line<'a> {
    start: usize,
    end: usize,
    past: usize,
    what: What<'a>,
}

enum What<'a> {
    Blank,
    Comment,
    Header(&'a str),
    /// A `key=value` line, in the section of the header above it, if any; its value trimmed.
    Setting {
        section: Option<&'a str>,
        key: &'a str,
        value: Range<usize>,
    },
    /// A line Qt can't read as one setting.
    Other,
}

impl IniText {
    /// No settings, the text of a file that isn't there yet.
    pub fn empty() -> Self {
        Self { text: String::new() }
    }

    /// `text`, when each of its lines is blank, a comment, a `[section]` header or a `key=value`
    /// setting on one line.
    ///
    /// # Errors
    ///
    /// The first line that is none of them.
    pub fn parse(text: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let ini = Self { text: text.to_owned() };
        if let Some(line) = ini.lines().find(|l| matches!(l.what, What::Other)) {
            let line = &ini.text[line.start..line.end];
            return Err(format!("{line:?} isn't a `[section]` or a one-line `key=value`").into());
        }
        Ok(ini)
    }

    /// The file's text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Whether it has nothing but blank space.
    pub fn is_empty(&self) -> bool {
        self.text.trim_start_matches(BOM).trim().is_empty()
    }

    /// Whether `section` is there: a header of it, or for General, a key before the first header.
    pub fn has_section(&self, section: &str) -> bool {
        self.lines().any(|l| match l.what {
            What::Header(name) => same_section(Some(name), section),
            What::Setting { section: None, .. } => same_section(None, section),
            _ => false,
        })
    }

    /// The value of `key`'s last line in `section`, the one Qt reads, when it has one.
    pub fn get(&self, section: &str, key: &str) -> Option<String> {
        self.found(section, key)
            .last()
            .map(|(_, _, value)| self.text[value.clone()].to_owned())
    }

    /// Sets `key` in `section` to `value` on its last line, or on a new line after the section's
    /// last, making the section at the end of the file when it has none, and takes out the key's
    /// other lines in it. `None` takes out every line of the key in the section, then, unless
    /// `keep_section`, the section's header a step appended, once nothing but blank lines is under
    /// it. `value` is one line.
    pub fn set(&mut self, section: &str, key: &str, value: Option<&str>, keep_section: bool) {
        let found = self.found(section, key);
        let newline = if self.text.contains("\r\n") { "\r\n" } else { "\n" };
        let separator = self.separator().to_owned();
        let mut text = self.text.clone();
        // From the last line up, so the earlier lines' places stay right.
        for (i, (start, past, old)) in found.iter().cloned().enumerate().rev() {
            match value {
                Some(value) if i == found.len() - 1 => {
                    // An empty value after a spaced `=` gets the space a written one has before it.
                    let space = old.is_empty() && separator.ends_with(' ') && !text[..old.start].ends_with(' ');
                    text.replace_range(old, &format!("{}{value}", if space { " " } else { "" }));
                }
                _ => text.replace_range(start..past, ""),
            }
        }
        self.text = text;
        match value {
            Some(value) if found.is_empty() => {
                let line = format!("{key}{separator}{value}");
                self.add(section, &line, newline);
            }
            None if !keep_section => self.remove_appended(section),
            _ => {}
        }
    }

    /// Where each line of `key` in `section` starts, where the next line starts, and its value.
    fn found(&self, section: &str, key: &str) -> Vec<(usize, usize, Range<usize>)> {
        self.lines()
            .filter_map(|l| match l.what {
                What::Setting {
                    section: here,
                    key: k,
                    value,
                } if k == key && same_section(here, section) => Some((l.start, l.past, value)),
                _ => None,
            })
            .collect()
    }

    /// `line` after the last line of `section` that isn't blank, or in a new section at the end.
    /// Before the first header, only a setting is General's: a comment there may be the file's.
    fn add(&mut self, section: &str, line: &str, newline: &str) {
        let mut last = None;
        let mut current = None;
        for l in self.lines() {
            if let What::Header(name) = l.what {
                current = Some(name);
            }
            let ours = match (current, &l.what) {
                (_, What::Blank) | (None, What::Comment | What::Header(_) | What::Other) => false,
                (None, What::Setting { .. }) => same_section(None, section),
                (Some(name), _) => same_section(Some(name), section),
            };
            if ours {
                last = Some((l.end, l.past));
            }
        }
        if let Some((end, past)) = last {
            let at = if past == end {
                // The section ends the file with no line break after its last line.
                self.text.insert_str(end, newline);
                end + newline.len()
            } else {
                past
            };
            self.text.insert_str(at, &format!("{line}{newline}"));
            return;
        }
        if !self.is_empty() {
            if !self.text.ends_with('\n') {
                self.text.push_str(newline);
            }
            self.text.push_str(newline);
        }
        for part in ["[", section, "]", newline, line, newline] {
            self.text.push_str(part);
        }
    }

    /// Takes out the header of `section` as a step appends it: the file's last header, nothing
    /// but blank lines after it, with the blank line before it that set it apart. A header
    /// anywhere else, empty or not, was there before or is Qt's, and stays.
    fn remove_appended(&mut self, section: &str) {
        let lines: Vec<Line<'_>> = self.lines().collect();
        let Some(i) = lines.iter().rposition(|l| !matches!(l.what, What::Blank)) else {
            return;
        };
        if !matches!(lines[i].what, What::Header(name) if same_section(Some(name), section)) {
            return;
        }
        let start = match i.checked_sub(1).map(|b| &lines[b]) {
            Some(before) if matches!(before.what, What::Blank) => before.start,
            _ => lines[i].start,
        };
        let past = lines[i].past;
        self.text.replace_range(start..past, "");
    }

    /// What the file's first setting has between its key and its value, else Qt's `=`.
    fn separator(&self) -> &str {
        self.lines()
            .find_map(|l| match l.what {
                What::Setting { key, value, .. } if !value.is_empty() => {
                    let after_key = self.text[l.start..].find(key).map(|n| l.start + n + key.len())?;
                    Some(&self.text[after_key..value.start])
                }
                _ => None,
            })
            .unwrap_or(SEPARATOR)
    }

    /// The file's lines, each with the section it is in.
    fn lines(&self) -> impl Iterator<Item = Line<'_>> {
        let mut start = 0;
        let mut section = None;
        self.text.split_inclusive('\n').map(move |raw| {
            let mut here = start;
            start += raw.len();
            let mut line = raw.trim_end_matches(['\n', '\r']);
            if here == 0
                && let Some(rest) = line.strip_prefix(BOM)
            {
                here = BOM.len_utf8();
                line = rest;
            }
            let trimmed = line.trim();
            let what = if trimmed.is_empty() {
                What::Blank
            } else if trimmed.starts_with(';') {
                What::Comment
            } else if let Some(name) = header(trimmed) {
                section = Some(name);
                What::Header(name)
            } else {
                setting(line, here, section)
            };
            Line {
                start: here,
                end: here + line.len(),
                past: start,
                what,
            }
        })
    }
}

/// `line`, at `here` in the text, as Qt reads a setting: its key up to the first `=` that no `\`
/// escapes, its value from there to a `;` outside quotes, or the line's end.
fn setting<'a>(line: &'a str, here: usize, section: Option<&'a str>) -> What<'a> {
    let bytes = line.as_bytes();
    let mut equals = None;
    let mut end = bytes.len();
    let mut quoted = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            // A `\` last on the line continues the value on the next one.
            b'\\' if i + 1 == bytes.len() => return What::Other,
            b'\\' => i += 1,
            b'"' => quoted = !quoted,
            b'=' if equals.is_none() => equals = Some(i),
            b';' if !quoted => {
                end = i;
                break;
            }
            _ => {}
        }
        i += 1;
    }
    let Some(equals) = equals.filter(|&e| e < end) else {
        return What::Other;
    };
    let key = line[..equals].trim();
    if quoted || key.is_empty() {
        return What::Other;
    }
    let value = &line[equals + 1..end];
    let at = here + equals + 1 + (value.len() - value.trim_start().len());
    What::Setting {
        section,
        key,
        value: at..at + value.trim().len(),
    }
}

/// Whether a header's `found` name, or none before the first header, is `wanted`: General is
/// the keys before any header and a `[general]` in any case, as Qt reads them.
fn same_section(found: Option<&str>, wanted: &str) -> bool {
    let general = |name: &str| name.eq_ignore_ascii_case("general");
    match found {
        None => general(wanted),
        Some(name) if general(wanted) => general(name),
        Some(name) => name == wanted,
    }
}

/// The section a trimmed `[section]` line names.
fn header(line: &str) -> Option<&str> {
    line.strip_prefix('[')?.strip_suffix(']').map(str::trim)
}
