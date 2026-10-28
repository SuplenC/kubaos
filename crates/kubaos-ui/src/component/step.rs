//! Steps: the rows of a stepper or a checklist, each saying where it stands.
//!
//! A step is to do, current, done, or waiting on an action (a reboot). Each state looks its own:
//! to do is a plain chip, done a quiet tonal one with a check, an action the accent with its
//! symbol, since that is what to do next; the current step is the selected row. A step's subtitle
//! wraps to as many lines as it needs, so a description is read whole.
//!
//! # Owns
//!
//! - [`step`], built up with [`Step`], and its [`StepState`].
//!
//! # Not here
//!
//! - The well the rows sit in: [`crate::component::list`].

use iced::Background;
use iced::Border;
use iced::Element;
use iced::Length;
use iced::Shadow;
use iced::Theme;
use iced::widget;

use crate::color;
use crate::component::chip::ChipKind;
use crate::component::chip::chip;
use crate::component::layered;
use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::Extent;
use crate::layout::Inset;
use crate::layout::row;
use crate::layout::stack;
use crate::look::Corner;
use crate::look::Look;
use crate::look::nested;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::text::line;
use crate::text::text;
use crate::tokens;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// A step's corners: a row's, concentric inside a list's well.
const CORNER: Corner = nested(Radius::Surface.corner(), Space::S12);

/// Where a step stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    /// Not done yet.
    Todo,
    /// The one the user is on.
    Current,
    Done,
    /// Done as far as it can be, waiting on an action: `label` (Reboot) and its symbol.
    Action {
        label: &'static str,
        glyph: Icon,
    },
}

impl StepState {
    /// The state's word in the spec.
    pub fn name(self) -> &'static str {
        match self {
            StepState::Todo => "todo",
            StepState::Current => "current",
            StepState::Done => "done",
            StepState::Action { .. } => "action",
        }
    }
}

/// One step, built up then drawn with [`Step::view`].
#[derive(Debug)]
pub struct Step<M> {
    title: String,
    subtitle: Option<String>,
    glyph: Option<Icon>,
    state: StepState,
    mark: bool,
    selected: bool,
    on_press: Option<M>,
}

/// A step reading `title`, to do.
pub fn step<M>(title: impl Into<String>) -> Step<M> {
    Step {
        title: title.into(),
        subtitle: None,
        glyph: None,
        state: StepState::Todo,
        mark: false,
        selected: false,
        on_press: None,
    }
}

impl<M: Clone> Step<M> {
    /// A line under the title, wrapped to the room there is.
    #[must_use]
    pub fn subtitle(self, subtitle: impl Into<String>) -> Self {
        Step {
            subtitle: Some(subtitle.into()),
            ..self
        }
    }

    /// A symbol before the title: what the step is about.
    #[must_use]
    pub fn icon(self, glyph: Icon) -> Self {
        Step {
            glyph: Some(glyph),
            ..self
        }
    }

    #[must_use]
    pub fn state(self, state: StepState) -> Self {
        Step { state, ..self }
    }

    /// The state as a mark at the end in place of a chip, for a narrow rail: a check when done,
    /// the action's symbol when waiting on one, nothing else, its room kept so the ends line up.
    #[must_use]
    pub fn mark(self) -> Self {
        Step { mark: true, ..self }
    }

    /// The row the keyboard is on, or the one picked. The current step is selected too.
    #[must_use]
    pub fn selected(self, selected: bool) -> Self {
        Step { selected, ..self }
    }

    /// `message` when there is one; a step without one takes no press.
    #[must_use]
    pub fn on_press_maybe(self, message: Option<M>) -> Self {
        Step {
            on_press: message,
            ..self
        }
    }

    pub fn view<'a>(self, look: &Look) -> Element<'a, M>
    where
        M: 'a,
    {
        let c = look.colors;
        let selected = self.selected || self.state == StepState::Current;
        let (fill, main, muted) = if selected {
            (
                Some(c.secondary_container),
                c.on_secondary_container,
                color::alpha(c.on_secondary_container, Opacity::SubduedText.value()),
            )
        } else {
            (None, c.on_surface, c.on_surface_variant)
        };
        let two_line = self.subtitle.is_some();
        let mut words = stack(look, Space::S4)
            .width(Extent::Fill)
            .push(line(self.title, Type::BodyStrong, main));
        if let Some(subtitle) = self.subtitle {
            words = words.push(text(subtitle, Type::Small, muted));
        }
        let mut content = row(look, Space::S12).center().width(Extent::Fill);
        if let Some(glyph) = self.glyph {
            // Beside a title over a subtitle, the icon spans both lines.
            let size = if two_line {
                tokens::TWO_LINE_ICON
            } else {
                Type::BodyStrong.icon()
            };
            content = content.push(icon(look, glyph, size, main));
        }
        content = content.push(words);
        if self.mark {
            content = content.push(mark(look, self.state, main));
        } else if let Some(chip) = state_chip(look, self.state) {
            content = content.push(chip);
        }
        // A row with one line is a list row's 44, its parts centred down it, so a chip moves
        // nothing; with a subtitle, 56 for its first line and taller by a line for each line more.
        let (content, height, inset) = if two_line {
            (content, Length::Shrink, Inset::xy(Space::S12, Space::S8))
        } else {
            (
                content.height(Extent::Fill),
                Length::Fixed(f32::from(Size::Row.px())),
                Inset::x(Space::S12),
            )
        };
        let enabled = self.on_press.is_some();
        let pressable = widget::button(content)
            .width(Length::Fill)
            .height(height)
            .padding(inset.padding())
            .on_press_maybe(self.on_press)
            .style(move |_theme: &Theme, _status| widget::button::Style {
                background: fill.map(Background::Color),
                text_color: main,
                border: Border::default().rounded(radius(CORNER)),
                shadow: Shadow::default(),
                snap: true,
            });
        let view = layered(pressable, CORNER, c.on_surface, enabled, None);
        let variant = [
            self.state.name(),
            if self.glyph.is_some() { "icon" } else { "" },
            if self.mark { "mark" } else { "" },
        ];
        probe::component("Step", variant, Fixed::NONE, view)
    }
}

/// The state as a chip: to do plain, done tonal with a check, an action the accent with its
/// symbol, the current step none.
fn state_chip<'a, M: Clone + 'a>(look: &Look, state: StepState) -> Option<Element<'a, M>> {
    match state {
        StepState::Todo => Some(chip(look, ChipKind::Neutral, "To do", None, None, None)),
        StepState::Done => Some(chip(look, ChipKind::Selected, "Done", Some(Icon::Check), None, None)),
        StepState::Action { label, glyph } => Some(chip(look, ChipKind::Accent, label, Some(glyph), None, None)),
        StepState::Current => None,
    }
}

/// The state as a mark the size of the title's icon: a check when done, the action's symbol in
/// the accent when waiting on one, else the room for one.
fn mark<'a, M: 'a>(look: &Look, state: StepState, main: iced::Color) -> Element<'a, M> {
    let size = Type::BodyStrong.icon();
    match state {
        StepState::Done => icon(look, Icon::Check, size, main),
        StepState::Action { glyph, .. } => icon(look, glyph, size, look.colors.primary),
        StepState::Todo | StepState::Current => widget::Space::new()
            .width(f32::from(size.px()))
            .height(f32::from(size.px()))
            .into(),
    }
}

#[cfg(test)]
mod tests;
