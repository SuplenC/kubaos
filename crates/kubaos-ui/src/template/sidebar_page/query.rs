//! What is typed in one of the sidebar page's two fields (Home's search and the find box), and its
//! caret: the edits the keys and the pointer make.

use crate::component::field::Caret;
use crate::component::field::Pointer;

/// A field's value and caret.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Query {
    pub text: String,
    pub caret: Caret,
}

impl Query {
    /// `text`, the caret at its end.
    pub fn new(text: impl Into<String>) -> Query {
        let text = text.into();
        let caret = Caret::at(text.chars().count());
        Query { text, caret }
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// `typed` in place of the selection, the caret after it.
    pub fn insert(&mut self, typed: &str) {
        let (start, end) = self.caret.range();
        self.splice(start, end, typed);
        self.caret = Caret::at(start + typed.chars().count());
    }

    /// The selection, or the character (`word`: the word) before the caret.
    pub fn backspace(&mut self, word: bool) {
        let (start, end) = self.caret.range();
        let from = if start != end {
            start
        } else if word {
            self.word_start(start)
        } else {
            start.saturating_sub(1)
        };
        self.splice(from, end, "");
        self.caret = Caret::at(from);
    }

    /// The selection, or the character (`word`: the word) after the caret.
    pub fn delete(&mut self, word: bool) {
        let (start, end) = self.caret.range();
        let to = if start != end {
            end
        } else if word {
            self.word_end(end)
        } else {
            (end + 1).min(self.len())
        };
        self.splice(start, to, "");
        self.caret = Caret::at(start);
    }

    /// The caret a character (`word`: a word) to the left.
    pub fn left(&mut self, word: bool) {
        let (start, end) = self.caret.range();
        self.caret = Caret::at(if start != end {
            start
        } else if word {
            self.word_start(start)
        } else {
            start.saturating_sub(1)
        });
    }

    /// The caret a character (`word`: a word) to the right.
    pub fn right(&mut self, word: bool) {
        let (start, end) = self.caret.range();
        self.caret = Caret::at(if start != end {
            end
        } else if word {
            self.word_end(end)
        } else {
            (end + 1).min(self.len())
        });
    }

    pub fn home(&mut self) {
        self.caret = Caret::at(0);
    }

    pub fn end(&mut self) {
        self.caret = Caret::at(self.len());
    }

    pub fn select_all(&mut self) {
        self.caret = Caret::all(&self.text);
    }

    pub fn clear(&mut self) {
        *self = Query::default();
    }

    /// What the pointer did in the field.
    pub fn point(&mut self, pointer: Pointer) {
        self.caret = match pointer {
            Pointer::Press { pos, extend: false } => Caret::at(pos),
            Pointer::Press { pos, extend: true } | Pointer::Drag(pos) => Caret {
                pos,
                anchor: self.caret.anchor,
            },
            Pointer::Double(pos) => Caret {
                pos: self.word_end(pos),
                anchor: self.word_start(pos),
            },
            Pointer::Triple => Caret::all(&self.text),
        }
        .clamp(&self.text);
    }

    fn len(&self) -> usize {
        self.text.chars().count()
    }

    fn chars(&self) -> Vec<char> {
        self.text.chars().collect()
    }

    /// Where the word ending at or before `at` starts: spaces first, then letters.
    fn word_start(&self, at: usize) -> usize {
        let chars = self.chars();
        let mut i = at.min(chars.len());
        while i > 0 && chars[i - 1].is_whitespace() {
            i -= 1;
        }
        while i > 0 && !chars[i - 1].is_whitespace() {
            i -= 1;
        }
        i
    }

    /// Where the word starting at or after `at` ends: spaces first, then letters.
    fn word_end(&self, at: usize) -> usize {
        let chars = self.chars();
        let mut i = at.min(chars.len());
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        while i < chars.len() && !chars[i].is_whitespace() {
            i += 1;
        }
        i
    }

    /// The characters `start..end` replaced with `with`.
    fn splice(&mut self, start: usize, end: usize, with: &str) {
        let byte = |at: usize| self.text.char_indices().nth(at).map_or(self.text.len(), |(b, _)| b);
        let (from, to) = (byte(start), byte(end));
        self.text.replace_range(from..to, with);
    }
}

#[cfg(test)]
mod tests;
