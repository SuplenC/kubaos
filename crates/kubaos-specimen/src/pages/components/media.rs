//! The components page's "Media" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Media.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
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

use super::Section;
use super::samples::PICTURES;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Media",
    lede: Some(
        "Pictures in a box of a fixed aspect, reserved before they load, so nothing moves when they arrive: the placeholder, the picture, then pictures to pick: picked with its name under it, focused with its name over its foot, and disabled.",
    ),
    view,
};

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
            picture_choice(Some(picture.clone()), Aspect::Wallpaper, "Golden Gai")
                .name(NamePlace::Over)
                .badge(Icon::PlayArrow, "Video")
                .focused(true)
                .on_press(Message::Sample)
                .view(look),
        )
        .push(
            picture_choice(Some(picture), Aspect::Wallpaper, "Omoide Yokocho")
                .on_press(Message::Sample)
                .disabled(true)
                .view(look),
        )
        .into()
}
