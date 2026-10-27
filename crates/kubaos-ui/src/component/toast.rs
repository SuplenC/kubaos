//! Toasts: one line that says what just happened, then goes.
//!
//! A toast arrives over [`Duration::Short`], rising into place as it fades in on
//! [`Easing::EmphasizedDecel`], stays [`DEFAULT_STAY`], and leaves over [`Duration::Medium`],
//! fading out on [`Easing::StandardAccel`]: [`toast_at`].

use std::time;

use iced::Background;
use iced::Border;
use iced::Element;
use iced::widget;
use iced::widget::container;

use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::Inset;
use crate::layout::row;
use crate::look::Look;
use crate::look::radius;
use crate::motion::Motion;
use crate::probe;
use crate::probe::Fixed;
use crate::snap::snap;
use crate::text::text;
use crate::tokens::Duration;
use crate::tokens::Easing;
use crate::tokens::Radius;
use crate::tokens::Space;
use crate::tokens::Type;

/// A toast saying `message`, with an optional leading icon, in the inverse of the page's colours
/// so it reads over anything. Its box is as wide as its text, rounded up to a device pixel.
pub fn toast<'a, M: 'a>(look: &Look, message: &'a str, leading: Option<Icon>) -> Element<'a, M> {
    let c = look.colors;
    let (fill, content) = (c.on_surface, c.surface);
    let mut line = row(look, Space::S12).center();
    if let Some(glyph) = leading {
        line = line.push(icon(look, glyph, Type::Body.icon(), content));
    }
    line = line.push(text(message, Type::Body, content));
    let surface = widget::container(line)
        .padding(Inset::xy(Space::S16, Space::S12).padding())
        .style(move |_theme| container::Style {
            background: Some(Background::Color(fill)),
            border: Border::default().rounded(radius(Radius::Surface)),
            ..container::Style::default()
        });
    let view: Element<'a, M> = snap(look.scale, surface);
    probe::component("Toast", ["", "", ""], Fixed::NONE, view)
}

/// How long a toast stays once it has arrived, before it leaves: long enough to read its line
/// twice.
pub const DEFAULT_STAY: time::Duration = time::Duration::from_millis(3600);

/// A toast's whole life when it stays `stay`: arriving, staying and leaving.
pub fn life(stay: time::Duration) -> time::Duration {
    let motion = Motion::default();
    motion.duration(Duration::Short) + stay + motion.duration(Duration::Medium)
}

/// How a toast `age` into its life shows: its opacity, and how far under its place it still is, in
/// logical pixels.
pub fn presence(age: time::Duration, stay: time::Duration) -> (f32, f32) {
    let motion = Motion::default();
    let arrived = motion.progress(Duration::Short, Easing::EmphasizedDecel, age);
    let leaving = age.saturating_sub(motion.duration(Duration::Short) + stay);
    let gone = motion.progress(Duration::Medium, Easing::StandardAccel, leaving);
    let rise = (1.0 - arrived) * f32::from(Space::S32.px());
    (arrived * (1.0 - gone), rise)
}

/// [`toast`] `age` into a life in which it stays `stay`, in a box [`Space::S32`] taller than
/// itself: it rises into the top of the box as it arrives, a whole device pixel at a time so its
/// edges stay sharp, and fades as it leaves.
pub fn toast_at<'a, M: 'a>(
    look: &Look,
    message: &'a str,
    leading: Option<Icon>,
    age: time::Duration,
    stay: time::Duration,
) -> Element<'a, M> {
    let (opacity, rise) = presence(age, stay);
    let rise = look.scale.snap(rise);
    let room = f32::from(Space::S32.px());
    widget::container(toast(&look.faded(opacity), message, leading))
        .padding(iced::Padding::ZERO.top(rise).bottom(room - rise))
        .into()
}
