//! Progress: a bar for work that knows how far it is, a spinner for work that doesn't.

use std::f32::consts::PI;
use std::time::Instant;

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Event;
use iced::Length;
use iced::Point;
use iced::Radians;
use iced::Rectangle;
use iced::Renderer;
use iced::Shadow;
use iced::Size as IcedSize;
use iced::Theme;
use iced::Vector;
use iced::advanced::Clipboard;
use iced::advanced::Layout;
use iced::advanced::Renderer as _;
use iced::advanced::Shell;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::renderer;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree;
use iced::mouse;
use iced::widget::canvas;
use iced::widget::canvas::Path;
use iced::widget::canvas::Stroke;

use crate::color;
use crate::look::Look;
use crate::look::radius;
use crate::motion::Tween;
use crate::probe;
use crate::probe::Align;
use crate::probe::Axis;
use crate::probe::Fixed;
use crate::probe::Tag;
use crate::scale::Scale;
use crate::tokens::Duration;
use crate::tokens::Easing;
use crate::tokens::IconSize;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Shadow as ShadowToken;
use crate::tokens::Size;

/// A bar filled to `fraction` (0 to 1), the fill glowing in its own colour. The fill follows the
/// work over [`Duration::Short`] on [`Easing::Linear`], so it moves with the work instead of
/// jumping; a bar that shows up already part full starts there.
pub fn bar<'a, M: 'a>(look: &Look, fraction: f32) -> Element<'a, M> {
    drawn(look, fraction, true)
}

/// [`bar`] for a fraction the app already times against the clock (how long a held button has been
/// held): drawn where it is given, with no easing of its own to trail behind it.
pub fn timed_bar<'a, M: 'a>(look: &Look, fraction: f32) -> Element<'a, M> {
    drawn(look, fraction, false)
}

fn drawn<'a, M: 'a>(look: &Look, fraction: f32, follows: bool) -> Element<'a, M> {
    let c = look.colors;
    let fill = Bar {
        fraction: fraction.clamp(0.0, 1.0),
        follows,
        fill: c.primary,
        track: color::alpha(c.surface_container_highest, Opacity::Track.value()),
        scale: look.scale,
    };
    // The fill's row, as the geometry check has always seen it: the shell's bar reports the same.
    let parts = probe::probe(
        Tag::Layout {
            axis: Axis::X,
            gap: 0.0,
            align: Align::Start,
        },
        Element::new(fill),
    );
    probe::component("ProgressBar", ["", "", ""], Fixed::HEIGHT, parts)
}

struct Bar {
    fraction: f32,
    /// Whether the fill eases to a new fraction, or is drawn where it is given.
    follows: bool,
    fill: Color,
    track: Color,
    scale: Scale,
}

/// How far the fill shows, on its way to the work's fraction.
struct Shown(Tween);

/// `shown` on its way to `fraction` from `now`, or there at once when the bar doesn't follow.
fn move_to(shown: &mut Tween, fraction: f32, follows: bool, now: Instant) {
    if follows {
        shown.to(fraction, now);
    } else {
        *shown = Tween::new(fraction, Duration::Short, Easing::Linear);
    }
}

impl<M> Widget<M, Theme, Renderer> for Bar {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Shown>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Shown(Tween::new(self.fraction, Duration::Short, Easing::Linear)))
    }

    fn diff(&self, tree: &mut Tree) {
        move_to(
            &mut tree.state.downcast_mut::<Shown>().0,
            self.fraction,
            self.follows,
            Instant::now(),
        );
    }

    fn size(&self) -> IcedSize<Length> {
        IcedSize::new(Length::Fill, Length::Fixed(f32::from(Size::Progress.px())))
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.resolve(
            Length::Fill,
            Length::Fixed(f32::from(Size::Progress.px())),
            IcedSize::ZERO,
        ))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        _viewport: &Rectangle,
    ) {
        if let Event::Window(iced::window::Event::RedrawRequested(now)) = event
            && tree.state.downcast_ref::<Shown>().0.moving(*now)
        {
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let pill = Border::default().rounded(radius(Radius::Full));
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: pill,
                snap: true,
                ..renderer::Quad::default()
            },
            Background::Color(self.track),
        );
        let shown = tree
            .state
            .downcast_ref::<Shown>()
            .0
            .value(Instant::now())
            .clamp(0.0, 1.0);
        let width = self.scale.snap_up(bounds.width * shown).min(bounds.width);
        if width <= 0.0 {
            return;
        }
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle { width, ..bounds },
                border: pill,
                shadow: Shadow {
                    color: color::alpha(self.fill, ShadowToken::Glow.opacity()),
                    offset: Vector::new(0.0, f32::from(ShadowToken::Glow.y())),
                    blur_radius: f32::from(ShadowToken::Glow.blur()),
                },
                snap: true,
            },
            Background::Color(self.fill),
        );
    }
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

#[cfg(test)]
mod tests;
