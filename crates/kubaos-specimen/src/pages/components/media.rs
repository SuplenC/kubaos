//! The components page's "Media" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Media.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use std::sync::LazyLock;
use iced::Element;
use iced::widget::image;
use kubaos_ui::Look;
use kubaos_ui::Mode;
use kubaos_ui::component::button::Preview;
use kubaos_ui::component::media::Aspect;
use kubaos_ui::component::media::media;
use kubaos_ui::component::picture::NamePlace;
use kubaos_ui::component::picture::picture_choice;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::grid;
use kubaos_ui::tokens::Radius;
use kubaos_ui::tokens::Space;

use crate::specimen::Message;
use super::Section;

pub const SECTION: Section = Section {
    title: "Media",
    lede: Some("Pictures in a box of a fixed aspect, reserved before they load, so nothing moves when they arrive: the placeholder, the picture, then two pictures to pick, picked with its name under it and focused with its name over its foot."),
    view,
};

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

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let picture = PICTURES[usize::from(look.mode == Mode::Light)].clone();
    grid(look, 4, Space::S16)
        .push(media(look, None, Aspect::Wallpaper, Radius::Control))
        .push(media(look, Some(picture.clone()), Aspect::Wallpaper, Radius::Control))
        .push(
            picture_choice(Some(picture.clone()), Aspect::Wallpaper, "Shinjuku, night")
                .picked(true)
                .mark(Icon::Check, "Current")
                .preview(Preview::Hovered)
                .on_press(Message::Sample)
                .view(look),
        )
        .push(
            picture_choice(Some(picture), Aspect::Wallpaper, "Golden Gai")
                .name(NamePlace::Over)
                .badge(Icon::PlayArrow, "Video")
                .focused(true)
                .on_press(Message::Sample)
                .view(look),
        )
        .into()
}
