//! The QML copies of the tokens, in `KubaOS.Components`: `Tokens.qml`, a singleton holding every
//! value by its name in `design/tokens.toml`, and `Symbols.qml`, the symbols' drawings.
//!
//! Names stay as the tokens spell them (`Tokens.size.control_md`, `Tokens.color.on_primary`): QML
//! reads a property named `onPrimary` as a signal handler, and one spelling in every toolkit is
//! easier to find than two.
//!
//! # Owns
//!
//! - The two files' text: the values, the fallback palettes, the live colour roles over them, and
//!   each symbol's SVG.
//!
//! # Not here
//!
//! - Drawing with them: the components of `KubaOS.Components`, and its `Look` singleton for alpha,
//!   nesting and device pixels.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use crate::model::Family;
use crate::model::Rgb;
use crate::model::Tokens;
use crate::model::Weight;
use crate::rust::wrap;
use crate::target::Target;

/// Where the symbols' drawings are, relative to the repo's root.
const SYMBOLS: &str = "crates/kubaos-ui/assets/symbols";

/// A pill's radius: larger than any shape, which a `Rectangle` clamps to half its shorter side.
const PILL: u16 = 4096;

/// `qml/KubaOS/Components/Tokens.qml`.
#[derive(Debug)]
pub struct QmlTokens;

/// `qml/KubaOS/Components/Symbols.qml`.
#[derive(Debug)]
pub struct QmlSymbols;

impl Target for QmlTokens {
    fn path(&self) -> &'static str {
        "qml/KubaOS/Components/Tokens.qml"
    }

    fn render(&self, tokens: &Tokens, _root: &Path) -> io::Result<String> {
        let mut out = Writer::default();
        header(&mut out);
        out.line("// The design tokens, every one by its name in `design/tokens.toml`: lengths, type styles,");
        out.line("// durations, curves, opacities, shadows and colour roles. The colour roles follow `mode`, and");
        out.line("// matugen's colours from the wallpaper once the host sets `matugen`.");
        out.line("QtObject {");
        out.line("    id: tokens");
        out.blank();
        out.line("    // `dark` or `light`: the fallback palette the roles start from.");
        out.line("    property string mode: \"dark\"");
        out.line("    // matugen's colours by matugen's names, as `colors.json` holds them; null for the fallback");
        out.line("    // alone.");
        out.line("    property var matugen: null");
        rules(&mut out, tokens);
        lengths(&mut out, tokens);
        types(&mut out, tokens);
        motion(&mut out, tokens);
        surfaces(&mut out, tokens);
        colors(&mut out, tokens);
        out.line("}");
        Ok(out.text)
    }
}

impl Target for QmlSymbols {
    fn path(&self) -> &'static str {
        "qml/KubaOS/Components/Symbols.qml"
    }

    fn render(&self, tokens: &Tokens, root: &Path) -> io::Result<String> {
        let mut out = Writer::default();
        header(&mut out);
        out.line("// KubaOS Symbols' drawings, by the Material Symbols name each replaces: an SVG on a 24 unit");
        out.line("// grid whose `stroke-width=\"2\"` and `currentColor` `Icon` replaces with the stroke for its size");
        out.line("// and scale and its colour.");
        out.line("QtObject {");
        out.line("    readonly property var svg: ({");
        let last = tokens.symbols.len().saturating_sub(1);
        for (index, name) in tokens.symbols.keys().enumerate() {
            let path = root.join(SYMBOLS).join(format!("{name}.svg"));
            let drawing = minify(&fs::read_to_string(&path)?);
            recolourable(&drawing)
                .map_err(|why| io::Error::new(io::ErrorKind::InvalidData, format!("{}: {why}", path.display())))?;
            let comma = if index == last { "" } else { "," };
            out.line(format!("            {name}: {drawing:?}{comma}"));
        }
        out.line("        })");
        out.line("}");
        Ok(out.text)
    }
}

/// The text `Icon` replaces in every drawing: the stroke for its size and scale, and its colour.
const STROKE: &str = "stroke-width=\"2\"";
const COLOUR: &str = "currentColor";

/// Whether `Icon` can restroke and recolour `svg`: its `String.replace` changes the first match
/// only, so each of `STROKE` and `COLOUR` must be there exactly once.
fn recolourable(svg: &str) -> Result<(), String> {
    for wanted in [STROKE, COLOUR] {
        let found = svg.matches(wanted).count();
        if found != 1 {
            return Err(format!("has {found} `{wanted}`, and `Icon` replaces exactly one"));
        }
    }
    Ok(())
}

/// `svg` on one line: no comments, no Tabler class names, one space wherever there was any.
fn minify(svg: &str) -> String {
    let mut text = svg.to_owned();
    while let Some(start) = text.find("<!--") {
        let end = text[start..].find("-->").map_or(text.len(), |at| start + at + 3);
        text.replace_range(start..end, "");
    }
    while let Some(start) = text.find(" class=\"") {
        let value = start + " class=\"".len();
        let end = text[value..].find('"').map_or(text.len(), |at| value + at + 1);
        text.replace_range(start..end, "");
    }
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace("> <", "><")
        .replace(" >", ">")
}

/// A file's text, built line by line.
#[derive(Default)]
struct Writer {
    text: String,
}

impl Writer {
    fn line(&mut self, line: impl AsRef<str>) {
        self.text.push_str(line.as_ref());
        self.text.push('\n');
    }

    fn blank(&mut self) {
        self.text.push('\n');
    }

    fn comment(&mut self, indent: &str, text: &str) {
        for line in wrap(text, 100 - indent.len() - 3) {
            self.line(format!("{indent}// {line}"));
        }
    }

    /// `name: { … }`, a group of read-only values.
    fn group(&mut self, indent: &str, comment: Option<&str>, name: &str, body: impl FnOnce(&mut Writer)) {
        self.blank();
        if let Some(comment) = comment {
            self.comment(indent, comment);
        }
        self.line(format!("{indent}readonly property QtObject {name}: QtObject {{"));
        body(self);
        self.line(format!("{indent}}}"));
    }
}

fn header(out: &mut Writer) {
    out.line("// @generated by `kubaos-tokens` from `design/tokens.toml`. Change the tokens there and run");
    out.line("// `cargo run -p kubaos-tokens -- write`; a test fails while this file differs from them.");
    out.blank();
    out.line("pragma Singleton");
    out.blank();
    out.line("import QtQuick");
    out.blank();
}

/// A number as QML writes it: `1`, `0.25`.
fn number(value: f32) -> String {
    let text = format!("{value:?}");
    text.strip_suffix(".0").map_or(text.clone(), str::to_owned)
}

fn rules(out: &mut Writer, tokens: &Tokens) {
    let rules = &tokens.rules;
    out.group("    ", None, "rules", |out| {
        out.comment(
            "        ",
            "Every spacing, size, radius, icon size, line height and shadow is a multiple of this, in logical pixels.",
        );
        out.line(format!("        readonly property int grid: {}", rules.grid));
        out.line("        // The only font sizes, in logical pixels.");
        let sizes: Vec<String> = rules.font_sizes.iter().map(u16::to_string).collect();
        out.line(format!(
            "        readonly property list<int> font_sizes: [{}]",
            sizes.join(", ")
        ));
        out.line("        // The display scales every length lands on whole device pixels at.");
        let scales: Vec<String> = rules.scales.iter().map(|&s| number(s)).collect();
        out.line(format!(
            "        readonly property list<real> scales: [{}]",
            scales.join(", ")
        ));
    });
    let stroke = &tokens.stroke;
    out.group("    ", None, "stroke", |out| {
        out.line("        // A hairline's width in device pixels, at every scale.");
        out.line(format!(
            "        readonly property int hairline_device_px: {}",
            stroke.hairline_device_px
        ));
        out.comment(
            "        ",
            "The focus ring's, the spinner's and a partial checkbox's stroke in logical pixels, drawn rounded half up to whole device pixels.",
        );
        out.line(format!("        readonly property int thin: {}", stroke.thin));
        out.comment(
            "        ",
            "A symbol's stroke on its 24 unit grid, drawn scaled to the symbol's size, then rounded half up to whole device pixels.",
        );
        out.line(format!("        readonly property int symbol: {}", stroke.symbol));
    });
    out.group("    ", None, "focus", |out| {
        out.line("        // How far the focus ring's outer edge stands outside its control, in logical pixels.");
        out.line(format!("        readonly property int offset: {}", tokens.focus.offset));
    });
}

fn lengths(out: &mut Writer, tokens: &Tokens) {
    out.group(
        "    ",
        Some("The spacing scale: gaps, paddings and insets."),
        "space",
        |out| {
            for v in &tokens.space.values {
                out.line(format!("        readonly property int s{v}: {v}"));
            }
        },
    );
    let r = &tokens.radius;
    out.group(
        "    ",
        Some("Corner radii by role. A radius between them comes from `Look.nested`, never a number."),
        "radius",
        |out| {
            out.line("        // Marks inside controls: the checkbox, a hint, focus details, inline tags.");
            out.line(format!("        readonly property int small: {}", r.small));
            out.line("        // Everything clicked or typed in: buttons, fields, list rows, menu items, chips.");
            out.line(format!("        readonly property int control: {}", r.control));
            out.line("        // Everything that holds content: cards, dialogs, popups, panels, list wells.");
            out.line(format!("        readonly property int surface: {}", r.surface));
            if r.full {
                out.comment(
                    "        ",
                    "Half the shape's shorter side, a pill or a circle: larger than any shape, which a Rectangle clamps.",
                );
                out.line(format!("        readonly property int full: {PILL}"));
            }
        },
    );
    out.group(
        "    ",
        Some("Icon sizes: the symbol's square box, its 24 unit grid scaled to it."),
        "icon",
        |out| {
            for v in &tokens.icon.sizes {
                out.line(format!("        readonly property int s{v}: {v}"));
            }
        },
    );
    out.group("    ", Some("Fixed dimensions of components."), "size", |out| {
        for (name, value) in &tokens.size {
            out.line(format!("        readonly property int {name}: {value}"));
        }
    });
}

fn weight_value(weight: Weight) -> u16 {
    match weight {
        Weight::Light => 300,
        Weight::Regular => 400,
        Weight::Medium => 500,
        Weight::Semibold => 600,
    }
}

fn weight_name(weight: Weight) -> &'static str {
    match weight {
        Weight::Light => "light",
        Weight::Regular => "regular",
        Weight::Medium => "medium",
        Weight::Semibold => "semibold",
    }
}

fn family_name(family: Family) -> &'static str {
    match family {
        Family::Sans => "sans",
        Family::Mono => "mono",
        Family::Jp => "jp",
    }
}

fn types(out: &mut Writer, tokens: &Tokens) {
    let families = &tokens.font.family;
    out.group("    ", Some("Type families, by the role they play."), "font", |out| {
        for family in Family::ALL {
            out.line(format!(
                "        readonly property string {}: {:?}",
                family_name(family),
                families.name(family)
            ));
        }
    });
    out.group(
        "    ",
        Some("Font weights on CSS's 100 to 900 scale, which `font.weight` takes."),
        "weight",
        |out| {
            for weight in Weight::ALL {
                out.line(format!(
                    "        readonly property int {}: {}",
                    weight_name(weight),
                    weight_value(weight)
                ));
            }
        },
    );
    // Largest first, so the file reads like the type scale.
    let mut styles: Vec<_> = tokens.types.iter().collect();
    styles.sort_by_key(|(name, style)| (std::cmp::Reverse(style.size), std::cmp::Reverse(style.line), *name));
    out.group(
        "    ",
        Some("Type styles: what a piece of text is drawn with. Pick one by what the text is."),
        "type",
        |out| {
            for (name, style) in styles {
                out.blank();
                out.comment("        ", &style.purpose);
                out.line(format!("        readonly property QtObject {name}: QtObject {{"));
                out.line(format!("            readonly property string name: {name:?}"));
                out.line(format!("            readonly property int size: {}", style.size));
                out.line(format!("            readonly property int line: {}", style.line));
                out.line(format!(
                    "            readonly property int weight: {}",
                    weight_value(style.weight)
                ));
                out.line(format!(
                    "            readonly property string family: {:?}",
                    families.name(style.family)
                ));
                out.line("        }");
            }
        },
    );
}

fn motion(out: &mut Writer, tokens: &Tokens) {
    let mut durations: Vec<_> = tokens.duration.iter().collect();
    durations.sort_by_key(|(name, duration)| (duration.ms, *name));
    out.group(
        "    ",
        Some("Durations in milliseconds, at speed 1. Each animation takes the one for what it is."),
        "duration",
        |out| {
            for (name, duration) in durations {
                out.comment("        ", &duration.purpose);
                out.line(format!("        readonly property int {name}: {}", duration.ms));
            }
        },
    );
    out.group(
        "    ",
        Some("Easing curves as `Easing.BezierSpline` takes them: the CSS cubic bézier, then its end point."),
        "easing",
        |out| {
            for (name, easing) in &tokens.easing {
                let [x1, y1, x2, y2] = easing.curve;
                out.comment("        ", &easing.purpose);
                out.line(format!(
                    "        readonly property list<real> {name}: [{}, {}, {}, {}, 1, 1]",
                    number(x1),
                    number(y1),
                    number(x2),
                    number(y2)
                ));
            }
        },
    );
}

fn surfaces(out: &mut Writer, tokens: &Tokens) {
    out.group(
        "    ",
        Some("Opacities laid over a colour role, 0 to 1."),
        "opacity",
        |out| {
            for (name, value) in &tokens.opacity {
                out.line(format!("        readonly property real {name}: {}", number(*value)));
            }
        },
    );
    out.group(
        "    ",
        Some("Shadows: how far down and how blurred, in logical pixels, and the `shadow` role's opacity."),
        "shadow",
        |out| {
            for (name, shadow) in &tokens.shadow {
                out.line(format!("        readonly property QtObject {name}: QtObject {{"));
                out.line(format!("            readonly property int y: {}", shadow.y));
                out.line(format!("            readonly property int blur: {}", shadow.blur));
                out.line(format!(
                    "            readonly property real opacity: {}",
                    number(shadow.opacity)
                ));
                out.line("        }");
            }
        },
    );
}

fn colors(out: &mut Writer, tokens: &Tokens) {
    let colors = &tokens.color;
    out.group(
        "    ",
        Some(
            "The colour roles: matugen's colour from the wallpaper where the role has one and the host has set `matugen`, otherwise the fallback of `mode`.",
        ),
        "color",
        |out| {
            for (role, source) in &colors.source {
                if source.is_empty() {
                    out.line("        // Ours: the same whatever the wallpaper.");
                } else {
                    out.line(format!("        // matugen's `{source}`."));
                }
                out.line(format!("        readonly property color {role}: tokens.role({role:?})"));
            }
        },
    );
    out.blank();
    out.line("    // The matugen colour each role is read from; empty for a role of our own.");
    out.line("    readonly property var source: ({");
    let last = colors.source.len().saturating_sub(1);
    for (index, (role, source)) in colors.source.iter().enumerate() {
        let comma = if index == last { "" } else { "," };
        out.line(format!("            {role}: {source:?}{comma}"));
    }
    out.line("        })");
    out.blank();
    out.line("    // The fallback palettes: matugen's output for the default wallpaper, in each mode.");
    out.line("    readonly property var palette: ({");
    palette(out, "dark", &colors.dark, ",");
    palette(out, "light", &colors.light, "");
    out.line("        })");
    out.blank();
    out.line("    // `name`'s colour: matugen's when it has one for the role, else the fallback of `mode`.");
    out.line("    function role(name: string): color {");
    out.line("        const from = tokens.source[name];");
    out.line("        const live = tokens.matugen && from ? tokens.matugen[from] : undefined;");
    out.line("        return live ?? tokens.palette[tokens.mode === \"light\" ? \"light\" : \"dark\"][name];");
    out.line("    }");
}

fn palette(out: &mut Writer, name: &str, roles: &BTreeMap<String, Rgb>, comma: &str) {
    out.line(format!("            {name}: {{"));
    let last = roles.len().saturating_sub(1);
    for (index, (role, rgb)) in roles.iter().enumerate() {
        let comma = if index == last { "" } else { "," };
        out.line(format!("                {role}: \"{rgb:?}\"{comma}"));
    }
    out.line(format!("            }}{comma}"));
}

#[cfg(test)]
mod tests;
