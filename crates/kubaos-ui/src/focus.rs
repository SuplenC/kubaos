//! The neon focus: the keyboard's place on screen, the focused control's own edge lit as a neon
//! tube with its glow outside it.
//!
//! The tube is drawn on the control's edge, inside its box and over its outline and all it holds,
//! so there is only ever one line and nothing dark between the tube and the glow. The glow fades
//! out from the tube over [`tokens::FOCUS_NEAR`] and, faintly, [`tokens::FOCUS_FAR`]. Neither
//! takes layout space: focus moving never moves anything. Focus is a state of the control, drawn
//! as part of it: the glow is always its full size, clipped by whatever clips the control (a
//! scroll, a list, a well) and covered by whatever is drawn over the control.
//!
//! # Owns
//!
//! - [`focus_ring`] and [`focus_ring_on_fill`], the glow's opacity out from the tube ([`glow`],
//!   [`rings`]), and how the tube lights ([`Light`]): it strikes on as focus arrives, as the neon
//!   sign does, unless the stutter is off or motion is reduced, when it fades in over
//!   [`Duration::Short`]; it fades out over the same as focus leaves.
//!
//! # Not here
//!
//! - Which control has focus: the app, which moves it with Tab and the arrows.
//! - The stutter and reduce-motion settings: [`Motion`], which the app puts in its [`Look`].

use std::slice;
use std::time::Instant;

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Event;
use iced::Length;
use iced::Rectangle;
use iced::Renderer;
use iced::Size;
use iced::Theme;
use iced::Vector;
use iced::advanced::Clipboard;
use iced::advanced::Layout;
use iced::advanced::Renderer as _;
use iced::advanced::Shell;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::Operation;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree;

use crate::color;
use crate::look::Corner;
use crate::look::Look;
use crate::look::outset;
use crate::look::radius;
use crate::motion::Motion;
use crate::scale::Scale;
use crate::snap::snapped;
use crate::tokens;
use crate::tokens::Duration;
use crate::tokens::Easing;

/// Which colour the tube takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingColor {
    /// The accent: every control.
    Accent,
    /// The cold accent, on cards where the accent already means picked.
    Cold,
    /// The error colour, on a control whose value is wrong, so focus never hides that.
    Error,
}

/// `content`, a control whose corners have radius `radius`, with its edge lit as the focus tube
/// when `focused`.
pub fn focus_ring<'a, M: 'a>(
    look: &Look,
    content: impl Into<Element<'a, M>>,
    radius: impl Into<Corner>,
    focused: bool,
    color: RingColor,
) -> Element<'a, M> {
    ring(look, content.into(), radius.into(), focused, color, None)
}

/// [`focus_ring`] for a control filled with the focus colour (a `Filled` button, a tile that is
/// on), whose own edge can't show a tube of that colour: the fill is the tube, and the tube's hot
/// core (`primary_core`, `cold_core`) rides just outside it, [`Scale::thin`] wide, with the glow
/// beyond.
pub fn focus_ring_on_fill<'a, M: 'a>(
    look: &Look,
    content: impl Into<Element<'a, M>>,
    radius: impl Into<Corner>,
    focused: bool,
    color: RingColor,
) -> Element<'a, M> {
    let core = match color {
        RingColor::Accent => look.colors.primary_core,
        RingColor::Cold => look.colors.cold_core,
        RingColor::Error => look.colors.error,
    };
    ring(look, content.into(), radius.into(), focused, color, Some(core))
}

fn ring<'a, M: 'a>(
    look: &Look,
    content: Element<'a, M>,
    radius: Corner,
    focused: bool,
    color: RingColor,
    core: Option<Color>,
) -> Element<'a, M> {
    let tint = match color {
        RingColor::Accent => look.colors.primary,
        RingColor::Cold => look.colors.cold,
        RingColor::Error => look.colors.error,
    };
    Element::new(Ring {
        content,
        color: tint,
        core,
        focused,
        radius,
        tube: look.scale.stroke(tokens::FOCUS_TUBE),
        thin: look.scale.thin(),
        scale: look.scale,
        motion: look.motion,
    })
}

/// The glow's opacity `d` logical pixels outside the tube: the near glow and the far one, each
/// fading from its opacity at the tube to nothing at its reach.
pub fn glow(d: f32) -> f32 {
    let fade = |reach: u16, opacity: f32| {
        let reach = f32::from(reach);
        if d >= reach {
            return 0.0;
        }
        opacity * (1.0 - d / reach).powi(2)
    };
    let near = fade(tokens::FOCUS_NEAR, tokens::FOCUS_NEAR_OPACITY);
    let far = fade(tokens::FOCUS_FAR, tokens::FOCUS_FAR_OPACITY);
    1.0 - (1.0 - near) * (1.0 - far)
}

/// The glow at `scale`, as bands that all start at the control's edge
/// and reach a device pixel further out each: each band's reach, in logical pixels outside the
/// control, and its opacity. Laid over one another they make [`glow`] at the middle of every device
/// pixel out. Bands, not rings side by side, so no two edges meet along a curve: each one's outer
/// edge is smoothed once, with every wider band whole under it, and the corners show no seams.
pub fn rings(scale: Scale) -> impl Iterator<Item = (f32, f32)> {
    let hairline = scale.hairline();
    let count = (f32::from(tokens::FOCUS_FAR) * scale.factor() - 1e-3).ceil().max(0.0) as u16;
    let at = move |i: u16| {
        if i >= count {
            return 0.0;
        }
        glow(hairline * (f32::from(i) + 0.5))
    };
    (0..count).map(move |i| {
        // Every band out to this one covers this device pixel, so this one adds what takes the
        // glow from the next pixel out's to its own.
        let opacity = 1.0 - (1.0 - at(i)) / (1.0 - at(i + 1));
        (hairline * (f32::from(i) + 1.0), opacity)
    })
}

/// How brightly the tube and its glow show, 0 to 1, as focus comes and goes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Light {
    focused: bool,
    /// When focus last came or went.
    at: Instant,
    /// The brightness then, which a fade starts from.
    from: f32,
    /// Whether focus arriving strikes the tube on, rather than fading it in.
    strikes: bool,
}

impl Light {
    /// At rest, lit when `focused`: a control that shows focused has its tube at once, since
    /// nothing moved.
    pub fn new(focused: bool, now: Instant) -> Light {
        Light {
            focused,
            at: now,
            from: if focused { 1.0 } else { 0.0 },
            strikes: false,
        }
    }

    /// Focus came or went at `now`; the same again changes nothing, so a view can give it on
    /// every frame.
    pub fn to(&mut self, focused: bool, motion: Motion, now: Instant) {
        if focused == self.focused {
            return;
        }
        self.from = self.value(motion, now);
        self.focused = focused;
        self.at = now;
        self.strikes = focused && motion.strikes();
    }

    /// The brightness at `now`.
    pub fn value(&self, motion: Motion, now: Instant) -> f32 {
        let elapsed = now.saturating_duration_since(self.at);
        if self.focused && self.strikes {
            return motion.strike(elapsed);
        }
        let t = motion.progress(Duration::Short, Easing::Standard, elapsed);
        let to = if self.focused { 1.0 } else { 0.0 };
        self.from + (to - self.from) * t
    }

    /// Whether it is still changing at `now`.
    pub fn moving(&self, motion: Motion, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(self.at);
        if self.focused && self.strikes {
            return elapsed < motion.duration(Duration::Medium);
        }
        let to = if self.focused { 1.0 } else { 0.0 };
        elapsed < motion.duration(Duration::Short) && (to - self.from).abs() > f32::EPSILON
    }
}

struct Ring<'a, M> {
    content: Element<'a, M>,
    color: Color,
    /// On a control filled with the focus colour: the core's colour, drawn just outside the fill.
    core: Option<Color>,
    focused: bool,
    radius: Corner,
    /// The tube's width in logical pixels, whole device pixels at the scale.
    tube: f32,
    /// The core's width outside a fill, in logical pixels.
    thin: f32,
    /// The control's box is rounded up to it, so the tube on its edge and every ring of the glow
    /// land on device pixels too.
    scale: Scale,
    motion: Motion,
}

impl<M> Widget<M, Theme, Renderer> for Ring<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Light>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Light::new(self.focused, Instant::now()))
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.state
            .downcast_mut::<Light>()
            .to(self.focused, self.motion, Instant::now());
        tree.diff_children(slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let node = snapped(
            self.scale,
            &self
                .content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits),
        );
        layout::Node::with_children(node.size(), vec![node])
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        if let Some(content) = layout.children().next() {
            self.content
                .as_widget_mut()
                .operate(&mut tree.children[0], content, renderer, operation);
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        if let Event::Window(iced::window::Event::RedrawRequested(now)) = event
            && tree.state.downcast_ref::<Light>().moving(self.motion, *now)
        {
            shell.request_redraw();
        }
        if let Some(content) = layout.children().next() {
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                content,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        layout
            .children()
            .next()
            .map_or_else(mouse::Interaction::default, |content| {
                self.content
                    .as_widget()
                    .mouse_interaction(&tree.children[0], content, cursor, viewport, renderer)
            })
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if let Some(content) = layout.children().next() {
            self.content
                .as_widget()
                .draw(&tree.children[0], renderer, theme, style, content, cursor, viewport);
        }
        let lit = tree.state.downcast_ref::<Light>().value(self.motion, Instant::now());
        if lit <= 0.0 {
            return;
        }
        let bounds = layout.bounds();
        // On a fill the glow starts outside the core, so it is laid out from there.
        let inset = if self.core.is_some() { self.thin } else { 0.0 };
        let edge = bounds.expand(inset);
        for (out, opacity) in rings(self.scale) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: edge.expand(out),
                    border: Border {
                        color: color::alpha(self.color, opacity * lit),
                        width: out,
                        radius: outset(self.radius, inset + out).into(),
                    },
                    snap: true,
                    ..renderer::Quad::default()
                },
                Background::Color(Color::TRANSPARENT),
            );
        }
        let (tube, color, corner) = match self.core {
            Some(core) => (edge, core, outset(self.radius, inset)),
            None => (bounds, self.color, radius(self.radius)),
        };
        let width = if self.core.is_some() { self.thin } else { self.tube };
        // The tube is the control's edge, over everything the control holds. Left in this layer it
        // would sit under a picture the control holds, since a layer draws its pictures after its
        // quads, and under the layers a control opens of its own (a list's scroll, a stack laid
        // over a picture, a field's text), since a later layer draws over an earlier one. Its own
        // layer, opened after the content's, keeps it on top. A layer's clip replaces its
        // parent's instead of narrowing it, so it is clipped to the viewport by hand, a device
        // pixel wider than the tube: the clip has no anti-aliasing, and would cut the tube's
        // half-covered outer pixels where a scroll leaves it between device pixels.
        let Some(clip) = tube.expand(self.scale.hairline()).intersection(viewport) else {
            return;
        };
        renderer.with_layer(clip, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: tube,
                    border: Border {
                        color: color::alpha(color, lit),
                        width,
                        radius: corner.into(),
                    },
                    snap: true,
                    ..renderer::Quad::default()
                },
                Background::Color(Color::TRANSPARENT),
            );
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        let content = layout.children().next()?;
        self.content
            .as_widget_mut()
            .overlay(&mut tree.children[0], content, renderer, viewport, translation)
    }
}

#[cfg(test)]
mod tests;
