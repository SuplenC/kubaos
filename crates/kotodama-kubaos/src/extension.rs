//! What the Extension offers and does, apart from the protocol loop: its
//! Items, from the registry and the toggles, and what running each Action
//! comes to.
//!
//! # Owns
//!
//! - Which Items there are under the current config, `hide` applied, and what the update check
//!   changes in the update actions.
//! - Sending a run to the registry, a toggle, an audio device or a keyboard Item.
//!
//! # Not here
//!
//! - Reading the config: [`crate::registry`]. The toggles: [`crate::toggles`]. The keyboard
//!   layouts: [`crate::keyboard`].
//! - The protocol loop and sending messages: `main.rs`.

use std::collections::BTreeMap;

use kotodama_extension::Chain;
use kotodama_protocol::Item;
use kotodama_protocol::ItemRef;
use kotodama_protocol::Table;

use crate::bus::Bus;
use crate::keyboard;
use crate::keyboard::Keyboards;
use crate::registry;
use crate::registry::Registry;
use crate::run::Runner;
use crate::toggles;
use crate::toggles::Ran;
use crate::toggles::Toggle;
use crate::updates::Pending;

/// The Extension's state: the registry its config holds, and what it runs
/// things with.
#[derive(Debug)]
pub struct Extension<B, R> {
    registry: Registry,
    pending: Pending,
    keyboards: Keyboards,
    bus: B,
    runner: R,
}

impl<B: Bus, R: Runner> Extension<B, R> {
    /// An Extension with no config yet: the toggles alone.
    pub fn new(bus: B, runner: R) -> Self {
        Extension {
            registry: Registry::default(),
            pending: Pending::default(),
            keyboards: Keyboards::default(),
            bus,
            runner,
        }
    }

    /// Takes the Extension's config table, as the Daemon sends it; what is
    /// wrong with it comes back, one problem a line.
    pub fn configure(&mut self, config: &Table) -> Vec<String> {
        let reserved: Vec<&str> = toggles::ids().into_iter().chain(keyboard::ids()).collect();
        let (registry, problems) = Registry::from_config(config, &reserved);
        self.registry = registry;
        problems
    }

    /// Takes what the update check found waiting.
    pub fn set_pending(&mut self, pending: Pending) {
        self.pending = pending;
    }

    /// Takes the keyboard layouts, as `/etc/default/keyboard` now has them.
    pub fn set_keyboards(&mut self, keyboards: Keyboards) {
        self.keyboards = keyboards;
    }

    /// The Items to index: the actions, the toggles and the keyboard Items `hide` leaves in, the
    /// update actions as the update check has them.
    pub fn items(&self) -> impl Iterator<Item = Item> + '_ {
        let toggles = Toggle::ALL
            .into_iter()
            .filter(|toggle| !self.registry.hides(toggle.id()))
            .map(Toggle::item);
        let keyboards = self
            .keyboards
            .items()
            .into_iter()
            .filter(|item| !self.registry.hides(&item.id));
        self.registry
            .items()
            .filter_map(|item| self.pending.apply(item))
            .chain(toggles)
            .chain(keyboards)
    }

    /// Runs `action` on `item`; an error is what the user reads.
    ///
    /// # Errors
    ///
    /// When there is no such Item or Action, or running it failed.
    pub fn run(&mut self, action: &str, item: &ItemRef, arguments: &BTreeMap<String, String>) -> Result<Ran, String> {
        let unknown = || format!("`{}` has no Action `{action}`", item.id);
        match item.kind.name() {
            registry::KIND if action == registry::RUN => {
                let command = self.registry.command(&item.id, arguments).ok_or_else(unknown)?;
                self.runner
                    .launch(&command.argv, command.delay)
                    .map_err(|err| Chain(&err).to_string())?;
                Ok(Ran::Close)
            }
            toggles::KIND => {
                let toggle = Toggle::from_id(&item.id)
                    .filter(|toggle| !self.registry.hides(toggle.id()))
                    .ok_or_else(unknown)?;
                toggle.run(action, arguments, &mut self.bus, &self.runner)
            }
            toggles::DEVICE_KIND if action == toggles::USE => toggles::use_device(&item.id, &self.runner),
            keyboard::KIND if !self.registry.hides(&item.id) => {
                self.keyboards.run(&item.id, action, arguments, &self.runner)
            }
            _ => Err(unknown()),
        }
    }
}

#[cfg(test)]
mod tests;
