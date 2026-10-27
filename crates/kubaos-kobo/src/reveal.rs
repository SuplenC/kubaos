//! Focus follows Tab on long screens: when the focused control is outside
//! the scrollable's viewport, scroll just enough to show it, with a margin,
//! and glide there instead of jumping. A control that's already in view
//! doesn't move the page at all.

use std::time::{Duration, Instant};

use iced::advanced::widget::{Id, Operation, operation};
use iced::{Rectangle, Vector};

use kubaos_ui::motion::ease;
use kubaos_ui::tokens;
use kubaos_ui::tokens::Easing;

/// How long the glide takes: focus moving.
pub const DURATION: Duration = Duration::from_millis(tokens::Duration::Short.ms() as u64);
/// Space kept between the control and the viewport's edge.
pub const MARGIN: f32 = tokens::Space::S24.px() as f32;

/// Where the scrollable is and where the control sits in its content.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Found {
    /// Current offset.
    pub offset: f32,
    /// Visible height.
    pub viewport: f32,
    /// Content height.
    pub content: f32,
    /// The control's top and bottom, relative to the content's top.
    pub top: f32,
    pub bottom: f32,
}

/// The offset that shows `top..bottom` with `MARGIN` around it, scrolling
/// as little as possible. `None` when it's already in view.
pub fn target(f: Found) -> Option<f32> {
    let max = (f.content - f.viewport).max(0.0);
    let (top, bottom) = (
        (f.top - MARGIN).max(0.0),
        (f.bottom + MARGIN).min(f.content),
    );
    let y = if top < f.offset {
        top
    } else if bottom > f.offset + f.viewport {
        // Taller than the viewport: its top wins.
        (bottom - f.viewport).min(top)
    } else {
        return None;
    };
    let y = y.clamp(0.0, max);
    ((y - f.offset).abs() >= 1.0).then_some(y)
}

/// A glide in progress.
#[derive(Debug, Clone, Copy)]
pub struct Glide {
    pub id: &'static str,
    pub from: f32,
    pub to: f32,
    pub at: Instant,
}

impl Glide {
    /// The offset at `now`, and whether the glide is over.
    pub fn at(&self, now: Instant) -> (f32, bool) {
        let t = now.saturating_duration_since(self.at).as_secs_f32() / DURATION.as_secs_f32();
        let e = ease(Easing::EmphasizedDecel, t.min(1.0));
        (self.from + (self.to - self.from) * e, t >= 1.0)
    }
}

/// Finds the scrollable `scroll` and the container `item` inside it.
pub struct Find {
    scroll: Id,
    item: Id,
    view: Option<(Rectangle, Rectangle, Vector)>,
    found: Option<Rectangle>,
}

impl Find {
    pub fn new(scroll: &'static str, item: Id) -> Self {
        Self {
            scroll: Id::new(scroll),
            item,
            view: None,
            found: None,
        }
    }
}

impl Operation<Found> for Find {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Found>)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn operation::Scrollable,
    ) {
        if id == Some(&self.scroll) {
            self.view = Some((bounds, content_bounds, translation));
        }
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if id == Some(&self.item) {
            self.found = Some(bounds);
        }
    }

    fn finish(&self) -> operation::Outcome<Found> {
        match (self.view, self.found) {
            // Layout bounds inside a scrollable aren't translated: the
            // content's top is the reference.
            (Some((b, c, t)), Some(r)) => operation::Outcome::Some(Found {
                offset: t.y,
                viewport: b.height,
                content: c.height,
                top: r.y - c.y,
                bottom: r.y + r.height - c.y,
            }),
            _ => operation::Outcome::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(offset: f32, top: f32, bottom: f32) -> Found {
        Found {
            offset,
            viewport: 400.0,
            content: 1000.0,
            top,
            bottom,
        }
    }

    #[test]
    fn in_view_stays_put() {
        assert_eq!(target(f(0.0, 100.0, 200.0)), None);
        assert_eq!(target(f(300.0, 400.0, 600.0)), None);
    }

    #[test]
    fn below_scrolls_just_enough() {
        // Bottom 500 + margin 24 = 524, minus the 400 viewport.
        assert_eq!(target(f(0.0, 450.0, 500.0)), Some(124.0));
    }

    #[test]
    fn above_scrolls_just_enough() {
        assert_eq!(target(f(300.0, 200.0, 250.0)), Some(176.0));
    }

    #[test]
    fn taller_than_the_viewport_shows_its_top() {
        assert_eq!(target(f(0.0, 300.0, 900.0)), Some(276.0));
    }

    #[test]
    fn clamped_to_the_content() {
        assert_eq!(target(f(0.0, 950.0, 1000.0)), Some(600.0));
        assert_eq!(target(f(300.0, 10.0, 50.0)), Some(0.0));
    }

    #[test]
    fn glide_ends_on_target() {
        let at = Instant::now();
        let g = Glide {
            id: "x",
            from: 0.0,
            to: 100.0,
            at,
        };
        assert_eq!(g.at(at), (0.0, false));
        let (y, done) = g.at(at + DURATION);
        assert!(done && (y - 100.0).abs() < 0.01);
    }
}
