//! The components page's "Selects and menus" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Menus.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::menu::menu;
use kubaos_ui::component::menu::menu_item;
use kubaos_ui::component::menu::menu_separator;
use kubaos_ui::component::menu::popover;
use kubaos_ui::component::select::select;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::stack;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;

use crate::specimen::Message;
use super::Section;

pub const SECTION: Section = Section {
    title: "Selects and menus",
    lede: Some(
                "A popover floats over the page: the surface radius, 12 inside, its items small controls. A select drops one from a field-like control.",
            ),
    view,
};

/// The keys of the menu samples' shortcuts.
const OPEN_KEYS: [&str; 1] = ["Enter"];

const COPY_KEYS: [&str; 2] = ["Ctrl", "C"];

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let countries = ["None", "Germany", "Italy", "Poland"];
    let picked = menu(
        look,
        countries.into_iter().map(|name| {
            menu_item(name)
                .checked(name == "Poland")
                .highlighted(name == "Italy")
                .on_press(Message::Sample)
                .view(look)
        }),
    );
    let open = stack(look, Space::S8)
        .width(Extent::Fill)
        .push(
            select("Poland")
                .icon(Icon::Flag)
                .open(true)
                .on_press(Message::Sample)
                .view(look),
        )
        .push(popover(look, Extent::Fill, picked));
    let context = popover(
        look,
        Extent::Fixed(Size::Menu),
        menu(
            look,
            [
                menu_item("Open")
                    .icon(Icon::FolderOpen)
                    .shortcut(&OPEN_KEYS)
                    .highlighted(true)
                    .on_press(Message::Sample)
                    .view(look),
                menu_item("Copy path")
                    .icon(Icon::ContentPaste)
                    .shortcut(&COPY_KEYS)
                    .on_press(Message::Sample)
                    .view(look),
                menu_separator(look),
                menu_item("Pin to the top")
                    .icon(Icon::PushPin)
                    .on_press(Message::Sample)
                    .view(look),
                menu_item::<Message>("Forget").icon(Icon::Close).view(look),
                menu_separator(look),
                menu_item("Quit")
                    .icon(Icon::Logout)
                    .danger()
                    .on_press(Message::Sample)
                    .view(look),
            ],
        ),
    );
    let closed = stack(look, Space::S16)
        .width(Extent::Fill)
        .push(select("Every 5 minutes").on_press(Message::Sample).view(look))
        .push(
            select("Every 5 minutes")
                .on_press(Message::Sample)
                .focused(true)
                .view(look),
        )
        .push(select::<Message>("Unavailable").view(look));
    grid(look, 3, Space::S24).push(open).push(context).push(closed).into()
}
