//! The states an action of the registry may be shown in, as its `shown_when` names them, read
//! from the files that hold them: each state's source of truth, never a command run for it.
//!
//! | State | Reads | From |
//! | --- | --- | --- |
//! | `packages.<group>` | on: the optional group is on | `/etc/kubaos/apt-groups`, else the manifest's default |
//! | `files.tidy-daily` | on: the daily Downloads tidy | `U/desktop/files/tidy-downloads` exists |
//! | `files.automount` | on: drives mount when plugged in | `U/desktop/files/no-automount` doesn't exist |
//! | `terminal.greeting` | on: the terminal greeting | `U/terminal/zsh/no-greeting` doesn't exist |
//! | `keyboard.mac-accents` | on: the accents on Left Alt | `U/desktop/keyboard/keyboard.toml` |
//! | `idle.keep-awake` | on: kept awake | `$XDG_RUNTIME_DIR/kubaos-keep-awake.json` |
//! | `phone.link` | on: the phone link | `U/desktop/phone/phone-link` exists |
//! | `proxy` | on: the user wants the Proxy | `$XDG_RUNTIME_DIR/kubaos-proxy.json` |
//! | `notifications.dnd` | on: Do not disturb is on | `$XDG_RUNTIME_DIR/kubaos-dnd.json` |
//! | `power.panel-power-saving` | on: panel power saving | `U/desktop/power/power.toml` |
//! | `power.scheduler` | `kernel` or `lavd` | the same |
//! | `power.low-battery` | `power-saver`, `super-saver` or `off` | the same |
//! | `nix.unfree` | on: the user's nixpkgs takes unfree packages | `U/home/kubaos.d/nixpkgs.toml` |
//!
//! `U` is the user layer, `~/.config/kubaos`. A state whose file can't be read has no reading,
//! and an action that names it shows whatever it declares.
//!
//! # Owns
//!
//! - The state names and what each reads as ([`values`]), reading them all ([`States::read`]), and
//!   the files they come from ([`Places::files`]).
//!
//! # Not here
//!
//! - Which action shows in which state: `shown_when` in [`crate::registry`].
//! - Watching the files: `main.rs`.
//! - The files' formats: the crates that write them.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use kubaos_core::apt::Choices;
use kubaos_core::apt::Manifest;
use kubaos_core::apt::Switch;
use kubaos_core::keyboard::settings as keyboard_settings;
use kubaos_core::state::DIR_NAME as STATE_DIR;
use kubaos_core::state::NIXPKGS_FILE;
use kubaos_core::state::user_flake;
use kubaos_core::unfree::Setting as UnfreeSetting;
use kubaos_core::unfree::Unfree;
use kubaos_core::update::record::SystemPaths;
use kubaos_user_layer::App;

/// Whether the daily Downloads tidy is on.
pub const TIDY_DAILY: &str = "files.tidy-daily";
/// Whether drives mount when plugged in.
pub const AUTOMOUNT: &str = "files.automount";
/// Whether a new Ghostty window greets with fastfetch.
pub const GREETING: &str = "terminal.greeting";
/// Whether the Mac-style accents are on Left Alt.
pub const MAC_ACCENTS: &str = "keyboard.mac-accents";
/// Whether the machine is kept awake.
pub const KEEP_AWAKE: &str = "idle.keep-awake";
/// Whether the phone link is on.
pub const PHONE_LINK: &str = "phone.link";
/// Whether the user wants the Proxy on in this session.
pub const PROXY: &str = "proxy";
/// Whether Do not disturb is on, by hand or by itself.
pub const DND: &str = "notifications.dnd";
/// Whether the saver stops dim the panel.
pub const PANEL_POWER_SAVING: &str = "power.panel-power-saving";
/// The CPU scheduler's name in the power settings.
pub const SCHEDULER: &str = "power.scheduler";
/// Where the low-battery drop goes, or `off`.
pub const LOW_BATTERY: &str = "power.low-battery";
/// Whether the user's nixpkgs takes unfree packages.
pub const UNFREE: &str = "nix.unfree";

/// What starts the state of an optional package group, `packages.chrome`.
const PACKAGES: &str = "packages.";

/// The CPU schedulers' names in the power settings, `kubaos_power::settings::CpuScheduler::ALL`'s.
const SCHEDULERS: &[&str] = &["kernel", "lavd"];
/// Where the low-battery drop may go, `kubaos_power::settings::Settings::LOW_BATTERY_STOPS`' ids,
/// then `off`.
const LOW_BATTERY_STOPS: &[&str] = &["power-saver", "super-saver", "off"];

/// What a state reads as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Values {
    /// True or false.
    Bool,
    /// One of these words.
    Words(&'static [&'static str]),
}

impl Values {
    /// Whether a state of these values may read as `reading`.
    pub fn allow(self, reading: &Reading) -> bool {
        match (self, reading) {
            (Values::Bool, Reading::Bool(_)) => true,
            (Values::Words(words), Reading::Word(word)) => words.contains(&word.as_str()),
            _ => false,
        }
    }
}

/// Do not disturb's state file in the runtime dir: `config/quickshell/services/Notifs.qml`
/// writes it.
const DND_FILE: &str = "kubaos-dnd.json";

/// The file that turns the greeting off, in the zsh folder of the user layer.
/// `scripts/dot-greeting` owns it.
const NO_GREETING: &str = "no-greeting";

/// A state, as its file has it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading {
    /// On or off.
    Bool(bool),
    /// One of a few named choices.
    Word(String),
}

/// The states that could be read, by name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct States {
    readings: BTreeMap<String, Reading>,
}

impl States {
    /// Every state the files at `places` hold. A few small files, read whole: cheap enough for
    /// each change to one of them, never for a keystroke.
    pub fn read(places: &Places) -> States {
        let mut states = States::default();
        states.read_packages(places);
        if let Some(config) = &places.config_home {
            states.read_user_layer(config);
        }
        if let Some(active) = places.keep_awake.as_deref().and_then(kept_awake) {
            states.set(KEEP_AWAKE, Reading::Bool(active));
        }
        if let Some(dnd) = places.dnd.as_deref().and_then(dnd_on) {
            states.set(DND, Reading::Bool(dnd));
        }
        if let Some(wanted) = places.proxy.as_deref().and_then(proxy_wanted) {
            states.set(PROXY, Reading::Bool(wanted));
        }
        states
    }

    /// The state `name`, `None` when it couldn't be read.
    pub fn get(&self, name: &str) -> Option<&Reading> {
        self.readings.get(name)
    }

    /// These states with `name` read as `reading`.
    #[must_use]
    pub fn with(mut self, name: impl Into<String>, reading: Reading) -> Self {
        self.readings.insert(name.into(), reading);
        self
    }

    fn set(&mut self, name: impl Into<String>, reading: Reading) {
        self.readings.insert(name.into(), reading);
    }

    /// Each optional group the manifest lists, on or off by the user's choice or its default, and
    /// each group the user chose that the manifest can't tell about.
    fn read_packages(&mut self, places: &Places) {
        let choices = match fs::read_to_string(&places.apt_choices) {
            Ok(text) => Choices::parse(&text),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Choices::default(),
            Err(_) => return,
        };
        for (group, switch) in choices.iter() {
            self.set(format!("{PACKAGES}{group}"), Reading::Bool(switch == Switch::On));
        }
        let Ok(Some(manifest)) = Manifest::read(&places.apt_manifest) else {
            return;
        };
        for group in manifest.groups().iter().filter(|group| group.optional.is_some()) {
            self.set(
                format!("{PACKAGES}{}", group.name),
                Reading::Bool(group.enabled(&choices)),
            );
        }
    }

    fn read_user_layer(&mut self, config: &Path) {
        let files = Files::new(config);
        self.set(TIDY_DAILY, Reading::Bool(files.tidy.daily()));
        self.set(AUTOMOUNT, Reading::Bool(files.automount.automount()));
        self.set(GREETING, Reading::Bool(!files.no_greeting.exists()));
        self.set(PHONE_LINK, Reading::Bool(files.phone.is_on()));
        if let Ok(keyboard) = keyboard_settings::Settings::load(&files.keyboard) {
            self.set(MAC_ACCENTS, Reading::Bool(keyboard.mac_accents));
        }
        if let Ok(power) = kubaos_power::settings::Settings::load(&files.power) {
            self.set(PANEL_POWER_SAVING, Reading::Bool(power.panel_power_saving));
            self.set(SCHEDULER, Reading::Word(power.scheduler.name().to_owned()));
            let low_battery = power.low_battery_stop.map_or("off", |stop| stop.id());
            self.set(LOW_BATTERY, Reading::Word(low_battery.to_owned()));
        }
        if let Ok(unfree) = UnfreeSetting::read(&files.state_dir) {
            self.set(UNFREE, Reading::Bool(unfree == Unfree::Allowed));
        }
    }
}

/// What the state `name` reads as; `None` for a name that is no state.
pub fn values(name: &str) -> Option<Values> {
    match name {
        SCHEDULER => Some(Values::Words(SCHEDULERS)),
        LOW_BATTERY => Some(Values::Words(LOW_BATTERY_STOPS)),
        TIDY_DAILY | AUTOMOUNT | GREETING | MAC_ACCENTS | KEEP_AWAKE | PHONE_LINK | PROXY | DND
        | PANEL_POWER_SAVING | UNFREE => Some(Values::Bool),
        _ if name.strip_prefix(PACKAGES).is_some_and(|group| !group.is_empty()) => Some(Values::Bool),
        _ => None,
    }
}

/// Where the states are read from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Places {
    /// `/etc/kubaos/apt-groups`, the user's choices of optional package groups.
    pub apt_choices: PathBuf,
    /// The installed release's package manifest, with each optional group's default.
    pub apt_manifest: PathBuf,
    /// `XDG_CONFIG_HOME`, else `~/.config`: the user layer's settings.
    pub config_home: Option<PathBuf>,
    /// Keep awake's state file in the runtime dir.
    pub keep_awake: Option<PathBuf>,
    /// The Proxy's state file in the runtime dir.
    pub proxy: Option<PathBuf>,
    /// Do not disturb's state file in the runtime dir, which the shell writes.
    pub dnd: Option<PathBuf>,
}

impl Places {
    /// The places of this machine and session.
    pub fn from_env() -> Places {
        let machine = SystemPaths::machine();
        Places {
            apt_choices: machine.apt_choices(),
            apt_manifest: machine.manifest(),
            config_home: kubaos_user_layer::config_home(),
            keep_awake: kubaos_idle::runtime_dir().map(|runtime| kubaos_idle::awake::path(&runtime)),
            proxy: kubaos_proxy::paths::Paths::from_env().map(|paths| paths.shell()),
            dnd: kubaos_idle::runtime_dir().map(|runtime| runtime.join(DND_FILE)),
        }
    }

    /// Every file a state is read from, so a change to any of them reads the states again.
    pub fn files(&self) -> Vec<PathBuf> {
        let mut files = vec![self.apt_choices.clone(), self.apt_manifest.clone()];
        if let Some(config) = &self.config_home {
            let user_layer = Files::new(config);
            files.extend([
                user_layer.tidy.path().to_path_buf(),
                user_layer.automount.path().to_path_buf(),
                user_layer.no_greeting,
                user_layer.phone.path().to_path_buf(),
                user_layer.keyboard,
                user_layer.power,
                user_layer.state_dir.join(NIXPKGS_FILE),
            ]);
        }
        files.extend(self.keep_awake.iter().chain(&self.proxy).chain(&self.dnd).cloned());
        files
    }
}

/// The user layer's files the states come from.
struct Files {
    tidy: kubaos_files::tidy::Setting,
    automount: kubaos_automount::settings::Settings,
    no_greeting: PathBuf,
    phone: kubaos_phone::setting::Setting,
    keyboard: PathBuf,
    power: PathBuf,
    /// The user flake's state directory, `U/home/kubaos.d`.
    state_dir: PathBuf,
}

impl Files {
    fn new(config: &Path) -> Files {
        Files {
            tidy: kubaos_files::tidy::Setting::new(config),
            automount: kubaos_automount::settings::Settings::new(config),
            no_greeting: kubaos_user_layer::dir(config, App::Zsh).join(NO_GREETING),
            phone: kubaos_phone::setting::Setting::new(config),
            keyboard: keyboard_settings::path(config),
            power: kubaos_user_layer::dir(config, App::Power).join(kubaos_power::settings::FILE),
            state_dir: user_flake(config).join(STATE_DIR),
        }
    }
}

/// Whether keep awake's state file says it holds: not without the file, as `kubaos-idle` writes it
/// only once kept awake; `None` when it doesn't read.
fn kept_awake(file: &Path) -> Option<bool> {
    match fs::read_to_string(file) {
        Ok(text) => serde_json::from_str::<kubaos_idle::awake::State>(&text)
            .ok()
            .map(|state| state.active),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Some(false),
        Err(_) => None,
    }
}

/// Whether Do not disturb's state file says it is on; `None` when it doesn't read, as while the
/// shell isn't running, which writes it at its start.
fn dnd_on(file: &Path) -> Option<bool> {
    let text = fs::read_to_string(file).ok()?;
    let state: serde_json::Value = serde_json::from_str(&text).ok()?;
    state.get("on")?.as_bool()
}

/// Whether the Proxy's state file says the user wants it on; `None` when it doesn't read.
fn proxy_wanted(file: &Path) -> Option<bool> {
    let text = fs::read_to_string(file).ok()?;
    let status: serde_json::Value = serde_json::from_str(&text).ok()?;
    status.get("wanted")?.as_bool()
}

#[cfg(test)]
mod tests;
