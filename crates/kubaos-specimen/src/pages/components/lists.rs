//! The components page's "Lists" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Lists.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::chip::ChipKind;
use kubaos_ui::component::list::empty_list;
use kubaos_ui::component::list::list;
use kubaos_ui::component::list::list_row;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::grid;
use kubaos_ui::tokens::Space;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Lists",
    lede: Some(
        "Rows in a well: a surface, 24, with 12 inside, so the rows take 24 − 12 = 12, the control radius. The focus tube lights the well's edge; the row shows where the keyboard is.",
    ),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let rows = |look: &Look| -> Vec<Element<'a, Message>> {
        vec![
            list_row("English (US)")
                .chip(ChipKind::Accent, "Default")
                .detail("us")
                .checked(true)
                .on_press(Message::Sample)
                .view(look),
            list_row("Italian")
                .detail("it")
                .checked(false)
                .selected(true)
                .on_press(Message::Sample)
                .view(look),
            list_row("German")
                .detail("de")
                .checked(false)
                .on_press(Message::Sample)
                .view(look),
        ]
    };
    let two_line = vec![
        list_row("Home-5G")
            .subtitle("Secured · strong")
            .icon(Icon::Wifi)
            .selected(true)
            .on_press(Message::Sample)
            .view(look),
        list_row("FRITZ!Box 7590 XL Gästenetz OG")
            .subtitle("Secured · fair")
            .icon(Icon::Wifi)
            .on_press(Message::Sample)
            .view(look),
        list_row("Samsung SSD 990 PRO")
            .subtitle("nvme0n1 · 2 TB · Linux")
            .icon(Icon::HardDrive)
            .detail("2 TB")
            .on_press(Message::Sample)
            .view(look),
    ];
    let power = vec![
        list_row("Reboot")
            .icon(Icon::RestartAlt)
            .on_press(Message::Sample)
            .view(look),
        list_row("Shut down")
            .icon(Icon::PowerSettingsNew)
            .danger()
            .selected(true)
            .on_press(Message::Sample)
            .view(look),
    ];
    let tiled = vec![
        list_row("English (US)")
            .subtitle("us")
            .icon(Icon::StarFilled)
            .tile()
            .accent()
            .checked(false)
            .on_press(Message::Sample)
            .view(look),
        list_row("Italian")
            .subtitle("it")
            .icon(Icon::Keyboard)
            .tile()
            .checked(true)
            .selected(true)
            .on_press(Message::Sample)
            .view(look),
    ];
    let radios = vec![
        list_row("Shinjuku-5G")
            .subtitle("Connected · 5 GHz")
            .subtitle_icon(Icon::Lock)
            .icon(Icon::SignalWifi4Bar)
            .tile()
            .accent()
            .checked(true)
            .on_press(Message::Sample)
            .view(look),
        list_row("Omoide Yokocho")
            .subtitle("Secured")
            .subtitle_icon(Icon::Lock)
            .icon(Icon::NetworkWifi2Bar)
            .tile()
            .checked(false)
            .on_press(Message::Sample)
            .view(look),
    ];
    grid(look, 3, Space::S24)
        .push(list(look, rows(look), false))
        .push(list(look, two_line, true))
        .push(empty_list(look, "Nothing matches. Typo?"))
        .push(list(look, power, false))
        .push(list(look, tiled, false))
        .push(list(look, radios, false))
        .into()
}
