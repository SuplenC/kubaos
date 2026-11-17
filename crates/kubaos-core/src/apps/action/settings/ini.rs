//! An INI settings file with sections edited in place, as Zoom's `zoomus.conf` is: Qt's
//! `QSettings` writes it, `[General]` and the other sections each a header line followed by
//! `key=value` lines. A line starting with `;` or `#` is a comment, and a value is the text after
//! a key's first `=`, trimmed. A key's value is replaced where it stands, a line is added after
//! its section's last line or a section at the end of the file, or a line or an emptied section is
//! taken out, and every other line, blank and comment lines included, stays as it was.
//!
//! Keys before the first header belong to no section, and a section there twice is one section,
//! as `QSettings` reads it.
//!
//! # Owns
//!
//! - [`IniText`]: one such file's text, the keys of its sections found and changed in it.
//!
//! # Not here
//!
//! - The format-independent setting: `settings.rs`.

use std::ops::Range;

/// What goes between a key and its value when the file has no setting to copy it from: Qt's.
const SEPARATOR: &str = "=";

/// One INI settings file's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IniText {
    text: String,
}

/// One line of the file: where it starts, where its text ends before its line break, where the
/// next line starts, and what it is.
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
}

impl IniText {
    /// No settings, the text of a file that isn't there yet.
    pub fn empty() -> Self {
        Self { text: String::new() }
    }

    /// `text`, when each of its lines is blank, a comment, a `[section]` header or a `key=value`
    /// setting.
    ///
    /// # Errors
    ///
    /// The first line that is none of them.
    pub fn parse(text: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || is_comment(line) || header(line).is_some() {
                continue;
            }
            match line.split_once('=') {
                Some((key, _)) if !key.trim().is_empty() => {}
                _ => return Err(format!("{line:?} isn't a `[section]` or a `key=value` line").into()),
            }
        }
        Ok(Self { text: text.to_owned() })
    }

    /// The file's text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Whether it has nothing but blank space.
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }

    /// Whether `section` has a header.
    pub fn has_section(&self, section: &str) -> bool {
        self.lines()
            .any(|l| matches!(l.what, What::Header(name) if name == section))
    }

    /// The value of `key`'s first line in `section`, when it has one.
    pub fn get(&self, section: &str, key: &str) -> Option<String> {
        self.lines().find_map(|l| match l.what {
            What::Setting {
                section: Some(s),
                key: k,
                value,
            } if s == section && k == key => Some(self.text[value].to_owned()),
            _ => None,
        })
    }

    /// Sets `key` in `section` to `value` on its first line, or on a new line after the section's
    /// last, making the section at the end of the file when it has none, and takes out the key's
    /// other lines in it. `None` takes out every line of the key in the section, then, unless
    /// `keep_section`, the section's header once nothing but blank lines is under it, with the
    /// blank line before it that set it apart. `value` is one line.
    pub fn set(&mut self, section: &str, key: &str, value: Option<&str>, keep_section: bool) {
        let found: Vec<(usize, usize, Range<usize>)> = self
            .lines()
            .filter_map(|l| match l.what {
                What::Setting {
                    section: Some(s),
                    key: k,
                    value,
                } if s == section && k == key => Some((l.start, l.past, value)),
                _ => None,
            })
            .collect();
        let newline = if self.text.contains("\r\n") { "\r\n" } else { "\n" };
        let separator = self.separator().to_owned();
        let mut text = self.text.clone();
        // From the last line up, so the earlier lines' places stay right.
        for (i, (start, past, old)) in found.iter().cloned().enumerate().rev() {
            match value {
                Some(value) if i == 0 => {
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
            None if !keep_section => self.remove_empty(section),
            _ => {}
        }
    }

    /// `line` after the last line of `section` that isn't blank, or in a new section at the end.
    fn add(&mut self, section: &str, line: &str, newline: &str) {
        let mut last = None;
        let mut current = None;
        for l in self.lines() {
            if let What::Header(name) = l.what {
                current = Some(name);
            }
            if current == Some(section) && !matches!(l.what, What::Blank) {
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
        if !self.text.is_empty() {
            if !self.text.ends_with('\n') {
                self.text.push_str(newline);
            }
            self.text.push_str(newline);
        }
        for part in ["[", section, "]", newline, line, newline] {
            self.text.push_str(part);
        }
    }

    /// Takes out each header of `section` with nothing but blank lines under it, and the blank
    /// line before it.
    fn remove_empty(&mut self, section: &str) {
        let lines: Vec<Line<'_>> = self.lines().collect();
        let mut gone = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if !matches!(line.what, What::Header(name) if name == section) {
                continue;
            }
            let under = lines[i + 1..].iter().take_while(|l| !matches!(l.what, What::Header(_)));
            if under.into_iter().any(|l| !matches!(l.what, What::Blank)) {
                continue;
            }
            let start = match i.checked_sub(1).map(|b| &lines[b]) {
                Some(before) if matches!(before.what, What::Blank) => before.start,
                _ => line.start,
            };
            gone.push(start..line.past);
        }
        for range in gone.into_iter().rev() {
            self.text.replace_range(range, "");
        }
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
            let here = start;
            start += raw.len();
            let line = raw.trim_end_matches(['\n', '\r']);
            let trimmed = line.trim();
            let what = if trimmed.is_empty() {
                What::Blank
            } else if is_comment(trimmed) {
                What::Comment
            } else if let Some(name) = header(trimmed) {
                section = Some(name);
                What::Header(name)
            } else if let Some((key, value)) = line.split_once('=') {
                let at = here + line.len() - value.trim_start().len();
                What::Setting {
                    section,
                    key: key.trim(),
                    value: at..at + value.trim().len(),
                }
            } else {
                What::Comment
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

fn is_comment(line: &str) -> bool {
    line.starts_with([';', '#'])
}

/// The section a trimmed `[section]` line names.
fn header(line: &str) -> Option<&str> {
    line.strip_prefix('[')?.strip_suffix(']').map(str::trim)
}

#[cfg(test)]
mod tests {
    use super::IniText;

    /// A `zoomus.conf` as Zoom 6 writes it: Qt's `[General]` first, `key=value` with no spaces,
    /// Qt's quoting and `@ByteArray`, and a section of its own after a blank line.
    const ZOOM: &str = "[General]\nGeoLocation=\"Europe/Warsaw\"\nautoPlayGif=false\n\
                        conf.webserver=@ByteArray(https://zoom.us)\nxwayland=true\n\n\
                        [AS]\nshowframewindow=true\n";

    #[test]
    fn a_set_changes_only_the_keys_value_and_keeps_every_other_byte() {
        let mut ini = IniText::parse(ZOOM).unwrap();

        ini.set("General", "xwayland", Some("false"), true);

        assert_eq!(ini.text(), ZOOM.replace("xwayland=true", "xwayland=false"));
        assert_eq!(ini.get("General", "xwayland").as_deref(), Some("false"));
        assert_eq!(ini.get("AS", "xwayland"), None);
    }

    #[test]
    fn a_new_key_goes_after_its_sections_last_line_and_its_removal_leaves_the_text_as_it_was() {
        let mut ini = IniText::parse(ZOOM).unwrap();

        ini.set("General", "enableWaylandShare", Some("true"), true);
        assert_eq!(
            ini.text(),
            ZOOM.replace("xwayland=true\n", "xwayland=true\nenableWaylandShare=true\n")
        );
        ini.set("General", "enableWaylandShare", None, true);

        assert_eq!(ini.text(), ZOOM);
    }

    #[test]
    fn a_missing_section_is_made_at_the_end_and_taken_out_with_its_blank_line_once_empty() {
        let mine = "[AS]\nshowframewindow=true";
        let mut ini = IniText::parse(mine).unwrap();

        ini.set("General", "xwayland", Some("false"), true);
        assert_eq!(ini.text(), "[AS]\nshowframewindow=true\n\n[General]\nxwayland=false\n");
        assert!(ini.has_section("General"));
        ini.set("General", "xwayland", None, false);

        assert_eq!(ini.text(), "[AS]\nshowframewindow=true\n");
        assert!(!ini.has_section("General"));
    }

    #[test]
    fn an_empty_file_gets_the_section_and_the_key_and_is_empty_again_after() {
        let mut ini = IniText::empty();

        ini.set("General", "xwayland", Some("false"), false);
        assert_eq!(ini.text(), "[General]\nxwayland=false\n");
        ini.set("General", "xwayland", None, false);

        assert!(ini.is_empty());
    }

    #[test]
    fn a_section_kept_stays_when_its_last_key_goes() {
        let mut ini = IniText::parse("[General]\nxwayland=false\n").unwrap();

        ini.set("General", "xwayland", None, true);

        assert_eq!(ini.text(), "[General]\n");
    }

    #[test]
    fn a_key_goes_under_a_section_that_ends_the_file_without_a_line_break() {
        let mut ini = IniText::parse("[General]\nautoPlayGif=false").unwrap();

        ini.set("General", "xwayland", Some("false"), true);

        assert_eq!(ini.text(), "[General]\nautoPlayGif=false\nxwayland=false\n");
    }

    #[test]
    fn a_key_there_twice_is_set_on_its_first_line_and_taken_out_whole() {
        let mut ini = IniText::parse("[General]\na=1\nb=2\na=3\n[Other]\na=9\n").unwrap();

        ini.set("General", "a", Some("4"), true);
        assert_eq!(ini.text(), "[General]\na=4\nb=2\n[Other]\na=9\n");
        ini.set("General", "a", None, true);

        assert_eq!(ini.text(), "[General]\nb=2\n[Other]\na=9\n");
    }

    #[test]
    fn the_files_own_spacing_and_line_endings_are_used_for_a_new_line() {
        let mut ini = IniText::parse("; mine\r\n[General]\r\nname = den\r\n").unwrap();

        ini.set("General", "xwayland", Some("false"), true);

        assert_eq!(ini.text(), "; mine\r\n[General]\r\nname = den\r\nxwayland = false\r\n");
    }

    #[test]
    fn a_key_before_any_header_or_in_another_section_isnt_the_sections() {
        let ini = IniText::parse("xwayland=true\n[AS]\nxwayland=true\n").unwrap();

        assert_eq!(ini.get("General", "xwayland"), None);
        assert!(!ini.has_section("General"));
    }

    #[test]
    fn an_empty_value_is_found_empty_and_set_after_its_equals_sign() {
        let mut ini = IniText::parse("[General]\nxwayland=\nname = \n").unwrap();

        assert_eq!(ini.get("General", "xwayland").as_deref(), Some(""));
        ini.set("General", "xwayland", Some("false"), true);
        ini.set("General", "name", Some("den"), true);

        assert_eq!(ini.text(), "[General]\nxwayland=false\nname = den\n");
    }

    #[test]
    fn a_line_that_isnt_a_setting_or_a_section_is_refused() {
        assert!(IniText::parse("[General]\nxwayland\n").is_err());
        assert!(IniText::parse("= false\n").is_err());
        assert!(IniText::parse("  ; indented comment\n# another\n\n[General]\nkey=\n").is_ok());
    }
}
