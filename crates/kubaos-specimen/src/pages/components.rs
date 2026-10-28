//! The components page: every component of the closed set, each kind and size, resting, focused
//! and disabled, one section after another.
//!
//! Each section is a module of its own under `components/`, and `qml/specimen/sections/` holds the
//! same section drawn with `KubaOS.Components`, by the same name. A new component adds its module
//! and its line in [`SECTIONS`], and its QML file and its line in `qml/specimen/Components.qml`,
//! at the same place: parity holds the two pages to the same boxes, in order.
//!
//! # Owns
//!
//! - The order of the sections, and laying them out as the page.
//!
//! # Not here
//!
//! - What each section shows: its module. The page's header and the bar: [`super`].

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::layout::section;
use kubaos_ui::layout::stack;
use kubaos_ui::tokens::Space;

use crate::specimen::Message;

/// One section of the page: a titled part, its lede, and the samples under them.
pub struct Section {
    pub title: &'static str,
    pub lede: Option<&'static str>,
    pub view: fn(&Look) -> Element<'static, Message>,
}

/// Declares each section's module, and [`SECTIONS`], the sections in the order they show.
macro_rules! sections {
    ($($module:ident),* $(,)?) => {
        $(mod $module;)*

        /// The page's sections, in the order they show.
        const SECTIONS: &[Section] = &[$($module::SECTION),*];
    };
}

sections! {
    buttons,
    icon_buttons,
    fields,
    toggles,
    sliders,
    tabs,
    tiles,
    icon_tiles,
    skeletons,
    lists,
    page_scroll,
    chips,
    keycaps,
    tooltips,
    cards,
    sheets,
    calendars,
    media,
    banners,
    steps,
    description_lists,
    dialogs,
    progress,
    meters,
    empty_states,
    menus,
    tables,
    collapsibles,
    errors,
}

pub fn view<'a>(look: &Look) -> Element<'a, Message> {
    stack(look, Space::S48)
        .extend(
            SECTIONS
                .iter()
                .map(|part| section(look, part.title, part.lede, (part.view)(look))),
        )
        .into()
}
