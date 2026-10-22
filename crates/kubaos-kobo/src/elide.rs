//! One line of text that ends in "…" when it doesn't fit, measured with the
//! real font (iced's `text` either wraps or clips mid-glyph).

use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::text::{self, Paragraph as _, Text, paragraph::Plain};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::{
    Color, Element, Font, Length, Pixels, Point, Rectangle, Renderer, Size, Theme, alignment,
};

type P = <Renderer as text::Renderer>::Paragraph;

pub struct Elide {
    content: String,
    font: Font,
    size: f32,
    color: Color,
    width: Length,
}

/// `s` on one line, cut with an ellipsis to the width it gets.
pub fn elide(s: impl Into<String>, font: Font, size: f32, color: Color) -> Elide {
    Elide {
        content: s.into(),
        font,
        size,
        color,
        width: Length::Shrink,
    }
}

impl Elide {
    /// `Fill` takes the leftover space in a row (and elides to it);
    /// `Shrink` (the default) is the text's own width, up to what's there.
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
}

#[derive(Default)]
struct State {
    para: Plain<P>,
    /// What was fitted, to what width: layout runs every frame while
    /// something animates, and measuring isn't free.
    fitted: Option<(String, u32)>,
}

impl Elide {
    fn text<'b>(&self, s: &'b str) -> Text<&'b str, Font> {
        Text {
            content: s,
            bounds: Size::new(f32::INFINITY, f32::INFINITY),
            size: Pixels(self.size),
            line_height: text::LineHeight::default(),
            font: self.font,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Top,
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::None,
        }
    }

    fn width_of(&self, s: &str) -> f32 {
        P::with_text(self.text(s)).min_width()
    }

    /// The longest prefix (plus "…") that fits in `max`.
    fn fit(&self, max: f32) -> String {
        if self.width_of(&self.content) <= max {
            return self.content.clone();
        }
        let chars: Vec<char> = self.content.chars().collect();
        let (mut lo, mut hi) = (0usize, chars.len());
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            let cand: String = chars[..mid]
                .iter()
                .collect::<String>()
                .trim_end()
                .to_string()
                + "…";
            if self.width_of(&cand) <= max {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        chars[..lo]
            .iter()
            .collect::<String>()
            .trim_end()
            .to_string()
            + "…"
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Elide {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        let max = limits.max().width;
        let key = (self.content.clone(), max.to_bits());
        if state.fitted.as_ref() != Some(&key) {
            let shown = self.fit(max);
            state.para.update(self.text(&shown));
            state.fitted = Some(key);
        }
        let b = state.para.min_bounds();
        let w = match self.width {
            Length::Shrink => b.width.min(max),
            _ => max,
        };
        layout::Node::new(limits.resolve(self.width, Length::Shrink, Size::new(w, b.height)))
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
        let state = tree.state.downcast_ref::<State>();
        let b = layout.bounds();
        renderer.fill_paragraph(
            state.para.raw(),
            Point::new(b.x, b.y),
            self.color,
            *viewport,
        );
    }
}

impl<'a, Message: 'a> From<Elide> for Element<'a, Message> {
    fn from(e: Elide) -> Self {
        Element::new(e)
    }
}
