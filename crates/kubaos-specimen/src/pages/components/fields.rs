//! The components page's "Fields" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Fields.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::button::ButtonSize;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::field::Caret;
use kubaos_ui::component::field::State;
use kubaos_ui::component::field::field;
use kubaos_ui::component::field::labelled;
use kubaos_ui::component::field::note;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::grid;
use kubaos_ui::tokens::Space;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Fields",
    lede: Some("As tall as a regular button. The label's row fits a small button, so fields side by side line up."),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let show = button::<Message>("Show")
        .kind(Kind::Text)
        .size(ButtonSize::Sm)
        .icon(Icon::Visibility)
        .on_press(Message::Sample)
        .view(look);
    grid(look, 3, Space::S24)
        .push(labelled(
            look,
            "Full name",
            None,
            field("", "Your name").view(look),
            None,
        ))
        .push(labelled(
            look,
            "Hostname",
            None,
            field("tuxedo", "").icon(Icon::Computer).view(look),
            Some(note(look, "Letters, digits and dashes.", false)),
        ))
        .push(labelled(
            look,
            "Search",
            None,
            field("Shinj", "Search")
                .icon(Icon::Search)
                .caret(Caret::at("Shinj".chars().count()))
                .view(look),
            Some(note(look, "Focused: the caret, and the tube on its own edge.", false)),
        ))
        .push(labelled(
            look,
            "Password",
            Some(show),
            field("hunter2", "").secret().view(look),
            None,
        ))
        .push(labelled(
            look,
            "Email for git",
            None,
            field("not-an-email", "")
                .icon(Icon::Mail)
                .state(State::Invalid)
                .view(look),
            Some(note(look, "That's not an email. Typo?", true)),
        ))
        .push(labelled(
            look,
            "User name",
            None,
            field("kuba", "").icon(Icon::Lock).state(State::Locked).view(look),
            Some(note(look, "Set when you installed; read-only here.", false)),
        ))
        .push(labelled(
            look,
            "Proxy",
            None,
            field("", "proxy.example.com:3128")
                .icon(Icon::Lan)
                .state(State::Disabled)
                .view(look),
            Some(note(look, "Disabled: turn the proxy on to set it.", false)),
        ))
        .into()
}
