//! The components page's "Dialogs" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Dialogs.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use iced::Length;
use iced::widget;
use kubaos_ui::Look;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::dialog::Tone;
use kubaos_ui::component::dialog::dialog;
use kubaos_ui::component::dialog::prompt;
use kubaos_ui::component::dialog::scrim;
use kubaos_ui::component::field::State;
use kubaos_ui::component::field::field;
use kubaos_ui::component::field::labelled;
use kubaos_ui::component::field::note;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::grid;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Dialogs",
    lede: Some("A question, the confirmation before something is destroyed, and a prompt for what only you can type."),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let normal = dialog(
        look,
        Tone::Normal,
        Icon::PowerSettingsNew,
        "Restart now?",
        "Everything open closes. I'll be back in a few seconds.",
        Some(button("Cancel").kind(Kind::Text).on_press(Message::Sample).view(look)),
        [
            button("Shut down")
                .kind(Kind::Tonal)
                .icon(Icon::PowerSettingsNew)
                .on_press(Message::Sample)
                .view(look),
            button("Restart")
                .icon(Icon::RestartAlt)
                .on_press(Message::Sample)
                .focused(true)
                .view(look),
        ],
    );
    let danger = dialog(
        look,
        Tone::Danger,
        Icon::Warning,
        "Last chance",
        "Everything on Samsung SSD 990 PRO (nvme0n1, 2 TB) gets erased. There's no undo.",
        Some(
            button("Cancel")
                .kind(Kind::Text)
                .on_press(Message::Sample)
                .focused(true)
                .view(look),
        ),
        [button("Erase and install")
            .kind(Kind::Danger)
            .icon(Icon::Warning)
            .on_press(Message::Sample)
            .view(look)],
    );
    let asking = prompt(
        look,
        Icon::AdminPanelSettings,
        "Authentication required",
        "Installing btop needs your password.",
        labelled(
            look,
            "Password for Kuba",
            None,
            field("hunter", "")
                .icon(Icon::Key)
                .secret()
                .state(State::Invalid)
                .view(look),
            Some(note(look, "That didn't work. Fat fingers? Try again.", true)),
        ),
        Some(button("Cancel").kind(Kind::Text).on_press(Message::Sample).view(look)),
        [button("Authenticate").on_press(Message::Sample).view(look)],
    );
    let stage = |d: Element<'a, Message>| -> Element<'a, Message> {
        widget::container(scrim(look, d))
            .height(Length::Fixed(f32::from(Size::Column.px() / 2 + Space::S64.px())))
            .into()
    };
    grid(look, 2, Space::S24)
        .push(stage(normal))
        .push(stage(danger))
        .push(stage(asking))
        .into()
}
