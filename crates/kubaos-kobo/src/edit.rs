//! One-line text editing: the keys a text field is expected to understand,
//! applied to its value and its `Caret` (`kubaos-ui`'s, which the field
//! draws). Pure, so it's tested without a window.
//!
//! Positions are in `char`s, not bytes. Kōbō draws secrets as one bullet
//! per char, so a position means the same thing in the value and on screen.

pub use kubaos_ui::component::field::Caret;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    /// Arrow left/right: `word` jumps words (Ctrl/Alt), `select` extends
    /// the selection (Shift).
    Left {
        word: bool,
        select: bool,
    },
    Right {
        word: bool,
        select: bool,
    },
    Home {
        select: bool,
    },
    End {
        select: bool,
    },
    Backspace {
        word: bool,
    },
    Delete {
        word: bool,
    },
    SelectAll,
    /// Typing or pasting: replaces the selection.
    Insert(String),
    /// Ctrl+U: everything goes.
    Clear,
    /// Cut: the selection goes (the caller copied it first).
    DeleteSelection,
}

fn byte(value: &str, pos: usize) -> usize {
    value
        .char_indices()
        .nth(pos)
        .map_or(value.len(), |(i, _)| i)
}

/// The selected text (empty without a selection).
pub fn selected(value: &str, c: Caret) -> &str {
    let (a, b) = c.clamp(value).range();
    &value[byte(value, a)..byte(value, b)]
}

/// Start of the word before `pos`: skip spaces, then non-spaces.
fn word_left(chars: &[char], pos: usize) -> usize {
    let mut i = pos;
    while i > 0 && !is_word(chars[i - 1]) {
        i -= 1;
    }
    while i > 0 && is_word(chars[i - 1]) {
        i -= 1;
    }
    i
}

/// End of the word after `pos`.
fn word_right(chars: &[char], pos: usize) -> usize {
    let mut i = pos;
    while i < chars.len() && !is_word(chars[i]) {
        i += 1;
    }
    while i < chars.len() && is_word(chars[i]) {
        i += 1;
    }
    i
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The word around `pos` (double-click).
pub fn word_at(value: &str, pos: usize) -> Caret {
    let chars: Vec<char> = value.chars().collect();
    let pos = pos.min(chars.len());
    let on_word = |i: usize| i < chars.len() && is_word(chars[i]);
    if !(on_word(pos) || pos > 0 && is_word(chars[pos - 1])) {
        return Caret::at(pos);
    }
    let mut a = pos;
    while a > 0 && is_word(chars[a - 1]) {
        a -= 1;
    }
    let mut b = pos;
    while b < chars.len() && is_word(chars[b]) {
        b += 1;
    }
    Caret { pos: b, anchor: a }
}

/// Applies `e`. Returns true when the value changed.
pub fn apply(value: &mut String, c: &mut Caret, e: Edit) -> bool {
    *c = c.clamp(value);
    let chars: Vec<char> = value.chars().collect();
    let n = chars.len();
    let (start, end) = c.range();
    let remove = |value: &mut String, c: &mut Caret, a: usize, b: usize| {
        let (ba, bb) = (byte(value, a), byte(value, b));
        value.replace_range(ba..bb, "");
        *c = Caret::at(a);
        a != b
    };
    let mv = |c: &mut Caret, to: usize, select: bool| {
        c.pos = to;
        if !select {
            c.anchor = to;
        }
    };
    match e {
        Edit::Left { word, select } => {
            let to = if c.has_selection() && !select && !word {
                start
            } else if word {
                word_left(&chars, c.pos)
            } else {
                c.pos.saturating_sub(1)
            };
            mv(c, to, select);
            false
        }
        Edit::Right { word, select } => {
            let to = if c.has_selection() && !select && !word {
                end
            } else if word {
                word_right(&chars, c.pos)
            } else {
                (c.pos + 1).min(n)
            };
            mv(c, to, select);
            false
        }
        Edit::Home { select } => {
            mv(c, 0, select);
            false
        }
        Edit::End { select } => {
            mv(c, n, select);
            false
        }
        Edit::SelectAll => {
            *c = Caret { pos: n, anchor: 0 };
            false
        }
        Edit::Backspace { word } => {
            if c.has_selection() {
                remove(value, c, start, end)
            } else if word {
                let a = word_left(&chars, c.pos);
                remove(value, c, a, c.pos)
            } else if c.pos > 0 {
                remove(value, c, c.pos - 1, c.pos)
            } else {
                false
            }
        }
        Edit::Delete { word } => {
            if c.has_selection() {
                remove(value, c, start, end)
            } else if word {
                let b = word_right(&chars, c.pos);
                remove(value, c, c.pos, b)
            } else if c.pos < n {
                remove(value, c, c.pos, c.pos + 1)
            } else {
                false
            }
        }
        Edit::DeleteSelection => remove(value, c, start, end),
        Edit::Clear => {
            let changed = !value.is_empty();
            value.clear();
            *c = Caret::default();
            changed
        }
        Edit::Insert(t) => {
            let t: String = t.chars().filter(|ch| !ch.is_control()).collect();
            let (ba, bb) = (byte(value, start), byte(value, end));
            value.replace_range(ba..bb, &t);
            *c = Caret::at(start + t.chars().count());
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(v: &str, c: Caret, edits: Vec<Edit>) -> (String, Caret) {
        let mut v = v.to_string();
        let mut c = c;
        for e in edits {
            apply(&mut v, &mut c, e);
        }
        (v, c)
    }

    #[test]
    fn insert_in_the_middle_and_move() {
        let (v, c) = run(
            "Kua",
            Caret::at(3),
            vec![
                Edit::Left {
                    word: false,
                    select: false,
                },
                Edit::Insert("b".into()),
            ],
        );
        assert_eq!(v, "Kuba");
        assert_eq!(c, Caret::at(3));
        let (_, c) = run(&v, c, vec![Edit::Home { select: false }]);
        assert_eq!(c, Caret::at(0));
        let (_, c) = run(&v, c, vec![Edit::End { select: false }]);
        assert_eq!(c, Caret::at(4));
    }

    #[test]
    fn select_all_then_type_replaces() {
        let v = "Kuba Example";
        let (v, c) = run(
            v,
            Caret::at(3),
            vec![Edit::SelectAll, Edit::Insert("Kuba".into())],
        );
        assert_eq!(v, "Kuba");
        assert_eq!(c, Caret::at(4));
    }

    #[test]
    fn word_delete_and_jump() {
        let (v, c) = run(
            "correct horse battery",
            Caret::at(21),
            vec![Edit::Backspace { word: true }],
        );
        assert_eq!(v, "correct horse ");
        assert_eq!(c.pos, 14);
        let (_, c) = run(
            &v,
            Caret::at(14),
            vec![Edit::Left {
                word: true,
                select: false,
            }],
        );
        assert_eq!(c.pos, 8);
        let (v, _) = run(
            "correct horse",
            Caret::at(0),
            vec![Edit::Delete { word: true }],
        );
        assert_eq!(v, " horse");
    }

    #[test]
    fn shift_selection_and_backspace() {
        let (v, c) = run(
            "hello world",
            Caret::at(11),
            vec![
                Edit::Left {
                    word: true,
                    select: true,
                },
                Edit::Backspace { word: false },
            ],
        );
        assert_eq!(v, "hello ");
        assert_eq!(c, Caret::at(6));
    }

    #[test]
    fn arrows_collapse_a_selection() {
        let (_, c) = run(
            "abcdef",
            Caret { pos: 4, anchor: 1 },
            vec![Edit::Left {
                word: false,
                select: false,
            }],
        );
        assert_eq!(c, Caret::at(1));
        let (_, c) = run(
            "abcdef",
            Caret { pos: 1, anchor: 4 },
            vec![Edit::Right {
                word: false,
                select: false,
            }],
        );
        assert_eq!(c, Caret::at(4));
    }

    #[test]
    fn multibyte_chars() {
        let (v, c) = run(
            "Zażółć",
            Caret::at(6),
            vec![
                Edit::Left {
                    word: false,
                    select: false,
                },
                Edit::Backspace { word: false },
            ],
        );
        assert_eq!(v, "Zażóć");
        assert_eq!(c, Caret::at(4));
        assert_eq!(selected("Zażółć", Caret { pos: 2, anchor: 5 }), "żół");
    }

    #[test]
    fn double_click_word() {
        assert_eq!(word_at("you@example.com", 5), Caret { pos: 11, anchor: 4 });
        assert_eq!(word_at("a  b", 2), Caret::at(2));
    }

    #[test]
    fn paste_strips_newlines() {
        let (v, _) = run("", Caret::at(0), vec![Edit::Insert("pass\nword\r".into())]);
        assert_eq!(v, "password");
    }
}
