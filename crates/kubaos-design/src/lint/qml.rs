//! The lint's rules for QML.
//!
//! - **Colour:** a hex colour string anywhere, `Qt.rgba`, `Qt.rgb`, `Qt.hsla`, `Qt.hsva`, and a
//!   named colour (`"white"`) bound to a colour property. `"transparent"` is the absence of one.
//! - **Space, size, radius, duration, font:** a number other than 0 in the binding of a property
//!   that holds one ([`category`]), or in a custom `int` or `real` property named for one
//!   (`iconSize`, `cardRadius`). Halving (`/ 2`) is a derivation, not a literal.
//! - **Font:** a family named in a string, and a weight named in place (`Font.Medium`), in a
//!   binding or in a `Qt.font({ … })`.
//! - **A custom number property** set to a literal counts where it is used: `property int rowH: 44`
//!   bound to a `height` is a size, whatever the property is called.
//! - **Component:** Qt Quick Controls' own controls, in a file that imports them, where
//!   `KubaOS.Components` has the component.

use crate::lint::Hit;
use crate::lint::Rule;
use crate::lint::source;
use crate::lint::source::Line;

/// Qt Quick Controls' types that the component library replaces.
const CONTROLS: &[&str] = &[
    "Button",
    "RoundButton",
    "ToolButton",
    "DelayButton",
    "TextField",
    "TextArea",
    "Switch",
    "CheckBox",
    "RadioButton",
    "Slider",
    "RangeSlider",
    "ComboBox",
    "SpinBox",
    "ProgressBar",
    "BusyIndicator",
    "Dial",
    "Tumbler",
    "ToolTip",
    "MenuItem",
    "TabButton",
];

/// The rule a property's literal breaks, by the property's name; `None` for properties that hold
/// no design value (`opacity`, `z`, `interval`, `from`).
fn category(property: &str) -> Option<Rule> {
    let last = property.rsplit('.').next().unwrap_or(property);
    let lower = last.to_ascii_lowercase();
    if property.starts_with("font.") {
        return matches!(
            last,
            "pixelSize" | "pointSize" | "weight" | "letterSpacing" | "wordSpacing"
        )
        .then_some(Rule::Font);
    }
    if matches!(last, "lineHeight") {
        return Some(Rule::Font);
    }
    if lower.ends_with("duration") {
        return Some(Rule::Duration);
    }
    if lower.ends_with("radius") && !lower.contains("blur") {
        return Some(Rule::Radius);
    }
    if lower.ends_with("spacing")
        || lower.ends_with("padding")
        || lower.ends_with("margin")
        || lower.ends_with("margins")
        || lower.ends_with("inset")
        || lower == "gap"
    {
        return Some(Rule::Space);
    }
    let size = matches!(
        last,
        "x" | "y"
            | "width"
            | "height"
            | "implicitWidth"
            | "implicitHeight"
            | "implicitSize"
            | "size"
            | "contentWidth"
            | "contentHeight"
            | "preferredWidth"
            | "preferredHeight"
            | "minimumWidth"
            | "minimumHeight"
            | "maximumWidth"
            | "maximumHeight"
            | "strokeWidth"
            | "lineWidth"
            | "thickness"
            | "sourceSize"
            | "shadowVerticalOffset"
            | "shadowHorizontalOffset"
    ) || lower.ends_with("size")
        || lower.ends_with("width")
        || lower.ends_with("height")
        || lower.ends_with("offset");
    size.then_some(Rule::Size)
}

/// A custom property's rule, by its name, when it is declared as a number.
fn custom_category(declaration: &str, name: &str) -> Option<Rule> {
    let numeric = ["int ", "real ", "double "].iter().any(|t| declaration.contains(t));
    if !numeric {
        return None;
    }
    let lower = name.to_ascii_lowercase();
    if lower.ends_with("duration") || lower.ends_with("delay") {
        Some(Rule::Duration)
    } else if lower.ends_with("radius") {
        Some(Rule::Radius)
    } else if ["spacing", "padding", "margin", "gap", "inset"]
        .iter()
        .any(|w| lower.ends_with(w))
    {
        Some(Rule::Space)
    } else if ["size", "width", "height", "thickness", "stroke"]
        .iter()
        .any(|w| lower.ends_with(w))
    {
        Some(Rule::Size)
    } else {
        None
    }
}

/// A property binding on a line: the declaration before the name (`readonly property int `), the
/// name, and the column its value starts at.
fn binding(bare: &str) -> Option<(&str, &str, usize)> {
    let colon = bare.find(':')?;
    let head = &bare[..colon];
    // An object opened on the same line (`Behavior on x { NumberAnimation { duration: 200 } }`)
    // binds after its brace.
    let head = head.rsplit(['{', ';']).next().unwrap_or(head);
    let trimmed = head.trim_start();
    let name_start = trimmed.rfind(' ').map_or(0, |space| space + 1);
    let name = &trimmed[name_start..];
    let declaration = &trimmed[..name_start];
    let plain_name = !name.is_empty()
        && name.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_')
        && name.chars().next().is_some_and(char::is_alphabetic);
    let declaration_ok = declaration.is_empty() || declaration.trim_end().split(' ').any(|w| w == "property");
    (plain_name && declaration_ok).then(|| (declaration, name, bare[..=colon].chars().count()))
}

/// Whether the file imports Qt Quick Controls.
fn imports_controls(lines: &[Line]) -> bool {
    lines
        .iter()
        .any(|line| line.bare.trim_start().starts_with("import QtQuick.Controls"))
}

/// Each line's hits.
pub fn hits(lines: &[Line]) -> Vec<Vec<Hit>> {
    let controls = imports_controls(lines);
    let mut hits: Vec<Vec<Hit>> = lines.iter().map(|line| line_hits(line, controls)).collect();
    for (index, hit) in used_as_design_values(lines) {
        if !hits[index].iter().any(|h| h.column == hit.column) {
            hits[index].push(hit);
        }
    }
    hits
}

/// A custom number property set to a literal (`property int rowH: 44`), counted by where it is
/// used: bound as a length (alone or in a sum) to a size, a gap, a radius, a duration or a font
/// size anywhere in the file, the literal is that design value, whatever the property's name.
fn used_as_design_values(lines: &[Line]) -> Vec<(usize, Hit)> {
    let mut declared = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let Some((declaration, name, value_at)) = binding(&line.bare) else {
            continue;
        };
        let numeric = ["int ", "real ", "double "].iter().any(|t| declaration.contains(t));
        let value = &line.bare[source::byte_index(&line.bare, value_at)..];
        let literal = value.trim();
        // A negative literal (`-4`) is a number too; its sign comes before the digits.
        let unsigned = literal.strip_prefix('-').map_or(literal, str::trim_start);
        let number = source::numbers(unsigned).first().copied();
        let only_a_number =
            number.is_some_and(|n| n.column == 0 && n.end == unsigned.chars().count() && n.value != 0.0);
        if numeric && only_a_number {
            let column = value_at + value.chars().take_while(|c| c.is_whitespace()).count();
            declared.push((index, name.to_owned(), column, literal.to_owned()));
        }
    }
    let mut found = Vec::new();
    for (index, name, column, literal) in declared {
        let rule = lines.iter().find_map(|line| {
            let (declaration, bound, value_at) = binding(&line.bare)?;
            let rule = if declaration.is_empty() { category(bound) } else { None }?;
            let value = &line.bare[source::byte_index(&line.bare, value_at)..];
            // A factor (`(1 - enter) * 10`, `parent.width * remaining`) scales a value; a length is
            // used as one, alone or in a sum.
            let as_length = !value.contains(['*', '/']);
            (as_length && mentions(value, &name)).then_some(rule)
        });
        if let Some(rule) = rule {
            found.push((
                index,
                Hit {
                    column,
                    rule,
                    what: format!("{name}: {literal}"),
                },
            ));
        }
    }
    found
}

/// Whether `code` names `name` as a whole word: `rowH`, `root.rowH`, never `rowHeight`.
fn mentions(code: &str, name: &str) -> bool {
    code.match_indices(name).any(|(at, _)| {
        let before = code[..at].chars().next_back();
        let after = code[at + name.len()..].chars().next();
        let joined = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
        !joined(before) && !joined(after)
    })
}

/// `Qt.font({ family: "…", pixelSize: 13, weight: Font.Medium })`: the font written in place.
fn font_object_hits(line: &Line, hits: &mut Vec<Hit>) {
    let bare = line.bare.as_str();
    for (at, call) in bare.match_indices("Qt.font(") {
        let open = at + call.len();
        let end = source::close(bare, open);
        let span = &bare[open..end];
        let column_of = |byte: usize| bare[..byte].chars().count();
        for key in ["pixelSize:", "pointSize:", "letterSpacing:", "wordSpacing:"] {
            for (offset, _) in span.match_indices(key) {
                let start = open + offset + key.len();
                let value = &bare[start..end];
                let value = &value[..value.find([',', '}']).unwrap_or(value.len())];
                for number in source::numbers(value).into_iter().filter(|n| n.value != 0.0) {
                    let literal: String = value
                        .chars()
                        .skip(number.column)
                        .take(number.end - number.column)
                        .collect();
                    hits.push(Hit {
                        column: column_of(start + source::byte_index(value, number.column)),
                        rule: Rule::Font,
                        what: format!("Qt.font {} {literal}", key.trim_end_matches(':')),
                    });
                }
            }
        }
        if let Some(offset) = span.find("weight:")
            && span[offset..].contains("Font.")
        {
            hits.push(Hit {
                column: column_of(open + offset),
                rule: Rule::Font,
                what: "Qt.font weight".to_owned(),
            });
        }
        if let Some(offset) = span.find("family:") {
            let after = column_of(open + offset);
            let before_end = column_of(end);
            if let Some((column, content)) = line.strings.iter().find(|(c, _)| *c > after && *c < before_end) {
                hits.push(Hit {
                    column: *column,
                    rule: Rule::Font,
                    what: format!("Qt.font family \"{content}\""),
                });
            }
        }
    }
}

fn line_hits(line: &Line, controls: bool) -> Vec<Hit> {
    let mut hits = Vec::new();
    font_object_hits(line, &mut hits);
    let bare = line.bare.as_str();
    for (column, content) in &line.strings {
        if source::is_hex_colour(content) {
            hits.push(Hit {
                column: *column,
                rule: Rule::Colour,
                what: format!("\"{content}\""),
            });
        }
    }
    for call in ["Qt.rgba(", "Qt.rgb(", "Qt.hsla(", "Qt.hsva(", "Qt.hsl("] {
        for (byte, _) in bare.match_indices(call) {
            hits.push(Hit {
                column: bare[..byte].chars().count(),
                rule: Rule::Colour,
                what: call.trim_end_matches('(').to_owned(),
            });
        }
    }
    if controls {
        let trimmed = bare.trim_start();
        if let Some(name) = CONTROLS.iter().find(|name| {
            trimmed
                .strip_prefix(**name)
                .is_some_and(|rest| rest.trim_start().starts_with('{'))
        }) {
            hits.push(Hit {
                column: bare.len() - trimmed.len(),
                rule: Rule::Component,
                what: format!("QtQuick.Controls {name}"),
            });
        }
    }
    let Some((declaration, name, value_at)) = binding(bare) else {
        return hits;
    };
    let value = &bare[source::byte_index(bare, value_at)..];
    let rule = if declaration.is_empty() {
        category(name)
    } else {
        custom_category(declaration, name)
    };
    let family = name == "font.family" || name.ends_with(".family");
    if family && let Some((column, content)) = line.strings.iter().find(|(c, _)| *c >= value_at) {
        hits.push(Hit {
            column: *column,
            rule: Rule::Font,
            what: format!("{name}: \"{content}\""),
        });
    }
    if name == "font.weight" && value.contains("Font.") {
        hits.push(Hit {
            column: value_at + value.chars().take_while(|c| c.is_whitespace()).count(),
            rule: Rule::Font,
            what: format!("{name}: {}", value.trim()),
        });
    }
    let lower = name.to_ascii_lowercase();
    if lower.ends_with("color") || lower.ends_with("colour") || declaration.contains("color ") {
        for (column, content) in line.strings.iter().filter(|(c, _)| *c >= value_at) {
            let before = bare[..source::byte_index(bare, *column)].trim_end();
            let bound = before.ends_with(':') || before.ends_with('?');
            let named = bound
                && !content.is_empty()
                && content != "transparent"
                && !content.starts_with('#')
                && content.chars().all(|c| c.is_ascii_alphabetic());
            if named {
                hits.push(Hit {
                    column: *column,
                    rule: Rule::Colour,
                    what: format!("{name}: \"{content}\""),
                });
            }
        }
    }
    let Some(rule) = rule else {
        return hits;
    };
    for number in source::numbers(value) {
        if number.value == 0.0 || source::is_halving(value, number) || within_colour_call(value, number.column) {
            continue;
        }
        let literal: String = value
            .chars()
            .skip(number.column)
            .take(number.end - number.column)
            .collect();
        hits.push(Hit {
            column: value_at + number.column,
            rule,
            what: format!("{name}: {literal}"),
        });
    }
    hits
}

/// Whether column `at` of `value` is inside a colour call, which the colour rule reports already.
fn within_colour_call(value: &str, at: usize) -> bool {
    let before = &value[..source::byte_index(value, at)];
    ["Qt.rgba(", "Qt.rgb(", "Qt.hsla(", "Qt.hsva(", "Qt.hsl("]
        .iter()
        .filter_map(|call| before.rfind(call))
        .any(|start| !before[start..].contains(')'))
}
