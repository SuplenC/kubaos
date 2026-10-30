//! What more than one section of the components page draws: the sample buttons, their states and
//! size names, which the button tables and the page scroll share, and the picture the media and
//! icon tile samples load.
//!
//! # Owns
//!
//! - The shared samples, as `qml/specimen/sections/samples.js` holds them for the QML page.
//!
//! # Not here
//!
//! - The sections that draw them: their modules beside this one.

use std::sync::LazyLock;

use iced::Element;
use iced::widget::image;
use kubaos_ui::Look;
use kubaos_ui::Mode;
use kubaos_ui::component::button::ButtonSize;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::Preview;
use kubaos_ui::component::button::button;
use kubaos_ui::icon::Icon;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Type;

use crate::specimen::Message;

/// The states a button is shown in, left to right.
pub(super) const STATES: [&str; 5] = ["Resting", "Hovered", "Pressed", "Focused", "Disabled"];

/// A label button of `kind` and `size` in the `state`th of [`STATES`].
pub(super) fn button_in<'a>(look: &Look, kind: Kind, size: ButtonSize, state: usize) -> Element<'a, Message> {
    let (label, glyph) = match kind {
        Kind::Filled => ("Continue", Icon::ArrowForward),
        Kind::Tonal => ("Save log", Icon::Save),
        Kind::Text => ("Back", Icon::ArrowBack),
        Kind::Danger => ("Erase disk", Icon::Warning),
    };
    let b = button::<Message>(label).kind(kind).size(size);
    let b = if kind == Kind::Filled {
        b.trailing_icon(glyph)
    } else {
        b.icon(glyph)
    };
    match state {
        0 => b.on_press(Message::Sample),
        1 => b.on_press(Message::Sample).preview(Preview::Hovered),
        2 => b.on_press(Message::Sample).preview(Preview::Pressed),
        3 => b.on_press(Message::Sample).focused(true),
        _ => b,
    }
    .view(look)
}

/// A button size's name over its table, lowercase as sizes are written, then its height.
pub(super) fn size_name(size: ButtonSize) -> String {
    let (name, height) = match size {
        ButtonSize::Sm => ("sm", Size::ControlSm),
        ButtonSize::Md => ("md", Size::ControlMd),
        ButtonSize::Lg => ("lg", Size::ControlLg),
    };
    format!("{name} · {}", height.px())
}

/// The states' names over a table's columns, after the kinds' column.
pub(super) fn state_heads<'a>(look: &Look) -> Vec<Element<'a, Message>> {
    let c = look.colors;
    std::iter::once(text("", Type::Label, c.on_surface_variant))
        .chain(STATES.map(|state| text(state, Type::Label, c.on_surface_variant)))
        .collect()
}

/// The picture the media samples load, in each mode: the accent fading into the cold accent, made
/// once, since a handle made from pixels is uploaded again whenever it is made.
pub(super) static PICTURES: LazyLock<[image::Handle; 2]> = LazyLock::new(|| [Mode::Dark, Mode::Light].map(gradient));

/// 16 by 10 pixels from `mode`'s accent on the left to its cold accent on the right.
fn gradient(mode: Mode) -> image::Handle {
    let palette = kubaos_ui::tokens::palette(mode);
    let (from, to) = (palette.primary, palette.cold);
    let (width, height) = (16_u32, 10_u32);
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for _ in 0..height {
        for x in 0..width {
            let t = x as f32 / (width - 1) as f32;
            let blend = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8;
            pixels.extend([blend(from.r, to.r), blend(from.g, to.g), blend(from.b, to.b), u8::MAX]);
        }
    }
    image::Handle::from_rgba(width, height, pixels)
}
