//! Checkboxes and radio buttons.

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Length;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Theme;
use iced::mouse;
use iced::widget;
use iced::widget::canvas;
use iced::widget::container;
use iced::widget::text::IntoFragment;

use crate::color;
use crate::focus::RingColor;
use crate::focus::focus_ring;
use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::row;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::scale::Scale;
use crate::snap::centre;
use crate::text::text;
use crate::tokens::IconSize;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// How much of something is picked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tick {
    Empty,
    /// Some of a group: a dash.
    Partial,
    Full,
}

impl Tick {
    /// `picked` out of `total`. An empty group reads as unpicked, not as fully picked.
    pub fn of(picked: u32, total: u32) -> Tick {
        match picked {
            0 => Tick::Empty,
            n if n >= total => Tick::Full,
            _ => Tick::Partial,
        }
    }
}

/// A checkbox reading `label`. Space or a click sends `on_toggle`; without one it is disabled.
pub fn checkbox<'a, M: Clone + 'a>(
    look: &Look,
    label: impl IntoFragment<'a>,
    tick: Tick,
    on_toggle: Option<M>,
    focused: bool,
) -> Element<'a, M> {
    let c = look.colors;
    let enabled = on_toggle.is_some();
    let dim = move |color: Color| {
        if enabled {
            color
        } else {
            color::alpha(color, Opacity::DisabledContent.value())
        }
    };
    let (fill, edge) = match tick {
        Tick::Empty => (c.surface_container_highest, Some(c.outline)),
        Tick::Partial | Tick::Full => (c.primary, None),
    };
    let mark: Element<'a, M> = match tick {
        Tick::Empty => widget::Space::new().into(),
        Tick::Full => icon(look, Icon::Check, IconSize::S16, dim(c.on_primary)),
        Tick::Partial => canvas(Dash {
            color: dim(c.on_primary),
            scale: look.scale,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
    };
    let side = f32::from(Size::Check.px());
    let hairline = look.hairline();
    let shape = widget::container(centre(look.scale, mark))
        .width(side)
        .height(side)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(dim(fill))),
            border: Border {
                color: edge.map_or(Color::TRANSPARENT, dim),
                width: if edge.is_some() { hairline } else { 0.0 },
                radius: radius(Radius::Small).into(),
            },
            ..container::Style::default()
        });
    let view: Element<'a, M> = labelled(look, shape, Radius::Small, label, on_toggle, focused);
    probe::component("Checkbox", ["", "", ""], Fixed::HEIGHT, view)
}

/// A radio button reading `label`, one of a group of which exactly one is picked.
pub fn radio<'a, M: Clone + 'a>(
    look: &Look,
    label: impl IntoFragment<'a>,
    picked: bool,
    on_pick: Option<M>,
    focused: bool,
) -> Element<'a, M> {
    let c = look.colors;
    let enabled = on_pick.is_some();
    let dim = move |color: Color| {
        if enabled {
            color
        } else {
            color::alpha(color, Opacity::DisabledContent.value())
        }
    };
    let dot: Element<'a, M> = if picked {
        let on_primary = dim(c.on_primary);
        widget::container(widget::Space::new())
            .width(f32::from(Size::RadioDot.px()))
            .height(f32::from(Size::RadioDot.px()))
            .style(move |_theme| container::Style {
                background: Some(Background::Color(on_primary)),
                border: Border::default().rounded(radius(Radius::Full)),
                ..container::Style::default()
            })
            .into()
    } else {
        widget::Space::new().into()
    };
    let (fill, edge) = if picked {
        (c.primary, None)
    } else {
        (c.surface_container_highest, Some(c.outline))
    };
    let hairline = look.hairline();
    let side = f32::from(Size::Check.px());
    let shape = widget::container(centre(look.scale, dot))
        .width(side)
        .height(side)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(dim(fill))),
            border: Border {
                color: edge.map_or(Color::TRANSPARENT, dim),
                width: if edge.is_some() { hairline } else { 0.0 },
                radius: radius(Radius::Full).into(),
            },
            ..container::Style::default()
        });
    let view: Element<'a, M> = labelled(look, shape, Radius::Full, label, on_pick, focused);
    probe::component("Radio", ["", "", ""], Fixed::HEIGHT, view)
}

/// `shape` with the focus tube, then `label`, all one press target.
fn labelled<'a, M: Clone + 'a>(
    look: &Look,
    shape: impl Into<Element<'a, M>>,
    shape_radius: Radius,
    label: impl IntoFragment<'a>,
    on_press: Option<M>,
    focused: bool,
) -> Element<'a, M> {
    let c = look.colors;
    let colour = if on_press.is_some() {
        c.on_surface
    } else {
        color::alpha(c.on_surface, Opacity::DisabledContent.value())
    };
    let content = row(look, Space::S12)
        .center()
        .push(focus_ring(look, shape, shape_radius, focused, RingColor::Accent))
        .push(text(label, Type::Body, colour));
    widget::button(content)
        .padding(0)
        .on_press_maybe(on_press)
        .style(|_theme: &Theme, _status| widget::button::Style {
            background: None,
            ..widget::button::Style::default()
        })
        .into()
}

/// A partial tick's dash: two thirds of the box wide, a thin stroke tall, on whole device pixels.
struct Dash {
    color: Color,
    scale: Scale,
}

impl<M> canvas::Program<M> for Dash {
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
        let thick = self.scale.thin();
        let width = f32::from(Size::Check.px() - 2 * Space::S4.px());
        let x = (bounds.width - width) / 2.0;
        let y = self.scale.snap((bounds.height - thick) / 2.0);
        frame.fill_rectangle(Point::new(x, y), iced::Size::new(width, thick), self.color);
        vec![frame.into_geometry()]
    }
}

#[cfg(test)]
mod tests;
