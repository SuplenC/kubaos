//! The on/off switch.

use iced::Color;
use iced::Element;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Size as IcedSize;
use iced::Theme;
use iced::mouse;
use iced::widget;
use iced::widget::canvas;
use iced::widget::canvas::Frame;
use iced::widget::canvas::Geometry;
use iced::widget::canvas::Path;
use iced::widget::canvas::Stroke;

use crate::color;
use crate::color::Colors;
use crate::focus::RingColor;
use crate::focus::focus_ring;
use crate::look::Look;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Size;

/// A switch, `on` or off, `progress` (0 to 1) of the way to that side while it moves. Space,
/// Enter or a click on it sends `on_toggle`; without one it is disabled.
pub fn switch<'a, M: Clone + 'a>(
    look: &Look,
    on: bool,
    progress: f32,
    on_toggle: Option<M>,
    focused: bool,
) -> Element<'a, M> {
    let t = if on { progress } else { 1.0 - progress };
    let knob = canvas(Knob {
        t: t.clamp(0.0, 1.0),
        enabled: on_toggle.is_some(),
        colors: look.colors,
        hairline: look.hairline(),
    })
    .width(f32::from(Size::SwitchWidth.px()))
    .height(f32::from(Size::SwitchHeight.px()));
    let pressable = widget::button(knob)
        .padding(0)
        .on_press_maybe(on_toggle)
        .style(|_theme: &Theme, _status| widget::button::Style {
            background: None,
            ..widget::button::Style::default()
        });
    focus_ring(look, pressable, Radius::Full, focused, RingColor::Accent)
}

struct Knob {
    /// 0 off, 1 on, between while it moves.
    t: f32,
    enabled: bool,
    colors: Colors,
    hairline: f32,
}

impl<M> canvas::Program<M> for Knob {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let colors = &self.colors;
        let on = self.t;
        let opacity = if self.enabled {
            1.0
        } else {
            Opacity::DisabledContent.value()
        };
        let (width, height) = (bounds.width, bounds.height);
        let mut frame = Frame::new(renderer, bounds.size());
        let mut track = color::mix(colors.surface_container_highest, colors.primary, on);
        if self.enabled && cursor.is_over(bounds) {
            track = color::mix(track, colors.on_surface, Opacity::Hover.value());
        }
        frame.fill(
            &Path::rounded_rectangle(Point::ORIGIN, bounds.size(), (height / 2.0).into()),
            color::alpha(track, opacity),
        );
        if on < 1.0 {
            // A hairline inside the track's edge, centred on the outermost device pixel.
            let inset = self.hairline / 2.0;
            let outline = Path::rounded_rectangle(
                Point::new(inset, inset),
                IcedSize::new(width - self.hairline, height - self.hairline),
                ((height - self.hairline) / 2.0).into(),
            );
            frame.stroke(
                &outline,
                Stroke::default()
                    .with_width(self.hairline)
                    .with_color(color::alpha(colors.outline, opacity * (1.0 - on))),
            );
        }
        let (off_px, on_px) = (f32::from(Size::SwitchKnobOff.px()), f32::from(Size::SwitchKnobOn.px()));
        let diameter = off_px + (on_px - off_px) * on;
        let centre = Point::new(height / 2.0 + (width - height) * on, height / 2.0);
        let knob: Color = color::mix(colors.outline, colors.on_primary, on);
        frame.fill(&Path::circle(centre, diameter / 2.0), color::alpha(knob, opacity));
        vec![frame.into_geometry()]
    }
}
