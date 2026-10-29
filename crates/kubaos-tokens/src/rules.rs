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

use crate::contrast;
use crate::contrast::Mode;
use crate::model::Family;
use crate::model::Need;
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
    /// A type style's icon that isn't one of the icon sizes.
    IconNotASize { token: String, icon: u16 },
    /// A type style's icon taller than its line, so it would make the line taller.
    IconTallerThanLine { token: String, icon: u16, line: u16 },
    /// The container's widths and paddings don't pair up, or its first width isn't 0.
    ContainerSteps,
    /// A container padding that isn't a step of the spacing scale.
    OffScalePadding { token: String, value: u16 },
    /// An aspect ratio with a side of 0.
    EmptyAspect { token: String },
    /// A contrast pair naming a colour role or an opacity that doesn't exist.
    UnknownInPair { token: String, name: String },
    /// A token naming a type style, an opacity or a colour role that doesn't exist.
    UnknownName { token: String, name: String },
    /// The terminals' type style isn't in the monospace family.
    TerminalNotMono { style: String },
    /// A brightness in the neon's strike outside 0..=1.
    StrikeOutOfRange { index: usize, value: f32 },
    /// A stroke of the mark whose path isn't absolute `M`, `L`, `H` and `Q` commands, each with
    /// its numbers, starting with `M`.
    BadMarkPath { index: usize },
    /// A size of the mark that isn't positive and finite, or a fraction outside 0..=1.
    BadMarkSize { token: String },
    /// GRUB's scale isn't one of the scales every length is whole device pixels at.
    GrubScaleNotAScale { scale: f32 },
    /// The frame layer's blur cut-off at or under what its scrim and shadow cover, so Hyprland
    /// blurs the desktop beside the glass too, or at or over the glass, so it blurs nothing.
    FrameBlurOffGlass { cutoff: f32, shade: f32, glass: f32 },
    /// A contrast pair under the minimum its foreground needs, in one of the fallback palettes.
    LowContrast {
        token: String,
        mode: &'static str,
        ratio: f32,
        minimum: f32,
    },
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
            Violation::IconNotASize { token, icon } => {
                write!(f, "{token}'s icon {icon} isn't in icon.sizes")
            }
            Violation::IconTallerThanLine { token, icon, line } => {
                write!(f, "{token}'s icon {icon} is taller than its line {line}")
            }
            Violation::ContainerSteps => write!(
                f,
                "container.from and container.padding need one padding per width, from 0 up"
            ),
            Violation::OffScalePadding { token, value } => {
                write!(f, "{token} is {value}, which isn't on the spacing scale")
            }
            Violation::EmptyAspect { token } => write!(f, "{token} has a side of 0"),
            Violation::UnknownInPair { token, name } => {
                write!(f, "{token} names {name}, which isn't a colour role or an opacity")
            }
            Violation::UnknownName { token, name } => write!(f, "{token} names {name}, which doesn't exist"),
            Violation::TerminalNotMono { style } => {
                write!(f, "terminal.type is {style}, which isn't set in the mono family")
            }
            Violation::StrikeOutOfRange { index, value } => {
                write!(f, "neon.strike[{index}] is {value}, outside 0..=1")
            }
            Violation::BadMarkPath { index } => write!(
                f,
                "neon.mark.strokes[{index}] isn't absolute M, L, H and Q commands, starting with M"
            ),
            Violation::BadMarkSize { token } => {
                write!(f, "{token} isn't a positive size, or a fraction in 0..=1")
            }
            Violation::GrubScaleNotAScale { scale } => {
                write!(f, "boot.grub_scale is {scale}, which isn't in rules.scales")
            }
            Violation::FrameBlurOffGlass { cutoff, shade, glass } => write!(
                f,
                "opacity.frame_blur is {cutoff}, which isn't between the frame's scrim under its shadow \
                 ({shade:.2}) and its glass ({glass})"
            ),
            Violation::LowContrast {
                token,
                mode,
                ratio,
                minimum,
            } => write!(f, "{token} is {ratio:.2}:1 in {mode}, under its {minimum}:1"),
        }
    }
}

/// Every rule `tokens` breaks, in the file's order of tables. Empty when all hold.
pub fn check(tokens: &Tokens) -> Vec<Violation> {
    let mut found = Vec::new();
    check_rules(tokens, &mut found);
    check_lengths(tokens, &mut found);
    check_container(tokens, &mut found);
    check_types(tokens, &mut found);
    check_motion(tokens, &mut found);
    check_colors(tokens, &mut found);
    check_contrast(tokens, &mut found);
    check_terminal(tokens, &mut found);
    check_symbols(tokens, &mut found);
    check_neon(tokens, &mut found);
    check_frame_blur(tokens, &mut found);
    if !tokens.rules.scales.contains(&tokens.boot.grub_scale) {
        found.push(Violation::GrubScaleNotAScale {
            scale: tokens.boot.grub_scale,
        });
    }
    found
}

fn check_terminal(tokens: &Tokens, found: &mut Vec<Violation>) {
    let terminal = &tokens.terminal;
    match tokens.types.get(&terminal.style) {
        None => found.push(Violation::UnknownName {
            token: "terminal.type".into(),
            name: terminal.style.clone(),
        }),
        Some(style) if style.family != Family::Mono => found.push(Violation::TerminalNotMono {
            style: terminal.style.clone(),
        }),
        Some(_) => {}
    }
    for (name, value) in [("padding_x", terminal.padding_x), ("padding_y", terminal.padding_y)] {
        if !tokens.space.values.contains(&value) {
            found.push(Violation::OffScalePadding {
                token: format!("terminal.{name}"),
                value,
            });
        }
    }
    if !tokens.opacity.contains_key(&terminal.opacity) {
        found.push(Violation::UnknownName {
            token: "terminal.opacity".into(),
            name: terminal.opacity.clone(),
        });
    }
    let mut known = true;
    for (slot, role) in terminal.roles.slots() {
        if !tokens.color.source.contains_key(role) {
            known = false;
            found.push(Violation::UnknownName {
                token: format!("terminal.roles.{slot}"),
                name: role.into(),
            });
        }
    }
    if known {
        check_dim(tokens, found);
    }
}

/// ANSI 8, `white` dimmed toward `background`, keeps body text's contrast on it in both palettes.
fn check_dim(tokens: &Tokens, found: &mut Vec<Violation>) {
    let roles = &tokens.terminal.roles;
    for mode in Mode::ALL {
        let (palette, toward) = match mode {
            Mode::Dark => (&tokens.color.dark, -1.0),
            Mode::Light => (&tokens.color.light, 1.0),
        };
        let (Some(&white), Some(&background)) = (palette.get(&roles.white), palette.get(&roles.background)) else {
            continue;
        };
        let dim = white.lighten(toward * f32::from(tokens.terminal.dim));
        let ratio = contrast::ratio(contrast::channels(dim), contrast::channels(background));
        if ratio < Need::Text.minimum() {
            found.push(Violation::LowContrast {
                token: format!("terminal.dim {} on {}", roles.white, roles.background),
                mode: mode.name(),
                ratio,
                minimum: Need::Text.minimum(),
            });
        }
    }
}

fn check_container(tokens: &Tokens, found: &mut Vec<Violation>) {
    let container = &tokens.container;
    if container.from.len() != container.padding.len() || container.from.first() != Some(&0) {
        found.push(Violation::ContainerSteps);
    }
    if !container.from.windows(2).all(|pair| pair[0] < pair[1]) {
        found.push(Violation::NotAscending {
            token: "container.from".into(),
        });
    }
    for &value in &container.padding {
        if !tokens.space.values.contains(&value) {
            found.push(Violation::OffScalePadding {
                token: format!("container.padding {value}"),
                value,
            });
        }
    }
    for (name, sides) in &tokens.aspect {
        if sides.contains(&0) {
            found.push(Violation::EmptyAspect {
                token: format!("aspect.{name}"),
            });
        }
    }
}

fn check_contrast(tokens: &Tokens, found: &mut Vec<Violation>) {
    let token = |pair: &crate::model::ContrastPair| {
        let at = pair
            .opacity
            .as_ref()
            .map_or_else(String::new, |opacity| format!(" at {opacity}"));
        format!("contrast {}{at} on {}", pair.fg, pair.bg)
    };
    for pair in &tokens.contrast.pairs {
        let roles = [&pair.fg, &pair.bg]
            .into_iter()
            .filter(|role| !tokens.color.source.contains_key(*role));
        let opacity = pair.opacity.iter().filter(|name| !tokens.opacity.contains_key(*name));
        for name in roles.chain(opacity) {
            found.push(Violation::UnknownInPair {
                token: token(pair),
                name: name.clone(),
            });
        }
    }
    for name in &tokens.contrast.decorative {
        if !tokens.color.source.contains_key(name) {
            found.push(Violation::UnknownInPair {
                token: "contrast.decorative".into(),
                name: name.clone(),
            });
        }
    }
    for reading in crate::contrast::readings(tokens) {
        if let Some(ratio) = reading.ratio
            && !reading.holds()
        {
            found.push(Violation::LowContrast {
                token: token(reading.pair),
                mode: reading.mode.name(),
                ratio,
                minimum: reading.pair.need.minimum(),
            });
        }
    }
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
        if (tokens.focus.tube * scale).round() < 1.0 {
            found.push(Violation::StrokeUnderDevicePixel {
                token: "focus.tube".into(),
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
    on_grid("focus.near".into(), tokens.focus.near);
    on_grid("focus.far".into(), tokens.focus.far);
    let radius = &tokens.radius;
    for (role, value) in [
        ("small", radius.small),
        ("control", radius.control),
        ("surface", radius.surface),
    ] {
        on_grid(format!("radius.{role}"), value);
    }
    for (table, values) in [
        ("space", &tokens.space.values),
        ("icon", &tokens.icon.sizes),
        ("container.from", &tokens.container.from),
    ] {
        for &value in values {
            on_grid(format!("{table} {value}"), value);
        }
    }
    for (name, &value) in &tokens.size {
        let token = format!("size.{name}");
        if !tokens.rules.off_grid.contains(&token) {
            on_grid(token, value);
        }
    }
    for (name, shadow) in &tokens.shadow {
        on_grid(format!("shadow.{name}.y"), shadow.y);
        on_grid(format!("shadow.{name}.blur"), shadow.blur);
    }
    for (name, style) in &tokens.types {
        on_grid(format!("type.{name}.line"), style.line);
    }

    for listed in &tokens.rules.off_grid {
        let known = listed
            .strip_prefix("size.")
            .is_some_and(|name| tokens.size.contains_key(name));
        if !known {
            found.push(Violation::UnknownName {
                token: "rules.off_grid".into(),
                name: listed.clone(),
            });
        }
    }

    let roles = [radius.small, radius.control, radius.surface];
    for (table, values) in [
        ("space.values", &tokens.space.values),
        ("radius.small, control, surface", &roles.to_vec()),
        ("icon.sizes", &tokens.icon.sizes),
        ("rules.font_sizes", &tokens.rules.font_sizes),
    ] {
        if !values.windows(2).all(|pair| pair[0] < pair[1]) {
            found.push(Violation::NotAscending { token: table.into() });
        }
    }
    for (name, shadow) in &tokens.shadow {
        check_opacity(format!("shadow.{name}.opacity"), shadow.opacity, found);
        if let Some(light) = shadow.light_opacity {
            check_opacity(format!("shadow.{name}.light_opacity"), light, found);
        }
    }
    for (name, &value) in &tokens.opacity {
        check_opacity(format!("opacity.{name}"), value, found);
    }
    check_opacity("focus.near_opacity".into(), tokens.focus.near_opacity, found);
    check_opacity("focus.far_opacity".into(), tokens.focus.far_opacity, found);
    if tokens.focus.near >= tokens.focus.far {
        found.push(Violation::NotAscending {
            token: "focus.near, focus.far".into(),
        });
    }
}

/// Hyprland blurs behind the frame's layer where it is more opaque than `opacity.frame_blur`. The
/// layer draws its glass at `frame`, and outside it the darker scrim with the panels' shadow over
/// it, at `frame` too; the cut-off lies between the two, so only the glass is blurred.
fn check_frame_blur(tokens: &Tokens, found: &mut Vec<Violation>) {
    let mut opacity = |name: &str| {
        let value = tokens.opacity.get(name).copied();
        if value.is_none() {
            found.push(Violation::UnknownName {
                token: "opacity.frame_blur".into(),
                name: format!("opacity.{name}"),
            });
        }
        value
    };
    let (Some(cutoff), Some(glass), Some(scrim), Some(panel_scrim)) = (
        opacity("frame_blur"),
        opacity("frame"),
        opacity("scrim"),
        opacity("panel_scrim"),
    ) else {
        return;
    };
    let Some(shadow) = tokens.shadow.get("panel") else {
        found.push(Violation::UnknownName {
            token: "opacity.frame_blur".into(),
            name: "shadow.panel".into(),
        });
        return;
    };
    let shade = 1.0 - (1.0 - scrim.max(panel_scrim)) * (1.0 - shadow.opacity * glass);
    if cutoff <= shade || cutoff >= glass {
        found.push(Violation::FrameBlurOffGlass { cutoff, shade, glass });
    }
}

fn check_opacity(token: String, value: f32, found: &mut Vec<Violation>) {
    if !(0.0..=1.0).contains(&value) {
        found.push(Violation::OpacityOutOfRange { token, value });
    }
}

fn check_types(tokens: &Tokens, found: &mut Vec<Violation>) {
    if !tokens.icon.sizes.contains(&tokens.icon.two_line) {
        found.push(Violation::IconNotASize {
            token: "icon.two_line".into(),
            icon: tokens.icon.two_line,
        });
    }
    for (name, style) in &tokens.types {
        let token = format!("type.{name}");
        if !tokens.icon.sizes.contains(&style.icon) {
            found.push(Violation::IconNotASize {
                token: token.clone(),
                icon: style.icon,
            });
        }
        if style.icon > style.line {
            found.push(Violation::IconTallerThanLine {
                token: token.clone(),
                icon: style.icon,
                line: style.line,
            });
        }
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

fn check_neon(tokens: &Tokens, found: &mut Vec<Violation>) {
    for (index, &value) in tokens.neon.strike.iter().enumerate() {
        if !(0.0..=1.0).contains(&value) {
            found.push(Violation::StrikeOutOfRange { index, value });
        }
    }
    let mark = &tokens.neon.mark;
    let sizes = [("units", mark.units), ("margin", mark.margin), ("width", mark.width)];
    let fractions = [
        ("cold_width", mark.cold_width),
        ("core", mark.core),
        ("cold_core", mark.cold_core),
    ];
    let bad_sizes = sizes.iter().filter(|(_, v)| !(v.is_finite() && *v > 0.0));
    let bad_fractions = fractions.iter().filter(|(_, v)| !(0.0..=1.0).contains(v));
    for (name, _) in bad_sizes.chain(bad_fractions) {
        found.push(Violation::BadMarkSize {
            token: format!("neon.mark.{name}"),
        });
    }
    for (index, stroke) in mark.strokes.iter().enumerate() {
        if !is_mark_path(&stroke.path) {
            found.push(Violation::BadMarkPath { index });
        }
    }
}

/// Whether `path` is absolute `M`, `L`, `H` and `Q` commands, each letter starting a word and
/// followed by the numbers it takes, the first an `M`: what every reader of the mark parses.
fn is_mark_path(path: &str) -> bool {
    let mut words = path.split_whitespace();
    let mut first = true;
    while let Some(word) = words.next() {
        let (command, number) = word.split_at(word.chars().next().map_or(0, char::len_utf8));
        let takes = match command {
            "M" | "L" => 2,
            "H" => 1,
            "Q" => 4,
            _ => return false,
        };
        if first && command != "M" {
            return false;
        }
        first = false;
        let mut numbers = vec![number];
        while numbers.len() < takes {
            match words.next() {
                Some(next) => numbers.push(next),
                None => return false,
            }
        }
        if !numbers.iter().all(|n| n.parse::<f32>().is_ok_and(f32::is_finite)) {
            return false;
        }
    }
    !first
}

/// Every name that becomes an identifier in an output, as `table.name`.
fn names(tokens: &Tokens) -> Vec<String> {
    let tables: [(&str, Vec<&String>); 9] = [
        ("size", tokens.size.keys().collect()),
        ("aspect", tokens.aspect.keys().collect()),
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
