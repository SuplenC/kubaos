//! What the Extension offers and does, apart from the protocol loop: its
//! Items, from the registry and the toggles, and what running each Action
//! comes to.
//!
//! # Owns
//!
//! - Which Items there are under the current config, `hide` and the states applied, and what the
//!   update check changes in the update actions.
//! - Sending a run to the registry, a toggle, an audio device, a keyboard Item, a wallpaper set, a
//!   user folder, a desk position, a drive, an idle setting, a shell setting, the power stop's
//!   status, the CPU scheduler's or the Proxy's.
//!
//! # Not here
//!
//! - Reading the config: [`crate::registry`]. Reading the states: [`crate::states`]. The toggles:
//!   [`crate::toggles`]. The keyboard layouts: [`crate::keyboard`]. The wallpaper sets:
//!   [`crate::wallpapers`]. The user folders: [`crate::files`]. The desk positions:
//!   [`crate::desk`]. The drives: [`crate::drives`]. The idle settings: [`crate::idle`]. The
//!   shell's settings: [`crate::shell`]. The power stop's status and the CPU scheduler's:
//!   [`crate::power`]. The Proxy's status: [`crate::proxy`].
//! - The protocol loop and sending messages: `main.rs`.

use std::collections::BTreeMap;

use kotodama_extension::Chain;
use kotodama_protocol::Item;
use kotodama_protocol::ItemRef;
use kotodama_protocol::Table;
use kubaos_power::live::Now;
use kubaos_power::machine::Machine;
use kubaos_power::settings::Settings as PowerSettings;

use crate::bus::Bus;
use crate::desk;
use crate::desk::Desk;
use crate::drives;
use crate::drives::Drives;
use crate::files;
use crate::files::UserFolders;
use crate::idle;
use crate::keyboard;
use crate::keyboard::Keyboards;
use crate::power;
use crate::proxy;
use crate::registry;
use crate::registry::Registry;
use crate::run::Runner;
use crate::shell;
use crate::states::States;
use crate::toggles;
use crate::toggles::Ran;
use crate::toggles::Toggle;
use crate::updates::Pending;
use crate::wallpapers;
use crate::wallpapers::WallpaperSets;

/// The Extension's state: the registry its config holds, and what it runs
/// things with.
#[derive(Debug)]
pub struct Extension<B, R> {
    registry: Registry,
    pending: Pending,
    keyboards: Keyboards,
    wallpapers: WallpaperSets,
    folders: UserFolders,
    desk: Desk,
    drives: Drives,
    states: States,
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
            wallpapers: WallpaperSets::default(),
            folders: UserFolders::default(),
            desk: Desk::default(),
            drives: Drives::default(),
            states: States::default(),
            bus,
            runner,
        }
    }

    /// Takes the Extension's config table, as the Daemon sends it; what is
    /// wrong with it comes back, one problem a line.
    pub fn configure(&mut self, config: &Table) -> Vec<String> {
        let reserved: Vec<&str> = toggles::ids()
            .into_iter()
            .chain(keyboard::ids())
            .chain(idle::ids())
            .chain(shell::ids())
            .chain(power::ids())
            .chain(proxy::ids())
            .collect();
        let (registry, problems) = Registry::from_config(config, &reserved);
        self.registry = registry;
        problems
    }

    /// Takes what the update check found waiting.
    pub fn set_pending(&mut self, pending: Pending) {
        self.pending = pending;
    }

    /// Takes the keyboard layouts, as the user's settings or `/etc/default/keyboard` now have them.
    pub fn set_keyboards(&mut self, keyboards: Keyboards) {
        self.keyboards = keyboards;
    }

    /// Takes the wallpaper sets, as the sets file now shows them.
    pub fn set_wallpaper_sets(&mut self, wallpapers: WallpaperSets) {
        self.wallpapers = wallpapers;
    }

    /// Takes the user folders, as they are on disk now.
    pub fn set_user_folders(&mut self, folders: UserFolders) {
        self.folders = folders;
    }

    /// Takes the mounted drives, as the drives file now lists them.
    pub fn set_drives(&mut self, drives: Drives) {
        self.drives = drives;
    }

    /// Takes the desk positions, as Hyprland's screens file now lists them.
    pub fn set_desk(&mut self, desk: Desk) {
        self.desk = desk;
    }

    /// Takes the states the actions are shown in, as their files now have them.
    pub fn set_states(&mut self, states: States) {
        self.states = states;
    }

    /// The Items to index: the actions the states show, the toggles, the idle settings, the
    /// shell's, the power stop's, the CPU scheduler's and the Proxy's status, the keyboard
    /// Items, the wallpaper sets, the user folders, the desk positions and the drives `hide`
    /// leaves in, the update actions as the update check has them.
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
        let wallpapers = self
            .wallpapers
            .items()
            .into_iter()
            .filter(|item| !self.registry.hides(&item.id));
        let folders = self
            .folders
            .items()
            .into_iter()
            .filter(|item| !self.registry.hides(&item.id));
        let desk = self
            .desk
            .items()
            .into_iter()
            .filter(|item| !self.registry.hides(&item.id));
        let drives = self
            .drives
            .items()
            .into_iter()
            .filter(|item| !self.registry.hides(&item.id));
        let idle = idle::items().into_iter().filter(|item| !self.registry.hides(&item.id));
        let shell = shell::items().into_iter().filter(|item| !self.registry.hides(&item.id));
        let power = [power::item(), power::scheduler_item(), proxy::item()]
            .into_iter()
            .filter(|item| !self.registry.hides(&item.id));
        self.registry
            .items(&self.states)
            .filter_map(|item| self.pending.apply(item))
            .chain(toggles)
            .chain(idle)
            .chain(shell)
            .chain(power)
            .chain(keyboards)
            .chain(wallpapers)
            .chain(folders)
            .chain(desk)
            .chain(drives)
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
            wallpapers::KIND if !self.registry.hides(&item.id) => self.wallpapers.run(&item.id, action, &self.runner),
            files::KIND if !self.registry.hides(&item.id) => self.folders.run(&item.id, action, &self.runner),
            desk::KIND if !self.registry.hides(&item.id) => self.desk.run(&item.id, action, arguments, &self.runner),
            drives::KIND if !self.registry.hides(&item.id) => self.drives.run(&item.id, action, &self.runner),
            idle::KIND if action == idle::CHOOSE && !self.registry.hides(&item.id) => {
                idle::run(&item.id, arguments, &idle_settings(), &self.runner)
            }
            idle::CHOICE_KIND => idle::choose(&item.id, action, &self.runner),
            shell::KIND if !self.registry.hides(&item.id) => {
                let file = shell::path().ok_or("There's no home folder to keep the setting in")?;
                shell::run(&item.id, action, &file)
            }
            power::KIND if action == power::SHOW && !self.registry.hides(&item.id) => {
                Ok(Ran::Push(power::view(&power_now())))
            }
            power::SCHEDULER_KIND if action == power::SHOW && !self.registry.hides(&item.id) => {
                let (settings, _) = power_settings();
                Ok(Ran::Push(power::scheduler_view(&kubaos_power::live::scheduler(
                    &settings,
                    &Machine::real(),
                ))))
            }
            proxy::KIND if action == proxy::SHOW && !self.registry.hides(&item.id) => {
                Ok(Ran::Push(proxy::view(&proxy::now())))
            }
            _ => Err(unknown()),
        }
    }
}

/// The idle settings as the file has them now, which a choice list marks; read at each list, not
/// watched, as only the list shows them. A file that doesn't read marks KubaOS's defaults.
fn idle_settings() -> kubaos_idle::settings::Settings {
    kubaos_idle::settings::path()
        .and_then(|path| kubaos_idle::settings::Settings::load(&path).ok())
        .unwrap_or_default()
}

/// The power stop now, under the settings as the file has them; a file that doesn't read is
/// KubaOS's defaults, and the View's title says why.
fn power_now() -> Now {
    let (settings, problem) = power_settings();
    kubaos_power::live::read(&settings, problem.as_deref(), &Machine::real())
}

/// The power settings as the file has them, or KubaOS's defaults and why.
fn power_settings() -> (PowerSettings, Option<String>) {
    match kubaos_power::settings::path().map(|path| PowerSettings::load(&path)) {
        Some(Ok(settings)) => (settings, None),
        Some(Err(err)) => (PowerSettings::default(), Some(Chain(&err).to_string())),
        None => (PowerSettings::default(), None),
    }
}

#[cfg(test)]
mod tests;
