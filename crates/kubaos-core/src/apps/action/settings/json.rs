//! A JSON settings file edited in place: a key's value is replaced, inserted or taken out where it
//! stands in the text, and nothing else of the file changes, its indentation, spacing, key order
//! and the text of its other values (a number as `1.50` or `1e3`) included.
//!
//! The text is checked as JSON before anything reads it, and every edit leaves JSON, so the
//! scanner here only finds where things are; it never decides what is valid.
//!
//! # Owns
//!
//! - [`JsonText`]: one JSON settings file's text, its keys found and changed in it.
//!
//! # Not here
//!
//! - The format-independent setting: `settings.rs`.

use std::ops::Range;

/// The indentation a new nested object gets when the file shows none to copy: what
/// `JSON.stringify(value, null, 2)`, which writes most programs' settings, uses.
const DEFAULT_INDENT: &str = "  ";

/// What goes between a key and its value when the file has no key to copy it from.
const DEFAULT_COLON: &str = ": ";

/// One JSON settings file's text, its top level an object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonText {
    text: String,
}

/// An object in the text: where its braces are and its members.
#[derive(Debug)]
struct Object {
    open: usize,
    close: usize,
    members: Vec<Member>,
}

/// A key of an object and its value, as they stand in the text.
#[derive(Debug)]
struct Member {
    key: String,
    /// The key's opening quote.
    start: usize,
    /// Just past the key's closing quote.
    key_end: usize,
    value: Range<usize>,
}

/// What the text holds under a dotted key: the objects on the way, from the top level down, as
/// far as they are there.
#[derive(Debug)]
struct Found {
    objects: Vec<Object>,
}

/// Where the dotted key's path goes through a value that isn't an object: the part it was.
#[derive(Debug)]
pub struct Through(pub String);

impl JsonText {
    /// An empty object, the settings of a file that isn't there yet.
    pub fn empty() -> Self {
        Self { text: "{}\n".to_owned() }
    }

    /// `text`, when it is JSON whose top level is an object.
    ///
    /// # Errors
    ///
    /// Why it isn't.
    pub fn parse(text: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let value: serde_json::Value = serde_json::from_str(text)?;
        if !value.is_object() {
            return Err("its top level isn't an object".into());
        }
        let parsed = Self { text: text.to_owned() };
        parsed.root().ok_or("its text can't be scanned")?;
        Ok(parsed)
    }

    /// The file's text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Whether its top level has no key.
    pub fn is_empty(&self) -> bool {
        self.root().is_none_or(|root| root.members.is_empty())
    }

    /// How many of the objects named by `above` are there, from the top level down.
    pub fn tables(&self, above: &[&str]) -> Result<usize, Through> {
        Ok(self.find(above)?.objects.len().saturating_sub(1))
    }

    /// The text of the value of `last` under `above`, when it is there; `Ok(Some(None))` when it
    /// is an object.
    pub fn get(&self, above: &[&str], last: &str) -> Result<Option<Option<String>>, Through> {
        let found = self.find(above)?;
        if found.objects.len() <= above.len() {
            return Ok(None);
        }
        let Some(member) = found.objects.last().and_then(|o| o.member(last)) else {
            return Ok(None);
        };
        let value = &self.text[member.value.clone()];
        Ok(Some((!value.starts_with('{')).then(|| value.to_owned())))
    }

    /// `last` under `above` set to the JSON `value`, the objects above it made where they are
    /// missing.
    pub fn set(&mut self, above: &[&str], last: &str, value: &str) -> Result<(), Through> {
        let found = self.find(above)?;
        let depth = found.objects.len().saturating_sub(1);
        let Some(object) = found.objects.last() else {
            return Ok(());
        };
        if depth == above.len()
            && let Some(member) = object.member(last)
        {
            self.text.replace_range(member.value.clone(), value);
            return Ok(());
        }
        let mut parts: Vec<&str> = above[depth..].to_vec();
        parts.push(last);
        self.insert(object, &parts, value);
        Ok(())
    }

    /// `last` under `above` taken out, then each object above it past the first `keep` that is
    /// left empty.
    pub fn remove(&mut self, above: &[&str], last: &str, keep: usize) -> Result<(), Through> {
        let found = self.find(above)?;
        if found.objects.len() <= above.len() {
            return Ok(());
        }
        let Some(object) = found.objects.last() else {
            return Ok(());
        };
        self.take_out(object, last);
        for depth in (keep..above.len()).rev() {
            let found = self.find(&above[..=depth])?;
            let empty = found.objects.last().is_some_and(|o| o.members.is_empty());
            if !empty || found.objects.len() <= depth + 1 {
                break;
            }
            self.take_out(&found.objects[depth], above[depth]);
        }
        Ok(())
    }

    fn root(&self) -> Option<Object> {
        object(&self.text, skip_ws(&self.text, 0))
    }

    fn find(&self, above: &[&str]) -> Result<Found, Through> {
        let mut objects = Vec::new();
        let Some(mut here) = self.root() else {
            return Ok(Found { objects });
        };
        for part in above {
            let Some(member) = here.member(part) else {
                break;
            };
            let next = match self.text.as_bytes().get(member.value.start) {
                Some(b'{') => object(&self.text, member.value.start),
                _ => None,
            };
            let Some(next) = next else {
                return Err(Through((*part).to_owned()));
            };
            objects.push(here);
            here = next;
        }
        objects.push(here);
        Ok(Found { objects })
    }

    /// Adds the member `parts[0]`, with the objects for the rest of `parts` around `value`, at
    /// the end of `object`, laid out as the object's members already are.
    fn insert(&mut self, object: &Object, parts: &[&str], value: &str) {
        let colon = self.colon();
        let inner = &self.text[object.open + 1..object.close];
        let multiline = if object.members.is_empty() {
            self.root()
                .is_none_or(|root| root.members.is_empty() || self.text[root.open..root.close].contains('\n'))
        } else {
            inner.contains('\n')
        };
        let unit = self.indent_unit();
        let base = line_indent(&self.text, object.open);
        let Some(last) = object.members.last() else {
            let indent = format!("{base}{unit}");
            let member = render(parts, value, &colon, multiline.then_some((&indent, &unit)));
            let interior = if multiline {
                format!("\n{indent}{member}\n{base}")
            } else {
                member
            };
            self.text.replace_range(object.open + 1..object.close, &interior);
            return;
        };
        let gap = match object.members.len() {
            1 => self.text[object.open + 1..last.start].to_owned(),
            n => {
                let before = &object.members[n - 2];
                let comma = self.text[before.value.end..last.start].find(',').map_or(0, |i| i + 1);
                self.text[before.value.end + comma..last.start].to_owned()
            }
        };
        let gap = if gap.is_empty() && !multiline && colon.ends_with(' ') {
            " ".to_owned()
        } else {
            gap
        };
        let indent = gap.rsplit('\n').next().unwrap_or_default().to_owned();
        let member = render(parts, value, &colon, multiline.then_some((&indent, &unit)));
        self.text.insert_str(last.value.end, &format!(",{gap}{member}"));
    }

    /// Takes the member `key` out of `object`, with the comma that joined it to the others.
    fn take_out(&mut self, object: &Object, key: &str) {
        let Some(at) = object.members.iter().rposition(|m| m.key == key) else {
            return;
        };
        let members = &object.members;
        let range = if members.len() == 1 {
            object.open + 1..object.close
        } else if at > 0 {
            members[at - 1].value.end..members[at].value.end
        } else {
            members[0].start..members[1].start
        };
        self.text.replace_range(range, "");
    }

    /// What the file puts between a key and its value, from its first key.
    fn colon(&self) -> String {
        self.root()
            .and_then(|root| {
                let first = root.members.first()?;
                Some(self.text[first.key_end..first.value.start].to_owned())
            })
            .unwrap_or_else(|| DEFAULT_COLON.to_owned())
    }

    /// One level of the file's indentation: how far its first key is indented past the line of
    /// the top level's opening brace.
    fn indent_unit(&self) -> String {
        let Some(root) = self.root() else {
            return DEFAULT_INDENT.to_owned();
        };
        let Some(first) = root.members.first() else {
            return DEFAULT_INDENT.to_owned();
        };
        let before = &self.text[root.open + 1..first.start];
        if !before.contains('\n') {
            return DEFAULT_INDENT.to_owned();
        }
        let indent = line_indent(&self.text, first.start);
        let base = line_indent(&self.text, root.open);
        match indent.strip_prefix(base.as_str()) {
            Some(unit) if !unit.is_empty() => unit.to_owned(),
            _ => DEFAULT_INDENT.to_owned(),
        }
    }
}

impl Object {
    /// The member `key`; the last one, as a JSON reader keeps, when the key is there twice.
    fn member(&self, key: &str) -> Option<&Member> {
        self.members.iter().rev().find(|m| m.key == key)
    }
}

/// The member `parts[0]` holding, through objects for the rest of `parts`, `value`; on lines of
/// their own at `indent`, one `unit` deeper each, when `lines` gives them, else on one line.
fn render(parts: &[&str], value: &str, colon: &str, lines: Option<(&String, &String)>) -> String {
    let Some((first, rest)) = parts.split_first() else {
        return value.to_owned();
    };
    let key = serde_json::to_string(first).unwrap_or_default();
    if rest.is_empty() {
        return format!("{key}{colon}{value}");
    }
    match lines {
        Some((indent, unit)) => {
            let deeper = format!("{indent}{unit}");
            let inner = render(rest, value, colon, Some((&deeper, unit)));
            format!("{key}{colon}{{\n{deeper}{inner}\n{indent}}}")
        }
        None => format!("{key}{colon}{{{}}}", render(rest, value, colon, None)),
    }
}

/// The whitespace that starts the line `at` is on.
fn line_indent(text: &str, at: usize) -> String {
    let start = text[..at].rfind('\n').map_or(0, |i| i + 1);
    text[start..]
        .chars()
        .take_while(|c| matches!(c, ' ' | '\t'))
        .collect()
}

fn skip_ws(text: &str, mut at: usize) -> usize {
    let bytes = text.as_bytes();
    while bytes.get(at).is_some_and(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r')) {
        at += 1;
    }
    at
}

/// Just past the string whose opening quote is at `at`.
fn string_end(bytes: &[u8], at: usize) -> Option<usize> {
    let mut i = at + 1;
    loop {
        match bytes.get(i)? {
            b'\\' => i += 2,
            b'"' => return Some(i + 1),
            _ => i += 1,
        }
    }
}

/// Just past the value that starts at `at`.
fn value_end(bytes: &[u8], at: usize) -> Option<usize> {
    match bytes.get(at)? {
        b'"' => string_end(bytes, at),
        b'{' | b'[' => {
            let mut depth = 0_usize;
            let mut i = at;
            loop {
                match bytes.get(i)? {
                    b'"' => i = string_end(bytes, i)?,
                    b'{' | b'[' => {
                        depth += 1;
                        i += 1;
                    }
                    b'}' | b']' => {
                        depth -= 1;
                        i += 1;
                        if depth == 0 {
                            return Some(i);
                        }
                    }
                    _ => i += 1,
                }
            }
        }
        _ => {
            let mut i = at;
            while bytes
                .get(i)
                .is_some_and(|b| !matches!(b, b',' | b'}' | b']' | b' ' | b'\t' | b'\n' | b'\r'))
            {
                i += 1;
            }
            Some(i)
        }
    }
}

/// The object whose opening brace is at `open`.
fn object(text: &str, open: usize) -> Option<Object> {
    let bytes = text.as_bytes();
    if bytes.get(open) != Some(&b'{') {
        return None;
    }
    let mut members = Vec::new();
    let mut i = skip_ws(text, open + 1);
    if bytes.get(i) == Some(&b'}') {
        return Some(Object {
            open,
            close: i,
            members,
        });
    }
    loop {
        let start = i;
        let key_end = string_end(bytes, start)?;
        let key: String = serde_json::from_str(&text[start..key_end]).ok()?;
        i = skip_ws(text, key_end);
        if bytes.get(i) != Some(&b':') {
            return None;
        }
        let value_start = skip_ws(text, i + 1);
        let end = value_end(bytes, value_start)?;
        members.push(Member {
            key,
            start,
            key_end,
            value: value_start..end,
        });
        i = skip_ws(text, end);
        match bytes.get(i)? {
            b',' => i = skip_ws(text, i + 1),
            b'}' => {
                return Some(Object {
                    open,
                    close: i,
                    members,
                });
            }
            _ => return None,
        }
    }
}
