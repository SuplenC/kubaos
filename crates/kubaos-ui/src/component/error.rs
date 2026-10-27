//! Errors: what broke and what to do next, with the log behind it.

use iced::Background;
use iced::Border;
use iced::Element;
use iced::Length;
use iced::widget;
use iced::widget::container;

use crate::color;
use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::Extent;
use crate::layout::Gap;
use crate::layout::Inset;
use crate::layout::row;
use crate::layout::stack;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::text::text;
use crate::tokens::IconSize;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Space;
use crate::tokens::Type;

/// Past this many characters a message drops from the subheading to body size: a whole command
/// line in 16 px semibold is a wall.
const LONG_MESSAGE: usize = 90;

/// The error: `message`, what broke, and under it `hint`, the next thing to try.
pub fn error_block<'a, M: 'a>(look: &Look, message: &'a str, hint: &'a str) -> Element<'a, M> {
    let c = look.colors;
    let long = message.chars().count() > LONG_MESSAGE;
    let style = if long { Type::BodyStrong } else { Type::Subheading };
    let fill = color::alpha(c.error_container, Opacity::ErrorFill.value());
    let edge = color::alpha(c.error, Opacity::ErrorEdge.value());
    let hairline = look.hairline();
    let view: Element<'a, M> = widget::container(
        row(look, Space::S16)
            .center()
            .push(icon(look, Icon::Error, IconSize::S24, c.error))
            .push(
                stack(look, Space::S4)
                    .width(Extent::Fill)
                    .push(text(message, style, c.on_surface))
                    .push(text(hint, Type::Body, c.on_surface_variant)),
            ),
    )
    .padding(Inset::all(Space::S20).padding())
    .width(Length::Fill)
    .style(move |_theme| container::Style {
        background: Some(Background::Color(fill)),
        border: Border {
            color: edge,
            width: hairline,
            radius: radius(Radius::R16).into(),
        },
        ..container::Style::default()
    })
    .into();
    probe::component(
        "ErrorBlock",
        [if long { "long" } else { "" }, "", ""],
        Fixed::NONE,
        view,
    )
}

/// How loud a log line is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warn,
    /// The start of a step.
    Step,
    Info,
    Debug,
}

/// The log, newest line at the bottom, `height` tall.
pub fn log<'a, M: 'a>(look: &Look, lines: &[(Level, &'a str)], height: Extent) -> Element<'a, M> {
    let c = look.colors;
    let rows = lines.iter().map(|&(level, line)| {
        let colour = match level {
            Level::Error => c.error,
            Level::Warn => c.primary,
            Level::Step => c.cold,
            Level::Info => c.on_surface,
            Level::Debug => c.on_surface_variant,
        };
        text(line, Type::Mono, colour)
    });
    let fill = color::alpha(c.surface_container_lowest, Opacity::Log.value());
    let edge = look.outline_soft();
    let hairline = look.hairline();
    let view: Element<'a, M> = widget::container(
        widget::scrollable(stack(look, Gap::NONE).extend(rows))
            .anchor_bottom()
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .padding(Inset::xy(Space::S16, Space::S12).padding())
    .width(Length::Fill)
    .height(height.length())
    .style(move |_theme| container::Style {
        background: Some(Background::Color(fill)),
        border: Border {
            color: edge,
            width: hairline,
            radius: radius(Radius::R12).into(),
        },
        ..container::Style::default()
    })
    .into();
    probe::component("Log", ["", "", ""], Fixed::NONE, view)
}
