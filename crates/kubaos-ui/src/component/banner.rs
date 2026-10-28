//! Banners: a message that matters on the page itself, in one of three tones, with what to do
//! about it inside.
//!
//! A banner says something the user must read before going on: what an install is about to erase,
//! a step that needs a reboot, an update that failed. Its actions sit inside it, beside the words,
//! so a destructive one is read with the sentence that names what goes.
//!
//! # Owns
//!
//! - [`banner`], built up with [`Banner`], and its [`BannerTone`].
//!
//! # Not here
//!
//! - What broke and what to try next, with a log behind it: [`crate::component::error`]. A question
//!   that stops everything: [`crate::component::dialog`].

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Length;
use iced::widget;
use iced::widget::container;

use crate::color;
use crate::icon::Icon;
use crate::icon::icon;
use crate::layout::Extent;
use crate::layout::Inset;
use crate::layout::row;
use crate::layout::stack;
use crate::look::Look;
use crate::look::radius;
use crate::probe;
use crate::probe::Fixed;
use crate::text::text;
use crate::tokens::Opacity;
use crate::tokens::Radius;
use crate::tokens::Space;
use crate::tokens::Type;

/// How much a banner matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerTone {
    /// Worth knowing: where an install goes, what a step will do.
    Info,
    /// Worth acting on soon: a battery that may not last, a step left to do.
    Warning,
    /// About to destroy something: a disk about to be erased. Really red, so it can't be skimmed.
    Danger,
}

impl BannerTone {
    pub const ALL: [BannerTone; 3] = [BannerTone::Info, BannerTone::Warning, BannerTone::Danger];

    /// The tone's name in the spec.
    pub fn name(self) -> &'static str {
        match self {
            BannerTone::Info => "Info",
            BannerTone::Warning => "Warning",
            BannerTone::Danger => "Danger",
        }
    }
}

/// A tone's colours: the fill, the hairline round it, the symbol, the title and the body.
struct Colours {
    fill: Color,
    edge: Color,
    glyph: Color,
    title: Color,
    body: Color,
}

fn colours(look: &Look, tone: BannerTone) -> Colours {
    let c = look.colors;
    match tone {
        BannerTone::Info => Colours {
            fill: c.surface_container_high,
            edge: c.primary,
            glyph: c.primary,
            title: c.on_surface,
            body: c.on_surface_variant,
        },
        BannerTone::Warning => Colours {
            fill: c.surface_container_high,
            edge: c.warning,
            glyph: c.warning,
            title: c.on_surface,
            body: c.on_surface_variant,
        },
        BannerTone::Danger => Colours {
            fill: c.error_container,
            edge: c.error,
            glyph: c.error,
            title: c.on_error_container,
            body: color::alpha(c.on_error_container, Opacity::SubduedText.value()),
        },
    }
}

/// A banner, built up then drawn with [`Banner::view`].
pub struct Banner<'a, M> {
    tone: BannerTone,
    glyph: Icon,
    title: String,
    body: String,
    paragraphs: Vec<String>,
    actions: Vec<Element<'a, M>>,
    foot: Option<Element<'a, M>>,
}

impl<M> std::fmt::Debug for Banner<'_, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Banner")
            .field("tone", &self.tone)
            .field("title", &self.title)
            .field("actions", &self.actions.len())
            .finish_non_exhaustive()
    }
}

/// A banner in `tone`: `glyph` beside `title` and `body`, which wrap to the room there is.
pub fn banner<'a, M>(
    tone: BannerTone,
    glyph: Icon,
    title: impl Into<String>,
    body: impl Into<String>,
) -> Banner<'a, M> {
    Banner {
        tone,
        glyph,
        title: title.into(),
        body: body.into(),
        paragraphs: Vec::new(),
        actions: Vec::new(),
        foot: None,
    }
}

impl<'a, M: 'a> Banner<'a, M> {
    /// More of the body, a paragraph each, 4 apart: the steps of what to do.
    #[must_use]
    pub fn paragraphs(mut self, paragraphs: impl IntoIterator<Item = String>) -> Self {
        self.paragraphs.extend(paragraphs);
        self
    }

    /// What to do about it, beside the words at the banner's end, 12 apart: the action the banner
    /// is about last.
    #[must_use]
    pub fn actions(mut self, actions: impl IntoIterator<Item = Element<'a, M>>) -> Self {
        self.actions.extend(actions);
        self
    }

    /// A line across the banner's foot, 16 under the rest: how far a held button is.
    #[must_use]
    pub fn foot(self, foot: impl Into<Element<'a, M>>) -> Self {
        Banner {
            foot: Some(foot.into()),
            ..self
        }
    }

    pub fn view(self, look: &Look) -> Element<'a, M> {
        let colours = colours(look, self.tone);
        let words = stack(look, Space::S4)
            .width(Extent::Fill)
            .push(text(self.title, Type::Subheading, colours.title))
            .push(text(self.body, Type::Body, colours.body))
            .extend(
                self.paragraphs
                    .into_iter()
                    .map(|paragraph| text(paragraph, Type::Body, colours.body)),
            );
        let mut line = row(look, Space::S16)
            .center()
            .push(icon(look, self.glyph, Type::Subheading.icon(), colours.glyph))
            .push(words);
        if !self.actions.is_empty() {
            line = line.push(row(look, Space::S12).center().extend(self.actions));
        }
        let mut content = stack(look, Space::S16).push(line);
        if let Some(foot) = self.foot {
            content = content.push(foot);
        }
        let hairline = look.hairline();
        let (fill, edge) = (colours.fill, colours.edge);
        let view = widget::container(content)
            .padding(Inset::all(Space::S16).padding())
            .width(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(Background::Color(fill)),
                border: Border {
                    color: edge,
                    width: hairline,
                    radius: radius(Radius::Surface).into(),
                },
                ..container::Style::default()
            });
        probe::component("Banner", [self.tone.name(), "", ""], Fixed::NONE, view)
    }
}
