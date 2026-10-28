//! KubaOS Symbols: Tabler's outline icons, and our own drawn on their grid, as SVGs drawn with a
//! stroke of whole device pixels.
//!
//! An icon font can't keep a stroke whole: its 2 unit stroke is 1.33 px at 16 and 1.67 px at 20,
//! between device pixels, so every line blurs. Each symbol here is drawn from its SVG with the
//! stroke set for the size and the scale it shows at: 2 units scaled to the size, then rounded half
//! up to whole device pixels ([`device_stroke`]).
//!
//! | size | 1x | 1.25x | 1.5x | 2x |
//! |---|---|---|---|---|
//! | 16 | 1 | 2 | 2 | 3 |
//! | 20 | 2 | 2 | 3 | 3 |
//! | 24 | 2 | 3 | 3 | 4 |
//! | 32 | 3 | 3 | 4 | 5 |
//! | 48 | 4 | 5 | 6 | 8 |
//!
//! # Owns
//!
//! - [`icon`], a symbol in a square box of its size, and its stroke at a scale.
//!
//! # Not here
//!
//! - The list of symbols and where each drawing comes from: `design/tokens.toml`, `[symbols]`,
//!   generated into [`crate::tokens::Icon`]. The drawings: `assets/symbols/`.

use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

use iced::Color;
use iced::Element;
use iced::Length;
use iced::widget;
use iced::widget::svg;

use crate::look::Look;
use crate::probe::Tag;
use crate::probe::probe;
use crate::scale::Scale;
use crate::tokens;
pub use crate::tokens::Icon;
use crate::tokens::IconSize;

/// The grid every symbol is drawn on, in units.
const GRID: f32 = 24.0;

/// The stroke the drawings are written with, as their SVG spells it.
const WRITTEN_STROKE: &str = "stroke-width=\"2\"";

/// `glyph` at `size`, in `color`, in a square box as wide and tall as `size`.
pub fn icon<'a, M: 'a>(look: &Look, glyph: Icon, size: IconSize, color: Color) -> Element<'a, M> {
    let px = f32::from(size.px());
    // The colour filter keeps only the colour's channels; its alpha goes to the opacity, so a
    // disabled or fading icon dims like its label.
    let opaque = Color { a: 1.0, ..color };
    let drawn = widget::svg(symbol(glyph, size, look.scale))
        .width(Length::Fixed(px))
        .height(Length::Fixed(px))
        .opacity(color.a)
        .style(move |_theme, _status| svg::Style { color: Some(opaque) });
    probe(Tag::Icon(glyph.name()), drawn)
}

/// `glyph`'s drawing at `size` and `scale`, its stroke whole device pixels, for a widget that draws
/// the symbol itself.
pub(crate) fn symbol(glyph: Icon, size: IconSize, scale: Scale) -> svg::Handle {
    let px = f32::from(size.px());
    handle(glyph, device_stroke(size, scale) / scale.factor() * GRID / px)
}

/// The stroke of a symbol at `size`, in device pixels at `scale`: the 2 unit stroke scaled to the
/// size, rounded half up to a whole device pixel, never under one.
pub fn device_stroke(size: IconSize, scale: Scale) -> f32 {
    let logical = f32::from(tokens::SYMBOL_STROKE) * f32::from(size.px()) / GRID;
    scale.to_device(scale.stroke(logical))
}

/// `glyph`'s SVG with its stroke set to `grid_stroke` units.
pub fn drawing(glyph: Icon, grid_stroke: f32) -> String {
    glyph
        .svg()
        .replace(WRITTEN_STROKE, &format!("stroke-width=\"{grid_stroke}\""))
}

/// Each drawing at each stroke, made once: a handle made from bytes is hashed and decoded again
/// every time, and a view asks for the same few on every frame.
static HANDLES: LazyLock<Mutex<HashMap<(Icon, u32), svg::Handle>>> = LazyLock::new(Mutex::default);

fn handle(glyph: Icon, grid_stroke: f32) -> svg::Handle {
    // Thousandths of a unit tell every stroke at the supported sizes and scales apart.
    let key = (glyph, (grid_stroke * 1000.0).round() as u32);
    let mut handles = HANDLES.lock().expect("symbol handle cache poisoned");
    handles
        .entry(key)
        .or_insert_with(|| svg::Handle::from_memory(drawing(glyph, grid_stroke).into_bytes()))
        .clone()
}

#[cfg(test)]
mod tests;
