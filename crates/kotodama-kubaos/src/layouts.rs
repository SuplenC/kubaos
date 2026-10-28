//! The "Add a keyboard layout" app (`kotodama-kubaos layouts`, word `layout`): every layout and
//! variant xkb knows that the user doesn't type with yet, one Item each, by name. Its one Action
//! adds it through `kubaos keyboard add`.
//!
//! It is an app of its own, apart from the KubaOS Extension, so that a search over the layouts
//! alone is Kotodama's: the control center's "Add a language…" enters it with an embedded
//! `KotodamaSearch` and shows its Results, and the Command menu finds it as an app too. Its Items
//! stay in the app; the whole search would drown in a thousand layouts.
//!
//! # Owns
//!
//! - The app's Items and what their Action runs.
//!
//! # Not here
//!
//! - The layouts, the catalogue and the rules: `kubaos_core::keyboard`.
//! - Writing the change: `kubaos keyboard` (`crates/kubaos-cli`).
//! - The loop and the watch on the layouts: `main.rs`.

use kotodama_extension::Chain;
use kotodama_protocol::Action;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;
use kubaos_core::keyboard;
use kubaos_core::keyboard::Layout;

use crate::run::RunError;
use crate::run::Runner;
use crate::toggles::Ran;

/// The Item kind of every layout.
pub const KIND: &str = "kubaos-layouts.layout";
/// The one Action: add the layout.
pub const ADD: &str = "add";
/// A layout's Item is this and its spelling, `layout:us(intl)`.
const PREFIX: &str = "layout:";

const ICON: &str = "input-keyboard";

/// The layouts the user can add.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LayoutsApp {
    /// Those the user types with, left out.
    layouts: Vec<Layout>,
    /// Every layout and variant by name, sorted by name.
    catalogue: Vec<(Layout, String)>,
}

impl LayoutsApp {
    /// The catalogue in xkb's rules list text `lst`, without `layouts`.
    pub fn new(layouts: Vec<Layout>, lst: &str) -> Self {
        LayoutsApp {
            layouts,
            catalogue: keyboard::catalogue(lst),
        }
    }

    /// Takes the layouts the user types with now.
    pub fn set_layouts(&mut self, layouts: Vec<Layout>) {
        self.layouts = layouts;
    }

    /// One Item a layout the user doesn't type with, in the catalogue's order, the order the app
    /// lists them in before anything is typed.
    pub fn items(&self) -> Vec<Item> {
        self.catalogue
            .iter()
            .filter(|(layout, _)| !self.layouts.contains(layout))
            .map(|(layout, name)| item(layout, name))
            .collect()
    }

    /// Runs `action` on the layout Item `id`; the error is what the user reads.
    ///
    /// # Errors
    ///
    /// When there is no such Item or Action, or `kubaos keyboard add` fails: its last line.
    pub fn run(&self, id: &str, action: &str, runner: &impl Runner) -> Result<Ran, String> {
        let spelled = id
            .strip_prefix(PREFIX)
            .filter(|_| action == ADD)
            .ok_or_else(|| format!("`{id}` has no Action `{action}`"))?;
        let argv = ["kubaos", "keyboard", "add", spelled].map(str::to_owned);
        runner.output(&argv).map_err(|err| said(&err))?;
        Ok(Ran::Close)
    }
}

/// What a failed `kubaos keyboard` says, as one sentence: its last line, which is its own.
fn said(err: &RunError) -> String {
    let RunError::Failed { stderr, .. } = err else {
        return Chain(err).to_string();
    };
    let line = stderr
        .lines()
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .unwrap_or_default();
    let line = line.strip_prefix("kubaos: ").unwrap_or(line);
    let mut chars = line.chars();
    chars.next().map_or_else(
        || Chain(err).to_string(),
        |first| first.to_uppercase().chain(chars).collect(),
    )
}

fn item(layout: &Layout, name: &str) -> Item {
    let spelled = layout.to_string();
    Item {
        id: format!("{PREFIX}{spelled}"),
        kind: ItemKind::Namespaced(NamespacedKind::new(KIND).expect("the app's kind is valid")),
        identity: None,
        title: name.to_owned(),
        subtitle: Some(spelled.clone()),
        icon: Some(ICON.to_owned()),
        keyword: None,
        keywords: vec![spelled, layout.layout.clone()],
        hidden: false,
        app_only: true,
        weak_keywords: Vec::new(),
        actions: vec![Action::new(ADD, "Add")],
    }
}

#[cfg(test)]
mod tests;
