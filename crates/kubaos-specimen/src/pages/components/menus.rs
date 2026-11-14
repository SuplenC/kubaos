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
use kubaos_ui::component::button::Preview;
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

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Selects and menus",
    lede: Some(
        "A popover floats over the page: the surface radius, 12 inside, its items small controls. A select drops one from a field-like control. A danger item lights in the error, and presses deeper. A held item is taken only once held: the error sweeps across it. An item may end in a value of its own.",
    ),
    view,
};

/// The keys of the menu samples' shortcuts.
const OPEN_KEYS: [&str; 1] = ["Enter"];

const COPY_KEYS: [&str; 2] = ["Ctrl", "C"];

/// How far the held sample's sweep has gone.
const HELD_REACH: f32 = 0.4;

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
    // A counter's menu, its held item lit and held part of the way.
    let counter = popover(
        look,
        Extent::Fixed(Size::Menu),
        menu(
            look,
            [
                menu_item("Rename").on_press(Message::Sample).view(look),
                menu_item("Reset now").on_press(Message::Sample).view(look),
                menu_item("History").on_press(Message::Sample).view(look),
                menu_separator(look),
                menu_item("Hold to remove")
                    .held(HELD_REACH, Message::Sample, Message::Sample)
                    .highlighted(true)
                    .view(look),
            ],
        ),
    );
    grid(look, 3, Space::S24)
        .push(open)
        .push(context)
        .push(closed)
        .push(counter)
        .push(danger(look, None))
        .push(danger(look, Some(Preview::Pressed)))
        .push(auto_reset(look))
        .into()
}

/// A danger item lit, pinned pressed when `preview` says so.
fn danger<'a>(look: &Look, preview: Option<Preview>) -> Element<'a, Message> {
    let kill = menu_item("Kill")
        .icon(Icon::Close)
        .danger()
        .highlighted(true)
        .on_press(Message::Sample);
    let kill = match preview {
        Some(preview) => kill.preview(preview),
        None => kill,
    };
    popover(
        look,
        Extent::Fixed(Size::Menu),
        menu(
            look,
            [
                menu_item("Copy command")
                    .icon(Icon::ContentPaste)
                    .on_press(Message::Sample)
                    .view(look),
                menu_separator(look),
                kill.view(look),
            ],
        ),
    )
}

/// A counter's Auto reset choices: When it reaches carries its stop as a value of its own.
fn auto_reset<'a>(look: &Look) -> Element<'a, Message> {
    popover(
        look,
        Extent::Fixed(Size::Menu),
        menu(
            look,
            [
                menu_item("Never").checked(false).on_press(Message::Sample).view(look),
                menu_item("When it reaches")
                    .value("10")
                    .checked(true)
                    .highlighted(true)
                    .on_press(Message::Sample)
                    .view(look),
            ],
        ),
    )
}
