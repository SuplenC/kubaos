//! Icons from KubaOS Symbols, at a size from the scale.
//!
//! The bundled font is Kōbō's subset of Material Symbols Rounded under our name, until the base of
//! KubaOS Symbols is picked from the specimen's comparison sheet; [`Icon`] is the font's interface,
//! so the components keep their names when the glyphs change.
//!
//! # Owns
//!
//! - [`Icon`] and its codepoints, and [`icon`], a glyph in a square box of its size.
//!
//! # Not here
//!
//! - The font file: `assets/fonts/KubaOSSymbols-subset.ttf`, cut by `assets/make-fonts.sh`.

use iced::Color;
use iced::Length;
use iced::alignment;
use iced::widget;

use crate::text;
use crate::tokens::Family;
use crate::tokens::IconSize;
use crate::tokens::Weight;

/// A glyph of KubaOS Symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    ArrowBack,
    ArrowForward,
    Bolt,
    Check,
    CheckCircle,
    ChevronRight,
    Close,
    Code,
    Computer,
    Download,
    Error,
    ExpandLess,
    ExpandMore,
    HardDrive,
    Key,
    Keyboard,
    Language,
    Lock,
    LockOpen,
    Mail,
    Person,
    Power,
    Restart,
    Rocket,
    Save,
    Schedule,
    Search,
    Storage,
    Terminal,
    Timer,
    Visibility,
    VisibilityOff,
    Wallpaper,
    Warning,
    Wifi,
    WifiOff,
}

impl Icon {
    /// The glyph's character in the bundled subset.
    pub fn glyph(self) -> char {
        // The subset's own codepoints, from the Google Fonts API that cut it; some differ from the
        // full Material Symbols font's.
        let codepoint: u32 = match self {
            Icon::ArrowBack => 0xe5c4,
            Icon::ArrowForward => 0xe5c8,
            Icon::Bolt => 0xea0b,
            Icon::Check => 0xe668,
            Icon::CheckCircle => 0xf0be,
            Icon::ChevronRight => 0xe5cc,
            Icon::Close => 0xe5cd,
            Icon::Code => 0xe86f,
            Icon::Computer => 0xe31e,
            Icon::Download => 0xf090,
            Icon::Error => 0xf8b6,
            Icon::ExpandLess => 0xe5ce,
            Icon::ExpandMore => 0xe5cf,
            Icon::HardDrive => 0xf80e,
            Icon::Key => 0xe73c,
            Icon::Keyboard => 0xe312,
            Icon::Language => 0xea07,
            Icon::Lock => 0xe899,
            Icon::LockOpen => 0xe898,
            Icon::Mail => 0xe159,
            Icon::Person => 0xf0d3,
            Icon::Power => 0xf8c7,
            Icon::Restart => 0xf053,
            Icon::Rocket => 0xeb9b,
            Icon::Save => 0xe161,
            Icon::Schedule => 0xefd6,
            Icon::Search => 0xef7a,
            Icon::Storage => 0xe1db,
            Icon::Terminal => 0xeb8e,
            Icon::Timer => 0xe425,
            Icon::Visibility => 0xe8f4,
            Icon::VisibilityOff => 0xe8f5,
            Icon::Wallpaper => 0xe1bc,
            Icon::Warning => 0xf083,
            Icon::Wifi => 0xe63e,
            Icon::WifiOff => 0xe648,
        };
        char::from_u32(codepoint).unwrap_or(char::REPLACEMENT_CHARACTER)
    }
}

/// `icon` at `size`, coloured `color`, centred in a square box as wide and tall as `size`.
pub fn icon<'a>(icon: Icon, size: IconSize, color: Color) -> widget::Text<'a> {
    glyph(
        icon.glyph(),
        text::family_font(Family::Icons, Weight::Regular),
        size,
        color,
    )
}

/// The character `c` of an icon font, centred in a square box of `size`.
pub fn glyph<'a>(c: char, font: iced::Font, size: IconSize, color: Color) -> widget::Text<'a> {
    let px = f32::from(size.px());
    widget::text(c.to_string())
        .font(font)
        .size(px)
        .line_height(widget::text::LineHeight::Absolute(px.into()))
        .width(Length::Fixed(px))
        .height(Length::Fixed(px))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .color(color)
}
