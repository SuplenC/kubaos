//! The components page's "Steps" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Steps.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::list::list;
use kubaos_ui::component::step::StepState;
use kubaos_ui::component::step::step;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::split;
use kubaos_ui::layout::stack;
use kubaos_ui::tokens::Space;

use crate::specimen::Message;
use super::Section;

pub const SECTION: Section = Section {
    title: "Steps",
    lede: Some("To do, done and waiting on an action each look their own, and the action carries the accent. A description wraps; a rail marks the state instead."),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let reboot = StepState::Action {
        label: "Reboot",
        glyph: Icon::RestartAlt,
    };
    let checklist = vec![
        step("TUXEDO drivers")
            .subtitle("Key queued. Reboot and enroll it on the blue MokManager screen, before KubaOS starts.")
            .icon(Icon::Computer)
            .state(reboot)
            .selected(true)
            .on_press_maybe(Some(Message::Sample))
            .view(look),
        step("Fingerprint")
            .subtitle("One finger: right index. sudo and password dialogs take it.")
            .icon(Icon::Fingerprint)
            .state(StepState::Done)
            .on_press_maybe(Some(Message::Sample))
            .view(look),
        step("Bitwarden")
            .subtitle("Not logged in.")
            .icon(Icon::Lock)
            .on_press_maybe(Some(Message::Sample))
            .view(look),
    ];
    let rail = stack(look, Gap::NONE)
        .push(
            step("Language")
                .icon(Icon::Language)
                .state(StepState::Done)
                .mark()
                .on_press_maybe(Some(Message::Sample))
                .view(look),
        )
        .push(
            step("Disk")
                .icon(Icon::HardDrive)
                .state(StepState::Current)
                .mark()
                .view(look),
        )
        .push(step("Git").icon(Icon::Code).mark().view(look))
        .push(step("Reboot").icon(Icon::Computer).state(reboot).mark().view(look));
    split(look, &[2, 1], Space::S24)
        .push(list(look, checklist, true))
        .push(rail)
        .into()
}
