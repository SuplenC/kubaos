//! The icons page: KubaOS Symbols, every one at the sizes Kōbō draws them with the stroke each
//! scale gets, then the comparison sheet that picked Tabler as their base, kept as a record: the same
//! 40 glyphs from Material Symbols Rounded, Phosphor, Lucide and Tabler, side by side and in context.

use iced::Background;
use iced::Border;
use iced::Element;
use iced::Font;
use iced::Length;
use iced::alignment;
use iced::widget;
use iced::widget::container;
use kubaos_ui::Look;
use kubaos_ui::Scale;
use kubaos_ui::icon::Icon;
use kubaos_ui::icon::device_stroke;
use kubaos_ui::icon::icon;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::fill;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::row;
use kubaos_ui::layout::section;
use kubaos_ui::layout::split;
use kubaos_ui::layout::stack;
use kubaos_ui::look::radius;
use kubaos_ui::text::text;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Radius;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use crate::specimen::Message;

/// The four sets' fonts, cut by `assets/make-icons.sh`.
pub const FONTS: [&[u8]; 4] = [
    include_bytes!("../../assets/icons/material.ttf"),
    include_bytes!("../../assets/icons/phosphor.ttf"),
    include_bytes!("../../assets/icons/lucide.ttf"),
    include_bytes!("../../assets/icons/tabler.ttf"),
];

/// Each glyph's codepoint in each set, from `assets/make-icons.sh`.
const TABLE: &str = include_str!("../../assets/icons/icons.tsv");

/// A set: its name as shown, its font, and its stroke at 24 px.
struct Set {
    name: &'static str,
    font: Font,
    note: &'static str,
}

const SETS: [Set; 4] = [
    Set {
        name: "Material Symbols Rounded",
        font: Font::with_name("Specimen Material Symbols"),
        note: "Kōbō's today: weight 400, 24 optical size, outlined.",
    },
    Set {
        name: "Phosphor",
        font: Font::with_name("Specimen Phosphor"),
        note: "Regular weight: a 1.5 px stroke at 24.",
    },
    Set {
        name: "Lucide",
        font: Font::with_name("Specimen Lucide"),
        note: "A 2 px stroke at 24, round caps.",
    },
    Set {
        name: "Tabler",
        font: Font::with_name("Specimen Tabler"),
        note: "The 2 px stroke at 24, round caps.",
    },
];

/// A glyph's id and its character in each set.
struct Glyph {
    id: &'static str,
    chars: [char; 4],
}

fn glyphs() -> Vec<Glyph> {
    TABLE
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let id = fields.next()?;
            let mut chars = ['\u{fffd}'; 4];
            for slot in &mut chars {
                *slot = u32::from_str_radix(fields.next()?, 16).ok().and_then(char::from_u32)?;
            }
            Some(Glyph { id, chars })
        })
        .collect()
}

/// The sizes Kōbō draws icons at, on the grid.
const SIZES: [IconSize; 3] = [IconSize::S16, IconSize::S20, IconSize::S24];

/// The character `ch` of a comparison set's font, centred in a square box of `size`.
fn glyph<'a>(ch: char, font: Font, size: IconSize, color: iced::Color) -> Element<'a, Message> {
    let px = f32::from(size.px());
    widget::text(ch.to_string())
        .font(font)
        .size(px)
        .line_height(widget::text::LineHeight::Absolute(px.into()))
        .width(Length::Fixed(px))
        .height(Length::Fixed(px))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .color(color)
        .into()
}

pub fn view<'a>(look: &Look) -> Element<'a, Message> {
    stack(look, Space::S48)
        .push(section(
            look,
            "KubaOS Symbols",
            Some(
                "Decided: Tabler's outline icons are the base. Each symbol is named for the Material \
                 Symbol it replaces; the four Tabler lacks are drawn on its grid, with its stroke and \
                 caps, and marked ours.",
            ),
            symbols(look),
        ))
        .push(section(
            look,
            "Drawn by us",
            Some("The four Tabler lacks, at 48 beside the Tabler symbols they borrow from: same grid, stroke, caps and badges."),
            drawn(look),
        ))
        .push(section(
            look,
            "The stroke at each size",
            Some(
                "2 units on the 24 grid, scaled to the size and rounded half up to whole device \
                 pixels, so no symbol has a soft line. Device pixels:",
            ),
            strokes(look),
        ))
        .push(comparison(look))
        .into()
}

/// Every symbol at 16, 20 and 24, with its name, and its Tabler name or "ours".
fn symbols<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    grid(look, 4, Space::S16)
        .extend(Icon::ALL.into_iter().map(|glyph| {
            let source = glyph.tabler().unwrap_or("ours");
            let tone = if glyph.tabler().is_some() {
                c.on_surface_variant
            } else {
                c.cold
            };
            let mut sizes = row(look, Space::S8).bottom();
            for size in SIZES {
                sizes = sizes.push(icon(look, glyph, size, c.on_surface));
            }
            row(look, Space::S12)
                .center()
                .push(sizes)
                .push(
                    stack(look, Gap::NONE)
                        .push(text(glyph.name(), Type::Mono, c.on_surface))
                        .push(text(source, Type::Mono, tone)),
                )
                .into()
        }))
        .into()
}

/// Each of our drawings at 48, then the Tabler symbols it borrows its parts from.
fn drawn<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let families = [
        (Icon::HardDrive, [Icon::Storage, Icon::DeployedCode]),
        (Icon::WifiLock, [Icon::Wifi, Icon::Lock]),
        (Icon::WifiFind, [Icon::Wifi, Icon::Search]),
        (Icon::BluetoothSearching, [Icon::Bluetooth, Icon::Search]),
    ];
    grid(look, 4, Space::S24)
        .extend(families.into_iter().map(|(ours, relatives)| {
            let mut line = row(look, Space::S16).push(icon(look, ours, IconSize::S48, c.on_surface));
            for relative in relatives {
                line = line.push(icon(look, relative, IconSize::S48, c.on_surface_variant));
            }
            stack(look, Space::S8)
                .push(line)
                .push(text(ours.name(), Type::Mono, c.cold))
                .into()
        }))
        .into()
}

/// The device pixels of the stroke at every size and supported scale.
fn strokes<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let mut table = split(look, &[1, 1, 1, 1, 1], Space::S8)
        .push(text("size", Type::Label, c.on_surface_variant))
        .extend(
            Scale::SUPPORTED
                .map(|scale| text(format!("{}×", scale.factor()), Type::Label, c.on_surface_variant).into()),
        );
    for size in IconSize::ALL {
        table = table.push(text(format!("{}", size.px()), Type::Mono, c.on_surface));
        for scale in Scale::SUPPORTED {
            table = table.push(text(
                format!("{}", device_stroke(size, scale)),
                Type::Mono,
                c.on_surface,
            ));
        }
    }
    widget::container(table)
        .width(f32::from(kubaos_ui::tokens::Size::Column.px()))
        .into()
}

/// The comparison that picked Tabler, as Kuba saw it.
fn comparison<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let glyphs = glyphs();
    let half = glyphs.len().div_ceil(2);
    let column = |part: &[Glyph]| -> Element<'a, Message> {
        let mut table = split(look, &[2, 3, 3, 3, 3], Space::S12)
            .center()
            .push(text("", Type::Label, c.on_surface_variant))
            .extend(SETS.iter().map(|set| {
                text(
                    set.name.split(' ').next().unwrap_or(set.name),
                    Type::Label,
                    c.on_surface_variant,
                )
                .into()
            }));
        for g in part {
            table = table.push(text(g.id, Type::Mono, c.on_surface_variant));
            for (set, &ch) in SETS.iter().zip(&g.chars) {
                let mut sizes = row(look, Space::S8).bottom();
                for size in SIZES {
                    sizes = sizes.push(glyph(ch, set.font, size, c.on_surface));
                }
                table = table.push(sizes);
            }
        }
        table.into()
    };
    let sheet = grid(look, 2, Space::S48)
        .push(column(&glyphs[..half]))
        .push(column(&glyphs[half..]));
    let notes = grid(look, 4, Space::S24).extend(SETS.iter().map(|set| {
        stack(look, Space::S4)
            .push(text(set.name, Type::BodyStrong, c.on_surface))
            .push(text(set.note, Type::Small, c.on_surface_variant))
            .into()
    }));
    let big = grid(look, 4, Space::S24).extend(SETS.iter().enumerate().map(|(i, set)| {
        let mut wall = grid(look, 5, Space::S12);
        for g in glyphs.iter().take(10) {
            wall = wall.push(glyph(g.chars[i], set.font, IconSize::S32, c.on_surface));
        }
        stack(look, Space::S8)
            .push(text(set.name, Type::Label, c.on_surface_variant))
            .push(wall)
            .into()
    }));
    let context = grid(look, 4, Space::S24).extend((0..SETS.len()).map(|i| in_context(look, &glyphs, i)));
    stack(look, Space::S48)
        .push(section(
            look,
            "The comparison, a record",
            Some(
                "The sheet the base was picked from: each set at 16, 20 and 24 from its own font, in \
                 on_surface. Ids are Kōbō's and the shell's uses. Tabler was picked.",
            ),
            stack(look, Space::S24).push(notes).push(sheet),
        ))
        .push(section(
            look,
            "At 32",
            Some("The first ten, big, to judge the drawing itself."),
            big,
        ))
        .push(section(
            look,
            "In context",
            Some("The same few glyphs where Kōbō puts them: a list row, the footer's button, a chip, an error."),
            context,
        ))
        .into()
}

fn find<'g>(glyphs: &'g [Glyph], id: &str) -> Option<&'g Glyph> {
    glyphs.iter().find(|g| g.id == id)
}

/// Kōbō's places for icons, drawn with set `i`: a Wi-Fi row with its lock, the Continue button, a
/// demo chip and an error line.
fn in_context<'a>(look: &Look, glyphs: &[Glyph], i: usize) -> Element<'a, Message> {
    let c = look.colors;
    let set = &SETS[i];
    let ch = |id: &str| find(glyphs, id).map_or('\u{fffd}', |g| g.chars[i]);
    let wifi_row = widget::container(
        row(look, Space::S12)
            .center()
            .push(glyph(ch("wifi"), set.font, IconSize::S20, c.on_secondary_container))
            .push(text("Home-5G", Type::BodyStrong, c.on_secondary_container))
            .push(fill())
            .push(glyph(ch("lock"), set.font, IconSize::S16, c.on_secondary_container)),
    )
    .height(f32::from(Size::Row.px()))
    .center_y(f32::from(Size::Row.px()))
    .padding(Inset::x(Space::S12).padding())
    .style(move |_theme| container::Style {
        background: Some(Background::Color(c.secondary_container)),
        border: Border::default().rounded(radius(Radius::R12)),
        ..container::Style::default()
    });
    let next = widget::container(
        row(look, Space::S8)
            .center()
            .push(text("Continue", Type::Button, c.on_primary))
            .push(glyph(ch("forward"), set.font, IconSize::S20, c.on_primary)),
    )
    .height(f32::from(Size::ControlMd.px()))
    .center_y(f32::from(Size::ControlMd.px()))
    .padding(Inset::x(Space::S24).padding())
    .style(move |_theme| container::Style {
        background: Some(Background::Color(c.primary)),
        border: Border::default().rounded(radius(Radius::Full)),
        ..container::Style::default()
    });
    let error = row(look, Space::S8)
        .center()
        .push(glyph(ch("error"), set.font, IconSize::S16, c.error))
        .push(text("Doesn't match.", Type::Small, c.error));
    let chrome = row(look, Space::S16)
        .center()
        .push(glyph(ch("power"), set.font, IconSize::S20, c.on_surface_variant))
        .push(glyph(ch("bolt"), set.font, IconSize::S16, c.cold))
        .push(glyph(ch("keyboard"), set.font, IconSize::S20, c.on_surface_variant))
        .push(glyph(ch("settings"), set.font, IconSize::S20, c.on_surface_variant));
    stack(look, Space::S16)
        .push(text(set.name, Type::Label, c.on_surface_variant))
        .push(wifi_row)
        .push(row(look, Gap::NONE).push(next))
        .push(error)
        .push(chrome)
        .into()
}

#[cfg(test)]
mod tests;
