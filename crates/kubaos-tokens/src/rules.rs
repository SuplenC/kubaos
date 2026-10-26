//! The rules of the design language, checked against every token: lengths on the 4 px grid, font
//! sizes from the list, colour roles complete in both modes.
//!
//! # Owns
//!
//! - What a broken rule is ([`Violation`]) and finding every one in a set of tokens ([`check`]).
//!
//! # Not here
//!
//! - The rules' values (the grid, the font sizes, the scales): `design/tokens.toml`, `[rules]`.

use std::collections::BTreeSet;
use std::fmt;

use crate::model::Tokens;

/// One broken rule, naming the token that breaks it.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Violation {
    /// A length that isn't a multiple of the grid, so it misses whole device pixels at some scale.
    OffGrid { token: String, value: u16, grid: u16 },
    /// A type style whose size isn't on the list.
    OffListFontSize { token: String, size: u16 },
    /// A type style whose line is shorter than its letters.
    LineShorterThanSize { token: String, size: u16, line: u16 },
    /// The grid itself misses whole device pixels at one of the scales.
    GridOffDevicePixels { grid: u16, scale: f32 },
    /// A thin stroke that would be under one device pixel at one of the scales.
    StrokeUnderDevicePixel { token: String, scale: f32 },
    /// A scale whose steps aren't strictly increasing.
    NotAscending { token: String },
    /// A name that can't become an identifier: lowercase words joined by `_`.
    BadName { token: String },
    /// An easing curve whose x control points leave 0..=1, so time would run backwards.
    CurveOutOfRange { token: String },
    /// An opacity outside 0..=1.
    OpacityOutOfRange { token: String, value: f32 },
    /// A colour role missing from one of `source`, `dark` and `light`.
    RoleMissing { role: String, table: &'static str },
    /// A symbol that names both a Tabler icon and a drawing of ours, or neither.
    SymbolSource { token: String },
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Violation::OffGrid { token, value, grid } => {
                write!(f, "{token} is {value}, which isn't a multiple of {grid}")
            }
            Violation::OffListFontSize { token, size } => {
                write!(f, "{token} has font size {size}, which isn't in rules.font_sizes")
            }
            Violation::LineShorterThanSize { token, size, line } => {
                write!(f, "{token}'s line height {line} is shorter than its size {size}")
            }
            Violation::GridOffDevicePixels { grid, scale } => {
                write!(f, "a grid of {grid} isn't whole device pixels at scale {scale}")
            }
            Violation::StrokeUnderDevicePixel { token, scale } => {
                write!(f, "{token} is under one device pixel at scale {scale}")
            }
            Violation::NotAscending { token } => write!(f, "{token} isn't strictly increasing"),
            Violation::BadName { token } => {
                write!(f, "{token} isn't lowercase words joined by `_`")
            }
            Violation::CurveOutOfRange { token } => {
                write!(f, "{token} has an x control point outside 0..=1")
            }
            Violation::OpacityOutOfRange { token, value } => {
                write!(f, "{token} is {value}, outside 0..=1")
            }
            Violation::RoleMissing { role, table } => {
                write!(f, "colour role {role} is missing from color.{table}")
            }
            Violation::SymbolSource { token } => {
                write!(f, "{token} needs exactly one of `tabler` and `drawn`")
            }
        }
    }
}

/// Every rule `tokens` breaks, in the file's order of tables. Empty when all hold.
pub fn check(tokens: &Tokens) -> Vec<Violation> {
    let mut found = Vec::new();
    check_rules(tokens, &mut found);
    check_lengths(tokens, &mut found);
    check_types(tokens, &mut found);
    check_motion(tokens, &mut found);
    check_colors(tokens, &mut found);
    check_symbols(tokens, &mut found);
    found
}

fn check_rules(tokens: &Tokens, found: &mut Vec<Violation>) {
    let grid = tokens.rules.grid;
    for &scale in &tokens.rules.scales {
        if !is_whole(f32::from(grid) * scale) {
            found.push(Violation::GridOffDevicePixels { grid, scale });
        }
        if (f32::from(tokens.stroke.thin) * scale).round() < 1.0 {
            found.push(Violation::StrokeUnderDevicePixel {
                token: "stroke.thin".into(),
                scale,
            });
        }
    }
}

fn check_lengths(tokens: &Tokens, found: &mut Vec<Violation>) {
    let grid = tokens.rules.grid;
    let mut on_grid = |token: String, value: u16| {
        if !value.is_multiple_of(grid) {
            found.push(Violation::OffGrid { token, value, grid });
        }
    };
    on_grid("focus.offset".into(), tokens.focus.offset);
    for (table, values) in [
        ("space", &tokens.space.values),
        ("radius", &tokens.radius.values),
        ("icon", &tokens.icon.sizes),
    ] {
        for &value in values {
            on_grid(format!("{table} {value}"), value);
        }
    }
    for (name, &value) in &tokens.size {
        on_grid(format!("size.{name}"), value);
    }
    for (name, shadow) in &tokens.shadow {
        on_grid(format!("shadow.{name}.y"), shadow.y);
        on_grid(format!("shadow.{name}.blur"), shadow.blur);
    }
    for (name, style) in &tokens.types {
        on_grid(format!("type.{name}.line"), style.line);
    }

    for (table, values) in [
        ("space.values", &tokens.space.values),
        ("radius.values", &tokens.radius.values),
        ("icon.sizes", &tokens.icon.sizes),
        ("rules.font_sizes", &tokens.rules.font_sizes),
    ] {
        if !values.windows(2).all(|pair| pair[0] < pair[1]) {
            found.push(Violation::NotAscending { token: table.into() });
        }
    }
    for (name, shadow) in &tokens.shadow {
        check_opacity(format!("shadow.{name}.opacity"), shadow.opacity, found);
    }
    for (name, &value) in &tokens.opacity {
        check_opacity(format!("opacity.{name}"), value, found);
    }
}

fn check_opacity(token: String, value: f32, found: &mut Vec<Violation>) {
    if !(0.0..=1.0).contains(&value) {
        found.push(Violation::OpacityOutOfRange { token, value });
    }
}

fn check_types(tokens: &Tokens, found: &mut Vec<Violation>) {
    for (name, style) in &tokens.types {
        let token = format!("type.{name}");
        if !tokens.rules.font_sizes.contains(&style.size) {
            found.push(Violation::OffListFontSize {
                token: token.clone(),
                size: style.size,
            });
        }
        if style.line < style.size {
            found.push(Violation::LineShorterThanSize {
                token,
                size: style.size,
                line: style.line,
            });
        }
    }
}

fn check_motion(tokens: &Tokens, found: &mut Vec<Violation>) {
    for (name, easing) in &tokens.easing {
        let [x1, _, x2, _] = easing.curve;
        if !(0.0..=1.0).contains(&x1) || !(0.0..=1.0).contains(&x2) {
            found.push(Violation::CurveOutOfRange {
                token: format!("easing.{name}"),
            });
        }
    }
}

fn check_colors(tokens: &Tokens, found: &mut Vec<Violation>) {
    let colors = &tokens.color;
    let tables: [(&'static str, BTreeSet<&String>); 3] = [
        ("source", colors.source.keys().collect()),
        ("dark", colors.dark.keys().collect()),
        ("light", colors.light.keys().collect()),
    ];
    let every: BTreeSet<&String> = tables.iter().flat_map(|(_, roles)| roles.iter().copied()).collect();
    for role in every {
        for (table, roles) in &tables {
            if !roles.contains(role) {
                found.push(Violation::RoleMissing {
                    role: role.clone(),
                    table,
                });
            }
        }
    }
    for name in names(tokens) {
        if !is_identifier(&name) {
            found.push(Violation::BadName { token: name });
        }
    }
}

fn check_symbols(tokens: &Tokens, found: &mut Vec<Violation>) {
    for (name, symbol) in &tokens.symbols {
        if symbol.tabler.is_some() == symbol.drawn.is_some() {
            found.push(Violation::SymbolSource {
                token: format!("symbols.{name}"),
            });
        }
    }
}

/// Every name that becomes an identifier in an output, as `table.name`.
fn names(tokens: &Tokens) -> Vec<String> {
    let tables: [(&str, Vec<&String>); 8] = [
        ("size", tokens.size.keys().collect()),
        ("type", tokens.types.keys().collect()),
        ("duration", tokens.duration.keys().collect()),
        ("easing", tokens.easing.keys().collect()),
        ("opacity", tokens.opacity.keys().collect()),
        ("shadow", tokens.shadow.keys().collect()),
        ("color", tokens.color.source.keys().collect()),
        ("symbols", tokens.symbols.keys().collect()),
    ];
    tables
        .into_iter()
        .flat_map(|(table, keys)| keys.into_iter().map(move |key| format!("{table}.{key}")))
        .collect()
}

fn is_identifier(token: &str) -> bool {
    let name = token.rsplit('.').next().unwrap_or(token);
    name.split('_')
        .all(|word| !word.is_empty() && word.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()))
        && name.bytes().next().is_some_and(|b| b.is_ascii_lowercase())
}

fn is_whole(value: f32) -> bool {
    (value - value.round()).abs() < 1e-4
}

#[cfg(test)]
mod tests;
