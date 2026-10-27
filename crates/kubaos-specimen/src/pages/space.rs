//! The space page: the spacing scale, radii, icon and component sizes, hairlines, strokes, the
//! focus ring, concentric corners and the grid's device-pixel columns.

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Length;
use iced::widget;
use iced::widget::container;
use kubaos_ui::Look;
use kubaos_ui::color;
use kubaos_ui::focus::RingColor;
use kubaos_ui::focus::focus_ring;
use kubaos_ui::grid::columns;
use kubaos_ui::icon::Icon;
use kubaos_ui::icon::icon;

use kubaos_ui::layout::Inset;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::row;
use kubaos_ui::layout::section;
use kubaos_ui::layout::split;
use kubaos_ui::layout::stack;
use kubaos_ui::look::nested;
use kubaos_ui::look::radius;
use kubaos_ui::snap::snap;
use kubaos_ui::text::text;
use kubaos_ui::tokens;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Opacity;
use kubaos_ui::tokens::Radius;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use crate::specimen::Message;

/// The concentric sample's corners: a 24 dialog holds 16 eight in, which holds 12 four in.
const IN_DIALOG: Radius = nested(Radius::R24, Space::S8);
const IN_WELL: Radius = nested(IN_DIALOG, Space::S4);

/// A plain filled box, `width` by `height` logical pixels.
fn block<'a>(width: Length, height: f32, fill: Color, corner: Radius) -> Element<'a, Message> {
    widget::container(widget::Space::new())
        .width(width)
        .height(height)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(fill)),
            border: Border::default().rounded(radius(corner)),
            ..container::Style::default()
        })
        .into()
}

/// A token's name and value, in the left column of a scale.
fn name<'a>(look: &Look, label: String, value: String) -> Element<'a, Message> {
    let c = look.colors;
    row(look, Space::S8)
        .center()
        .push(text(label, Type::BodyStrong, c.on_surface))
        .push(text(value, Type::Mono, c.on_surface_variant))
        .into()
}

pub fn view<'a>(look: &Look) -> Element<'a, Message> {
    stack(look, Space::S48)
        .push(section(look, "Spacing", Some("Gaps, paddings and insets."), spacing(look)))
        .push(section(look, "Radii", Some("Corners, and full for pills."), radii(look)))
        .push(section(look, "Icon sizes", Some("The em box; the glyph's ink sits inside its own margin."), icons(look)))
        .push(section(look, "Component sizes", None, sizes(look)))
        .push(section(
            look,
            "Hairlines, strokes and the focus ring",
            Some("A hairline is one device pixel; the thin stroke is 2 rounded to whole device pixels; the ring stands 4 outside its control and takes no layout space."),
            strokes(look),
        ))
        .push(section(
            look,
            "Concentric corners",
            Some("24 with 8 inside is 16; 16 with 4 inside is 12. The focus ring adds 4 outside."),
            nesting(look),
        ))
        .push(section(
            look,
            "Columns",
            Some("A grid splits its width in device pixels: columns differ by one device pixel at most and every edge is sharp."),
            columns_demo(look),
        ))
        .into()
}

fn spacing<'a>(look: &Look) -> Element<'a, Message> {
    let fill = look.colors.primary;
    let mut table = split(look, &[1, 3], Space::S8).center();
    for space in Space::ALL {
        table = table
            .push(name(look, format!("{space:?}"), format!("{} px", space.px())))
            .push(block(
                Length::Fixed(f32::from(space.px())),
                f32::from(Space::S16.px()),
                fill,
                Radius::R4,
            ));
    }
    table.into()
}

fn radii<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let side = f32::from(Space::S64.px());
    let fill = color::alpha(c.primary, Opacity::Tint.value());
    grid(look, 7, Space::S24)
        .extend(Radius::ALL.map(|corner| {
            let label = corner.px().map_or_else(|| "full".to_string(), |px| px.to_string());
            stack(look, Space::S8)
                .push(block(Length::Fixed(side), side, fill, corner))
                .push(text(format!("{corner:?} · {label}"), Type::Mono, c.on_surface_variant))
                .into()
        }))
        .into()
}

fn icons<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let edge = look.outline_soft();
    let hairline = look.hairline();
    row(look, Space::S32)
        .bottom()
        .extend(IconSize::ALL.map(|size| {
            let boxed =
                widget::container(icon(look, Icon::Wifi, size, c.on_surface)).style(move |_theme| container::Style {
                    border: Border {
                        color: edge,
                        width: hairline,
                        radius: 0.0.into(),
                    },
                    ..container::Style::default()
                });
            stack(look, Space::S8)
                .push(boxed)
                .push(text(format!("{}", size.px()), Type::Mono, c.on_surface_variant))
                .into()
        }))
        .into()
}

fn sizes<'a>(look: &Look) -> Element<'a, Message> {
    let fill = color::alpha(look.colors.primary, Opacity::Tint.value());
    let mut table = split(look, &[1, 3], Space::S8).center();
    for size in Size::ALL {
        table = table
            .push(name(look, format!("{size:?}"), format!("{} px", size.px())))
            .push(block(
                Length::Fixed(f32::from(size.px())),
                f32::from(Space::S16.px()),
                fill,
                Radius::R4,
            ));
    }
    table.into()
}

fn strokes<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let scale = look.scale;
    let hairline = look.hairline();
    let thin = scale.thin();
    let line = |thickness: f32, fill: Color| {
        widget::container(widget::Space::new())
            .width(Length::Fill)
            .height(thickness)
            .style(move |_theme| container::Style::default().background(fill))
    };
    let target = widget::container(snap(look.scale, text("Focused", Type::Body, c.on_surface)))
        .padding(Inset::xy(Space::S16, Space::S8).padding())
        .style(move |_theme| container::Style {
            background: Some(Background::Color(c.surface_container_high)),
            border: Border::default().rounded(radius(Radius::R12)),
            ..container::Style::default()
        });
    let rings = row(look, Space::S32)
        .push(focus_ring(look, target, Radius::R12, true, RingColor::Accent))
        .push(focus_ring(
            look,
            widget::container(text("Picked card", Type::Body, c.on_surface))
                .padding(Inset::xy(Space::S16, Space::S8).padding())
                .style(move |_theme| container::Style {
                    background: Some(Background::Color(c.secondary_container)),
                    border: Border::default().rounded(radius(Radius::R16)),
                    ..container::Style::default()
                }),
            Radius::R16,
            true,
            RingColor::Cold,
        ));
    split(look, &[1, 3], Space::S24)
        .center()
        .push(name(
            look,
            "Hairline".into(),
            format!("{} device px = {:.3} logical", tokens::HAIRLINE_DEVICE_PX, hairline),
        ))
        .push(line(hairline, c.outline))
        .push(name(
            look,
            "Thin stroke".into(),
            format!("{} device px = {:.3} logical", scale.to_device(thin), thin),
        ))
        .push(line(thin, c.primary))
        .push(name(
            look,
            "Focus ring".into(),
            format!("offset {}", tokens::FOCUS_OFFSET),
        ))
        .push(widget::container(rings).padding(Inset::all(Space::S8).padding()))
        .into()
}

fn nesting<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let shape = |content: Element<'a, Message>, corner: Radius, inset: Space, fill: Color| -> Element<'a, Message> {
        let content = snap(look.scale, content);
        widget::container(content)
            .padding(Inset::all(inset).padding())
            .style(move |_theme| container::Style {
                background: Some(Background::Color(fill)),
                border: Border::default().rounded(radius(corner)),
                ..container::Style::default()
            })
            .into()
    };
    let (r16, r12) = (IN_DIALOG, IN_WELL);
    let core = widget::container(snap(look.scale, text("12", Type::Mono, c.on_primary)))
        .padding(Inset::xy(Space::S32, Space::S12).padding())
        .style(move |_theme| container::Style {
            background: Some(Background::Color(c.primary)),
            border: Border::default().rounded(radius(r12)),
            ..container::Style::default()
        });
    let middle = shape(core.into(), r16, Space::S4, c.surface_container_highest);
    let outer = shape(middle, Radius::R24, Space::S8, c.surface_container_high);
    row(look, Space::S32)
        .center()
        .push(outer)
        .push(text(
            format!("Radius::R24 → {r16:?} → {r12:?}"),
            Type::Mono,
            c.on_surface_variant,
        ))
        .into()
}

fn columns_demo<'a>(look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let fill = color::alpha(c.primary, Opacity::Tint.value());
    let cells = (1..=6).map(|i| -> Element<'a, Message> {
        widget::container(text(format!("{i}"), Type::Mono, c.on_surface))
            .padding(Inset::all(Space::S8).padding())
            .width(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(Background::Color(fill)),
                border: Border::default().rounded(radius(Radius::R8)),
                ..container::Style::default()
            })
            .into()
    });
    // What the columns come to in device pixels at this scale, for the page's width of 1200.
    let widths: Vec<String> = columns(1200.0, 7, Space::S16, look.scale)
        .iter()
        .map(|span| format!("{}", look.scale.to_device(span.width)))
        .collect();
    stack(look, Space::S12)
        .push(grid(look, 7, Space::S16).extend(cells.chain(std::iter::once(Element::from(widget::Space::new())))))
        .push(text(
            format!(
                "Seven columns of 1200 at {}×: {} device px",
                look.scale.factor(),
                widths.join(", ")
            ),
            Type::Mono,
            c.on_surface_variant,
        ))
        .into()
}
