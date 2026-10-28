//! The components page's "Toasts and progress" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/Progress.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use kubaos_ui::Look;
use kubaos_ui::component::progress::bar;
use kubaos_ui::component::progress::spinner;
use kubaos_ui::component::toast::toast;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::fill;
use kubaos_ui::layout::row;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::Section;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Toasts and progress",
    lede: None,
    view,
};

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let toasts = row(look, Space::S16)
        .center()
        .push(toast(look, "Copied. Evidence secured.", None))
        .push(toast(look, "Replay saved.", Some(Icon::CheckCircle)));
    let bars = stack(look, Space::S16)
        .push(bar(look, 0.0))
        .push(bar(look, 0.42))
        .push(bar(look, 1.0));
    let spinners = row(look, Space::S24)
        .center()
        .extend(
            IconSize::ALL
                .into_iter()
                .take(3)
                .map(|size| spinner(look, size, 0.7, c.primary)),
        )
        .push(text("Poking at the hardware…", Type::Body, c.on_surface_variant));
    let install = stack(look, Space::S12)
        .push(
            row(look, Space::S12)
                .bottom()
                .push(text("Installing", Type::Title, c.on_surface))
                .push(fill())
                .push(text("42%", Type::HeroNumber, c.on_surface)),
        )
        .push(bar(look, 0.42))
        .push(
            row(look, Space::S12)
                .center()
                .push(text("Unpacking the system", Type::BodyStrong, c.on_surface))
                .push(text("1.8 / 4.2 GB", Type::Mono, c.primary))
                .push(fill())
                .push(text("01:42", Type::Mono, c.on_surface_variant)),
        );
    stack(look, Space::S32)
        .push(toasts)
        .push(
            row(look, Space::S48)
                .push(stack(look, Space::S16).width(Extent::Fill).push(bars).push(spinners))
                .push(stack(look, Gap::NONE).width(Extent::Fill).push(install)),
        )
        .into()
}
