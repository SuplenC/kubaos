//! The components page: every component of the closed set, each kind and size, resting, focused
//! and disabled, one section after another.
//!
//! Each section is a module of its own under `components/`, and `qml/specimen/sections/` holds the
//! same section drawn with `KubaOS.Components`, by the same name. A new component adds its module,
//! its `mod` line and its line in [`SECTIONS`], and its QML file and its line in
//! `qml/specimen/Components.qml`, at the same place: parity holds the two pages to the same boxes,
//! in order. What more than one section draws is in `samples`.
//!
//! # Owns
//!
//! - The order of the sections, and laying them out as the page.
//!
//! # Not here
//!
//! - What each section shows: its module. The page's header and the bar: [`super`].

mod banners;
mod buttons;
mod calendars;
mod cards;
mod chips;
mod collapsibles;
mod description_lists;
mod dialogs;
mod empty_states;
mod errors;
mod fields;
mod icon_buttons;
mod icon_tiles;
mod keycaps;
mod lists;
mod media;
mod menus;
mod meters;
mod page_scroll;
mod progress;
mod samples;
mod sheets;
mod skeletons;
mod sliders;
mod steps;
mod tables;
mod tabs;
mod tiles;
mod toggles;
mod tooltips;

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::layout::section;
use kubaos_ui::layout::stack;
use kubaos_ui::tokens::Space;

use crate::specimen::Message;

/// One section of the page: a titled part, its lede, and the samples under them.
#[derive(Debug)]
pub struct Section {
    pub title: &'static str,
    pub lede: Option<&'static str>,
    pub view: fn(&Look) -> Element<'static, Message>,
}

/// The page's sections, in the order they show.
const SECTIONS: &[Section] = &[
    buttons::SECTION,
    icon_buttons::SECTION,
    fields::SECTION,
    toggles::SECTION,
    sliders::SECTION,
    tabs::SECTION,
    tiles::SECTION,
    icon_tiles::SECTION,
    skeletons::SECTION,
    lists::SECTION,
    page_scroll::SECTION,
    chips::SECTION,
    keycaps::SECTION,
    tooltips::SECTION,
    cards::SECTION,
    sheets::SECTION,
    calendars::SECTION,
    media::SECTION,
    banners::SECTION,
    steps::SECTION,
    description_lists::SECTION,
    dialogs::SECTION,
    progress::SECTION,
    meters::SECTION,
    empty_states::SECTION,
    menus::SECTION,
    tables::SECTION,
    collapsibles::SECTION,
    errors::SECTION,
];

pub fn view<'a>(look: &Look) -> Element<'a, Message> {
    stack(look, Space::S48)
        .extend(
            SECTIONS
                .iter()
                .map(|part| section(look, part.title, part.lede, (part.view)(look))),
        )
        .into()
}
