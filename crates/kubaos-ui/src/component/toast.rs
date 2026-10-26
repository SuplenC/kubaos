//! Toasts: one line that says what just happened, then goes.

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
use crate::snap::snap;
use crate::text::text;
use crate::tokens::IconSize;
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
        line = line.push(icon(look, glyph, IconSize::S20, content));
    }
    line = line.push(text(message, Type::Body, content));
    let surface = widget::container(line)
        .padding(Inset::xy(Space::S16, Space::S12).padding())
        .style(move |_theme| container::Style {
            background: Some(Background::Color(fill)),
            border: Border::default().rounded(radius(Radius::R12)),
            ..container::Style::default()
        });
    snap(look.scale, surface)
}
