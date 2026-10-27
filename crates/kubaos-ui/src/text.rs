//! Text in a type style, and the fonts every app loads.
//!
//! # Owns
//!
//! - Turning a [`Type`] into an iced font, size and line height ([`text`], [`font`]), and the
//!   bundled font files ([`FONTS`]).
//!
//! # Not here
//!
//! - The styles' values: [`crate::tokens`]. Icons: [`crate::icon`].

use iced::Color;
use iced::Font;
use iced::Pixels;
use iced::font;
use iced::widget;
use iced::widget::text::IntoFragment;
use iced::widget::text::LineHeight;

use crate::tokens::Family;
use crate::tokens::Type;
use crate::tokens::Weight;

/// Every font the type styles use, for `iced::Settings::fonts`. An app loads them at
/// start, so it draws the same on a machine with none installed.
pub const FONTS: &[&[u8]] = &[
    include_bytes!("../assets/fonts/Geist-Light.ttf"),
    include_bytes!("../assets/fonts/Geist-Regular.ttf"),
    include_bytes!("../assets/fonts/Geist-Medium.ttf"),
    include_bytes!("../assets/fonts/Geist-SemiBold.ttf"),
    include_bytes!("../assets/fonts/GeistMono-Regular.ttf"),
    include_bytes!("../assets/fonts/GeistMono-Medium.ttf"),
    include_bytes!("../assets/fonts/MPLUS1-subset.ttf"),
];

/// The font of `family` at `weight`.
pub fn family_font(family: Family, weight: Weight) -> Font {
    Font {
        family: font::Family::Name(family.name()),
        weight: match weight {
            Weight::Light => font::Weight::Light,
            Weight::Regular => font::Weight::Normal,
            Weight::Medium => font::Weight::Medium,
            Weight::Semibold => font::Weight::Semibold,
        },
        ..Font::DEFAULT
    }
}

/// The font a type style draws with.
pub fn font(style: Type) -> Font {
    family_font(style.family(), style.weight())
}

/// The font apps draw with when a widget names none: body text.
pub fn default_font() -> Font {
    font(Type::Body)
}

/// `content` in `style`, coloured `color`, on the style's line height.
pub fn text<'a>(content: impl IntoFragment<'a>, style: Type, color: Color) -> widget::Text<'a> {
    widget::text(content)
        .font(font(style))
        .size(f32::from(style.size()))
        .line_height(line_height(style))
        .color(color)
}

/// A type style's line height, in absolute pixels so it stays on the grid.
pub fn line_height(style: Type) -> LineHeight {
    LineHeight::Absolute(Pixels(f32::from(style.line())))
}
