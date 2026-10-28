//! The type page: every style, its numbers, its line box, and the icon that sits beside it.

use iced::Element;
use iced::widget;
use iced::widget::container;
use kubaos_ui::Look;
use kubaos_ui::color;
use kubaos_ui::icon::Icon;
use kubaos_ui::icon::icon;
use kubaos_ui::layout::row;
use kubaos_ui::layout::section;
use kubaos_ui::layout::split;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Family;
use kubaos_ui::tokens::Opacity;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;
use kubaos_ui::tokens::Weight;

use crate::specimen::Message;

/// What each style is shown saying: a line it would really carry.
fn sample(style: Type) -> &'static str {
    match style {
        Type::Clock => "21:47",
        Type::Hero => "Let's fix this shit.",
        Type::HeroNumber => "42%  09:41",
        Type::Display => "Done in 1m 42s.",
        Type::Title => "Your machine. Your rules.",
        Type::Headline => "Restart now?",
        Type::Heading => "Wi-Fi",
        Type::BodyLarge => "Whatever's on here, it's had its chance.",
        Type::Subheading => "The disk is in use, so nothing got written.",
        Type::ButtonLarge => "Install",
        Type::Button => "Continue",
        Type::Body => "Installing. Go stare at a wall, it'll be done before you get bored.",
        Type::BodyStrong => "Samsung SSD 990 PRO",
        Type::MonoLarge => "kubaos update --dry-run",
        Type::Accent => "書体見本 ようこそ",
        Type::ButtonSmall => "Show",
        Type::Label => "Full name",
        Type::LabelNumber => "62%  100%",
        Type::Mono => "/dev/nvme0n1 · 2 TB · 00:42",
        Type::Small => "At least 8 characters. Longer is better.",
        Type::Caption => "Kōbō · Shinjuku 0.1 · what a key does",
        Type::Keycap => "Ctrl",
    }
}

fn weight(w: Weight) -> &'static str {
    match w {
        Weight::Light => "light",
        Weight::Regular => "regular",
        Weight::Medium => "medium",
        Weight::Semibold => "semibold",
    }
}

pub fn view<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    // The line box, tinted, so its height shows against the grid.
    let line_box = color::alpha(c.primary, Opacity::Tint.value());
    let mut styles = split(look, &[1, 3], Space::S24);
    for style in Type::ALL {
        let meta = format!(
            "{} / {} · icon {} · {} · {}",
            style.size(),
            style.line(),
            style.icon().px(),
            weight(style.weight()),
            style.family().name()
        );
        styles = styles
            .push(
                stack(look, Space::S4)
                    .push(text(format!("{style:?}"), Type::BodyStrong, c.on_surface))
                    .push(text(meta, Type::Mono, c.on_surface_variant)),
            )
            .push(
                row(look, Space::S8)
                    .center()
                    .push(icon(look, Icon::Star, style.icon(), c.on_surface))
                    .push(
                        widget::container(text(sample(style), style, c.on_surface))
                            .style(move |_theme| container::Style::default().background(line_box)),
                    ),
            );
    }
    let mut families = split(look, &[1, 3], Space::S24).center();
    for (family, style, sample) in [
        (
            Family::Sans,
            Type::BodyLarge,
            "Aa Bb Cc 0123456789 — Kōbō, Shinjuku, Ångström",
        ),
        (Family::Mono, Type::MonoLarge, "0O 1lI {}[]() => != -> 0x2a fn main()"),
        (Family::Jp, Type::Accent, "ようこそ 壁紙 署名 確認 完了 失敗"),
    ] {
        families = families
            .push(text(family.name(), Type::BodyStrong, c.on_surface))
            .push(text(sample, style, c.on_surface));
    }
    stack(look, Space::S48)
        .push(section(
            look,
            "Styles",
            Some("Largest first. The tint is each style's line box: its height is the line height. The symbol before it is the icon size that sits beside the style."),
            styles,
        ))
        .push(section(look, "Families", None, families))
        .into()
}
