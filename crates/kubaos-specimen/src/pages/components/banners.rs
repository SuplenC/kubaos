//! The components page's "Banners" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Banners.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::banner::BannerTone;
use kubaos_ui::component::banner::banner;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::keycap::keycap;
use kubaos_ui::component::progress::danger_timed_bar;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::row;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Opacity;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Banners",
    lede: Some(
        "A message that matters on the page itself, its actions inside: info, a warning, and danger, really red, holding the button that destroys.",
    ),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let on_red = kubaos_ui::component::banner::inside(look, BannerTone::Danger);
    let hold = stack(look, Space::S8)
        .push(
            button("Hold to erase and install")
                .kind(Kind::Danger)
                .icon(Icon::Download)
                .on_press(Message::Sample)
                .view(look),
        )
        .push(
            row(look, Space::S4)
                .center()
                .push(text(
                    "or hold",
                    Type::Caption,
                    kubaos_ui::color::alpha(c.on_error_container, Opacity::SubduedText.value()),
                ))
                .push(keycap(look, "Enter")),
        );
    stack(look, Space::S16)
        .push(
            banner(
                BannerTone::Info,
                Icon::Storage,
                "KubaOS goes into the free space on Samsung SSD 990 PRO.",
                "Windows stays where it is.",
            )
            .view(look),
        )
        .push(
            banner(
                BannerTone::Warning,
                Icon::Warning,
                "12% left. Plug the damn thing in.",
                "An install on battery may not finish.",
            )
            .view(look),
        )
        .push(
            banner(
                BannerTone::Danger,
                Icon::Warning,
                "Everything on Samsung SSD 990 PRO (nvme0n1, 2 TB) gets erased.",
                "All 4 partitions, Windows included. There's no undo.",
            )
            .actions([
                button("Back")
                    .kind(Kind::Text)
                    .icon(Icon::ArrowBack)
                    .on_press(Message::Sample)
                    .view(&on_red),
                hold.into(),
            ])
            .foot(danger_timed_bar(look, 0.4))
            .view(look),
        )
        .into()
}
