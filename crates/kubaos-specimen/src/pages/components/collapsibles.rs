//! The components page's "Collapsibles" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Collapsibles.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::collapsible::collapsible;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use crate::specimen::Message;
use super::Section;

pub const SECTION: Section = Section {
    title: "Collapsibles",
    lede: Some("A titled part that opens to show more."),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    grid(look, 2, Space::S24)
        .push(collapsible(
            look,
            "firefox · pid 4127",
            true,
            Message::Sample,
            false,
            stack(look, Space::S4)
                .push(text(
                    "/usr/lib/firefox/firefox -contentproc",
                    Type::Mono,
                    c.on_surface_variant,
                ))
                .push(text(
                    "kuba · sleeping · 92 threads · 2.1 GB",
                    Type::Small,
                    c.on_surface_variant,
                )),
        ))
        .push(collapsible(
            look,
            "Hyprland · pid 1203",
            false,
            Message::Sample,
            true,
            text("Hidden while closed.", Type::Small, c.on_surface_variant),
        ))
        .into()
}
