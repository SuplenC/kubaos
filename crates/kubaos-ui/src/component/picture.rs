//! A picture to pick: a wallpaper in Kōbō's grid or the shell's picker.
//!
//! The picture covers the whole tile, edge to edge in the tile's own rounded shape, with no frame
//! round it, so it shows as large as the tile allows. What is picked shows by the focus tube on
//! its edge and a mark over the picture (a check, "Current"), which each use names; the name sits
//! under the picture or over its foot.
//!
//! # Owns
//!
//! - [`picture_choice`], the tile, and [`NamePlace`].
//!
//! # Not here
//!
//! - Reserving the picture's box before it loads: [`crate::component::media`], which the tile is
//!   built on.

use iced::Background;
use iced::Border;
use iced::Element;
use iced::Gradient;
use iced::Length;
use iced::Radians;
use iced::Shadow;
use iced::Theme;
use iced::alignment;
use iced::gradient::Linear;
use iced::widget;
use iced::widget::container;
use iced::widget::image;

use crate::color;
use crate::component::button::Preview;
use crate::component::chip::ChipKind;
use crate::component::chip::chip;
use crate::component::layered;
use crate::component::media::Aspect;
use crate::component::media::media_under;
use crate::focus::RingColor;
use crate::focus::focus_ring;
use crate::icon::Icon;
use crate::layout::Inset;
use crate::layout::stack;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::snap::snap;
use crate::text::line;
use crate::tokens;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Space;
use crate::tokens::Type;

/// The tile's corner: something you click.
const CORNER: Radius = Radius::Control;

/// Where a picture's name sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamePlace {
    /// Under the picture, 8 below it: a picker whose pictures are the point.
    Under,
    /// Over the picture's foot, on a scrim that darkens toward it: a grid that wants each tile
    /// no taller than its picture.
    Over,
}

/// A picture to pick, named `name`: [`picture_choice`].
#[derive(Debug)]
pub struct PictureChoice<'a, M> {
    picture: Option<image::Handle>,
    aspect: Aspect,
    name: String,
    place: NamePlace,
    picked: bool,
    mark: Option<(Icon, String)>,
    badge: Option<(Icon, String)>,
    _words: std::marker::PhantomData<&'a ()>,
    focused: bool,
    on_press: Option<M>,
    preview: Option<Preview>,
}

/// `picture` (or its placeholder until it loads) as a tile to pick, at `aspect`, named `name`
/// under it. Picked, it shows its mark and the focus tube; the keyboard on it lights the tube too,
/// cold when it is picked, since the accent already means picked.
pub fn picture_choice<'a, M>(
    picture: Option<image::Handle>,
    aspect: Aspect,
    name: impl Into<String>,
) -> PictureChoice<'a, M> {
    PictureChoice {
        picture,
        aspect,
        name: name.into(),
        place: NamePlace::Under,
        picked: false,
        mark: None,
        badge: None,
        focused: false,
        on_press: None,
        preview: None,
        _words: std::marker::PhantomData,
    }
}

impl<'a, M: Clone + 'a> PictureChoice<'a, M> {
    /// Where the name sits.
    #[must_use]
    pub fn name(self, place: NamePlace) -> Self {
        PictureChoice { place, ..self }
    }

    /// Whether it is picked.
    #[must_use]
    pub fn picked(self, picked: bool) -> Self {
        PictureChoice { picked, ..self }
    }

    /// What says it is picked, over the picture's top right corner while it is: a check, "Current".
    #[must_use]
    pub fn mark(self, glyph: Icon, label: impl Into<String>) -> Self {
        PictureChoice {
            mark: Some((glyph, label.into())),
            ..self
        }
    }

    /// A fact about the picture whatever its state, over its top left corner: "Video".
    #[must_use]
    pub fn badge(self, glyph: Icon, label: impl Into<String>) -> Self {
        PictureChoice {
            badge: Some((glyph, label.into())),
            ..self
        }
    }

    #[must_use]
    pub fn focused(self, focused: bool) -> Self {
        PictureChoice { focused, ..self }
    }

    #[must_use]
    pub fn on_press(self, message: M) -> Self {
        PictureChoice {
            on_press: Some(message),
            ..self
        }
    }

    /// Shown hovered or pressed whatever the pointer does.
    #[must_use]
    pub fn preview(self, preview: Preview) -> Self {
        PictureChoice {
            preview: Some(preview),
            ..self
        }
    }

    pub fn view(self, look: &Look) -> Element<'a, M> {
        let c = look.colors;
        let corner = radius(CORNER);
        let mut layers: Vec<Element<'a, M>> = Vec::new();
        if self.place == NamePlace::Over {
            // Over a photo the name is always light on a dark foot, whatever the mode: the
            // scrim's colour at the backdrop's opacity, from nothing 40% down the tile.
            let foot = color::alpha(c.scrim, Opacity::Backdrop.value());
            let scrim = Linear::new(Radians(std::f32::consts::PI))
                .add_stop(0.4, color::alpha(foot, 0.0))
                .add_stop(1.0, foot);
            layers.push(
                widget::container(widget::Space::new())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |_theme| container::Style {
                        background: Some(Background::Gradient(Gradient::Linear(scrim))),
                        border: Border::default().rounded(corner),
                        ..container::Style::default()
                    })
                    .into(),
            );
            let light = color::rgb(tokens::DARK.on_surface);
            layers.push(
                widget::container(snap(look.scale, line(self.name.clone(), Type::Small, light)))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Inset::all(Space::S12).padding())
                    .align_y(alignment::Vertical::Bottom)
                    .into(),
            );
        }
        if let Some((glyph, label)) = self.badge {
            layers.push(corner_chip(
                look,
                ChipKind::Neutral,
                glyph,
                label,
                alignment::Horizontal::Left,
            ));
        }
        if self.picked
            && let Some((glyph, label)) = self.mark
        {
            layers.push(corner_chip(
                look,
                ChipKind::Selected,
                glyph,
                label,
                alignment::Horizontal::Right,
            ));
        }
        let content = c.on_surface;
        let enabled = self.on_press.is_some();
        let picture = media_under(look, self.picture, self.aspect, CORNER, layers);
        let pressable = widget::button(picture)
            .padding(0)
            .width(Length::Fill)
            .on_press_maybe(self.on_press)
            .style(move |_theme: &Theme, _status| widget::button::Style {
                background: None,
                text_color: content,
                border: Border::default().rounded(corner),
                shadow: Shadow::default(),
                snap: true,
            });
        let tile = layered(pressable, CORNER, content, enabled, self.preview.map(Preview::pin));
        // The tube says picked, or where the keyboard is; on a picked tile the keyboard's is cold.
        let ring = if self.picked && self.focused {
            RingColor::Cold
        } else {
            RingColor::Accent
        };
        let ringed = focus_ring(look, tile, CORNER, self.picked || self.focused, ring);
        let mut column = stack(look, Space::S8).center().push(ringed);
        if self.place == NamePlace::Under {
            let words = if self.picked {
                c.on_surface
            } else {
                c.on_surface_variant
            };
            column = column.push(line(self.name, Type::Small, words));
        }
        let view: Element<'a, M> = column.into();
        let place = match self.place {
            NamePlace::Under => "name-under",
            NamePlace::Over => "name-over",
        };
        probe::component("PictureChoice", [place, "", ""], Fixed::HOLDS, view)
    }
}

/// A chip 8 in from the picture's top corner on `side`.
fn corner_chip<'a, M: Clone + 'a>(
    look: &Look,
    kind: ChipKind,
    glyph: Icon,
    label: String,
    side: alignment::Horizontal,
) -> Element<'a, M> {
    widget::container(chip(look, kind, label, Some(glyph), None, None))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Inset::all(Space::S8).padding())
        .align_x(side)
        .into()
}

#[cfg(test)]
mod tests;
