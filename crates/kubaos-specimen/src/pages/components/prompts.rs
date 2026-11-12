//! The components page's "Prompts" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Prompts.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::field::Caret;
use kubaos_ui::component::field::State;
use kubaos_ui::component::field::field;
use kubaos_ui::component::field::note;
use kubaos_ui::component::menu::menu;
use kubaos_ui::component::menu::menu_item;
use kubaos_ui::component::menu::menu_separator;
use kubaos_ui::component::menu::popover;
use kubaos_ui::component::prompt::prompt;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::stack;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Prompts",
    lede: Some(
        "One value asked for in a popover, a menu wide: its label and Enter, the field with all of it selected, its note. It floats under the text it changes, or sits inside a menu.",
    ),
    view,
};

/// The value the samples ask about.
const NAME: &str = "Coffee";

const RULE: &str = "every day at 9";

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let rename = prompt("Rename", field(NAME, "").caret(Caret::all(NAME)).view(look)).view(look);
    let invalid = prompt(
        "Set value",
        field("3.5", "")
            .state(State::Invalid)
            .caret(Caret::at("3.5".chars().count()))
            .view(look),
    )
    .note(note(look, "A whole number, 0 or more.", true))
    .view(look);
    let rule = prompt("When it resets", field(RULE, "").caret(Caret::all(RULE)).view(look))
        .note(note(look, "Resets every day at 9:00.", false))
        .in_menu()
        .view(look);
    let auto = popover(
        look,
        Extent::Fixed(Size::Menu),
        stack(look, Gap::NONE)
            .width(Extent::Fill)
            .push(menu(
                look,
                [menu_item("Back")
                    .icon(Icon::ArrowBack)
                    .on_press(Message::Sample)
                    .view(look)],
            ))
            .push(menu_separator(look))
            .push(rule)
            .push(menu_separator(look))
            .push(menu(
                look,
                [
                    menu_item("Never").checked(true).on_press(Message::Sample).view(look),
                    menu_item("When it reaches")
                        .checked(false)
                        .on_press(Message::Sample)
                        .view(look),
                ],
            )),
    );
    grid(look, 3, Space::S24).push(rename).push(invalid).push(auto).into()
}
