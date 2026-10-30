//! The specimen's state and its view: a bar with the pages, then the page.
//!
//! # Owns
//!
//! - [`Specimen`], what is shown ([`Page`], mode, scale, the motion clock), and [`view`].
//!
//! # Not here
//!
//! - Each page's content: [`crate::pages`]. The window: [`crate::app`]. Screenshots:
//!   [`crate::shot`].

use iced::Background;
use iced::Border;
use iced::Element;
use iced::Length;
use iced::widget;
use kubaos_ui::Look;
use kubaos_ui::Mode;
use kubaos_ui::Scale;
use kubaos_ui::color;
use kubaos_ui::component::button::ButtonSize;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::button::icon_button;
use kubaos_ui::component::chip::ChipKind;
use kubaos_ui::component::chip::chip;
use kubaos_ui::component::keycap::key_hints;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::container;
use kubaos_ui::layout::divider;
use kubaos_ui::layout::fill;
use kubaos_ui::layout::page;
use kubaos_ui::layout::row;
use kubaos_ui::layout::stack;
use kubaos_ui::look::radius;
use kubaos_ui::template::sidebar_page;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Opacity;
use kubaos_ui::tokens::Radius;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use crate::pages;
use crate::pages::templates;
use crate::pages::templates::Demo;

/// A page of the specimen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Type,
    Colour,
    Space,
    Components,
    Icons,
    Motion,
    Templates,
}

impl Page {
    pub const ALL: [Page; 7] = [
        Page::Type,
        Page::Colour,
        Page::Space,
        Page::Components,
        Page::Icons,
        Page::Motion,
        Page::Templates,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Page::Type => "Type",
            Page::Colour => "Colour",
            Page::Space => "Space and shape",
            Page::Components => "Components",
            Page::Icons => "Icons",
            Page::Motion => "Motion",
            Page::Templates => "Templates",
        }
    }

    /// The page's name in file names.
    pub fn slug(self) -> &'static str {
        match self {
            Page::Type => "type",
            Page::Colour => "colour",
            Page::Space => "space",
            Page::Components => "components",
            Page::Icons => "icons",
            Page::Motion => "motion",
            Page::Templates => "templates",
        }
    }

    /// The page whose [`Page::slug`] is `slug`, if there is one.
    pub fn from_slug(slug: &str) -> Option<Page> {
        Page::ALL.into_iter().find(|page| page.slug() == slug)
    }

    /// The page after this one, wrapping round; `back` goes the other way.
    pub fn step(self, back: bool) -> Page {
        let at = Page::ALL.iter().position(|&p| p == self).unwrap_or(0);
        let n = Page::ALL.len();
        Page::ALL[if back { (at + n - 1) % n } else { (at + 1) % n }]
    }
}

/// What the specimen shows.
#[derive(Debug, Clone, Copy)]
pub struct Specimen {
    pub page: Page,
    pub mode: Mode,
    /// The scale the page is judged at: the window's in the live specimen, the shot's otherwise.
    pub scale: Scale,
    /// Seconds on the motion samples' clock.
    pub seconds: f32,
    /// In a window: the page scrolls. In a screenshot it is laid out whole.
    pub live: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Page(Page),
    Mode(Mode),
    /// The window's display scale changed.
    Scaled(f32),
    Frame(std::time::Instant),
    Key(iced::keyboard::Event),
    /// A press on a sample: the components look enabled, and nothing happens.
    Sample,
    /// The live template's own.
    Template(sidebar_page::Message),
}

/// The id of the specimen's whole content, which a screenshot measures.
pub const ROOT: &str = "specimen-root";

/// The specimen; on the templates page in the window, `demo` live in all the room under the bar,
/// so resizing the window resizes it.
pub fn view<'a>(specimen: &'a Specimen, demo: Option<&'a Demo>) -> Element<'a, Message> {
    let look = Look::new(specimen.mode, specimen.scale);
    let body: Element<'a, Message> =
        if let Some(demo) = demo.filter(|_| specimen.live && specimen.page == Page::Templates) {
            templates::live(demo, &look)
        } else if specimen.live {
            let content = pages::view(specimen, &look);
            widget::scrollable(page(&look, content))
                .height(Length::Fill)
                .style(move |_theme, status| scroller(&look, status))
                .into()
        } else {
            page(&look, pages::view(specimen, &look))
        };
    let whole = stack(&look, Gap::NONE)
        .push(bar(specimen, &look))
        .push(divider(&look))
        .push(body);
    widget::container(whole)
        .id(ROOT)
        .width(Length::Fill)
        .height(if specimen.live { Length::Fill } else { Length::Shrink })
        .style(move |_theme| widget::container::Style::default().background(look.colors.surface))
        .into()
}

/// The bar above the page: the pages, then the mode and the scale it is shown at.
fn bar<'a>(specimen: &Specimen, look: &Look) -> Element<'a, Message> {
    let c = look.colors;
    let mut tabs = row(look, Space::S4).center();
    for page in Page::ALL {
        let kind = if page == specimen.page { Kind::Tonal } else { Kind::Text };
        tabs = tabs.push(
            button(page.title())
                .kind(kind)
                .size(ButtonSize::Sm)
                .on_press(Message::Page(page))
                .view(look),
        );
    }
    // The bar is the same size in both modes: switching modes changes colours, never geometry.
    let scale = format!("{}×", specimen.scale.factor());
    let (other, glyph, label) = match specimen.mode {
        Mode::Dark => (Mode::Light, Icon::LightMode, "Light mode"),
        Mode::Light => (Mode::Dark, Icon::DarkMode, "Dark mode"),
    };
    // A toolbar band: the container padding at the sides, as the page under it, and a small
    // control's room above and below.
    container(
        row(look, Space::S16)
            .center()
            .padding(Inset::y(Space::S16))
            .width(Extent::Fill)
            .push(text("KubaOS design specimen", Type::Subheading, c.on_surface))
            .push(tabs)
            .push(fill())
            .push(key_hints(look, hints(specimen)))
            .push(chip(look, ChipKind::Neutral, scale, None, None, None::<Message>))
            .push(
                icon_button(glyph, label)
                    .kind(Kind::Text)
                    .size(ButtonSize::Sm)
                    .on_press(Message::Mode(other))
                    .view(look),
            ),
    )
}

/// The keys the bar names: the specimen's own, or, on the templates page in the window, where its
/// keys are the template's, the template's.
fn hints(specimen: &Specimen) -> &'static [(&'static str, &'static str)] {
    if specimen.live && specimen.page == Page::Templates {
        &[("Ctrl+Tab", "next page"), ("/", "find"), ("Ctrl+PgDn", "next section")]
    } else {
        &[("Tab", "next page"), ("← →", "pages"), ("M", "mode")]
    }
}

/// Kōbō's scroller: no rail, a thin pill in the muted text colour, stronger under the pointer.
fn scroller(look: &Look, status: widget::scrollable::Status) -> widget::scrollable::Style {
    let c = look.colors;
    let hot = matches!(
        status,
        widget::scrollable::Status::Hovered { .. } | widget::scrollable::Status::Dragged { .. }
    );
    let strength = if hot {
        Opacity::Placeholder.value()
    } else {
        Opacity::DisabledContent.value()
    };
    let rail = widget::scrollable::Rail {
        background: None,
        border: Border::default(),
        scroller: widget::scrollable::Scroller {
            background: Background::Color(color::alpha(c.on_surface_variant, strength)),
            border: Border::default().rounded(radius(Radius::Full)),
        },
    };
    widget::scrollable::Style {
        container: widget::container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
        auto_scroll: widget::scrollable::AutoScroll {
            background: Background::Color(c.surface_container_high),
            border: Border::default().rounded(radius(Radius::Full)),
            shadow: iced::Shadow::default(),
            icon: c.on_surface,
        },
    }
}

#[cfg(test)]
mod tests;
