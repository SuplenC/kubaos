//! The colour page: every role as a swatch, and the pairs text is drawn in with their contrast.

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Length;
use iced::widget;
use iced::widget::container;
use kubaos_ui::Look;
use kubaos_ui::color;
use kubaos_ui::color::Colors;
use kubaos_ui::color::Contrast;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::row;
use kubaos_ui::layout::section;
use kubaos_ui::layout::split;
use kubaos_ui::layout::stack;
use kubaos_ui::look::radius;
use kubaos_ui::text::text;
use kubaos_ui::tokens;
use kubaos_ui::tokens::Opacity;
use kubaos_ui::tokens::Radius;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use crate::specimen::Message;

/// Every role, in the order a reader looks for them.
fn roles(c: &Colors) -> Vec<(&'static str, Color)> {
    vec![
        ("primary", c.primary),
        ("on_primary", c.on_primary),
        ("primary_container", c.primary_container),
        ("on_primary_container", c.on_primary_container),
        ("secondary", c.secondary),
        ("on_secondary", c.on_secondary),
        ("secondary_container", c.secondary_container),
        ("on_secondary_container", c.on_secondary_container),
        ("cold", c.cold),
        ("on_cold", c.on_cold),
        ("cold_container", c.cold_container),
        ("on_cold_container", c.on_cold_container),
        ("error", c.error),
        ("on_error", c.on_error),
        ("error_container", c.error_container),
        ("on_error_container", c.on_error_container),
        ("warning", c.warning),
        ("success", c.success),
        ("sharing", c.sharing),
        ("microphone", c.microphone),
        ("camera", c.camera),
        ("surface", c.surface),
        ("on_surface", c.on_surface),
        ("on_surface_variant", c.on_surface_variant),
        ("surface_container_lowest", c.surface_container_lowest),
        ("surface_container_low", c.surface_container_low),
        ("surface_container", c.surface_container),
        ("surface_container_high", c.surface_container_high),
        ("surface_container_highest", c.surface_container_highest),
        ("surface_bright", c.surface_bright),
        ("outline", c.outline),
        ("outline_variant", c.outline_variant),
        ("inverse_surface", c.inverse_surface),
        ("inverse_on_surface", c.inverse_on_surface),
        ("inverse_primary", c.inverse_primary),
        ("scrim", c.scrim),
    ]
}

fn hex(c: Color) -> String {
    let [r, g, b, _] = c.into_rgba8();
    format!("#{r:02x}{g:02x}{b:02x}")
}

pub fn view<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let hairline = look.hairline();
    let edge = look.outline_soft();
    let swatches = grid(look, 4, Space::S16).extend(roles(&c).into_iter().map(|(name, colour)| {
        let chip = widget::container(widget::Space::new())
            .width(Length::Fill)
            .height(f32::from(Space::S48.px()))
            .style(move |_theme| container::Style {
                background: Some(Background::Color(colour)),
                border: Border {
                    color: edge,
                    width: hairline,
                    radius: radius(Radius::Small).into(),
                },
                ..container::Style::default()
            });
        stack(look, Space::S8)
            .push(chip)
            .push(
                stack(look, Gap::NONE)
                    .push(text(name, Type::Label, c.on_surface))
                    .push(text(hex(colour), Type::Mono, c.on_surface_variant)),
            )
            .into()
    }));
    let mut table = split(look, &[1, 2, 2], Space::S8).center();
    for pair in tokens::CONTRAST {
        let bg = pair.bg.of(&c);
        let fg = color::alpha(pair.fg.of(&c), pair.opacity.map_or(1.0, Opacity::value));
        let need = Contrast::from(pair.need);
        let ratio = color::pair_ratio(&pair, &c);
        let at = pair
            .opacity
            .map_or_else(String::new, |opacity| format!(" at {}", opacity.name()));
        let (fg_name, bg_name) = (pair.fg.name(), pair.bg.name());
        let verdict = if ratio >= need.minimum() {
            format!("{ratio:.1}:1, needs {}:1", need.minimum())
        } else {
            format!("{ratio:.1}:1, FAILS {}:1", need.minimum())
        };
        let sample = widget::container(text("Aa Kōbō", Type::BodyStrong, fg))
            .padding(Inset::xy(Space::S12, Space::S4).padding())
            .style(move |_theme| container::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    color: edge,
                    width: hairline,
                    radius: radius(Radius::Small).into(),
                },
                ..container::Style::default()
            });
        table = table
            .push(row(look, Gap::NONE).push(sample))
            .push(text(format!("{fg_name}{at} on {bg_name}"), Type::Body, c.on_surface))
            .push(text(verdict, Type::Mono, c.on_surface_variant));
    }
    stack(look, Space::S48)
        .push(section(
            look,
            "Roles",
            Some("cold is the sign's blue: the wallpaper's complement, used rarely."),
            swatches,
        ))
        .push(section(
            look,
            "Pairs",
            Some("Every pair the tokens list, each at the opacity it is drawn at. WCAG 2.1 AA: 4.5:1 for text, 3:1 for large text, outlines and icons; the tokens refuse a pair under its minimum."),
            table,
        ))
        .into()
}
