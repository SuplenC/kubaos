//! A flat `key = value` settings file edited in place, as Sunshine's `sunshine.conf` is: one
//! setting a line, unquoted, and a `#` anywhere starting a comment to the end of its line, as
//! Sunshine's own reader cuts it. A value is the text between the first `=` and the comment,
//! trimmed. A key's value is replaced where it stands, its line's comment kept, or a line is added
//! at the end or taken out, and every other line, blank and comment lines included, stays as it
//! was.
//!
//! # Owns
//!
//! - [`ConfText`]: one such file's text, its keys found and changed in it.
//!
//! # Not here
//!
//! - The format-independent setting: `settings.rs`.

use std::ops::Range;

/// What goes between a key and its value when the line is new.
const SEPARATOR: &str = " = ";

/// One `key = value` settings file's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfText {
    text: String,
}

/// A setting's line: where it starts and ends in the text, its line break left out, its key, and
/// where its value is, trimmed of the spaces around it.
struct Line<'a> {
    start: usize,
    end: usize,
    key: &'a str,
    value: Range<usize>,
}

impl ConfText {
    /// No settings, the text of a file that isn't there yet.
    pub fn empty() -> Self {
        Self { text: String::new() }
    }

    /// `text`, when each of its lines is blank, a comment or a `key = value` setting.
    ///
    /// # Errors
    ///
    /// The first line that is none of them.
    pub fn parse(text: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        for line in text.lines() {
            let line = uncommented(line).trim();
            if line.is_empty() {
                continue;
            }
            match line.split_once('=') {
                Some((key, _)) if !key.trim().is_empty() => {}
                _ => return Err(format!("{line:?} isn't a `key = value` line").into()),
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

    /// The value of `key`'s first line, when it has one.
    pub fn get(&self, key: &str) -> Option<String> {
        self.lines()
            .find(|l| l.key == key)
            .map(|l| self.text[l.value].to_owned())
    }

    /// Sets `key` to `value` on its first line, the line's comment kept, or on a new last line,
    /// and takes out its other lines; `None` takes out every line of it. `value` is one line with
    /// no `#`, which would start a comment.
    pub fn set(&mut self, key: &str, value: Option<&str>) {
        let found: Vec<(usize, usize, Range<usize>)> = self
            .lines()
            .filter(|l| l.key == key)
            .map(|l| (l.start, l.end, l.value))
            .collect();
        let mut text = self.text.clone();
        // From the last line up, so the earlier lines' places stay right.
        for (i, (start, end, old)) in found.iter().cloned().enumerate().rev() {
            match value {
                Some(value) if i == 0 => {
                    // An empty value stands between the `=` and a comment or the line's end: it
                    // gets the spaces a written one has around it.
                    let before = old.is_empty() && !text[..old.start].ends_with(char::is_whitespace);
                    let after = old.is_empty() && text[old.end..].starts_with('#');
                    let value = format!(
                        "{}{value}{}",
                        if before { " " } else { "" },
                        if after { " " } else { "" }
                    );
                    text.replace_range(old, &value);
                }
                _ => {
                    let past = text[end..].find('\n').map_or(text.len(), |n| end + n + 1);
                    text.replace_range(start..past, "");
                }
            }
        }
        if found.is_empty()
            && let Some(value) = value
        {
            let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
            if !text.is_empty() && !text.ends_with('\n') {
                text.push_str(newline);
            }
            for part in [key, SEPARATOR, value, newline] {
                text.push_str(part);
            }
        }
        self.text = text;
    }

    /// The settings' lines, comments and blank lines left out.
    fn lines(&self) -> impl Iterator<Item = Line<'_>> {
        let mut start = 0;
        self.text.split_inclusive('\n').filter_map(move |raw| {
            let here = start;
            start += raw.len();
            let line = raw.trim_end_matches(['\n', '\r']);
            let setting = uncommented(line);
            let (key, value) = setting.split_once('=')?;
            let key = key.trim();
            if key.is_empty() {
                return None;
            }
            let at = here + setting.len() - value.trim_start().len();
            Some(Line {
                start: here,
                end: here + line.len(),
                key,
                value: at..at + value.trim().len(),
            })
        })
    }
}

/// `line` up to its comment: the text before its first `#`.
fn uncommented(line: &str) -> &str {
    line.split_once('#').map_or(line, |(setting, _)| setting)
}

#[cfg(test)]
mod tests {
    use super::ConfText;

    #[test]
    fn a_set_changes_only_the_keys_value_and_keeps_every_other_line() {
        let mut conf = ConfText::parse("# mine\nsunshine_name = den\n\nport=47989  \n").unwrap();

        conf.set("port", Some("48000"));

        assert_eq!(conf.text(), "# mine\nsunshine_name = den\n\nport=48000  \n");
        assert_eq!(conf.get("port").as_deref(), Some("48000"));
    }

    #[test]
    fn a_new_key_goes_on_a_last_line_and_its_removal_leaves_the_text_as_it_was() {
        let mine = "sunshine_name = den";
        let mut conf = ConfText::parse(mine).unwrap();

        conf.set("origin_web_ui_allowed", Some("pc"));
        assert_eq!(conf.text(), "sunshine_name = den\norigin_web_ui_allowed = pc\n");
        conf.set("origin_web_ui_allowed", None);

        assert_eq!(conf.text(), "sunshine_name = den\n");
    }

    #[test]
    fn a_key_there_twice_is_set_on_its_first_line_and_taken_out_whole() {
        let mut conf = ConfText::parse("a = 1\nb = 2\na = 3\n").unwrap();

        conf.set("a", Some("4"));
        assert_eq!(conf.text(), "a = 4\nb = 2\n");
        conf.set("b", None);

        assert_eq!(conf.text(), "a = 4\n");
    }

    #[test]
    fn a_comment_after_a_value_is_left_out_of_it_and_kept_as_the_value_changes() {
        let mut conf = ConfText::parse("origin_web_ui_allowed = pc # mine\nport=47989# base\n").unwrap();

        assert_eq!(conf.get("origin_web_ui_allowed").as_deref(), Some("pc"));
        assert_eq!(conf.get("port").as_deref(), Some("47989"));
        conf.set("port", Some("48000"));

        assert_eq!(conf.text(), "origin_web_ui_allowed = pc # mine\nport=48000# base\n");
    }

    #[test]
    fn an_empty_value_before_a_comment_is_set_with_spaces_around_it() {
        let mut conf = ConfText::parse("key = # to do\n").unwrap();

        assert_eq!(conf.get("key").as_deref(), Some(""));
        conf.set("key", Some("pc"));

        assert_eq!(conf.text(), "key = pc # to do\n");
    }

    #[test]
    fn a_line_that_isnt_a_setting_is_refused() {
        assert!(ConfText::parse("[section]\n").is_err());
        assert!(ConfText::parse("= pc\n").is_err());
        assert!(ConfText::parse("key # = pc\n").is_err());
        assert!(ConfText::parse("  # indented comment\n\nkey =\n").is_ok());
    }

    #[test]
    fn an_empty_value_is_found_empty_and_set_after_its_equals_sign() {
        let mut conf = ConfText::parse("key =\n").unwrap();

        assert_eq!(conf.get("key").as_deref(), Some(""));
        conf.set("key", Some("pc"));

        assert_eq!(conf.text(), "key = pc\n");
    }
}
