//! The CSS copy of the tokens, for the mockups in `design/mockups/`: every token a custom property,
//! by its name in `design/tokens.toml` (`--size-control_md`, `--color-on_primary`).
//!
//! A length is its logical pixels times `--px`, the CSS pixels a logical pixel takes in the box it
//! is read in. The lengths are declared on `:root` and again on every `.mock`, so a box drawn at
//! another scale sets its own `--scale` and every length inside it follows; so do the durations,
//! as `--motion-*`, under the box's motion settings. The colours and the shadows' opacities follow
//! `data-mode` on the box they are read in: dark by default.
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

use crate::model::Family;
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
        mode(&mut out, tokens, &tokens.color.dark, false);
        out.close();
        out.blank();
        out.open("[data-mode=\"light\"]");
        mode(&mut out, tokens, &tokens.color.light, true);
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
    out.length("focus-offset", focus.offset);
    for (name, shadow) in &tokens.shadow {
        out.length(&format!("shadow-{name}-y"), shadow.y);
        out.length(&format!("shadow-{name}-blur"), shadow.blur);
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
fn mode(out: &mut Css, tokens: &Tokens, palette: &BTreeMap<String, Rgb>, light: bool) {
    out.comment("The colour roles: the fallback palette, matugen's colours for the default wallpaper.");
    for (role, rgb) in palette {
        out.var(&format!("color-{role}"), format!("{rgb:?}"));
    }
    out.comment("The derived roles, computed from their tone wherever it is set, so they follow it.");
    for (role, derived) in &tokens.color.derived {
        let percent = number(derived.white * 100.0);
        out.var(
            &format!("color-{role}"),
            format!("color-mix(in srgb, var(--color-{}), white {percent}%)", derived.mix),
        );
    }
    out.comment("The `shadow` role's opacity for each shadow.");
    for (name, shadow) in &tokens.shadow {
        let opacity = match (light, shadow.light_opacity) {
            (true, Some(opacity)) => opacity,
            _ => shadow.opacity,
        };
        out.var(&format!("shadow-{name}-opacity"), number(opacity));
    }
}

#[cfg(test)]
mod tests;
