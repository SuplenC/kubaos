//! The keyboard layouts in the menu: one Item a layout to switch to it, make it the default, move
//! it up or down or remove it, and "Keyboard: add a layout". They are the layouts of `/etc/default/keyboard`, the
//! file the install wrote and `kubaos keyboard` changes.
//!
//! Switching is `hyprctl switchxkblayout`, at once. A change to the layouts is checked here, so
//! a refusal reads in the menu, then launched as `kubaos keyboard`, which asks localed (and so
//! polkit) and reloads Hyprland; the file's change comes back through the watch in `main.rs`.
//!
//! # Owns
//!
//! - Reading the layouts and their names, the keyboard Items, and what their Actions run.
//!
//! # Not here
//!
//! - The layouts, the file's fields and the rules: `kubaos_core::keyboard`.
//! - Writing the change: `kubaos keyboard` (`crates/kubaos-cli`).
//! - Switching to the next layout: the `SUPER + SHIFT + Space` bind (`config/hypr/keybinds.lua`),
//!   which the Bundled `hyprland` Extension puts in the search.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

use kotodama_extension::Chain;
use kotodama_protocol::Action;
use kotodama_protocol::Argument;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;
use kubaos_core::keyboard;
use kubaos_core::keyboard::Change;
use kubaos_core::keyboard::KeyboardFile;
use kubaos_core::keyboard::Layout;

use crate::run::Runner;
use crate::toggles::Ran;

/// The Item kind of every keyboard Item.
pub const KIND: &str = "kubaos.keyboard";
/// The Item that adds a layout.
pub const ADD: &str = "keyboard-add";
/// A layout's Item is this and its spelling, `keyboard:us(intl)`.
const LAYOUT_PREFIX: &str = "keyboard:";

/// The one Action of add, and a layout's primary Action: switch to it.
pub const RUN: &str = "run";
pub const MAKE_DEFAULT: &str = "default";
pub const MOVE_UP: &str = "move-up";
pub const MOVE_DOWN: &str = "move-down";
pub const REMOVE: &str = "remove";

/// The Argument of add.
const LAYOUT: &str = "layout";

const ICON: &str = "input-keyboard";

/// The layouts, the default first, with what they are called.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Keyboards {
    layouts: Vec<Layout>,
    /// Display names by spelling, from xkb's rules list.
    names: HashMap<String, String>,
    /// The layout names the machine has; empty when the rules list couldn't be read.
    known: Vec<String>,
}

impl Keyboards {
    /// Reads `file`, shaped as `/etc/default/keyboard`, and xkb's rules list `lst`. Either
    /// missing leaves no layouts, or no names.
    pub fn read(file: &Path, lst: &Path) -> Self {
        let read = |path: &Path| fs::read_to_string(path).unwrap_or_default();
        Self::parse(&read(file), &read(lst))
    }

    /// The layouts in `file`'s text, named by the rules list's text `lst`.
    pub fn parse(file: &str, lst: &str) -> Self {
        Keyboards {
            layouts: KeyboardFile::parse(file).layouts,
            names: keyboard::names(lst),
            known: kubaos_core::engine::machine::parse_xkb_layouts(lst),
        }
    }

    /// The Items: each layout, then add. None without a layout, as on a machine KubaOS didn't
    /// install, where `kubaos keyboard` has no file to change.
    pub fn items(&self) -> Vec<Item> {
        if self.layouts.is_empty() {
            return Vec::new();
        }
        let several = self.layouts.len() > 1;
        let last = self.layouts.len() - 1;
        let layouts = self.layouts.iter().enumerate().map(|(index, layout)| {
            let mut actions = vec![Action::new(RUN, "Type with it")];
            if index > 0 {
                actions.push(Action::new(MAKE_DEFAULT, "Make default"));
                actions.push(Action::new(MOVE_UP, "Move up"));
            }
            if index < last {
                actions.push(Action::new(MOVE_DOWN, "Move down"));
            }
            if several {
                actions.push(Action::new(REMOVE, "Remove"));
            }
            let subtitle = if index == 0 {
                "The default keyboard layout"
            } else {
                "A keyboard layout"
            };
            item(
                &format!("{LAYOUT_PREFIX}{layout}"),
                format!("Keyboard: {}", keyboard::name(layout, &self.names)),
                format!("{subtitle}, {layout}"),
                actions,
            )
        });
        let add = item(
            ADD,
            "Keyboard: add a layout".to_owned(),
            format!("Another layout to switch to, {} at most", keyboard::MAX_LAYOUTS),
            vec![Action::new(RUN, "Add").arguments(vec![Argument {
                name: LAYOUT.to_owned(),
                placeholder: Some("it, or us(intl)".to_owned()),
                optional: false,
            }])],
        );
        layouts.chain(std::iter::once(add)).collect()
    }

    /// Runs `action` on the keyboard Item `id`; the error is what the user reads.
    ///
    /// # Errors
    ///
    /// When there is no such Item or Action, the rules refuse the change, or a program fails.
    pub fn run(
        &self,
        id: &str,
        action: &str,
        arguments: &BTreeMap<String, String>,
        runner: &impl Runner,
    ) -> Result<Ran, String> {
        let unknown = || format!("`{id}` has no Action `{action}`");
        if (id, action) == (ADD, RUN) {
            let typed = arguments.get(LAYOUT).map_or("", |typed| typed.trim());
            return self.change(runner, Change::Add(Layout::parse(typed)), &["add", typed]);
        }
        let spelled = id.strip_prefix(LAYOUT_PREFIX).ok_or_else(unknown)?;
        let layout = Layout::parse(spelled);
        let index = self
            .layouts
            .iter()
            .position(|l| *l == layout)
            .ok_or_else(|| format!("`{spelled}` isn't one of the keyboard layouts"))?;
        match action {
            RUN => switch(runner, index),
            MAKE_DEFAULT => self.change(runner, Change::MakeDefault(layout), &["default", spelled]),
            MOVE_UP => {
                let to = index
                    .checked_sub(1)
                    .ok_or_else(|| format!("`{spelled}` is the first keyboard layout already"))?;
                self.move_to(runner, layout, to)
            }
            MOVE_DOWN => {
                let to = Some(index + 1)
                    .filter(|&to| to < self.layouts.len())
                    .ok_or_else(|| format!("`{spelled}` is the last keyboard layout already"))?;
                self.move_to(runner, layout, to)
            }
            REMOVE => self.change(runner, Change::Remove(layout), &["remove", spelled]),
            _ => Err(unknown()),
        }
    }

    /// Moves `layout` to the place at `to`, which `kubaos keyboard move` counts from 1.
    fn move_to(&self, runner: &impl Runner, layout: Layout, to: usize) -> Result<Ran, String> {
        let spelled = layout.to_string();
        let place = (to + 1).to_string();
        self.change(runner, Change::Move(layout, to), &["move", &spelled, &place])
    }

    /// Checks `change`, then leaves the writing to `kubaos keyboard <words>`.
    fn change(&self, runner: &impl Runner, change: Change, words: &[&str]) -> Result<Ran, String> {
        change
            .apply(&self.layouts, &self.known)
            .map_err(|err| capitalise(&err.to_string()))?;
        let argv: Vec<String> = ["kubaos", "keyboard"]
            .iter()
            .chain(words)
            .map(|&word| word.to_owned())
            .collect();
        runner
            .launch(&argv, Duration::ZERO)
            .map_err(|err| Chain(&err).to_string())?;
        Ok(Ran::Close)
    }
}

/// Every id of an Item this module always sends, which no action of the registry may take.
pub fn ids() -> [&'static str; 1] {
    [ADD]
}

/// Switches every keyboard to the layout at `index`.
fn switch(runner: &impl Runner, index: usize) -> Result<Ran, String> {
    let index = index.to_string();
    let argv = ["hyprctl", "switchxkblayout", "all", &index].map(str::to_owned);
    runner.output(&argv).map_err(|err| Chain(&err).to_string())?;
    Ok(Ran::Close)
}

fn item(id: &str, title: String, subtitle: String, actions: Vec<Action>) -> Item {
    Item {
        id: id.to_owned(),
        kind: ItemKind::Namespaced(NamespacedKind::new(KIND).expect("the Extension's kinds are valid")),
        identity: None,
        title,
        subtitle: Some(subtitle),
        icon: Some(ICON.to_owned()),
        keyword: None,
        keywords: ["keyboard layout", "keymap", "language", "input"]
            .map(str::to_owned)
            .to_vec(),
        hidden: false,
        app_only: false,
        actions,
    }
}

fn capitalise(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
