//! An app's initial on its icon tile, in the symbol's place.
//!
//! # Owns
//!
//! - [`initial`], the letter a name is shown by, and [`letter`], the widget that places it on the
//!   tile: across on its advance, down on its ink, both on whole device pixels.
//!
//! # Not here
//!
//! - The tile around it: [`super::IconTile`]. Measuring the ink: [`crate::text::ink`].

use iced::Color;
use iced::Element;
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
use iced::advanced::text::paragraph::Plain;
use iced::advanced::widget::Operation;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree;
use iced::alignment;
use kubaos_geometry::Role;
use unicode_segmentation::UnicodeSegmentation as _;

use crate::scale::Scale;
use crate::text::font;
use crate::text::ink;
use crate::text::line_height;
use crate::tokens::Type;

type Paragraph = <Renderer as text::Renderer>::Paragraph;

/// The first character of `name`, upper-cased: one letter, in whatever script the name is in.
pub(super) fn initial(name: &str) -> String {
    name.trim()
        .graphemes(true)
        .next()
        .map(str::to_uppercase)
        .unwrap_or_default()
}

/// `letter` in `style`, coloured `color`, on a tile as big as the room it is given at `scale`.
pub(super) fn letter<'a, M: 'a>(letter: String, style: Type, color: Color, scale: Scale) -> Element<'a, M> {
    Element::new(Initial {
        letter,
        style,
        color,
        scale,
    })
}

struct Initial {
    letter: String,
    style: Type,
    color: Color,
    scale: Scale,
}

/// The letter shaped, and the middle of its ink, kept while the letter stays.
#[derive(Default)]
struct InitialState {
    shown: Plain<Paragraph>,
    letter: String,
    /// The middle of its ink above the baseline, in ems.
    middle: f32,
}

/// `logical` pixels in whole device pixels at `scale`: the tokens' sizes are whole at every
/// supported scale.
fn device(scale: Scale, logical: f32) -> f32 {
    scale.to_device(logical).round()
}

/// Half of `spare` device pixels, a tie going up, as CSS's `round(nearest)` does in the mockup.
fn half_up(spare: f32) -> f32 {
    (spare / 2.0 + 0.5).floor()
}

impl Initial {
    fn shaped(&self) -> text::Text<&str, iced::Font> {
        text::Text {
            content: &self.letter,
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
}

impl<M> Widget<M, Theme, Renderer> for Initial {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<InitialState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(InitialState::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    /// The tile, holding the letter's box: its advance rounded up to a device pixel wide and its
    /// style's line tall, centred on the nearest device pixel, a tie after the odd one, as the
    /// mockup's `.k-icon-tile-initial` places it. Worked in whole device pixels: half of an odd
    /// count is a tie, which float error in logical pixels breaks either way.
    fn layout(&mut self, tree: &mut Tree, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let state = tree.state.downcast_mut::<InitialState>();
        if state.letter != self.letter {
            state.shown.update(self.shaped());
            state.middle = ink::middle(self.style, &self.letter);
            state.letter.clone_from(&self.letter);
        }
        let room = limits.max();
        let scale = self.scale;
        let width = (scale.to_device(state.shown.min_width()) - 1e-3).ceil().max(0.0);
        let line = device(scale, f32::from(self.style.line()));
        let x = half_up(device(scale, room.width) - width);
        let y = half_up(device(scale, room.height) - line);
        let factor = scale.factor();
        let letter =
            layout::Node::new(Size::new(width / factor, line / factor)).move_to(Point::new(x / factor, y / factor));
        layout::Node::with_children(room, vec![letter])
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, _renderer: &Renderer, operation: &mut dyn Operation) {
        let Some(letter) = layout.children().next() else {
            return;
        };
        let bounds = letter.bounds();
        let natural = tree.state.downcast_ref::<InitialState>().shown.min_bounds();
        let mut role = Role::Text {
            content: self.letter.clone(),
            natural: kubaos_geometry::Size {
                width: natural.width,
                height: f32::from(self.style.line()),
            },
        };
        operation.custom(None, bounds, &mut role);
        operation.traverse(&mut |operation| operation.text(None, bounds, &self.letter));
    }

    /// The letter with its baseline where its ink's middle lands in the middle of its line, on
    /// the nearest whole device pixel.
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
        let Some(letter) = layout.children().next() else {
            return;
        };
        let state = tree.state.downcast_ref::<InitialState>();
        let scale = self.scale;
        let bounds = letter.bounds();
        let line = device(scale, f32::from(self.style.line()));
        let size = device(scale, f32::from(self.style.size()));
        let baseline = device(scale, bounds.y) + (line / 2.0 + state.middle * size + 0.5).floor();
        let paragraph = state.shown.raw();
        let laid = paragraph
            .buffer()
            .layout_runs()
            .next()
            .map_or(0.0, |run| scale.to_device(run.line_y).round());
        // HAZARD: the renderers truncate a glyph's top to a device pixel and add the line's
        // baseline, rounded, after. A top a hair under the whole pixel the baseline needs drops
        // the letter a row; half a pixel down keeps it on that pixel.
        let top = (baseline - laid + 0.5) / scale.factor();
        renderer.fill_paragraph(paragraph, Point::new(bounds.x, top), self.color, *viewport);
    }
}
