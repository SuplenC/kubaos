//! Text in a type style, and the fonts every app loads.
//!
//! # Owns
//!
//! - Turning a [`Type`] into an iced font, size and line height ([`text`], [`font`]), and the
//!   bundled font files ([`FONTS`]).
//! - [`line`]: one line that ends in an ellipsis where it doesn't fit, for boxes whose height is
//!   fixed (a list row, a card's name).
//!
//! # Not here
//!
//! - The styles' values: [`crate::tokens`]. Icons: [`crate::icon`].

use iced::Color;
use iced::Element;
use iced::Font;
use iced::Length;
use iced::Pixels;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Size;
use iced::Theme;
use iced::advanced::Layout;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::text;
use iced::advanced::text::Paragraph as _;
use iced::advanced::text::paragraph::Plain;
use iced::advanced::widget::Operation;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree;
use iced::alignment;
use iced::font;
use iced::widget;
use iced::widget::text::IntoFragment;
use iced::widget::text::LineHeight;

use crate::probe::Tag;
use crate::probe::probe;
use crate::tokens::Family;
use crate::tokens::Type;
use crate::tokens::Weight;

/// Every font the type styles use, for `iced::Settings::fonts`. An app loads them at
/// start, so it draws the same on a machine with none installed.
pub const FONTS: &[&[u8]] = &[
    include_bytes!("../assets/fonts/Geist-Light.ttf"),
    include_bytes!("../assets/fonts/Geist-Regular.ttf"),
    include_bytes!("../assets/fonts/Geist-Medium.ttf"),
    include_bytes!("../assets/fonts/Geist-SemiBold.ttf"),
    include_bytes!("../assets/fonts/GeistMono-Regular.ttf"),
    include_bytes!("../assets/fonts/GeistMono-Medium.ttf"),
    include_bytes!("../assets/fonts/MPLUS1-subset.ttf"),
];

/// The font of `family` at `weight`.
pub fn family_font(family: Family, weight: Weight) -> Font {
    Font {
        family: font::Family::Name(family.name()),
        weight: match weight {
            Weight::Light => font::Weight::Light,
            Weight::Regular => font::Weight::Normal,
            Weight::Medium => font::Weight::Medium,
            Weight::Semibold => font::Weight::Semibold,
        },
        ..Font::DEFAULT
    }
}

/// The font a type style draws with.
pub fn font(style: Type) -> Font {
    family_font(style.family(), style.weight())
}

/// The font apps draw with when a widget names none: body text.
pub fn default_font() -> Font {
    font(Type::Body)
}

/// `content` in `style`, coloured `color`, on the style's line height.
pub fn text<'a, M: 'a>(content: impl IntoFragment<'a>, style: Type, color: Color) -> Element<'a, M> {
    let shaped = widget::text(content)
        .font(font(style))
        .size(f32::from(style.size()))
        .line_height(line_height(style))
        .color(color);
    probe(Tag::Text, shaped)
}

/// A type style's line height, in absolute pixels so it stays on the grid.
pub fn line_height(style: Type) -> LineHeight {
    LineHeight::Absolute(Pixels(f32::from(style.line())))
}

type Paragraph = <Renderer as text::Renderer>::Paragraph;

/// What ends a line cut short.
const ELLIPSIS: &str = "…";

/// `content` in `style` on one line, as wide as the room it gets at most: cut after the last
/// character that fits, with an ellipsis, where it is wider. Its box is the line it shows.
pub fn line<'a, M: 'a>(content: impl Into<String>, style: Type, color: Color) -> Element<'a, M> {
    probe(
        Tag::Text,
        Element::new(Line {
            content: content.into(),
            style,
            color,
        }),
    )
}

struct Line {
    content: String,
    style: Type,
    color: Color,
}

/// A [`line`]'s shaped text, and what it was fitted to: layout runs on every frame while anything
/// animates, and measuring isn't free.
#[derive(Default)]
pub(crate) struct LineState {
    shown: Plain<Paragraph>,
    /// The content and the room it was last fitted to.
    content: String,
    room: Option<u32>,
}

impl LineState {
    /// The size the text it shows needs.
    pub(crate) fn min_bounds(&self) -> Size {
        self.shown.min_bounds()
    }
}

impl Line {
    fn shaped<'b>(&self, content: &'b str) -> text::Text<&'b str, Font> {
        text::Text {
            content,
            bounds: Size::INFINITE,
            size: Pixels(f32::from(self.style.size())),
            line_height: line_height(self.style),
            font: font(self.style),
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Top,
            shaping: text::Shaping::Auto,
            wrapping: text::Wrapping::None,
        }
    }

    fn width_of(&self, content: &str) -> f32 {
        Paragraph::with_text(self.shaped(content)).min_width()
    }

    /// The longest start of the content that fits in `room` with an ellipsis after it, for content
    /// that doesn't fit whole.
    fn fit(&self, room: f32) -> String {
        let chars: Vec<char> = self.content.chars().collect();
        let cut = |n: usize| chars[..n].iter().collect::<String>().trim_end().to_owned() + ELLIPSIS;
        let (mut fits, mut over) = (0, chars.len());
        while over - fits > 1 {
            let mid = usize::midpoint(fits, over);
            if self.width_of(&cut(mid)) <= room {
                fits = mid;
            } else {
                over = mid;
            }
        }
        cut(fits)
    }
}

impl<M> Widget<M, Theme, Renderer> for Line {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<LineState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(LineState::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&mut self, tree: &mut Tree, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let state = tree.state.downcast_mut::<LineState>();
        let room = limits.max().width;
        if state.room != Some(room.to_bits()) || state.content != self.content {
            // Shaped whole first: most lines fit, and then that is the only shaping.
            state.shown.update(self.shaped(&self.content));
            if state.shown.min_width() > room {
                let cut = self.fit(room);
                state.shown.update(self.shaped(&cut));
            }
            state.content.clone_from(&self.content);
            state.room = Some(room.to_bits());
        }
        let needed = state.shown.min_bounds();
        layout::Node::new(limits.resolve(
            Length::Shrink,
            Length::Shrink,
            Size::new(needed.width.min(room), f32::from(self.style.line())),
        ))
    }

    fn operate(&mut self, _tree: &mut Tree, layout: Layout<'_>, _renderer: &Renderer, operation: &mut dyn Operation) {
        operation.text(None, layout.bounds(), &self.content);
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::text::Renderer as _;
        let state = tree.state.downcast_ref::<LineState>();
        let bounds = layout.bounds();
        renderer.fill_paragraph(state.shown.raw(), Point::new(bounds.x, bounds.y), self.color, *viewport);
    }
}

#[cfg(test)]
mod tests;
