//! Progress: a bar for work that knows how far it is, a spinner for work that doesn't.

use std::f32::consts::PI;

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Length;
use iced::Point;
use iced::Radians;
use iced::Rectangle;
use iced::Renderer;
use iced::Shadow;
use iced::Theme;
use iced::Vector;
use iced::mouse;
use iced::widget;
use iced::widget::canvas;
use iced::widget::canvas::Path;
use iced::widget::canvas::Stroke;
use iced::widget::container;

use crate::color;
use crate::layout::Gap;
use crate::layout::row;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::tokens::IconSize;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Shadow as ShadowToken;
use crate::tokens::Size;

/// How finely the bar's fill is split: a ten-thousandth of the bar.
const PARTS: u16 = 10_000;

/// A bar filled to `fraction` (0 to 1), the fill glowing in its own colour.
pub fn bar<'a, M: 'a>(look: &Look, fraction: f32) -> Element<'a, M> {
    let c = look.colors;
    let height = f32::from(Size::Progress.px());
    let filled = (fraction.clamp(0.0, 1.0) * f32::from(PARTS)).round() as u16;
    let fill = c.primary;
    let track = color::alpha(c.surface_container_highest, Opacity::Track.value());
    let mut parts = row(look, Gap::NONE).height(crate::layout::Extent::Fixed(Size::Progress));
    if filled > 0 {
        parts = parts.push(
            widget::container(widget::Space::new())
                .width(Length::FillPortion(filled))
                .height(height)
                .style(move |_theme| container::Style {
                    background: Some(Background::Color(fill)),
                    border: Border::default().rounded(radius(Radius::Full)),
                    shadow: Shadow {
                        color: color::alpha(fill, ShadowToken::Glow.opacity()),
                        offset: Vector::new(0.0, f32::from(ShadowToken::Glow.y())),
                        blur_radius: f32::from(ShadowToken::Glow.blur()),
                    },
                    ..container::Style::default()
                }),
        );
    }
    if filled < PARTS {
        parts = parts.push(widget::Space::new().width(Length::FillPortion(PARTS - filled)));
    }
    let view: Element<'a, M> = widget::container(parts)
        .width(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(track)),
            border: Border::default().rounded(radius(Radius::Full)),
            ..container::Style::default()
        })
        .into();
    probe::component("ProgressBar", ["", "", ""], Fixed::HEIGHT, view)
}

/// A spinner `size` across, `seconds` into its turn: an arc that turns and breathes over a faint
/// full ring, so its shortest arc still reads as a spinner.
pub fn spinner<'a, M: 'a>(look: &Look, size: IconSize, seconds: f32, color: Color) -> Element<'a, M> {
    let variant = [size_name(size), "", ""];
    let px = f32::from(size.px());
    let view: Element<'a, M> = canvas(Spinner {
        seconds,
        color,
        width: look.scale.thin(),
    })
    .width(px)
    .height(px)
    .into();
    probe::component("Spinner", variant, Fixed::BOTH, view)
}

struct Spinner {
    seconds: f32,
    color: Color,
    width: f32,
}

impl<M> canvas::Program<M> for Spinner {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let r = bounds.width.min(bounds.height) / 2.0 - self.width;
        let centre = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        frame.stroke(
            &Path::circle(centre, r),
            Stroke::default()
                .with_width(self.width)
                .with_color(color::alpha(self.color, Opacity::SpinnerTrack.value())),
        );
        // Material's spinner: 0.8 turns a second, the arc breathing between a quarter and 0.85 of
        // a half turn.
        let turn = self.seconds * 2.0 * PI * 0.8;
        let sweep = PI * (0.45 + 0.6 * (0.5 + 0.5 * (self.seconds * PI * 1.2).sin()));
        let arc = Path::new(|b| {
            b.arc(canvas::path::Arc {
                center: centre,
                radius: r,
                start_angle: Radians(turn),
                end_angle: Radians(turn + sweep),
            });
        });
        frame.stroke(
            &arc,
            Stroke::default()
                .with_width(self.width)
                .with_color(self.color)
                .with_line_cap(canvas::LineCap::Round),
        );
        vec![frame.into_geometry()]
    }
}

/// A spinner's size as its variant's name.
fn size_name(size: IconSize) -> &'static str {
    match size {
        IconSize::S16 => "16",
        IconSize::S20 => "20",
        IconSize::S24 => "24",
        IconSize::S32 => "32",
        IconSize::S48 => "48",
    }
}
