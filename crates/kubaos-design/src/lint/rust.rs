//! The lint's rules for Rust UI code.
//!
//! - **Colour:** `Color::from_rgb*`, `Color::from_linear_rgba`, `Color::WHITE` and `Color::BLACK`,
//!   iced's `color!`, a `Color { r, g, b, a }` literal, and a hex colour string.
//! - **Space, size, radius, font:** a number other than 0 in the arguments of iced's layout calls
//!   (`.padding(`, `.spacing(`, `.width(`, `Length::Fixed(`, `Pixels(`, `.rounded(`, `.size(`…) or
//!   bound to a field that holds one (`width: 1.0`, `radius: 8.0.into()`). Halving (`/ 2.0`) is a
//!   derivation, not a literal.
//! - **Duration:** a number in `Duration::from_millis(` and its siblings.
//! - **Font:** a family named in `Font::with_name(` or `Family::Name(`.
//! - **Component:** iced's own controls and layout (`widget::button`, `text_input`, `row!`), which
//!   `kubaos-ui` wraps on the grid. Containers, scrolling, canvas and images have no library
//!   counterpart yet and are left alone.

use crate::lint::Hit;
use crate::lint::Rule;
use crate::lint::source;
use crate::lint::source::Line;

/// Calls whose number arguments are design values, and the rule each breaks.
const CALLS: &[(&str, Rule)] = &[
    (".padding(", Rule::Space),
    (".spacing(", Rule::Space),
    ("Padding::new(", Rule::Space),
    ("Padding::from(", Rule::Space),
    (".width(", Rule::Size),
    (".height(", Rule::Size),
    (".max_width(", Rule::Size),
    (".max_height(", Rule::Size),
    ("Length::Fixed(", Rule::Size),
    ("Size::new(", Rule::Size),
    ("Vector::new(", Rule::Size),
    ("Point::new(", Rule::Size),
    ("Pixels(", Rule::Size),
    (".rounded(", Rule::Radius),
    ("Radius::new(", Rule::Radius),
    ("radius::Radius::from(", Rule::Radius),
    (".size(", Rule::Font),
    (".line_height(", Rule::Font),
    ("LineHeight::Absolute(", Rule::Font),
    ("LineHeight::Relative(", Rule::Font),
    ("Duration::from_millis(", Rule::Duration),
    ("Duration::from_secs(", Rule::Duration),
    ("Duration::from_secs_f32(", Rule::Duration),
    ("Duration::from_secs_f64(", Rule::Duration),
    ("Duration::from_micros(", Rule::Duration),
];

/// Struct fields whose number is a design value.
const FIELDS: &[(&str, Rule)] = &[
    ("width", Rule::Size),
    ("height", Rule::Size),
    ("radius", Rule::Radius),
    ("blur_radius", Rule::Size),
    ("spacing", Rule::Space),
    ("top", Rule::Space),
    ("right", Rule::Space),
    ("bottom", Rule::Space),
    ("left", Rule::Space),
];

const COLOUR_CALLS: &[&str] = &[
    "Color::from_rgb(",
    "Color::from_rgba(",
    "Color::from_rgb8(",
    "Color::from_rgba8(",
    "Color::from_linear_rgba(",
    "Color::WHITE",
    "Color::BLACK",
    "color!(",
];

const FONT_CALLS: &[&str] = &["Font::with_name(", "Family::Name("];

/// iced's widgets that `kubaos-ui` replaces.
const WIDGETS: &[&str] = &[
    "button",
    "text_input",
    "text",
    "rich_text",
    "toggler",
    "checkbox",
    "radio",
    "pick_list",
    "combo_box",
    "slider",
    "vertical_slider",
    "progress_bar",
    "tooltip",
    "text_editor",
    "column",
    "row",
    "grid",
    "Button",
    "TextInput",
    "Text",
    "Toggler",
    "Checkbox",
    "Radio",
    "PickList",
    "ComboBox",
    "Slider",
    "ProgressBar",
    "Tooltip",
    "Column",
    "Row",
    "Grid",
];

/// iced's macros that build the same.
const MACROS: &[&str] = &["column!", "row!", "text!", "rich_text!"];

/// Each line's hits, up to the file's inline test module. `longest_ms` is the longest duration token:
/// a literal duration longer than any animation is a timeout or a poll, not motion.
pub fn hits(lines: &[Line], longest_ms: u32) -> Vec<Vec<Hit>> {
    let mut out = Vec::with_capacity(lines.len());
    let mut in_tests = false;
    for (index, line) in lines.iter().enumerate() {
        let opens_tests = line.bare.trim() == "#[cfg(test)]"
            && lines
                .get(index + 1)
                .is_some_and(|next| next.bare.trim_start().starts_with("mod ") && next.bare.trim_end().ends_with('{'));
        in_tests |= opens_tests;
        out.push(if in_tests { Vec::new() } else { line_hits(line, longest_ms) });
    }
    out
}

fn line_hits(line: &Line, longest_ms: u32) -> Vec<Hit> {
    let bare = line.bare.as_str();
    let mut hits = Vec::new();
    let column_of = |byte: usize| bare[..byte].chars().count();
    for (column, content) in &line.strings {
        if source::is_hex_colour(content) {
            hits.push(Hit {
                column: *column,
                rule: Rule::Colour,
                what: format!("\"{content}\""),
            });
        }
    }
    for call in COLOUR_CALLS {
        for (byte, _) in bare.match_indices(call) {
            // `Color::from_rgb8(c.r, c.g, c.b)` converts a token's colour; only numbers make one up.
            let made_up =
                !call.ends_with('(') || !source::numbers(&bare[byte..close(bare, byte + call.len())]).is_empty();
            if word_starts(bare, byte) && made_up {
                hits.push(Hit {
                    column: column_of(byte),
                    rule: Rule::Colour,
                    what: call.trim_end_matches('(').to_owned(),
                });
            }
        }
    }
    if let Some(byte) = colour_literal(bare) {
        hits.push(Hit {
            column: column_of(byte),
            rule: Rule::Colour,
            what: "Color { r, g, b, a }".to_owned(),
        });
    }
    for call in FONT_CALLS {
        for (byte, _) in bare.match_indices(call) {
            let open = column_of(byte + call.len());
            if let Some((column, content)) = line.strings.iter().find(|(c, _)| *c >= open) {
                hits.push(Hit {
                    column: *column,
                    rule: Rule::Font,
                    what: format!("{}\"{content}\")", call),
                });
            }
        }
    }
    let colour_spans: Vec<(usize, usize)> = COLOUR_CALLS
        .iter()
        .flat_map(|call| bare.match_indices(call).map(|(byte, _)| (byte, close(bare, byte + call.len()))))
        .collect();
    for (call, rule) in CALLS {
        for (byte, _) in bare.match_indices(call) {
            if !call.starts_with('.') && !word_starts(bare, byte) {
                continue;
            }
            let open = byte + call.len();
            let end = close(bare, open);
            if *rule == Rule::Duration && !is_motion(&bare[open..end], call, longest_ms) {
                continue;
            }
            literal_hits(bare, open, end, *rule, call.trim_end_matches('('), &colour_spans, &mut hits);
        }
    }
    for (field, rule) in FIELDS {
        let pattern = format!("{field}:");
        for (byte, _) in bare.match_indices(&pattern) {
            let after = &bare[byte + pattern.len()..];
            let is_field = word_starts(bare, byte) && !after.starts_with(':');
            let value = after.trim_start();
            let starts_with_number = value.chars().next().is_some_and(|c| c.is_ascii_digit());
            if is_field && starts_with_number {
                let open = bare.len() - value.len();
                let end = open + value.find([',', '}', ';']).unwrap_or(value.len());
                literal_hits(bare, open, end, *rule, field, &colour_spans, &mut hits);
            }
        }
    }
    component_hits(bare, &mut hits);
    hits.sort_by_key(|hit| hit.column);
    // One number inside two matched calls (`.padding(Padding::new(8))`) is one literal.
    hits.dedup_by(|a, b| a.column == b.column && a.rule == b.rule && a.rule != Rule::Component);
    hits
}

/// Whether a `Duration` built from `argument` by `call` is one an animation could take: a number
/// written in place (not a conversion like `(ms * 1000.0)`), no longer than `longest_ms`.
fn is_motion(argument: &str, call: &str, longest_ms: u32) -> bool {
    let argument = argument.trim();
    if !argument.starts_with(|c: char| c.is_ascii_digit()) {
        return false;
    }
    let Some(number) = source::numbers(argument).first().copied() else {
        return false;
    };
    let per_ms = match call {
        "Duration::from_millis(" => 1.0,
        "Duration::from_micros(" => 0.001,
        _ => 1000.0,
    };
    number.value * per_ms <= f64::from(longest_ms)
}

/// The numbers other than 0 between bytes `open` and `end` of `bare`, as hits of `rule`.
fn literal_hits(
    bare: &str,
    open: usize,
    end: usize,
    rule: Rule,
    what: &str,
    colour_spans: &[(usize, usize)],
    hits: &mut Vec<Hit>,
) {
    let span = &bare[open..end];
    for number in source::numbers(span) {
        let byte = open + source::byte_index(span, number.column);
        let in_colour = colour_spans.iter().any(|&(start, stop)| byte > start && byte < stop);
        if number.value == 0.0 || source::is_halving(span, number) || in_colour {
            continue;
        }
        let literal: String = span.chars().skip(number.column).take(number.end - number.column).collect();
        hits.push(Hit {
            column: bare[..byte].chars().count(),
            rule,
            what: format!("{what} {literal}"),
        });
    }
}

/// The byte where the parenthesis opened before `open` closes, or the line's end.
fn close(bare: &str, open: usize) -> usize {
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

/// Whether byte `at` of `bare` starts a word: no name or path continues into it.
fn word_starts(bare: &str, at: usize) -> bool {
    bare[..at]
        .chars()
        .next_back()
        .is_none_or(|c| !(c.is_alphanumeric() || c == '_'))
}

/// The byte a `Color { r: …` literal starts at, if the line has one.
fn colour_literal(bare: &str) -> Option<usize> {
    bare.match_indices("Color {").map(|(byte, _)| byte).find(|&byte| {
        word_starts(bare, byte) && {
            let inside = bare[byte + "Color {".len()..].trim_start();
            inside.starts_with("r:") || inside.starts_with("r ")
        }
    })
}

/// iced's own controls: imported (`use iced::widget::button;`, `use iced::widget::{row, text}`),
/// built by path (`widget::button(`, `widget::Row::new(`), or built with a macro (`row![`). A path
/// to a style or status type (`widget::button::Style`) builds nothing.
fn component_hits(bare: &str, hits: &mut Vec<Hit>) {
    let trimmed = bare.trim_start();
    let indent = bare.len() - trimmed.len();
    if let Some(imported) = trimmed.strip_prefix("use iced::widget::") {
        let names = imported.trim_end().trim_end_matches(';');
        let names = names.trim_start_matches('{').trim_end_matches('}');
        for name in names.split(',').map(str::trim) {
            let name = name.split(" as ").next().unwrap_or(name).trim();
            if WIDGETS.contains(&name) {
                hits.push(Hit {
                    column: indent,
                    rule: Rule::Component,
                    what: format!("iced::widget::{name}"),
                });
            }
        }
        return;
    }
    for (byte, _) in bare.match_indices("widget::") {
        let rest = &bare[byte + "widget::".len()..];
        let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        let after = &rest[name.len()..];
        let built = after.starts_with('(') || ["::new(", "::with_children(", "::from_vec("].iter().any(|c| after.starts_with(c));
        let is_iced = bare[..byte].ends_with("iced::") || word_starts(bare, byte);
        if is_iced && built && WIDGETS.contains(&name.as_str()) {
            hits.push(Hit {
                column: bare[..byte].chars().count(),
                rule: Rule::Component,
                what: format!("iced::widget::{name}"),
            });
        }
    }
    for name in MACROS {
        for (byte, _) in bare.match_indices(name) {
            let opens = bare[byte + name.len()..].starts_with(['[', '(']);
            if opens && word_starts(bare, byte) {
                hits.push(Hit {
                    column: bare[..byte].chars().count(),
                    rule: Rule::Component,
                    what: (*name).to_owned(),
                });
            }
        }
    }
}
