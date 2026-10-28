//! Skeletons: the shape of what is loading, held in its place until it arrives.
//!
//! A list being scanned or a picture being decoded shows its skeleton, in the placeholder colour a
//! [`media`](crate::component::media) box holds until its picture loads, so what arrives takes the
//! room already kept for it and moves nothing.
//!
//! # Owns
//!
//! - [`skeleton`] and its [`Bone`]s.
//!
//! # Not here
//!
//! - A picture's own placeholder: [`crate::component::media`].

use iced::Background;
use iced::Border;
use iced::Element;
use iced::Length;
use iced::widget;
use iced::widget::container;

use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::tokens::IconSize;
use crate::tokens::Radius;
use crate::tokens::Size;
use crate::tokens::Type;

/// What a skeleton stands in for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bone {
    /// A line of text in a style: as wide as its room, the style's line tall, the small radius.
    Line(Type),
    /// A symbol or a tile: an icon size's circle.
    Circle(IconSize),
    /// A box: as wide as its room, a size tall, at a radius role.
    Block(Size, Radius),
}

impl Bone {
    fn word(self) -> &'static str {
        match self {
            Bone::Line(_) => "line",
            Bone::Circle(_) => "circle",
            Bone::Block(..) => "block",
        }
    }
}

/// A skeleton of `bone`, in the placeholder colour, the high container.
pub fn skeleton<'a, M: 'a>(look: &Look, bone: Bone) -> Element<'a, M> {
    let (width, height, corner) = match bone {
        Bone::Line(style) => (Length::Fill, f32::from(style.line()), Radius::Small),
        Bone::Circle(across) => {
            let side = f32::from(across.px());
            (Length::Fixed(side), side, Radius::Full)
        }
        Bone::Block(size, corner) => (Length::Fill, f32::from(size.px()), corner),
    };
    let fill = look.colors.surface_container_high;
    let shape = widget::container(widget::Space::new())
        .width(width)
        .height(height)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(fill)),
            border: Border::default().rounded(radius(corner)),
            ..container::Style::default()
        });
    probe::component("Skeleton", [bone.word(), "", ""], Fixed::NONE, shape)
}
