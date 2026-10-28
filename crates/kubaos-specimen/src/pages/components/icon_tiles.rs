//! The components page's "Icon tiles and badges" section.
//!
//! # Owns
//!
//! - Its samples, which `qml/specimen/sections/IconTiles.qml` draws with `KubaOS.Components`.
//!
//! # Not here
//!
//! - Where the section goes on the page: [`super::SECTIONS`]. The components: `kubaos-ui`.

use iced::Element;
use iced::widget;
use kubaos_ui::Look;
use kubaos_ui::Mode;
use kubaos_ui::component::avatar::TileSize;
use kubaos_ui::component::avatar::accent_badge;
use kubaos_ui::component::avatar::badge;
use kubaos_ui::component::avatar::icon_tile;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::row;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;

use super::Section;
use super::samples::PICTURES;
use crate::specimen::Message;

pub const SECTION: Section = Section {
    title: "Icon tiles and badges",
    lede: Some(
        "Every row's icon on one tile, a symbol until a picture loads, so rows read the same size: small, regular and large, an app's icon, a photo, a critical one. Then badges, a count or a dot over an icon's corner, the accent's for something unread.",
    ),
    view,
};

/// `glyph` 24 in a small control's box, `mark` over its top right corner.
fn badged<'a>(look: &Look, glyph: Icon, mark: Element<'a, Message>) -> Element<'a, Message> {
    let c = look.colors;
    let side = f32::from(Size::ControlSm.px());
    widget::Stack::with_children([
        widget::container(kubaos_ui::snap::centre(
            look.scale,
            kubaos_ui::icon::icon(look, glyph, IconSize::S24, c.on_surface_variant),
        ))
        .width(side)
        .height(side)
        .into(),
        widget::container(mark)
            .width(side)
            .height(side)
            .align_x(iced::alignment::Horizontal::Right)
            .into(),
    ])
    .into()
}

fn view<'a>(look: &Look) -> Element<'a, Message> {
    let picture = PICTURES[usize::from(look.mode == Mode::Light)].clone();
    row(look, Space::S24)
        .center()
        .push(icon_tile(Icon::Terminal, TileSize::Sm).view(look))
        .push(icon_tile(Icon::Terminal, TileSize::Md).view(look))
        .push(icon_tile(Icon::Terminal, TileSize::Lg).view(look))
        .push(
            icon_tile(Icon::Notifications, TileSize::Md)
                .photo(Some(picture))
                .view(look),
        )
        .push(icon_tile(Icon::Warning, TileSize::Md).danger().view(look))
        .push(badge(look, Some(3)))
        .push(badge(look, Some(128)))
        .push(badge(look, None))
        .push(accent_badge(look, None))
        .push(badged(look, Icon::Notifications, badge(look, Some(3))))
        .push(badged(look, Icon::SystemUpdateAlt, badge(look, None)))
        .into()
}
