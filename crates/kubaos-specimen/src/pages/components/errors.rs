//! The components page's "Errors" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Errors.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::error::Level;
use kubaos_ui::component::error::error_block;
use kubaos_ui::component::error::log;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::fill;
use kubaos_ui::layout::row;
use kubaos_ui::layout::stack;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Errors",
    lede: Some("What broke, what to do next, and the log behind it."),
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let lines = [
        (Level::Step, "==> Bootloader"),
        (Level::Info, "Installing for x86_64-efi platform."),
        (Level::Debug, "efibootmgr -c -d /dev/nvme0n1 -p 1 -L ubuntu"),
        (
            Level::Warn,
            "warning: the firmware reports Secure Boot's boot order as locked",
        ),
        (
            Level::Error,
            "grub-install: error: efibootmgr failed to register the boot entry: Operation not permitted.",
        ),
    ];
    stack(look, Space::S16)
        .push(error_block(
            look,
            "efibootmgr failed to register the boot entry: Operation not permitted.",
            "The firmware refused a new boot entry. Turn off Secure Boot's \"lock boot order\" in the setup menu and try again.",
        ))
        .push(error_block(
            look,
            "grub-install --target=x86_64-efi --efi-directory=/boot/efi --bootloader-id=ubuntu failed with exit status 1.",
            "A message this long drops to body size.",
        ))
        .push(log(look, &lines, Extent::Fixed(Size::Log)))
        .push(
            row(look, Space::S12)
                .center()
                .push(button("Back to the review").kind(Kind::Text).icon(Icon::ArrowBack).on_press(Message::Sample).view(look))
                .push(button("Hide log").kind(Kind::Text).icon(Icon::Terminal).on_press(Message::Sample).view(look))
                .push(fill())
                .push(button("Save log").kind(Kind::Tonal).icon(Icon::Save).on_press(Message::Sample).view(look))
                .push(button("Try again").icon(Icon::RestartAlt).on_press(Message::Sample).focused(true).view(look)),
        )
        .into()
}
