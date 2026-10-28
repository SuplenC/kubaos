//! Description lists: facts beside their labels, as a summary reads them.
//!
//! # Owns
//!
//! - [`description_list`] and [`Fact`], one label and its value.
//!
//! # Not here
//!
//! - Rows that are picked or pressed: [`crate::component::list`].

use iced::Element;

use crate::layout::split;
use crate::layout::stack;
use crate::look::Look;
use crate::probe;
use crate::probe::Fixed;
use crate::text::line;
use crate::text::text;
use crate::tokens::Space;
use crate::tokens::Type;

/// One fact: its label, its value, and an optional quieter line under the value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact<'a> {
    label: &'a str,
    value: String,
    sub: Option<String>,
}

impl<'a> Fact<'a> {
    #[must_use]
    pub fn new(label: &'a str, value: impl Into<String>) -> Fact<'a> {
        Fact {
            label,
            value: value.into(),
            sub: None,
        }
    }

    /// A second line under the value, in mono, one line: a path, an email, a version, which
    /// break badly wrapped mid-token, so they get a line of their own.
    #[must_use]
    pub fn detail(self, detail: impl Into<String>) -> Fact<'a> {
        Fact {
            sub: Some(detail.into()),
            ..self
        }
    }
}

/// `facts` one under the other, 8 apart: each label in `small` a quarter of the width, its value in
/// `body` the rest, wrapping.
pub fn description_list<'a, M: 'a>(look: &Look, facts: impl IntoIterator<Item = Fact<'a>>) -> Element<'a, M> {
    let c = look.colors;
    let rows = facts.into_iter().map(|fact| {
        let mut value = stack(look, Space::S4).push(text(fact.value, Type::Body, c.on_surface));
        if let Some(sub) = fact.sub {
            value = value.push(line(sub, Type::Mono, c.on_surface_variant));
        }
        split(look, &[1, 3], Space::S8)
            .push(text(fact.label, Type::Small, c.on_surface_variant))
            .push(value)
            .into()
    });
    probe::component(
        "DescriptionList",
        ["", "", ""],
        Fixed::NONE,
        stack(look, Space::S8).extend(rows),
    )
}
