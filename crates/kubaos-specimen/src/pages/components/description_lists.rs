//! The components page's "Description lists" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/DescriptionLists.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::card::card;
use kubaos_ui::component::description::Fact;
use kubaos_ui::component::description::description_list;
use kubaos_ui::layout::grid;
use kubaos_ui::tokens::Space;

use crate::specimen::Message;
use super::Section;

pub const SECTION: Section = Section {
    title: "Description lists",
    lede: Some("Facts beside their labels, a quarter of the width; a path or an email gets a mono line of its own."),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    grid(look, 2, Space::S24)
        .push(card(
            look,
            description_list(
                look,
                [
                    Fact::new("Name", "Kuba Example"),
                    Fact::new("Login", "kuba@tuxedo"),
                    Fact::new("Git", "Kuba Example").detail("kuba@example.com"),
                ],
            ),
        ))
        .push(card(
            look,
            description_list(
                look,
                [
                    Fact::new("Disk", "Samsung SSD 990 PRO 2TB").detail("/dev/nvme0n1 · 2.0 TB"),
                    Fact::new("Mode", "Erase everything"),
                    Fact::new("Encryption", "LUKS2, account password"),
                ],
            ),
        ))
        .into()
}
