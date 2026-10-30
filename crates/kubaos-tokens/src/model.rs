//! The tokens as `design/tokens.toml` holds them. Each table's entries come out sorted (by name,
//! or by value where the table is a scale), so every output is the same for the same file.
//!
//! # Owns
//!
//! - The shape of the file: what a type style, a duration or a colour role is, and reading it.
//!
//! # Not here
//!
//! - The rules the values must follow: [`crate::rules`]. Writing them out: [`crate::target`].

use std::collections::BTreeMap;
use std::fmt;

use serde::Deserialize;
use serde::de;

use crate::contrast::Mode;
use crate::oklch::Oklch;
use crate::oklch::fit_text;

/// Every token in `design/tokens.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tokens {
    pub rules: Rules,
    pub stroke: Stroke,
    pub focus: Focus,
    pub space: Steps,
    pub radius: Radii,
    pub icon: IconSizes,
    /// Fixed dimensions of components, by name.
    pub size: BTreeMap<String, u16>,
    pub container: Container,
    /// Aspect ratios a picture's box keeps, width to height, by name.
    pub aspect: BTreeMap<String, [u16; 2]>,
    pub font: Font,
    #[serde(rename = "type")]
    pub types: BTreeMap<String, TypeStyle>,
    pub duration: BTreeMap<String, Duration>,
    pub easing: BTreeMap<String, Easing>,
    /// Opacities laid over a colour role, by name.
    pub opacity: BTreeMap<String, f32>,
    pub shadow: BTreeMap<String, Shadow>,
    pub neon: Neon,
    pub boot: Boot,
    pub color: Colors,
    pub contrast: ContrastPairs,
    pub terminal: Terminal,
    /// KubaOS Symbols, by the Material Symbols name each replaces.
    pub symbols: BTreeMap<String, Symbol>,
}

/// The rules every other value is checked against.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    /// Every length is a multiple of this, in logical pixels.
    pub grid: u16,
    /// The only font sizes a type style may take.
    pub font_sizes: Vec<u16>,
    /// The display scales every length must land on whole device pixels at.
    pub scales: Vec<f32>,
    /// Sizes allowed off the grid, as `size.<name>`: marks drawn centred on a point that moves
    /// with a value, so their edges miss whole device pixels whatever their size.
    #[serde(default)]
    pub off_grid: Vec<String>,
}

/// Lines thinner than the grid, drawn in whole device pixels.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stroke {
    pub hairline_device_px: u16,
    /// Logical pixels, rounded half up to whole device pixels at each scale.
    pub thin: u16,
    /// A symbol's stroke on its 24 unit grid.
    pub symbol: u16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Focus {
    /// The tube's width in logical pixels, drawn rounded half up to whole device pixels.
    pub tube: f32,
    /// The near glow's reach outside the tube, in logical pixels.
    pub near: u16,
    /// The near glow's opacity at the tube.
    pub near_opacity: f32,
    /// The far glow's reach outside the tube, in logical pixels.
    pub far: u16,
    /// The far glow's opacity at the tube.
    pub far_opacity: f32,
    /// Content drawn in its tone, lit: its near glow's reach outside the content, in logical
    /// pixels.
    pub content_near: u16,
    /// Content drawn in its tone, lit: its near glow's opacity at the content.
    pub content_glow: f32,
    /// A lit label on a fill (a Filled button's, at night): its glow's reach outside the letters,
    /// in logical pixels.
    pub label_near: u16,
    /// A lit label on a fill: its glow's opacity at the letters.
    pub label_glow: f32,
    /// How much brighter a pressed control's glow is than a lit one's, as a multiple.
    pub pressed_glow: f32,
    /// The band a picture keeps round itself inside its own box where its neighbours leave no
    /// more room, in logical pixels.
    pub offset: u16,
    /// In daylight, the opacity of an unlit tube's glass over its control's container.
    pub glass: f32,
}

/// A scale of lengths, smallest first.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Steps {
    pub values: Vec<u16>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Radii {
    /// Marks inside controls.
    pub small: u16,
    /// Everything clicked or typed in.
    pub control: u16,
    /// Everything that holds content.
    pub surface: u16,
    /// Whether there is a pill radius, half the shape's shorter side.
    pub full: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IconSizes {
    pub sizes: Vec<u16>,
    /// A row's icon beside a title over a subtitle: one of `sizes`.
    pub two_line: u16,
}

/// A page's or a panel's padding by its width: each padding applies from its width up.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Container {
    /// The width each padding applies from, smallest first; the first is 0.
    pub from: Vec<u16>,
    /// The padding inside every edge, one per width in `from`.
    pub padding: Vec<u16>,
}

/// Every foreground drawn on a background, and the contrast each needs.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContrastPairs {
    pub pairs: Vec<ContrastPair>,
    /// Roles drawn only as decoration, which carries no meaning and needs no contrast.
    pub decorative: Vec<String>,
}

/// One foreground role on one background role.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContrastPair {
    pub fg: String,
    pub bg: String,
    pub need: Need,
    /// An opacity the foreground is drawn at, by its name in `[opacity]`.
    pub opacity: Option<String>,
    /// Where the pair is drawn, in a few words.
    #[serde(rename = "use")]
    pub purpose: String,
}

/// The contrast a pair needs, by what the foreground is (WCAG 2.1 AA).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Need {
    /// Body text and labels: 4.5:1.
    Text,
    /// Text of 24 px and up, or 19 px semibold and up: 3:1.
    LargeText,
    /// Icons, outlines and controls that carry meaning: 3:1.
    Graphic,
}

impl Need {
    /// The lowest ratio that passes.
    pub fn minimum(self) -> f32 {
        match self {
            Need::Text => 4.5,
            Need::LargeText | Need::Graphic => 3.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Font {
    pub family: Families,
}

/// The type families, by the role they play.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Families {
    pub sans: String,
    pub mono: String,
    pub jp: String,
}

impl Families {
    /// The family's name as the font files spell it.
    pub fn name(&self, family: Family) -> &str {
        match family {
            Family::Sans => &self.sans,
            Family::Mono => &self.mono,
            Family::Jp => &self.jp,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Family {
    Sans,
    Mono,
    Jp,
}

impl Family {
    pub const ALL: [Family; 3] = [Family::Sans, Family::Mono, Family::Jp];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Weight {
    Light,
    Regular,
    Medium,
    Semibold,
}

impl Weight {
    pub const ALL: [Weight; 4] = [Weight::Light, Weight::Regular, Weight::Medium, Weight::Semibold];
}

/// A type style: what a piece of text is drawn with.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeStyle {
    pub size: u16,
    pub line: u16,
    /// The icon size that sits beside the style on its line.
    pub icon: u16,
    pub weight: Weight,
    pub family: Family,
    /// Whether its figures are tabular (OpenType `tnum`), all one width, so a number that changes
    /// in place keeps its width. Off unless the style says so.
    #[serde(default)]
    pub tabular: bool,
    /// What the style is for, in a sentence.
    #[serde(rename = "use")]
    pub purpose: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Duration {
    pub ms: u32,
    #[serde(rename = "use")]
    pub purpose: String,
}

/// A CSS cubic bézier: x1, y1, x2, y2.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Easing {
    pub curve: [f32; 4],
    #[serde(rename = "use")]
    pub purpose: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shadow {
    pub y: u16,
    pub blur: u16,
    pub opacity: f32,
    /// Its opacity in light mode, where a shadow of the same strength reads heavier on the light
    /// surfaces; `opacity` in both modes when absent.
    #[serde(default)]
    pub light_opacity: Option<f32>,
}

/// The neon sign: how a tube strikes, and the クバ mark drawn as tubes.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Neon {
    /// A striking tube's brightness, 0 to 1, through even steps over `medium`.
    pub strike: Vec<f32>,
    pub mark: Mark,
}

/// The クバ mark: its strokes and how wide its tubes are, in its drawing's units.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mark {
    /// The drawing's side.
    pub units: f32,
    /// How far inside the drawing the strokes' coordinates start.
    pub margin: f32,
    /// A warm tube's width.
    pub width: f32,
    /// A cold tube's width, of a warm one's.
    pub cold_width: f32,
    /// A warm tube's core's width, of a warm tube's.
    pub core: f32,
    /// A cold tube's core's width, of a warm tube's.
    pub cold_core: f32,
    /// In the order they are written.
    pub strokes: Vec<MarkStroke>,
}

/// One stroke of the mark.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkStroke {
    /// SVG path data: absolute `M`, `L`, `H` and `Q` commands, each letter starting a word.
    pub path: String,
    pub tone: Tone,
}

/// A tube's colour family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    /// `primary`, with `logo_core_warm` along its middle.
    Warm,
    /// `cold`, with `logo_core_cold` along its middle.
    Cold,
}

/// How the boot screens, which draw in device pixels, scale the tokens.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Boot {
    /// The one scale GRUB's theme draws at: one of `rules.scales`.
    pub grub_scale: f32,
    /// The screen height in device pixels from which Plymouth draws at 2.
    pub plymouth_2x_from: u16,
}

/// Where a symbol's drawing comes from: Tabler's outline icon of a name, or our own, described.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Symbol {
    pub tabler: Option<String>,
    pub drawn: Option<String>,
}

/// The colour roles: where each comes from at runtime, and its fallback in each mode.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Colors {
    /// The matugen colour each role is read from; empty for a role of our own.
    pub source: BTreeMap<String, String>,
    /// The fallback palettes. A role of our own given by a recipe has its value here too once the
    /// tokens are read ([`Colors::fill_recipes`]).
    pub dark: BTreeMap<String, Rgb>,
    pub light: BTreeMap<String, Rgb>,
    /// The roles computed from another role, whatever its value in a palette: never a fixed value,
    /// so they follow the wallpaper. A role of our own may be one of them instead of a value in
    /// each palette.
    #[serde(default)]
    pub derived: BTreeMap<String, Derived>,
}

/// A role computed from others, whichever palette is drawn, so it follows the wallpaper. A recipe
/// starts from a role (`mix` moved toward white or black in sRGB, or `from` as it is), may move its
/// OKLCH lightness and chroma (fitted back into sRGB at the same hue and lightness), and may then
/// fit its lightness until it holds text contrast on every role in `text_on`. A role with a recipe
/// for each mode gives it in `dark` and `light` and nothing else. `rules` refuses anything else.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Derived {
    /// The role it is mixed from, in sRGB, toward white or black.
    #[serde(default)]
    pub mix: Option<String>,
    /// How far toward white, 0 to 1.
    #[serde(default)]
    pub white: Option<f32>,
    /// How far toward black, 0 to 1.
    #[serde(default)]
    pub black: Option<f32>,
    /// The role it starts from as it is.
    #[serde(default)]
    pub from: Option<String>,
    /// Its OKLCH lightness times this.
    #[serde(default)]
    pub lightness: Option<f32>,
    /// Its OKLCH lightness set to this.
    #[serde(default)]
    pub at_lightness: Option<f32>,
    /// Its OKLCH lightness raised by this.
    #[serde(default)]
    pub lighter: Option<f32>,
    /// Its OKLCH lightness lowered by this.
    #[serde(default)]
    pub darker: Option<f32>,
    /// Its OKLCH chroma times this, as far as sRGB holds it.
    #[serde(default)]
    pub chroma: Option<f32>,
    /// The backgrounds it is text on: its lightness moves until it holds 4.5:1 on each.
    #[serde(default)]
    pub text_on: Option<Vec<String>>,
    /// Its recipe in dark mode, for a role computed differently in each mode.
    #[serde(default)]
    pub dark: Option<Box<Derived>>,
    /// Its recipe in light mode.
    #[serde(default)]
    pub light: Option<Box<Derived>>,
}

/// Which way a derived role moves from its tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Toward {
    White,
    Black,
}

impl Toward {
    /// Its name, as the tokens and the outputs write it.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Toward::White => "white",
            Toward::Black => "black",
        }
    }
}

impl Derived {
    /// Which way it moves and how far, for a role that is only its tone mixed toward white or
    /// black, the same in both modes: one every toolkit computes as a formula of the live tone.
    /// `None` for every other recipe.
    #[must_use]
    pub fn toward(&self) -> Option<(Toward, f32)> {
        if !self.is_mix() {
            return None;
        }
        self.way()
    }

    /// Which way its `mix` moves and how far, or `None` for one that names both or neither.
    #[must_use]
    pub fn way(&self) -> Option<(Toward, f32)> {
        match (self.white, self.black) {
            (Some(t), None) => Some((Toward::White, t)),
            (None, Some(t)) => Some((Toward::Black, t)),
            _ => None,
        }
    }

    /// Whether it is only `mix` toward white or black, in one recipe for both modes.
    #[must_use]
    pub fn is_mix(&self) -> bool {
        self.mix.is_some() && !self.moves_oklch() && self.from.is_none() && self.text_on.is_none() && !self.by_mode()
    }

    /// The role its `mix` names, or nothing.
    #[must_use]
    pub fn tone(&self) -> &str {
        self.mix.as_deref().unwrap_or_default()
    }

    /// Whether it gives a recipe for each mode.
    #[must_use]
    pub fn by_mode(&self) -> bool {
        self.dark.is_some() || self.light.is_some()
    }

    /// Whether it moves the colour's OKLCH lightness or chroma.
    #[must_use]
    pub fn moves_oklch(&self) -> bool {
        self.lightness.is_some()
            || self.at_lightness.is_some()
            || self.lighter.is_some()
            || self.darker.is_some()
            || self.chroma.is_some()
    }

    /// Its recipe in `mode`.
    #[must_use]
    pub fn in_mode(&self, mode: Mode) -> &Derived {
        let own = match mode {
            Mode::Dark => self.dark.as_deref(),
            Mode::Light => self.light.as_deref(),
        };
        own.unwrap_or(self)
    }

    /// Every role its recipe in `mode` reads.
    pub fn reads(&self, mode: Mode) -> impl Iterator<Item = &String> {
        let recipe = self.in_mode(mode);
        recipe
            .mix
            .iter()
            .chain(recipe.from.iter())
            .chain(recipe.text_on.iter().flatten())
    }

    /// Its colour in `mode` from the roles it reads, as `role` gives them, or `None` when one is
    /// missing.
    fn compute(&self, mode: Mode, role: &mut dyn FnMut(&str) -> Option<Rgb>) -> Option<Rgb> {
        let recipe = self.in_mode(mode);
        let mut colour = match (&recipe.mix, &recipe.from, recipe.way()) {
            (Some(mix), None, Some((Toward::White, t))) => role(mix)?.toward_white(t),
            (Some(mix), None, Some((Toward::Black, t))) => role(mix)?.toward_black(t),
            (None, Some(from), None) => role(from)?,
            _ => return None,
        };
        if recipe.moves_oklch() {
            let mut lch = Oklch::of(colour);
            if let Some(at) = recipe.at_lightness {
                lch.l = f64::from(at);
            }
            lch.l *= f64::from(recipe.lightness.unwrap_or(1.0));
            lch.l += f64::from(recipe.lighter.unwrap_or(0.0)) - f64::from(recipe.darker.unwrap_or(0.0));
            lch.c *= f64::from(recipe.chroma.unwrap_or(1.0));
            colour = lch.to_rgb();
        }
        if let Some(on) = &recipe.text_on {
            let backgrounds = on.iter().map(|name| role(name)).collect::<Option<Vec<_>>>()?;
            colour = fit_text(colour, &backgrounds, Need::Text.minimum());
        }
        Some(colour)
    }
}

impl Colors {
    /// Whether `role` is one of ours given by a recipe rather than by a value in each palette.
    #[must_use]
    pub fn by_recipe(&self, role: &str) -> bool {
        self.source.get(role).is_some_and(String::is_empty) && self.derived.contains_key(role)
    }

    /// Puts each role of ours given by a recipe into both fallback palettes, computed from them,
    /// so the outputs that draw a palette as values (iced, QML, the terminals) have it.
    pub fn fill_recipes(&mut self) {
        let dark = self.with_derived(&self.dark, Mode::Dark);
        let light = self.with_derived(&self.light, Mode::Light);
        let roles: Vec<String> = self
            .source
            .keys()
            .filter(|role| self.by_recipe(role))
            .cloned()
            .collect();
        for role in roles {
            if let Some(&rgb) = dark.get(&role) {
                self.dark.insert(role.clone(), rgb);
            }
            if let Some(&rgb) = light.get(&role) {
                self.light.insert(role, rgb);
            }
        }
    }

    /// `palette`, the colours of `mode`, with the derived roles computed from its own values. A
    /// derived role whose roles the palette lacks, or that reads itself, is left out; `rules`
    /// refuses either.
    #[must_use]
    pub fn with_derived(&self, palette: &BTreeMap<String, Rgb>, mode: Mode) -> BTreeMap<String, Rgb> {
        let mut out = palette.clone();
        let mut reading = Vec::new();
        for role in self.derived.keys() {
            self.resolve(role, mode, &mut out, &mut reading);
        }
        out
    }

    fn resolve(
        &self,
        name: &str,
        mode: Mode,
        out: &mut BTreeMap<String, Rgb>,
        reading: &mut Vec<String>,
    ) -> Option<Rgb> {
        if let Some(&known) = out.get(name) {
            return Some(known);
        }
        let derived = self.derived.get(name)?;
        if reading.iter().any(|each| each == name) {
            return None;
        }
        reading.push(name.to_owned());
        let colour = derived.compute(mode, &mut |role| self.resolve(role, mode, out, reading));
        reading.pop();
        if let Some(colour) = colour {
            out.insert(name.to_owned(), colour);
        }
        colour
    }
}

/// An opaque colour, written `#rrggbb`, or `oklch(L C H)` for a fixed tone decided in OKLCH.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    /// Reads `#rrggbb`, or `None` for anything else.
    pub fn parse(hex: &str) -> Option<Rgb> {
        let digits = hex.strip_prefix('#')?;
        if digits.len() != 6 || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let channel = |at: usize| u8::from_str_radix(&digits[at..at + 2], 16).ok();
        Some(Rgb {
            r: channel(0)?,
            g: channel(2)?,
            b: channel(4)?,
        })
    }
}

impl Rgb {
    /// Reads `oklch(L C H)`, fitted into sRGB at its hue and lightness, or `None` for anything
    /// else.
    pub fn parse_oklch(written: &str) -> Option<Rgb> {
        let inside = written.strip_prefix("oklch(")?.strip_suffix(')')?;
        let numbers: Vec<f64> = inside
            .split_whitespace()
            .map(str::parse)
            .collect::<Result<_, _>>()
            .ok()?;
        let [l, c, h] = numbers.as_slice() else { return None };
        Some(Oklch { l: *l, c: *c, h: *h }.to_rgb())
    }
}

impl fmt::Debug for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl<'de> Deserialize<'de> for Rgb {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let written = String::deserialize(deserializer)?;
        Rgb::parse(&written)
            .or_else(|| Rgb::parse_oklch(&written))
            .ok_or_else(|| de::Error::custom(format!("`{written}` isn't a colour written #rrggbb or oklch(L C H)")))
    }
}

impl Rgb {
    /// This colour mixed `t` of the way to white, each channel rounded half up.
    #[must_use]
    pub fn toward_white(self, t: f32) -> Rgb {
        let channel = |c: u8| {
            let c = f32::from(c);
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "Clamped to 0..=255 and rounded first."
            )]
            let byte = (c + (255.0 - c) * t + 0.5).floor().clamp(0.0, 255.0) as u8;
            byte
        };
        Rgb {
            r: channel(self.r),
            g: channel(self.g),
            b: channel(self.b),
        }
    }

    /// This colour mixed `t` of the way to black, each channel rounded half up.
    #[must_use]
    pub fn toward_black(self, t: f32) -> Rgb {
        let channel = |c: u8| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "Clamped to 0..=255 and rounded first."
            )]
            let byte = (f32::from(c) * (1.0 - t) + 0.5).floor().clamp(0.0, 255.0) as u8;
            byte
        };
        Rgb {
            r: channel(self.r),
            g: channel(self.g),
            b: channel(self.b),
        }
    }

    /// This colour with its HSL lightness moved by `points` of 100, clamped: matugen's `lighten`.
    #[must_use]
    pub fn lighten(self, points: f32) -> Rgb {
        let [red, green, blue] = [self.r, self.g, self.b].map(|c| f32::from(c) / 255.0);
        let max = red.max(green).max(blue);
        let min = red.min(green).min(blue);
        let chroma = max - min;
        let lightness = f32::midpoint(max, min);
        let grey = chroma <= f32::EPSILON;
        let saturation = if grey {
            0.0
        } else {
            chroma / (1.0 - (2.0 * lightness - 1.0).abs())
        };
        // The hue as a sixth of the circle, 0 to 6, from the channel that is largest.
        let sector = if grey {
            0.0
        } else if max.total_cmp(&red).is_eq() {
            ((green - blue) / chroma).rem_euclid(6.0)
        } else if max.total_cmp(&green).is_eq() {
            (blue - red) / chroma + 2.0
        } else {
            (red - green) / chroma + 4.0
        };
        let lightness = (lightness + points / 100.0).clamp(0.0, 1.0);
        let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
        let second = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());
        let (red, green, blue) = match sector {
            at if at < 1.0 => (chroma, second, 0.0),
            at if at < 2.0 => (second, chroma, 0.0),
            at if at < 3.0 => (0.0, chroma, second),
            at if at < 4.0 => (0.0, second, chroma),
            at if at < 5.0 => (second, 0.0, chroma),
            _ => (chroma, 0.0, second),
        };
        let floor = lightness - chroma / 2.0;
        let channel = |value: f32| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "Clamped to 0..=255 and rounded first."
            )]
            let byte = ((value + floor) * 255.0).round().clamp(0.0, 255.0) as u8;
            byte
        };
        Rgb {
            r: channel(red),
            g: channel(green),
            b: channel(blue),
        }
    }
}

/// The terminals' text, padding and palette.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Terminal {
    /// The type style the terminals set their text in, by its name in `[type]`.
    #[serde(rename = "type")]
    pub style: String,
    /// The font the glyphs the type family lacks come from.
    pub glyphs: String,
    /// Logical pixels between the window's edge and the cells, at the sides.
    pub padding_x: u16,
    /// Logical pixels between the window's edge and the cells, above and below.
    pub padding_y: u16,
    /// The window's background's opacity, by its name in `[opacity]`.
    pub opacity: String,
    /// HSL lightness points a bright ANSI colour stands further from the surface than its normal
    /// one.
    pub bright: u8,
    /// HSL lightness points ANSI 8 stands closer to the surface than ANSI 7.
    pub dim: u8,
    pub roles: TerminalRoles,
    pub hues: Hues,
}

/// The terminal's own colours, each a colour role's name.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminalRoles {
    pub background: String,
    pub foreground: String,
    pub cursor: String,
    pub cursor_text: String,
    pub selection: String,
    pub selection_text: String,
    pub url: String,
    pub divider: String,
    /// ANSI 0.
    pub black: String,
    /// ANSI 7, and ANSI 8 dimmed from it.
    pub white: String,
    /// ANSI 15.
    pub bright_white: String,
}

impl TerminalRoles {
    /// Every slot by its name, with the role it takes.
    pub fn slots(&self) -> [(&'static str, &str); 11] {
        [
            ("background", &self.background),
            ("foreground", &self.foreground),
            ("cursor", &self.cursor),
            ("cursor_text", &self.cursor_text),
            ("selection", &self.selection),
            ("selection_text", &self.selection_text),
            ("url", &self.url),
            ("divider", &self.divider),
            ("black", &self.black),
            ("white", &self.white),
            ("bright_white", &self.bright_white),
        ]
    }
}

/// ANSI 1 to 6: seeds matugen harmonizes toward the wallpaper, as custom colours of these names.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hues {
    pub red: Rgb,
    pub green: Rgb,
    pub yellow: Rgb,
    pub blue: Rgb,
    pub magenta: Rgb,
    pub cyan: Rgb,
}

impl Hues {
    /// Each hue by its name, in ANSI order, 1 to 6.
    pub fn ansi(&self) -> [(&'static str, Rgb); 6] {
        [
            ("red", self.red),
            ("green", self.green),
            ("yellow", self.yellow),
            ("blue", self.blue),
            ("magenta", self.magenta),
            ("cyan", self.cyan),
        ]
    }
}
