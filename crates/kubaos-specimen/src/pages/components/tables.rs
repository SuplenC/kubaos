//! The components page's "Tables" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Tables.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::table::Column;
use kubaos_ui::component::table::table;
use kubaos_ui::component::table::table_row;

use crate::specimen::Message;
use super::Section;

pub const SECTION: Section = Section {
    title: "Tables",
    lede: Some(
                "Readings in columns, a line of text with 4 above and below a row; the whole row takes the pointer, numbers line up at their column's right edge.",
            ),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let columns = [
        Column::new("Name", 4),
        Column::new("CPU", 1).numeric(),
        Column::new("Memory", 1).numeric(),
    ];
    let processes = [
        ("firefox", "38.4%", "2.1 GB"),
        ("Hyprland", "12.1%", "384 MB"),
        ("rust-analyzer", "9.7%", "1.8 GB"),
        ("quickshell", "4.2%", "256 MB"),
    ];
    let rows: Vec<Element<'a, Message>> = processes
        .into_iter()
        .map(|(name, cpu, memory)| {
            table_row(vec![name.to_owned(), cpu.to_owned(), memory.to_owned()])
                .selected(name == "Hyprland")
                .on_press(Message::Sample)
                .view(look, &columns)
        })
        .collect();
    table(look, &columns, rows)
}
