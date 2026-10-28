//! The one-line text field, and its label and note.
//!
//! The app owns the field's value and its editing: it keeps the [`Caret`], turns keys into edits,
//! and gives the field both on every view, so real keys, a scripted tour and the tests all go
//! through one code path. The field draws (the value or the placeholder, the selection, a blinking
//! caret, sideways scrolling that keeps the caret in view) and turns the pointer's presses and
//! drags into [`Pointer`]s, measured on the text it draws.

use std::fmt;
use std::time::Duration;
use std::time::Instant;

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Event;
use iced::Font;
use iced::Length;
use iced::Pixels;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Size as IcedSize;
use iced::Theme;
use iced::advanced::Clipboard;
use iced::advanced::Layout;
use iced::advanced::Renderer as _;
use iced::advanced::Shell;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::mouse::click;
use iced::advanced::renderer;
use iced::advanced::text;
use iced::advanced::text::Paragraph as _;
use iced::advanced::text::paragraph::Plain;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree;
use iced::alignment;
use iced::keyboard;
use iced::widget;
use iced::window;

use crate::color;
use crate::color::Colors;
use crate::focus::RingColor;
use crate::focus::focus_ring;
use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::Gap;
use crate::layout::fill;
use crate::layout::row;
use crate::layout::stack;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::scale::Scale;
use crate::snap::centre;
use crate::text as type_style;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// The type the value and the placeholder are set in.
const VALUE: Type = Type::BodyLarge;

/// How long the caret shows, then hides, while nothing moves it. It stays solid for one of these
/// after it moves, so it never blinks away while you type.
// design-allow(duration): the caret's blink is behaviour, not motion.
pub const BLINK: Duration = Duration::from_millis(530);

/// How a field shows its value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Editable.
    Normal,
    /// The value is wrong: the error colour, with the reason in the note under it.
    Invalid,
    /// Read-only: dimmed, never focused.
    Locked,
}

/// Where the caret is in a field's value and what is selected, in `char`s: a field drawn with
/// bullets has one per `char`, so a position means the same thing in the value and on screen.
/// `anchor == pos` is no selection; otherwise the selection runs between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Caret {
    pub pos: usize,
    pub anchor: usize,
}

impl Caret {
    /// The caret at `pos`, nothing selected.
    #[must_use]
    pub fn at(pos: usize) -> Caret {
        Caret { pos, anchor: pos }
    }

    /// Everything selected, the caret at the end.
    #[must_use]
    pub fn all(value: &str) -> Caret {
        Caret {
            pos: value.chars().count(),
            anchor: 0,
        }
    }

    pub fn has_selection(self) -> bool {
        self.pos != self.anchor
    }

    /// The selection as its start and end, in `char`s.
    pub fn range(self) -> (usize, usize) {
        (self.pos.min(self.anchor), self.pos.max(self.anchor))
    }

    /// Held to `value`'s length: the value can change under the caret.
    #[must_use]
    pub fn clamp(self, value: &str) -> Caret {
        let n = value.chars().count();
        Caret {
            pos: self.pos.min(n),
            anchor: self.anchor.min(n),
        }
    }
}

/// What the pointer did in a field, in `char`s of its value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pointer {
    /// A press at `pos`; with Shift held, `extend` grows the selection to it.
    Press { pos: usize, extend: bool },
    /// A drag from the last press to `pos`.
    Drag(usize),
    /// A double press on the word at `pos`.
    Double(usize),
    /// A triple press: all of it.
    Triple,
}

/// A text field, built up then drawn with [`Field::view`].
pub struct Field<'a, M> {
    value: &'a str,
    placeholder: &'a str,
    icon: Option<Icon>,
    secret: bool,
    state: State,
    caret: Option<Caret>,
    blink_from: Option<Instant>,
    on_pointer: Option<Box<dyn Fn(Pointer) -> M + 'a>>,
}

impl<M> fmt::Debug for Field<'_, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Field")
            .field("value", if self.secret { &"<secret>" } else { &self.value })
            .field("placeholder", &self.placeholder)
            .field("icon", &self.icon)
            .field("state", &self.state)
            .field("caret", &self.caret)
            .finish_non_exhaustive()
    }
}

/// A field showing `value`, or the dimmed `placeholder` while it is empty: a hint, never a value
/// that looks typed.
pub fn field<'a, M>(value: &'a str, placeholder: &'a str) -> Field<'a, M> {
    Field {
        value,
        placeholder,
        icon: None,
        secret: false,
        state: State::Normal,
        caret: None,
        blink_from: None,
        on_pointer: None,
    }
}

impl<'a, M: Clone + 'a> Field<'a, M> {
    /// `icon` inside the field, before the text.
    #[must_use]
    pub fn icon(self, icon: Icon) -> Self {
        Field {
            icon: Some(icon),
            ..self
        }
    }

    /// Bullets instead of the value, one per `char`.
    #[must_use]
    pub fn secret(self) -> Self {
        Field { secret: true, ..self }
    }

    #[must_use]
    pub fn state(self, state: State) -> Self {
        Field { state, ..self }
    }

    /// The keyboard is in the field, its caret at `caret`: the focus tube on its edge, and the
    /// caret or the selection.
    #[must_use]
    pub fn caret(self, caret: Caret) -> Self {
        Field {
            caret: Some(caret),
            ..self
        }
    }

    /// When the caret last moved: it stays solid for a blink from then, so it never hides while you
    /// type. Unset, it never blinks.
    #[must_use]
    pub fn blink_from(self, at: Instant) -> Self {
        Field {
            blink_from: Some(at),
            ..self
        }
    }

    /// `on_pointer` with each press, drag and double or triple press on the text.
    #[must_use]
    pub fn on_pointer(self, on_pointer: impl Fn(Pointer) -> M + 'a) -> Self {
        Field {
            on_pointer: Some(Box::new(on_pointer)),
            ..self
        }
    }

    pub fn view(self, look: &Look) -> Element<'a, M> {
        let variant = [if self.icon.is_some() { "icon" } else { "" }, "", ""];
        let locked = self.state == State::Locked;
        let caret = self.caret.filter(|_| !locked);
        let shown = if self.secret {
            "•".repeat(self.value.chars().count())
        } else {
            self.value.to_owned()
        };
        let input = Element::new(Input {
            shown,
            placeholder: self.placeholder,
            icon: self.icon.is_some(),
            state: self.state,
            caret: caret.map(|caret| caret.clamp(self.value)),
            blink_from: self.blink_from,
            colors: look.colors,
            scale: look.scale,
            on_pointer: self.on_pointer.filter(|_| !locked),
        });
        let focused = caret.is_some();
        let tint = if self.state == State::Invalid {
            RingColor::Error
        } else {
            RingColor::Accent
        };
        let Some(glyph) = self.icon else {
            let view: Element<'a, M> = focus_ring(look, input, Radius::Control, focused, tint);
            return probe::component("Field", variant, Fixed::HEIGHT, view);
        };
        // The icon is drawn over the field, in a box 12 wider than itself on each side, so it
        // centres on the field's height on a whole device pixel.
        let well = f32::from(Space::S12.px() * 2 + Type::BodyLarge.icon().px());
        let mark = widget::container(centre(
            look.scale,
            icon(look, glyph, Type::BodyLarge.icon(), look.colors.on_surface_variant),
        ))
        .width(well)
        .height(Length::Fill);
        let field = widget::Stack::with_children([input, mark.into()]);
        let view: Element<'a, M> = focus_ring(look, field, Radius::Control, focused, tint);
        probe::component("Field", variant, Fixed::HEIGHT, view)
    }
}

type Paragraph = <Renderer as text::Renderer>::Paragraph;

/// The field itself: its box, its text, its caret.
struct Input<'a, M> {
    shown: String,
    placeholder: &'a str,
    icon: bool,
    state: State,
    caret: Option<Caret>,
    blink_from: Option<Instant>,
    colors: Colors,
    scale: Scale,
    on_pointer: Option<Box<dyn Fn(Pointer) -> M + 'a>>,
}

#[derive(Default)]
struct InputState {
    value: Plain<Paragraph>,
    placeholder: Plain<Paragraph>,
    last_click: Option<click::Click>,
    dragging: bool,
    shift: bool,
    hovered: bool,
}

/// `content` shaped in the field's type, on one line.
fn shaped(content: &str) -> text::Text<&str, Font> {
    text::Text {
        content,
        bounds: IcedSize::new(f32::INFINITY, f32::from(VALUE.line())),
        size: Pixels(f32::from(VALUE.size())),
        line_height: type_style::line_height(VALUE),
        font: type_style::font(VALUE),
        align_x: text::Alignment::Default,
        align_y: alignment::Vertical::Top,
        shaping: text::Shaping::Auto,
        wrapping: text::Wrapping::None,
    }
}

/// How far into the value's text `pos` falls, in logical pixels.
fn caret_x(value: &Plain<Paragraph>, pos: usize) -> f32 {
    if pos == 0 {
        return 0.0;
    }
    value
        .raw()
        .grapheme_position(0, pos)
        .map_or_else(|| value.min_width(), |at| at.x)
}

impl<M> Input<'_, M> {
    /// Where the text goes in the field's `bounds`: 12 in, or past the icon and 8 more, and the
    /// line in the middle of the field's height.
    fn text_bounds(&self, bounds: Rectangle) -> Rectangle {
        let left = if self.icon {
            Space::S12.px() + Type::BodyLarge.icon().px() + Space::S8.px()
        } else {
            Space::S12.px()
        };
        let right = Space::S12.px();
        let line = VALUE.line();
        let top = (Size::ControlMd.px() - line) / 2;
        Rectangle {
            x: bounds.x + f32::from(left),
            y: bounds.y + f32::from(top),
            width: (bounds.width - f32::from(left + right)).max(0.0),
            height: f32::from(line),
        }
    }

    /// How far the text is scrolled left, so the caret stays in view.
    fn offset(&self, state: &InputState, room: Rectangle) -> f32 {
        let Some(caret) = self.caret else {
            return 0.0;
        };
        (caret_x(&state.value, caret.pos) + self.scale.thin() - room.width).max(0.0)
    }

    /// The `char` position nearest `x`.
    fn pos_at(&self, state: &InputState, room: Rectangle, x: f32) -> usize {
        let local = x - room.x + self.offset(state, room);
        let count = self.shown.chars().count();
        if local <= 0.0 {
            return 0;
        }
        if local >= state.value.min_width() {
            return count;
        }
        state
            .value
            .raw()
            .hit_test(Point::new(local, room.height / 2.0))
            .map_or(count, |hit| {
                let byte = hit.cursor().min(self.shown.len());
                self.shown
                    .get(..byte)
                    .map_or(count, |start| start.chars().count())
                    .min(count)
            })
    }

    /// Whether the caret shows at `now`: always for a blink after it moved, then every other blink.
    fn caret_shows(&self, now: Instant) -> bool {
        let Some(from) = self.blink_from else {
            return true;
        };
        let since = now.saturating_duration_since(from).as_millis();
        (since / BLINK.as_millis()).is_multiple_of(2)
    }

    /// The field's edge and fill.
    fn look(&self, hovered: bool) -> (Color, Color) {
        let c = &self.colors;
        match self.state {
            State::Locked => (
                color::alpha(c.outline_variant, Opacity::Locked.value()),
                color::alpha(c.surface_container_high, Opacity::Locked.value()),
            ),
            State::Invalid => (c.error, c.surface_container_high),
            State::Normal if self.caret.is_some() => (c.primary, c.surface_container_highest),
            State::Normal if hovered => (c.outline, c.surface_container_highest),
            State::Normal => (c.outline_variant, c.surface_container_high),
        }
    }
}

impl<M> Widget<M, Theme, Renderer> for Input<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<InputState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(InputState::default())
    }

    fn size(&self) -> IcedSize<Length> {
        IcedSize::new(Length::Fill, Length::Fixed(f32::from(Size::ControlMd.px())))
    }

    fn layout(&mut self, tree: &mut Tree, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let state = tree.state.downcast_mut::<InputState>();
        state.value.update(shaped(&self.shown));
        state.placeholder.update(shaped(self.placeholder));
        let height = f32::from(Size::ControlMd.px());
        layout::Node::new(limits.resolve(Length::Fill, height, IcedSize::new(0.0, height)))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let room = self.text_bounds(bounds);
        let state = tree.state.downcast_mut::<InputState>();
        match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => state.shift = modifiers.shift(),
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let (Some(on_pointer), Some(at)) = (&self.on_pointer, cursor.position_over(bounds)) else {
                    return;
                };
                let press = click::Click::new(at, mouse::Button::Left, state.last_click);
                state.last_click = Some(press);
                let pos = self.pos_at(state, room, at.x);
                let pointer = match press.kind() {
                    click::Kind::Single => Pointer::Press {
                        pos,
                        extend: state.shift,
                    },
                    click::Kind::Double => Pointer::Double(pos),
                    click::Kind::Triple => Pointer::Triple,
                };
                state.dragging = matches!(pointer, Pointer::Press { .. });
                shell.publish(on_pointer(pointer));
                shell.capture_event();
            }
            // The cursor, not the event's position: in a scrolled page the cursor is where the
            // field is drawn, the event's position is where it is in the window.
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let hovered = self.state != State::Locked && cursor.is_over(bounds);
                if hovered != state.hovered {
                    state.hovered = hovered;
                    shell.request_redraw();
                }
                if state.dragging
                    && let Some(on_pointer) = &self.on_pointer
                    && let Some(at) = cursor.position()
                {
                    let pos = self.pos_at(state, room, at.x);
                    if self.caret.is_none_or(|caret| caret.pos != pos) {
                        shell.publish(on_pointer(Pointer::Drag(pos)));
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => state.dragging = false,
            Event::Mouse(mouse::Event::CursorLeft) if state.hovered => {
                state.hovered = false;
                shell.request_redraw();
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                if let (Some(caret), Some(from)) = (self.caret, self.blink_from)
                    && !caret.has_selection()
                {
                    let since = now.saturating_duration_since(from).as_millis();
                    let half = BLINK.as_millis();
                    let wait = u64::try_from(half - since % half).unwrap_or(u64::MAX);
                    shell.request_redraw_at(*now + Duration::from_millis(wait));
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.on_pointer.is_some() && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Text
        } else {
            mouse::Interaction::None
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
        use iced::advanced::text::Renderer as _;
        let state = tree.state.downcast_ref::<InputState>();
        let c = &self.colors;
        let bounds = layout.bounds();
        let (edge, fill) = self.look(state.hovered);
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: edge,
                    width: self.scale.hairline(),
                    radius: radius(Radius::Control).into(),
                },
                snap: true,
                ..renderer::Quad::default()
            },
            Background::Color(fill),
        );
        let room = self.text_bounds(bounds);
        let offset = self.offset(state, room);
        // The text may run a caret's width past its room, so a caret at the end shows whole.
        let thin = self.scale.thin();
        let clip = Rectangle {
            x: room.x - thin,
            width: room.width + 2.0 * thin,
            ..room
        };
        renderer.with_layer(clip, |renderer| {
            if self.shown.is_empty() {
                renderer.fill_paragraph(
                    state.placeholder.raw(),
                    Point::new(room.x, room.y),
                    color::alpha(c.on_surface_variant, Opacity::PlaceholderText.value()),
                    clip,
                );
            } else {
                if let Some(caret) = self.caret.filter(|caret| caret.has_selection()) {
                    let (start, end) = caret.range();
                    let (from, to) = (caret_x(&state.value, start), caret_x(&state.value, end));
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: room.x + from - offset,
                                width: to - from,
                                ..room
                            },
                            border: Border::default().rounded(radius(Radius::Small)),
                            ..renderer::Quad::default()
                        },
                        Background::Color(color::alpha(c.primary, Opacity::Selection.value())),
                    );
                }
                let value = if self.state == State::Locked {
                    c.on_surface_variant
                } else {
                    c.on_surface
                };
                renderer.fill_paragraph(state.value.raw(), Point::new(room.x - offset, room.y), value, clip);
            }
            if let Some(caret) = self.caret.filter(|caret| !caret.has_selection())
                && self.caret_shows(Instant::now())
            {
                let x = self
                    .scale
                    .snap(room.x + caret_x(&state.value, caret.pos) - offset - thin / 2.0);
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle { x, width: thin, ..room },
                        snap: true,
                        ..renderer::Quad::default()
                    },
                    Background::Color(c.primary),
                );
            }
        });
    }
}

/// `control` under its label, with an optional small action at the label's right (a Show toggle,
/// a small button) and an optional note under it. The label's row is as tall as a small button
/// whether or not there is one, so fields side by side line up.
pub fn labelled<'a, M: 'a>(
    look: &Look,
    label: &'a str,
    action: Option<Element<'a, M>>,
    control: impl Into<Element<'a, M>>,
    note: Option<Element<'a, M>>,
) -> Element<'a, M> {
    let c = look.colors;
    let mut head = row(look, Gap::NONE)
        .center()
        .height(crate::layout::Extent::Fixed(Size::ControlSm))
        .push(type_style::text(label, Type::Label, c.on_surface_variant))
        .push(fill());
    if let Some(action) = action {
        head = head.push(action);
    }
    let mut column = stack(look, Space::S4).push(head).push(control);
    if let Some(note) = note {
        column = column.push(note);
    }
    let view: Element<'a, M> = column.into();
    probe::component("Labelled", ["", "", ""], Fixed::NONE, view)
}

/// A note under a field: `message` in the error colour when `invalid`, otherwise muted.
pub fn note<'a, M: 'a>(look: &Look, message: impl Into<String>, invalid: bool) -> Element<'a, M> {
    let c = look.colors;
    let view: Element<'a, M> = type_style::text(
        message.into(),
        Type::Small,
        if invalid { c.error } else { c.on_surface_variant },
    );
    probe::component("Note", ["", "", ""], Fixed::NONE, view)
}

#[cfg(test)]
mod tests;
