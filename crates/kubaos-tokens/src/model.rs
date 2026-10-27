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
    pub font: Font,
    #[serde(rename = "type")]
    pub types: BTreeMap<String, TypeStyle>,
    pub duration: BTreeMap<String, Duration>,
    pub easing: BTreeMap<String, Easing>,
    /// Opacities laid over a colour role, by name.
    pub opacity: BTreeMap<String, f32>,
    pub shadow: BTreeMap<String, Shadow>,
    pub color: Colors,
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
    /// How far the ring's outer edge stands outside the control, in logical pixels.
    pub offset: u16,
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
    pub weight: Weight,
    pub family: Family,
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
    pub dark: BTreeMap<String, Rgb>,
    pub light: BTreeMap<String, Rgb>,
}

/// An opaque colour, written `#rrggbb`.
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

impl fmt::Debug for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl<'de> Deserialize<'de> for Rgb {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let hex = String::deserialize(deserializer)?;
        Rgb::parse(&hex).ok_or_else(|| de::Error::custom(format!("`{hex}` isn't a colour written #rrggbb")))
    }
}
