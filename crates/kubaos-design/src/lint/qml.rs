//! The lint's rules for QML.
//!
//! - **Colour:** a hex colour string anywhere, `Qt.rgba`, `Qt.rgb`, `Qt.hsla`, `Qt.hsva`, and a
//!   named colour (`"white"`) bound to a colour property. `"transparent"` is the absence of one.
//! - **Space, size, radius, duration, font:** a number other than 0 in the binding of a property
//!   that holds one ([`category`]), or in a custom `int` or `real` property named for one
//!   (`iconSize`, `cardRadius`). Halving (`/ 2`) is a derivation, not a literal.
//! - **Font:** a family named in a string, and a weight named in place (`Font.Medium`).
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
    lines.iter().map(|line| line_hits(line, controls)).collect()
}

fn line_hits(line: &Line, controls: bool) -> Vec<Hit> {
    let mut hits = Vec::new();
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
