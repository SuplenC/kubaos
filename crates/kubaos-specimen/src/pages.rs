//! The specimen's pages, each a header and its sections.
//!
//! # Owns
//!
//! - Picking the page, and each page's content: [`type_scale`], [`colour`], [`space`],
//!   [`components`], [`icons`], [`motion`] and [`templates`].
//!
//! # Not here
//!
//! - The bar above them: [`crate::specimen`].

mod colour;
mod components;
pub mod icons;
mod motion;
mod space;
pub mod templates;
mod type_scale;

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::layout::header;
use kubaos_ui::layout::stack;
use kubaos_ui::tokens::Space;

use crate::specimen::Message;
use crate::specimen::Page;
use crate::specimen::Specimen;

pub fn view<'a>(specimen: &Specimen, look: &Look) -> Element<'a, Message> {
    let (accent, lede) = match specimen.page {
        Page::Type => (
            "書体",
            "Geist for the interface, Geist Mono for anything typed or read by a machine, M PLUS 1 for \
             the Japanese accents. Eleven sizes, 64 for the hero and 80 for the lock screen's clock \
             alone; every line height a multiple of 4.",
        ),
        Page::Colour => (
            "色",
            "Roles, never hex: matugen derives them from the wallpaper. These are the fallback, from \
             the default wallpaper, with each pair's contrast.",
        ),
        Page::Space => (
            "間隔",
            "Everything on a 4 px grid, so it lands on whole device pixels at 1, 1.25, 1.5 and 2. \
             Hairlines are one device pixel; nested corners are concentric.",
        ),
        Page::Components => (
            "部品",
            "The closed set: every kind and size, resting, hovered, pressed, focused and disabled.",
        ),
        Page::Icons => (
            "記号",
            "KubaOS Symbols: Tabler's outline icons, and four of ours where Tabler has none, drawn \
             with a stroke of whole device pixels. The comparison that picked Tabler follows.",
        ),
        Page::Motion => (
            "動き",
            "Seven durations, seven curves. Each animation takes the pair for what it is, never by \
             feel.",
        ),
        Page::Templates => (
            "型",
            "A screen's layout before an app fills it. SidebarPage: an app's sections beside a page \
             that scrolls, Home's search at the top. Here in its states; in the window, live: resize \
             it, and drive it with the keys.",
        ),
    };
    let body = match specimen.page {
        Page::Type => type_scale::view(look),
        Page::Colour => colour::view(look),
        Page::Space => space::view(look),
        Page::Components => components::view(look),
        Page::Icons => icons::view(look),
        Page::Motion => motion::view(look, specimen.seconds, specimen.live),
        Page::Templates => templates::view(look),
    };
    stack(look, Space::S40)
        .push(header(look, accent, specimen.page.title(), lede))
        .push(body)
        .into()
}
