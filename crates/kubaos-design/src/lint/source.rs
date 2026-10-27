//! A source file split into lines with its comments and string literals told apart, so the rules
//! never match inside a comment and only look in strings where they mean to.
//!
//! One scanner serves Rust and QML: both have `//` and `/* */` comments and quoted strings. Rust adds
//! raw strings and character literals, QML single-quoted and template strings.

/// The language a file is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Language {
    Rust,
    Qml,
}

/// One line of a source file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Line {
    /// 1-based.
    pub number: usize,
    /// The code with its comments blanked and string literals' contents blanked, quotes kept: every
    /// column is where it was in the file.
    pub bare: String,
    /// The text of the line's comments, `//` and `/*` included.
    pub comment: String,
    /// The string literals that start or continue on this line: their column and contents.
    pub strings: Vec<(usize, String)>,
    /// The line as written.
    pub text: String,
}

impl Line {
    /// Whether the line is nothing but a comment, or empty.
    pub fn is_comment_only(&self) -> bool {
        self.bare.trim().is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Code,
    LineComment,
    BlockComment(u32),
    /// A quoted string ending at its quote; Rust's `"`, QML's `"`, `'` and `` ` ``.
    Quoted(char),
    /// A Rust raw string ending at `"` and this many `#`.
    Raw(usize),
}

/// `text` split into [`Line`]s.
pub fn lines(text: &str, language: Language) -> Vec<Line> {
    let mut out = Vec::new();
    let mut state = State::Code;
    for (index, raw) in text.lines().enumerate() {
        let chars: Vec<char> = raw.chars().collect();
        let mut line = Line {
            number: index + 1,
            text: raw.to_owned(),
            ..Line::default()
        };
        let mut string = matches!(state, State::Quoted(_) | State::Raw(_)).then(|| (0, String::new()));
        if state == State::LineComment {
            state = State::Code;
        }
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            let next = chars.get(i + 1).copied();
            match state {
                State::Code => {
                    if c == '/' && next == Some('/') {
                        state = State::LineComment;
                        line.comment.extend(&chars[i..]);
                        break;
                    } else if c == '/' && next == Some('*') {
                        state = State::BlockComment(1);
                        line.comment.push_str("/*");
                        line.bare.push_str("  ");
                        i += 2;
                        continue;
                    } else if let Some(hashes) = raw_string_start(&chars, i, language) {
                        let opener = 1 + hashes + 1;
                        state = State::Raw(hashes);
                        string = Some((i, String::new()));
                        line.bare.extend(&chars[i..i + opener]);
                        i += opener;
                        continue;
                    } else if c == '"' || (language == Language::Qml && (c == '\'' || c == '`')) {
                        state = State::Quoted(c);
                        string = Some((i, String::new()));
                        line.bare.push(c);
                    } else if language == Language::Rust && c == '\'' {
                        let length = char_literal_length(&chars, i);
                        line.bare.push('\'');
                        line.bare.extend(std::iter::repeat_n(' ', length.saturating_sub(2)));
                        if length >= 2 {
                            line.bare.push('\'');
                        }
                        i += length.max(1);
                        continue;
                    } else {
                        line.bare.push(c);
                    }
                }
                State::LineComment => unreachable!("a line comment ends its line"),
                State::BlockComment(depth) => {
                    line.bare.push(' ');
                    line.comment.push(c);
                    if c == '*' && next == Some('/') {
                        line.bare.push(' ');
                        line.comment.push('/');
                        state = if depth == 1 {
                            State::Code
                        } else {
                            State::BlockComment(depth - 1)
                        };
                        i += 2;
                        continue;
                    } else if language == Language::Rust && c == '/' && next == Some('*') {
                        state = State::BlockComment(depth + 1);
                    }
                }
                State::Quoted(quote) => {
                    if c == '\\' {
                        line.bare.push_str(if next.is_some() { "  " } else { " " });
                        if let Some((_, s)) = string.as_mut() {
                            s.push(c);
                            s.extend(next);
                        }
                        i += 2;
                        continue;
                    }
                    if c == quote {
                        line.bare.push(c);
                        state = State::Code;
                        line.strings.extend(string.take());
                    } else {
                        line.bare.push(' ');
                        if let Some((_, s)) = string.as_mut() {
                            s.push(c);
                        }
                    }
                }
                State::Raw(hashes) => {
                    if c == '"' && chars[i + 1..].iter().take(hashes).filter(|&&h| h == '#').count() == hashes {
                        line.bare.push('"');
                        line.bare.extend(std::iter::repeat_n('#', hashes));
                        state = State::Code;
                        line.strings.extend(string.take());
                        i += 1 + hashes;
                        continue;
                    }
                    line.bare.push(' ');
                    if let Some((_, s)) = string.as_mut() {
                        s.push(c);
                    }
                }
            }
            i += 1;
        }
        line.strings.extend(string);
        out.push(line);
    }
    out
}

/// The number of `#` of a Rust raw string starting at `at` (`r"`, `r#"`, `br#"`), if one does.
fn raw_string_start(chars: &[char], at: usize, language: Language) -> Option<usize> {
    if language != Language::Rust || chars[at] != 'r' {
        return None;
    }
    let before = at.checked_sub(1).map(|b| chars[b]);
    if before.is_some_and(|b| b != 'b' && (b.is_alphanumeric() || b == '_')) {
        return None;
    }
    let hashes = chars[at + 1..].iter().take_while(|&&c| c == '#').count();
    (chars.get(at + 1 + hashes) == Some(&'"')).then_some(hashes)
}

/// How many characters a Rust character literal starting at `at` spans, quotes included; 1 for a
/// lifetime's quote.
fn char_literal_length(chars: &[char], at: usize) -> usize {
    match chars.get(at + 1) {
        Some('\\') => chars[at + 2..].iter().position(|&c| c == '\'').map_or(1, |end| end + 3),
        Some(_) if chars.get(at + 2) == Some(&'\'') => 3,
        _ => 1,
    }
}

/// A number written in the code.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Number {
    /// The column it starts at.
    pub column: usize,
    pub end: usize,
    pub value: f64,
}

/// Every numeric literal in `code`, which holds no comments or strings. A number that is part of a
/// name (`S40`, `x2`) or a field index (`.0`) isn't one.
pub fn numbers(code: &str) -> Vec<Number> {
    let chars: Vec<char> = code.chars().collect();
    let mut found = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let joined = i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_' || chars[i - 1] == '.');
        if !c.is_ascii_digit() || joined {
            i += 1;
            continue;
        }
        let start = i;
        let mut digits = String::new();
        while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '_') {
            digits.push(chars[i]);
            i += 1;
        }
        if i + 1 < chars.len() && chars[i] == '.' && chars[i + 1].is_ascii_digit() {
            digits.push('.');
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '_') {
                digits.push(chars[i]);
                i += 1;
            }
        } else if i < chars.len() && chars[i] == '.' && chars.get(i + 1).is_none_or(|n| !n.is_alphabetic()) {
            i += 1;
        }
        if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
            let sign = usize::from(matches!(chars.get(i + 1), Some('-' | '+')));
            if chars.get(i + 1 + sign).is_some_and(char::is_ascii_digit) {
                digits.push('e');
                digits.extend(&chars[i + 1..=i + sign]);
                i += 1 + sign;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    digits.push(chars[i]);
                    i += 1;
                }
            }
        }
        while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
            i += 1;
        }
        let value = digits.replace('_', "").parse().unwrap_or(f64::NAN);
        found.push(Number {
            column: start,
            end: i,
            value,
        });
    }
    found
}

/// Whether `number` only derives a value from a token rather than adding one: `/ 2` (centring, a
/// pill's radius), `* 2` and `2 *` (a padding on both sides), `* 0.5`, and a whole count times a
/// name (`cell * 7`, seven columns of a cell). A count times a number is still a literal length.
#[expect(
    clippy::float_cmp,
    reason = "a literal as written is compared with the literals 2 and 0.5"
)]
pub fn is_halving(code: &str, number: Number) -> bool {
    let before = code[..byte_index(code, number.column)].trim_end();
    let after = code[byte_index(code, number.end)..].trim_start();
    let operand_of = |op: char| before.ends_with(op);
    let halving = (number.value == 2.0 && (operand_of('/') || operand_of('*') || after.starts_with('*')))
        || (number.value == 0.5 && operand_of('*'));
    halving || is_count(before, after, number)
}

/// Whether `number`, a whole number, multiplies a name: `cell * 7`, `3 * row.height`.
fn is_count(before: &str, after: &str, number: Number) -> bool {
    if number.value.fract() != 0.0 {
        return false;
    }
    let names_a_value = |token: &str| {
        token
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_' || c == ')' || c == '(')
    };
    let left = before
        .strip_suffix('*')
        .map(|rest| rest.trim_end().chars().rev().collect::<String>());
    let right = after.strip_prefix('*').map(str::trim_start);
    left.is_some_and(|l| names_a_value(&l)) || right.is_some_and(names_a_value)
}

/// The byte index of character `column` in `text`.
pub fn byte_index(text: &str, column: usize) -> usize {
    text.char_indices().nth(column).map_or(text.len(), |(b, _)| b)
}

/// Whether `text` is a colour written in hex: `#rgb`, `#rgba`, `#rrggbb` or `#aarrggbb`.
pub fn is_hex_colour(text: &str) -> bool {
    text.strip_prefix('#')
        .is_some_and(|hex| matches!(hex.len(), 3 | 4 | 6 | 8) && hex.chars().all(|c| c.is_ascii_hexdigit()))
}

/// The byte where the bracket opened just before byte `open` of `bare` closes, or the line's end.
pub fn close(bare: &str, open: usize) -> usize {
    let mut depth = 1;
    for (offset, c) in bare[open..].char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                depth -= 1;
                if depth == 0 {
                    return open + offset;
                }
            }
            _ => {}
        }
    }
    bare.len()
}
