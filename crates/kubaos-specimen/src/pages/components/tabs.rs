//! The components page's "Tabs" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Tabs.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::tabs::Tab;
use kubaos_ui::component::tabs::tabs;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::row;
use kubaos_ui::tokens::Space;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Tabs",
    lede: Some(
        "One of a few views or values, picked from small buttons: the picked one tonal. The same row is a setting's segmented control; the last is disabled.",
    ),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let views = [
        Tab::new("Wi-Fi"),
        Tab::new("Bluetooth"),
        Tab::new("Output"),
        Tab::new("Keyboard"),
    ];
    let tools = [Tab::new("Timer").icon(Icon::Timer), Tab::new("Alarm").icon(Icon::Alarm)];
    let motion = [Tab::new("Always"), Tab::new("On AC"), Tab::new("Never")];
    row(look, Space::S32)
        .center()
        .push(tabs(&views, 0, |_| Message::Sample).view(look))
        .push(tabs(&tools, 1, |_| Message::Sample).focused(true).view(look))
        .push(tabs(&motion, 1, |_| Message::Sample).view(look))
        .push(tabs(&motion, 0, |_| Message::Sample).disabled(true).view(look))
        .into()
}
