//! Icon tiles and badges: what an app or a sender looks like in a row, and what is new about it.
//!
//! An icon tile is a symbol or a picture on a tinted rounded square of one size, so a list of
//! apps reads as one set whatever each vendor drew, and a row with no picture keeps the room a
//! picture would take. A badge is a count, or a dot, over an icon's corner.
//!
//! # Owns
//!
//! - [`icon_tile`], built up with [`IconTile`], its [`TileSize`], and [`badge`].
//!
//! # Not here
//!
//! - A picture in a box of an aspect ratio: [`crate::component::media`].

use iced::Background;
use iced::Border;
use iced::ContentFit;
use iced::Element;
use iced::Length;
use iced::widget;
use iced::widget::container;
use iced::widget::image;

use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::Inset;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::snap::centre;
use crate::snap::centre_x;
use crate::snap::snap;
use crate::text::text;
use crate::tokens;
use crate::tokens::IconSize;
use crate::tokens::Radius;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// An icon tile's corner: the control's, the shape of what it sits in.
const CORNER: Radius = Radius::Control;

/// How big an icon tile is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileSize {
    /// 32, a small control: beside one line.
    Sm,
    /// 40, a regular control: beside a title over a subtitle.
    Md,
    /// 48, a large control: the one thing a view is about, over its `headline` name (a network
    /// or a device being connected).
    Lg,
}

impl TileSize {
    pub const ALL: [TileSize; 3] = [TileSize::Sm, TileSize::Md, TileSize::Lg];

    /// The size's name in the spec.
    pub fn name(self) -> &'static str {
        match self {
            TileSize::Sm => "Sm",
            TileSize::Md => "Md",
            TileSize::Lg => "Lg",
        }
    }

    /// The tile's side.
    pub fn side(self) -> Size {
        match self {
            TileSize::Sm => Size::ControlSm,
            TileSize::Md => Size::ControlMd,
            TileSize::Lg => Size::ControlLg,
        }
    }

    /// The symbol or app icon inside it: the icon of the text it sits beside.
    pub fn icon(self) -> IconSize {
        match self {
            TileSize::Sm => Type::Body.icon(),
            TileSize::Md => tokens::TWO_LINE_ICON,
            TileSize::Lg => Type::Headline.icon(),
        }
    }
}

/// What a tile shows.
#[derive(Debug, Clone)]
enum Face {
    /// An app's icon, inset to the icon size, so its drawing keeps its own margins.
    Icon(image::Handle),
    /// A photo (a notification's image, a person), covering the whole tile.
    Photo(image::Handle),
}

/// An icon tile showing `glyph` until it has a picture, built up then drawn with
/// [`IconTile::view`].
#[derive(Debug, Clone)]
pub struct IconTile {
    glyph: Icon,
    size: TileSize,
    face: Option<Face>,
    danger: bool,
    accent: bool,
}

/// A `size` tile showing `glyph` on the highest container, which reads on a list, a card and a
/// selected row alike: an Item's symbol, or the placeholder
/// a row shows where it has no picture of its own.
#[must_use]
pub fn icon_tile(glyph: Icon, size: TileSize) -> IconTile {
    IconTile {
        glyph,
        size,
        face: None,
        danger: false,
        accent: false,
    }
}

impl IconTile {
    /// An app's icon in place of the symbol, inset to the icon size; the symbol stays while there
    /// is none.
    #[must_use]
    pub fn picture(self, picture: Option<image::Handle>) -> Self {
        IconTile {
            face: picture.map(Face::Icon).or(self.face),
            ..self
        }
    }

    /// A photo in place of the symbol, covering the tile; the symbol stays while there is none.
    #[must_use]
    pub fn photo(self, photo: Option<image::Handle>) -> Self {
        IconTile {
            face: photo.map(Face::Photo).or(self.face),
            ..self
        }
    }

    /// On the error colour: a critical notification's.
    #[must_use]
    pub fn danger(self) -> Self {
        IconTile { danger: true, ..self }
    }

    /// Its symbol in the accent: the one of a list that comes first, as the default keyboard
    /// layout's star.
    #[must_use]
    pub fn accent(self) -> Self {
        IconTile { accent: true, ..self }
    }

    pub fn view<'a, M: 'a>(self, look: &Look) -> Element<'a, M> {
        let c = look.colors;
        let (tint, ink) = if self.danger {
            (c.error, c.on_error)
        } else if self.accent {
            (c.surface_container_highest, c.primary)
        } else {
            (c.surface_container_highest, c.on_surface_variant)
        };
        let side = f32::from(self.size.side().px());
        let glyph = f32::from(self.size.icon().px());
        let corner = radius(CORNER);
        let inside: Element<'a, M> = match self.face {
            None => centre(look.scale, icon(look, self.glyph, self.size.icon(), ink)),
            Some(Face::Icon(handle)) => centre(
                look.scale,
                widget::image(handle)
                    .width(glyph)
                    .height(glyph)
                    .content_fit(ContentFit::Contain),
            ),
            Some(Face::Photo(handle)) => widget::image(handle)
                .content_fit(ContentFit::Cover)
                .width(Length::Fill)
                .height(Length::Fill)
                .border_radius(corner)
                .into(),
        };
        let tile = widget::container(inside)
            .width(side)
            .height(side)
            .style(move |_theme| container::Style {
                background: Some(Background::Color(tint)),
                border: Border::default().rounded(corner),
                ..container::Style::default()
            });
        probe::component("IconTile", [self.size.name(), "", ""], Fixed::BOTH, tile)
    }
}

/// The most a badge counts to; past it, it reads as this and a plus.
pub const MOST: u32 = 99;

/// A badge: `count` in a pill a line of label text tall, or a dot when there is no count, in the
/// error colour so it reads over any icon. One digit makes a circle; more pad 4 either side.
pub fn badge<'a, M: 'a>(look: &Look, count: Option<u32>) -> Element<'a, M> {
    toned_badge(look, count, false)
}

/// A [`badge`] in the accent: something unread rather than something wrong, as an unread
/// notification's dot on its tile.
pub fn accent_badge<'a, M: 'a>(look: &Look, count: Option<u32>) -> Element<'a, M> {
    toned_badge(look, count, true)
}

fn toned_badge<'a, M: 'a>(look: &Look, count: Option<u32>, accent: bool) -> Element<'a, M> {
    let c = look.colors;
    let (fill, ink) = if accent { (c.primary, c.on_primary) } else { (c.error, c.on_error) };
    let tone = if accent { "accent" } else { "" };
    let pill_style = move |_theme: &iced::Theme| container::Style {
        background: Some(Background::Color(fill)),
        border: Border::default().rounded(radius(Radius::Full)),
        ..container::Style::default()
    };
    let Some(n) = count else {
        let side = f32::from(Size::BadgeDot.px());
        let dot = widget::container(widget::Space::new())
            .width(side)
            .height(side)
            .style(pill_style);
        return probe::component("Badge", ["dot", tone, ""], Fixed::HEIGHT, dot);
    };
    let words = if n > MOST { format!("{MOST}+") } else { n.to_string() };
    let height = f32::from(Size::Badge.px());
    let label = snap(look.scale, text(words, Type::Label, ink));
    let pill = if n < 10 {
        widget::container(centre_x(look.scale, label)).width(height)
    } else {
        widget::container(label).padding(Inset::x(Space::S4).padding())
    };
    let view = pill.height(height).style(pill_style);
    probe::component("Badge", ["count", tone, ""], Fixed::HEIGHT, view)
}

#[cfg(test)]
mod tests;
