//! Sheets: a panel that grows out of an edge of the screen frame, as every shell panel does.
//!
//! The sheet and the frame are one surface: where the sheet meets the frame's edge it curves into
//! it through concave fillets of the surface radius, the frame's own inner corner, and its far
//! corners are the surface radius. In the shell the sheet slides out from under the frame as it
//! opens (`Sheet` in `KubaOS.Components`); here it is drawn open, hanging from the edge above it.
//!
//! # Owns
//!
//! - [`sheet`], an open sheet under the frame's top edge.
//!
//! # Not here
//!
//! - The frame itself and the sliding: the shell's `modules/frame/Frame.qml` and `Sheet.qml`.

use std::f32::consts::FRAC_PI_2;
use std::f32::consts::PI;

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Theme;
use iced::alignment;
use iced::border;
use iced::mouse;
use iced::widget;
use iced::widget::canvas;
use iced::widget::canvas::Path;
use iced::widget::canvas::path::Arc;
use iced::widget::container;

use crate::color;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::tokens::Opacity;
use crate::tokens::Radius;

/// The sheet's far corners and its fillets into the frame: it holds content, and the frame's inner
/// corner is the surface radius too.
const CORNER: Radius = Radius::Surface;

/// `content` on an open sheet, the frame's surface, hanging from the edge above it: the fillets
/// curve into that edge on either side, and the far corners round. The sheet is as large as its
/// content, which keeps its own padding.
pub fn sheet<'a, M: 'a>(look: &Look, content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    let surface = surface(look);
    let corner = radius(CORNER);
    let body = widget::container(content).style(move |_theme| container::Style {
        background: Some(Background::Color(surface)),
        border: Border {
            radius: border::Radius::default().bottom(corner),
            ..Border::default()
        },
        ..container::Style::default()
    });
    let body = probe::component("Sheet", ["", "", ""], Fixed::HOLDS, body);
    widget::row![fillet(surface, Side::Start), body, fillet(surface, Side::End)]
        .align_y(alignment::Vertical::Top)
        .into()
}

/// The frame's colour, which a sheet shares: the surface, at the frame's glass.
#[must_use]
pub fn surface(look: &Look) -> Color {
    color::alpha(look.colors.surface, Opacity::Frame.value())
}

/// Which side of the sheet a fillet is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Start,
    End,
}

/// The concave corner where the sheet meets the edge above it on `side`: the square of the
/// surface radius less the quarter circle round its outer foot.
fn fillet<'a, M: 'a>(colour: Color, side: Side) -> Element<'a, M> {
    let r = radius(CORNER);
    canvas(Fillet { colour, side }).width(r).height(r).into()
}

struct Fillet {
    colour: Color,
    side: Side,
}

impl<M> canvas::Program<M> for Fillet {
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
        let r = bounds.width;
        // The square's corner on the sheet's side, then round the circle centred on the far foot,
        // back to the edge.
        let (near, far, from, to) = match self.side {
            Side::Start => (r, 0.0, 0.0, -FRAC_PI_2),
            Side::End => (0.0, r, PI, PI + FRAC_PI_2),
        };
        let shape = Path::new(|b| {
            b.move_to(Point::new(far, 0.0));
            b.line_to(Point::new(near, 0.0));
            b.line_to(Point::new(near, r));
            b.arc(Arc {
                center: Point::new(far, r),
                radius: r,
                start_angle: iced::Radians(from),
                end_angle: iced::Radians(to),
            });
            b.close();
        });
        frame.fill(&shape, self.colour);
        vec![frame.into_geometry()]
    }
}
