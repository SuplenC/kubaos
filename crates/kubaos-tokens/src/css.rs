//! The CSS copy of the tokens, for the mockups in `design/mockups/`: every token a custom property,
//! by its name in `design/tokens.toml` (`--size-control_md`, `--color-on_primary`).
//!
//! A length is its logical pixels times `--px`, the CSS pixels a logical pixel takes in the box it
//! is read in. The lengths are declared on `:root` and again on every `.mock`, so a box drawn at
//! another scale sets its own `--scale` and every length inside it follows; so do the durations,
//! as `--motion-*`, under the box's motion settings. The colours and the shadows' opacities follow
//! `data-mode` on the box they are read in: dark by default. What the tokens say in words, and the
//! lists the foundations pages draw from (where each colour role comes from, the contrast pairs),
//! are strings on `:root`.
//!
//! # Owns
//!
//! - The file's text.
//!
//! # Not here
//!
//! - Drawing with them: the mockups' own CSS and TypeScript, under `design/mockups/src/`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io;
use std::path::Path;

use crate::contrast::Mode;
use crate::model::Family;
use crate::model::Need;
use crate::model::Rgb;
use crate::model::Tokens;
use crate::model::Weight;
use crate::qml::number;
use crate::target::BUILT_HEADER;
use crate::target::Kept;
use crate::target::Target;

/// `design/mockups/src/tokens.css`.
#[derive(Debug)]
pub struct CssTokens;

impl Target for CssTokens {
    fn path(&self) -> &'static str {
        "design/mockups/src/tokens.css"
    }

    fn kept(&self) -> Kept {
        Kept::Built
    }

    fn render(&self, tokens: &Tokens, _root: &Path) -> io::Result<String> {
        let mut out = Css::default();
        let _ = writeln!(out.text, "/* {}", BUILT_HEADER[0]);
        let _ = writeln!(out.text, "   {} */", BUILT_HEADER[1]);
        out.blank();
        out.open(":root");
        out.comment("The scale a box is drawn at: device pixels per logical pixel. A `.mock` may set its own.");
        out.var("scale", "1");
        out.comment(
            "The motion settings: the speed dividing every duration, a player's slow motion multiplying it, and",
        );
        out.comment("the longest a duration may be, which reduce motion lowers to `short`.");
        out.var("speed", "1");
        out.var("slow", "1");
        out.var("cap", "3600s");
        out.close();
        out.blank();
        out.open(":root,\n.mock");
        out.var("px", "calc(var(--scale) * 1px)");
        lengths(&mut out, tokens);
        types(&mut out, tokens);
        motion(&mut out, tokens);
        out.close();
        out.blank();
        out.open(":root,\n[data-mode=\"dark\"]");
        mode(&mut out, tokens, &tokens.color.dark, Mode::Dark);
        out.close();
        out.blank();
        out.open("[data-mode=\"light\"]");
        mode(&mut out, tokens, &tokens.color.light, Mode::Light);
        out.close();
        out.blank();
        out.open(":root");
        words(&mut out, tokens);
        out.close();
        Ok(out.text)
    }
}

/// A style sheet's text, built a declaration at a time.
#[derive(Default)]
struct Css {
    text: String,
}

impl Css {
    fn open(&mut self, selector: &str) {
        let _ = writeln!(self.text, "{selector} {{");
    }

    fn close(&mut self) {
        self.text.push_str("}\n");
    }

    fn blank(&mut self) {
        self.text.push('\n');
    }

    fn comment(&mut self, text: &str) {
        let _ = writeln!(self.text, "  /* {text} */");
    }

    fn var(&mut self, name: &str, value: impl AsRef<str>) {
        let _ = writeln!(self.text, "  --{name}: {};", value.as_ref());
    }

    /// `--name`, `value` logical pixels in the box's CSS pixels.
    fn length(&mut self, name: &str, value: impl Into<f32>) {
        self.var(name, format!("calc({} * var(--px))", number(value.into())));
    }
}

fn lengths(out: &mut Css, tokens: &Tokens) {
    out.comment("The grid every length is a multiple of.");
    out.length("grid", tokens.rules.grid);
    out.comment("The spacing scale: gaps, paddings and insets.");
    for &value in &tokens.space.values {
        out.length(&format!("space-s{value}"), value);
    }
    let radius = &tokens.radius;
    out.comment("Corner radii by role; a pill's is longer than any side, which a corner clamps to half.");
    out.length("radius-small", radius.small);
    out.length("radius-control", radius.control);
    out.length("radius-surface", radius.surface);
    if radius.full {
        out.var("radius-full", "9999px");
    }
    out.comment("Icon sizes: the symbol's square box.");
    for &value in &tokens.icon.sizes {
        out.length(&format!("icon-s{value}"), value);
    }
    out.length("icon-two_line", tokens.icon.two_line);
    out.comment("Fixed dimensions of components.");
    for (name, &value) in &tokens.size {
        out.length(&format!("size-{name}"), value);
    }
    out.comment("The container's padding by width: step N applies from `from` logical pixels up.");
    let container = &tokens.container;
    for (step, (&from, &padding)) in container.from.iter().zip(&container.padding).enumerate() {
        out.var(&format!("container-{step}-from"), from.to_string());
        out.length(&format!("container-{step}-padding"), padding);
    }
    for (name, [width, height]) in &tokens.aspect {
        out.var(&format!("aspect-{name}"), format!("{width} / {height}"));
    }
    let stroke = &tokens.stroke;
    out.comment("A hairline is device pixels at every scale; the thin stroke is drawn rounded half up to them.");
    out.var("stroke-hairline_device_px", format!("{}px", stroke.hairline_device_px));
    out.length("stroke-thin", stroke.thin);
    out.comment("A symbol's stroke in its drawing's 24 units.");
    out.var("stroke-symbol", stroke.symbol.to_string());
    let focus = &tokens.focus;
    out.comment("The neon focus: the tube on the control's edge, rounded half up to device pixels, and its glow.");
    out.length("focus-tube", focus.tube);
    out.length("focus-near", focus.near);
    out.var("focus-near_opacity", number(focus.near_opacity));
    out.length("focus-far", focus.far);
    out.var("focus-far_opacity", number(focus.far_opacity));
    out.comment("Content drawn in its tone, lit: a faint near glow only.");
    out.length("focus-content_near", focus.content_near);
    out.var("focus-content_glow", number(focus.content_glow));
    out.length("focus-label_near", focus.label_near);
    out.var("focus-label_glow", number(focus.label_glow));
    out.var("focus-pressed_glow", number(focus.pressed_glow));
    out.length("focus-offset", focus.offset);
    let caret = &tokens.caret;
    out.comment("A field's caret: its height, and its glow's reach and opacity.");
    out.length("caret-height", caret.height);
    out.length("caret-near", caret.near);
    out.var("caret-glow", number(caret.glow));
    for (name, shadow) in &tokens.shadow {
        out.length(&format!("shadow-{name}-y"), shadow.y);
        out.length(&format!("shadow-{name}-blur"), shadow.blur);
    }
    let elevation = &tokens.elevation;
    out.comment("The ladder inside a plane: each step's contact and ambient layer, down and blurred.");
    out.var("elevation-steps", elevation.steps.to_string());
    for step in elevation.raised() {
        for (layer, values) in [("contact", &elevation.contact), ("ambient", &elevation.ambient)] {
            let (y, blur) = values.at(step);
            out.length(&format!("elevation-{step}-{layer}-y"), y);
            out.length(&format!("elevation-{step}-{layer}-blur"), blur);
        }
    }
}

fn weight(weight: Weight) -> u16 {
    match weight {
        Weight::Light => 300,
        Weight::Regular => 400,
        Weight::Medium => 500,
        Weight::Semibold => 600,
    }
}

fn types(out: &mut Css, tokens: &Tokens) {
    let families = &tokens.font.family;
    out.comment("Type families, by the role they play.");
    for (role, family) in [("sans", Family::Sans), ("mono", Family::Mono), ("jp", Family::Jp)] {
        out.var(&format!("font-{role}"), format!("{:?}", families.name(family)));
    }
    out.comment("Type styles: what a piece of text is drawn with, picked by what the text is.");
    for (name, style) in &tokens.types {
        out.length(&format!("type-{name}-size"), style.size);
        out.length(&format!("type-{name}-line"), style.line);
        out.length(&format!("type-{name}-icon"), style.icon);
        out.var(&format!("type-{name}-weight"), weight(style.weight).to_string());
        out.var(
            &format!("type-{name}-family"),
            format!("{:?}", families.name(style.family)),
        );
        let features = if style.tabular { "\"tnum\" 1" } else { "normal" };
        out.var(&format!("type-{name}-features"), features);
    }
}

fn motion(out: &mut Css, tokens: &Tokens) {
    out.comment("Durations at speed 1, and easing curves.");
    for (name, duration) in &tokens.duration {
        out.var(&format!("duration-{name}"), format!("{}ms", duration.ms));
    }
    out.comment("Each duration under the motion settings: capped, then divided by the speed (`Motion::duration`).");
    for name in tokens.duration.keys() {
        out.var(
            &format!("motion-{name}"),
            format!("calc(min(var(--duration-{name}), var(--cap)) / var(--speed) * var(--slow))"),
        );
    }
    for (name, easing) in &tokens.easing {
        let [x1, y1, x2, y2] = easing.curve.map(number);
        out.var(
            &format!("easing-{name}"),
            format!("cubic-bezier({x1}, {y1}, {x2}, {y2})"),
        );
    }
    out.comment("Opacities laid over a colour role.");
    for (name, &value) in &tokens.opacity {
        out.var(&format!("opacity-{name}"), number(value));
    }
    out.comment("A neon tube striking: its brightness, 0 to 1, through even steps over `medium`.");
    for (step, &value) in tokens.neon.strike.iter().enumerate() {
        out.var(&format!("neon-strike-{step}"), number(value));
    }
    out.var("neon-strike-steps", tokens.neon.strike.len().to_string());
}

/// The colour roles of one palette, and the shadows' opacities in that mode.
fn mode(out: &mut Css, tokens: &Tokens, palette: &BTreeMap<String, Rgb>, mode: Mode) {
    out.comment("The colour roles: the fallback palette, matugen's colours for the default wallpaper.");
    for (role, rgb) in palette
        .iter()
        .filter(|(role, _)| !tokens.color.derived.contains_key(*role))
    {
        out.var(&format!("color-{role}"), format!("{rgb:?}"));
    }
    out.comment("The derived roles that are a tone mixed toward white or black, computed from their tone wherever");
    out.comment("it is set, so they follow it.");
    for (role, derived) in &tokens.color.derived {
        let Some((toward, t)) = derived.toward() else { continue };
        let percent = number(t * 100.0);
        out.var(
            &format!("color-{role}"),
            format!(
                "color-mix(in srgb, var(--color-{}), {} {percent}%)",
                derived.tone(),
                toward.name()
            ),
        );
    }
    out.comment("The derived roles CSS can't compute (OKLCH, gamut and contrast fitting), for this palette. A page");
    out.comment("painted in another computes them again from `--derived-dark` or `--derived-light`.");
    let computed = tokens.color.with_derived(palette, mode);
    for (role, derived) in &tokens.color.derived {
        if derived.toward().is_some() {
            continue;
        }
        if let Some(rgb) = computed.get(role) {
            out.var(&format!("color-{role}"), format!("{rgb:?}"));
        }
    }
    out.comment("The `shadow` role's opacity for each shadow.");
    for (name, shadow) in &tokens.shadow {
        let opacity = match (mode, shadow.light_opacity) {
            (Mode::Light, Some(opacity)) => opacity,
            _ => shadow.opacity,
        };
        out.var(&format!("shadow-{name}-opacity"), number(opacity));
    }
    let elevation = &tokens.elevation;
    out.comment("The ladder's layers' opacities of `shadow`, its rim's of `on_surface`, and how many container");
    out.comment("roles above its day one a raised part sits.");
    out.var("elevation-contact-opacity", number(elevation.contact.opacity_in(mode)));
    out.var("elevation-ambient-opacity", number(elevation.ambient.opacity_in(mode)));
    out.var("elevation-rim", number(elevation.rim_in(mode)));
    let lighter = match mode {
        Mode::Dark => elevation.lighter_at_night,
        Mode::Light => 0,
    };
    out.var("elevation-lighter", lighter.to_string());
}

/// Every derived role's recipe in `mode`, as JSON: what the mockups compute a palette's derived
/// roles from (`design/mockups/src/site/derive.ts`).
fn recipes(tokens: &Tokens, mode: Mode) -> String {
    let list = |names: &[String]| {
        names
            .iter()
            .map(|name| format!("{name:?}"))
            .collect::<Vec<_>>()
            .join(",")
    };
    let mut entries = Vec::new();
    for (role, derived) in &tokens.color.derived {
        let recipe = derived.in_mode(mode);
        let mut fields = Vec::new();
        if let Some(mix) = &recipe.mix {
            fields.push(format!("\"mix\":{mix:?}"));
        }
        if let Some(from) = &recipe.from {
            fields.push(format!("\"from\":{from:?}"));
        }
        let numbers = [
            ("white", recipe.white),
            ("black", recipe.black),
            ("lightness", recipe.lightness),
            ("at_lightness", recipe.at_lightness),
            ("lighter", recipe.lighter),
            ("darker", recipe.darker),
            ("chroma", recipe.chroma),
        ];
        for (name, value) in numbers {
            if let Some(value) = value {
                fields.push(format!("\"{name}\":{}", number(value)));
            }
        }
        if let Some(on) = &recipe.text_on {
            fields.push(format!("\"text_on\":[{}]", list(on)));
        }
        fields.push(format!("\"formula\":{}", derived.toward().is_some()));
        entries.push(format!("{role:?}:{{{}}}", fields.join(",")));
    }
    format!("{{{}}}", entries.join(","))
}

/// What each type style, duration and curve is for, where each colour role comes from, and every
/// contrast pair: strings and lists, read by the foundations pages rather than drawn with.
fn words(out: &mut Css, tokens: &Tokens) {
    out.comment("What each type style, duration and curve is for.");
    for (name, style) in &tokens.types {
        out.var(&format!("type-{name}-use"), string(&style.purpose));
    }
    for (name, duration) in &tokens.duration {
        out.var(&format!("duration-{name}-use"), string(&duration.purpose));
    }
    for (name, easing) in &tokens.easing {
        out.var(&format!("easing-{name}-use"), string(&easing.purpose));
    }
    out.comment("The matugen colour each role is read from at runtime; empty for a role of our own.");
    for (role, source) in &tokens.color.source {
        out.var(&format!("source-{role}"), string(source));
    }
    out.comment("Every foreground drawn on a background, at its opacity, and the ratio it needs.");
    for (index, pair) in tokens.contrast.pairs.iter().enumerate() {
        out.var(&format!("contrast-{index}-fg"), string(&pair.fg));
        out.var(&format!("contrast-{index}-bg"), string(&pair.bg));
        out.var(&format!("contrast-{index}-need"), string(need(pair.need)));
        out.var(&format!("contrast-{index}-minimum"), number(pair.need.minimum()));
        out.var(
            &format!("contrast-{index}-opacity"),
            string(pair.opacity.as_deref().unwrap_or("")),
        );
        out.var(&format!("contrast-{index}-use"), string(&pair.purpose));
    }
    out.var("contrast-pairs", tokens.contrast.pairs.len().to_string());
    out.comment("Every derived role's recipe in each mode, as JSON.");
    out.var("derived-dark", string(&recipes(tokens, Mode::Dark)));
    out.var("derived-light", string(&recipes(tokens, Mode::Light)));
    out.comment("The roles drawn only as decoration, which need no contrast.");
    out.var("contrast-decorative", string(&tokens.contrast.decorative.join(" ")));
}

fn need(need: Need) -> &'static str {
    match need {
        Need::Text => "text",
        Need::LargeText => "large_text",
        Need::Graphic => "graphic",
    }
}

/// `text` as a CSS string, which a script reads back as JSON.
fn string(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests;
